//! Session Manager - SQLite-based session tracking
//!
//! Manages CLI session metadata in SQLite for:
//! - Session discovery (find last session)
//! - Log correlation (session_id → log file path)
//! - Session history (track all sessions)
//! - Reproducibility (state + logs = observability)
//!
//! **Note**: Full logs are stored in files (~/.rustation/logs/), not in the DB.
//! The DB only stores metadata for fast queries.

use crate::domain::paths::data_dir;
use crate::{Result, RscliError};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

/// Session metadata stored in SQLite
#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub session_id: String,
    pub command_type: String,
    pub created_at: i64,
    pub status: String,
    pub log_file: Option<String>,
}

/// SQLite-based session manager
pub struct SessionManager {
    db: Connection,
}

impl SessionManager {
    /// Open or create the sessions database
    pub fn open() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RscliError::Other(anyhow::anyhow!(
                    "Failed to create sessions directory: {}",
                    e
                ))
            })?;
        }

        let db = Connection::open(&db_path).map_err(|e| {
            RscliError::Other(anyhow::anyhow!("Failed to open sessions database: {}", e))
        })?;

        // Initialize schema
        db.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                command_type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                status TEXT NOT NULL,
                log_file TEXT
            )",
            [],
        )
        .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to create schema: {}", e)))?;

        // Create index for fast queries
        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_created_at
             ON sessions(created_at DESC)",
            [],
        )
        .map_err(|e| {
            RscliError::Other(anyhow::anyhow!("Failed to create index: {}", e))
        })?;

        Ok(Self { db })
    }

    /// Get the database path: ~/.rstn/sessions.db
    fn get_db_path() -> Result<PathBuf> {
        let dir = data_dir().map_err(|e| {
            RscliError::Other(anyhow::anyhow!("Failed to get data directory: {}", e))
        })?;
        Ok(dir.join("sessions.db"))
    }

    /// Save a session record
    pub fn save_session(&self, session: &SessionRecord) -> Result<()> {
        self.db
            .execute(
                "INSERT OR REPLACE INTO sessions
                 (session_id, command_type, created_at, status, log_file)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    &session.session_id,
                    &session.command_type,
                    session.created_at,
                    &session.status,
                    &session.log_file,
                ],
            )
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to save session: {}", e)))?;
        Ok(())
    }

    /// Get the most recent session for a given command type
    pub fn get_last_session(&self, command_type: &str) -> Result<Option<String>> {
        let mut stmt = self
            .db
            .prepare(
                "SELECT session_id FROM sessions
                 WHERE command_type = ?1
                 ORDER BY created_at DESC
                 LIMIT 1",
            )
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to prepare query: {}", e)))?;

        let result = stmt
            .query_row(params![command_type], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to query session: {}", e)))?;

        Ok(result)
    }

    /// Get a session record by ID
    pub fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>> {
        let mut stmt = self
            .db
            .prepare(
                "SELECT session_id, command_type, created_at, status, log_file
                 FROM sessions
                 WHERE session_id = ?1",
            )
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to prepare query: {}", e)))?;

        let result = stmt
            .query_row(params![session_id], |row| {
                Ok(SessionRecord {
                    session_id: row.get(0)?,
                    command_type: row.get(1)?,
                    created_at: row.get(2)?,
                    status: row.get(3)?,
                    log_file: row.get(4)?,
                })
            })
            .optional()
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to query session: {}", e)))?;

        Ok(result)
    }

    /// List recent sessions (most recent first)
    pub fn list_recent_sessions(&self, limit: usize) -> Result<Vec<SessionRecord>> {
        let mut stmt = self
            .db
            .prepare(
                "SELECT session_id, command_type, created_at, status, log_file
                 FROM sessions
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to prepare query: {}", e)))?;

        let sessions = stmt
            .query_map(params![limit as i64], |row| {
                Ok(SessionRecord {
                    session_id: row.get(0)?,
                    command_type: row.get(1)?,
                    created_at: row.get(2)?,
                    status: row.get(3)?,
                    log_file: row.get(4)?,
                })
            })
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to query sessions: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to collect sessions: {}", e)))?;

        Ok(sessions)
    }

    /// Update session status (e.g., "active" → "completed")
    pub fn update_session_status(&self, session_id: &str, status: &str) -> Result<()> {
        self.db
            .execute(
                "UPDATE sessions SET status = ?1 WHERE session_id = ?2",
                params![status, session_id],
            )
            .map_err(|e| {
                RscliError::Other(anyhow::anyhow!("Failed to update session status: {}", e))
            })?;
        Ok(())
    }

    /// Delete a session from the database
    ///
    /// **Note**: This only deletes the metadata. Log files must be deleted separately.
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        self.db
            .execute("DELETE FROM sessions WHERE session_id = ?1", params![session_id])
            .map_err(|e| {
                RscliError::Other(anyhow::anyhow!("Failed to delete session: {}", e))
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_basic_operations() {
        // Use in-memory database for testing
        let manager = SessionManager {
            db: Connection::open_in_memory().unwrap(),
        };

        // Create schema
        manager
            .db
            .execute(
                "CREATE TABLE sessions (
                    session_id TEXT PRIMARY KEY,
                    command_type TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    status TEXT NOT NULL,
                    log_file TEXT
                )",
                [],
            )
            .unwrap();

        // Save a session
        let session = SessionRecord {
            session_id: "test-session-123".to_string(),
            command_type: "prompt".to_string(),
            created_at: 1234567890,
            status: "completed".to_string(),
            log_file: Some("/path/to/log.log".to_string()),
        };

        manager.save_session(&session).unwrap();

        // Retrieve it
        let retrieved = manager.get_session("test-session-123").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.session_id, "test-session-123");
        assert_eq!(retrieved.command_type, "prompt");
        assert_eq!(retrieved.status, "completed");

        // Get last session
        let last = manager.get_last_session("prompt").unwrap();
        assert_eq!(last, Some("test-session-123".to_string()));

        // Update status
        manager
            .update_session_status("test-session-123", "failed")
            .unwrap();

        let updated = manager.get_session("test-session-123").unwrap().unwrap();
        assert_eq!(updated.status, "failed");
    }

    #[test]
    fn test_list_recent_sessions() {
        let manager = SessionManager {
            db: Connection::open_in_memory().unwrap(),
        };

        manager
            .db
            .execute(
                "CREATE TABLE sessions (
                    session_id TEXT PRIMARY KEY,
                    command_type TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    status TEXT NOT NULL,
                    log_file TEXT
                )",
                [],
            )
            .unwrap();

        // Save multiple sessions
        for i in 0..5 {
            let session = SessionRecord {
                session_id: format!("session-{}", i),
                command_type: "prompt".to_string(),
                created_at: 1000 + i,
                status: "completed".to_string(),
                log_file: None,
            };
            manager.save_session(&session).unwrap();
        }

        // List recent (should be in reverse order)
        let recent = manager.list_recent_sessions(3).unwrap();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].session_id, "session-4");
        assert_eq!(recent[1].session_id, "session-3");
        assert_eq!(recent[2].session_id, "session-2");
    }
}
