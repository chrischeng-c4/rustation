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
