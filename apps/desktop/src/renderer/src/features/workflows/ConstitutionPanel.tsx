import { useState, useCallback } from 'react'
import { FileText, RefreshCw, CheckCircle, ChevronRight, AlertCircle, Sparkles } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Textarea } from '@/components/ui/textarea'
import { useAppState } from '@/hooks/useAppState'
import ReactMarkdown from 'react-markdown'

/**
 * Constitution initialization workflow panel.
 * Guides user through questions and generates .rstn/constitution.md via Claude.
 * Also supports one-click default constitution application.
 */
export function ConstitutionPanel() {
  const { state, dispatch, isLoading } = useAppState()
  const [currentAnswer, setCurrentAnswer] = useState('')

  // Note: active_project is not serialized, use projects[active_project_index]
  const activeProject = state?.projects?.[state?.active_project_index ?? 0]
  const worktree = activeProject?.worktrees?.[activeProject?.active_worktree_index ?? 0]
  const workflow = worktree?.tasks?.constitution_workflow
  const constitutionExists = worktree?.tasks?.constitution_exists

  const questions = [
    {
      key: 'tech_stack',
      question: 'What technology stack does this project use?',
      hint: 'e.g., React + Rust, Python + Django',
    },
    {
      key: 'security',
      question: 'What security requirements must all code follow?',
      hint: 'e.g., JWT auth required, no SQL injection',
    },
    {
      key: 'code_quality',
      question: 'What code quality standards?',
      hint: 'e.g., 80% test coverage, ESLint rules',
    },
    {
      key: 'architecture',
      question: 'Any architectural constraints?',
      hint: 'e.g., state-first, no singletons',
    },
  ]

  const handleApplyDefault = useCallback(async () => {
    await dispatch({ type: 'ApplyDefaultConstitution' })
  }, [dispatch])

  const handleStartQA = useCallback(async () => {
    await dispatch({ type: 'StartConstitutionWorkflow' })
  }, [dispatch])

  const handleAnswerSubmit = useCallback(async () => {
    if (!currentAnswer.trim()) return

    await dispatch({
      type: 'AnswerConstitutionQuestion',
      payload: { answer: currentAnswer.trim() },
    })
    setCurrentAnswer('')
  }, [currentAnswer, dispatch])

  const handleGenerate = useCallback(async () => {
    await dispatch({ type: 'GenerateConstitution' })
  }, [dispatch])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        handleAnswerSubmit()
      }
    },
    [handleAnswerSubmit]
  )

  // Loading state - checking existence
  if (isLoading || constitutionExists === null || constitutionExists === undefined) {
    return (
      <div className="flex h-full items-center justify-center rounded-lg border">
        <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
        <span className="ml-2 text-sm text-muted-foreground">Checking constitution...</span>
      </div>
    )
  }

  // Constitution exists - show success state (only when no active workflow)
  if (constitutionExists === true && !workflow) {
    return (
      <div className="flex h-full flex-col rounded-lg border">
        <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
          <div className="flex items-center gap-2">
            <CheckCircle className="h-4 w-4 text-green-500" />
            <span className="text-sm font-medium">Constitution Active</span>
          </div>
        </div>
        <div className="flex flex-1 items-center justify-center p-4">
          <Card className="p-6 text-center border-green-500/50 bg-green-50 dark:bg-green-950/20">
            <CheckCircle className="mx-auto h-12 w-12 text-green-500 mb-4" />
            <h3 className="text-lg font-medium mb-2">Constitution Exists</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Your project has a constitution at <code className="text-xs">.rstn/constitutions/</code>
            </p>
            <Button variant="outline" size="sm" onClick={handleStartQA}>
              Regenerate with Q&A
            </Button>
          </Card>
        </div>
      </div>
    )
  }

  // Constitution missing - show initial options (only when no active workflow)
  if (constitutionExists === false && !workflow) {
    return (
      <div className="flex h-full flex-col rounded-lg border">
        <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
          <div className="flex items-center gap-2">
            <AlertCircle className="h-4 w-4 text-amber-500" />
            <span className="text-sm font-medium">No Constitution</span>
          </div>
        </div>
        <div className="flex flex-1 items-center justify-center p-4">
          <div className="max-w-md space-y-4">
            <Card className="p-6 border-blue-500/50 bg-blue-50 dark:bg-blue-950/20">
              <h3 className="text-lg font-medium mb-2">Initialize Constitution</h3>
              <p className="text-sm text-muted-foreground mb-4">
                A constitution defines development standards for AI-assisted coding.
              </p>

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
                  Answer questions to generate a customized constitution
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
    return (
      <div className="flex h-full items-center justify-center rounded-lg border">
        <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  const currentQuestionIndex = workflow.current_question
  const status = workflow.status
  const output = workflow.output

  // Collecting answers phase
  if (status === 'collecting') {
    const allQuestionsAnswered = currentQuestionIndex >= questions.length
    const currentQ = questions[currentQuestionIndex]

    return (
      <div className="flex h-full flex-col rounded-lg border">
        {/* Header */}
        <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
          <div className="flex items-center gap-2">
            <FileText className="h-4 w-4 text-blue-500" />
            <span className="text-sm font-medium">Initialize Constitution</span>
          </div>
          <span className="text-xs text-muted-foreground">
            {currentQuestionIndex} / {questions.length}
          </span>
        </div>

        <ScrollArea className="flex-1 p-4">
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

              <Textarea
                value={currentAnswer}
                onChange={(e) => setCurrentAnswer(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Type your answer..."
                className="min-h-[100px] resize-none text-sm mb-3"
                autoFocus
              />

              <Button
                onClick={handleAnswerSubmit}
                disabled={!currentAnswer.trim()}
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
      <div className="flex h-full flex-col rounded-lg border">
        {/* Header */}
        <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
          <div className="flex items-center gap-2">
            <RefreshCw className="h-4 w-4 animate-spin text-blue-500" />
            <span className="text-sm font-medium">Generating Constitution...</span>
          </div>
        </div>

        <ScrollArea className="flex-1 p-4">
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

  // Complete phase
  if (status === 'complete') {
    return (
      <div className="flex h-full flex-col rounded-lg border">
        {/* Header */}
        <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
          <div className="flex items-center gap-2">
            <CheckCircle className="h-4 w-4 text-green-500" />
            <span className="text-sm font-medium">Constitution Generated</span>
          </div>
        </div>

        <ScrollArea className="flex-1 p-4">
          <Card className="p-4 mb-3 border-green-500/50 bg-green-50 dark:bg-green-950/20">
            <div className="flex items-center gap-2">
              <CheckCircle className="h-4 w-4 text-green-500" />
              <span className="text-xs font-medium">
                Constitution saved to <code className="text-xs">.rstn/constitution.md</code>
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

  // Fallback (shouldn't happen)
  return (
    <div className="flex h-full items-center justify-center rounded-lg border">
      <p className="text-sm text-muted-foreground">Unknown workflow status</p>
    </div>
  )
}
