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

    /// Fetch all branches for the active project
    FetchBranches,

    /// Set branches (internal, after list_branches completes)
    SetBranches { branches: Vec<BranchData> },

    /// Set branches loading state
    SetBranchesLoading { is_loading: bool },

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

    /// Execute the plan using Claude Code (CESDD Phase 5)
    ExecutePlan { change_id: String },

    /// Append content to implementation output (streaming from Claude)
    AppendImplementationOutput { change_id: String, content: String },

    /// Mark implementation as complete (success)
    CompleteImplementation { change_id: String },

    /// Mark implementation as failed
    FailImplementation { change_id: String, error: String },

    /// Cancel a change (sets status to Cancelled)
    CancelChange { change_id: String },

    /// Select a change to view details
    SelectChange { change_id: Option<String> },

    /// Refresh changes list from .rstn/changes/
    RefreshChanges,

    /// Set changes list (internal, after refresh)
    SetChanges { changes: Vec<ChangeData> },

    /// Start a ReviewGate review for a proposal (CESDD ReviewGate integration)
    StartProposalReview { change_id: String },

    /// Start a ReviewGate review for a plan (CESDD ReviewGate integration)
    StartPlanReview { change_id: String },

    /// Set changes loading state
    SetChangesLoading { is_loading: bool },

    /// Add a source file to change context for Claude injection
    AddContextFile { change_id: String, path: String },

    /// Remove a source file from change context
    RemoveContextFile { change_id: String, path: String },

    /// Clear all context files for a change
    ClearContextFiles { change_id: String },

    /// Validate a context file path
    ValidateContextFile { path: String },

    /// Set context validation result (internal)
    SetContextValidationResult { result: ValidationResultData },

    // ========================================================================
    // Living Context Actions (CESDD Phase 3)
    // ========================================================================
    /// Load context files from .rstn/context/
    LoadContext,

    /// Set context files (internal, after load)
    SetContext { files: Vec<ContextFileData> },

    /// Set context loading state
    SetContextLoading { is_loading: bool },

    /// Initialize context directory with default template files
    InitializeContext,

    /// Refresh context from disk (re-read all files)
    RefreshContext,

    /// Update a single context file content (for auto-curation)
    UpdateContextFile {
        name: String,
        content: String,
    },

    /// Check if context directory exists
    CheckContextExists,

    /// Set context initialization status (internal)
    SetContextInitialized { initialized: bool },

    // ========================================================================
    // Context Generation Actions (AI-powered)
    // ========================================================================
    /// Generate context files by analyzing codebase with AI
    GenerateContext,

    /// Append content to context generation output (streaming from Claude)
    AppendGenerateContextOutput { content: String },

    /// Mark context generation as complete
    CompleteGenerateContext,

    /// Mark context generation as failed
    FailGenerateContext { error: String },

    // ========================================================================
    // Context Sync & Archive Actions (CESDD Phase 4)
    // ========================================================================
    /// Archive a completed change to .rstn/archive/
    ArchiveChange { change_id: String },

    /// Sync context: extract valuable info from proposal/plan and update context files
    SyncContext { change_id: String },

    /// Append content to context sync output (streaming from Claude)
    AppendContextSyncOutput { change_id: String, content: String },

    /// Mark context sync as complete
    CompleteContextSync { change_id: String },

    /// Set change status to Archived (internal, after archive completes)
    SetChangeArchived { change_id: String },

    /// Submit an answer to the current question and advance
    AnswerConstitutionQuestion { answer: String },

    /// Generate constitution using Claude (after all questions answered)
    GenerateConstitution,

    /// Append content to constitution output (streaming from Claude)
    AppendConstitutionOutput { content: String },

    /// Save the generated constitution to .rstn/constitutions/custom.md
    SaveConstitution,

    /// Set constitution generation error (internal, called when Claude fails)
    SetConstitutionError { error: String },

    /// Check if constitution file exists (async trigger)
    CheckConstitutionExists,

    /// Set constitution existence status (internal, after check)
    SetConstitutionExists { exists: bool },

    /// Apply default constitution template without Q&A
    ApplyDefaultConstitution,

    /// Read constitution content (async trigger)
    ReadConstitution,

    /// Set constitution content (internal, after read)
    SetConstitutionContent { content: Option<String> },

    /// Set CLAUDE.md existence status (internal, after check)
    SetClaudeMdExists { exists: bool },

    /// Read CLAUDE.md content for preview (async trigger)
    ReadClaudeMd,

    /// Set CLAUDE.md content (internal, after read)
    SetClaudeMdContent { content: Option<String> },

    /// Import CLAUDE.md to .rstn/constitutions/claude.md (async)
    ImportClaudeMd,

    /// Skip importing CLAUDE.md, show normal init flow
    SkipClaudeMdImport,

    /// Set whether to reference CLAUDE.md during constitution generation
    SetUseClaudeMdReference { use_reference: bool },

    // ========================================================================
    // Constitution Mode & Presets Actions (integrated from Agent Rules)
    // ========================================================================
    /// Set constitution mode (Rules or Presets)
    SetConstitutionMode { mode: ConstitutionModeData },

    /// Select and activate a constitution preset (None = deactivate)
    SelectConstitutionPreset { preset_id: Option<String> },

    /// Create a new custom constitution preset
    CreateConstitutionPreset { name: String, prompt: String },

    /// Update an existing constitution preset
    UpdateConstitutionPreset {
        id: String,
        name: String,
        prompt: String,
    },

    /// Delete a custom constitution preset (cannot delete built-in presets)
    DeleteConstitutionPreset { id: String },

    /// Set temp file path for preset (internal, after generation)
    SetConstitutionPresetTempFile { path: Option<String> },

    // ========================================================================
    // ReviewGate Actions (CESDD ReviewGate Layer)
    // ========================================================================
    /// Start a new review session for workflow output
    StartReview {
        workflow_node_id: String,
        content: ReviewContentData,
        policy: ReviewPolicyData,
    },

    /// Add a comment to a review session
    AddReviewComment {
        session_id: String,
        target: CommentTargetData,
        content: String,
    },

    /// Mark a comment as resolved
    ResolveReviewComment {
        session_id: String,
        comment_id: String,
    },

    /// Submit all unresolved feedback to Claude for iteration
    SubmitReviewFeedback { session_id: String },

    /// Approve the review content
    ApproveReview { session_id: String },

    /// Reject the review content
    RejectReview { session_id: String, reason: String },

    /// Update review content after Claude iteration (internal)
    UpdateReviewContent {
        session_id: String,
        content: ReviewContentData,
    },

    /// Set review session status (internal)
    SetReviewStatus {
        session_id: String,
        status: ReviewStatusData,
    },

    /// Set ReviewGate loading state (internal)
    SetReviewGateLoading { is_loading: bool },

    /// Set ReviewGate error (internal)
    SetReviewGateError { error: Option<String> },

    /// Set active review session (internal)
    SetActiveReviewSession { session_id: Option<String> },

    /// Clear a review session (internal, after complete)
    ClearReviewSession { session_id: String },

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

    /// Set the connection string result (internal, after CreateDatabase/CreateVhost)
    SetDockerConnectionString { connection_string: Option<String> },

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
    /// Load justfile commands for the active worktree
    LoadJustfileCommands,

    /// Refresh justfile commands
    RefreshJustfile,

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

    // ========================================================================
    // UI Layout Actions (Right Icon Bar & Log Panels)
    // ========================================================================
    /// Toggle a log panel (expand if collapsed, collapse if same panel clicked)
    ToggleLogPanel { panel_type: LogPanelTypeData },

    /// Close the active log panel
    CloseLogPanel,

    /// Set panel width (user resizes)
    SetLogPanelWidth { width: u32 },

    // ========================================================================
    // File Explorer Actions (Worktree scope)
    // ========================================================================
    /// Load directory content (async)
    ExploreDir { path: String },

    /// Set file entries (internal, after load)
    SetExplorerEntries {
        path: String,
        entries: Vec<FileEntryData>,
    },

    /// Set directory cache entries (internal, after load for tree view)
    SetDirectoryCache {
        path: String,
        entries: Vec<FileEntryData>,
    },

    /// Set comments for selected file (internal)
    SetFileComments {
        path: String,
        comments: Vec<CommentData>,
    },

    /// Navigate back in history
    NavigateBack,

    /// Navigate forward in history
    NavigateForward,

    /// Go to parent directory
    NavigateUp,

    /// Select a file or directory (show details/preview)
    SelectFile { path: Option<String> },

    /// Set sorting preferences
    SetExplorerSort {
        field: SortFieldData,
        direction: SortDirectionData,
    },

    /// Set filter query
    SetExplorerFilter { query: String },

    /// Expand a directory in the tree view (loads contents if not cached)
    ExpandDirectory { path: String },

    /// Collapse a directory in the tree view (hides children)
    CollapseDirectory { path: String },

    /// Create new file or directory
    CreateFile {
        path: String,
        kind: FileKindData,
    },

    /// Rename file or directory
    RenameFile {
        old_path: String,
        new_name: String,
    },

    /// Delete file or directory (move to trash)
    DeleteFile { path: String },

    /// Reveal file in OS file explorer (Finder/Explorer)
    RevealInOS { path: String },

    /// Add a comment to a file (with optional line number for inline comments)
    AddFileComment {
        path: String,
        content: String,
        /// Line number for inline comments (None for file-level comments)
        line_number: Option<usize>,
    },

    /// Delete a comment from a file
    DeleteFileComment {
        path: String,
        comment_id: String,
    },

    // ========================================================================
    // Tab Management Actions (VSCode-style preview tabs)
    // ========================================================================
    /// Open a file in a tab (single-click behavior - opens as preview tab)
    /// Preview tabs are shown in italic and get replaced by next preview
    OpenFileTab { path: String },

    /// Pin the current tab (double-click behavior - converts preview to pinned)
    /// Pinned tabs are shown in normal font and persist until explicitly closed
    PinTab { path: String },

    /// Close a tab by path
    CloseTab { path: String },

    /// Switch to a different tab (activate it)
    SwitchTab { path: String },

    // ========================================================================
    // File Viewer Actions
    // ========================================================================
    /// Read a file for viewing
    ReadFile { path: String },

    /// Set the content of the file viewer
    SetFileContent {
        path: String,
        content: Option<String>,
        error: Option<String>,
    },

    /// Set file viewer loading state
    SetFileLoading { is_loading: bool },

    /// Read a binary file for viewing (images, PDFs, videos, etc.)
    ReadBinaryFile { path: String },

    /// Set binary file content
    SetBinaryFileContent {
        path: String,
        content: Option<Vec<u8>>,
        error: Option<String>,
    },

    // ========================================================================
    // A2UI Actions (Experimental)
    // ========================================================================
    /// Set the A2UI payload for dynamic rendering
    SetA2UIPayload { payload: Option<serde_json::Value> },
}

