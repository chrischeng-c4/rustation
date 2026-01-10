/**
 * Application state types (matching Rust AppState).
 *
 * These types mirror the Rust state structs in packages/core/src/app_state.rs
 * IMPORTANT: Keep these in sync with Rust types!
 */

// ============================================================================
// Navigation & Views (Three-Scope Model)
// ============================================================================

export type FeatureTab = 'tasks' | 'dockers' | 'settings'

/**
 * Active view in the main content area.
 * Maps to scope levels:
 * - tasks, settings, mcp, chat, terminal, workflows: Worktree scope
 * - dockers: Global scope
 * - env: Project scope
 */
export type ActiveView = 'tasks' | 'explorer' | 'settings' | 'dockers' | 'env' | 'mcp' | 'chat' | 'terminal' | 'workflows' | 'claude-code' | 'a2ui'

// ============================================================================
// File Explorer Types
// ============================================================================

export type FileKind = 'file' | 'directory' | 'symlink'

export type GitFileStatus =
  | 'modified'
  | 'added'
  | 'deleted'
  | 'untracked'
  | 'ignored'
  | 'clean'

export interface FileEntry {
  name: string
  path: string
  kind: FileKind
  size: number
  permissions: string
  updated_at: string
  comment_count: number
  git_status?: GitFileStatus
}

export interface Comment {
  id: string
  content: string
  author: string
  created_at: string
  /** Line number for inline comments (null for file-level comments) */
  line_number: number | null
}

export type SortField = 'name' | 'size' | 'date' | 'kind'
export type SortDirection = 'asc' | 'desc'

export interface SortConfig {
  field: SortField
  direction: SortDirection
}

export interface NavigationHistory {
  back_stack: string[]
  forward_stack: string[]
}

/** A single file tab in the explorer (VSCode-style) */
export interface FileTab {
  path: string
  is_pinned: boolean
}

export interface FileExplorerState {
  current_path: string
  entries: FileEntry[]
  selected_path?: string
  selected_comments: Comment[]
  sort_config: SortConfig
  filter_query: string
  history: NavigationHistory
  is_loading: boolean
  error?: string
  /** Open file tabs (VSCode-style preview tabs) */
  tabs: FileTab[]
  /** Currently active tab path */
  active_tab_path?: string
}

// ============================================================================
// Docker State
// ============================================================================

export type ServiceStatus = 'running' | 'stopped' | 'starting' | 'stopping' | 'error'

export type ServiceType = 'Database' | 'MessageBroker' | 'Cache' | 'Other'

export interface DockerServiceInfo {
  id: string
  name: string
  image: string
  status: ServiceStatus
  port: number | null
  service_type: ServiceType
  project_group: string | null
  is_rstn_managed: boolean
}

export interface ConflictingContainer {
  id: string
  name: string
  image: string
  is_rstn_managed: boolean
}

export interface PortConflict {
  requested_port: number
  conflicting_container: ConflictingContainer
  suggested_port: number
}

export interface PendingConflict {
  service_id: string
  conflict: PortConflict
}

export interface DockersState {
  docker_available: boolean | null
  services: DockerServiceInfo[]
  selected_service_id: string | null
  logs: string[]
  is_loading: boolean
  is_loading_logs: boolean
  pending_conflict: PendingConflict | null
  port_overrides: Record<string, number>
  last_connection_string: string | null
}

// ============================================================================
// Change Management State (CESDD Phase 2)
// ============================================================================

export type ChangeStatus =
  | 'proposed'
  | 'planning'
  | 'planned'
  | 'implementing'
  | 'testing'
  | 'done'
  | 'archived'
  | 'cancelled'
  | 'failed'

export interface Change {
  id: string
  name: string
  status: ChangeStatus
  intent: string
  proposal: string | null
  plan: string | null
  streaming_output: string
  created_at: string
  updated_at: string
  /** ReviewGate session ID for proposal review */
  proposal_review_session_id: string | null
  /** ReviewGate session ID for plan review */
  plan_review_session_id: string | null
  /** Source files selected for context injection */
  context_files: string[]
}

export interface ChangesState {
  changes: Change[]
  selected_change_id: string | null
  is_loading: boolean
}

// ============================================================================
// Living Context State (CESDD Phase 3)
// ============================================================================

export type ContextType =
  | 'product'
  | 'tech-stack'
  | 'architecture'
  | 'api-contracts'
  | 'data-models'
  | 'recent-changes'
  | 'custom'

export interface ContextFile {
  name: string
  path: string
  content: string
  context_type: ContextType
  last_updated: string
  token_estimate: number
}

export interface ContextState {
  files: ContextFile[]
  is_loading: boolean
  is_initialized: boolean
  last_refreshed: string | null

  // Context Sync (after Change completion)
  is_syncing: boolean
  sync_output: string
  sync_error: string | null

  // Context Generation (AI-powered)
  is_generating: boolean
  generation_output: string
  generation_error: string | null
}

// ============================================================================
// Tasks State
// ============================================================================

export type TaskStatus = 'idle' | 'running' | 'success' | 'error'

export interface JustCommandInfo {
  name: string
  description: string | null
  recipe: string
}

/**
 * Constitution workflow status (CESDD Phase 1)
 */
export type WorkflowStatus = 'collecting' | 'generating' | 'complete' | 'error'

/**
 * Constitution initialization workflow state
 */
export interface ConstitutionWorkflow {
  /** Current question index (0-based) */
  current_question: number
  /** User answers so far (question_key -> answer) */
  answers: Record<string, string>
  /** Generated constitution content (streamed from Claude) */
  output: string
  /** Current workflow status */
  status: WorkflowStatus
  /** Whether to include CLAUDE.md content as reference during generation */
  use_claude_md_reference: boolean
  /** Error message if generation failed */
  error?: string
}

