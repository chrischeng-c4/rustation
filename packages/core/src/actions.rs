//! Actions for state mutations.
//!
//! All state changes go through dispatch(action) -> reducer -> new state.
//! Actions are serializable for logging, debugging, and replay.

use crate::app_state::{FeatureTab, Theme};
use serde::{Deserialize, Serialize};

/// All possible actions that can mutate application state.
///
/// Actions follow the pattern: `{ type: "ActionName", payload: { ... } }`
/// when serialized to JSON for IPC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum Action {
    // ========================================================================
    // Project Management
    // ========================================================================
    /// Open a project folder
    OpenProject { path: String },

    /// Close a project tab
    CloseProject { index: usize },

    /// Switch to a different project tab
    SwitchProject { index: usize },

    /// Set the feature tab within the active project
    SetFeatureTab { tab: FeatureTab },

    // ========================================================================
    // Docker Actions
    // ========================================================================
    /// Check if Docker is available on this system
    CheckDockerAvailability,

    /// Set Docker availability status (internal, after check completes)
    SetDockerAvailable { available: bool },

    /// Refresh the list of Docker services
    RefreshDockerServices,

    /// Set the services list (internal, after refresh completes)
    SetDockerServices { services: Vec<DockerServiceData> },

    /// Start a Docker service
    StartDockerService { service_id: String },

    /// Stop a Docker service
    StopDockerService { service_id: String },

    /// Restart a Docker service
    RestartDockerService { service_id: String },

    /// Select a service to view details/logs
    SelectDockerService { service_id: Option<String> },

    /// Fetch logs for a service
    FetchDockerLogs { service_id: String, tail: u32 },

    /// Set logs (internal, after fetch completes)
    SetDockerLogs { logs: Vec<String> },

    /// Create a database in a database container
    CreateDatabase { service_id: String, db_name: String },

    /// Create a vhost in RabbitMQ
    CreateVhost { service_id: String, vhost_name: String },

    /// Set loading state for Docker operations
    SetDockerLoading { is_loading: bool },

    /// Set loading state for logs
    SetDockerLogsLoading { is_loading: bool },

    // ========================================================================
    // Tasks Actions
    // ========================================================================
    /// Load justfile commands from a path
    LoadJustfileCommands { path: String },

    /// Set commands (internal, after load completes)
    SetJustfileCommands { commands: Vec<JustCommandData> },

    /// Run a just command
    RunJustCommand { name: String, cwd: String },

    /// Set task status
    SetTaskStatus { name: String, status: TaskStatusData },

    /// Set active command
    SetActiveCommand { name: Option<String> },

    /// Append output line
    AppendTaskOutput { line: String },

    /// Clear task output
    ClearTaskOutput,

    /// Set tasks loading state
    SetTasksLoading { is_loading: bool },

    /// Set tasks error
    SetTasksError { error: Option<String> },

    // ========================================================================
    // Settings Actions
    // ========================================================================
    /// Set UI theme
    SetTheme { theme: Theme },

    /// Set default project path
    SetProjectPath { path: Option<String> },

    // ========================================================================
    // Error Handling
    // ========================================================================
    /// Set a global error
    SetError {
        code: String,
        message: String,
        context: Option<String>,
    },

    /// Clear the global error
    ClearError,
}

/// Docker service data for actions (lightweight, serializable)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DockerServiceData {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub port: Option<u32>,
    pub service_type: String,
}

/// Just command data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JustCommandData {
    pub name: String,
    pub description: Option<String>,
    pub recipe: String,
}

/// Task status for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatusData {
    Idle,
    Running,
    Success,
    Error,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_serialization_simple() {
        let action = Action::SetFeatureTab {
            tab: FeatureTab::Dockers,
        };
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(
            json,
            r#"{"type":"SetFeatureTab","payload":{"tab":"dockers"}}"#
        );

        let loaded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, loaded);
    }

    #[test]
    fn test_project_actions_serialization() {
        let action = Action::OpenProject {
            path: "/home/user/project".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        let loaded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, loaded);

        let action = Action::SwitchProject { index: 2 };
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, r#"{"type":"SwitchProject","payload":{"index":2}}"#);
    }

    #[test]
    fn test_action_serialization_no_payload() {
        let action = Action::CheckDockerAvailability;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, r#"{"type":"CheckDockerAvailability"}"#);

        let loaded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, loaded);
    }

    #[test]
    fn test_action_serialization_with_data() {
        let action = Action::SetDockerServices {
            services: vec![DockerServiceData {
                id: "abc123".to_string(),
                name: "PostgreSQL".to_string(),
                image: "postgres:16".to_string(),
                status: "running".to_string(),
                port: Some(5432),
                service_type: "Database".to_string(),
            }],
        };
        let json = serde_json::to_string_pretty(&action).unwrap();
        println!("Serialized action:\n{}", json);

        let loaded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, loaded);
    }

    #[test]
    fn test_action_from_frontend_json() {
        // Simulate what frontend would send
        let frontend_json = r#"{
            "type": "StartDockerService",
            "payload": {
                "service_id": "rstn-postgres"
            }
        }"#;

        let action: Action = serde_json::from_str(frontend_json).unwrap();
        assert!(matches!(action, Action::StartDockerService { service_id } if service_id == "rstn-postgres"));
    }
}
