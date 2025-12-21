//! Session Manager - Dual-layer session tracking
//!
//! Manages two layers of sessions:
//! 1. **Claude Sessions** - Individual LLM interactions (UUID from Claude Code)
//! 2. **rstn Sessions** - Workflow-level sessions (Prompt, Specify, Plan, Tasks)
//!
//! Also manages legacy CLI session metadata in SQLite for:
//! - Session discovery (find last session)
//! - Log correlation (session_id → log file path)
//! - Session history (track all sessions)
//! - Reproducibility (state + logs = observability)
//!
//! **Note**: Full logs are stored in files (~/.rustation/logs/), not in the DB.
//! The DB only stores metadata for fast queries.

use crate::domain::paths::{data_dir, rstn_home};
use crate::{Result, RscliError};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, warn};

// ============================================================================
// Dual-Layer Session Management
// ============================================================================

/// Workflow type for rstn sessions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowType {
    Prompt,
    Specify,
    Plan,
    Tasks,
    Implement,
}

impl std::fmt::Display for WorkflowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowType::Prompt => write!(f, "Prompt"),
            WorkflowType::Specify => write!(f, "Specify"),
            WorkflowType::Plan => write!(f, "Plan"),
            WorkflowType::Tasks => write!(f, "Tasks"),
            WorkflowType::Implement => write!(f, "Implement"),
        }
    }
}

/// Claude session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaudeSessionStatus {
    Running,
    Completed,
    MaxTurns,
    Error { message: String },
}

/// Claude session (from Claude Code) - individual LLM interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSession {
    /// UUID from Claude Code (session_id in stream-json)
    pub uuid: String,

    /// What this Claude session is for
    pub purpose: String,

    /// Session metadata
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Session outcome
    pub status: ClaudeSessionStatus,
    pub turns_used: usize,
    pub max_turns: usize,
    pub total_cost_usd: Option<f64>,

    /// Link to parent rstn session
    pub rstn_session_id: String,
}

/// rstn session (workflow level) - tracks entire workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RstnSession {
    /// rstn-generated ID (e.g., "rstn-sess-20251221-001")
    pub id: String,

    /// Workflow type
    pub workflow: WorkflowType,

    /// Feature being worked on
    pub feature_number: Option<String>,
    pub feature_name: Option<String>,

    /// Session metadata
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,

    /// All Claude sessions in this workflow
    pub claude_sessions: HashMap<String, ClaudeSession>,

    /// Current active Claude session
    pub active_claude_session: Option<String>,

    /// Total cost across all Claude sessions
    pub total_cost_usd: f64,
}

impl RstnSession {
    /// Create a new rstn session
    pub fn new(workflow: WorkflowType) -> Self {
        let now = chrono::Utc::now();
        let id = format!(
            "rstn-sess-{}",
            now.format("%Y%m%d-%H%M%S")
        );

        Self {
            id,
            workflow,
            feature_number: None,
            feature_name: None,
            started_at: now,
            completed_at: None,
            claude_sessions: HashMap::new(),
            active_claude_session: None,
            total_cost_usd: 0.0,
        }
    }
}