// ============================================================================
// Constitution Presets (integrated from Agent Rules)
// ============================================================================

/**
 * Constitution mode - Rules (modular) or Presets (full prompt)
 */
export type ConstitutionMode = 'rules' | 'presets'

/**
 * Constitution preset with custom system prompt
 */
export interface ConstitutionPreset {
  id: string
  name: string
  prompt: string
  is_builtin: boolean
  created_at: string
  updated_at: string
}

/**
 * Constitution presets configuration (worktree-level)
 */
export interface ConstitutionPresetsConfig {
  active_preset_id: string | null
  presets: ConstitutionPreset[]
  temp_file_path?: string
}

export interface TasksState {
  commands: JustCommandInfo[]
  task_statuses: Record<string, TaskStatus>
  active_command: string | null
  output: string[]
  is_loading: boolean
  error: string | null
  /** Constitution workflow state (CESDD Phase 1) */
  constitution_workflow: ConstitutionWorkflow | null
  /** Whether a constitution exists (modular or legacy) */
  constitution_exists: boolean | null
  /** Constitution content (null = not read yet) */
  constitution_content: string | null
  /** Whether project root has CLAUDE.md (null = not checked yet) */
  claude_md_exists: boolean | null
  /** CLAUDE.md content for preview (null = not read yet) */
  claude_md_content: string | null
  /** User skipped importing CLAUDE.md */
  claude_md_skipped: boolean
  /** ReviewGate sessions (CESDD ReviewGate Layer) */
  review_gate: ReviewGateState
  /** Constitution mode: Rules (modular) or Presets (full prompt replacement) */
  constitution_mode: ConstitutionMode
  /** Constitution presets configuration (integrated from Agent Rules) */
  constitution_presets: ConstitutionPresetsConfig
}

// ============================================================================
// ReviewGate Types (CESDD ReviewGate Layer)
// ============================================================================

export type ReviewPolicy = 'AutoApprove' | 'AgentDecides' | 'AlwaysReview'

export type ReviewContentType = 'Plan' | 'Proposal' | 'Code' | 'Artifact'

export type ReviewFileAction = 'create' | 'modify' | 'delete'

export type ReviewStatus = 'pending' | 'reviewing' | 'iterating' | 'approved' | 'rejected'

export type CommentAuthor = 'user' | 'system'

export interface ReviewFileChange {
  path: string
  action: ReviewFileAction
  summary: string
}

export interface ReviewContent {
  content_type: ReviewContentType
  content: string
  file_changes: ReviewFileChange[]
}

export type CommentTarget =
  | { type: 'document' }
  | { type: 'section'; id: string }
  | { type: 'file'; path: string }

export interface ReviewComment {
  id: string
  target: CommentTarget
  content: string
  author: CommentAuthor
  resolved: boolean
  created_at: string
}

export interface ReviewSession {
  id: string
  workflow_node_id: string
  status: ReviewStatus
  content: ReviewContent
  policy: ReviewPolicy
  comments: ReviewComment[]
  iteration: number
  created_at: string
  updated_at: string
}

export interface ReviewGateState {
  sessions: Record<string, ReviewSession>
  active_session_id: string | null
  is_loading: boolean
  error: string | null
}

// ============================================================================
// Environment Configuration (Project-level)
// ============================================================================

export interface EnvCopyResult {
  copied_files: string[]
  failed_files: [string, string][]
  timestamp: string
}

export interface EnvConfig {
  tracked_patterns: string[]
  auto_copy_enabled: boolean
  source_worktree: string | null
  last_copy_result: EnvCopyResult | null
}

// ============================================================================
// Agent Rules (Project scope)
// ============================================================================

export interface AgentProfile {
  id: string
  name: string
  prompt: string
  is_builtin: boolean
  created_at: string
  updated_at: string
}

export interface AgentRulesConfig {
  enabled: boolean
  active_profile_id?: string
  profiles: AgentProfile[]
  temp_file_path?: string
}

export interface BranchInfo {
  name: string
  hasWorktree: boolean
  isCurrent: boolean
}

// ============================================================================
// Notifications
// ============================================================================

export type NotificationType = 'info' | 'success' | 'warning' | 'error'

export interface Notification {
  id: string
  message: string
  notification_type: NotificationType
  created_at: string
  read: boolean
}

// ============================================================================
// MCP State
// ============================================================================

export type McpStatus = 'stopped' | 'starting' | 'running' | 'error'

export type McpLogDirection = 'in' | 'out'

export interface McpLogEntry {
  timestamp: string
  direction: McpLogDirection
  method: string
  tool_name?: string
  payload: string
  is_error: boolean
}

export interface McpTool {
  name: string
  description: string
  input_schema: unknown
}

export interface McpState {
  status: McpStatus
  port?: number
  config_path?: string
  error?: string
  log_entries?: McpLogEntry[]
  available_tools?: McpTool[]
}

// ============================================================================
// Chat State
// ============================================================================

export type ChatRole = 'user' | 'assistant' | 'system'

export interface ChatMessage {
  id: string
  role: ChatRole
  content: string
  timestamp: string
  is_streaming?: boolean
}

export interface ChatState {
  messages: ChatMessage[]
  is_typing: boolean
  error?: string
}

// ============================================================================
// Terminal State
// ============================================================================

export interface TerminalState {
  session_id?: string
  cols: number
  rows: number
}

// ============================================================================
// Worktree State
// ============================================================================

