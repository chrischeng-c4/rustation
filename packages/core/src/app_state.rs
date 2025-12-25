//! Application state - Single Source of Truth
//!
//! All state MUST be JSON serializable for:
//! - State persistence (save/load sessions)
//! - State sync (push to React via IPC)
//! - Testing (state round-trip tests)
//! - Debugging (time-travel, bug reproduction)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Main application state - single source of truth
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    /// App version
    pub version: String,
    /// All open projects
    pub projects: Vec<ProjectState>,
    /// Index of the currently active project
    pub active_project_index: usize,
    /// Global settings (theme, etc.)
    pub global_settings: GlobalSettings,
    /// Recent projects for "Open Recent" menu
    pub recent_projects: Vec<RecentProject>,
    /// Global error (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AppError>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            projects: Vec::new(),
            active_project_index: 0,
            global_settings: GlobalSettings::default(),
            recent_projects: Vec::new(),
            error: None,
        }
    }
}

impl AppState {
    /// Get the active project (if any)
    pub fn active_project(&self) -> Option<&ProjectState> {
        self.projects.get(self.active_project_index)
    }

    /// Get the active project mutably (if any)
    pub fn active_project_mut(&mut self) -> Option<&mut ProjectState> {
        self.projects.get_mut(self.active_project_index)
    }
}

// ============================================================================
// Project State
// ============================================================================

/// State for a single open project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectState {
    /// Unique identifier
    pub id: String,
    /// Filesystem path to the project
    pub path: String,
    /// Display name (folder name)
    pub name: String,
    /// Whether the project has unsaved changes or running tasks
    pub is_modified: bool,
    /// Currently active feature tab within this project
    pub active_tab: FeatureTab,
    /// Tasks state for this project
    pub tasks: TasksState,
    /// Docker state for this project
    pub dockers: DockersState,
}

impl ProjectState {
    /// Create a new project from a path
    pub fn new(path: String) -> Self {
        let name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();

        Self {
            id: Uuid::new_v4().to_string(),
            path,
            name,
            is_modified: false,
            active_tab: FeatureTab::Tasks,
            tasks: TasksState::default(),
            dockers: DockersState::default(),
        }
    }
}

/// Feature tabs within a project (sidebar)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FeatureTab {
    #[default]
    Tasks,
    Dockers,
    Settings,
}

/// Recent project entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecentProject {
    /// Filesystem path
    pub path: String,
    /// Display name
    pub name: String,
    /// Last opened timestamp (ISO 8601)
    pub last_opened: String,
}

// ============================================================================
// Global Settings
// ============================================================================

/// Global application settings
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GlobalSettings {
    /// UI theme
    pub theme: Theme,
    /// Default project path for "Open Folder" dialog
    pub default_project_path: Option<String>,
}

// ============================================================================
// Docker State
// ============================================================================

/// Docker tab state
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DockersState {
    /// Whether Docker is available on this system
    pub docker_available: Option<bool>,
    /// List of Docker services
    pub services: Vec<DockerServiceInfo>,
    /// Currently selected service ID
    pub selected_service_id: Option<String>,
    /// Logs for selected service
    pub logs: Vec<String>,
    /// Loading state for services list
    pub is_loading: bool,
    /// Loading state for logs
    pub is_loading_logs: bool,
}

/// Docker service info (matches existing DockerService but owned by state)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DockerServiceInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: ServiceStatus,
    pub port: Option<u32>,
    pub service_type: ServiceType,
}

/// Service status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Running,
    #[default]
    Stopped,
    Starting,
    Stopping,
    Error,
}

/// Service type - determines available features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ServiceType {
    /// Database (PostgreSQL, MySQL, MongoDB) - can create databases
    Database,
    /// Message broker (RabbitMQ) - can create vhosts
    MessageBroker,
    /// Cache (Redis)
    Cache,
    /// Other services
    #[default]
    Other,
}

// ============================================================================
// Tasks State
// ============================================================================

/// Tasks tab state
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TasksState {
    /// Justfile commands
    pub commands: Vec<JustCommandInfo>,
    /// Status of each task (by name)
    pub task_statuses: HashMap<String, TaskStatus>,
    /// Currently active/running command
    pub active_command: Option<String>,
    /// Output from last command
    pub output: Vec<String>,
    /// Loading state
    pub is_loading: bool,
    /// Error message
    pub error: Option<String>,
}

/// Justfile command info
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JustCommandInfo {
    pub name: String,
    pub description: Option<String>,
    pub recipe: String,
}

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    #[default]
    Idle,
    Running,
    Success,
    Error,
}

/// UI theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

// ============================================================================
// Error Type
// ============================================================================

/// Application error
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppError {
    /// Error code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_serialization_roundtrip() {
        let state = AppState::default();
        let json = serde_json::to_string(&state).unwrap();
        let loaded: AppState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, loaded);
    }

    #[test]
    fn test_app_state_with_project_roundtrip() {
        let mut state = AppState::default();

        // Add a project
        let mut project = ProjectState::new("/test/project".to_string());
        project.dockers.services.push(DockerServiceInfo {
            id: "test-id".to_string(),
            name: "PostgreSQL".to_string(),
            image: "postgres:16".to_string(),
            status: ServiceStatus::Running,
            port: Some(5432),
            service_type: ServiceType::Database,
        });
        project.tasks.commands.push(JustCommandInfo {
            name: "test".to_string(),
            description: Some("Run tests".to_string()),
            recipe: "cargo test".to_string(),
        });
        state.projects.push(project);

        let json = serde_json::to_string_pretty(&state).unwrap();
        println!("Serialized state:\n{}", json);

        let loaded: AppState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.projects.len(), loaded.projects.len());
        assert_eq!(
            state.projects[0].name,
            loaded.projects[0].name
        );
    }

    #[test]
    fn test_project_state_new() {
        let project = ProjectState::new("/Users/chris/my-project".to_string());
        assert_eq!(project.name, "my-project");
        assert_eq!(project.path, "/Users/chris/my-project");
        assert!(!project.is_modified);
        assert_eq!(project.active_tab, FeatureTab::Tasks);
    }

    #[test]
    fn test_active_project() {
        let mut state = AppState::default();
        assert!(state.active_project().is_none());

        state.projects.push(ProjectState::new("/test/a".to_string()));
        state.projects.push(ProjectState::new("/test/b".to_string()));

        assert_eq!(state.active_project().unwrap().name, "a");

        state.active_project_index = 1;
        assert_eq!(state.active_project().unwrap().name, "b");
    }

    #[test]
    fn test_feature_tab_serialization() {
        assert_eq!(
            serde_json::to_string(&FeatureTab::Dockers).unwrap(),
            "\"dockers\""
        );
        assert_eq!(
            serde_json::to_string(&FeatureTab::Tasks).unwrap(),
            "\"tasks\""
        );
        assert_eq!(
            serde_json::to_string(&FeatureTab::Settings).unwrap(),
            "\"settings\""
        );
    }

    #[test]
    fn test_service_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ServiceStatus::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&ServiceStatus::Stopped).unwrap(),
            "\"stopped\""
        );
    }
}
