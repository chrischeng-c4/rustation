//! Session management for Claude Code sessions per feature
//!
//! Each feature gets its own session ID stored in ~/.rstn/sessions/<feature>.session
//! This enables Claude Code to use cached context for cost savings.

use std::path::PathBuf;

/// Get the base directory for rstn data (with migration from old paths)
pub fn get_data_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    let new_path = home.join(".rstn");
    let old_path = home.join(".rustation");
    let legacy_path = home.join(".rust-station");

    // Priority: .rstn > .rustation > .rust-station
    if !new_path.exists() {
        if old_path.exists() {
            // Migrate .rustation → .rstn
            if let Err(e) = std::fs::rename(&old_path, &new_path) {
                eprintln!(
                    "Warning: Could not migrate data from {} to {}: {}",
                    old_path.display(),
                    new_path.display(),
                    e
                );
                eprintln!("Continuing with old path. You may want to manually migrate.");
                return old_path; // Fall back to old path
            } else {
                println!(
                    "✓ Migrated rstn data: {} → {}",
                    old_path.display(),
                    new_path.display()
                );
            }
        } else if legacy_path.exists() {
            // Migrate .rust-station → .rstn
            if let Err(e) = std::fs::rename(&legacy_path, &new_path) {
                eprintln!(
                    "Warning: Could not migrate legacy data from {} to {}: {}",
                    legacy_path.display(),
                    new_path.display(),
                    e
                );
                return legacy_path; // Fall back to legacy path
            } else {
                println!(
                    "✓ Migrated legacy data: {} → {}",
                    legacy_path.display(),
                    new_path.display()
                );
            }
        }
    }

    new_path
}

/// Get the sessions directory
pub fn get_sessions_dir() -> PathBuf {
    get_data_dir().join("sessions")
}

/// Get the path to a feature's session file
pub fn get_session_path(feature_num: &str) -> PathBuf {
    get_sessions_dir().join(format!("{}.session", feature_num))
}

/// Load session ID for a feature
pub fn load_session_id(feature_num: &str) -> Option<String> {
    let path = get_session_path(feature_num);
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Save session ID for a feature
pub fn save_session_id(feature_num: &str, session_id: &str) -> std::io::Result<()> {
    let path = get_session_path(feature_num);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, session_id)
}

/// Clear session for a feature
pub fn clear_session(feature_num: &str) -> std::io::Result<()> {
    let path = get_session_path(feature_num);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Clear all sessions
pub fn clear_all_sessions() -> std::io::Result<()> {
    let dir = get_sessions_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

/// List all feature sessions
pub fn list_sessions() -> std::io::Result<Vec<(String, String)>> {
    let dir = get_sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "session") {
            if let Some(stem) = path.file_stem() {
                let feature = stem.to_string_lossy().to_string();
                if let Ok(session_id) = std::fs::read_to_string(&path) {
                    sessions.push((feature, session_id.trim().to_string()));
                }
            }
        }
    }
    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_session_path() {
        let path = get_session_path("041");
        assert!(path
            .to_string_lossy()
            .contains(".rstn/sessions/041.session"));
    }
}
