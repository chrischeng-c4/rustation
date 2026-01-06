import { useState, useCallback, useEffect } from 'react'
import { FileText, RefreshCw, CheckCircle, ChevronRight, Sparkles, FileCode, ChevronDown, Scroll, AlertCircle, XCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Checkbox } from '@/components/ui/checkbox'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { PageHeader } from '@/components/shared/PageHeader'
import { WorkflowHeader } from '@/components/shared/WorkflowHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { useAppState } from '@/hooks/useAppState'
import ReactMarkdown from 'react-markdown'
import { Textarea } from '@/components/ui/textarea'

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
 * Guides user through questions and generates .rstn/constitutions/custom.md via Claude.
 * Also supports one-click default constitution application.
 *
 * Flow:
 * 1. Check if constitution exists (.rstn/constitutions/ or legacy)
 * 2. Check if CLAUDE.md exists in project root
 * 3. If CLAUDE.md exists but no constitution → show preview with import option
 * 4. If neither exists → show "Apply Default" / "Create with Q&A" options
 */
export function ConstitutionPanel() {
  const { state, dispatch, isLoading } = useAppState()
  const [selectedOptions, setSelectedOptions] = useState<Record<string, string[]>>({})
  const [customNotes, setCustomNotes] = useState<Record<string, string>>({})

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
      <div className="flex h-full flex-col">
        <PageHeader
          title="Found CLAUDE.md"
          description="Existing project instructions detected"
          icon={<FileCode className="h-5 w-5 text-blue-500" />}
        />
        <div className="flex min-h-0 flex-1 flex-col p-4 pt-0">
          <Card className="flex min-h-0 flex-1 flex-col border-blue-500/50 bg-blue-50/50 dark:bg-blue-950/20">
            <div className="p-4 border-b">
              <h3 className="text-sm font-medium mb-1">Use existing instructions?</h3>
              <p className="text-xs text-muted-foreground">
                Your project has a <code className="text-xs bg-muted px-1 rounded">CLAUDE.md</code> file.
                Would you like to use it as your constitution?
              </p>
            </div>

            {/* Preview */}
            <ScrollArea className="flex min-h-0 flex-1 p-4">
              {claudeMdContent ? (
                <div className="prose prose-sm dark:prose-invert max-w-none">
                  <ReactMarkdown>{claudeMdContent}</ReactMarkdown>
                </div>
              ) : (
                <LoadingState message="Loading preview..." className="h-full" />
              )}
            </ScrollArea>

            {/* Actions */}
            <div className="p-4 border-t bg-muted/20 flex gap-2">
              <Button className="flex-1" onClick={handleImportClaudeMd}>
                <CheckCircle className="mr-2 h-4 w-4" />
                Use This
              </Button>
              <Button variant="outline" className="flex-1" onClick={handleSkipClaudeMd}>
                Skip, Create New
              </Button>
            </div>
          </Card>
        </div>
      </div>
    )
  }

  // Constitution exists - show content (only when no active workflow)
  if (constitutionExists === true && !workflow) {
    return (
      <div className="flex h-full flex-col">
        <PageHeader
          title="Constitution"
          description="Governance rules for AI development"
          icon={<CheckCircle className="h-5 w-5 text-green-500" />}
        >
          <Button variant="outline" size="sm" onClick={handleStartQA}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Regenerate
          </Button>
        </PageHeader>
        <ScrollArea className="flex-1 px-4 pb-4">
          {constitutionContent ? (
            <Card className="p-4">
              <div className="prose prose-sm dark:prose-invert max-w-none">
                <ReactMarkdown>{constitutionContent}</ReactMarkdown>
              </div>
            </Card>
          ) : (
            <LoadingState message="Loading constitution..." />
          )}
        </ScrollArea>
      </div>
    )
  }

  // Constitution missing - show initial options (only when no active workflow)
  if (constitutionExists === false && !workflow) {
    return (
      <div className="flex h-full flex-col">
        <PageHeader
          title="Constitution Management"
          description="Initialize Constitution - Define development standards for AI-assisted coding"
          icon={<Scroll className="h-5 w-5" />}
        />
        <div className="flex flex-1 items-center justify-center p-4">
          <div className="max-w-md w-full space-y-4">
            <Card className="p-6 border-blue-500/50 bg-blue-50 dark:bg-blue-950/20">
              <div className="space-y-3">
                <Button className="w-full" onClick={handleApplyDefault}>
                  <Sparkles className="mr-2 h-4 w-4" />
                  Apply Default Template
                </Button>
                <p className="text-xs text-center text-muted-foreground">
                  Auto-detects languages and creates modular rules
                </p>

                <div className="relative py-2">
                  <div className="absolute inset-0 flex items-center">
                    <span className="w-full border-t" />
                  </div>
                  <div className="relative flex justify-center text-xs">
                    <span className="bg-background px-2 text-muted-foreground">or</span>
                  </div>
                </div>

                <Button variant="outline" className="w-full" onClick={handleStartQA}>
                  <FileText className="mr-2 h-4 w-4" />
                  Create with Q&A
                </Button>
                <p className="text-xs text-center text-muted-foreground">
                  Answer questions to generate a custom module
                </p>
              </div>
            </Card>
          </div>
        </div>
      </div>
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
    const hasCurrentAnswer =
      currentSelections.length > 0 || currentNotes.trim().length > 0

    return (
      <div className="flex h-full flex-col">
        <WorkflowHeader
          title="Initialize Constitution"
          subtitle={`${currentQuestionIndex} / ${questions.length} questions answered`}
          icon={<FileText className="h-4 w-4 text-blue-500" />}
        />

        <ScrollArea className="flex-1 p-4 pt-0">
          {/* CLAUDE.md Reference Option */}
          {claudeMdExists && (
            <Card className="mb-4 p-3 border-blue-500/30 bg-blue-50/50 dark:bg-blue-950/20">
              <div className="flex items-start gap-3">
                <Checkbox
                  id="use-claude-md"
                  checked={workflow.use_claude_md_reference}
                  onCheckedChange={(checked) =>
                    handleToggleClaudeMdReference(!!checked)
                  }
                  className="mt-0.5"
                />
                <div className="flex-1 min-w-0">
                  <label htmlFor="use-claude-md" className="text-sm font-medium cursor-pointer">
                    Reference existing CLAUDE.md
                  </label>
                  <p className="text-xs text-muted-foreground mt-0.5">
                    Include your project's CLAUDE.md as context for generation
                  </p>
                  {claudeMdContent && (
                    <Collapsible>
                      <CollapsibleTrigger className="flex items-center gap-1 text-xs text-blue-600 dark:text-blue-400 mt-2 hover:underline">
                        <ChevronDown className="h-3 w-3" />
                        Preview
                      </CollapsibleTrigger>
                      <CollapsibleContent>
                        <ScrollArea className="h-32 mt-2 rounded border bg-muted/30 p-2">
                          <div className="prose prose-xs dark:prose-invert max-w-none">
                            <ReactMarkdown>{claudeMdContent}</ReactMarkdown>
                          </div>
                        </ScrollArea>
                      </CollapsibleContent>
                    </Collapsible>
                  )}
                </div>
              </div>
            </Card>
          )}

          {/* Progress */}
          <div className="mb-4 space-y-2">
            {questions.map((q, idx) => (
              <div
                key={q.key}
                className={`flex items-center gap-2 text-xs ${
                  idx < currentQuestionIndex
                    ? 'text-muted-foreground'
                    : idx === currentQuestionIndex
                      ? 'text-foreground'
                      : 'text-muted-foreground/50'
                }`}
              >
                {idx < currentQuestionIndex ? (
                  <CheckCircle className="h-3.5 w-3.5 text-green-500" />
                ) : (
                  <div className="h-3.5 w-3.5 rounded-full border" />
                )}
                <span>{q.question}</span>
              </div>
            ))}
          </div>

          {/* Current Question */}
          {!allQuestionsAnswered && currentQ && (
            <Card className="p-4">
              <h3 className="text-sm font-medium mb-1">{currentQ.question}</h3>
              <p className="text-xs text-muted-foreground mb-3">{currentQ.hint}</p>

              <div className="flex flex-wrap gap-2 mb-4">
                {currentQ.options.map((option) => {
                  const isSelected = currentSelections.includes(option)
                  return (
                    <Button
                      key={option}
                      type="button"
                      size="sm"
                      variant={isSelected ? 'default' : 'secondary'}
                      className="h-7 px-2 text-xs"
                      onClick={() => toggleOption(currentQ.key, option)}
                    >
                      {option}
                    </Button>
                  )
                })}
              </div>

              <Textarea
                value={currentNotes}
                onChange={(e) =>
                  setCustomNotes((prev) => ({
                    ...prev,
                    [currentQ.key]: e.target.value,
                  }))
                }
                placeholder="Optional notes (keep it short)"
                className="min-h-[90px] resize-none text-sm mb-3"
              />

              <Button
                onClick={() => handleAnswerSubmit(currentQ.key)}
                disabled={!hasCurrentAnswer}
                className="w-full"
                size="sm"
              >
                Next
                <ChevronRight className="ml-1 h-4 w-4" />
              </Button>
            </Card>
          )}

          {/* All questions answered - ready to generate */}
          {allQuestionsAnswered && (
            <Card className="p-4 border-green-500/50 bg-green-50 dark:bg-green-950/20">
              <div className="flex items-center gap-2 mb-3">
                <CheckCircle className="h-5 w-5 text-green-500" />
                <h3 className="text-sm font-medium">All questions answered!</h3>
              </div>
              <p className="text-xs text-muted-foreground mb-4">
                Ready to generate your project constitution using Claude.
              </p>
              <Button onClick={handleGenerate} className="w-full" size="sm">
                Generate Constitution
              </Button>
            </Card>
          )}
        </ScrollArea>
      </div>
    )
  }

  // Generating phase
  if (status === 'generating') {
    return (
      <div className="flex h-full flex-col">
        <WorkflowHeader
          title="Generating Constitution"
          subtitle="Claude is creating your rules"
          icon={<RefreshCw className="h-4 w-4 animate-spin text-blue-500" />}
        />

        <ScrollArea className="flex-1 p-4 pt-0">
          <Card className="p-4">
            <div className="prose prose-sm dark:prose-invert max-w-none">
              <ReactMarkdown>{output || 'Waiting for Claude...'}</ReactMarkdown>
            </div>
            <div className="mt-2 flex items-center gap-2 text-xs text-muted-foreground">
              <RefreshCw className="h-3 w-3 animate-spin" />
              <span>Streaming from Claude Code...</span>
            </div>
          </Card>
        </ScrollArea>
      </div>
    )
  }

  // Error phase
  if (status === 'error') {
    return (
      <div className="flex h-full flex-col">
        <WorkflowHeader
          title="Generation Failed"
          subtitle="An error occurred while generating"
          icon={<XCircle className="h-4 w-4 text-red-500" />}
        />

        <ScrollArea className="flex-1 p-4 pt-0">
          <Card className="p-4 mb-3 border-red-500/50 bg-red-50 dark:bg-red-950/20">
            <div className="flex items-start gap-2">
              <AlertCircle className="h-4 w-4 text-red-500 mt-0.5 flex-shrink-0" />
              <div className="flex-1">
                <p className="text-sm font-medium text-red-700 dark:text-red-400">Error</p>
                <p className="text-xs text-red-600 dark:text-red-300 mt-1">{error || 'Unknown error occurred'}</p>
              </div>
            </div>
          </Card>

          {output && (
            <Card className="p-4 mt-3">
              <p className="text-xs font-medium text-muted-foreground mb-2">Partial output before failure:</p>
              <div className="prose prose-sm dark:prose-invert max-w-none">
                <ReactMarkdown>{output}</ReactMarkdown>
              </div>
            </Card>
          )}

          <div className="mt-4 flex gap-2">
            <Button
              onClick={() => dispatch({ type: 'ClearConstitutionWorkflow' })}
              variant="outline"
              size="sm"
            >
              Start Over
            </Button>
          </div>
        </ScrollArea>
      </div>
    )
  }

  // Complete phase
  if (status === 'complete') {
    return (
      <div className="flex h-full flex-col">
        <WorkflowHeader
          title="Constitution Generated"
          subtitle="Saved to .rstn/constitutions/custom.md"
          icon={<CheckCircle className="h-4 w-4 text-green-500" />}
        />

        <ScrollArea className="flex-1 p-4 pt-0">
          <Card className="p-4 mb-3 border-green-500/50 bg-green-50 dark:bg-green-950/20">
            <div className="flex items-center gap-2">
              <CheckCircle className="h-4 w-4 text-green-500" />
              <span className="text-xs font-medium">
                Constitution saved to <code className="text-xs">.rstn/constitutions/custom.md</code>
              </span>
            </div>
          </Card>

          <Card className="p-4">
            <div className="prose prose-sm dark:prose-invert max-w-none">
              <ReactMarkdown>{output}</ReactMarkdown>
            </div>
          </Card>
        </ScrollArea>
      </div>
    )
  }

    // Fallback
    return <EmptyState icon={AlertCircle} title="Unknown State" description="The workflow entered an invalid state." />
  }
}
