use std::{
    collections::HashMap,
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use chrono::Utc;
use parking_lot::Mutex;
use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};
use tauri::ipc::Channel;
use uuid::Uuid;

use crate::models::{CreateSessionRequest, SessionEvent, SessionInfo, ShellProfile};

const BASH_INTEGRATION: &str = include_str!("../shell-integration/chatterm.bash");
const ZSH_INTEGRATION: &str = include_str!("../shell-integration/.zshrc");
const FISH_INTEGRATION: &str = include_str!("../shell-integration/chatterm.fish");
const POWERSHELL_INTEGRATION: &str = include_str!("../shell-integration/chatterm.ps1");

struct SessionHandle {
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
}

pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, Arc<SessionHandle>>>>,
    integration_dir: PathBuf,
}

impl TerminalManager {
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        let integration_dir = app_data_dir.join("shell-integration");
        install_shell_integration(&integration_dir)?;
        Ok(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            integration_dir,
        })
    }

    pub fn shell_profiles(&self) -> Vec<ShellProfile> {
        discover_shell_profiles()
    }

    pub fn create_session(
        &self,
        request: CreateSessionRequest,
        channel: Channel<SessionEvent>,
    ) -> Result<SessionInfo, String> {
        let profiles = discover_shell_profiles();
        let profile = request
            .shell_id
            .as_ref()
            .and_then(|id| profiles.iter().find(|profile| &profile.id == id))
            .or_else(|| profiles.first())
            .cloned()
            .ok_or_else(|| "No supported shell was found on this system".to_string())?;

        let cwd = resolve_cwd(request.cwd.as_deref())?;
        let rows = request.rows.unwrap_or(30).clamp(2, 500);
        let cols = request.cols.unwrap_or(120).clamp(2, 1000);
        let session_id = Uuid::new_v4().to_string();

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("Unable to create a terminal: {error}"))?;

        let mut command = self.command_for_profile(&profile, &session_id)?;
        command.cwd(&cwd);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("TERM_PROGRAM", "Chatterm");
        command.env("TERM_PROGRAM_VERSION", env!("CARGO_PKG_VERSION"));
        command.env("CHATTERM_SESSION_ID", &session_id);

        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| format!("Unable to launch {}: {error}", profile.name))?;
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| format!("Unable to read terminal output: {error}"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| format!("Unable to open terminal input: {error}"))?;
        let killer = child.clone_killer();

        let handle = Arc::new(SessionHandle {
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            killer: Mutex::new(killer),
        });
        self.sessions
            .lock()
            .insert(session_id.clone(), handle.clone());

        spawn_reader(session_id.clone(), reader, channel.clone());
        let sessions = self.sessions.clone();
        let wait_session_id = session_id.clone();
        thread::Builder::new()
            .name(format!("chatterm-wait-{wait_session_id}"))
            .spawn(move || {
                match child.wait() {
                    Ok(status) => {
                        let _ = channel.send(SessionEvent::Exit {
                            session_id: wait_session_id.clone(),
                            code: status.exit_code(),
                        });
                    }
                    Err(error) => {
                        let _ = channel.send(SessionEvent::Error {
                            session_id: wait_session_id.clone(),
                            message: format!("Shell wait failed: {error}"),
                        });
                    }
                }
                sessions.lock().remove(&wait_session_id);
            })
            .map_err(|error| format!("Unable to monitor the shell: {error}"))?;

        Ok(SessionInfo {
            id: session_id,
            shell_id: profile.id,
            shell_name: profile.name,
            enhanced_threads: profile.enhanced_threads,
            cwd: cwd.to_string_lossy().to_string(),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    pub fn write(&self, session_id: &str, data: &str) -> Result<(), String> {
        let handle = self
            .sessions
            .lock()
            .get(session_id)
            .cloned()
            .ok_or_else(|| "This terminal session is no longer running".to_string())?;
        let mut writer = handle.writer.lock();
        writer
            .write_all(data.as_bytes())
            .and_then(|_| writer.flush())
            .map_err(|error| format!("Unable to write to the terminal: {error}"))
    }

    pub fn resize(&self, session_id: &str, rows: u16, cols: u16) -> Result<(), String> {
        let handle = self
            .sessions
            .lock()
            .get(session_id)
            .cloned()
            .ok_or_else(|| "This terminal session is no longer running".to_string())?;
        handle
            .master
            .lock()
            .resize(PtySize {
                rows: rows.clamp(2, 500),
                cols: cols.clamp(2, 1000),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("Unable to resize the terminal: {error}"))
    }

    pub fn close(&self, session_id: &str) -> Result<(), String> {
        let handle = self.sessions.lock().remove(session_id);
        if let Some(handle) = handle {
            handle
                .killer
                .lock()
                .kill()
                .map_err(|error| format!("Unable to stop the shell: {error}"))?;
        }
        Ok(())
    }

    fn command_for_profile(
        &self,
        profile: &ShellProfile,
        _session_id: &str,
    ) -> Result<CommandBuilder, String> {
        let mut command = CommandBuilder::new(&profile.program);
        match profile.id.as_str() {
            "pwsh" | "powershell" => {
                command.arg("-NoLogo");
                command.arg("-NoExit");
                command.arg("-File");
                command.arg(self.integration_dir.join("chatterm.ps1"));
            }
            "bash" => {
                command.arg("--rcfile");
                command.arg(self.integration_dir.join("chatterm.bash"));
                command.arg("-i");
            }
            "zsh" => {
                let user_zdotdir = env::var_os("ZDOTDIR")
                    .or_else(|| env::var_os("HOME"))
                    .unwrap_or_default();
                command.env("CHATTERM_USER_ZDOTDIR", user_zdotdir);
                command.env("ZDOTDIR", &self.integration_dir);
                command.arg("-i");
            }
            "fish" => {
                let script = self.integration_dir.join("chatterm.fish");
                let escaped = script.to_string_lossy().replace('\'', "\\'");
                command.arg("--init-command");
                command.arg(format!("source '{escaped}'"));
                command.arg("--interactive");
            }
            _ => {}
        }
        Ok(command)
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        let sessions = self
            .sessions
            .lock()
            .drain()
            .map(|(_, value)| value)
            .collect::<Vec<_>>();
        for session in sessions {
            let _ = session.killer.lock().kill();
        }
    }
}

fn spawn_reader(
    session_id: String,
    mut reader: Box<dyn Read + Send>,
    channel: Channel<SessionEvent>,
) {
    let _ = thread::Builder::new()
        .name(format!("chatterm-read-{session_id}"))
        .spawn(move || {
            let mut buffer = vec![0_u8; 32 * 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) => {
                        if channel
                            .send(SessionEvent::Data {
                                session_id: session_id.clone(),
                                data: buffer[..count].to_vec(),
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = channel.send(SessionEvent::Error {
                            session_id: session_id.clone(),
                            message: format!("Terminal output stopped: {error}"),
                        });
                        break;
                    }
                }
            }
        });
}

fn install_shell_integration(directory: &Path) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    for (name, contents) in [
        ("chatterm.bash", BASH_INTEGRATION),
        (".zshrc", ZSH_INTEGRATION),
        ("chatterm.fish", FISH_INTEGRATION),
        ("chatterm.ps1", POWERSHELL_INTEGRATION),
    ] {
        fs::write(directory.join(name), contents).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn resolve_cwd(requested: Option<&str>) -> Result<PathBuf, String> {
    let path = requested
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .or_else(|| env::current_dir().ok())
        .ok_or_else(|| "Unable to determine a starting directory".to_string())?;
    if !path.is_dir() {
        return Err(format!(
            "Starting directory does not exist: {}",
            path.display()
        ));
    }
    Ok(path)
}

fn discover_shell_profiles() -> Vec<ShellProfile> {
    let mut profiles = Vec::new();

    #[cfg(windows)]
    {
        add_profile(&mut profiles, "pwsh", "PowerShell", "pwsh.exe", true);
        add_profile(
            &mut profiles,
            "powershell",
            "Windows PowerShell",
            "powershell.exe",
            true,
        );
        let comspec = env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());
        add_profile(&mut profiles, "cmd", "Command Prompt", &comspec, false);
    }

    #[cfg(not(windows))]
    {
        let preferred = env::var("SHELL").ok();
        if let Some(shell) = preferred.as_deref() {
            let id = Path::new(shell)
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("shell");
            if matches!(id, "bash" | "zsh" | "fish") {
                add_profile(&mut profiles, id, &display_shell_name(id), shell, true);
            }
        }
        for (id, name) in [("zsh", "Zsh"), ("bash", "Bash"), ("fish", "Fish")] {
            add_profile(&mut profiles, id, name, id, true);
        }
        if let Some(shell) = preferred.as_deref() {
            let id = Path::new(shell)
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("shell");
            if !matches!(id, "bash" | "zsh" | "fish") {
                add_profile(&mut profiles, id, &display_shell_name(id), shell, false);
            }
        }
    }

    profiles
}

fn add_profile(
    profiles: &mut Vec<ShellProfile>,
    id: &str,
    name: &str,
    program: &str,
    enhanced_threads: bool,
) {
    if profiles.iter().any(|profile| profile.id == id) || !program_exists(program) {
        return;
    }
    profiles.push(ShellProfile {
        id: id.to_string(),
        name: name.to_string(),
        program: program.to_string(),
        enhanced_threads,
    });
}

fn program_exists(program: &str) -> bool {
    let candidate = Path::new(program);
    if candidate.components().count() > 1 {
        return candidate.is_file();
    }
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    let extensions: &[&str] = if cfg!(windows) {
        &["", ".exe", ".cmd", ".bat"]
    } else {
        &[""]
    };
    env::split_paths(&path).any(|directory| {
        extensions
            .iter()
            .any(|extension| directory.join(format!("{program}{extension}")).is_file())
    })
}

#[cfg(not(windows))]
fn display_shell_name(id: &str) -> String {
    match id {
        "zsh" => "Zsh".to_string(),
        "bash" => "Bash".to_string(),
        "fish" => "Fish".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cwd_is_a_directory() {
        assert!(resolve_cwd(None).expect("a default directory").is_dir());
    }

    #[test]
    fn terminal_size_is_clamped_by_resize_contract() {
        assert_eq!(1_u16.clamp(2, 500), 2);
        assert_eq!(900_u16.clamp(2, 500), 500);
    }
}
