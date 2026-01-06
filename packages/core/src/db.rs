//! SQLite Database Management
//!
//! Handles project-level persistence for structured data like comments and logs.

use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

/// Database manager for a specific project
pub struct DbManager {
    conn: Mutex<Connection>,
}

impl DbManager {
    /// Initialize database at the given project root
    pub fn init(project_root: &Path) -> Result<Self> {
        let rstn_dir = project_root.join(".rstn");
        if !rstn_dir.exists() {
            std::fs::create_dir_all(&rstn_dir).map_err(|e| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(e))
            })?;
        }

        let db_path = rstn_dir.join("rstn.db");
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
        // Table: File Comments
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_comments (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                content TEXT NOT NULL,
                author TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Index for comments
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_comments_path ON file_comments(file_path)",
            [],
        )?;

        // Table: Activity Logs
        conn.execute(
            "CREATE TABLE IF NOT EXISTS activity_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                level TEXT NOT NULL,
                summary TEXT NOT NULL,
                detail_json TEXT,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    // ========================================================================
    // File Comments CRUD
    // ========================================================================

    pub fn add_comment(&self, file_path: &str, content: &str, author: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO file_comments (id, file_path, content, author, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, file_path, content, author, now, now],
        )?;

        Ok(id)
    }

    pub fn get_comments(&self, file_path: &str) -> Result<Vec<CommentRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, author, created_at FROM file_comments 
             WHERE file_path = ?1 ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(params![file_path], |row| {
            Ok(CommentRow {
                id: row.get(0)?,
                content: row.get(1)?,
                author: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn delete_comment(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM file_comments WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_comment_count(&self, file_path: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM file_comments WHERE file_path = ?1")?;
        let count: usize = stmt.query_row(params![file_path], |row| row.get(0))?;
        Ok(count)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CommentRow {
    pub id: String,
    pub content: String,
    pub author: String,
    pub created_at: String,
}

// Activity Log integration will be added in Phase B1.3
