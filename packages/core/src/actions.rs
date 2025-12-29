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

    /// Add MCP log entry (internal, from MCP server)
    AddMcpLogEntry { entry: McpLogEntryData },

    /// Clear MCP logs
    ClearMcpLogs,

    /// Update available MCP tools (internal, after fetch)
    UpdateMcpTools { tools: Vec<McpToolData> },

    // ========================================================================
    // Chat Actions (worktree scope)
    // ========================================================================
    /// Send a chat message to Claude
    SendChatMessage { text: String },

    /// Add a chat message (user or assistant)
    AddChatMessage { message: ChatMessageData },

    /// Append content to the last assistant message (streaming)
    AppendChatContent { content: String },

    /// Set chat typing/streaming status
    SetChatTyping { is_typing: bool },

    /// Set chat error
    SetChatError { error: String },

    /// Clear chat error
    ClearChatError,

    /// Clear all chat messages
    ClearChat,

    // ========================================================================
    // Constitution Workflow Actions (CESDD Phase 1)
    // ========================================================================
    /// Start the Constitution initialization workflow
    StartConstitutionWorkflow,

    /// Clear/reset the Constitution workflow (for fresh start)
    ClearConstitutionWorkflow,

    // ========================================================================
    // Change Management Actions (CESDD Phase 2)
    // ========================================================================
    /// Create a new change from user intent
    CreateChange { intent: String },

    /// Generate proposal.md using Claude (starts streaming)
    GenerateProposal { change_id: String },

    /// Append content to proposal output (streaming from Claude)
    AppendProposalOutput { change_id: String, content: String },

    /// Mark proposal generation as complete
    CompleteProposal { change_id: String },

    /// Generate plan.md using Claude (starts streaming)
    GeneratePlan { change_id: String },

    /// Append content to plan output (streaming from Claude)
    AppendPlanOutput { change_id: String, content: String },

    /// Mark plan generation as complete
    CompletePlan { change_id: String },

    /// Approve the plan and transition to Implementing status
    ApprovePlan { change_id: String },

    /// Cancel a change (sets status to Cancelled)
    CancelChange { change_id: String },

    /// Select a change to view details
    SelectChange { change_id: Option<String> },

    /// Refresh changes list from .rstn/changes/
    RefreshChanges,

    /// Set changes list (internal, after refresh)
    SetChanges { changes: Vec<ChangeData> },

    /// Set changes loading state
    SetChangesLoading { is_loading: bool },

    /// Submit an answer to the current question and advance
    AnswerConstitutionQuestion { answer: String },

    /// Generate constitution using Claude (after all questions answered)
    GenerateConstitution,

    /// Append content to constitution output (streaming from Claude)
    AppendConstitutionOutput { content: String },

    /// Save the generated constitution to .rstn/constitution.md
    SaveConstitution,

    /// Check if constitution file exists (async trigger)
    CheckConstitutionExists,

    /// Set constitution existence status (internal, after check)
    SetConstitutionExists { exists: bool },

    /// Apply default constitution template without Q&A
    ApplyDefaultConstitution,

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
    // Agent Rules Actions (Project scope)
    // ========================================================================
    /// Toggle custom agent rules on/off
    SetAgentRulesEnabled { enabled: bool },

    /// Update custom system prompt text (deprecated, use profile actions)
    SetAgentRulesPrompt { prompt: String },

    /// Set temp file path (internal, after generation)
    SetAgentRulesTempFile { path: Option<String> },

    /// Create a new agent profile
    CreateAgentProfile { name: String, prompt: String },

    /// Update an existing agent profile
    UpdateAgentProfile {
        id: String,
        name: String,
        prompt: String,
    },

    /// Delete an agent profile
    DeleteAgentProfile { id: String },

    /// Select and activate an agent profile (None = disable)
    SelectAgentProfile { profile_id: Option<String> },

    // ========================================================================
    // Notification Actions
    // ========================================================================
    /// Add a notification (toast)
    AddNotification {
        message: String,
        notification_type: NotificationTypeData,
    },

    /// Dismiss a notification (removes from list)
    DismissNotification { id: String },

    /// Mark a notification as read (keeps in history but dismisses toast)
    MarkNotificationRead { id: String },

    /// Mark all notifications as read
    MarkAllNotificationsRead,

    /// Clear all notifications
    ClearNotifications,

    // ========================================================================
    // View Actions
    // ========================================================================
    /// Set the active view in the main content area
    SetActiveView { view: ActiveViewData },

    // ========================================================================
    // Terminal Actions (worktree scope)
    // ========================================================================
    /// Spawn a new terminal session
    SpawnTerminal { cols: u16, rows: u16 },

    /// Resize an existing terminal session
    ResizeTerminal { session_id: String, cols: u16, rows: u16 },

    /// Write data to terminal (user input)
    WriteTerminal { session_id: String, data: String },

    /// Kill a terminal session
    KillTerminal { session_id: String },

    /// Set terminal session ID (internal, after spawn completes)
    SetTerminalSession { session_id: Option<String> },

    /// Set terminal dimensions (internal)
    SetTerminalSize { cols: u16, rows: u16 },

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

    // ========================================================================
    // Dev Log Actions (global scope, dev mode only)
    // ========================================================================
    /// Add a dev log entry (for debugging)
    AddDevLog { log: DevLogData },

    /// Clear all dev logs
    ClearDevLogs,
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

