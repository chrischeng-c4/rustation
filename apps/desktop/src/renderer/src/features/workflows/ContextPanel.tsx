import { useCallback, useEffect } from 'react'
import {
  FileText,
  RefreshCw,
  Clock,
  Wand2,
  Loader2,
  BookOpen,
  Plus,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { PageHeader } from '@/components/shared/PageHeader'
import { WorkflowHeader } from '@/components/shared/WorkflowHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { ErrorBanner } from '@/components/shared/ErrorBanner'
import { useAppState } from '@/hooks/useAppState'
import ReactMarkdown from 'react-markdown'

/**
 * Context viewing panel for Living Context Layer (CESDD Phase 3).
 * Displays context files from .rstn/context/ directory.
 * Supports AI-powered context generation and sync.
 */
export function ContextPanel() {
  const { state, dispatch, isLoading } = useAppState()

  const activeProject = state?.projects?.[state?.active_project_index ?? 0]
  const worktree = activeProject?.worktrees?.[activeProject?.active_worktree_index ?? 0]
  const context = worktree?.context
  const contextFiles = context?.files ?? []
  const isInitialized = context?.is_initialized
  const lastRefreshed = context?.last_refreshed

  // AI Generation state
  const isGenerating = context?.is_generating ?? false
  const generationOutput = context?.generation_output ?? ''
  const generationError = context?.generation_error

  // Context Sync state
  const isSyncing = context?.is_syncing ?? false
  const syncOutput = context?.sync_output ?? ''
  const syncError = context?.sync_error

  // Check constitution on mount (triggers CheckConstitutionExists in handler)
  useEffect(() => {
    // Trigger context check when panel mounts
    dispatch({ type: 'RefreshContext' })
  }, [dispatch])

  const handleInitialize = useCallback(async () => {
    await dispatch({ type: 'InitializeContext' })
  }, [dispatch])

  const handleGenerateContext = useCallback(async () => {
    await dispatch({ type: 'GenerateContext' })
  }, [dispatch])

  const handleRefresh = useCallback(async () => {
    await dispatch({ type: 'RefreshContext' })
  }, [dispatch])

  // Loading state
  if (isLoading || context?.is_loading) {
    return <LoadingState message="Loading project context..." />
  }

  // AI Generation in progress
  if (isGenerating) {
    return (
      <div className="flex h-full flex-col">
        <WorkflowHeader
          title="Generating Context"
          subtitle="Analyzing codebase"
          icon={<Loader2 className="h-4 w-4 animate-spin text-blue-500" />}
        />
        <ScrollArea className="flex-1 p-4 pt-0">
          <div className="space-y-4">
            <Card className="p-4 bg-muted/30">
              <pre className="text-xs font-mono whitespace-pre-wrap text-muted-foreground">
                {generationOutput || 'Analyzing codebase...'}
              </pre>
            </Card>
            {generationError && <ErrorBanner error={generationError} />}
          </div>
        </ScrollArea>
      </div>
    )
  }

  // Context Sync in progress
  if (isSyncing) {
    return (
      <div className="flex h-full flex-col">
        <WorkflowHeader
          title="Syncing Context"
          subtitle="Extracting latest changes"
          icon={<Loader2 className="h-4 w-4 animate-spin text-green-500" />}
        />
        <ScrollArea className="flex-1 p-4 pt-0">
          <div className="space-y-4">
            <Card className="p-4 bg-muted/30">
              <pre className="text-xs font-mono whitespace-pre-wrap text-muted-foreground">
                {syncOutput || 'Extracting context updates...'}
              </pre>
            </Card>
            {syncError && <ErrorBanner error={syncError} />}
          </div>
        </ScrollArea>
      </div>
    )
  }

  // Context not initialized
  if (!isInitialized) {
    return (
      <div className="flex h-full flex-col">
        <PageHeader
          title="Living Context"
          description="Source of truth for project knowledge"
          icon={<BookOpen className="h-5 w-5 text-blue-500" />}
        />
        <div className="flex-1 px-4 pb-4">
          <div className="max-w-md mx-auto h-full flex flex-col justify-center gap-4">
            <Card className="p-6 border-blue-500/50 bg-blue-50 dark:bg-blue-950/20">
              <h3 className="text-lg font-medium mb-2">Initialize Living Context</h3>
              <p className="text-sm text-muted-foreground mb-4">
                Living context provides AI with project knowledge including tech stack, architecture, and recent changes.
              </p>

              <div className="space-y-3">
                <Button className="w-full" onClick={handleGenerateContext}>
                  <Wand2 className="mr-2 h-4 w-4" />
                  Generate with AI
                </Button>
                <p className="text-xs text-center text-muted-foreground">
                  Analyzes codebase and generates comprehensive context files
                </p>

                <div className="relative py-2">
                  <div className="absolute inset-0 flex items-center">
                    <span className="w-full border-t" />
                  </div>
                  <div className="relative flex justify-center text-xs">
                    <span className="bg-background px-2 text-muted-foreground">or</span>
                  </div>
                </div>

                <Button variant="outline" className="w-full" onClick={handleInitialize}>
                  <FileText className="mr-2 h-4 w-4" />
                  Use Templates
                </Button>
                <p className="text-xs text-center text-muted-foreground">
                  Create empty template files to fill in manually
                </p>
              </div>
            </Card>
            <p className="text-xs text-center text-muted-foreground">
              Context files are stored in <code>.rstn/context/</code>
            </p>
          </div>
        </div>
      </div>
    )
  }

  // Context exists - show files
  return (
    <div className="flex h-full flex-col">
      <WorkflowHeader
        title="Living Context"
        subtitle="Project memory and architectural source of truth"
        icon={<BookOpen className="h-4 w-4 text-blue-500" />}
        status={`${contextFiles.length} files`}
        statusColor="bg-blue-500/10 text-blue-600"
      >
        <div className="flex items-center gap-2">
          {lastRefreshed && (
            <span className="text-[10px] text-muted-foreground flex items-center gap-1 bg-muted px-2 py-1 rounded-full mr-2">
              <Clock className="h-3 w-3" />
              {new Date(lastRefreshed).toLocaleTimeString()}
            </span>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={handleGenerateContext}
            title="Regenerate context with AI"
            className="h-8 gap-1.5"
          >
            <Wand2 className="h-3.5 w-3.5" />
            AI Refresh
          </Button>
          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={handleRefresh} title="Refresh">
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </WorkflowHeader>

      <div className="flex-1 flex flex-col overflow-hidden px-4 pb-4 pt-4">
        {contextFiles.length === 0 ? (
          <EmptyState
            icon={FileText}
            title="No Context Files"
            description="No context files found in .rstn/context/ directory."
            action={{
              label: "Initialize Templates",
              onClick: handleInitialize,
              icon: Plus
            }}
          />
        ) : (
          <Tabs defaultValue={contextFiles[0]?.name} className="flex-1 flex flex-col border rounded-lg bg-background overflow-hidden">
            <div className="border-b bg-muted/20 px-2 py-1">
              <TabsList className="h-9 bg-transparent w-full justify-start gap-1 p-0">
                {contextFiles.map((file) => (
                  <TabsTrigger
                    key={file.name}
                    value={file.name}
                    className="text-xs gap-1.5 data-[state=active]:bg-background data-[state=active]:shadow-sm"
                  >
                    <FileText className="h-3.5 w-3.5 text-muted-foreground" />
                    {formatContextName(file.name)}
                  </TabsTrigger>
                ))}
              </TabsList>
            </div>

            {contextFiles.map((file) => (
              <TabsContent key={file.name} value={file.name} className="flex-1 m-0 overflow-hidden">
                <ScrollArea className="h-full">
                  <div className="p-6">
                    {/* File metadata */}
                    <div className="mb-6 flex items-center gap-4 text-[10px] uppercase tracking-wider font-semibold text-muted-foreground/70">
                      <span className="flex items-center gap-1.5">
                        <Badge variant="outline" className="text-[10px] px-1.5 h-5 bg-muted/50 border-none">
                          {file.context_type}
                        </Badge>
                      </span>
                      {file.last_updated && (
                        <span className="flex items-center gap-1">
                          <Clock className="h-3 w-3" />
                          Updated {formatDate(file.last_updated)}
                        </span>
                      )}
                      <span>~{file.token_estimate} tokens</span>
                    </div>

                    {/* File content */}
                    <div className="prose prose-sm dark:prose-invert max-w-none prose-headings:font-semibold prose-h1:text-xl prose-h2:text-lg prose-h3:text-base">
                      <ReactMarkdown>{file.content}</ReactMarkdown>
                    </div>
                  </div>
                </ScrollArea>
              </TabsContent>
            ))}
          </Tabs>
        )}
      </div>
    </div>
  )
}

/** Format context file name for display */
function formatContextName(name: string): string {
  return name
    .replace(/-/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase())
}

/** Format ISO date string for display */
function formatDate(isoDate: string): string {
  try {
    return new Date(isoDate).toLocaleDateString()
  } catch {
    return isoDate
  }
}
