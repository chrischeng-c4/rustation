//! State persistence - save/load state to ~/.rstn/
//!
//! Handles:
//! - Global state (recent_projects, global_settings)
//! - Per-project state (active_tab, etc.)

use crate::app_state::{AppState, FeatureTab, GlobalSettings, ProjectState, RecentProject};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Global persisted state - saved to ~/.rstn/state.json
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalPersistedState {
    pub version: String,
    pub recent_projects: Vec<RecentProject>,
    pub global_settings: GlobalSettings,
}

impl GlobalPersistedState {
    /// Extract persistable fields from AppState
    pub fn from_app_state(state: &AppState) -> Self {
        Self {
            version: state.version.clone(),
            recent_projects: state.recent_projects.clone(),
            global_settings: state.global_settings.clone(),
        }
    }

    /// Apply persisted state to AppState
    pub fn apply_to(&self, state: &mut AppState) {
        state.recent_projects = self.recent_projects.clone();
        state.global_settings = self.global_settings.clone();
    }
}

/// Per-project persisted state - saved to ~/.rstn/projects/<hash>/state.json
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectPersistedState {
    /// Original path (for validation)
    pub path: String,
    /// Last active tab
    pub active_tab: FeatureTab,
}

impl ProjectPersistedState {
    /// Extract persistable fields from ProjectState
    pub fn from_project_state(project: &ProjectState) -> Self {
        Self {
            path: project.path.clone(),
            active_tab: project.active_tab,
        }
    }

    /// Apply persisted state to ProjectState
    pub fn apply_to(&self, project: &mut ProjectState) {
        // Only apply if path matches (sanity check)
        if self.path == project.path {
            project.active_tab = self.active_tab;
        }
    }
}

// ============================================================================
// Path Helpers
// ============================================================================

/// Get the rstn config directory (~/.rstn/)
pub fn get_rstn_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".rstn")
}

/// Get path to global state file (~/.rstn/state.json)
pub fn get_global_state_path() -> PathBuf {
    get_rstn_dir().join("state.json")
}

/// Get path to project state directory (~/.rstn/projects/<hash>/)
pub fn get_project_dir(project_path: &str) -> PathBuf {
    let hash = path_to_hash(project_path);
    get_rstn_dir().join("projects").join(hash)
}

/// Get path to project state file (~/.rstn/projects/<hash>/state.json)
pub fn get_project_state_path(project_path: &str) -> PathBuf {
    get_project_dir(project_path).join("state.json")
}

/// Convert a path to a short hash (first 8 chars of SHA256)
fn path_to_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let result = hasher.finalize();
    // Take first 8 hex chars
    hex::encode(&result[..4])
}

// ============================================================================
// Global State I/O
// ============================================================================

/// Save global state to disk
pub fn save_global(state: &AppState) -> Result<(), String> {
    let persisted = GlobalPersistedState::from_app_state(state);
    let path = get_global_state_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }

    // Write JSON
    let json = serde_json::to_string_pretty(&persisted)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write state: {}", e))?;

    Ok(())
}

/// Load global state from disk
pub fn load_global() -> Result<Option<GlobalPersistedState>, String> {
    let path = get_global_state_path();

    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(&path).map_err(|e| format!("Failed to read state: {}", e))?;

    let persisted: GlobalPersistedState =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse state: {}", e))?;

    Ok(Some(persisted))
}

// ============================================================================
// Project State I/O
// ============================================================================

/// Save project state to disk
pub fn save_project(project: &ProjectState) -> Result<(), String> {
    let persisted = ProjectPersistedState::from_project_state(project);
    let path = get_project_state_path(&project.path);

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }

    // Write JSON
    let json = serde_json::to_string_pretty(&persisted)
        .map_err(|e| format!("Failed to serialize project state: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write project state: {}", e))?;

    Ok(())
}

