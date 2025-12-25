/**
 * Application state types (matching Rust AppState).
 *
 * These types mirror the Rust state structs in packages/core/src/app_state.rs
 * IMPORTANT: Keep these in sync with Rust types!
 */

// ============================================================================
// Navigation
// ============================================================================

export type FeatureTab = 'tasks' | 'dockers' | 'settings'

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
}

export interface DockersState {
  docker_available: boolean | null
  services: DockerServiceInfo[]
  selected_service_id: string | null
  logs: string[]
  is_loading: boolean
  is_loading_logs: boolean
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

export interface TasksState {
  commands: JustCommandInfo[]
  task_statuses: Record<string, TaskStatus>
  active_command: string | null
  output: string[]
  is_loading: boolean
  error: string | null
}

// ============================================================================
// Project State
// ============================================================================

export interface ProjectState {
  id: string
  path: string
  name: string
  is_modified: boolean
  active_tab: FeatureTab
  tasks: TasksState
  dockers: DockersState
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
// Main AppState
// ============================================================================

export interface AppState {
  version: string
  projects: ProjectState[]
  active_project_index: number
  global_settings: GlobalSettings
  recent_projects: RecentProject[]
  error?: AppError
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
}

export interface JustCommandData {
  name: string
  description: string | null
  recipe: string
}

export type TaskStatusData = 'idle' | 'running' | 'success' | 'error'

// Union type of all actions
export type Action =
  | OpenProjectAction
  | CloseProjectAction
  | SwitchProjectAction
  | SetFeatureTabAction
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
  | SetErrorAction
  | ClearErrorAction

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
