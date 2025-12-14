use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Tracks file modification times for change detection
pub struct FileChangeTracker {
    file_mtimes: HashMap<PathBuf, SystemTime>,
}

impl FileChangeTracker {
    /// Create new empty tracker
    pub fn new() -> Self {
        Self {
            file_mtimes: HashMap::new(),
        }
    }

    /// Check files for changes, return paths of modified files
    pub fn check_files(&mut self, paths: &[PathBuf]) -> Vec<PathBuf> {
        let mut changed = Vec::new();

        for path in paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(mtime) = metadata.modified() {
                    if let Some(&stored_mtime) = self.file_mtimes.get(path) {
                        if mtime != stored_mtime {
                            // File changed since last check
                            changed.push(path.clone());
                            self.file_mtimes.insert(path.clone(), mtime);
                        }
                    } else {
                        // First time seeing this file - store mtime
                        self.file_mtimes.insert(path.clone(), mtime);
                    }
                }
            }
            // If file doesn't exist or metadata fails, skip silently
        }

        changed
    }
}

impl Default for FileChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}