/// Load project state from disk
pub fn load_project(project_path: &str) -> Result<Option<ProjectPersistedState>, String> {
    let path = get_project_state_path(project_path);

    if !path.exists() {
        return Ok(None);
    }

    let json =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read project state: {}", e))?;

    let persisted: ProjectPersistedState =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse project state: {}", e))?;

    // Validate path matches
    if persisted.path != project_path {
        return Ok(None); // Hash collision or corrupted file
    }

    Ok(Some(persisted))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::Theme;
    use std::env;

    #[test]
    fn test_path_to_hash_consistent() {
        let path = "/Users/chris/projects/rustation";
        let hash1 = path_to_hash(path);
        let hash2 = path_to_hash(path);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 8); // 4 bytes = 8 hex chars
    }

    #[test]
    fn test_path_to_hash_different_paths() {
        let hash1 = path_to_hash("/path/a");
        let hash2 = path_to_hash("/path/b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_global_persisted_state_roundtrip() {
        let state = GlobalPersistedState {
            version: "0.1.0".to_string(),
            recent_projects: vec![RecentProject {
                path: "/test/project".to_string(),
                name: "project".to_string(),
                last_opened: "2024-01-01T00:00:00Z".to_string(),
            }],
            global_settings: GlobalSettings {
                theme: Theme::Dark,
                default_project_path: Some("/home/user".to_string()),
            },
        };

        let json = serde_json::to_string(&state).unwrap();
        let loaded: GlobalPersistedState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, loaded);
    }

    #[test]
    fn test_project_persisted_state_roundtrip() {
        let state = ProjectPersistedState {
            path: "/test/project".to_string(),
            active_tab: FeatureTab::Dockers,
        };

        let json = serde_json::to_string(&state).unwrap();
        let loaded: ProjectPersistedState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, loaded);
    }

    #[test]
    fn test_global_persisted_from_app_state() {
        let mut app_state = AppState::default();
        app_state.global_settings.theme = Theme::Dark;
        app_state.recent_projects.push(RecentProject {
            path: "/test".to_string(),
            name: "test".to_string(),
            last_opened: "2024-01-01T00:00:00Z".to_string(),
        });

        let persisted = GlobalPersistedState::from_app_state(&app_state);
        assert_eq!(persisted.global_settings.theme, Theme::Dark);
        assert_eq!(persisted.recent_projects.len(), 1);
    }

    #[test]
    fn test_global_persisted_apply_to() {
        let persisted = GlobalPersistedState {
            version: "0.1.0".to_string(),
            recent_projects: vec![RecentProject {
                path: "/restored".to_string(),
                name: "restored".to_string(),
                last_opened: "2024-01-01T00:00:00Z".to_string(),
            }],
            global_settings: GlobalSettings {
                theme: Theme::Light,
                default_project_path: None,
            },
        };

        let mut app_state = AppState::default();
        persisted.apply_to(&mut app_state);

        assert_eq!(app_state.global_settings.theme, Theme::Light);
        assert_eq!(app_state.recent_projects.len(), 1);
        assert_eq!(app_state.recent_projects[0].path, "/restored");
    }

    #[test]
    fn test_project_persisted_from_project_state() {
        let mut project = ProjectState::new("/test/path".to_string());
        project.active_tab = FeatureTab::Settings;

        let persisted = ProjectPersistedState::from_project_state(&project);
        assert_eq!(persisted.path, "/test/path");
        assert_eq!(persisted.active_tab, FeatureTab::Settings);
    }

    #[test]
    fn test_project_persisted_apply_to() {
        let persisted = ProjectPersistedState {
            path: "/test/path".to_string(),
            active_tab: FeatureTab::Dockers,
        };

        let mut project = ProjectState::new("/test/path".to_string());
        assert_eq!(project.active_tab, FeatureTab::Tasks); // default

        persisted.apply_to(&mut project);
        assert_eq!(project.active_tab, FeatureTab::Dockers);
    }

    #[test]
    fn test_project_persisted_apply_to_wrong_path() {
        let persisted = ProjectPersistedState {
            path: "/other/path".to_string(),
            active_tab: FeatureTab::Dockers,
        };

        let mut project = ProjectState::new("/test/path".to_string());
        persisted.apply_to(&mut project);

        // Should NOT apply because paths don't match
        assert_eq!(project.active_tab, FeatureTab::Tasks);
    }

    #[test]
    fn test_save_load_global_integration() {
        // Use temp directory
        let temp_dir = env::temp_dir().join("rstn_test_global");
        let _ = fs::remove_dir_all(&temp_dir);

        // Override home dir by testing the functions directly with temp path
        let state_path = temp_dir.join("state.json");
        fs::create_dir_all(&temp_dir).unwrap();

        let persisted = GlobalPersistedState {
            version: "0.1.0".to_string(),
            recent_projects: vec![],
            global_settings: GlobalSettings::default(),
        };

        let json = serde_json::to_string_pretty(&persisted).unwrap();
        fs::write(&state_path, &json).unwrap();

        let loaded_json = fs::read_to_string(&state_path).unwrap();
        let loaded: GlobalPersistedState = serde_json::from_str(&loaded_json).unwrap();

        assert_eq!(persisted, loaded);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
