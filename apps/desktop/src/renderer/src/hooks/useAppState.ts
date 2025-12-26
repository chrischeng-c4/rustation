/**
 * State-first React hooks.
 *
 * These hooks subscribe to the Rust-owned AppState and provide
 * a dispatch function for triggering state changes.
 */

import { useState, useEffect, useCallback, useMemo } from 'react'
import type {
  AppState,
  Action,
  DockersState,
  TasksState,
  GlobalSettings,
  ProjectState,
  WorktreeState,
  EnvConfig,
  McpState,
  ChatState,
  TerminalState,
  Notification,
} from '../types/state'

// ============================================================================
// Core Hook
// ============================================================================

interface UseAppStateResult {
  /** Current application state (null if not yet loaded) */
  state: AppState | null
  /** Dispatch an action to update state */
  dispatch: (action: Action) => Promise<void>
  /** Whether state has been loaded */
  isLoading: boolean
}

/**
 * Main hook for accessing application state.
 *
 * Subscribes to state updates from Rust and provides a dispatch function.
 *
 * @example
 * ```tsx
 * function App() {
 *   const { state, dispatch, isLoading } = useAppState()
 *
 *   if (isLoading || !state) return <Loading />
 *
 *   return (
 *     <div>
 *       <p>Projects: {state.projects.length}</p>
 *       <button onClick={() => dispatch({ type: 'OpenProject', payload: { path: '/path' } })}>
 *         Open Project
 *       </button>
 *     </div>
 *   )
 * }
 * ```
 */
export function useAppState(): UseAppStateResult {
  const [state, setState] = useState<AppState | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    // Subscribe to state updates
    const unsubscribe = window.stateApi.onStateUpdate((stateJson: string) => {
      try {
        const parsed = JSON.parse(stateJson) as AppState
        setState(parsed)
        setIsLoading(false)
      } catch (error) {
        console.error('Failed to parse state:', error)
      }
    })

    // Get initial state
    window.stateApi.getState().then((stateJson) => {
      try {
        const parsed = JSON.parse(stateJson) as AppState
        setState(parsed)
        setIsLoading(false)
      } catch (error) {
        console.error('Failed to parse initial state:', error)
      }
    })

    return unsubscribe
  }, [])

  const dispatch = useCallback(async (action: Action): Promise<void> => {
    await window.stateApi.dispatch(action)
  }, [])

  return { state, dispatch, isLoading }
}

// ============================================================================
// Project Hooks
// ============================================================================