export interface WorktreeState {
  id: string
  path: string
  branch: string
  is_main: boolean
  mcp: McpState
  chat: ChatState
  terminal: TerminalState
  explorer: FileExplorerState
  is_modified: boolean
  active_tab: FeatureTab
  tasks: TasksState
  changes: ChangesState
  context: ContextState
  // NOTE: dockers moved to AppState.docker (global scope)
}

// ============================================================================
// Project State
// ============================================================================

export interface ProjectState {
  id: string
  path: string
  name: string
  worktrees: WorktreeState[]
  active_worktree_index: number
  env_config: EnvConfig
  agent_rules_config: AgentRulesConfig
  available_branches: BranchInfo[]
  is_loading_branches: boolean
}

// ============================================================================
// Global Settings
// ============================================================================

export type Theme = 'system' | 'light' | 'dark'

export interface GlobalSettings {
  theme: Theme
  default_project_path: string | null
}

export interface RecentProject {
  path: string
  name: string
  last_opened: string
}

// ============================================================================
// Error
// ============================================================================

export interface AppError {
  code: string
  message: string
  context?: string
}

// ============================================================================
// Dev Logs (Development Mode Only)
// ============================================================================

export type DevLogSource = 'rust' | 'frontend' | 'claude' | 'ipc'

export type DevLogType = 'action' | 'state' | 'claude' | 'error' | 'info'

export interface DevLog {
  id: string
  timestamp: string
  source: DevLogSource
  log_type: DevLogType
  summary: string
  data: unknown
}

export interface FileViewerState {
  path: string | null
  /** Text file content (UTF-8) */
  content: string | null
  /** Binary file content (raw bytes) */
  binary_content: Uint8Array | null
  is_loading: boolean
  error: string | null
}

export interface A2UIState {
  payload: any | null
}

// ============================================================================
// Main AppState
// ============================================================================

export interface AppState {
  version: string
  projects: ProjectState[]
  active_project_index: number
  global_settings: GlobalSettings
  recent_projects: RecentProject[]
  error?: AppError
  // Three-Scope Model additions
  docker: DockersState
  notifications: Notification[]
  active_view: ActiveView
  // Dev logs (development mode only)
  dev_logs?: DevLog[]
  file_viewer: FileViewerState
  a2ui: A2UIState
}

// ============================================================================
// Actions
// ============================================================================

// Project Management Actions
export interface OpenProjectAction {
  type: 'OpenProject'
  payload: { path: string }
}

export interface CloseProjectAction {
  type: 'CloseProject'
  payload: { index: number }
}

export interface SwitchProjectAction {
  type: 'SwitchProject'
  payload: { index: number }
}

export interface SetFeatureTabAction {
  type: 'SetFeatureTab'
  payload: { tab: FeatureTab }
}

// Worktree Actions
export interface SwitchWorktreeAction {
  type: 'SwitchWorktree'
  payload: { index: number }
}

export interface RefreshWorktreesAction {
  type: 'RefreshWorktrees'
}

export interface SetWorktreesAction {
  type: 'SetWorktrees'
  payload: { worktrees: WorktreeData[] }
}

export interface AddWorktreeAction {
  type: 'AddWorktree'
  payload: { branch: string }
}

export interface AddWorktreeNewBranchAction {
  type: 'AddWorktreeNewBranch'
  payload: { branch: string }
}

export interface RemoveWorktreeAction {
  type: 'RemoveWorktree'
  payload: { worktree_path: string }
}

export interface FetchBranchesAction {
  type: 'FetchBranches'
}

export interface SetBranchesAction {
  type: 'SetBranches'
  payload: { branches: BranchData[] }
}

export interface SetBranchesLoadingAction {
  type: 'SetBranchesLoading'
  payload: { is_loading: boolean }
}

// MCP Actions
export interface StartMcpServerAction {
  type: 'StartMcpServer'
}

export interface StopMcpServerAction {
  type: 'StopMcpServer'
}

export interface SetMcpStatusAction {
  type: 'SetMcpStatus'
  payload: { status: McpStatusData }
}

export interface SetMcpPortAction {
  type: 'SetMcpPort'
  payload: { port: number }
}

export interface SetMcpConfigPathAction {
  type: 'SetMcpConfigPath'
  payload: { path: string }
}

export interface SetMcpErrorAction {
  type: 'SetMcpError'
  payload: { error: string }
}

export interface AddMcpLogEntryAction {
  type: 'AddMcpLogEntry'
  payload: { entry: McpLogEntryData }
}

export interface ClearMcpLogsAction {
  type: 'ClearMcpLogs'
}

export interface UpdateMcpToolsAction {
  type: 'UpdateMcpTools'
  payload: { tools: McpToolData[] }
}

// Chat Actions
export interface SendChatMessageAction {
  type: 'SendChatMessage'
  payload: { text: string }
}

export interface AddChatMessageAction {
  type: 'AddChatMessage'
  payload: { message: ChatMessageData }
}

export interface AppendChatContentAction {
  type: 'AppendChatContent'
  payload: { content: string }
}

export interface SetChatTypingAction {
  type: 'SetChatTyping'
  payload: { is_typing: boolean }
}

export interface SetChatErrorAction {
  type: 'SetChatError'
  payload: { error: string }
}

export interface ClearChatErrorAction {
  type: 'ClearChatError'
}

export interface ClearChatAction {
  type: 'ClearChat'
}

// Constitution Workflow Actions
export interface StartConstitutionWorkflowAction {
  type: 'StartConstitutionWorkflow'
}

