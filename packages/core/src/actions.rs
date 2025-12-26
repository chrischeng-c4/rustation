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

    /// Set the feature tab within the active worktree
    SetFeatureTab { tab: FeatureTab },

    // ========================================================================
    // Worktree Actions
    // ========================================================================
    /// Switch to a different worktree within the active project
    SwitchWorktree { index: usize },

    /// Refresh worktrees for the active project (re-run `git worktree list`)
    RefreshWorktrees,

    /// Set worktrees (internal, after git worktree list completes)
    SetWorktrees { worktrees: Vec<WorktreeData> },

    /// Add a worktree from an existing branch
    AddWorktree { branch: String },

    /// Add a worktree with a new branch
    AddWorktreeNewBranch { branch: String },

    /// Remove a worktree (cannot remove main worktree)
    RemoveWorktree { worktree_path: String },

    // ========================================================================
    // MCP Actions
    // ========================================================================
    /// Start MCP server for the active worktree
    StartMcpServer,

    /// Stop MCP server for the active worktree
    StopMcpServer,

    /// Set MCP server status (internal)
    SetMcpStatus { status: McpStatusData },

    /// Set MCP server port (internal, after server starts)
    SetMcpPort { port: u16 },

    /// Set MCP config path (internal)
    SetMcpConfigPath { path: String },

    /// Set MCP error (internal)
    SetMcpError { error: String },

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

    /// Set a port conflict that requires user resolution
    SetPortConflict {
        service_id: String,
        conflict: PortConflictData,
    },

    /// Clear the pending port conflict (user cancelled or resolved)
    ClearPortConflict,

    /// Start a Docker service with a specific port override
    StartDockerServiceWithPort { service_id: String, port: u16 },

    /// Stop a conflicting container and start the rstn service
    ResolveConflictByStoppingContainer {
        conflicting_container_id: String,
        service_id: String,
    },

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
    // Env Actions (Project scope)
    // ========================================================================
    /// Copy env files from one worktree to another
    CopyEnvFiles {
        from_worktree_path: String,
        to_worktree_path: String,
        /// Optional patterns to copy (None = use tracked_patterns)
        patterns: Option<Vec<String>>,
    },

    /// Set the result of an env copy operation (internal)
    SetEnvCopyResult { result: EnvCopyResultData },

    /// Update tracked patterns for the active project
    SetEnvTrackedPatterns { patterns: Vec<String> },

    /// Toggle auto-copy on worktree creation
    SetEnvAutoCopy { enabled: bool },

    /// Set source worktree for env copying
    SetEnvSourceWorktree { worktree_path: Option<String> },

    // ========================================================================
    // Notification Actions
    // ========================================================================
    /// Add a notification (toast)
    AddNotification {
        message: String,
        notification_type: NotificationTypeData,
    },

    /// Dismiss a notification
    DismissNotification { id: String },

    /// Clear all notifications
    ClearNotifications,

    // ========================================================================
    // View Actions
    // ========================================================================
    /// Set the active view in the main content area
    SetActiveView { view: ActiveViewData },

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

/// Worktree data for actions (from `git worktree list`)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorktreeData {
    pub path: String,
    pub branch: String,
    pub is_main: bool,
}

/// Branch data for UI (from `git branch`)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BranchData {
    pub name: String,
    pub has_worktree: bool,
    pub is_current: bool,
}

/// MCP status for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum McpStatusData {
    Stopped,
    Starting,
    Running,
    Error,
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
    pub project_group: Option<String>,
    pub is_rstn_managed: bool,
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

/// Port conflict data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortConflictData {
    pub requested_port: u16,
    pub conflicting_container: ConflictingContainerData,
    pub suggested_port: u16,
}

/// Conflicting container data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictingContainerData {
    pub id: String,
    pub name: String,
    pub image: String,
    pub is_rstn_managed: bool,
}

/// Env copy result data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvCopyResultData {
    /// Files that were successfully copied
    pub copied_files: Vec<String>,
    /// Files that failed to copy (path, error)
    pub failed_files: Vec<(String, String)>,
    /// Timestamp of the operation (ISO 8601)
    pub timestamp: String,
}

/// Notification type for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationTypeData {
    Info,
    Success,
    Warning,
    Error,
}

/// Active view for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ActiveViewData {
    Tasks,
    Settings,
    Dockers,
    Env,
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
                project_group: Some("rstn".to_string()),
                is_rstn_managed: true,
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