/// File kind for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileKindData {
    File,
    Directory,
    Symlink,
}

/// File entry data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileEntryData {
    pub name: String,
    pub path: String,
    pub kind: FileKindData,
    pub size: u64,
    pub permissions: String,
    pub updated_at: String,
    pub comment_count: usize,
    pub git_status: Option<GitFileStatusData>,
}

/// Comment data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentData {
    pub id: String,
    pub content: String,
    pub author: String,
    pub created_at: String,
    /// Line number for inline comments (None for file-level comments)
    pub line_number: Option<usize>,
}

/// Git file status for actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GitFileStatusData {
    Modified,
    Added,
    Deleted,
    Untracked,
    Ignored,
    Clean,
}

/// Sort field for explorer
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortFieldData {
    Name,
    Size,
    Date,
    Kind,
}

/// Sort direction for explorer
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirectionData {
    Asc,
    Desc,
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
    Explorer,
    ClaudeCode,
    A2UI,
}

/// Constitution mode for actions (Rules = modular, Presets = full prompt)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConstitutionModeData {
    /// Modular rules from .rstn/constitutions/*.md (context-aware)
    Rules,
    /// Single preset replaces entire system prompt
    Presets,
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
    /// ReviewGate session ID for proposal review
    pub proposal_review_session_id: Option<String>,
    /// ReviewGate session ID for plan review
    pub plan_review_session_id: Option<String>,
    /// Source files selected for context injection
    #[serde(default)]
    pub context_files: Vec<String>,
}

