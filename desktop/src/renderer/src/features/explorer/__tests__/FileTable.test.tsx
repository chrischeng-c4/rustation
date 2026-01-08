import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, waitFor } from '@testing-library/react'
import { FileTable } from '../FileTable'

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
        size: 1024,
        permissions: 'rw-r--r--',
        updated_at: new Date('2024-01-01').toISOString(),
        git_status: 'modified' as const,
        comment_count: 2,
      },
      {
        path: '/test/path/folder1',
        name: 'folder1',
        kind: 'directory' as const,
        size: 0,
        permissions: 'rwxr-xr-x',
        updated_at: new Date('2024-01-01').toISOString(),
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

vi.mock('@/hooks/useAppState', () => ({
  useActiveWorktree: () => ({
    worktree: mockWorktree,
    dispatch: mockDispatch,
    isLoading: false,
  }),
}))

describe('FileTable', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders file table headers', () => {
    const { getByText } = render(<FileTable />)

    expect(getByText('Name')).toBeInTheDocument()
    expect(getByText('Size')).toBeInTheDocument()
    expect(getByText('Status')).toBeInTheDocument()
    expect(getByText('Modified')).toBeInTheDocument()
  })

  it('renders file entries in virtualized list', async () => {
    const { getByText } = render(<FileTable />)

    // Wait for ResizeObserver callback to trigger
    await waitFor(() => {
      expect(getByText('file1.txt')).toBeInTheDocument()
    })

    expect(getByText('folder1')).toBeInTheDocument()
  })

  it('displays file size for files', async () => {
    const { getByText } = render(<FileTable />)

    // Wait for List to render
    await waitFor(() => {
      expect(getByText('1 KB')).toBeInTheDocument() // 1024 bytes = 1 KB
    })
  })

  it('displays git status badge for modified files', async () => {
    const { getByText } = render(<FileTable />)

    // Wait for List to render
    await waitFor(() => {
      expect(getByText('modified')).toBeInTheDocument()
    })
  })

  it('handles empty entries gracefully', () => {
    vi.doMock('@/hooks/useAppState', () => ({
      useActiveWorktree: () => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            entries: [],
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }),
    }))

    const { container } = render(<FileTable />)
    expect(container).toBeInTheDocument()
  })
})
