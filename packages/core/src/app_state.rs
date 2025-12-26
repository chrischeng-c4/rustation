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
    /// Global Docker state (shared across all projects)
    #[serde(default)]
    pub docker: DockersState,
    /// App-wide notifications (toasts)
    #[serde(default)]
    pub notifications: Vec<Notification>,
    /// Currently active view
    #[serde(default)]
    pub active_view: ActiveView,
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
            docker: DockersState::default(),
            notifications: Vec::new(),
            active_view: ActiveView::default(),
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
// Project State (Git Repo)
// ============================================================================

/// State for a single open project (git repo)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectState {
    /// Unique identifier
    pub id: String,
    /// Filesystem path to the main worktree (git repo root)
    pub path: String,
    /// Display name (repo folder name)
    pub name: String,
    /// All worktrees for this project
    pub worktrees: Vec<WorktreeState>,
    /// Index of the currently active worktree
    pub active_worktree_index: usize,
    /// Environment file configuration (project-level)
    #[serde(default)]
    pub env_config: EnvConfig,
}

impl ProjectState {
    /// Create a new project from a path (with main worktree)
    pub fn new(path: String) -> Self {
        let name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();

        // Create main worktree
        let main_worktree = WorktreeState::new(path.clone(), "main".to_string(), true);

        Self {
            id: Uuid::new_v4().to_string(),
            path: path.clone(),
            name,
            worktrees: vec![main_worktree],
            active_worktree_index: 0,
            env_config: EnvConfig::with_source(path),
        }
    }

    /// Get the active worktree (if any)
    pub fn active_worktree(&self) -> Option<&WorktreeState> {
        self.worktrees.get(self.active_worktree_index)
    }

    /// Get the active worktree mutably (if any)
    pub fn active_worktree_mut(&mut self) -> Option<&mut WorktreeState> {
        self.worktrees.get_mut(self.active_worktree_index)
    }
}

// ============================================================================
// Worktree State (Git Worktree)
// ============================================================================

/// State for a single git worktree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorktreeState {
    /// Unique identifier
    pub id: String,
    /// Filesystem path to the worktree
    pub path: String,
    /// Branch name (e.g., "main", "feature/auth")
    pub branch: String,
    /// Is this the main worktree?
    pub is_main: bool,
    /// MCP server state
    pub mcp: McpState,
    /// Whether the worktree has unsaved changes or running tasks
    pub is_modified: bool,
    /// Currently active feature tab within this worktree (legacy, use AppState.active_view)
    pub active_tab: FeatureTab,
    /// Tasks state for this worktree
    pub tasks: TasksState,
    // Note: Docker state moved to AppState.docker (global scope)
}

impl WorktreeState {
    /// Create a new worktree
    pub fn new(path: String, branch: String, is_main: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            path,
            branch,
            is_main,
            mcp: McpState::default(),
            is_modified: false,
            active_tab: FeatureTab::Tasks,
            tasks: TasksState::default(),
        }
    }
}

// ============================================================================
// MCP Server State
// ============================================================================

/// MCP server status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpStatus {
    #[default]
    Stopped,
    Starting,
    Running,
    Error,
}

/// MCP server state for a worktree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct McpState {
    /// Server status
    pub status: McpStatus,
    /// Assigned port (if running)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Path to mcp-session.json config file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    /// Error message (if status is Error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Feature tabs within a project (sidebar) - legacy, prefer ActiveView
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FeatureTab {
    #[default]
    Tasks,
    Dockers,
    Settings,
}

// ============================================================================
// Active View (Three-Scope Model)
// ============================================================================

/// Currently active view in the main content area
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ActiveView {
    /// Tasks page (worktree scope)
    #[default]
    Tasks,
    /// Settings page (worktree scope)
    Settings,
    /// Docker page (global scope)
    Dockers,
    /// Env management page (project scope)
    Env,
}

// ============================================================================
// Environment Configuration (Project-level)
// ============================================================================

/// Environment file configuration for a project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvConfig {
    /// Patterns of files/folders to track for env copying
    pub tracked_patterns: Vec<String>,
    /// Automatically copy env files when creating new worktree
    pub auto_copy_enabled: bool,
    /// Default source worktree path for copying (usually main worktree)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_worktree: Option<String>,
    /// Result of the last copy operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_copy_result: Option<EnvCopyResult>,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            tracked_patterns: vec![
                ".env".to_string(),
                ".envrc".to_string(),
                ".claude/".to_string(),
                ".vscode/".to_string(),
            ],
            auto_copy_enabled: true,
            source_worktree: None,
            last_copy_result: None,
        }
    }
}

