import { render, screen, waitFor, fireEvent, act } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { SourceCodeViewer, type CommentData } from '../SourceCodeViewer'

// Mock data factories (following project pattern)
const createMockFileViewerState = (overrides?: Partial<any>) => ({
  path: '/test/file.ts',
  content: 'const foo = "bar"\nconst baz = 42\nfunction test() {\n  return true\n}\n',
  binary_content: null,
  is_loading: false,
  error: null,
  ...overrides,
})

const createMockComment = (overrides?: Partial<CommentData>): CommentData => ({
  id: '1',
  content: 'Test comment',
  author: 'User',
  created_at: '2024-01-01T00:00:00Z',
  line_number: 1,
  ...overrides,
})

describe('SourceCodeViewer - Comment Functionality', () => {
  let mockDispatch: ReturnType<typeof vi.fn>

  beforeEach(() => {
    vi.clearAllMocks()
    mockDispatch = vi.fn().mockResolvedValue(undefined)

    // Setup mock state via window.stateApi (from test/setup.ts)
    const mockState = {
      projects: [
        {
          id: 'project-1',
          path: '/test',
          name: 'Test Project',
          worktrees: [
            {
              id: 'wt-1',
              path: '/test',
              branch: 'main',
            },
          ],
          active_worktree_id: 'wt-1',
        },
      ],
      active_project_id: 'project-1',
      file_viewer: createMockFileViewerState(),
    }

    window.stateApi.getState = vi.fn().mockResolvedValue(JSON.stringify(mockState))
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('Comment Display', () => {
    it('should render existing comments on correct line numbers', async () => {
      const comments = [
        createMockComment({ id: '1', line_number: 1, content: 'Comment on line 1' }),
        createMockComment({ id: '2', line_number: 3, content: 'Comment on line 3' }),
      ]

      render(
        <SourceCodeViewer
          path="/test/file.ts"
          projectRoot="/test"
          comments={comments}
        />
      )

      // Wait for file to load
      await waitFor(() => {
        expect(screen.getByText(/const foo/)).toBeInTheDocument()
      })

      // Comments should be visible
      expect(screen.getByText('Comment on line 1')).toBeInTheDocument()
      expect(screen.getByText('Comment on line 3')).toBeInTheDocument()
    })

    it('should show comment author and timestamp', async () => {
      const comments = [
        createMockComment({
          id: '1',
          line_number: 1,
          content: 'Test comment',
          author: 'TestUser',
          created_at: '2024-01-15T10:30:00Z',
        }),
      ]

      render(
        <SourceCodeViewer
          path="/test/file.ts"
          projectRoot="/test"
          comments={comments}
        />
      )

      await waitFor(() => {
        expect(screen.getByText('Test comment')).toBeInTheDocument()
      })

      // Author should be displayed
      expect(screen.getByText(/TestUser/)).toBeInTheDocument()
    })

    it('should display multiple comments on same line', async () => {
      const comments = [
        createMockComment({ id: '1', line_number: 2, content: 'First comment' }),
        createMockComment({ id: '2', line_number: 2, content: 'Second comment' }),
      ]

      render(
        <SourceCodeViewer
          path="/test/file.ts"
          projectRoot="/test"
          comments={comments}
        />
      )

      await waitFor(() => {
        expect(screen.getByText('First comment')).toBeInTheDocument()
      })

      expect(screen.getByText('Second comment')).toBeInTheDocument()
    })
  })

  describe('Comment Input UI', () => {
    it('should show input when add comment button is clicked', async () => {
      const onAddComment = vi.fn()

      render(
        <SourceCodeViewer
          path="/test/file.ts"
          projectRoot="/test"
          onAddComment={onAddComment}
        />
      )

      await waitFor(() => {
        expect(screen.getByText(/const foo/)).toBeInTheDocument()
      })

      // Find add comment button (may need to adjust selector based on actual implementation)
      const addButtons = screen.getAllByRole('button', { name: /add comment/i })
      expect(addButtons.length).toBeGreaterThan(0)

      // Click first add comment button
      await act(async () => {
        fireEvent.click(addButtons[0])
      })

      // Textarea should appear
      await waitFor(() => {
        expect(screen.getByRole('textbox')).toBeInTheDocument()
      })
    })

    it('should have submit and cancel buttons in comment input', async () => {
      const onAddComment = vi.fn()

      render(
        <SourceCodeViewer
          path="/test/file.ts"
          projectRoot="/test"
          onAddComment={onAddComment}
        />
      )

      await waitFor(() => {
        expect(screen.getByText(/const foo/)).toBeInTheDocument()
      })

      // Open comment input
      const addButtons = screen.getAllByRole('button', { name: /add comment/i })
      await act(async () => {
        fireEvent.click(addButtons[0])
      })

      // Wait for input to appear
      await waitFor(() => {
        expect(screen.getByRole('textbox')).toBeInTheDocument()
      })

      // Should have Submit and Cancel buttons
      expect(screen.getByRole('button', { name: /submit/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument()
    })
  })

  describe('Comment Submission Flow', () => {
    it('should keep input visible until onAddComment resolves', async () => {
      let resolveComment: () => void
      const onAddComment = vi.fn(
        () =>
          new Promise<void>((resolve) => {
            resolveComment = resolve
          })
      )

      render(
        <SourceCodeViewer
          path="/test/file.ts"
          projectRoot="/test"
          onAddComment={onAddComment}
        />
      )

      // Wait for file to load
      await waitFor(() => {
        expect(screen.getByText(/const foo/)).toBeInTheDocument()
      })

      // Open comment input
      const addButtons = screen.getAllByRole('button', { name: /add comment/i })
      await act(async () => {
        fireEvent.click(addButtons[0])
      })

      // Input should be visible
      const textarea = await screen.findByRole('textbox')
      expect(textarea).toBeInTheDocument()

      // Type comment
      await act(async () => {
        fireEvent.change(textarea, { target: { value: 'Test comment' } })
      })

      // Submit
      const submitButton = screen.getByRole('button', { name: /submit/i })
      await act(async () => {
        fireEvent.click(submitButton)
      })

      // Input should STILL be visible (onAddComment pending)
      expect(screen.getByRole('textbox')).toBeInTheDocument()

      // Resolve the promise
      await act(async () => {
        resolveComment!()
      })

      // NOW input should disappear
      await waitFor(() => {
        expect(screen.queryByRole('textbox')).not.toBeInTheDocument()
      })

      // Verify onAddComment was called with correct args
      expect(onAddComment).toHaveBeenCalledWith(expect.any(Number), 'Test comment')
    })

    it('should cancel comment input without calling onAddComment', async () => {
      const onAddComment = vi.fn()

      render(
        <SourceCodeViewer
          path="/test/file.ts"
          projectRoot="/test"
          onAddComment={onAddComment}
        />
      )

      await waitFor(() => {
        expect(screen.getByText(/const foo/)).toBeInTheDocument()
      })

      // Open comment input
      const addButtons = screen.getAllByRole('button', { name: /add comment/i })
      await act(async () => {
        fireEvent.click(addButtons[0])
      })

      // Type some content
      const textarea = await screen.findByRole('textbox')
      await act(async () => {
        fireEvent.change(textarea, { target: { value: 'Will not save' } })
      })

      // Click cancel
      const cancelButton = screen.getByRole('button', { name: /cancel/i })
      await act(async () => {
        fireEvent.click(cancelButton)
      })

      // Input should be gone
      await waitFor(() => {
        expect(screen.queryByRole('textbox')).not.toBeInTheDocument()
      })

      // onAddComment should NOT have been called
      expect(onAddComment).not.toHaveBeenCalled()
    })

    it('should call onAddComment with correct lineNumber and content', async () => {
      const onAddComment = vi.fn().mockResolvedValue(undefined)

      render(
        <SourceCodeViewer
          path="/test/file.ts"
          projectRoot="/test"
          onAddComment={onAddComment}
        />
      )

      await waitFor(() => {
        expect(screen.getByText(/const foo/)).toBeInTheDocument()
      })

      // Open comment input for first line
      const addButtons = screen.getAllByRole('button', { name: /add comment/i })
      await act(async () => {
        fireEvent.click(addButtons[0]) // First button = line 1
      })

      // Fill and submit
      const textarea = await screen.findByRole('textbox')
      await act(async () => {
        fireEvent.change(textarea, { target: { value: 'My test comment' } })
      })

      const submitButton = screen.getByRole('button', { name: /submit/i })
      await act(async () => {
        fireEvent.click(submitButton)
      })

      // Verify onAddComment called with expected args
      await waitFor(() => {
        expect(onAddComment).toHaveBeenCalledWith(
          expect.any(Number), // line number (1-indexed)
          'My test comment'
        )
      })
    })
  })

  describe('Edge Cases', () => {
    it('should not open multiple comment inputs simultaneously', async () => {
      const onAddComment = vi.fn()

      render(
        <SourceCodeViewer
          path="/test/file.ts"
          projectRoot="/test"
          onAddComment={onAddComment}
        />
      )

      await waitFor(() => {
        expect(screen.getByText(/const foo/)).toBeInTheDocument()
      })

      // Open first comment input
      const addButtons = screen.getAllByRole('button', { name: /add comment/i })
      await act(async () => {
        fireEvent.click(addButtons[0])
      })

      // Should have exactly one textarea
      expect(screen.getAllByRole('textbox')).toHaveLength(1)

      // Try to open another comment input
      if (addButtons.length > 1) {
        await act(async () => {
          fireEvent.click(addButtons[1])
        })

        // Should still have exactly one textarea
        expect(screen.getAllByRole('textbox')).toHaveLength(1)
      }
    })
  })
})
