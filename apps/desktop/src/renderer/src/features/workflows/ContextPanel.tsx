import { useCallback, useEffect } from 'react'
import {
  FileText,
  RefreshCw,
  CheckCircle,
  AlertCircle,
  Sparkles,
  Clock,
  Wand2,
  Loader2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
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
    return (
      <div className="flex h-full items-center justify-center rounded-lg border">
        <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
        <span className="ml-2 text-sm text-muted-foreground">Loading context...</span>
      </div>
    )
  }

  // AI Generation in progress
  if (isGenerating) {
    return (
      <div className="flex h-full flex-col rounded-lg border">
        <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
          <div className="flex items-center gap-2">
            <Loader2 className="h-4 w-4 animate-spin text-blue-500" />
            <span className="text-sm font-medium">Generating Context with AI...</span>
          </div>
        </div>
        <ScrollArea className="flex-1">
          <div className="p-4">
            <Card className="p-4 bg-muted/30">
              <pre className="text-xs font-mono whitespace-pre-wrap text-muted-foreground">
                {generationOutput || 'Analyzing codebase...'}
              </pre>
            </Card>
            {generationError && (
              <Card className="mt-4 p-4 border-red-500/50 bg-red-50 dark:bg-red-950/20">
                <div className="flex items-center gap-2 text-red-600 dark:text-red-400">
                  <AlertCircle className="h-4 w-4" />
                  <span className="text-sm font-medium">Generation Error</span>
                </div>
                <p className="mt-2 text-sm text-red-600 dark:text-red-400">{generationError}</p>
              </Card>
            )}
          </div>
        </ScrollArea>
      </div>
    )
  }

  // Context Sync in progress
  if (isSyncing) {
    return (
      <div className="flex h-full flex-col rounded-lg border">
        <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
          <div className="flex items-center gap-2">
            <Loader2 className="h-4 w-4 animate-spin text-green-500" />
            <span className="text-sm font-medium">Syncing Context...</span>
          </div>
        </div>
        <ScrollArea className="flex-1">
          <div className="p-4">
            <Card className="p-4 bg-muted/30">
              <pre className="text-xs font-mono whitespace-pre-wrap text-muted-foreground">
                {syncOutput || 'Extracting context updates...'}
              </pre>
            </Card>
            {syncError && (
              <Card className="mt-4 p-4 border-red-500/50 bg-red-50 dark:bg-red-950/20">
                <div className="flex items-center gap-2 text-red-600 dark:text-red-400">
                  <AlertCircle className="h-4 w-4" />
                  <span className="text-sm font-medium">Sync Error</span>
                </div>
                <p className="mt-2 text-sm text-red-600 dark:text-red-400">{syncError}</p>
              </Card>
            )}
          </div>
        </ScrollArea>
      </div>
    )
  }

  // Context not initialized - show single card with two options (consistent with other panels)
  if (!isInitialized) {
    return (
      <div className="flex h-full flex-col rounded-lg border">
        <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
          <div className="flex items-center gap-2">
            <AlertCircle className="h-4 w-4 text-amber-500" />
            <span className="text-sm font-medium">No Living Context</span>
          </div>
        </div>
        <div className="flex flex-1 items-center justify-center p-4">
          <div className="max-w-md space-y-4">
            <Card className="p-6 border-blue-500/50 bg-blue-50 dark:bg-blue-950/20">
              <h3 className="text-lg font-medium mb-2">Initialize Living Context</h3>
              <p className="text-sm text-muted-foreground mb-4">
                Living context provides AI with project knowledge including tech stack, architecture, and recent changes.
              </p>

              <div className="space-y-3">
                <Button className="w-full" onClick={handleGenerateContext}>
                  <Sparkles className="mr-2 h-4 w-4" />
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
    <div className="flex h-full flex-col rounded-lg border">
      {/* Header */}
      <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
        <div className="flex items-center gap-2">
          <CheckCircle className="h-4 w-4 text-green-500" />
          <span className="text-sm font-medium">Living Context</span>
          <Badge variant="outline" className="text-xs">
            {contextFiles.length} files
          </Badge>
        </div>
        <div className="flex items-center gap-2">
          {lastRefreshed && (
            <span className="text-xs text-muted-foreground flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {new Date(lastRefreshed).toLocaleTimeString()}
            </span>
          )}
          <Button
            variant="ghost"
            size="sm"
            onClick={handleGenerateContext}
            title="Regenerate context with AI"
          >
            <Wand2 className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="sm" onClick={handleRefresh} title="Refresh">
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Content */}
      {contextFiles.length === 0 ? (
        <div className="flex flex-1 items-center justify-center p-4">
          <Card className="p-6 text-center">
            <FileText className="mx-auto h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No Context Files</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Context files will appear here after initialization or context sync.
            </p>
            <Button variant="outline" onClick={handleRefresh}>
              <RefreshCw className="mr-2 h-4 w-4" />
              Refresh
            </Button>
          </Card>
        </div>
      ) : (
        <Tabs defaultValue={contextFiles[0]?.name} className="flex-1 flex flex-col">
          <div className="border-b px-4 py-2">
            <TabsList className="h-auto flex-wrap gap-1">
              {contextFiles.map((file) => (
                <TabsTrigger key={file.name} value={file.name} className="text-xs gap-1">
                  <FileText className="h-3 w-3" />
                  {formatContextName(file.name)}
                </TabsTrigger>
              ))}
            </TabsList>
          </div>

          {contextFiles.map((file) => (
            <TabsContent key={file.name} value={file.name} className="flex-1 m-0">
              <ScrollArea className="h-full">
                <div className="p-4">
                  {/* File metadata */}
                  <div className="mb-4 flex items-center gap-4 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Badge variant="secondary" className="text-xs">
                        {file.context_type}
                      </Badge>
                    </span>
                    {file.last_updated && (
                      <span className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        Updated: {formatDate(file.last_updated)}
                      </span>
                    )}
                    <span>~{file.token_estimate} tokens</span>
                  </div>

                  {/* File content */}
                  <Card className="p-4">
                    <div className="prose prose-sm dark:prose-invert max-w-none">
                      <ReactMarkdown>{file.content}</ReactMarkdown>
                    </div>
                  </Card>
                </div>
              </ScrollArea>
            </TabsContent>
          ))}
        </Tabs>
      )}
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
