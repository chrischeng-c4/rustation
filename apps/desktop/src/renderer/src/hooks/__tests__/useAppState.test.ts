import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { useAppState, useActiveProject, useActiveWorktree, useDockersState, useTasksState, useSettingsState } from '../useAppState'
import type { AppState, ProjectState, WorktreeState } from '@/types/state'

// Setup mocks
const mockOnStateUpdate = vi.fn()
const mockGetState = vi.fn()
const mockDispatch = vi.fn()

const createMockWorktree = (overrides?: Partial<WorktreeState>): WorktreeState => ({
  id: 'wt-1',
  path: '/path/to/worktree',
  branch: 'main',
  is_main: true,
  is_modified: false,
  active_tab: 'tasks',
  mcp: {
    status: 'stopped',
  },
  tasks: {
    commands: [],
    task_statuses: {},
    output: [],
    active_command: null,
    is_loading: false,
    error: null,
  },
  // NOTE: dockers moved to AppState.docker (global scope)
  ...overrides,
})

const createMockProject = (overrides?: Partial<ProjectState>): ProjectState => ({
  id: 'proj-1',
  name: 'Test Project',
  path: '/path/to/project',
  worktrees: [createMockWorktree()],
  active_worktree_index: 0,
  env_config: {
    tracked_patterns: ['.env', '.envrc', '.claude/', '.vscode/'],
    auto_copy_enabled: true,
    source_worktree: null,
    last_copy_result: null,
  },
  ...overrides,
})

const createMockState = (overrides?: Partial<AppState>): AppState => ({
  version: '0.1.0',
  projects: [],
  active_project_index: 0,
  recent_projects: [],
  global_settings: {
    theme: 'system',
    default_project_path: null,
  },
  docker: {
    docker_available: null,
    services: [],
    selected_service_id: null,
    logs: [],
    is_loading: false,
    is_loading_logs: false,
    pending_conflict: null,
    port_overrides: {},
  },
  notifications: [],
  active_view: 'tasks',
  ...overrides,
})

describe('useAppState', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    let stateCallback: ((json: string) => void) | null = null

    mockOnStateUpdate.mockImplementation((callback: (json: string) => void) => {
      stateCallback = callback
      return () => {
        stateCallback = null
      }
    })

    mockGetState.mockResolvedValue(JSON.stringify(createMockState()))
    mockDispatch.mockResolvedValue(undefined)

    window.stateApi = {
      onStateUpdate: mockOnStateUpdate,
      getState: mockGetState,
      dispatch: mockDispatch,
    }
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('initializes with loading state', () => {
    mockGetState.mockImplementation(() => new Promise(() => {})) // Never resolves
    const { result } = renderHook(() => useAppState())
    expect(result.current.isLoading).toBe(true)
    expect(result.current.state).toBeNull()
  })

  it('loads initial state', async () => {
    const mockState = createMockState({ projects: [createMockProject()] })
    mockGetState.mockResolvedValue(JSON.stringify(mockState))

    const { result } = renderHook(() => useAppState())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.state).toEqual(mockState)
  })

  it('subscribes to state updates', async () => {
    const mockState = createMockState()
    mockGetState.mockResolvedValue(JSON.stringify(mockState))

    const { result } = renderHook(() => useAppState())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(mockOnStateUpdate).toHaveBeenCalledTimes(1)
  })

  it('dispatches actions', async () => {
    mockGetState.mockResolvedValue(JSON.stringify(createMockState()))
    const { result } = renderHook(() => useAppState())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    await act(async () => {
      await result.current.dispatch({ type: 'OpenProject', payload: { path: '/new/path' } })
    })

    expect(mockDispatch).toHaveBeenCalledWith({ type: 'OpenProject', payload: { path: '/new/path' } })
  })

  it('unsubscribes on unmount', async () => {
    const unsubscribe = vi.fn()
    mockOnStateUpdate.mockReturnValue(unsubscribe)
    mockGetState.mockResolvedValue(JSON.stringify(createMockState()))

    const { unmount } = renderHook(() => useAppState())

    await waitFor(() => {})

    unmount()
    expect(unsubscribe).toHaveBeenCalled()
  })
})

describe('useActiveProject', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockOnStateUpdate.mockReturnValue(() => {})
    mockDispatch.mockResolvedValue(undefined)
  })

  it('returns null project when no projects', async () => {
    mockGetState.mockResolvedValue(JSON.stringify(createMockState({ projects: [] })))
    window.stateApi = {
      onStateUpdate: mockOnStateUpdate,
      getState: mockGetState,
      dispatch: mockDispatch,
    }

    const { result } = renderHook(() => useActiveProject())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.project).toBeNull()
    expect(result.current.projects).toEqual([])
  })

  it('returns active project', async () => {
    const project = createMockProject()
    mockGetState.mockResolvedValue(JSON.stringify(createMockState({
      projects: [project],
      active_project_index: 0,
    })))
    window.stateApi = {
      onStateUpdate: mockOnStateUpdate,
      getState: mockGetState,
      dispatch: mockDispatch,
    }

    const { result } = renderHook(() => useActiveProject())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.project).toEqual(project)
    expect(result.current.activeIndex).toBe(0)
  })
})

