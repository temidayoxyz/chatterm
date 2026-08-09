use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellProfile {
    pub id: String,
    pub name: String,
    pub program: String,
    pub enhanced_threads: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    pub shell_id: Option<String>,
    pub cwd: Option<String>,
    pub rows: Option<u16>,
    pub cols: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: String,
    pub shell_id: String,
    pub shell_name: String,
    pub enhanced_threads: bool,
    pub cwd: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum SessionEvent {
    Data { session_id: String, data: Vec<u8> },
    Exit { session_id: String, code: u32 },
    Error { session_id: String, message: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedBlock {
    pub id: String,
    pub command: String,
    pub output: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSessionRequest {
    pub id: String,
    pub name: String,
    pub shell_name: String,
    pub cwd: String,
    pub created_at: String,
    pub blocks: Vec<SavedBlock>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSessionSummary {
    pub id: String,
    pub name: String,
    pub shell_name: String,
    pub cwd: String,
    pub command_count: u32,
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSessionDetail {
    pub id: String,
    pub name: String,
    pub shell_name: String,
    pub cwd: String,
    pub created_at: String,
    pub saved_at: String,
    pub blocks: Vec<SavedBlock>,
}
