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
    /// Dev logs for debugging (dev mode only, right panel)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dev_logs: Vec<DevLog>,
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
            dev_logs: Vec::new(),
        }
    }
}

/// Maximum number of dev log entries to keep
const MAX_DEV_LOGS: usize = 200;

impl AppState {
    /// Get the active project (if any)
    pub fn active_project(&self) -> Option<&ProjectState> {
        self.projects.get(self.active_project_index)
    }

    /// Get the active project mutably (if any)
    pub fn active_project_mut(&mut self) -> Option<&mut ProjectState> {
        self.projects.get_mut(self.active_project_index)
    }

    /// Add a dev log entry, keeping only the most recent MAX_DEV_LOGS
    pub fn add_dev_log(&mut self, log: DevLog) {
        self.dev_logs.push(log);
        if self.dev_logs.len() > MAX_DEV_LOGS {
            self.dev_logs.remove(0);
        }
    }

    /// Clear all dev logs
    pub fn clear_dev_logs(&mut self) {
        self.dev_logs.clear();
    }
}

// ============================================================================
// Dev Logs (Development Mode Only)
// ============================================================================

/// Source of the log entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DevLogSource {
    /// From Rust backend (reducer, async handlers)
    Rust,
    /// From React frontend
    Frontend,
    /// From Claude CLI
    Claude,
    /// From IPC layer
    Ipc,
}

/// Type/category of the log entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DevLogType {
    /// State action dispatched
    Action,
    /// State change
    State,
    /// Claude CLI output
    Claude,
    /// Error occurred
    Error,
    /// Informational
    Info,
}

/// Development log entry for debugging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DevLog {
    /// Unique identifier
    pub id: String,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Source of the log
    pub source: DevLogSource,
    /// Type/category
    pub log_type: DevLogType,
    /// Short summary for collapsed view (most important info)
    pub summary: String,
    /// Full structured data (JSON, shown when expanded)
    pub data: serde_json::Value,
}

impl DevLog {
    /// Create a new dev log entry
    pub fn new(
        source: DevLogSource,
        log_type: DevLogType,
        summary: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source,
            log_type,
            summary: summary.into(),
            data,
        }
    }

    /// Create an action log
    pub fn action(action_name: &str, payload: serde_json::Value) -> Self {
        Self::new(
            DevLogSource::Rust,
            DevLogType::Action,
            format!("Action: {}", action_name),
            payload,
        )
    }

    /// Create a state change log
    pub fn state_change(description: &str, details: serde_json::Value) -> Self {
        Self::new(
            DevLogSource::Rust,
            DevLogType::State,
            description.to_string(),
            details,
        )
    }

    /// Create an error log
    pub fn error(message: &str, details: serde_json::Value) -> Self {
        Self::new(
            DevLogSource::Rust,
            DevLogType::Error,
            format!("Error: {}", message),
            details,
        )
    }

    /// Create a Claude output log
    pub fn claude(summary: &str, output: serde_json::Value) -> Self {
        Self::new(
            DevLogSource::Claude,
            DevLogType::Claude,
            summary.to_string(),
            output,
        )
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
    /// Agent rules configuration (project-level)
    #[serde(default)]
    pub agent_rules_config: AgentRulesConfig,
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
            agent_rules_config: AgentRulesConfig::default(),
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
    /// Chat state for Claude assistant
    pub chat: ChatState,
    /// Terminal state for integrated PTY
    #[serde(default)]
    pub terminal: crate::terminal::TerminalState,
    /// Whether the worktree has unsaved changes or running tasks
    pub is_modified: bool,
    /// Currently active feature tab within this worktree (legacy, use AppState.active_view)
    pub active_tab: FeatureTab,
    /// Tasks state for this worktree
    pub tasks: TasksState,
    /// Changes state for CESDD Phase 2
    #[serde(default)]
    pub changes: ChangesState,
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
            chat: ChatState::default(),
            terminal: crate::terminal::TerminalState::new(),
            is_modified: false,
            active_tab: FeatureTab::Tasks,
            tasks: TasksState::default(),
            changes: ChangesState::default(),
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

/// Direction of MCP log entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpLogDirection {
    /// Incoming request from client
    In,
    /// Outgoing response to client
    Out,
}

/// MCP log entry for traffic inspection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpLogEntry {
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Direction (in/out)
    pub direction: McpLogDirection,
    /// Method name (e.g., "tools/call", "tools/list")
    pub method: String,
    /// Tool name (for tool calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Payload summary (truncated for large data)
    pub payload: String,
    /// Whether this was an error
    pub is_error: bool,
}

/// Maximum number of log entries to keep
const MAX_MCP_LOG_ENTRIES: usize = 100;

/// MCP tool information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpTool {
    /// Tool name (e.g. "read_file")
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema (JSON Schema format)
    pub input_schema: serde_json::Value,
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
    /// Recent log entries (limited to MAX_MCP_LOG_ENTRIES)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub log_entries: Vec<McpLogEntry>,
    /// Available MCP tools (from tools/list)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub available_tools: Vec<McpTool>,
}

