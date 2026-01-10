import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { ExplorerPage } from '../ExplorerPage'

// ResizeObserver mock is now centralized in test/setup.ts

// Mock window.explorerApi
const mockListDirectory = vi.fn().mockResolvedValue([])
Object.defineProperty(window, 'explorerApi', {
  value: {
    listDirectory: mockListDirectory,
  },
  writable: true,
})

// Mock the hooks
const mockDispatch = vi.fn().mockResolvedValue(undefined)

const mockWorktree = {
  id: 'test-worktree',
  path: '/test/path',
  branch: 'main',
  explorer: {
    current_path: '/test/path',
    entries: [
      {
        path: '/test/path/file1.txt',
        name: 'file1.txt',
        kind: 'file' as const,
        size: 100,
        permissions: 'rw-r--r--',
        updated_at: new Date().toISOString(),
        git_status: 'clean' as const,
        comment_count: 0,
      },
      {
        path: '/test/path/folder1',
        name: 'folder1',
        kind: 'directory' as const,
        size: 0,
        permissions: 'rwxr-xr-x',
        updated_at: new Date().toISOString(),
        git_status: 'clean' as const,
        comment_count: 0,
      },
    ],
    selected_path: undefined,
    selected_comments: [],
    sort_config: {
      field: 'name' as const,
      direction: 'asc' as const,
    },
    filter_query: '',
    history: {
      back_stack: [],
      forward_stack: [],
    },
    is_loading: false,
  },
}

let mockUseActiveWorktree = vi.fn(() => ({
  worktree: null,
  dispatch: mockDispatch,
  isLoading: false,
}))

vi.mock('@/hooks/useAppState', () => ({
  useActiveWorktree: () => mockUseActiveWorktree(),
}))

describe('ExplorerPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockUseActiveWorktree = vi.fn(() => ({
      worktree: null,
      dispatch: mockDispatch,
      isLoading: false,
    }))
  })

  it('shows empty state when no worktree selected', () => {
    render(<ExplorerPage />)
    expect(screen.getByText('No Worktree Selected')).toBeInTheDocument()
    expect(screen.getByText('Please select a project worktree to explore files.')).toBeInTheDocument()
  })
})

describe('ExplorerPage - with worktree', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockUseActiveWorktree = vi.fn(() => ({
      worktree: mockWorktree,
      dispatch: mockDispatch,
      isLoading: false,
    }))
  })

  it('shows file explorer header', () => {
    render(<ExplorerPage />)
    expect(screen.getByText('File Explorer')).toBeInTheDocument()
    expect(screen.getByText('Browse files, view metadata, and manage comments')).toBeInTheDocument()
  })

  it('renders FileTreeView with entries when worktree has explorer data', async () => {
    render(<ExplorerPage />)

    // Verify FileTreeView renders inside ExplorerPage
    await waitFor(() => {
      expect(screen.getByText('file1.txt')).toBeInTheDocument()
      expect(screen.getByText('folder1')).toBeInTheDocument()
    })
  })

  it('auto-dispatches ExploreDir when component mounts with empty entries', async () => {
    // Setup worktree with current_path but no entries (needs initial load)
    const worktreeWithEmptyEntries = {
      ...mockWorktree,
      explorer: {
        ...mockWorktree.explorer,
        entries: [],
      },
    }

    mockUseActiveWorktree = vi.fn(() => ({
      worktree: worktreeWithEmptyEntries,
      dispatch: mockDispatch,
      isLoading: false,
    }))

    render(<ExplorerPage />)

    // Wait for auto-dispatch to occur
    await waitFor(() => {
      expect(mockDispatch).toHaveBeenCalledWith({
        type: 'ExploreDir',
        payload: { path: '/test/path' },
      })
    })
  })

  it('does not auto-dispatch ExploreDir when current_path already exists', async () => {
    render(<ExplorerPage />)

    // Should not dispatch because current_path already exists
    await waitFor(() => {
      expect(mockDispatch).not.toHaveBeenCalledWith(
        expect.objectContaining({ type: 'ExploreDir' })
      )
    })
  })
})
