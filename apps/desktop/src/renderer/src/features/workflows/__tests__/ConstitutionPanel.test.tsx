import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { ConstitutionPanel } from '../ConstitutionPanel'

// Mock useAppState hook
const mockDispatch = vi.fn()
const mockUseAppState = vi.fn()

vi.mock('@/hooks/useAppState', () => ({
  useAppState: () => mockUseAppState(),
}))

// Helper to create mock state with Constitution workflow
const createMockState = (options: {
  constitutionExists?: boolean | null
  constitutionContent?: string | null
  claudeMdExists?: boolean | null
  claudeMdContent?: string | null
  claudeMdSkipped?: boolean
  workflow?: {
    status: 'collecting' | 'generating' | 'complete'
    currentQuestion?: number
    answers?: Record<string, string>
    output?: string
    useClaudeMdReference?: boolean
  } | null
}) => ({
  active_project_index: 0,
  projects: [
    {
      id: 'test-project',
      path: '/test/project',
      active_worktree_index: 0,
      worktrees: [
        {
          path: '/test/project',
          branch: 'main',
          is_main: true,
          tasks: {
            commands: [],
            task_statuses: {},
            active_command: 'constitution-management',
            output: [],
            is_loading: false,
            error: null,
            constitution_exists: options.constitutionExists ?? null,
            constitution_content: options.constitutionContent ?? null,
            claude_md_exists: options.claudeMdExists ?? null,
            claude_md_content: options.claudeMdContent ?? null,
            claude_md_skipped: options.claudeMdSkipped ?? false,
            constitution_workflow: options.workflow
              ? {
                  current_question: options.workflow.currentQuestion ?? 0,
                  answers: options.workflow.answers ?? {},
                  output: options.workflow.output ?? '',
                  status: options.workflow.status,
                  use_claude_md_reference: options.workflow.useClaudeMdReference ?? false,
                }
              : null,
          },
          chat: { messages: [], is_typing: false, error: null },
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
  ],
})

describe('ConstitutionPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockDispatch.mockResolvedValue(undefined)
  })

  describe('Mount Behavior', () => {
    it('dispatches ClearConstitutionWorkflow and CheckConstitutionExists on mount', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({ constitutionExists: null }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({ type: 'ClearConstitutionWorkflow' })
        expect(mockDispatch).toHaveBeenCalledWith({ type: 'CheckConstitutionExists' })
      })
    })

    it('shows loading spinner while constitution_exists is null', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({ constitutionExists: null }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Checking constitution.../)).toBeInTheDocument()
    })
  })

  describe('Constitution Exists State', () => {
    it('shows success message when constitution exists', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({ constitutionExists: true }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Constitution/)).toBeInTheDocument()
      expect(screen.getByText(/Governance rules for AI development/)).toBeInTheDocument()
    })

    it('shows Regenerate button when constitution exists', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({ constitutionExists: true }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByRole('button', { name: /Regenerate/ })).toBeInTheDocument()
    })
  })

  describe('Constitution Missing State', () => {
    it('shows initialization options when constitution does not exist', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({ constitutionExists: false }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Constitution Management/)).toBeInTheDocument()
      expect(screen.getByText(/Initialize Constitution/)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Apply Default Template/ })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Create with Q&A/ })).toBeInTheDocument()
    })

    it('calls dispatch when Apply Default Template is clicked', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({ constitutionExists: false }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const applyButton = screen.getByRole('button', { name: /Apply Default Template/ })
      fireEvent.click(applyButton)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({ type: 'ApplyDefaultConstitution' })
      })
    })

    it('calls dispatch when Create with Q&A is clicked', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({ constitutionExists: false }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const qaButton = screen.getByRole('button', { name: /Create with Q&A/ })
      fireEvent.click(qaButton)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({ type: 'StartConstitutionWorkflow' })
      })
    })
  })

  describe('CLAUDE.md Detection', () => {
    it('shows CLAUDE.md preview when detected and no constitution exists', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          claudeMdExists: true,
          claudeMdContent: '# Project Rules\n\nThis is CLAUDE.md content',
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Found CLAUDE.md/)).toBeInTheDocument()
      expect(screen.getByText(/Existing project instructions detected/)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Use This/ })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Skip, Create New/ })).toBeInTheDocument()
    })

    it('shows loading spinner while CLAUDE.md content is being fetched', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          claudeMdExists: true,
          claudeMdContent: null,
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Loading preview.../)).toBeInTheDocument()
    })

    it('calls dispatch to read CLAUDE.md when detected', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          claudeMdExists: true,
          claudeMdContent: null,
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({ type: 'ReadClaudeMd' })
      })
    })

    it('calls ImportClaudeMd when Use This is clicked', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          claudeMdExists: true,
          claudeMdContent: '# CLAUDE.md content',
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const useButton = screen.getByRole('button', { name: /Use This/ })
      fireEvent.click(useButton)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({ type: 'ImportClaudeMd' })
      })
    })

    it('calls SkipClaudeMdImport when Skip is clicked', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          claudeMdExists: true,
          claudeMdContent: '# CLAUDE.md content',
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const skipButton = screen.getByRole('button', { name: /Skip, Create New/ })
      fireEvent.click(skipButton)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({ type: 'SkipClaudeMdImport' })
      })
    })

    it('shows normal init options after user skips CLAUDE.md', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          claudeMdExists: true,
          claudeMdSkipped: true,
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Constitution Management/)).toBeInTheDocument()
      expect(screen.getByText(/Initialize Constitution/)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Apply Default Template/ })).toBeInTheDocument()
    })
  })

  describe('CLAUDE.md Reference in Q&A', () => {
    it('shows reference checkbox when CLAUDE.md exists during Q&A', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          claudeMdExists: true,
          claudeMdContent: '# Project Rules\n\nSome rules here',
          workflow: { status: 'collecting', currentQuestion: 0, useClaudeMdReference: true },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByLabelText(/Reference existing CLAUDE.md/)).toBeInTheDocument()
      expect(screen.getByText(/Include your project's CLAUDE.md/)).toBeInTheDocument()
    })

    it('does not show reference checkbox when CLAUDE.md does not exist', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          claudeMdExists: false,
          workflow: { status: 'collecting', currentQuestion: 0 },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.queryByLabelText(/Reference existing CLAUDE.md/)).not.toBeInTheDocument()
    })

    it('calls dispatch when reference checkbox is toggled', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          claudeMdExists: true,
          claudeMdContent: '# Rules',
          workflow: { status: 'collecting', currentQuestion: 0, useClaudeMdReference: true },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const checkbox = screen.getByLabelText(/Reference existing CLAUDE.md/)
      fireEvent.click(checkbox)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'SetUseClaudeMdReference',
          payload: { use_reference: false },
        })
      })
    })
  })

  describe('Collecting Phase', () => {
    it('renders first question in collecting phase', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: { status: 'collecting', currentQuestion: 0 },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      // Use heading role to find the question title specifically (not the progress list)
      expect(screen.getByRole('heading', { name: /What technology stack does this project use/ })).toBeInTheDocument()
    })

    it('shows progress indicator with question count', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: { status: 'collecting', currentQuestion: 1 },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText('1 / 4 questions answered')).toBeInTheDocument()
    })

    it('shows checkmarks for answered questions', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: {
            status: 'collecting',
            currentQuestion: 2,
            answers: { tech_stack: 'React', security: 'JWT' },
          },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      const { container } = render(<ConstitutionPanel />)
      const checkmarks = container.querySelectorAll('.text-green-500')
      expect(checkmarks.length).toBeGreaterThanOrEqual(1)
    })

    it('allows selecting options and typing optional notes', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: { status: 'collecting', currentQuestion: 0 },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const optionButton = screen.getByRole('button', { name: 'Rust' })
      fireEvent.click(optionButton)

      const textarea = screen.getByPlaceholderText('Optional notes (keep it short)')
      fireEvent.change(textarea, { target: { value: 'Targeting CLI tools' } })
      expect(textarea).toHaveValue('Targeting CLI tools')
    })

    it('calls dispatch when Next button clicked', async () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: { status: 'collecting', currentQuestion: 0 },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const optionButton = screen.getByRole('button', { name: 'Rust' })
      const nextButton = screen.getByRole('button', { name: /^Next$/ })

      fireEvent.click(optionButton)
      fireEvent.click(nextButton)

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'AnswerConstitutionQuestion',
          payload: { answer: 'Selections:\n- Rust' },
        })
      })
    })

    it('disables Next button when answer is empty', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: { status: 'collecting', currentQuestion: 0 },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      const nextButton = screen.getByRole('button', { name: /^Next$/ })
      expect(nextButton).toBeDisabled()
    })

    it('shows Generate button after all questions answered', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: {
            status: 'collecting',
            currentQuestion: 4,
            answers: {
              tech_stack: 'React',
              security: 'JWT',
              code_quality: '80%',
              architecture: 'State-first',
            },
          },
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
        state: createMockState({
          constitutionExists: false,
          workflow: {
            status: 'collecting',
            currentQuestion: 4,
            answers: {
              tech_stack: 'React',
              security: 'JWT',
              code_quality: '80%',
              architecture: 'State-first',
            },
          },
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
        state: createMockState({
          constitutionExists: false,
          workflow: { status: 'generating', currentQuestion: 4, output: '# Project Constitution\n' },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Generating Constitution/)).toBeInTheDocument()
      expect(screen.getByText(/Streaming from Claude Code.../)).toBeInTheDocument()
    })

    it('displays streaming output as markdown', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: {
            status: 'generating',
            currentQuestion: 4,
            output: '# Project Constitution\n\n## Technology Stack\n\nReact + Rust',
          },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText('Project Constitution')).toBeInTheDocument()
      expect(screen.getByText('Technology Stack')).toBeInTheDocument()
    })

    it('shows waiting message when output is empty', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: { status: 'generating', currentQuestion: 4, output: '' },
        }),
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
        state: createMockState({
          constitutionExists: false,
          workflow: {
            status: 'complete',
            currentQuestion: 4,
            output: '# Project Constitution\n\nComplete!',
          },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Constitution Generated/)).toBeInTheDocument()
      expect(screen.getByText(/Constitution saved to/)).toBeInTheDocument()
    })

    it('displays final constitution content', () => {
      const finalContent = '# Project Constitution\n\n## Rules\n\n- Rule 1\n- Rule 2'
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: { status: 'complete', currentQuestion: 4, output: finalContent },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText('Project Constitution')).toBeInTheDocument()
      expect(screen.getByText('Rules')).toBeInTheDocument()
    })

    it('shows success styling in complete state', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({
          constitutionExists: false,
          workflow: { status: 'complete', currentQuestion: 4, output: '# Done' },
        }),
        dispatch: mockDispatch,
        isLoading: false,
      })

      const { container } = render(<ConstitutionPanel />)
      const successIcons = container.querySelectorAll('.text-green-500')
      expect(successIcons.length).toBeGreaterThanOrEqual(1)
    })
  })

  describe('Loading State', () => {
    it('shows spinner when isLoading is true', () => {
      mockUseAppState.mockReturnValue({
        state: createMockState({ constitutionExists: false }),
        dispatch: mockDispatch,
        isLoading: true,
      })

      render(<ConstitutionPanel />)
      expect(screen.getByText(/Checking constitution.../)).toBeInTheDocument()
    })
  })
})
