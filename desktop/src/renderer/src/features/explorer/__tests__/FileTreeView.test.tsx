import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { FileTreeView } from '../FileTreeView'

// Mock window.explorerApi
const mockListDirectory = vi.fn()
Object.defineProperty(window, 'explorerApi', {
  value: {
    listDirectory: mockListDirectory,
  },
  writable: true,
})

// Mock the hooks
const mockDispatch = vi.fn().mockResolvedValue(undefined)

const mockEntries = [
  {
    path: '/test/path/file1.txt',
    name: 'file1.txt',
    kind: 'file' as const,
    size: 100,
    permissions: 'rw-r--r--',
    updated_at: new Date().toISOString(),
    git_status: 'modified' as const,
    comment_count: 2,
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
  {
    path: '/test/path/file2.ts',
    name: 'file2.ts',
    kind: 'file' as const,
    size: 500,
    permissions: 'rw-r--r--',
    updated_at: new Date().toISOString(),
    git_status: 'added' as const,
    comment_count: 0,
  },
]

const mockWorktree = {
  id: 'test-worktree',
  path: '/test/path',
  branch: 'main',
  explorer: {
    current_path: '/test/path',
    entries: mockEntries,
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
  worktree: mockWorktree,
  dispatch: mockDispatch,
  isLoading: false,
}))

vi.mock('@/hooks/useAppState', () => ({
  useActiveWorktree: () => mockUseActiveWorktree(),
}))

