use std::{fs, path::Path};

use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{Connection, params};

use crate::models::{SaveSessionRequest, SavedBlock, SavedSessionDetail, SavedSessionSummary};

pub struct HistoryStore {
    connection: Mutex<Connection>,
}

impl HistoryStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let connection = Connection::open(path).map_err(|error| error.to_string())?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 PRAGMA journal_mode = WAL;
                 CREATE TABLE IF NOT EXISTS saved_sessions (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    shell_name TEXT NOT NULL,
                    cwd TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    saved_at TEXT NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS saved_blocks (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL REFERENCES saved_sessions(id) ON DELETE CASCADE,
                    position INTEGER NOT NULL,
                    command TEXT NOT NULL,
                    output TEXT NOT NULL,
                    status TEXT NOT NULL,
                    exit_code INTEGER,
                    started_at TEXT NOT NULL,
                    ended_at TEXT
                 );
                 CREATE INDEX IF NOT EXISTS idx_saved_blocks_session
                    ON saved_blocks(session_id, position);",
            )
            .map_err(|error| error.to_string())?;

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn save(&self, request: SaveSessionRequest) -> Result<SavedSessionSummary, String> {
        let saved_at = Utc::now().to_rfc3339();
        let mut connection = self.connection.lock();
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;

        transaction
            .execute(
                "INSERT INTO saved_sessions (id, name, shell_name, cwd, created_at, saved_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    shell_name = excluded.shell_name,
                    cwd = excluded.cwd,
                    created_at = excluded.created_at,
                    saved_at = excluded.saved_at",
                params![
                    request.id,
                    request.name,
                    request.shell_name,
                    request.cwd,
                    request.created_at,
                    saved_at
                ],
            )
            .map_err(|error| error.to_string())?;

        transaction
            .execute(
                "DELETE FROM saved_blocks WHERE session_id = ?1",
                params![request.id],
            )
            .map_err(|error| error.to_string())?;

        for (position, block) in request.blocks.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO saved_blocks
                     (id, session_id, position, command, output, status, exit_code, started_at, ended_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        block.id,
                        request.id,
                        position as i64,
                        block.command,
                        block.output,
                        block.status,
                        block.exit_code,
                        block.started_at,
                        block.ended_at
                    ],
                )
                .map_err(|error| error.to_string())?;
        }

        transaction.commit().map_err(|error| error.to_string())?;

        Ok(SavedSessionSummary {
            id: request.id,
            name: request.name,
            shell_name: request.shell_name,
            cwd: request.cwd,
            command_count: request.blocks.len() as u32,
            saved_at,
        })
    }

    pub fn list(&self) -> Result<Vec<SavedSessionSummary>, String> {
        let connection = self.connection.lock();
        let mut statement = connection
            .prepare(
                "SELECT s.id, s.name, s.shell_name, s.cwd, COUNT(b.id), s.saved_at
                 FROM saved_sessions s
                 LEFT JOIN saved_blocks b ON b.session_id = s.id
                 GROUP BY s.id
                 ORDER BY s.saved_at DESC",
            )
            .map_err(|error| error.to_string())?;

        let rows = statement
            .query_map([], |row| {
                Ok(SavedSessionSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    shell_name: row.get(2)?,
                    cwd: row.get(3)?,
                    command_count: row.get::<_, i64>(4)? as u32,
                    saved_at: row.get(5)?,
                })
            })
            .map_err(|error| error.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn load(&self, id: &str) -> Result<SavedSessionDetail, String> {
        let connection = self.connection.lock();
        let mut session = connection
            .query_row(
                "SELECT id, name, shell_name, cwd, created_at, saved_at
                 FROM saved_sessions WHERE id = ?1",
                params![id],
                |row| {
                    Ok(SavedSessionDetail {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        shell_name: row.get(2)?,
                        cwd: row.get(3)?,
                        created_at: row.get(4)?,
                        saved_at: row.get(5)?,
                        blocks: Vec::new(),
                    })
                },
            )
            .map_err(|error| error.to_string())?;

        let mut statement = connection
            .prepare(
                "SELECT id, command, output, status, exit_code, started_at, ended_at
                 FROM saved_blocks WHERE session_id = ?1 ORDER BY position ASC",
            )
            .map_err(|error| error.to_string())?;
        let blocks = statement
            .query_map(params![id], |row| {
                Ok(SavedBlock {
                    id: row.get(0)?,
                    command: row.get(1)?,
                    output: row.get(2)?,
                    status: row.get(3)?,
                    exit_code: row.get(4)?,
                    started_at: row.get(5)?,
                    ended_at: row.get(6)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        session.blocks = blocks;
        Ok(session)
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        self.connection
            .lock()
            .execute("DELETE FROM saved_sessions WHERE id = ?1", params![id])
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn saved_sessions_round_trip_and_cascade() {
        let directory = std::env::temp_dir().join(format!("chatterm-history-{}", Uuid::new_v4()));
        let path = directory.join("history.sqlite3");
        let store = HistoryStore::open(&path).expect("history store should open");
        let request = SaveSessionRequest {
            id: "session-1".to_string(),
            name: "Release prep".to_string(),
            shell_name: "PowerShell".to_string(),
            cwd: "C:\\project".to_string(),
            created_at: "2026-08-09T10:00:00Z".to_string(),
            blocks: vec![SavedBlock {
                id: "block-1".to_string(),
                command: "npm test".to_string(),
                output: "5 passed".to_string(),
                status: "success".to_string(),
                exit_code: Some(0),
                started_at: "2026-08-09T10:00:01Z".to_string(),
                ended_at: Some("2026-08-09T10:00:02Z".to_string()),
            }],
        };

        let summary = store.save(request).expect("session should save");
        assert_eq!(summary.command_count, 1);
        assert_eq!(store.list().expect("sessions should list").len(), 1);
        let loaded = store.load("session-1").expect("session should load");
        assert_eq!(loaded.blocks[0].command, "npm test");

        store.delete("session-1").expect("session should delete");
        assert!(store.list().expect("sessions should list").is_empty());
        drop(store);
        fs::remove_dir_all(directory).expect("test directory should clean up");
    }
}
