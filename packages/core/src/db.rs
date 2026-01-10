//! SQLite Database Management
//!
//! Handles user-scoped persistence for structured data like comments and logs.
//! Database is stored at ~/.rstn/state.db with project_id column for data isolation.

use rusqlite::{params, Connection, Result};
use std::sync::Mutex;
use uuid::Uuid;

/// Database manager - single global instance for all projects
pub struct DbManager {
    conn: Mutex<Connection>,
}

impl DbManager {
    /// Initialize database at user-scope (~/.rstn/state.db)
    pub fn init() -> Result<Self> {
        let rstn_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".rstn");

        if !rstn_dir.exists() {
            std::fs::create_dir_all(&rstn_dir).map_err(|e| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(e))
            })?;
        }

        let db_path = rstn_dir.join("state.db");
        let conn = Connection::open(db_path)?;

        // Enable WAL mode for better concurrency
        conn.execute("PRAGMA journal_mode=WAL;", [])?;

        let manager = Self { conn: Mutex::new(conn) };
        manager.run_migrations()?;

        Ok(manager)
    }

    /// Run initial migrations to set up tables
    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Table: File Comments (with project_id for multi-project isolation)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_comments (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                content TEXT NOT NULL,
                author TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                line_number INTEGER
            )",
            [],
        )?;

        // Migration: Add project_id column if it doesn't exist (for legacy data)
        let has_project_id: bool = conn
            .prepare("SELECT project_id FROM file_comments LIMIT 1")
            .is_ok();
        if !has_project_id {
            let _ = conn.execute(
                "ALTER TABLE file_comments ADD COLUMN project_id TEXT NOT NULL DEFAULT ''",
                [],
            );
        }

        // Migration: Add line_number column if it doesn't exist
        let has_line_number: bool = conn
            .prepare("SELECT line_number FROM file_comments LIMIT 1")
            .is_ok();
        if !has_line_number {
            let _ = conn.execute("ALTER TABLE file_comments ADD COLUMN line_number INTEGER", []);
        }

        // Index for comments (by project_id and file_path)
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_comments_project_path ON file_comments(project_id, file_path)",
            [],
        )?;

        // Table: Activity Logs (with project_id for multi-project isolation)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS activity_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL,
                level TEXT NOT NULL,
                summary TEXT NOT NULL,
                detail_json TEXT,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        // Migration: Add project_id column to activity_logs if it doesn't exist
        let has_log_project_id: bool = conn
            .prepare("SELECT project_id FROM activity_logs LIMIT 1")
            .is_ok();
        if !has_log_project_id {
            let _ = conn.execute(
                "ALTER TABLE activity_logs ADD COLUMN project_id TEXT NOT NULL DEFAULT ''",
                [],
            );
        }

        // Index for logs by project_id
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_activity_logs_project ON activity_logs(project_id)",
            [],
        )?;

        Ok(())
    }

    // ========================================================================
    // File Comments CRUD (all queries require project_id)
    // ========================================================================

    pub fn add_comment(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
        author: &str,
        line_number: Option<usize>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO file_comments (id, project_id, file_path, content, author, created_at, updated_at, line_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                project_id,
                file_path,
                content,
                author,
                now,
                now,
                line_number.map(|n| n as i64)
            ],
        )?;

        Ok(id)
    }

    pub fn get_comments(&self, project_id: &str, file_path: &str) -> Result<Vec<CommentRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, author, created_at, line_number FROM file_comments
             WHERE project_id = ?1 AND file_path = ?2 ORDER BY line_number ASC NULLS FIRST, created_at ASC",
        )?;

        let rows = stmt.query_map(params![project_id, file_path], |row| {
            let line_number: Option<i64> = row.get(4)?;
            Ok(CommentRow {
                id: row.get(0)?,
                content: row.get(1)?,
                author: row.get(2)?,
                created_at: row.get(3)?,
                line_number: line_number.map(|n| n as usize),
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn delete_comment(&self, project_id: &str, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM file_comments WHERE project_id = ?1 AND id = ?2",
            params![project_id, id],
        )?;
        Ok(())
    }

    pub fn get_comment_count(&self, project_id: &str, file_path: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM file_comments WHERE project_id = ?1 AND file_path = ?2",
        )?;
        let count: usize = stmt.query_row(params![project_id, file_path], |row| row.get(0))?;
        Ok(count)
    }

    // ========================================================================
    // Activity Logs (all queries require project_id)
    // ========================================================================

    pub fn add_log(
        &self,
        project_id: &str,
        category: &str,
        level: &str,
        summary: &str,
        detail_json: Option<&str>,
    ) -> Result<i64> {
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO activity_logs (project_id, category, level, summary, detail_json, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![project_id, category, level, summary, detail_json, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_logs(&self, project_id: &str, limit: usize) -> Result<Vec<LogRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, category, level, summary, detail_json, timestamp
             FROM activity_logs WHERE project_id = ?1 ORDER BY timestamp DESC LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![project_id, limit], |row| {
            Ok(LogRow {
                id: row.get(0)?,
                category: row.get(1)?,
                level: row.get(2)?,
                summary: row.get(3)?,
                detail_json: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CommentRow {
    pub id: String,
    pub content: String,
    pub author: String,
    pub created_at: String,
    /// Line number for inline comments (None for file-level comments)
    pub line_number: Option<usize>,
}

#[derive(Debug, serde::Serialize)]
pub struct LogRow {
    pub id: i64,
    pub category: String,
    pub level: String,
    pub summary: String,
    pub detail_json: Option<String>,
    pub timestamp: String,
}

// Activity Log integration will be added in Phase B1.3