/// MCP log direction for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum McpLogDirectionData {
    In,
    Out,
}

/// MCP log entry for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpLogEntryData {
    pub timestamp: String,
    pub direction: McpLogDirectionData,
    pub method: String,
    pub tool_name: Option<String>,
    pub payload: String,
    pub is_error: bool,
}

/// MCP tool data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpToolData {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Chat role for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRoleData {
    User,
    Assistant,
    System,
}

/// Chat message for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessageData {
    pub id: String,
    pub role: ChatRoleData,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub is_streaming: bool,
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
    Workflows,
    Tasks,
    Settings,
    Dockers,
    Env,
    Mcp,
    Chat,
    Terminal,
    #[serde(rename = "agent_rules")]
    AgentRules,
}

/// Source of the dev log for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DevLogSourceData {
    Rust,
    Frontend,
    Claude,
    Ipc,
}

/// Type/category of the dev log for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DevLogTypeData {
    Action,
    State,
    Claude,
    Error,
    Info,
}

/// Dev log entry for actions (dev mode debugging)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DevLogData {
    /// Source of the log (rust, frontend, claude, ipc)
    pub source: DevLogSourceData,
    /// Type/category of the log
    pub log_type: DevLogTypeData,
    /// Short summary for collapsed view
    pub summary: String,
    /// Full structured data (JSON, shown when expanded)
    pub data: serde_json::Value,
}

/// Change status for actions (CESDD Phase 2)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeStatusData {
    Proposed,
    Planning,
    Planned,
    Implementing,
    Testing,
    Done,
    Archived,
    Cancelled,
    Failed,
}

/// Change data for actions (CESDD Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeData {
    pub id: String,
    pub name: String,
    pub status: ChangeStatusData,
    pub intent: String,
    pub proposal: Option<String>,
    pub plan: Option<String>,
    pub streaming_output: String,
    pub created_at: String,
    pub updated_at: String,
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

    #[test]
    fn test_change_actions_serialization() {
        // CreateChange
        let action = Action::CreateChange {
            intent: "Add user authentication".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        let loaded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, loaded);

        // GenerateProposal
        let action = Action::GenerateProposal {
            change_id: "change-123".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("GenerateProposal"));

        // SetChanges with ChangeData
        let action = Action::SetChanges {
            changes: vec![ChangeData {
                id: "change-123".to_string(),
                name: "feature-auth".to_string(),
                status: ChangeStatusData::Proposed,
                intent: "Add user authentication".to_string(),
                proposal: None,
                plan: None,
                streaming_output: String::new(),
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            }],
        };
        let json = serde_json::to_string(&action).unwrap();
        let loaded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, loaded);
    }
}