// ============================================================================
// Legacy Session Management
// ============================================================================

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
        debug!("Opening sessions database at: {:?}", db_path);

        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create sessions directory {:?}: {}", parent, e);
                RscliError::Other(anyhow::anyhow!(
                    "Failed to create sessions directory: {}",
                    e
                ))
            })?;
        }

        let db = Connection::open(&db_path).map_err(|e| {
            error!("Failed to open sessions database at {:?}: {}", db_path, e);
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
        .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to create index: {}", e)))?;

        debug!("Sessions database opened successfully");

        let manager = Self { db };

        // Initialize dual-layer session tables
        manager.init_dual_layer_tables()?;
        debug!("Dual-layer session tables initialized");

        Ok(manager)
    }

    /// Get the database path: ~/.rstn/sessions.db
    fn get_db_path() -> Result<PathBuf> {
        let new_dir = rstn_home().map_err(|e| {
            RscliError::Other(anyhow::anyhow!("Failed to get rstn home directory: {}", e))
        })?;
        let new_path = new_dir.join("sessions.db");

        // Migration: copy old XDG database if it exists and new doesn't
        if !new_path.exists() {
            if let Ok(old_dir) = data_dir() {
                let old_path = old_dir.join("sessions.db");
                if old_path.exists() {
                    // Copy old database to new location (preserve original)
                    let _ = std::fs::copy(&old_path, &new_path);
                }
            }
        }

        Ok(new_path)
    }

    /// Save a session record
    pub fn save_session(&self, session: &SessionRecord) -> Result<()> {
        debug!("Saving session: {}", session.session_id);
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
            .map_err(|e| {
                warn!("Failed to save session {}: {}", session.session_id, e);
                RscliError::Other(anyhow::anyhow!("Failed to save session: {}", e))
            })?;
        debug!("Session saved: {}", session.session_id);
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
        debug!("Listing recent sessions (limit: {})", limit);
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

        debug!("Loaded {} sessions", sessions.len());
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
            .execute(
                "DELETE FROM sessions WHERE session_id = ?1",
                params![session_id],
            )
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to delete session: {}", e)))?;
        Ok(())
    }

    // ========================================================================
    // Dual-Layer Session Management Methods
    // ========================================================================

    /// Initialize dual-layer tables in SQLite
    pub fn init_dual_layer_tables(&self) -> Result<()> {
        // rstn_sessions table
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS rstn_sessions (
                id TEXT PRIMARY KEY,
                workflow TEXT NOT NULL,
                feature_number TEXT,
                feature_name TEXT,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                total_cost_usd REAL NOT NULL DEFAULT 0.0
            )",
            [],
        ).map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to create rstn_sessions table: {}", e)))?;

        // claude_sessions table
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS claude_sessions (
                uuid TEXT PRIMARY KEY,
                rstn_session_id TEXT NOT NULL,
                purpose TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                status TEXT NOT NULL,
                turns_used INTEGER NOT NULL,
                max_turns INTEGER NOT NULL,
                total_cost_usd REAL,
                FOREIGN KEY (rstn_session_id) REFERENCES rstn_sessions(id)
            )",
            [],
        ).map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to create claude_sessions table: {}", e)))?;

        Ok(())
    }

    /// Save an rstn session
    pub fn save_rstn_session(&self, session: &RstnSession) -> Result<()> {
        // Create backup
        let _sessions_json = serde_json::to_string(&session.claude_sessions)
            .map_err(|e| anyhow::anyhow!("Failed to serialize sessions: {}", e))?;
        // fs::write(
        //     self.config.state_dir.join("sessions.json.bak"),
        //     sessions_json,
        // )?;

        self.db.execute(
            "INSERT OR REPLACE INTO rstn_sessions
             (id, workflow, feature_number, feature_name, started_at, completed_at, total_cost_usd)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &session.id,
                &session.workflow.to_string(),
                &session.feature_number,
                &session.feature_name,
                &session.started_at.to_rfc3339(),
                &session.completed_at.as_ref().map(|dt| dt.to_rfc3339()),
                session.total_cost_usd,
            ],
        ).map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to save rstn session: {}", e)))?;

        // Save all Claude sessions
        for claude_session in session.claude_sessions.values() {
            self.save_claude_session(claude_session)?;
        }

        Ok(())
    }

    /// Save a Claude session
    pub fn save_claude_session(&self, session: &ClaudeSession) -> Result<()> {
        let status_str = match &session.status {
            ClaudeSessionStatus::Running => "running".to_string(),
            ClaudeSessionStatus::Completed => "completed".to_string(),
            ClaudeSessionStatus::MaxTurns => "max_turns".to_string(),
            ClaudeSessionStatus::Error { message } => format!("error:{}", message),
        };

        self.db.execute(
            "INSERT OR REPLACE INTO claude_sessions
             (uuid, rstn_session_id, purpose, started_at, completed_at, status, turns_used, max_turns, total_cost_usd)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &session.uuid,
                &session.rstn_session_id,
                &session.purpose,
                &session.started_at.to_rfc3339(),
                &session.completed_at.as_ref().map(|dt| dt.to_rfc3339()),
                &status_str,
                session.turns_used as i64,
                session.max_turns as i64,
                session.total_cost_usd,
            ],
        ).map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to save claude session: {}", e)))?;

        Ok(())
    }

    /// Load an rstn session by ID
    pub fn load_rstn_session(&self, session_id: &str) -> Result<Option<RstnSession>> {
        let mut stmt = self.db.prepare(
            "SELECT id, workflow, feature_number, feature_name, started_at, completed_at, total_cost_usd
             FROM rstn_sessions
             WHERE id = ?1"
        ).map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to prepare query: {}", e)))?;

        let session = stmt.query_row(params![session_id], |row| {
            let workflow_str: String = row.get(1)?;
            let workflow = match workflow_str.as_str() {
                "Prompt" => WorkflowType::Prompt,
                "Specify" => WorkflowType::Specify,
                "Plan" => WorkflowType::Plan,
                "Tasks" => WorkflowType::Tasks,
                "Implement" => WorkflowType::Implement,
                _ => WorkflowType::Prompt,
            };

            let started_at_str: String = row.get(4)?;
            let completed_at_str: Option<String> = row.get(5)?;

            Ok(RstnSession {
                id: row.get(0)?,
                workflow,
                feature_number: row.get(2)?,
                feature_name: row.get(3)?,
                started_at: chrono::DateTime::parse_from_rfc3339(&started_at_str)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                completed_at: completed_at_str.map(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&chrono::Utc)
                }),
                claude_sessions: HashMap::new(), // Load separately
                active_claude_session: None,
                total_cost_usd: row.get(6)?,
            })
        }).optional()
        .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to load rstn session: {}", e)))?;

        // Load Claude sessions if rstn session exists
        if let Some(mut rstn_session) = session {
            let claude_sessions = self.load_claude_sessions_for_rstn(&rstn_session.id)?;
            for claude_session in claude_sessions {
                rstn_session.claude_sessions.insert(claude_session.uuid.clone(), claude_session);
            }
            Ok(Some(rstn_session))
        } else {
            Ok(None)
        }
    }

    /// Load all Claude sessions for an rstn session
    fn load_claude_sessions_for_rstn(&self, rstn_session_id: &str) -> Result<Vec<ClaudeSession>> {
        let mut stmt = self.db.prepare(
            "SELECT uuid, rstn_session_id, purpose, started_at, completed_at, status, turns_used, max_turns, total_cost_usd
             FROM claude_sessions
             WHERE rstn_session_id = ?1
             ORDER BY started_at ASC"
        ).map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to prepare query: {}", e)))?;

        let sessions = stmt.query_map(params![rstn_session_id], |row| {
            let status_str: String = row.get(5)?;
            let status = if status_str == "running" {
                ClaudeSessionStatus::Running
            } else if status_str == "completed" {
                ClaudeSessionStatus::Completed
            } else if status_str == "max_turns" {
                ClaudeSessionStatus::MaxTurns
            } else if let Some(msg) = status_str.strip_prefix("error:") {
                ClaudeSessionStatus::Error {
                    message: msg.to_string(),
                }
            } else {
                ClaudeSessionStatus::Running
            };

            let started_at_str: String = row.get(3)?;
            let completed_at_str: Option<String> = row.get(4)?;

            Ok(ClaudeSession {
                uuid: row.get(0)?,
                rstn_session_id: row.get(1)?,
                purpose: row.get(2)?,
                started_at: chrono::DateTime::parse_from_rfc3339(&started_at_str)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                completed_at: completed_at_str.map(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&chrono::Utc)
                }),
                status,
                turns_used: row.get::<_, i64>(6)? as usize,
                max_turns: row.get::<_, i64>(7)? as usize,
                total_cost_usd: row.get(8)?,
            })
        })
        .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to load claude sessions: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to collect claude sessions: {}", e)))?;

        Ok(sessions)
    }

    /// List recent rstn sessions
    pub fn list_recent_rstn_sessions(&self, limit: usize) -> Result<Vec<RstnSession>> {
        let mut stmt = self.db.prepare(
            "SELECT id FROM rstn_sessions
             ORDER BY started_at DESC
             LIMIT ?1"
        ).map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to prepare query: {}", e)))?;

        let session_ids: Vec<String> = stmt
            .query_map(params![limit as i64], |row| row.get(0))
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to query sessions: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to collect session IDs: {}", e)))?;

        let mut sessions = Vec::new();
        for id in session_ids {
            if let Some(session) = self.load_rstn_session(&id)? {
                sessions.push(session);
            }
        }

        Ok(sessions)
    }

    // ========================================================================
    // Convenience Methods for Session Lifecycle
    // ========================================================================

    /// Start a new rstn workflow session
    pub fn start_rstn_session(&self, workflow: WorkflowType) -> Result<RstnSession> {
        let session = RstnSession::new(workflow);
        self.save_rstn_session(&session)?;
        debug!("Started rstn session: {} ({:?})", session.id, session.workflow);
        Ok(session)
    }

    /// Start a new Claude session within an rstn session
    pub fn start_claude_session(
        &self,
        rstn_session_id: String,
        purpose: String,
        max_turns: usize,
    ) -> Result<ClaudeSession> {
        let uuid = uuid::Uuid::new_v4().to_string();
        let session = ClaudeSession {
            uuid: uuid.clone(),
            purpose,
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: ClaudeSessionStatus::Running,
            turns_used: 0,
            max_turns,
            total_cost_usd: None,
            rstn_session_id,
        };
        self.save_claude_session(&session)?;
        debug!("Started Claude session: {} for rstn session: {}", uuid, session.rstn_session_id);
        Ok(session)
    }

    /// Complete a Claude session
    pub fn complete_claude_session(
        &self,
        session_id: &str,
        status: ClaudeSessionStatus,
        turns_used: usize,
        total_cost_usd: Option<f64>,
    ) -> Result<()> {
        // Load the session directly from database
        let mut stmt = self.db.prepare(
            "SELECT uuid, rstn_session_id, purpose, started_at, completed_at, status,
                    turns_used, max_turns, total_cost_usd
             FROM claude_sessions WHERE uuid = ?1"
        ).map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt.query(params![session_id])
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to query session: {}", e)))?;

        let row = rows.next()
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to fetch row: {}", e)))?
            .ok_or_else(|| RscliError::Other(anyhow::anyhow!("Claude session not found: {}", session_id)))?;

        // Parse status string
        let status_str: String = row.get(5).unwrap();
        let parsed_status = if status_str == "running" {
            ClaudeSessionStatus::Running
        } else if status_str == "completed" {
            ClaudeSessionStatus::Completed
        } else if status_str == "max_turns" {
            ClaudeSessionStatus::MaxTurns
        } else if status_str.starts_with("error:") {
            ClaudeSessionStatus::Error {
                message: status_str.strip_prefix("error:").unwrap().to_string(),
            }
        } else {
            ClaudeSessionStatus::Error {
                message: format!("Unknown status: {}", status_str),
            }
        };

        // Update the session
        let mut session = ClaudeSession {
            uuid: row.get(0).unwrap(),
            rstn_session_id: row.get(1).unwrap(),
            purpose: row.get(2).unwrap(),
            started_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3).unwrap())
                .unwrap()
                .with_timezone(&chrono::Utc),
            completed_at: row.get::<_, Option<String>>(4).unwrap().map(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .unwrap()
                    .with_timezone(&chrono::Utc)
            }),
            status: parsed_status,
            turns_used: row.get::<_, i64>(6).unwrap() as usize,
            max_turns: row.get::<_, i64>(7).unwrap() as usize,
            total_cost_usd: row.get(8).unwrap(),
        };

        // Update fields
        session.completed_at = Some(chrono::Utc::now());
        session.status = status;
        session.turns_used = turns_used;
        session.total_cost_usd = total_cost_usd;

        // Save back
        self.save_claude_session(&session)?;
        debug!("Completed Claude session: {} with status: {:?}", session_id, session.status);

        Ok(())
    }

    /// Get the active rstn session (most recent incomplete)
    pub fn get_active_rstn_session(&self) -> Result<Option<RstnSession>> {
        let sessions = self.list_recent_rstn_sessions(10)?;
        Ok(sessions.into_iter().find(|s| s.completed_at.is_none()))
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

    #[test]
    fn test_dual_layer_session_management() {
        // Use in-memory database for testing
        let manager = SessionManager {
            db: Connection::open_in_memory().unwrap(),
        };

        // Initialize dual-layer tables
        manager.init_dual_layer_tables().unwrap();

        // 1. Start an rstn session
        let rstn_session = manager.start_rstn_session(WorkflowType::Prompt).unwrap();
        assert_eq!(rstn_session.workflow, WorkflowType::Prompt);
        assert!(rstn_session.completed_at.is_none());
        assert_eq!(rstn_session.claude_sessions.len(), 0);

        // 2. Start a Claude session within it
        let claude_session = manager
            .start_claude_session(
                rstn_session.id.clone(),
                "Test prompt".to_string(),
                10,
            )
            .unwrap();
        assert_eq!(claude_session.rstn_session_id, rstn_session.id);
        assert_eq!(claude_session.max_turns, 10);
        assert_eq!(claude_session.turns_used, 0);
        assert!(matches!(claude_session.status, ClaudeSessionStatus::Running));

        // 3. Complete the Claude session
        manager
            .complete_claude_session(
                &claude_session.uuid,
                ClaudeSessionStatus::Completed,
                5,
                Some(0.05),
            )
            .unwrap();

        // 4. Load and verify the Claude session was updated
        let claude_sessions = manager
            .load_claude_sessions_for_rstn(&rstn_session.id)
            .unwrap();
        assert_eq!(claude_sessions.len(), 1);
        let loaded_claude = &claude_sessions[0];
        assert!(loaded_claude.completed_at.is_some());
        assert!(matches!(loaded_claude.status, ClaudeSessionStatus::Completed));
        assert_eq!(loaded_claude.turns_used, 5);
        assert_eq!(loaded_claude.total_cost_usd, Some(0.05));

        // 5. Load and verify the rstn session
        let loaded_rstn = manager.load_rstn_session(&rstn_session.id).unwrap();
        assert!(loaded_rstn.is_some());
        let loaded_rstn = loaded_rstn.unwrap();
        assert_eq!(loaded_rstn.workflow, WorkflowType::Prompt);
        assert_eq!(loaded_rstn.claude_sessions.len(), 1);
        assert!(loaded_rstn.claude_sessions.contains_key(&claude_session.uuid));

        // 6. Test active session retrieval
        let active = manager.get_active_rstn_session().unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, rstn_session.id);

        // 7. Complete the rstn session
        let mut completed_rstn = loaded_rstn;
        completed_rstn.completed_at = Some(chrono::Utc::now());
        manager.save_rstn_session(&completed_rstn).unwrap();

        // 8. Verify no active session now
        let active = manager.get_active_rstn_session().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_multiple_claude_sessions_in_rstn() {
        let manager = SessionManager {
            db: Connection::open_in_memory().unwrap(),
        };
        manager.init_dual_layer_tables().unwrap();

        // Start an rstn session
        let rstn_session = manager.start_rstn_session(WorkflowType::Specify).unwrap();

        // Start multiple Claude sessions
        let claude1 = manager
            .start_claude_session(
                rstn_session.id.clone(),
                "Generate spec".to_string(),
                10,
            )
            .unwrap();

        let claude2 = manager
            .start_claude_session(
                rstn_session.id.clone(),
                "Refine spec".to_string(),
                5,
            )
            .unwrap();

        // Complete both sessions
        manager
            .complete_claude_session(
                &claude1.uuid,
                ClaudeSessionStatus::Completed,
                7,
                Some(0.1),
            )
            .unwrap();

        manager
            .complete_claude_session(
                &claude2.uuid,
                ClaudeSessionStatus::MaxTurns,
                5,
                Some(0.05),
            )
            .unwrap();

        // Verify both sessions are loaded
        let loaded_rstn = manager.load_rstn_session(&rstn_session.id).unwrap().unwrap();
        assert_eq!(loaded_rstn.claude_sessions.len(), 2);

        // Verify individual session statuses
        let session1 = loaded_rstn.claude_sessions.get(&claude1.uuid).unwrap();
        let session2 = loaded_rstn.claude_sessions.get(&claude2.uuid).unwrap();

        assert!(matches!(session1.status, ClaudeSessionStatus::Completed));
        assert!(matches!(session2.status, ClaudeSessionStatus::MaxTurns));
        assert_eq!(session1.turns_used, 7);
        assert_eq!(session2.turns_used, 5);
    }
}