export interface ClearConstitutionWorkflowAction {
  type: 'ClearConstitutionWorkflow'
}

export interface AnswerConstitutionQuestionAction {
  type: 'AnswerConstitutionQuestion'
  payload: { answer: string }
}

export interface GenerateConstitutionAction {
  type: 'GenerateConstitution'
}

export interface AppendConstitutionOutputAction {
  type: 'AppendConstitutionOutput'
  payload: { content: string }
}

export interface SaveConstitutionAction {
  type: 'SaveConstitution'
}

export interface CheckConstitutionExistsAction {
  type: 'CheckConstitutionExists'
}

export interface SetConstitutionExistsAction {
  type: 'SetConstitutionExists'
  payload: { exists: boolean }
}

export interface ApplyDefaultConstitutionAction {
  type: 'ApplyDefaultConstitution'
}

export interface ReadConstitutionAction {
  type: 'ReadConstitution'
}

export interface SetConstitutionContentAction {
  type: 'SetConstitutionContent'
  payload: { content: string | null }
}

export interface SetClaudeMdExistsAction {
  type: 'SetClaudeMdExists'
  exists: boolean
}

export interface ReadClaudeMdAction {
  type: 'ReadClaudeMd'
}

export interface SetClaudeMdContentAction {
  type: 'SetClaudeMdContent'
  content: string | null
}

export interface ImportClaudeMdAction {
  type: 'ImportClaudeMd'
}

export interface SkipClaudeMdImportAction {
  type: 'SkipClaudeMdImport'
}

export interface SetUseClaudeMdReferenceAction {
  type: 'SetUseClaudeMdReference'
  payload: { use_reference: boolean }
}

// Constitution Mode & Presets Actions
export interface SetConstitutionModeAction {
  type: 'SetConstitutionMode'
  payload: { mode: ConstitutionModeData }
}

export interface SelectConstitutionPresetAction {
  type: 'SelectConstitutionPreset'
  payload: { preset_id: string | null }
}

export interface CreateConstitutionPresetAction {
  type: 'CreateConstitutionPreset'
  payload: { name: string; prompt: string }
}

export interface UpdateConstitutionPresetAction {
  type: 'UpdateConstitutionPreset'
  payload: { id: string; name: string; prompt: string }
}

export interface DeleteConstitutionPresetAction {
  type: 'DeleteConstitutionPreset'
  payload: { id: string }
}

export interface SetConstitutionPresetTempFileAction {
  type: 'SetConstitutionPresetTempFile'
  payload: { path: string | null }
}

// ReviewGate Actions (CESDD ReviewGate Layer)
export interface StartReviewAction {
  type: 'StartReview'
  payload: {
    workflow_node_id: string
    content: {
      content_type: ReviewContentType
      content: string
      file_changes: ReviewFileChange[]
    }
    policy: ReviewPolicy
  }
}

export interface AddReviewCommentAction {
  type: 'AddReviewComment'
  payload: {
    session_id: string
    target: CommentTarget
    content: string
  }
}

export interface ResolveReviewCommentAction {
  type: 'ResolveReviewComment'
  payload: {
    session_id: string
    comment_id: string
  }
}

export interface SubmitReviewFeedbackAction {
  type: 'SubmitReviewFeedback'
  payload: { session_id: string }
}

export interface ApproveReviewAction {
  type: 'ApproveReview'
  payload: { session_id: string }
}

export interface RejectReviewAction {
  type: 'RejectReview'
  payload: {
    session_id: string
    reason: string
  }
}

export interface UpdateReviewContentAction {
  type: 'UpdateReviewContent'
  payload: {
    session_id: string
    content: {
      content_type: ReviewContentType
      content: string
      file_changes: ReviewFileChange[]
    }
  }
}

export interface SetReviewStatusAction {
  type: 'SetReviewStatus'
  payload: {
    session_id: string
    status: ReviewStatus
  }
}

export interface SetReviewGateLoadingAction {
  type: 'SetReviewGateLoading'
  payload: { is_loading: boolean }
}

export interface SetReviewGateErrorAction {
  type: 'SetReviewGateError'
  payload: { error: string | null }
}

export interface SetActiveReviewSessionAction {
  type: 'SetActiveReviewSession'
  payload: { session_id: string | null }
}

export interface ClearReviewSessionAction {
  type: 'ClearReviewSession'
  payload: { session_id: string }
}

// Change Management Actions (CESDD Phase 2)
export interface CreateChangeAction {
  type: 'CreateChange'
  payload: { intent: string }
}

export interface GenerateProposalAction {
  type: 'GenerateProposal'
  payload: { change_id: string }
}

export interface AppendProposalOutputAction {
  type: 'AppendProposalOutput'
  payload: { change_id: string; content: string }
}

export interface CompleteProposalAction {
  type: 'CompleteProposal'
  payload: { change_id: string }
}

export interface GeneratePlanAction {
  type: 'GeneratePlan'
  payload: { change_id: string }
}

export interface AppendPlanOutputAction {
  type: 'AppendPlanOutput'
  payload: { change_id: string; content: string }
}

export interface CompletePlanAction {
  type: 'CompletePlan'
  payload: { change_id: string }
}

export interface ApprovePlanAction {
  type: 'ApprovePlan'
  payload: { change_id: string }
}

export interface CancelChangeAction {
  type: 'CancelChange'
  payload: { change_id: string }
}

export interface SelectChangeAction {
  type: 'SelectChange'
  payload: { change_id: string | null }
}

export interface RefreshChangesAction {
  type: 'RefreshChanges'
}

export interface SetChangesAction {
  type: 'SetChanges'
  payload: { changes: ChangeData[] }
}

