//! Environment file management.
//!
//! Handles copying dotfiles between worktrees for environment synchronization.

use std::fs;
use std::path::Path;

/// Result of copying env files
#[derive(Debug, Clone)]
pub struct CopyEnvResult {
    /// Files that were successfully copied
    pub copied: Vec<String>,
    /// Files that failed to copy (path, error message)
    pub failed: Vec<(String, String)>,
}

impl CopyEnvResult {
    /// Check if the copy was fully successful
    pub fn is_success(&self) -> bool {
        self.failed.is_empty() && !self.copied.is_empty()
    }

    /// Check if the copy was partially successful
    pub fn is_partial(&self) -> bool {
        !self.copied.is_empty() && !self.failed.is_empty()
    }

    /// Create an empty result (nothing to copy)
    pub fn empty() -> Self {
        Self {
            copied: Vec::new(),
            failed: Vec::new(),
        }
    }
}

/// Default patterns to track for env copying
pub fn default_patterns() -> Vec<String> {
    vec![
        ".env".to_string(),
        ".envrc".to_string(),
        ".claude/".to_string(),
        ".vscode/".to_string(),
    ]
}

/// List env files matching patterns in a directory
///
/// Returns patterns that exist in the source directory.
pub fn list_env_files(dir: &str, patterns: &[String]) -> Vec<String> {
    let base = Path::new(dir);
    let mut files = Vec::new();

    for pattern in patterns {
        let target = base.join(pattern);
        if target.exists() {
            files.push(pattern.clone());
        }
    }

    files
}

/// Copy env files from source to destination worktree
///
/// # Arguments
/// * `from_path` - Source worktree path
/// * `to_path` - Destination worktree path
/// * `patterns` - Patterns of files/folders to copy
///
/// # Behavior
/// - Files are copied (not overwritten if they already exist)
/// - Directories are copied recursively
/// - Missing patterns are silently skipped
pub fn copy_env_files(
    from_path: &str,
    to_path: &str,
    patterns: &[String],
) -> Result<CopyEnvResult, String> {
    let from = Path::new(from_path);
    let to = Path::new(to_path);

    if !from.exists() {
        return Err(format!("Source path does not exist: {}", from_path));
    }

    if !to.exists() {
        return Err(format!("Destination path does not exist: {}", to_path));
    }

    let mut result = CopyEnvResult {
        copied: Vec::new(),
        failed: Vec::new(),
    };

    for pattern in patterns {
        let src = from.join(pattern);
        let dst = to.join(pattern);

        // Skip non-existent source files
        if !src.exists() {
            continue;
        }

        // Skip if destination already exists (don't overwrite)
        if dst.exists() {
            continue;
        }

        match copy_path(&src, &dst) {
            Ok(()) => result.copied.push(pattern.clone()),
            Err(e) => result.failed.push((pattern.clone(), e)),
        }
    }

    Ok(result)
}

/// Copy a file or directory
fn copy_path(src: &Path, dst: &Path) -> Result<(), String> {
    if src.is_dir() {
        copy_dir_recursive(src, dst)
    } else {
        // Ensure parent directory exists
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
        fs::copy(src, dst)
            .map(|_| ())
            .map_err(|e| format!("Failed to copy file: {}", e))
    }
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_default_patterns() {
        let patterns = default_patterns();
        assert!(patterns.contains(&".env".to_string()));
        assert!(patterns.contains(&".claude/".to_string()));
    }

    #[test]
    fn test_list_env_files_empty() {
        let temp = TempDir::new().unwrap();
        let files = list_env_files(temp.path().to_str().unwrap(), &default_patterns());
        assert!(files.is_empty());
    }

    #[test]
    fn test_list_env_files_with_matches() {
        let temp = TempDir::new().unwrap();

        // Create .env file
        let env_path = temp.path().join(".env");
        File::create(&env_path).unwrap().write_all(b"KEY=value").unwrap();

        // Create .claude directory
        let claude_dir = temp.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();

        let files = list_env_files(temp.path().to_str().unwrap(), &default_patterns());
        assert!(files.contains(&".env".to_string()));
        assert!(files.contains(&".claude/".to_string()));
        assert!(!files.contains(&".envrc".to_string())); // Not created
    }

    #[test]
    fn test_copy_env_files_basic() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        // Create source .env file
        let env_path = src_dir.path().join(".env");
        File::create(&env_path)
            .unwrap()
            .write_all(b"SECRET=12345")
            .unwrap();

        let result = copy_env_files(
            src_dir.path().to_str().unwrap(),
            dst_dir.path().to_str().unwrap(),
            &[".env".to_string()],
        )
        .unwrap();

        assert_eq!(result.copied.len(), 1);
        assert_eq!(result.copied[0], ".env");
        assert!(result.failed.is_empty());

        // Verify file was copied
        let dst_env = dst_dir.path().join(".env");
        assert!(dst_env.exists());
        assert_eq!(fs::read_to_string(dst_env).unwrap(), "SECRET=12345");
    }

    #[test]
    fn test_copy_env_files_skip_existing() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        // Create source .env file
        let src_env = src_dir.path().join(".env");
        File::create(&src_env)
            .unwrap()
            .write_all(b"NEW=value")
            .unwrap();

        // Create destination .env file (should not be overwritten)
        let dst_env = dst_dir.path().join(".env");
        File::create(&dst_env)
            .unwrap()
            .write_all(b"OLD=value")
            .unwrap();

        let result = copy_env_files(
            src_dir.path().to_str().unwrap(),
            dst_dir.path().to_str().unwrap(),
            &[".env".to_string()],
        )
        .unwrap();

        // Nothing should be copied (destination exists)
        assert!(result.copied.is_empty());
        assert!(result.failed.is_empty());

        // Verify destination file was NOT overwritten
        assert_eq!(fs::read_to_string(dst_env).unwrap(), "OLD=value");
    }

    #[test]
    fn test_copy_env_files_directory() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        // Create source .claude directory with files
        let claude_dir = src_dir.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();
        File::create(claude_dir.join("config.json"))
            .unwrap()
            .write_all(b"{}")
            .unwrap();

        let result = copy_env_files(
            src_dir.path().to_str().unwrap(),
            dst_dir.path().to_str().unwrap(),
            &[".claude/".to_string()],
        )
        .unwrap();

        assert_eq!(result.copied.len(), 1);

        // Verify directory and file were copied
        let dst_claude = dst_dir.path().join(".claude");
        assert!(dst_claude.exists());
        assert!(dst_claude.join("config.json").exists());
    }

    #[test]
    fn test_copy_env_result_helpers() {
        let empty = CopyEnvResult::empty();
        assert!(!empty.is_success());
        assert!(!empty.is_partial());

        let success = CopyEnvResult {
            copied: vec![".env".to_string()],
            failed: vec![],
        };
        assert!(success.is_success());
        assert!(!success.is_partial());

        let partial = CopyEnvResult {
            copied: vec![".env".to_string()],
            failed: vec![(".vscode/".to_string(), "error".to_string())],
        };
        assert!(!partial.is_success());
        assert!(partial.is_partial());
    }
}
