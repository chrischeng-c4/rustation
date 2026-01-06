import { useState, useCallback, useEffect } from 'react'
import {
  Description as FileTextIcon,
  Refresh as RefreshIcon,
  CheckCircle as CheckCircleIcon,
  ChevronRight as ChevronRightIcon,
  AutoAwesome as SparklesIcon,
  Code as FileCodeIcon,
  ExpandMore as ExpandMoreIcon,
  ErrorOutline as AlertCircleIcon,
  Cancel as XCircleIcon
} from '@mui/icons-material'
import {
  Button,
  Card,
  CardContent,
  Box,
  Stack,
  Typography,
  Checkbox,
  Collapse,
  TextField,
  FormControlLabel,
  IconButton,
  Chip,
  Paper,
  Divider
} from '@mui/material'
import { PageHeader } from '@/components/shared/PageHeader'
import { WorkflowHeader } from '@/components/shared/WorkflowHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { useAppState } from '@/hooks/useAppState'
import ReactMarkdown from 'react-markdown'

const QUESTION_CONFIGS = [
  {
    key: 'tech_stack',
    question: 'What technology stack does this project use?',
    hint: 'Pick all that apply. Add optional notes if needed.',
    options: [
      'Rust',
      'TypeScript',
      'JavaScript',
      'React',
      'Electron',
      'napi-rs',
      'Node.js',
      'Python',
      'uv',
      'Postgres',
      'SQLite',
      'Docker',
      'Tailwind CSS',
      'Vite',
      'Next.js',
      'Tauri',
    ],
  },
  {
    key: 'security',
    question: 'What security requirements must all code follow?',
    hint: 'Choose constraints. Add a short note if needed.',
    options: [
      'No secrets in repo',
      'Validate all user input',
      'Deny path traversal',
      'No network access without approval',
      'No destructive commands without review',
      'Limit file access to workspace',
      'Require ReviewGate for code changes',
    ],
  },
  {
    key: 'code_quality',
    question: 'What code quality standards?',
    hint: 'Select tests and quality bars.',
    options: [
      'cargo test must pass',
      'cargo clippy clean',
      'pnpm test must pass',
      'Formatters required',
      'No unwrap() in production',
      'State transition tests required',
      'E2E full-flow tests required',
      'Coverage >= 80%',
    ],
  },
  {
    key: 'architecture',
    question: 'Any architectural constraints?',
    hint: 'Select architectural rules or add notes.',
    options: [
      'State-first (serializable state)',
      'UI = render(state)',
      'No business logic in React',
      'Backend drives mutations',
      'No MOCK_* in production UI',
      'Bridge must use real napi-rs',
      'KB-first (spec before code)',
      'Split files at 500+ lines',
    ],
  },
] as const

/**
 * Constitution initialization workflow panel.
 */