export interface SetChangesLoadingAction {
  type: 'SetChangesLoading'
  payload: { is_loading: boolean }
}

// Context Files Actions (CESDD Phase A2 - File Reading)
export interface AddContextFileAction {
  type: 'AddContextFile'
  change_id: string
  path: string
}

export interface RemoveContextFileAction {
  type: 'RemoveContextFile'
  change_id: string
  path: string
}

export interface ClearContextFilesAction {
  type: 'ClearContextFiles'
  change_id: string
}

// ReviewGate Workflow Integration Actions (CESDD Phase B5)
export interface StartProposalReviewAction {
  type: 'StartProposalReview'
  payload: { change_id: string }
}

export interface StartPlanReviewAction {
  type: 'StartPlanReview'
  payload: { change_id: string }
}

// Living Context Actions (CESDD Phase 3)
export interface LoadContextAction {
  type: 'LoadContext'
}

export interface SetContextAction {
  type: 'SetContext'
  payload: { files: ContextFileData[] }
}

export interface SetContextLoadingAction {
  type: 'SetContextLoading'
  payload: { is_loading: boolean }
}

export interface InitializeContextAction {
  type: 'InitializeContext'
}

export interface RefreshContextAction {
  type: 'RefreshContext'
}

export interface UpdateContextFileAction {
  type: 'UpdateContextFile'
  payload: { name: string; content: string }
}

export interface CheckContextExistsAction {
  type: 'CheckContextExists'
}

export interface SetContextInitializedAction {
  type: 'SetContextInitialized'
  payload: { initialized: boolean }
}

// Context Sync & Archive Actions (CESDD Phase 4)
export interface ArchiveChangeAction {
  type: 'ArchiveChange'
  payload: { change_id: string }
}

export interface SyncContextAction {
  type: 'SyncContext'
  payload: { change_id: string }
}

export interface AppendContextSyncOutputAction {
  type: 'AppendContextSyncOutput'
  payload: { change_id: string; content: string }
}

export interface CompleteContextSyncAction {
  type: 'CompleteContextSync'
  payload: { change_id: string }
}

// Context Generation Actions (AI-powered)
export interface GenerateContextAction {
  type: 'GenerateContext'
}

export interface AppendGenerateContextOutputAction {
  type: 'AppendGenerateContextOutput'
  payload: { content: string }
}

export interface CompleteGenerateContextAction {
  type: 'CompleteGenerateContext'
}

export interface FailGenerateContextAction {
  type: 'FailGenerateContext'
  payload: { error: string }
}

export interface SetChangeArchivedAction {
  type: 'SetChangeArchived'
  payload: { change_id: string }
}

// Implementation Actions (CESDD Phase 5)
export interface ExecutePlanAction {
  type: 'ExecutePlan'
  payload: { change_id: string }
}

export interface AppendImplementationOutputAction {
  type: 'AppendImplementationOutput'
  payload: { change_id: string; content: string }
}

export interface CompleteImplementationAction {
  type: 'CompleteImplementation'
  payload: { change_id: string }
}

export interface FailImplementationAction {
  type: 'FailImplementation'
  payload: { change_id: string; error: string }
}

// Context file data for actions
export interface ContextFileData {
  name: string
  path: string
  content: string
  context_type: ContextType
  last_updated: string
  token_estimate: number
}

// Docker Actions
export interface CheckDockerAvailabilityAction {
  type: 'CheckDockerAvailability'
}

export interface SetDockerAvailableAction {
  type: 'SetDockerAvailable'
  payload: { available: boolean }
}

export interface RefreshDockerServicesAction {
  type: 'RefreshDockerServices'
}

export interface SetDockerServicesAction {
  type: 'SetDockerServices'
  payload: { services: DockerServiceData[] }
}

export interface StartDockerServiceAction {
  type: 'StartDockerService'
  payload: { service_id: string }
}

export interface StopDockerServiceAction {
  type: 'StopDockerService'
  payload: { service_id: string }
}

export interface RestartDockerServiceAction {
  type: 'RestartDockerService'
  payload: { service_id: string }
}

export interface SelectDockerServiceAction {
  type: 'SelectDockerService'
  payload: { service_id: string | null }
}

export interface FetchDockerLogsAction {
  type: 'FetchDockerLogs'
  payload: { service_id: string; tail: number }
}

export interface SetDockerLogsAction {
  type: 'SetDockerLogs'
  payload: { logs: string[] }
}

export interface CreateDatabaseAction {
  type: 'CreateDatabase'
  payload: { service_id: string; db_name: string }
}

export interface CreateVhostAction {
  type: 'CreateVhost'
  payload: { service_id: string; vhost_name: string }
}

export interface SetDockerConnectionStringAction {
  type: 'SetDockerConnectionString'
  payload: { connection_string: string | null }
}

export interface SetDockerLoadingAction {
  type: 'SetDockerLoading'
  payload: { is_loading: boolean }
}

export interface SetDockerLogsLoadingAction {
  type: 'SetDockerLogsLoading'
  payload: { is_loading: boolean }
}

export interface SetPortConflictAction {
  type: 'SetPortConflict'
  payload: { service_id: string; conflict: PortConflictData }
}

export interface ClearPortConflictAction {
  type: 'ClearPortConflict'
}

export interface StartDockerServiceWithPortAction {
  type: 'StartDockerServiceWithPort'
  payload: { service_id: string; port: number }
}

export interface ResolveConflictByStoppingContainerAction {
  type: 'ResolveConflictByStoppingContainer'
  payload: { conflicting_container_id: string; service_id: string }
}

