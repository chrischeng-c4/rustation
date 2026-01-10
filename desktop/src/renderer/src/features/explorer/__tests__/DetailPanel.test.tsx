import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { DetailPanel } from '../DetailPanel'

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
        git_status: undefined,
        comment_count: 0,
      },
      {
        path: '/test/path/folder1',
        name: 'folder1',
        kind: 'directory' as const,
        size: 0,
        permissions: 'rwxr-xr-x',
        updated_at: new Date().toISOString(),
        git_status: undefined,
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
    tabs: [],
    active_tab_path: undefined,
  },
}

let mockUseActiveWorktree = vi.fn(() => ({
  worktree: mockWorktree,
  dispatch: mockDispatch,
  isLoading: false,
}))

vi.mock('@/hooks/useAppState', () => ({
  useActiveWorktree: () => mockUseActiveWorktree(),
}))

// Mock window.api for file reading
vi.mock('@/components/shared/SourceCodeViewer', () => ({
  SourceCodeViewer: ({ path }: { path: string }) => (
    <div data-testid="source-code-viewer">Preview: {path}</div>
  ),
}))

describe('DetailPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockUseActiveWorktree = vi.fn(() => ({
      worktree: mockWorktree,
      dispatch: mockDispatch,
      isLoading: false,
    }))
  })

  describe('Empty State', () => {
    it('shows empty state when no file selected', () => {
      render(<DetailPanel />)
      expect(screen.getByText('Select a file to preview')).toBeInTheDocument()
    })
  })

  describe('Root Level Files', () => {
    it('shows preview for selected root file', () => {
      mockUseActiveWorktree = vi.fn(() => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            selected_path: '/test/path/file1.txt',
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }))

      render(<DetailPanel />)
      expect(screen.getByTestId('source-code-viewer')).toBeInTheDocument()
      expect(screen.getByText('Preview: /test/path/file1.txt')).toBeInTheDocument()
    })

    it('shows directory message for selected folder', () => {
      mockUseActiveWorktree = vi.fn(() => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            selected_path: '/test/path/folder1',
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }))

      render(<DetailPanel />)
      expect(screen.getByText('Preview only available for files')).toBeInTheDocument()
    })
  })

  describe('Child Files (Not in entries)', () => {
    it('shows preview for child file not in root entries', () => {
      // Child file path not in explorer.entries but selected
      mockUseActiveWorktree = vi.fn(() => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            selected_path: '/test/path/folder1/child.txt',
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }))

      render(<DetailPanel />)
      // Should show preview because path has file extension
      expect(screen.getByTestId('source-code-viewer')).toBeInTheDocument()
      expect(screen.getByText('Preview: /test/path/folder1/child.txt')).toBeInTheDocument()
    })

    it('shows preview for deeply nested file', () => {
      mockUseActiveWorktree = vi.fn(() => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            selected_path: '/test/path/folder1/subfolder/deep.ts',
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }))

      render(<DetailPanel />)
      expect(screen.getByTestId('source-code-viewer')).toBeInTheDocument()
      expect(screen.getByText('Preview: /test/path/folder1/subfolder/deep.ts')).toBeInTheDocument()
    })

    it('shows directory message for child folder path', () => {
      // Child folder path (no extension)
      mockUseActiveWorktree = vi.fn(() => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            selected_path: '/test/path/folder1/subfolder',
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }))

      render(<DetailPanel />)
      expect(screen.getByText('Preview only available for files')).toBeInTheDocument()
    })
  })

  describe('File Type Detection', () => {
    it('detects known dotfiles as files', () => {
      mockUseActiveWorktree = vi.fn(() => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            selected_path: '/test/path/.gitignore',
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }))

      render(<DetailPanel />)
      // .gitignore is a known dotfile that should be detected as a file
      expect(screen.getByTestId('source-code-viewer')).toBeInTheDocument()
    })

    it('detects dotfiles with extensions as files', () => {
      mockUseActiveWorktree = vi.fn(() => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            selected_path: '/test/path/.eslintrc.json',
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }))

      render(<DetailPanel />)
      expect(screen.getByTestId('source-code-viewer')).toBeInTheDocument()
    })

    it('treats unknown dotfiles without extensions as directories', () => {
      mockUseActiveWorktree = vi.fn(() => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            selected_path: '/test/path/.unknown',
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }))

      render(<DetailPanel />)
      // Unknown dotfiles without extensions are treated as directories
      expect(screen.getByText('Preview only available for files')).toBeInTheDocument()
    })
  })
})