impl McpState {
    /// Add a log entry, keeping only the most recent MAX_MCP_LOG_ENTRIES
    pub fn add_log_entry(&mut self, entry: McpLogEntry) {
        self.log_entries.push(entry);
        if self.log_entries.len() > MAX_MCP_LOG_ENTRIES {
            self.log_entries.remove(0);
        }
    }

    /// Clear all log entries
    pub fn clear_logs(&mut self) {
        self.log_entries.clear();
    }
}

// ============================================================================
// Chat State (Worktree-level)
// ============================================================================

/// Role of a chat message participant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

/// A single chat message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// Unique message ID
    pub id: String,
    /// Role (user, assistant, system)
    pub role: ChatRole,
    /// Message content (may include markdown)
    pub content: String,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Whether this message is still streaming
    #[serde(default)]
    pub is_streaming: bool,
}

/// Maximum number of chat messages to keep
const MAX_CHAT_MESSAGES: usize = 100;

/// Chat state for a worktree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatState {
    /// Chat messages
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    /// Whether the assistant is currently typing/streaming
    #[serde(default)]
    pub is_typing: bool,
    /// Error message (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            is_typing: false,
            error: None,
        }
    }
}

impl ChatState {
    /// Add a message, keeping only the most recent MAX_CHAT_MESSAGES
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        if self.messages.len() > MAX_CHAT_MESSAGES {
            self.messages.remove(0);
        }
    }

    /// Append content to the last assistant message (for streaming)
    pub fn append_to_last(&mut self, content: &str) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == ChatRole::Assistant {
                last.content.push_str(content);
            }
        }
    }

    /// Mark the last message as done streaming
    pub fn finish_streaming(&mut self) {
        if let Some(last) = self.messages.last_mut() {
            last.is_streaming = false;
        }
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.error = None;
    }
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
    /// Workflows page - guided, stateful workflows (worktree scope)
    #[default]
    Workflows,
    /// Tasks page - simple justfile commands (worktree scope)
    Tasks,
    /// Settings page (worktree scope)
    Settings,
    /// Docker page (global scope)
    Dockers,
    /// Env management page (project scope)
    Env,
    /// Agent Rules management page (project scope)
    #[serde(rename = "agent_rules")]
    AgentRules,
    /// MCP Inspector page (worktree scope)
    Mcp,
    /// Chat assistant page (worktree scope)
    Chat,
    /// Terminal page (worktree scope)
    Terminal,
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
// Agent Rules Configuration (Project-level)
// ============================================================================

/// Agent profile with custom system prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentProfile {
    /// Unique identifier (UUID)
    pub id: String,
    /// Display name (e.g. "Rust Expert", "Code Reviewer")
    pub name: String,
    /// System prompt content
    pub prompt: String,
    /// Whether this is a built-in (immutable) profile
    pub is_builtin: bool,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last update timestamp (ISO 8601)
    pub updated_at: String,
}

impl AgentProfile {
    /// Create the default Rust Expert profile
    pub fn default_rust_expert() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: "builtin-rust-expert".to_string(),
            name: "Rust Expert".to_string(),
            prompt: r#"You are a Rust programming expert.

Core Principles:
- Always use snake_case for function and variable names
- Prefer Result<T, E> over Option when errors are possible
- Write comprehensive tests for all new functions
- Use #[derive(Debug, Clone)] by default for structs
- Avoid unwrap() in production code - use proper error handling

Code Style:
- Maximum line length: 100 characters
- Use rustfmt and clippy
- Add doc comments for public APIs
- Use meaningful variable names"#.to_string(),
            is_builtin: true,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Create the default TypeScript Expert profile
    pub fn default_typescript_expert() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: "builtin-typescript-expert".to_string(),
            name: "TypeScript Expert".to_string(),
            prompt: r#"You are a TypeScript/React programming expert.

Core Principles:
- Always use TypeScript with strict mode
- Prefer functional components with hooks
- Use proper typing - avoid 'any'
- Follow React best practices
- Use meaningful component and variable names

Code Style:
- Use ESLint and Prettier
- Maximum line length: 100 characters
- Use const for immutable values
- Prefer async/await over .then()
- Add JSDoc comments for complex functions"#.to_string(),
            is_builtin: true,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Create the default Code Reviewer profile
    pub fn default_code_reviewer() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: "builtin-code-reviewer".to_string(),
            name: "Code Reviewer".to_string(),
            prompt: r#"You are a code reviewer focused on quality and best practices.

Review Focus:
- Check for potential bugs and edge cases
- Verify proper error handling
- Look for security vulnerabilities
- Check code readability and maintainability
- Suggest performance improvements
- Verify test coverage

Feedback Style:
- Be constructive and specific
- Provide examples when suggesting changes
- Explain the reasoning behind suggestions
- Prioritize critical issues over style preferences"#.to_string(),
            is_builtin: true,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Agent rules configuration for customizing Claude's system prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentRulesConfig {
    /// Whether custom agent rules are enabled
    pub enabled: bool,
    /// Currently active profile ID (None = use CLAUDE.md)
    pub active_profile_id: Option<String>,
    /// All available profiles (built-in + custom)
    pub profiles: Vec<AgentProfile>,
    /// Generated temp file path (internal, for cleanup)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_file_path: Option<String>,
}