// Tasks Actions
export interface LoadJustfileCommandsAction {
  type: 'LoadJustfileCommands'
}

export interface RefreshJustfileAction {
  type: 'RefreshJustfile'
}

export interface SetJustfileCommandsAction {
  type: 'SetJustfileCommands'
  payload: { commands: JustCommandData[] }
}

export interface RunJustCommandAction {
  type: 'RunJustCommand'
  payload: { name: string; cwd: string }
}

export interface SetTaskStatusAction {
  type: 'SetTaskStatus'
  payload: { name: string; status: TaskStatusData }
}

export interface SetActiveCommandAction {
  type: 'SetActiveCommand'
  payload: { name: string | null }
}

export interface AppendTaskOutputAction {
  type: 'AppendTaskOutput'
  payload: { line: string }
}

export interface ClearTaskOutputAction {
  type: 'ClearTaskOutput'
}

export interface SetTasksLoadingAction {
  type: 'SetTasksLoading'
  payload: { is_loading: boolean }
}

export interface SetTasksErrorAction {
  type: 'SetTasksError'
  payload: { error: string | null }
}

// Settings Actions
export interface SetThemeAction {
  type: 'SetTheme'
  payload: { theme: Theme }
}

export interface SetProjectPathAction {
  type: 'SetProjectPath'
  payload: { path: string | null }
}

// Env Actions (Project scope)
export interface CopyEnvFilesAction {
  type: 'CopyEnvFiles'
  payload: {
    from_worktree_path: string
    to_worktree_path: string
    patterns?: string[]
  }
}

export interface SetEnvCopyResultAction {
  type: 'SetEnvCopyResult'
  payload: { result: EnvCopyResultData }
}

export interface SetEnvTrackedPatternsAction {
  type: 'SetEnvTrackedPatterns'
  payload: { patterns: string[] }
}

export interface SetEnvAutoCopyAction {
  type: 'SetEnvAutoCopy'
  payload: { enabled: boolean }
}

export interface SetEnvSourceWorktreeAction {
  type: 'SetEnvSourceWorktree'
  payload: { worktree_path: string | null }
}

// Agent Rules Actions (Project scope)
export interface SetAgentRulesEnabledAction {
  type: 'SetAgentRulesEnabled'
  payload: { enabled: boolean }
}

export interface SetAgentRulesPromptAction {
  type: 'SetAgentRulesPrompt'
  payload: { prompt: string }
}

export interface SetAgentRulesTempFileAction {
  type: 'SetAgentRulesTempFile'
  payload: { path: string | null }
}

export interface CreateAgentProfileAction {
  type: 'CreateAgentProfile'
  payload: { name: string; prompt: string }
}

export interface UpdateAgentProfileAction {
  type: 'UpdateAgentProfile'
  payload: { id: string; name: string; prompt: string }
}

export interface DeleteAgentProfileAction {
  type: 'DeleteAgentProfile'
  payload: { id: string }
}

export interface SelectAgentProfileAction {
  type: 'SelectAgentProfile'
  payload: { profile_id?: string }
}

// Notification Actions
export interface AddNotificationAction {
  type: 'AddNotification'
  payload: { message: string; notification_type: NotificationTypeData }
}

export interface DismissNotificationAction {
  type: 'DismissNotification'
  payload: { id: string }
}

export interface MarkNotificationReadAction {
  type: 'MarkNotificationRead'
  payload: { id: string }
}

export interface MarkAllNotificationsReadAction {
  type: 'MarkAllNotificationsRead'
}

export interface ClearNotificationsAction {
  type: 'ClearNotifications'
}

// View Actions
export interface SetActiveViewAction {
  type: 'SetActiveView'
  payload: { view: ActiveViewData }
}

// Terminal Actions
export interface SpawnTerminalAction {
  type: 'SpawnTerminal'
  payload: { cols: number; rows: number }
}

export interface ResizeTerminalAction {
  type: 'ResizeTerminal'
  payload: { session_id: string; cols: number; rows: number }
}

export interface WriteTerminalAction {
  type: 'WriteTerminal'
  payload: { session_id: string; data: string }
}

export interface KillTerminalAction {
  type: 'KillTerminal'
  payload: { session_id: string }
}

export interface SetTerminalSessionAction {
  type: 'SetTerminalSession'
  payload: { session_id: string | null }
}

export interface SetTerminalSizeAction {
  type: 'SetTerminalSize'
  payload: { cols: number; rows: number }
}

// Error Actions
export interface SetErrorAction {
  type: 'SetError'
  payload: { code: string; message: string; context?: string }
}

export interface ClearErrorAction {
  type: 'ClearError'
}

// Action data types (for payload data)
export interface DockerServiceData {
  id: string
  name: string
  image: string
  status: string
  port: number | null
  service_type: string
  project_group: string | null
  is_rstn_managed: boolean
}

export interface ConflictingContainerData {
  id: string
  name: string
  image: string
  is_rstn_managed: boolean
}

export interface PortConflictData {
  requested_port: number
  conflicting_container: ConflictingContainerData
  suggested_port: number
}

export interface JustCommandData {
  name: string
  description: string | null
  recipe: string
}

export type TaskStatusData = 'idle' | 'running' | 'success' | 'error'

export interface WorktreeData {
  path: string
  branch: string
  is_main: boolean
}

export interface BranchData {
  name: string
  has_worktree: boolean
  is_current: boolean
}

export type McpStatusData = 'stopped' | 'starting' | 'running' | 'error'

export type McpLogDirectionData = 'in' | 'out'

