import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { ConstitutionPanel } from '../ConstitutionPanel'
import type { AppState } from '@/types/state'

// Mock useAppState hook
const mockDispatch = vi.fn()
const mockUseAppState = vi.fn()

vi.mock('@/hooks/useAppState', () => ({
  useAppState: () => mockUseAppState(),
}))

// Helper to create mock state with Constitution workflow
const createMockState = (
  status: 'collecting' | 'generating' | 'complete',
  currentQuestion: number = 0,
  answers: Record<string, string> = {},
  output: string = ''
): Partial<AppState> => ({
  active_project: {
    id: 'test-project',
    path: '/test/project',
    worktrees: [
      {
        path: '/test/project',
        branch: 'main',
        is_main: true,
        tasks: {
          commands: [],
          task_statuses: {},
          active_command: 'constitution-init',
          output: [],
          is_loading: false,
          error: null,
          constitution_workflow: {
            current_question: currentQuestion,
            answers,
            output,
            status,
          },
        },
        chat: { messages: [], is_typing: false, error: null, debug_logs: [] },
        mcp: {
          status: 'stopped',
          port: null,
          config_path: null,
          error: null,
          logs: [],
          available_tools: [],
        },
        terminal: null,
      },
    ],
    active_worktree_index: 0,
    agent_rules_config: {
      enabled: false,
      custom_prompt: '',
      temp_file_path: null,
      profiles: [],
      active_profile_id: null,
    },
    env_config: {
      tracked_patterns: [],
      auto_copy_on_create: false,
      source_worktree_path: null,
      last_copy_result: null,
    },
  },
})

describe('ConstitutionPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockDispatch.mockResolvedValue(undefined)
  })

  describe('Collecting Phase', () => {
    it('renders first question in collecting phase', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('collecting', 0, {}),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/What technology stack does this project use/)).toBeInTheDocument()
    })

    it('shows progress indicator with question count', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('collecting', 1, { tech_stack: 'React' }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText('1 / 4')).toBeInTheDocument()
    })

    it('shows checkmarks for answered questions', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('collecting', 2, {
          tech_stack: 'React',
          security: 'JWT',
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      const { container } = render(<ConstitutionPanel />)
      const checkmarks = container.querySelectorAll('.text-green-500')
      expect(checkmarks.length).toBeGreaterThanOrEqual(1)
    })

    it('allows typing answer in textarea', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('collecting', 0, {}),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const textarea = screen.getByPlaceholderText('Type your answer...')
      fireEvent.change(textarea, { target: { value: 'React + Rust' } })
      expect(textarea).toHaveValue('React + Rust')
    })

    it('calls dispatch when Next button clicked', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('collecting', 0, {}),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const textarea = screen.getByPlaceholderText('Type your answer...')
      const nextButton = screen.getByRole('button', { name: /Next/ })

      fireEvent.change(textarea, { target: { value: 'React + Rust' } })
      fireEvent.click(nextButton)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'AnswerConstitutionQuestion',
          payload: { answer: 'React + Rust' },
        })
      })
    })

    it('disables Next button when answer is empty', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('collecting', 0, {}),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const nextButton = screen.getByRole('button', { name: /Next/ })
      expect(nextButton).toBeDisabled()
    })

    it('shows Generate button after all questions answered', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('collecting', 4, {
          tech_stack: 'React',
          security: 'JWT',
          code_quality: '80%',
          architecture: 'State-first',
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/All questions answered!/)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Generate Constitution/ })).toBeInTheDocument()
    })

    it('calls dispatch when Generate Constitution clicked', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('collecting', 4, {
          tech_stack: 'React',
          security: 'JWT',
          code_quality: '80%',
          architecture: 'State-first',
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const generateButton = screen.getByRole('button', { name: /Generate Constitution/ })
      fireEvent.click(generateButton)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({ type: 'GenerateConstitution' })
      })
    })
  })

  describe('Generating Phase', () => {
    it('renders generating state with spinner', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('generating', 4, {}, '# Project Constitution\n'),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Generating Constitution.../)).toBeInTheDocument()
      expect(screen.getByText(/Streaming from Claude Code.../)).toBeInTheDocument()
    })

    it('displays streaming output as markdown', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState(
          'generating',
          4,
          {},
          '# Project Constitution\n\n## Technology Stack\n\nReact + Rust'
        ),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText('Project Constitution')).toBeInTheDocument()
      expect(screen.getByText('Technology Stack')).toBeInTheDocument()
    })

    it('shows waiting message when output is empty', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('generating', 4, {}, ''),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Waiting for Claude.../)).toBeInTheDocument()
    })
  })

  describe('Complete Phase', () => {
    it('renders completion state with success message', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('complete', 4, {}, '# Project Constitution\n\nComplete!'),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Constitution Generated/)).toBeInTheDocument()
      expect(screen.getByText(/Constitution saved to/)).toBeInTheDocument()
      expect(screen.getByText(/.rstn\/constitution.md/)).toBeInTheDocument()
    })

    it('displays final constitution content', () => {
      const finalContent = '# Project Constitution\n\n## Rules\n\n- Rule 1\n- Rule 2'
      mockUseAppState.mockReturnValue({
        state: createMockState('complete', 4, {}, finalContent),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText('Project Constitution')).toBeInTheDocument()
      expect(screen.getByText('Rules')).toBeInTheDocument()
    })

    it('shows success styling in complete state', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('complete', 4, {}, '# Done'),
        dispatch: mockDispatch,
        isLoading: false,
      })

      const { container } = render(<ConstitutionPanel />)
      const successIcons = container.querySelectorAll('.text-green-500')
      expect(successIcons.length).toBeGreaterThanOrEqual(1)
    })
  })

  describe('Loading State', () => {
    it('shows spinner when loading', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState('collecting', 0, {}),
        dispatch: mockDispatch,
        isLoading: true,
      })

      const { container } = render(<ConstitutionPanel />)
      const spinner = container.querySelector('.animate-spin')
      expect(spinner).toBeInTheDocument()
    })

    it('shows spinner when workflow is null', () => {
      mockUseAppState.mockReturnValue({
        state: {
          active_project: {
            worktrees: [{ tasks: { constitution_workflow: null } }],
            active_worktree_index: 0,
          },
        },
        dispatch: mockDispatch,
        isLoading: false,
      })

      const { container } = render(<ConstitutionPanel />)
      const spinner = container.querySelector('.animate-spin')
      expect(spinner).toBeInTheDocument()
    })
  })
})
