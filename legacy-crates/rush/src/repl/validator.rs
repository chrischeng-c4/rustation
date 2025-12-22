//! Validator module for syntax highlighting
//!
//! Provides functionality to validate commands and paths in real-time.

use std::path::Path;
use which::which;

/// Validates commands and paths
pub struct Validator;

impl Validator {
    /// Check if a command is valid (exists in PATH or is a builtin)
    pub fn validate_command(command: &str) -> bool {
        // Check builtins first
        match command {
            "cd" | "exit" | "quit" | "history" | "jobs" | "fg" | "bg" => return true,
            _ => {}
        }

        // Check PATH
        which(command).is_ok()
    }

    /// Check if a path is valid (exists on disk)
    pub fn validate_path(path_str: &str) -> bool {
        // Simple existence check
        // In a real shell, we might want to handle ~ expansion here too
        // For now, just check direct existence
        Path::new(path_str).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_validate_builtin() {
        assert!(Validator::validate_command("cd"));
        assert!(Validator::validate_command("jobs"));
        assert!(Validator::validate_command("exit"));
    }

    #[test]
    fn test_validate_path_existence() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file");
        File::create(&file_path).unwrap();

        assert!(Validator::validate_path(file_path.to_str().unwrap()));
        assert!(!Validator::validate_path("/nonexistent/path/definitely"));
    }
}
