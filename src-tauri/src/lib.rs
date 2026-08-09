mod history;
mod models;
mod terminal;

use history::HistoryStore;
use models::{
    CreateSessionRequest, SaveSessionRequest, SavedSessionDetail, SavedSessionSummary,
    SessionEvent, SessionInfo, ShellProfile,
};
use tauri::{Manager, State, ipc::Channel};
use terminal::TerminalManager;

#[tauri::command]
fn list_shell_profiles(manager: State<'_, TerminalManager>) -> Vec<ShellProfile> {
    manager.shell_profiles()
}

#[tauri::command]
fn create_session(
    request: CreateSessionRequest,
    on_event: Channel<SessionEvent>,
    manager: State<'_, TerminalManager>,
) -> Result<SessionInfo, String> {
    manager.create_session(request, on_event)
}

#[tauri::command]
fn write_to_session(
    session_id: String,
    data: String,
    manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    manager.write(&session_id, &data)
}

#[tauri::command]
fn resize_session(
    session_id: String,
    rows: u16,
    cols: u16,
    manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    manager.resize(&session_id, rows, cols)
}

#[tauri::command]
fn close_session(session_id: String, manager: State<'_, TerminalManager>) -> Result<(), String> {
    manager.close(&session_id)
}

#[tauri::command]
fn save_session(
    request: SaveSessionRequest,
    history: State<'_, HistoryStore>,
) -> Result<SavedSessionSummary, String> {
    history.save(request)
}

#[tauri::command]
fn list_saved_sessions(
    history: State<'_, HistoryStore>,
) -> Result<Vec<SavedSessionSummary>, String> {
    history.list()
}

#[tauri::command]
fn load_saved_session(
    id: String,
    history: State<'_, HistoryStore>,
) -> Result<SavedSessionDetail, String> {
    history.load(&id)
}

#[tauri::command]
fn delete_saved_session(id: String, history: State<'_, HistoryStore>) -> Result<(), String> {
    history.delete(&id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let history = HistoryStore::open(&app_data_dir.join("chatterm.sqlite3"))
                .map_err(std::io::Error::other)?;
            let terminals = TerminalManager::new(&app_data_dir).map_err(std::io::Error::other)?;
            app.manage(history);
            app.manage(terminals);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_shell_profiles,
            create_session,
            write_to_session,
            resize_session,
            close_session,
            save_session,
            list_saved_sessions,
            load_saved_session,
            delete_saved_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Chatterm");
}
