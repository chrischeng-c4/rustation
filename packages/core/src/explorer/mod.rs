//! File Explorer Backend Logic
//!
//! Handles directory traversal, Git status integration, and file metadata.

use crate::app_state::{FileEntry, FileKind, GitFileStatus};
use crate::db::DbManager;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Read a directory and return a list of file entries with Git status.
/// Respects .gitignore rules.
pub fn read_directory(
    path: &Path,
    project_root: &Path,
    project_id: &str,
    db: Option<&DbManager>,
) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    
    // 1. Get Git status for the project to overlay on files
    let git_status_map = get_git_status(project_root).unwrap_or_default();

    // 2. Read directory entries using 'ignore' crate
    // We only want immediate children, so we set max_depth to 1.
    let walker = WalkBuilder::new(path)
        .standard_filters(true) // respects .gitignore, etc.
        .max_depth(Some(1))
        .build();

    for result in walker {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Skip the directory itself
        if entry.path() == path {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        
        let file_path = entry.path();
        
        // Get relative path for Git matching and UI
        let rel_path = file_path
            .strip_prefix(project_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let name = entry.file_name().to_string_lossy().to_string();
        let kind = if metadata.is_dir() {
            FileKind::Directory
        } else if metadata.is_symlink() {
            FileKind::Symlink
        } else {
            FileKind::File
        };

        let permissions = get_permissions_string(&metadata);
        let updated_at = metadata
            .modified()
            .ok()
            .and_then(|t| {
                let dt: chrono::DateTime<chrono::Utc> = t.into();
                Some(dt.to_rfc3339())
            })
            .unwrap_or_default();

        let git_status = git_status_map.get(&rel_path).cloned();
        
        // Fetch comment count from SQLite (requires project_id for isolation)
        let comment_count = if let Some(db_mgr) = db {
            db_mgr.get_comment_count(project_id, &rel_path).unwrap_or(0)
        } else {
            0
        };

        entries.push(FileEntry {
            name,
            path: file_path.to_string_lossy().to_string(),
            kind,
            size: metadata.len(),
            permissions,
            updated_at,
            comment_count,
            git_status,
        });
    }

    Ok(entries)
}

/// Run `git status --porcelain` and parse results into a map of Path -> Status
fn get_git_status(project_root: &Path) -> Option<HashMap<String, GitFileStatus>> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .arg("--ignored")
        .current_dir(project_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut status_map = HashMap::new();

    for line in stdout.lines() {
        if line.len() < 4 { continue; }
        
        let status_code = &line[0..2];
        let path = line[3..].trim().trim_matches('"');
        
        let status = match status_code {
            " M" | "M " => GitFileStatus::Modified,
            " A" | "A " => GitFileStatus::Added,
            " D" | "D " => GitFileStatus::Deleted,
            "??" => GitFileStatus::Untracked,
            "!!" => GitFileStatus::Ignored,
            _ => continue,
        };
        
        status_map.insert(path.to_string(), status);
    }

    Some(status_map)
}

/// Convert metadata permissions into Unix-style string (e.g., "rwxr-xr-x")
fn get_permissions_string(metadata: &fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        let mut s = String::with_capacity(9);
        
        // Owner
        s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
        s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
        s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
        
        // Group
        s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
        s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
        s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
        
        // Others
        s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
        s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
        s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
        
        s
    }
    #[cfg(not(unix))]
    {
        if metadata.permissions().readonly() { "r--r--r--".to_string() } else { "rw-rw-rw-".to_string() }
    }
}