export interface McpLogEntryData {
  timestamp: string
  direction: McpLogDirectionData
  method: string
  tool_name?: string
  payload: string
  is_error: boolean
}

export interface McpToolData {
  name: string
  description: string
  input_schema: unknown
}

export type ChatRoleData = 'user' | 'assistant' | 'system'

export interface ChatMessageData {
  id: string
  role: ChatRoleData
  content: string
  timestamp: string
  is_streaming?: boolean
}

export interface EnvCopyResultData {
  copied_files: string[]
  failed_files: [string, string][]
  timestamp: string
}

export type NotificationTypeData = 'info' | 'success' | 'warning' | 'error'

export type ActiveViewData = 'tasks' | 'explorer' | 'settings' | 'dockers' | 'env' | 'mcp' | 'chat' | 'terminal' | 'workflows' | 'claude-code' | 'a2ui'

// Constitution Mode & Presets Data Types
export type ConstitutionModeData = 'rules' | 'presets'

export type DevLogSourceData = 'rust' | 'frontend' | 'claude' | 'ipc'

export type DevLogTypeData = 'action' | 'state' | 'claude' | 'error' | 'info'

export interface DevLogData {
  source: DevLogSourceData
  log_type: DevLogTypeData
  summary: string
  data: unknown
}

// Change Management Data Types (CESDD Phase 2)
export type ChangeStatusData =
  | 'proposed'
  | 'planning'
  | 'planned'
  | 'implementing'
  | 'testing'
  | 'done'
  | 'archived'
  | 'cancelled'
  | 'failed'

export interface ChangeData {
  id: string
  name: string
  status: ChangeStatusData
  intent: string
  proposal: string | null
  plan: string | null
  streaming_output: string
  created_at: string
  updated_at: string
  /** ReviewGate session ID for proposal review */
  proposal_review_session_id: string | null
  /** ReviewGate session ID for plan review */
  plan_review_session_id: string | null
  /** Source files selected for context injection */
  context_files: string[]
}

// Dev Log Actions
export interface AddDevLogAction {
  type: 'AddDevLog'
  payload: { log: DevLogData }
}

export interface ClearDevLogsAction {
  type: 'ClearDevLogs'
}

export interface ReadFileAction {
  type: 'ReadFile'
  payload: { path: string }
}

export interface SetFileContentAction {
  type: 'SetFileContent'
  payload: {
    path: string
    content: string | null
    error: string | null
  }
}

export interface SetFileLoadingAction {
  type: 'SetFileLoading'
  payload: { is_loading: boolean }
}

export interface ReadBinaryFileAction {
  type: 'ReadBinaryFile'
  payload: { path: string }
}

export interface SetBinaryFileContentAction {
  type: 'SetBinaryFileContent'
  payload: {
    path: string
    content: Uint8Array | null
    error: string | null
  }
}

export interface SetA2UIPayloadAction {
  type: 'SetA2UIPayload'
  payload: { payload: any | null }
}

// File Explorer Actions
export interface ExploreDirAction {
  type: 'ExploreDir'
  payload: { path: string }
}

export interface SetExplorerEntriesAction {
  type: 'SetExplorerEntries'
  payload: {
    path: string
    entries: FileEntry[]
  }
}

export interface SelectFileAction {
  type: 'SelectFile'
  payload: { path?: string }
}

export interface NavigateBackAction {
  type: 'NavigateBack'
}

export interface NavigateForwardAction {
  type: 'NavigateForward'
}

export interface NavigateUpAction {
  type: 'NavigateUp'
}

export interface SetFileCommentsAction {
  type: 'SetFileComments'
  payload: {
    path: string
    comments: Comment[]
  }
}

export interface AddFileCommentAction {
  type: 'AddFileComment'
  payload: {
    path: string
    content: string
    /** Line number for inline comments (null for file-level comments) */
    line_number: number | null
  }
}

export interface SetExplorerSortAction {
  type: 'SetExplorerSort'
  payload: {
    field: SortField
    direction: SortDirection
  }
}

export interface SetExplorerFilterAction {
  type: 'SetExplorerFilter'
  payload: { query: string }
}

// Tab Management Actions (VSCode-style preview tabs)
export interface OpenFileTabAction {
  type: 'OpenFileTab'
  payload: { path: string }
}

export interface PinTabAction {
  type: 'PinTab'
  payload: { path: string }
}

export interface CloseTabAction {
  type: 'CloseTab'
  payload: { path: string }
}

export interface SwitchTabAction {
  type: 'SwitchTab'
  payload: { path: string }
}