impl Default for AgentRulesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active_profile_id: None,
            profiles: vec![
                AgentProfile::default_rust_expert(),
                AgentProfile::default_typescript_expert(),
                AgentProfile::default_code_reviewer(),
            ],
            temp_file_path: None,
        }
    }
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
    /// Whether the notification has been read/dismissed from toast
    #[serde(default)]
    pub read: bool,
}

impl Notification {
    /// Create a new notification
    pub fn new(message: impl Into<String>, notification_type: NotificationType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message: message.into(),
            notification_type,
            created_at: chrono::Utc::now().to_rfc3339(),
            read: false,
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
            crate::actions::ActiveViewData::Workflows => ActiveView::Workflows,
            crate::actions::ActiveViewData::Tasks => ActiveView::Tasks,
            crate::actions::ActiveViewData::Settings => ActiveView::Settings,
            crate::actions::ActiveViewData::Dockers => ActiveView::Dockers,
            crate::actions::ActiveViewData::Env => ActiveView::Env,
            crate::actions::ActiveViewData::Mcp => ActiveView::Mcp,
            crate::actions::ActiveViewData::Chat => ActiveView::Chat,
            crate::actions::ActiveViewData::Terminal => ActiveView::Terminal,
            crate::actions::ActiveViewData::AgentRules => ActiveView::AgentRules,
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

impl From<crate::actions::ChangeStatusData> for ChangeStatus {
    fn from(data: crate::actions::ChangeStatusData) -> Self {
        match data {
            crate::actions::ChangeStatusData::Proposed => ChangeStatus::Proposed,
            crate::actions::ChangeStatusData::Planning => ChangeStatus::Planning,
            crate::actions::ChangeStatusData::Planned => ChangeStatus::Planned,
            crate::actions::ChangeStatusData::Implementing => ChangeStatus::Implementing,
            crate::actions::ChangeStatusData::Testing => ChangeStatus::Testing,
            crate::actions::ChangeStatusData::Done => ChangeStatus::Done,
            crate::actions::ChangeStatusData::Archived => ChangeStatus::Archived,
            crate::actions::ChangeStatusData::Cancelled => ChangeStatus::Cancelled,
            crate::actions::ChangeStatusData::Failed => ChangeStatus::Failed,
        }
    }
}

impl From<crate::actions::ChangeData> for Change {
    fn from(data: crate::actions::ChangeData) -> Self {
        Change {
            id: data.id,
            name: data.name,
            status: data.status.into(),
            intent: data.intent,
            proposal: data.proposal,
            plan: data.plan,
            streaming_output: data.streaming_output,
            created_at: data.created_at,
            updated_at: data.updated_at,
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
    /// Constitution workflow state (CESDD Phase 1)
    #[serde(default)]
    pub constitution_workflow: Option<ConstitutionWorkflow>,
    /// Whether .rstn/constitution.md exists (None = not checked yet)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constitution_exists: Option<bool>,
}

/// Constitution workflow status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStatus {
    /// Collecting user answers to guided questions
    Collecting,
    /// Claude is generating the constitution
    Generating,
    /// Generation complete, showing result
    Complete,
}

/// Constitution initialization workflow state (CESDD Phase 1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstitutionWorkflow {
    /// Current question index (0-based)
    pub current_question: usize,
    /// User answers so far (question_key -> answer)
    pub answers: std::collections::HashMap<String, String>,
    /// Generated constitution content (streamed from Claude)
    pub output: String,
    /// Current workflow status
    pub status: WorkflowStatus,
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

// ============================================================================
// Changes State (CESDD Phase 2)
// ============================================================================

/// State for managing Changes (CESDD Transactional Layer)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ChangesState {
    /// Active changes in .rstn/changes/
    pub changes: Vec<Change>,
    /// Currently selected change for detail view
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_change_id: Option<String>,
    /// Whether changes are being loaded
    pub is_loading: bool,
}

/// A single Change (feature, bugfix, refactor, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Change {
    /// Unique identifier (e.g., "feature-auth", "bugfix-login")
    pub id: String,
    /// Human-readable title derived from intent
    pub name: String,
    /// Current status in the workflow
    pub status: ChangeStatus,
    /// User's original intent (input)
    pub intent: String,
    /// Generated proposal content (from proposal.md)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposal: Option<String>,
    /// Generated plan content (from plan.md)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    /// Streaming output (during generation)
    #[serde(default)]
    pub streaming_output: String,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last update timestamp (ISO 8601)
    pub updated_at: String,
}

/// Change status in CESDD workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChangeStatus {
    /// Initial state - proposal.md created
    #[default]
    Proposed,
    /// Claude is generating plan.md (streaming)
    Planning,
    /// plan.md complete, waiting for approval
    Planned,
    /// User approved, implementation in progress
    Implementing,
    /// Implementation complete, testing
    Testing,
    /// All done, ready for archival
    Done,
    /// Archived to Living Context
    Archived,
    /// User cancelled
    Cancelled,
    /// Build/test errors
    Failed,
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
