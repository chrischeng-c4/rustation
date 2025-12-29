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
 * - tasks, settings, mcp, chat, terminal: Worktree scope
 * - dockers: Global scope
 * - env, agent_rules: Project scope
 */
export type ActiveView = 'tasks' | 'settings' | 'dockers' | 'env' | 'mcp' | 'chat' | 'terminal' | 'agent_rules' | 'workflows'

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
}

export interface ChangesState {
  changes: Change[]
  selected_change_id: string | null
  is_loading: boolean
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
export type WorkflowStatus = 'collecting' | 'generating' | 'complete'

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
  /** Whether .rstn/constitution.md exists (null = not checked yet) */
  constitution_exists: boolean | null
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
  is_modified: boolean
  active_tab: FeatureTab
  tasks: TasksState
  changes: ChangesState
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
  payload: { path: string }
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

export type ActiveViewData = 'tasks' | 'settings' | 'dockers' | 'env' | 'mcp' | 'chat' | 'terminal' | 'agent_rules' | 'workflows'

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
}

// Dev Log Actions
export interface AddDevLogAction {
  type: 'AddDevLog'
  payload: { log: DevLogData }
}

export interface ClearDevLogsAction {
  type: 'ClearDevLogs'
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
  | SetDockerLoadingAction
  | SetDockerLogsLoadingAction
  | SetPortConflictAction
  | ClearPortConflictAction
  | StartDockerServiceWithPortAction
  | ResolveConflictByStoppingContainerAction
  | LoadJustfileCommandsAction
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