describe('useActiveWorktree', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockOnStateUpdate.mockReturnValue(() => {})
    mockDispatch.mockResolvedValue(undefined)
  })

  it('returns null worktree when no projects', async () => {
    mockGetState.mockResolvedValue(JSON.stringify(createMockState({ projects: [] })))
    window.stateApi = {
      onStateUpdate: mockOnStateUpdate,
      getState: mockGetState,
      dispatch: mockDispatch,
    }

    const { result } = renderHook(() => useActiveWorktree())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.worktree).toBeNull()
    expect(result.current.project).toBeNull()
  })

  it('returns active worktree', async () => {
    const worktree = createMockWorktree({ branch: 'feature-1' })
    const project = createMockProject({ worktrees: [worktree], active_worktree_index: 0 })
    mockGetState.mockResolvedValue(JSON.stringify(createMockState({
      projects: [project],
      active_project_index: 0,
    })))
    window.stateApi = {
      onStateUpdate: mockOnStateUpdate,
      getState: mockGetState,
      dispatch: mockDispatch,
    }

    const { result } = renderHook(() => useActiveWorktree())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.worktree?.branch).toBe('feature-1')
    expect(result.current.project).toEqual(project)
  })
})

describe('useDockersState', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockOnStateUpdate.mockReturnValue(() => {})
    mockDispatch.mockResolvedValue(undefined)
  })

  it('returns docker state from global state', async () => {
    // Docker is now at global scope (AppState.docker)
    const docker = {
      services: [{ id: 'svc-1', name: 'postgres', image: 'postgres:16', status: 'running' as const, port: 5432, service_type: 'Database' as const, project_group: null, is_rstn_managed: true }],
      selected_service_id: null,
      logs: [],
      is_loading: false,
      is_loading_logs: false,
      docker_available: true,
      pending_conflict: null,
      port_overrides: {},
    }
    mockGetState.mockResolvedValue(JSON.stringify(createMockState({ docker })))
    window.stateApi = {
      onStateUpdate: mockOnStateUpdate,
      getState: mockGetState,
      dispatch: mockDispatch,
    }

    const { result } = renderHook(() => useDockersState())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.dockers).toEqual(docker)
    expect(result.current.dockers?.services).toHaveLength(1)
  })
})

describe('useTasksState', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockOnStateUpdate.mockReturnValue(() => {})
    mockDispatch.mockResolvedValue(undefined)
  })

  it('returns null tasks when no worktree', async () => {
    mockGetState.mockResolvedValue(JSON.stringify(createMockState({ projects: [] })))
    window.stateApi = {
      onStateUpdate: mockOnStateUpdate,
      getState: mockGetState,
      dispatch: mockDispatch,
    }

    const { result } = renderHook(() => useTasksState())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.tasks).toBeNull()
    expect(result.current.projectPath).toBeNull()
  })

  it('returns tasks state and project path', async () => {
    const tasks = {
      commands: [{ name: 'build', description: 'Build project', recipe: 'cargo build' }],
      task_statuses: {},
      output: [],
      active_command: null,
      is_loading: false,
      error: null,
    }
    const worktree = createMockWorktree({ tasks, path: '/path/to/worktree' })
    const project = createMockProject({ worktrees: [worktree] })
    mockGetState.mockResolvedValue(JSON.stringify(createMockState({ projects: [project] })))
    window.stateApi = {
      onStateUpdate: mockOnStateUpdate,
      getState: mockGetState,
      dispatch: mockDispatch,
    }

    const { result } = renderHook(() => useTasksState())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.tasks).toEqual(tasks)
    expect(result.current.projectPath).toBe('/path/to/worktree')
  })
})

describe('useSettingsState', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockOnStateUpdate.mockReturnValue(() => {})
    mockDispatch.mockResolvedValue(undefined)
  })

  it('returns global settings', async () => {
    const settings = { theme: 'dark' as const, default_project_path: null }
    mockGetState.mockResolvedValue(JSON.stringify(createMockState({ global_settings: settings })))
    window.stateApi = {
      onStateUpdate: mockOnStateUpdate,
      getState: mockGetState,
      dispatch: mockDispatch,
    }

    const { result } = renderHook(() => useSettingsState())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.settings).toEqual(settings)
  })
})