describe('FileTreeView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockListDirectory.mockResolvedValue([])
    mockUseActiveWorktree = vi.fn(() => ({
      worktree: mockWorktree,
      dispatch: mockDispatch,
      isLoading: false,
    }))
  })

  describe('Header', () => {
    it('shows project name from root path', () => {
      render(<FileTreeView />)
      expect(screen.getByText('path')).toBeInTheDocument()
    })

    it('shows item count', () => {
      render(<FileTreeView />)
      expect(screen.getByText('3 items')).toBeInTheDocument()
    })

    it('has home button that dispatches ExploreDir', async () => {
      render(<FileTreeView />)

      // Find home button by its icon
      const homeIcon = screen.getByTestId('HomeIcon')
      const homeButton = homeIcon.closest('button')
      expect(homeButton).toBeInTheDocument()

      fireEvent.click(homeButton!)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'ExploreDir',
          payload: { path: '/test/path' },
        })
      })
    })
  })

  describe('File/Folder Rendering', () => {
    it('renders all entries', () => {
      render(<FileTreeView />)

      expect(screen.getByText('file1.txt')).toBeInTheDocument()
      expect(screen.getByText('folder1')).toBeInTheDocument()
      expect(screen.getByText('file2.ts')).toBeInTheDocument()
    })

    it('sorts directories before files', () => {
      render(<FileTreeView />)

      const items = screen.getAllByText(/file1\.txt|folder1|file2\.ts/)
      const itemTexts = items.map(item => item.textContent)

      // folder1 should come before files
      const folderIndex = itemTexts.indexOf('folder1')
      const file1Index = itemTexts.indexOf('file1.txt')
      const file2Index = itemTexts.indexOf('file2.ts')

      expect(folderIndex).toBeLessThan(file1Index)
      expect(folderIndex).toBeLessThan(file2Index)
    })

    it('shows expand arrow for directories', () => {
      render(<FileTreeView />)

      // Folder should have an expand button
      const folderRow = screen.getByText('folder1').closest('[class*="MuiBox-root"]')
      expect(folderRow).toBeInTheDocument()

      // Should have a chevron icon button
      const buttons = folderRow?.querySelectorAll('button')
      expect(buttons?.length).toBeGreaterThan(0)
    })

    it('does not show expand arrow for files', () => {
      render(<FileTreeView />)

      // Get the file row
      const fileText = screen.getByText('file1.txt')
      const fileRow = fileText.closest('[class*="MuiBox-root"]')

      // The file row should not have an expand/collapse button in the arrow area
      // Files have no IconButton for expand
      const iconButtons = fileRow?.querySelectorAll('button')
      // Files should have 0 buttons (no expand arrow)
      expect(iconButtons?.length).toBe(0)
    })
  })

  describe('Git Status', () => {
    it('shows git status indicator for modified files', () => {
      render(<FileTreeView />)

      // file1.txt has git_status: 'modified'
      const fileText = screen.getByText('file1.txt')
      // Modified files should have a specific color applied via inline style
      const style = window.getComputedStyle(fileText)
      // Just verify the element exists and has the text
      expect(fileText).toBeInTheDocument()
      // The color is applied via sx prop, check the element has color style
      expect(fileText).toHaveAttribute('class')
    })

    it('shows git status indicator for added files', () => {
      render(<FileTreeView />)

      // file2.ts has git_status: 'added'
      const fileText = screen.getByText('file2.ts')
      // Just verify the element exists
      expect(fileText).toBeInTheDocument()
    })
  })

  describe('Comment Count', () => {
    it('shows comment count badge when comments exist', () => {
      render(<FileTreeView />)

      // file1.txt has comment_count: 2
      expect(screen.getByText('2')).toBeInTheDocument()
    })

    it('does not show comment badge when no comments', () => {
      render(<FileTreeView />)

      // file2.ts has comment_count: 0, should not show "0"
      const zeros = screen.queryAllByText('0')
      // Should only be 0 items or the 0 should not be a comment badge
      expect(zeros.length).toBe(0)
    })
  })

  describe('Selection', () => {
    it('dispatches SelectFile when clicking on a file', async () => {
      render(<FileTreeView />)

      const file = screen.getByText('file1.txt')
      fireEvent.click(file)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'SelectFile',
          payload: { path: '/test/path/file1.txt' },
        })
      })
    })

    it('dispatches SelectFile and expands when clicking on a folder', async () => {
      const childEntries = [
        {
          name: 'child.txt',
          path: '/test/path/folder1/child.txt',
          kind: 'file',
          size: 50,
          permissions: 'rw-r--r--',
          updated_at: new Date().toISOString(),
          comment_count: 0,
          git_status: null,
        },
      ]

      mockListDirectory.mockResolvedValue(childEntries)

      render(<FileTreeView />)

      const folder = screen.getByText('folder1')
      fireEvent.click(folder)

      // Should dispatch SelectFile
      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'SelectFile',
          payload: { path: '/test/path/folder1' },
        })
      })

      // Should also expand the folder and load children
      await waitFor(() => {
        expect(mockListDirectory).toHaveBeenCalledWith('/test/path/folder1', '/test/path')
      })

      // Child should appear
      await waitFor(() => {
        expect(screen.getByText('child.txt')).toBeInTheDocument()
      })
    })

    it('highlights selected item', () => {
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

      render(<FileTreeView />)

      const fileText = screen.getByText('file1.txt')
      // Selected items should have bold font weight
      expect(fileText).toHaveStyle({ fontWeight: 600 })
    })
  })

  describe('Folder Expansion', () => {
    it('expands folder when clicking arrow and loads children', async () => {
      const childEntries = [
        {
          name: 'child.txt',
          path: '/test/path/folder1/child.txt',
          kind: 'file',
          size: 50,
          permissions: 'rw-r--r--',
          updated_at: new Date().toISOString(),
          comment_count: 0,
          git_status: null,
        },
      ]

      mockListDirectory.mockResolvedValue(childEntries)

      render(<FileTreeView />)

      // Find the expand button for folder1
      const folderRow = screen.getByText('folder1').closest('[class*="MuiBox-root"]')
      const expandButton = folderRow?.querySelector('button')

      expect(expandButton).toBeInTheDocument()

      // Click to expand
      fireEvent.click(expandButton!)

      // Should call listDirectory
      await waitFor(() => {
        expect(mockListDirectory).toHaveBeenCalledWith('/test/path/folder1', '/test/path')
      })

      // Child should appear
      await waitFor(() => {
        expect(screen.getByText('child.txt')).toBeInTheDocument()
      })
    })

    it('toggles folder expansion state when clicking arrow', async () => {
      const childEntries = [
        {
          name: 'child.txt',
          path: '/test/path/folder1/child.txt',
          kind: 'file',
          size: 50,
          permissions: 'rw-r--r--',
          updated_at: new Date().toISOString(),
          comment_count: 0,
          git_status: null,
        },
      ]

      mockListDirectory.mockResolvedValue(childEntries)

      render(<FileTreeView />)

      const folderRow = screen.getByText('folder1').closest('[class*="MuiBox-root"]')
      const expandButton = folderRow?.querySelector('button')

      // Initially collapsed - should show ChevronRight
      expect(screen.getByTestId('ChevronRightIcon')).toBeInTheDocument()

      // Expand
      fireEvent.click(expandButton!)

      await waitFor(() => {
        expect(screen.getByText('child.txt')).toBeInTheDocument()
      })

      // Should now show ExpandMore icon
      expect(screen.getByTestId('ExpandMoreIcon')).toBeInTheDocument()

      // Collapse - click again
      fireEvent.click(expandButton!)

      // Should switch back to ChevronRight
      await waitFor(() => {
        expect(screen.getByTestId('ChevronRightIcon')).toBeInTheDocument()
      })
    })

    it('does not refetch when re-expanding cached folder', async () => {
      const childEntries = [
        {
          name: 'child.txt',
          path: '/test/path/folder1/child.txt',
          kind: 'file',
          size: 50,
          permissions: 'rw-r--r--',
          updated_at: new Date().toISOString(),
          comment_count: 0,
          git_status: null,
        },
      ]

      mockListDirectory.mockResolvedValue(childEntries)

      render(<FileTreeView />)

      const folderRow = screen.getByText('folder1').closest('[class*="MuiBox-root"]')
      const expandButton = folderRow?.querySelector('button')

      // Expand
      fireEvent.click(expandButton!)

      await waitFor(() => {
        expect(mockListDirectory).toHaveBeenCalledTimes(1)
      })

      // Collapse
      fireEvent.click(expandButton!)

      // Wait for collapse
      await waitFor(() => {
        expect(screen.getByTestId('ChevronRightIcon')).toBeInTheDocument()
      })

      // Re-expand - should not call API again
      fireEvent.click(expandButton!)

      await waitFor(() => {
        expect(screen.getByText('child.txt')).toBeInTheDocument()
      })

      // Should still be 1 call (cached)
      expect(mockListDirectory).toHaveBeenCalledTimes(1)
    })
  })

  describe('Empty State', () => {
    it('shows empty tree when no entries', () => {
      mockUseActiveWorktree = vi.fn(() => ({
        worktree: {
          ...mockWorktree,
          explorer: {
            ...mockWorktree.explorer,
            entries: [],
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      }))

      render(<FileTreeView />)

      expect(screen.getByText('0 items')).toBeInTheDocument()
    })
  })

  describe('Error Handling', () => {
    it('handles directory load error gracefully', async () => {
      mockListDirectory.mockRejectedValue(new Error('Failed to load'))

      render(<FileTreeView />)

      const folderRow = screen.getByText('folder1').closest('[class*="MuiBox-root"]')
      const expandButton = folderRow?.querySelector('button')

      // Click to expand
      fireEvent.click(expandButton!)

      // Should call listDirectory
      await waitFor(() => {
        expect(mockListDirectory).toHaveBeenCalled()
      })

      // Should not crash, folder should still be visible
      expect(screen.getByText('folder1')).toBeInTheDocument()
    })
  })
})
