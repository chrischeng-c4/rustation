//! State persistence - save/load state to ~/.rstn/
//!
//! Handles:
//! - Global state (recent_projects, global_settings)
//! - Per-project state (active_tab, etc.)
//! - Schema versioning and migration

use crate::app_state::{AppState, FeatureTab, GlobalSettings, ProjectState, RecentProject};
use crate::migration::{MigrationManager, CURRENT_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Global persisted state - saved to ~/.rstn/state.json
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalPersistedState {
    /// Schema version for migration support
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// App version string (informational)
    pub version: String,
    pub recent_projects: Vec<RecentProject>,
    pub global_settings: GlobalSettings,
}

/// Default schema version for legacy data
fn default_schema_version() -> u32 {
    1
}

impl GlobalPersistedState {
    /// Extract persistable fields from AppState
    pub fn from_app_state(state: &AppState) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
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
        // Get active_tab from the active worktree
        let active_tab = project
            .active_worktree()
            .map(|w| w.active_tab)
            .unwrap_or_default();

        Self {
            path: project.path.clone(),
            active_tab,
        }
    }

    /// Apply persisted state to ProjectState
    pub fn apply_to(&self, project: &mut ProjectState) {
        // Only apply if path matches (sanity check)
        if self.path == project.path {
            // Apply active_tab to the active worktree
            if let Some(worktree) = project.active_worktree_mut() {
                worktree.active_tab = self.active_tab;
            }
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
/// Used as project_id for database isolation
pub fn path_to_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let result = hasher.finalize();
    // Take first 8 hex chars
    hex::encode(&result[..4])
}

/// Get the project ID (hash) for a project path
/// This is the same as path_to_hash but with a more descriptive name
pub fn get_project_id(project_path: &str) -> String {
    path_to_hash(project_path)
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

/// Load global state from disk with migration support.
///
/// This function:
/// 1. Reads the state file
/// 2. Checks schema version
/// 3. Applies any necessary migrations
/// 4. Creates a backup before migrating
/// 5. Saves the migrated state back to disk
pub fn load_global() -> Result<Option<GlobalPersistedState>, String> {
    let path = get_global_state_path();

    if !path.exists() {
        return Ok(None);
    }

    // Use migration manager to load and migrate
    let manager = MigrationManager::new();

    match manager.load_and_migrate(&path, true) {
        Ok(Some(value)) => {
            // Parse the (possibly migrated) JSON into our struct
            let persisted: GlobalPersistedState = serde_json::from_value(value)
                .map_err(|e| format!("Failed to deserialize state after migration: {}", e))?;
            Ok(Some(persisted))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            // Migration failed - log error and return None (app will use default state)
            tracing::error!("Migration failed: {}. Using default state.", e);
            // Optionally notify user through notification system
            Err(format!("State migration failed: {}", e))
        }
    }
}

/// Load global state from disk without migration (for testing).
#[cfg(test)]
pub fn load_global_raw() -> Result<Option<GlobalPersistedState>, String> {
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
            schema_version: CURRENT_SCHEMA_VERSION,
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
    fn test_global_persisted_state_legacy_without_schema_version() {
        // Test that legacy JSON without schema_version field defaults to 1
        let json = r#"{
            "version": "0.1.0",
            "recent_projects": [],
            "global_settings": {
                "theme": "system",
                "default_project_path": null
            }
        }"#;

        let loaded: GlobalPersistedState = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.schema_version, 1); // Defaults to 1
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
            schema_version: CURRENT_SCHEMA_VERSION,
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
        // Set active_tab through the worktree
        if let Some(worktree) = project.active_worktree_mut() {
            worktree.active_tab = FeatureTab::Settings;
        }

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
        // Verify default is Tasks
        assert_eq!(
            project.active_worktree().unwrap().active_tab,
            FeatureTab::Tasks
        );

        persisted.apply_to(&mut project);
        assert_eq!(
            project.active_worktree().unwrap().active_tab,
            FeatureTab::Dockers
        );
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
        assert_eq!(
            project.active_worktree().unwrap().active_tab,
            FeatureTab::Tasks
        );
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
            schema_version: CURRENT_SCHEMA_VERSION,
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

    // ========================================================================
    // Startup Flow Tests (Recent Projects Protection)
    // ========================================================================

    #[test]
    fn test_apply_to_preserves_recent_projects_order() {
        // Ensures apply_to correctly restores recent_projects in the same order
        let persisted = GlobalPersistedState {
            schema_version: CURRENT_SCHEMA_VERSION,
            version: "0.1.0".to_string(),
            recent_projects: vec![
                RecentProject {
                    path: "/first".to_string(),
                    name: "first".to_string(),
                    last_opened: "2024-12-25T12:00:00Z".to_string(),
                },
                RecentProject {
                    path: "/second".to_string(),
                    name: "second".to_string(),
                    last_opened: "2024-12-24T12:00:00Z".to_string(),
                },
                RecentProject {
                    path: "/third".to_string(),
                    name: "third".to_string(),
                    last_opened: "2024-12-23T12:00:00Z".to_string(),
                },
            ],
            global_settings: GlobalSettings::default(),
        };

        let mut state = AppState::default();
        persisted.apply_to(&mut state);

        // Order must be preserved
        assert_eq!(state.recent_projects.len(), 3);
        assert_eq!(state.recent_projects[0].path, "/first");
        assert_eq!(state.recent_projects[1].path, "/second");
        assert_eq!(state.recent_projects[2].path, "/third");
    }

    #[test]
    fn test_apply_to_does_not_open_projects() {
        // apply_to only sets recent_projects, does NOT open projects
        // (project opening is done separately in state_init)
        let persisted = GlobalPersistedState {
            schema_version: CURRENT_SCHEMA_VERSION,
            version: "0.1.0".to_string(),
            recent_projects: vec![RecentProject {
                path: "/my/project".to_string(),
                name: "project".to_string(),
                last_opened: "2024-12-25T12:00:00Z".to_string(),
            }],
            global_settings: GlobalSettings::default(),
        };

        let mut state = AppState::default();
        persisted.apply_to(&mut state);

        // recent_projects is populated
        assert_eq!(state.recent_projects.len(), 1);
        // But NO projects are opened (projects list stays empty)
        assert!(state.projects.is_empty());
    }

    #[test]
    fn test_from_app_state_captures_recent_projects() {
        // Ensures from_app_state correctly captures recent_projects for persistence
        let mut app_state = AppState::default();
        app_state.recent_projects = vec![
            RecentProject {
                path: "/project/one".to_string(),
                name: "one".to_string(),
                last_opened: "2024-12-25T10:00:00Z".to_string(),
            },
            RecentProject {
                path: "/project/two".to_string(),
                name: "two".to_string(),
                last_opened: "2024-12-24T10:00:00Z".to_string(),
            },
        ];
        app_state.global_settings.theme = Theme::Light;

        let persisted = GlobalPersistedState::from_app_state(&app_state);

        assert_eq!(persisted.recent_projects.len(), 2);
        assert_eq!(persisted.recent_projects[0].path, "/project/one");
        assert_eq!(persisted.recent_projects[1].path, "/project/two");
        assert_eq!(persisted.global_settings.theme, Theme::Light);
    }

    #[test]
    fn test_save_load_with_recent_projects_integration() {
        // Full integration: save state with recent_projects, load it back
        let temp_dir = env::temp_dir().join("rstn_test_recent");
        let _ = fs::remove_dir_all(&temp_dir);
        let state_path = temp_dir.join("state.json");
        fs::create_dir_all(&temp_dir).unwrap();

        // Create state with recent projects
        let persisted = GlobalPersistedState {
            schema_version: CURRENT_SCHEMA_VERSION,
            version: "0.1.0".to_string(),
            recent_projects: vec![
                RecentProject {
                    path: "/Users/test/project-a".to_string(),
                    name: "project-a".to_string(),
                    last_opened: "2024-12-25T12:00:00Z".to_string(),
                },
                RecentProject {
                    path: "/Users/test/project-b".to_string(),
                    name: "project-b".to_string(),
                    last_opened: "2024-12-24T12:00:00Z".to_string(),
                },
            ],
            global_settings: GlobalSettings {
                theme: Theme::Dark,
                default_project_path: Some("/Users/test".to_string()),
            },
        };

        // Save
        let json = serde_json::to_string_pretty(&persisted).unwrap();
        fs::write(&state_path, &json).unwrap();

        // Load
        let loaded_json = fs::read_to_string(&state_path).unwrap();
        let loaded: GlobalPersistedState = serde_json::from_str(&loaded_json).unwrap();

        // Verify all fields
        assert_eq!(loaded.version, "0.1.0");
        assert_eq!(loaded.recent_projects.len(), 2);
        assert_eq!(loaded.recent_projects[0].path, "/Users/test/project-a");
        assert_eq!(loaded.recent_projects[0].name, "project-a");
        assert_eq!(loaded.recent_projects[1].path, "/Users/test/project-b");
        assert_eq!(loaded.global_settings.theme, Theme::Dark);
        assert_eq!(
            loaded.global_settings.default_project_path,
            Some("/Users/test".to_string())
        );

        // Apply to new state
        let mut new_state = AppState::default();
        loaded.apply_to(&mut new_state);

        assert_eq!(new_state.recent_projects.len(), 2);
        assert_eq!(new_state.global_settings.theme, Theme::Dark);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_recent_project_struct_serialization() {
        // Ensure RecentProject serializes correctly
        let recent = RecentProject {
            path: "/path/to/project".to_string(),
            name: "project".to_string(),
            last_opened: "2024-12-25T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&recent).unwrap();
        let loaded: RecentProject = serde_json::from_str(&json).unwrap();

        assert_eq!(recent.path, loaded.path);
        assert_eq!(recent.name, loaded.name);
        assert_eq!(recent.last_opened, loaded.last_opened);
    }
}
