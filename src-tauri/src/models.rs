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
    // The enum-level rename covers variant names only; each variant needs its
    // own attribute so inner fields reach the frontend as camelCase.
    #[serde(rename_all = "camelCase")]
    Data { session_id: String, data: Vec<u8> },
    #[serde(rename_all = "camelCase")]
    Exit { session_id: String, code: u32 },
    #[serde(rename_all = "camelCase")]
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

#[cfg(test)]
mod tests {
    use super::*;

    // The frontend reads event.sessionId/event.data; a regression to
    // snake_case fields would silently drop every session event.
    #[test]
    fn session_events_serialize_camel_case_fields() {
        let data = serde_json::to_string(&SessionEvent::Data {
            session_id: "s1".into(),
            data: vec![27, 91, 54, 110],
        })
        .expect("serialize data event");
        assert!(data.contains(r#""event":"data""#), "{data}");
        assert!(data.contains(r#""sessionId":"s1""#), "{data}");

        let exit = serde_json::to_string(&SessionEvent::Exit {
            session_id: "s1".into(),
            code: 0,
        })
        .expect("serialize exit event");
        assert!(exit.contains(r#""event":"exit""#), "{exit}");
        assert!(exit.contains(r#""sessionId":"s1""#), "{exit}");

        let error = serde_json::to_string(&SessionEvent::Error {
            session_id: "s1".into(),
            message: "boom".into(),
        })
        .expect("serialize error event");
        assert!(error.contains(r#""event":"error""#), "{error}");
        assert!(error.contains(r#""sessionId":"s1""#), "{error}");
    }
}
