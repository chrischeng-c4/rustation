//! Secure file reading with path validation.
//!
//! Provides file reading within allowed security scopes:
//! - Project root directory (and subdirectories)
//! - ~/.rstn/ directory (and subdirectories)

use crate::persistence::get_rstn_dir;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Maximum file size (10MB)
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// File reading errors
#[derive(Debug, Error)]
pub enum FileReadError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Security violation: path '{0}' is outside allowed scope")]
    SecurityViolation(String),

    #[error("File too large: {size} bytes exceeds limit of {limit} bytes")]
    FileTooLarge { size: u64, limit: u64 },

    #[error("File is not valid UTF-8 text")]
    NotUtf8,

    #[error("IO error: {0}")]
    Io(String),
}

/// Read a file with security validation.
///
/// # Arguments
/// * `path` - Path to file to read
/// * `project_root` - Project root directory (allowed scope)
///
/// # Security
/// File must be within:
/// - `project_root` or its subdirectories
/// - `~/.rstn/` or its subdirectories
///
/// # Returns
/// File contents as UTF-8 string, or error
pub fn read_file(path: &str, project_root: &str) -> Result<String, FileReadError> {
    let file_path = Path::new(path);

    // Canonicalize paths for security
    let canonical_path = file_path.canonicalize().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => FileReadError::NotFound(path.to_string()),
        std::io::ErrorKind::PermissionDenied => FileReadError::PermissionDenied(path.to_string()),
        _ => FileReadError::Io(e.to_string()),
    })?;

    // Build allowed roots
    let allowed_roots = build_allowed_roots(project_root)?;

    // Security check: path must be within allowed roots
    if !is_path_allowed(&canonical_path, &allowed_roots) {
        return Err(FileReadError::SecurityViolation(path.to_string()));
    }

    // Check file size
    let metadata =
        std::fs::metadata(&canonical_path).map_err(|e| FileReadError::Io(e.to_string()))?;

    if metadata.len() > MAX_FILE_SIZE {
        return Err(FileReadError::FileTooLarge {
            size: metadata.len(),
            limit: MAX_FILE_SIZE,
        });
    }

    // Read file
    std::fs::read_to_string(&canonical_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::InvalidData {
            FileReadError::NotUtf8
        } else {
            FileReadError::Io(e.to_string())
        }
    })
}

/// Read a binary file with security validation.
///
/// # Arguments
/// * `path` - Path to file to read
/// * `project_root` - Project root directory (allowed scope)
///
/// # Security
/// File must be within:
/// - `project_root` or its subdirectories
/// - `~/.rstn/` or its subdirectories
///
/// # Returns
/// File contents as raw bytes (Vec<u8>), or error
pub fn read_binary_file(path: &str, project_root: &str) -> Result<Vec<u8>, FileReadError> {
    let file_path = Path::new(path);

    // Canonicalize paths for security
    let canonical_path = file_path.canonicalize().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => FileReadError::NotFound(path.to_string()),
        std::io::ErrorKind::PermissionDenied => FileReadError::PermissionDenied(path.to_string()),
        _ => FileReadError::Io(e.to_string()),
    })?;

    // Build allowed roots
    let allowed_roots = build_allowed_roots(project_root)?;

    // Security check: path must be within allowed roots
    if !is_path_allowed(&canonical_path, &allowed_roots) {
        return Err(FileReadError::SecurityViolation(path.to_string()));
    }

    // Check file size
    let metadata =
        std::fs::metadata(&canonical_path).map_err(|e| FileReadError::Io(e.to_string()))?;

    if metadata.len() > MAX_FILE_SIZE {
        return Err(FileReadError::FileTooLarge {
            size: metadata.len(),
            limit: MAX_FILE_SIZE,
        });
    }

    // Read file as binary
    std::fs::read(&canonical_path).map_err(|e| FileReadError::Io(e.to_string()))
}

/// Build list of allowed root directories
fn build_allowed_roots(project_root: &str) -> Result<Vec<PathBuf>, FileReadError> {
    let mut roots = Vec::new();

    // Add project root (canonicalized)
    let project_path = Path::new(project_root);
    if project_path.exists() {
        if let Ok(canonical) = project_path.canonicalize() {
            roots.push(canonical);
        }
    }

    // Add ~/.rstn/
    let rstn_dir = get_rstn_dir();
    if rstn_dir.exists() {
        if let Ok(canonical) = rstn_dir.canonicalize() {
            roots.push(canonical);
        }
    } else {
        // ~/.rstn/ might not exist yet, add it anyway for future creation
        roots.push(rstn_dir);
    }

    Ok(roots)
}

/// Check if path is within any allowed root
fn is_path_allowed(path: &Path, allowed_roots: &[PathBuf]) -> bool {
    allowed_roots.iter().any(|root| path.starts_with(root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_read_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let result = read_file(
            file_path.to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_read_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let result = read_file(
            "/nonexistent/file.txt",
            temp_dir.path().to_str().unwrap(),
        );

        assert!(matches!(result, Err(FileReadError::NotFound(_))));
    }

    #[test]
    fn test_read_file_security_violation() {
        let temp_dir = TempDir::new().unwrap();
        let outside_dir = TempDir::new().unwrap();
        let file_path = outside_dir.path().join("secret.txt");
        fs::write(&file_path, "secret").unwrap();

        let result = read_file(
            file_path.to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
        );

        assert!(matches!(result, Err(FileReadError::SecurityViolation(_))));
    }

    #[test]
    fn test_read_file_in_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        let file_path = sub_dir.join("test.txt");
        fs::write(&file_path, "nested content").unwrap();

        let result = read_file(
            file_path.to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "nested content");
    }

    #[test]
    fn test_read_file_traversal_attack() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        // Try to escape with ../
        let attack_path = format!(
            "{}/../../../etc/passwd",
            temp_dir.path().to_str().unwrap()
        );

        let result = read_file(&attack_path, temp_dir.path().to_str().unwrap());

        // Should fail - either NotFound or SecurityViolation
        assert!(result.is_err());
    }

    #[test]
    fn test_is_path_allowed() {
        let roots = vec![
            PathBuf::from("/home/user/project"),
            PathBuf::from("/home/user/.rstn"),
        ];

        assert!(is_path_allowed(
            Path::new("/home/user/project/src/main.rs"),
            &roots
        ));
        assert!(is_path_allowed(
            Path::new("/home/user/.rstn/state.json"),
            &roots
        ));
        assert!(!is_path_allowed(Path::new("/etc/passwd"), &roots));
        assert!(!is_path_allowed(
            Path::new("/home/user/other/file"),
            &roots
        ));
    }
}