// Union type of all actions
export type Action =
  | OpenProjectAction
  | CloseProjectAction
  | SwitchProjectAction
  | SetFeatureTabAction
  | SwitchWorktreeAction
  | RefreshWorktreesAction
  | SetWorktreesAction
  | AddWorktreeAction
  | AddWorktreeNewBranchAction
  | RemoveWorktreeAction
  | FetchBranchesAction
  | SetBranchesAction
  | SetBranchesLoadingAction
  | StartMcpServerAction
  | StopMcpServerAction
  | SetMcpStatusAction
  | SetMcpPortAction
  | SetMcpConfigPathAction
  | SetMcpErrorAction
  | AddMcpLogEntryAction
  | ClearMcpLogsAction
  | UpdateMcpToolsAction
  | SendChatMessageAction
  | AddChatMessageAction
  | AppendChatContentAction
  | SetChatTypingAction
  | SetChatErrorAction
  | ClearChatErrorAction
  | ClearChatAction
  | StartConstitutionWorkflowAction
  | ClearConstitutionWorkflowAction
  | AnswerConstitutionQuestionAction
  | GenerateConstitutionAction
  | AppendConstitutionOutputAction
  | SaveConstitutionAction
  | CheckConstitutionExistsAction
  | SetConstitutionExistsAction
  | ApplyDefaultConstitutionAction
  | ReadConstitutionAction
  | SetConstitutionContentAction
  | SetClaudeMdExistsAction
  | ReadClaudeMdAction
  | SetClaudeMdContentAction
  | ImportClaudeMdAction
  | SkipClaudeMdImportAction
  | SetUseClaudeMdReferenceAction
  | SetConstitutionModeAction
  | SelectConstitutionPresetAction
  | CreateConstitutionPresetAction
  | UpdateConstitutionPresetAction
  | DeleteConstitutionPresetAction
  | SetConstitutionPresetTempFileAction
  | StartReviewAction
  | AddReviewCommentAction
  | ResolveReviewCommentAction
  | SubmitReviewFeedbackAction
  | ApproveReviewAction
  | RejectReviewAction
  | UpdateReviewContentAction
  | SetReviewStatusAction
  | SetReviewGateLoadingAction
  | SetReviewGateErrorAction
  | SetActiveReviewSessionAction
  | ClearReviewSessionAction
  | CreateChangeAction
  | GenerateProposalAction
  | AppendProposalOutputAction
  | CompleteProposalAction
  | GeneratePlanAction
  | AppendPlanOutputAction
  | CompletePlanAction
  | ApprovePlanAction
  | CancelChangeAction
  | SelectChangeAction
  | RefreshChangesAction
  | SetChangesAction
  | SetChangesLoadingAction
  | AddContextFileAction
  | RemoveContextFileAction
  | ClearContextFilesAction
  | StartProposalReviewAction
  | StartPlanReviewAction
  | LoadContextAction
  | SetContextAction
  | SetContextLoadingAction
  | InitializeContextAction
  | RefreshContextAction
  | UpdateContextFileAction
  | CheckContextExistsAction
  | SetContextInitializedAction
  | ArchiveChangeAction
  | SyncContextAction
  | AppendContextSyncOutputAction
  | CompleteContextSyncAction
  | GenerateContextAction
  | AppendGenerateContextOutputAction
  | CompleteGenerateContextAction
  | FailGenerateContextAction
  | SetChangeArchivedAction
  | ExecutePlanAction
  | AppendImplementationOutputAction
  | CompleteImplementationAction
  | FailImplementationAction
  | CheckDockerAvailabilityAction
  | SetDockerAvailableAction
  | RefreshDockerServicesAction
  | SetDockerServicesAction
  | StartDockerServiceAction
  | StopDockerServiceAction
  | RestartDockerServiceAction
  | SelectDockerServiceAction
  | FetchDockerLogsAction
  | SetDockerLogsAction
  | CreateDatabaseAction
  | CreateVhostAction
  | SetDockerConnectionStringAction
  | SetDockerLoadingAction
  | SetDockerLogsLoadingAction
  | SetPortConflictAction
  | ClearPortConflictAction
  | StartDockerServiceWithPortAction
  | ResolveConflictByStoppingContainerAction
  | LoadJustfileCommandsAction
  | RefreshJustfileAction
  | SetJustfileCommandsAction
  | RunJustCommandAction
  | SetTaskStatusAction
  | SetActiveCommandAction
  | AppendTaskOutputAction
  | ClearTaskOutputAction
  | SetTasksLoadingAction
  | SetTasksErrorAction
  | SetThemeAction
  | SetProjectPathAction
  | CopyEnvFilesAction
  | SetEnvCopyResultAction
  | SetEnvTrackedPatternsAction
  | SetEnvAutoCopyAction
  | SetEnvSourceWorktreeAction
  | SetAgentRulesEnabledAction
  | SetAgentRulesPromptAction
  | SetAgentRulesTempFileAction
  | CreateAgentProfileAction
  | UpdateAgentProfileAction
  | DeleteAgentProfileAction
  | SelectAgentProfileAction
  | AddNotificationAction
  | DismissNotificationAction
  | MarkNotificationReadAction
  | MarkAllNotificationsReadAction
  | ClearNotificationsAction
  | SetActiveViewAction
  | SpawnTerminalAction
  | ResizeTerminalAction
  | WriteTerminalAction
  | KillTerminalAction
  | SetTerminalSessionAction
  | SetTerminalSizeAction
  | SetErrorAction
  | ClearErrorAction
  | AddDevLogAction
  | ClearDevLogsAction
  | ReadFileAction
  | SetFileContentAction
  | SetFileLoadingAction
  | ReadBinaryFileAction
  | SetBinaryFileContentAction
  | SetA2UIPayloadAction
  | ExploreDirAction
  | SetExplorerEntriesAction
  | SelectFileAction
  | NavigateBackAction
  | NavigateForwardAction
  | NavigateUpAction
  | SetFileCommentsAction
  | AddFileCommentAction
  | SetExplorerSortAction
  | SetExplorerFilterAction
  | OpenFileTabAction
  | PinTabAction
  | CloseTabAction
  | SwitchTabAction

// ============================================================================
// UI Helpers
// ============================================================================

export const statusColors: Record<ServiceStatus, string> = {
  running: 'bg-green-500',
  stopped: 'bg-gray-400',
  starting: 'bg-yellow-500',
  stopping: 'bg-orange-500',
  error: 'bg-red-500',
}

export const statusLabels: Record<ServiceStatus, string> = {
  running: 'Running',
  stopped: 'Stopped',
  starting: 'Starting',
  stopping: 'Stopping',
  error: 'Error',
}