interface UseActiveProjectResult {
  /** Currently active project (null if no projects open) */
  project: ProjectState | null
  /** Index of the active project */
  activeIndex: number
  /** All open projects */
  projects: ProjectState[]
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing the active project state.
 *
 * @example
 * ```tsx
 * function ProjectView() {
 *   const { project, dispatch } = useActiveProject()
 *
 *   if (!project) return <NoProjectOpen />
 *
 *   return (
 *     <div>
 *       <h1>{project.name}</h1>
 *       <TabContent tab={project.active_tab} />
 *     </div>
 *   )
 * }
 * ```
 */
export function useActiveProject(): UseActiveProjectResult {
  const { state, dispatch, isLoading } = useAppState()

  const project = useMemo(() => {
    if (!state || state.projects.length === 0) return null
    return state.projects[state.active_project_index] ?? null
  }, [state])

  return {
    project,
    activeIndex: state?.active_project_index ?? 0,
    projects: state?.projects ?? [],
    dispatch,
    isLoading,
  }
}

// ============================================================================
// Worktree Hooks
// ============================================================================

interface UseActiveWorktreeResult {
  /** Currently active worktree (null if no projects open) */
  worktree: WorktreeState | null
  /** The parent project */
  project: ProjectState | null
  /** Index of the active worktree within the project */
  activeWorktreeIndex: number
  /** All worktrees in the active project */
  worktrees: WorktreeState[]
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing the active worktree state.
 *
 * @example
 * ```tsx
 * function WorktreeView() {
 *   const { worktree, project, dispatch } = useActiveWorktree()
 *
 *   if (!worktree) return <NoProjectOpen />
 *
 *   return (
 *     <div>
 *       <h1>{project.name} / {worktree.branch}</h1>
 *       <TabContent tab={worktree.active_tab} />
 *     </div>
 *   )
 * }
 * ```
 */
export function useActiveWorktree(): UseActiveWorktreeResult {
  const { project, dispatch, isLoading } = useActiveProject()

  const worktree = useMemo(() => {
    if (!project || project.worktrees.length === 0) return null
    return project.worktrees[project.active_worktree_index] ?? null
  }, [project])

  return {
    worktree,
    project,
    activeWorktreeIndex: project?.active_worktree_index ?? 0,
    worktrees: project?.worktrees ?? [],
    dispatch,
    isLoading,
  }
}

// ============================================================================
// Feature-specific Hooks
// ============================================================================

interface UseDockersStateResult {
  /** Docker-related state from the active project */
  dockers: DockersState | null
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing Docker state from the active worktree.
 *
 * @example
 * ```tsx
 * function DockersPage() {
 *   const { dockers, dispatch, isLoading } = useDockersState()
 *
 *   useEffect(() => {
 *     dispatch({ type: 'RefreshDockerServices' })
 *   }, [dispatch])
 *
 *   if (isLoading || !dockers) return <Loading />
 *
 *   return <ServiceList services={dockers.services} />
 * }
 * ```
 */
export function useDockersState(): UseDockersStateResult {
  // Docker is now at global scope (AppState.docker)
  const { state, dispatch, isLoading } = useAppState()
  return {
    dockers: state?.docker ?? null,
    dispatch,
    isLoading,
  }
}

interface UseTasksStateResult {
  /** Tasks-related state from the active worktree */
  tasks: TasksState | null
  /** The worktree path (for running tasks) */
  projectPath: string | null
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing Tasks state from the active worktree.
 */
export function useTasksState(): UseTasksStateResult {
  const { worktree, dispatch, isLoading } = useActiveWorktree()
  return {
    tasks: worktree?.tasks ?? null,
    projectPath: worktree?.path ?? null,
    dispatch,
    isLoading,
  }
}

interface UseSettingsStateResult {
  /** Global settings state */
  settings: GlobalSettings | null
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing global Settings state.
 */
export function useSettingsState(): UseSettingsStateResult {
  const { state, dispatch, isLoading } = useAppState()
  return {
    settings: state?.global_settings ?? null,
    dispatch,
    isLoading,
  }
}

// ============================================================================
// Env Configuration Hook
// ============================================================================

interface UseEnvStateResult {
  /** Env config from the active project */
  envConfig: EnvConfig | null
  /** Active project for context */
  project: ProjectState | null
  /** All worktrees in the active project */
  worktrees: WorktreeState[]
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing Env configuration state from the active project.
 *
 * @example
 * ```tsx
 * function EnvPage() {
 *   const { envConfig, worktrees, dispatch } = useEnvState()
 *
 *   if (!envConfig) return <NoProjectOpen />
 *
 *   return (
 *     <div>
 *       <p>Patterns: {envConfig.tracked_patterns.join(', ')}</p>
 *       <p>Auto-copy: {envConfig.auto_copy_enabled ? 'ON' : 'OFF'}</p>
 *     </div>
 *   )
 * }
 * ```
 */
export function useEnvState(): UseEnvStateResult {
  const { project, dispatch, isLoading } = useActiveProject()
  return {
    envConfig: project?.env_config ?? null,
    project,
    worktrees: project?.worktrees ?? [],
    dispatch,
    isLoading,
  }
}

// ============================================================================
// MCP State Hook
// ============================================================================

interface UseMcpStateResult {
  /** MCP state from the active worktree */
  mcp: McpState | null
  /** Active worktree path (for context) */
  worktreePath: string | null
  /** Active project name (for display) */
  projectName: string | null
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing MCP state from the active worktree.
 *
 * @example
 * ```tsx
 * function McpPage() {
 *   const { mcp, dispatch, isLoading } = useMcpState()
 *
 *   if (!mcp) return <NoProjectOpen />
 *
 *   return (
 *     <div>
 *       <p>Status: {mcp.status}</p>
 *       <p>Port: {mcp.port ?? 'Not running'}</p>
 *       <button onClick={() => dispatch({ type: 'StartMcpServer' })}>
 *         Start Server
 *       </button>
 *     </div>
 *   )
 * }
 * ```
 */
export function useMcpState(): UseMcpStateResult {
  const { worktree, dispatch, isLoading } = useActiveWorktree()
  const { project } = useActiveProject()
  return {
    mcp: worktree?.mcp ?? null,
    worktreePath: worktree?.path ?? null,
    projectName: project?.name ?? null,
    dispatch,
    isLoading,
  }
}

// ============================================================================
// Chat State Hook
// ============================================================================

interface UseChatStateResult {
  /** Chat state from the active worktree */
  chat: ChatState | null
  /** Active worktree path (for context) */
  worktreePath: string | null
  /** Active project name (for display) */
  projectName: string | null
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing Chat state from the active worktree.
 *
 * @example
 * ```tsx
 * function ChatPage() {
 *   const { chat, dispatch, isLoading } = useChatState()
 *
 *   if (!chat) return <NoProjectOpen />
 *
 *   const handleSend = (text: string) => {
 *     dispatch({ type: 'SendChatMessage', payload: { text } })
 *   }
 *
 *   return (
 *     <div>
 *       <MessageList messages={chat.messages} />
 *       <ChatInput onSend={handleSend} disabled={chat.is_typing} />
 *     </div>
 *   )
 * }
 * ```
 */
export function useChatState(): UseChatStateResult {
  const { worktree, dispatch, isLoading } = useActiveWorktree()
  const { project } = useActiveProject()
  return {
    chat: worktree?.chat ?? null,
    worktreePath: worktree?.path ?? null,
    projectName: project?.name ?? null,
    dispatch,
    isLoading,
  }
}

// ============================================================================
// Terminal State Hook
// ============================================================================

interface UseTerminalStateResult {
  /** Terminal state from the active worktree */
  terminal: TerminalState | null
  /** Active worktree path (for working directory) */
  worktreePath: string | null
  /** Active project name (for display) */
  projectName: string | null
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing Terminal state from the active worktree.
 *
 * @example
 * ```tsx
 * function TerminalPage() {
 *   const { terminal, worktreePath, dispatch } = useTerminalState()
 *
 *   if (!terminal) return <NoProjectOpen />
 *
 *   const handleSpawn = () => {
 *     dispatch({ type: 'SpawnTerminal', payload: { cols: 80, rows: 24 } })
 *   }
 *
 *   return <XTerm sessionId={terminal.session_id} />
 * }
 * ```
 */
export function useTerminalState(): UseTerminalStateResult {
  const { worktree, dispatch, isLoading } = useActiveWorktree()
  const { project } = useActiveProject()
  return {
    terminal: worktree?.terminal ?? null,
    worktreePath: worktree?.path ?? null,
    projectName: project?.name ?? null,
    dispatch,
    isLoading,
  }
}

// ============================================================================
// Notifications State Hook
// ============================================================================

interface UseNotificationsStateResult {
  /** All notifications */
  notifications: Notification[]
  /** Unread notification count */
  unreadCount: number
  /** Dispatch an action */
  dispatch: (action: Action) => Promise<void>
  /** Whether state is loading */
  isLoading: boolean
}

/**
 * Hook for accessing Notifications state.
 *
 * @example
 * ```tsx
 * function NotificationBell() {
 *   const { notifications, unreadCount, dispatch } = useNotificationsState()
 *
 *   const handleMarkAllRead = () => {
 *     dispatch({ type: 'MarkAllNotificationsRead' })
 *   }
 *
 *   return (
 *     <div>
 *       <Bell />
 *       {unreadCount > 0 && <Badge>{unreadCount}</Badge>}
 *     </div>
 *   )
 * }
 * ```
 */
export function useNotificationsState(): UseNotificationsStateResult {
  const { state, dispatch, isLoading } = useAppState()

  const notifications = useMemo(() => state?.notifications ?? [], [state])
  const unreadCount = useMemo(
    () => notifications.filter((n) => !n.read).length,
    [notifications]
  )

  return {
    notifications,
    unreadCount,
    dispatch,
    isLoading,
  }
}