export function ConstitutionPanel() {
  const { state, dispatch, isLoading } = useAppState()
  const [selectedOptions, setSelectedOptions] = useState<Record<string, string[]>>({})
  const [customNotes, setCustomNotes] = useState<Record<string, string>>({})
  const [previewOpen, setPreviewOpen] = useState(false)

  // Note: active_project is not serialized, use projects[active_project_index]
  const activeProject = state?.projects?.[state?.active_project_index ?? 0]
  const worktree = activeProject?.worktrees?.[activeProject?.active_worktree_index ?? 0]
  const workflow = worktree?.tasks?.constitution_workflow
  const constitutionExists = worktree?.tasks?.constitution_exists
  const constitutionContent = worktree?.tasks?.constitution_content

  // CLAUDE.md detection state
  const claudeMdExists = worktree?.tasks?.claude_md_exists
  const claudeMdContent = worktree?.tasks?.claude_md_content
  const claudeMdSkipped = worktree?.tasks?.claude_md_skipped ?? false

  // Check constitution exists on mount and clear any stale workflow
  useEffect(() => {
    const init = async () => {
      await dispatch({ type: 'ClearConstitutionWorkflow' })
      await dispatch({ type: 'CheckConstitutionExists' })
    }
    init()
  }, [dispatch])

  // Read constitution content when it exists
  useEffect(() => {
    if (constitutionExists === true && !constitutionContent) {
      dispatch({ type: 'ReadConstitution' })
    }
  }, [constitutionExists, constitutionContent, dispatch])

  // Read CLAUDE.md content when detected (for preview)
  useEffect(() => {
    if (claudeMdExists === true && !claudeMdContent && !claudeMdSkipped) {
      dispatch({ type: 'ReadClaudeMd' })
    }
  }, [claudeMdExists, claudeMdContent, claudeMdSkipped, dispatch])

  const questions = QUESTION_CONFIGS

  const handleApplyDefault = useCallback(async () => {
    await dispatch({ type: 'ApplyDefaultConstitution' })
  }, [dispatch])

  const handleImportClaudeMd = useCallback(async () => {
    await dispatch({ type: 'ImportClaudeMd' })
  }, [dispatch])

  const handleSkipClaudeMd = useCallback(async () => {
    await dispatch({ type: 'SkipClaudeMdImport' })
  }, [dispatch])

  const handleToggleClaudeMdReference = useCallback(async (checked: boolean) => {
    await dispatch({
      type: 'SetUseClaudeMdReference',
      payload: { use_reference: checked }
    })
  }, [dispatch])

  const handleStartQA = useCallback(async () => {
    setSelectedOptions({})
    setCustomNotes({})
    await dispatch({ type: 'StartConstitutionWorkflow' })
  }, [dispatch])

  const toggleOption = useCallback((questionKey: string, value: string) => {
    setSelectedOptions((prev) => {
      const current = prev[questionKey] ?? []
      if (current.includes(value)) {
        return { ...prev, [questionKey]: current.filter((item) => item !== value) }
      }
      return { ...prev, [questionKey]: [...current, value] }
    })
  }, [])

  const buildAnswer = useCallback(
    (questionKey: string) => {
      const selections = selectedOptions[questionKey] ?? []
      const notes = (customNotes[questionKey] ?? '').trim()
      if (selections.length === 0 && !notes) return ''

      const parts: string[] = []
      if (selections.length > 0) {
        parts.push(`Selections:\n- ${selections.join('\n- ')}`)
      }
      if (notes) {
        parts.push(`Notes:\n${notes}`)
      }
      return parts.join('\n\n')
    },
    [selectedOptions, customNotes]
  )

  const handleAnswerSubmit = useCallback(
    async (questionKey: string) => {
      const answer = buildAnswer(questionKey)
      if (!answer) return

      await dispatch({
        type: 'AnswerConstitutionQuestion',
        payload: { answer },
      })
    },
    [buildAnswer, dispatch]
  )

  const handleGenerate = useCallback(async () => {
    await dispatch({ type: 'GenerateConstitution' })
  }, [dispatch])

  // Loading state - checking existence
  if (isLoading || constitutionExists === null || constitutionExists === undefined) {
    return <LoadingState message="Checking constitution..." />
  }

  return renderRulesContent()

  function renderRulesContent() {
    // Found CLAUDE.md but no constitution - show import option with preview
    if (claudeMdExists === true && constitutionExists === false && !claudeMdSkipped && !workflow) {
      return (
        <Stack sx={{ height: '100%' }}>
          <PageHeader
            title="Found CLAUDE.md"
            description="Existing project instructions detected"
            icon={<FileCodeIcon sx={{ color: 'primary.main' }} />}
          />
          <Box sx={{ flex: 1, p: 3, pt: 0, display: 'flex', flexDirection: 'column' }}>
            <Paper variant="outlined" sx={{ flex: 1, display: 'flex', flexDirection: 'column', bgcolor: 'primary.container', borderColor: 'primary.main' }}>
              <Box sx={{ p: 2, borderBottom: 1, borderColor: 'primary.main' }}>
                <Typography variant="subtitle2" gutterBottom>Use existing instructions?</Typography>
                <Typography variant="body2" color="text.secondary">
                  Your project has a <Typography component="span" variant="caption" sx={{ fontFamily: 'monospace', bgcolor: 'action.hover', px: 0.5 }}>CLAUDE.md</Typography> file.
                  Would you like to use it as your constitution?
                </Typography>
              </Box>

              {/* Preview */}
              <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
                {claudeMdContent ? (
                  <Typography component="div" variant="body2" sx={{ '& pre': { overflow: 'auto' } }}>
                    <ReactMarkdown>{claudeMdContent}</ReactMarkdown>
                  </Typography>
                ) : (
                  <LoadingState message="Loading preview..." />
                )}
              </Box>

              {/* Actions */}
              <Stack direction="row" spacing={2} sx={{ p: 2, borderTop: 1, borderColor: 'divider', bgcolor: 'background.paper' }}>
                <Button
                  variant="contained"
                  fullWidth
                  onClick={handleImportClaudeMd}
                  startIcon={<CheckCircleIcon />}
                >
                  Use This
                </Button>
                <Button
                  variant="outlined"
                  fullWidth
                  onClick={handleSkipClaudeMd}
                >
                  Skip, Create New
                </Button>
              </Stack>
            </Paper>
          </Box>
        </Stack>
      )
    }

    // Constitution exists - show content (only when no active workflow)
    if (constitutionExists === true && !workflow) {
      return (
        <Stack sx={{ height: '100%' }}>
          <PageHeader
            title="Constitution"
            description="Governance rules for AI development"
            icon={<CheckCircleIcon color="success" />}
          >
            <Button variant="outlined" size="small" onClick={handleStartQA} startIcon={<RefreshIcon />}>
              Regenerate
            </Button>
          </PageHeader>
          <Box sx={{ flex: 1, overflow: 'auto', px: 3, pb: 3 }}>
            {constitutionContent ? (
              <Card elevation={0} variant="outlined">
                <CardContent>
                  <Typography component="div" variant="body2" sx={{ '& h1, & h2, & h3': { mt: 2, mb: 1, fontWeight: 600 }, '& ul': { pl: 2 }, '& pre': { bgcolor: 'action.hover', p: 1, borderRadius: 1, overflow: 'auto' } }}>
                    <ReactMarkdown>{constitutionContent}</ReactMarkdown>
                  </Typography>
                </CardContent>
              </Card>
            ) : (
              <LoadingState message="Loading constitution..." />
            )}
          </Box>
        </Stack>
      )
    }

    // Constitution missing - show initial options (only when no active workflow)
    if (constitutionExists === false && !workflow) {
      return (
        <Stack sx={{ height: '100%' }}>
          <PageHeader
            title="Constitution Management"
            description="Initialize Constitution - Define development standards for AI-assisted coding"
            icon={<FileTextIcon />}
          />
          <Box sx={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', p: 3 }}>
            <Paper variant="outlined" sx={{ maxWidth: 480, width: '100%', p: 4, bgcolor: 'surfaceContainerHigh.main' }}>
              <Stack spacing={3}>
                <Box>
                  <Button variant="contained" fullWidth onClick={handleApplyDefault} startIcon={<SparklesIcon />}>
                    Apply Default Template
                  </Button>
                  <Typography variant="caption" display="block" align="center" color="text.secondary" sx={{ mt: 1 }}>
                    Auto-detects languages and creates modular rules
                  </Typography>
                </Box>

                <Divider>
                  <Typography variant="caption" color="text.secondary">OR</Typography>
                </Divider>

                <Box>
                  <Button variant="outlined" fullWidth onClick={handleStartQA} startIcon={<FileTextIcon />}>
                    Create with Q&A
                  </Button>
                  <Typography variant="caption" display="block" align="center" color="text.secondary" sx={{ mt: 1 }}>
                    Answer questions to generate a custom module
                  </Typography>
                </Box>
              </Stack>
            </Paper>
          </Box>
        </Stack>
      )
    }

    // Workflow active - show workflow phases
    if (!workflow) {
      return <LoadingState />
    }

    const currentQuestionIndex = workflow.current_question
    const status = workflow.status
    const output = workflow.output
    const error = workflow.error

    // Collecting answers phase
    if (status === 'collecting') {
      const allQuestionsAnswered = currentQuestionIndex >= questions.length
      const currentQ = questions[currentQuestionIndex]
      const currentSelections = currentQ ? selectedOptions[currentQ.key] ?? [] : []
      const currentNotes = currentQ ? customNotes[currentQ.key] ?? '' : ''
      const hasCurrentAnswer = currentSelections.length > 0 || currentNotes.trim().length > 0

      return (
        <Stack sx={{ height: '100%' }}>
          <WorkflowHeader
            title="Initialize Constitution"
            subtitle={`${currentQuestionIndex} / ${questions.length} questions answered`}
            icon={<FileTextIcon color="primary" />}
          />

          <Box sx={{ flex: 1, overflow: 'auto', p: 3, pt: 0 }}>
            {/* CLAUDE.md Reference Option */}
            {claudeMdExists && (
              <Paper variant="outlined" sx={{ mb: 3, p: 2, bgcolor: 'secondary.container', borderColor: 'secondary.main' }}>
                <Stack spacing={1}>
                  <FormControlLabel
                    control={
                      <Checkbox
                        checked={workflow.use_claude_md_reference}
                        onChange={(e) => handleToggleClaudeMdReference(e.target.checked)}
                      />
                    }
                    label={<Typography variant="subtitle2">Reference existing CLAUDE.md</Typography>}
                  />
                  <Typography variant="caption" color="text.secondary" sx={{ pl: 4, display: 'block', mt: -1 }}>
                    Include your project's CLAUDE.md as context for generation
                  </Typography>

                  {claudeMdContent && (
                    <Box sx={{ pl: 4 }}>
                      <Button
                        size="small"
                        onClick={() => setPreviewOpen(!previewOpen)}
                        endIcon={<ExpandMoreIcon sx={{ transform: previewOpen ? 'rotate(180deg)' : 'none', transition: 'transform 0.2s' }} />}
                        sx={{ textTransform: 'none', p: 0, minWidth: 0, color: 'primary.main' }}
                      >
                        Preview
                      </Button>
                      <Collapse in={previewOpen}>
                        <Paper variant="outlined" sx={{ mt: 1, p: 2, maxHeight: 150, overflow: 'auto', bgcolor: 'background.paper' }}>
                          <Typography component="div" variant="caption">
                            <ReactMarkdown>{claudeMdContent}</ReactMarkdown>
                          </Typography>
                        </Paper>
                      </Collapse>
                    </Box>
                  )}
                </Stack>
              </Paper>
            )}

            {/* Progress */}
            <Stack spacing={1} sx={{ mb: 4 }}>
              {questions.map((q, idx) => (
                <Stack
                  key={q.key}
                  direction="row"
                  alignItems="center"
                  spacing={1}
                  sx={{
                    color: idx < currentQuestionIndex ? 'text.secondary' : idx === currentQuestionIndex ? 'text.primary' : 'action.disabled'
                  }}
                >
                  {idx < currentQuestionIndex ? (
                    <CheckCircleIcon fontSize="small" color="success" />
                  ) : (
                    <Box sx={{ width: 14, height: 14, borderRadius: '50%', border: 1, borderColor: 'currentColor' }} />
                  )}
                  <Typography variant="caption">{q.question}</Typography>
                </Stack>
              ))}
            </Stack>

            {/* Current Question */}
            {!allQuestionsAnswered && currentQ && (
              <Card variant="outlined">
                <CardContent>
                  <Typography variant="subtitle1" gutterBottom fontWeight={600}>{currentQ.question}</Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>{currentQ.hint}</Typography>

                  <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1, mb: 3 }}>
                    {currentQ.options.map((option) => {
                      const isSelected = currentSelections.includes(option)
                      return (
                        <Chip
                          key={option}
                          label={option}
                          onClick={() => toggleOption(currentQ.key, option)}
                          color={isSelected ? 'primary' : 'default'}
                          variant={isSelected ? 'filled' : 'outlined'}
                          sx={{ borderRadius: 1 }}
                        />
                      )
                    })}
                  </Box>

                  <TextField
                    fullWidth
                    multiline
                    minRows={3}
                    placeholder="Optional notes (keep it short)"
                    value={currentNotes}
                    onChange={(e) => setCustomNotes((prev) => ({ ...prev, [currentQ.key]: e.target.value }))}
                    sx={{ mb: 3 }}
                  />

                  <Button
                    fullWidth
                    variant="contained"
                    disabled={!hasCurrentAnswer}
                    onClick={() => handleAnswerSubmit(currentQ.key)}
                    endIcon={<ChevronRightIcon />}
                  >
                    Next
                  </Button>
                </CardContent>
              </Card>
            )}

            {/* All questions answered - ready to generate */}
            {allQuestionsAnswered && (
              <Paper variant="outlined" sx={{ p: 3, bgcolor: 'success.container', borderColor: 'success.main' }}>
                <Stack spacing={2} alignItems="center">
                  <CheckCircleIcon color="success" fontSize="large" />
                  <Typography variant="h6" align="center">All questions answered!</Typography>
                  <Typography variant="body2" color="text.secondary" align="center">
                    Ready to generate your project constitution using Claude.
                  </Typography>
                  <Button variant="contained" color="success" fullWidth onClick={handleGenerate}>
                    Generate Constitution
                  </Button>
                </Stack>
              </Paper>
            )}
          </Box>
        </Stack>
      )
    }

    // Generating phase
    if (status === 'generating') {
      return (
        <Stack sx={{ height: '100%' }}>
          <WorkflowHeader
            title="Generating Constitution"
            subtitle="Claude is creating your rules"
            icon={<RefreshIcon sx={{ animation: 'spin 2s linear infinite' }} color="primary" />}
          />

          <Box sx={{ flex: 1, overflow: 'auto', p: 3, pt: 0 }}>
            <Card variant="outlined">
              <CardContent>
                <Typography component="div" variant="body2">
                  <ReactMarkdown>{output || 'Waiting for Claude...'}</ReactMarkdown>
                </Typography>
                <Stack direction="row" alignItems="center" spacing={1} sx={{ mt: 2, color: 'text.secondary' }}>
                  <RefreshIcon fontSize="small" sx={{ animation: 'spin 2s linear infinite' }} />
                  <Typography variant="caption">Streaming from Claude Code...</Typography>
                </Stack>
              </CardContent>
            </Card>
          </Box>
          <style>{`
            @keyframes spin {
              0% { transform: rotate(0deg); }
              100% { transform: rotate(360deg); }
            }
          `}</style>
        </Stack>
      )
    }

    // Error phase
    if (status === 'error') {
      return (
        <Stack sx={{ height: '100%' }}>
          <WorkflowHeader
            title="Generation Failed"
            subtitle="An error occurred while generating"
            icon={<XCircleIcon color="error" />}
          />

          <Box sx={{ flex: 1, overflow: 'auto', p: 3, pt: 0 }}>
            <Paper variant="outlined" sx={{ mb: 2, p: 2, bgcolor: 'error.container', borderColor: 'error.main' }}>
              <Stack direction="row" spacing={2}>
                <AlertCircleIcon color="error" />
                <Box>
                  <Typography variant="subtitle2" color="error.main">Error</Typography>
                  <Typography variant="body2" color="error.dark">{error || 'Unknown error occurred'}</Typography>
                </Box>
              </Stack>
            </Paper>

            {output && (
              <Card variant="outlined" sx={{ mt: 2 }}>
                <CardContent>
                  <Typography variant="caption" color="text.secondary" gutterBottom>Partial output before failure:</Typography>
                  <Typography component="div" variant="body2">
                    <ReactMarkdown>{output}</ReactMarkdown>
                  </Typography>
                </CardContent>
              </Card>
            )}

            <Button
              variant="outlined"
              onClick={() => dispatch({ type: 'ClearConstitutionWorkflow' })}
              sx={{ mt: 2 }}
            >
              Start Over
            </Button>
          </Box>
        </Stack>
      )
    }

    // Complete phase
    if (status === 'complete') {
      return (
        <Stack sx={{ height: '100%' }}>
          <WorkflowHeader
            title="Constitution Generated"
            subtitle="Saved to .rstn/constitutions/custom.md"
            icon={<CheckCircleIcon color="success" />}
          />

          <Box sx={{ flex: 1, overflow: 'auto', p: 3, pt: 0 }}>
            <Paper variant="outlined" sx={{ mb: 2, p: 2, bgcolor: 'success.container', borderColor: 'success.main' }}>
              <Stack direction="row" spacing={1} alignItems="center">
                <CheckCircleIcon color="success" fontSize="small" />
                <Typography variant="caption" fontWeight={500}>
                  Constitution saved to .rstn/constitutions/custom.md
                </Typography>
              </Stack>
            </Paper>

            <Card variant="outlined">
              <CardContent>
                <Typography component="div" variant="body2">
                  <ReactMarkdown>{output}</ReactMarkdown>
                </Typography>
              </CardContent>
            </Card>
          </Box>
        </Stack>
      )
    }

    // Fallback
    return <EmptyState title="Unknown State" description="The workflow entered an invalid state." />
  }
}