impl EnvConfig {
    /// Create with a specific source worktree
    pub fn with_source(source_path: String) -> Self {
        Self {
            source_worktree: Some(source_path),
            ..Self::default()
        }
    }
}

/// Result of an env file copy operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvCopyResult {
    /// Files that were successfully copied
    pub copied_files: Vec<String>,
    /// Files that failed to copy (path, error message)
    pub failed_files: Vec<(String, String)>,
    /// Timestamp of the operation (ISO 8601)
    pub timestamp: String,
}

// ============================================================================
// Notifications (Toasts)
// ============================================================================

/// App notification (toast message)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    /// Unique identifier
    pub id: String,
    /// Notification message
    pub message: String,
    /// Type of notification
    pub notification_type: NotificationType,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
}

impl Notification {
    /// Create a new notification
    pub fn new(message: impl Into<String>, notification_type: NotificationType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message: message.into(),
            notification_type,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Success)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Error)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Info)
    }
}

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

// ============================================================================
// From Implementations (Actions -> State)
// ============================================================================

impl From<crate::actions::ActiveViewData> for ActiveView {
    fn from(data: crate::actions::ActiveViewData) -> Self {
        match data {
            crate::actions::ActiveViewData::Tasks => ActiveView::Tasks,
            crate::actions::ActiveViewData::Settings => ActiveView::Settings,
            crate::actions::ActiveViewData::Dockers => ActiveView::Dockers,
            crate::actions::ActiveViewData::Env => ActiveView::Env,
        }
    }
}

impl From<crate::actions::NotificationTypeData> for NotificationType {
    fn from(data: crate::actions::NotificationTypeData) -> Self {
        match data {
            crate::actions::NotificationTypeData::Info => NotificationType::Info,
            crate::actions::NotificationTypeData::Success => NotificationType::Success,
            crate::actions::NotificationTypeData::Warning => NotificationType::Warning,
            crate::actions::NotificationTypeData::Error => NotificationType::Error,
        }
    }
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
    /// Pending port conflict requiring user resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_conflict: Option<PendingConflict>,
    /// Custom port overrides for services (service_id -> port)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub port_overrides: HashMap<String, u16>,
}

/// Pending port conflict that requires user resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingConflict {
    /// The service that was trying to start
    pub service_id: String,
    /// The port conflict details
    pub conflict: PortConflict,
}

/// Information about a port conflict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortConflict {
    /// The port that was requested
    pub requested_port: u16,
    /// The container currently using this port
    pub conflicting_container: ConflictingContainer,
    /// Suggested alternative port
    pub suggested_port: u16,
}

/// Information about the container causing a port conflict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictingContainer {
    /// Docker container ID
    pub id: String,
    /// Container name (e.g., "tech-platform-postgres")
    pub name: String,
    /// Container image (e.g., "postgres:15-alpine")
    pub image: String,
    /// Whether this container is managed by rstn (rstn-* prefix)
    pub is_rstn_managed: bool,
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
    /// Project group (e.g., "tech-platform", "rstn", "pg-bench")
    pub project_group: Option<String>,
    /// Whether this container is managed by rstn (rstn-* prefix)
    pub is_rstn_managed: bool,
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
        // Access through worktree for tasks
        if let Some(worktree) = project.active_worktree_mut() {
            worktree.tasks.commands.push(JustCommandInfo {
                name: "test".to_string(),
                description: Some("Run tests".to_string()),
                recipe: "cargo test".to_string(),
            });
        }
        state.projects.push(project);

        // Docker services are now global (on state.docker)
        state.docker.services.push(DockerServiceInfo {
            id: "test-id".to_string(),
            name: "PostgreSQL".to_string(),
            image: "postgres:16".to_string(),
            status: ServiceStatus::Running,
            port: Some(5432),
            service_type: ServiceType::Database,
            project_group: Some("rstn".to_string()),
            is_rstn_managed: true,
        });

        let json = serde_json::to_string_pretty(&state).unwrap();
        println!("Serialized state:\n{}", json);

        let loaded: AppState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.projects.len(), loaded.projects.len());
        assert_eq!(state.projects[0].name, loaded.projects[0].name);
        assert_eq!(state.docker.services.len(), loaded.docker.services.len());
    }

    #[test]
    fn test_project_state_new() {
        let project = ProjectState::new("/Users/chris/my-project".to_string());
        assert_eq!(project.name, "my-project");
        assert_eq!(project.path, "/Users/chris/my-project");
        // Check worktree properties
        let worktree = project.active_worktree().unwrap();
        assert!(!worktree.is_modified);
        assert_eq!(worktree.active_tab, FeatureTab::Tasks);
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
