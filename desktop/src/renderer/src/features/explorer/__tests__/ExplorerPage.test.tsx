import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { ExplorerPage } from '../ExplorerPage'

// ResizeObserver mock is now centralized in test/setup.ts

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

  it('renders FileTable with entries when worktree has explorer data', async () => {
    render(<ExplorerPage />)

    // Verify FileTable renders inside ExplorerPage
    await waitFor(() => {
      expect(screen.getByText('file1.txt')).toBeInTheDocument()
      expect(screen.getByText('folder1')).toBeInTheDocument()
    })
  })
})