/// Context type for actions (CESDD Phase 3)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ContextTypeData {
    Product,
    TechStack,
    Architecture,
    ApiContracts,
    DataModels,
    RecentChanges,
    Custom,
}

/// Context file data for actions (CESDD Phase 3)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextFileData {
    pub name: String,
    pub path: String,
    pub content: String,
    pub context_type: ContextTypeData,
    pub last_updated: String,
    pub token_estimate: u32,
}

/// Validation result data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "message")]
pub enum ValidationResultData {
    Valid,
    Error(String),
}

// ============================================================================
// ReviewGate Data Types
// ============================================================================

/// Review policy data for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReviewPolicyData {
    AutoApprove,
    AgentDecides,
    AlwaysReview,
}

/// Content type data for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReviewContentTypeData {
    Plan,
    Proposal,
    Code,
    Artifact,
}

/// File action data for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewFileActionData {
    Create,
    Modify,
    Delete,
}

/// Review status data for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewStatusData {
    Pending,
    Reviewing,
    Iterating,
    Approved,
    Rejected,
}

/// Comment author data for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentAuthorData {
    User,
    System,
}

/// Comment target data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CommentTargetData {
    Document,
    Section { id: String },
    File { path: String },
}

/// File change data for review content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewFileChangeData {
    pub path: String,
    pub action: ReviewFileActionData,
    pub summary: String,
}

/// Review content data for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewContentData {
    pub content_type: ReviewContentTypeData,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_changes: Vec<ReviewFileChangeData>,
}

// ============================================================================
// UI Layout Data Types
// ============================================================================

/// Log panel type for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogPanelTypeData {
    Actions,
    Errors,
    Info,
    Debug,
    Metrics,
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
                proposal_review_session_id: None,
                plan_review_session_id: None,
                context_files: vec![],
            }],
        };
        let json = serde_json::to_string(&action).unwrap();
        let loaded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, loaded);
    }
}
