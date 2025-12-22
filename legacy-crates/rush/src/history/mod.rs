//! Command history management module
//!
//! Provides:
//! - Persistent command history
//! - History search and navigation
//! - Atomic file writes for crash safety

pub mod storage;

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// A record of a previously executed command
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    /// The executed command text
    pub command: String,

    /// Unix timestamp of execution
    pub timestamp: u64,

    /// Command exit code (None if still running)
    pub exit_code: Option<i32>,

    /// Directory where command was executed
    pub working_dir: PathBuf,

    /// Shell session identifier
    pub session_id: u64,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(command: String, working_dir: PathBuf, session_id: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self { command, timestamp, exit_code: None, working_dir, session_id }
    }

    /// Set the exit code after command completes
    pub fn with_exit_code(mut self, exit_code: i32) -> Self {
        self.exit_code = Some(exit_code);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_entry_new() {
        let entry = HistoryEntry::new("ls -la".to_string(), PathBuf::from("/home/user"), 42);

        assert_eq!(entry.command, "ls -la");
        assert!(entry.timestamp > 0);
        assert_eq!(entry.exit_code, None);
        assert_eq!(entry.working_dir, PathBuf::from("/home/user"));
        assert_eq!(entry.session_id, 42);
    }

    #[test]
    fn test_history_entry_with_exit_code() {
        let entry =
            HistoryEntry::new("echo test".to_string(), PathBuf::from("/tmp"), 1).with_exit_code(0);

        assert_eq!(entry.exit_code, Some(0));
    }

    #[test]
    fn test_history_entry_clone() {
        let entry1 = HistoryEntry::new("pwd".to_string(), PathBuf::from("/"), 5);
        let entry2 = entry1.clone();
        assert_eq!(entry1, entry2);
    }

    #[test]
    fn test_history_entry_timestamp() {
        let entry = HistoryEntry::new("test".to_string(), PathBuf::from("/tmp"), 1);

        // Timestamp should be recent (within last minute)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert!(entry.timestamp <= now);
        assert!(entry.timestamp > now - 60); // Within last minute
    }

    #[test]
    fn test_history_entry_session_ids() {
        let entry1 = HistoryEntry::new("cmd1".to_string(), PathBuf::from("/"), 100);
        let entry2 = HistoryEntry::new("cmd2".to_string(), PathBuf::from("/"), 200);

        assert_eq!(entry1.session_id, 100);
        assert_eq!(entry2.session_id, 200);
        assert_ne!(entry1.session_id, entry2.session_id);
    }

    #[test]
    fn test_history_entry_working_dir() {
        let dir1 = PathBuf::from("/home/user");
        let dir2 = PathBuf::from("/tmp");

        let entry1 = HistoryEntry::new("ls".to_string(), dir1.clone(), 1);
        let entry2 = HistoryEntry::new("pwd".to_string(), dir2.clone(), 1);

        assert_eq!(entry1.working_dir, dir1);
        assert_eq!(entry2.working_dir, dir2);
    }

    #[test]
    fn test_history_entry_chaining_with_exit_code() {
        let entry = HistoryEntry::new("test".to_string(), PathBuf::from("/"), 1).with_exit_code(42);

        assert_eq!(entry.exit_code, Some(42));
        assert_eq!(entry.command, "test");
    }

    #[test]
    fn test_history_entry_multiple_exit_codes() {
        let entry = HistoryEntry::new("cmd".to_string(), PathBuf::from("/"), 1)
            .with_exit_code(0)
            .with_exit_code(1); // Last one should win

        assert_eq!(entry.exit_code, Some(1));
    }

    #[test]
    fn test_history_entry_equality() {
        // Create two entries with same data
        let entry1 = HistoryEntry {
            command: "test".to_string(),
            timestamp: 12345,
            exit_code: Some(0),
            working_dir: PathBuf::from("/tmp"),
            session_id: 1,
        };

        let entry2 = HistoryEntry {
            command: "test".to_string(),
            timestamp: 12345,
            exit_code: Some(0),
            working_dir: PathBuf::from("/tmp"),
            session_id: 1,
        };

        assert_eq!(entry1, entry2);
    }

    #[test]
    fn test_history_entry_long_command() {
        let long_cmd = "a".repeat(1000);
        let entry = HistoryEntry::new(long_cmd.clone(), PathBuf::from("/"), 1);

        assert_eq!(entry.command, long_cmd);
        assert_eq!(entry.command.len(), 1000);
    }

    #[test]
    fn test_history_entry_unicode_command() {
        let entry = HistoryEntry::new("echo 你好世界 🚀".to_string(), PathBuf::from("/"), 1);

        assert_eq!(entry.command, "echo 你好世界 🚀");
    }

    #[test]
    fn test_history_entry_special_chars() {
        let entry = HistoryEntry::new(
            r#"echo "test" && ls | grep 'file'"#.to_string(),
            PathBuf::from("/"),
            1,
        );

        assert_eq!(entry.command, r#"echo "test" && ls | grep 'file'"#);
    }
}
