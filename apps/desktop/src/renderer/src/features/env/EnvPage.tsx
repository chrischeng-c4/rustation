import { useState, useCallback } from 'react'
import { Copy, RefreshCw, Settings2, ArrowRight } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { EnvPatternList } from './EnvPatternList'
import { WorktreeSelector } from './WorktreeSelector'
import { EnvCopyHistory } from './EnvCopyHistory'
import { useEnvState } from '@/hooks/useAppState'

/**
 * Environment Management Page.
 * Allows syncing dotfiles between worktrees in a project.
 */
export function EnvPage() {
  const { envConfig, project, worktrees, dispatch, isLoading } = useEnvState()

  // Local UI state for source/target selection
  const [selectedSource, setSelectedSource] = useState<string | null>(null)
  const [selectedTarget, setSelectedTarget] = useState<string | null>(null)
  const [isCopying, setIsCopying] = useState(false)

  // Use configured source or first worktree as default
  const effectiveSource =
    selectedSource ?? envConfig?.source_worktree ?? worktrees[0]?.path ?? null

  const handleAutoCopyToggle = useCallback(async () => {
    if (!envConfig) return
    await dispatch({
      type: 'SetEnvAutoCopy',
      payload: { enabled: !envConfig.auto_copy_enabled },
    })
  }, [envConfig, dispatch])

  const handlePatternsChange = useCallback(
    async (patterns: string[]) => {
      await dispatch({
        type: 'SetEnvTrackedPatterns',
        payload: { patterns },
      })
    },
    [dispatch]
  )

  const handleCopyEnvFiles = useCallback(async () => {
    if (!effectiveSource || !selectedTarget) return

    setIsCopying(true)
    try {
      await dispatch({
        type: 'CopyEnvFiles',
        payload: {
          from_worktree_path: effectiveSource,
          to_worktree_path: selectedTarget,
        },
      })
    } finally {
      setIsCopying(false)
    }
  }, [effectiveSource, selectedTarget, dispatch])

  // Loading state
  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  // No project open
  if (!project || !envConfig) {
    return (
      <div className="flex h-full flex-col items-center justify-center">
        <Settings2 className="h-12 w-12 text-muted-foreground" />
        <h2 className="mt-4 text-xl font-semibold">No Project Open</h2>
        <p className="mt-2 text-muted-foreground">
          Open a project to manage environment files.
        </p>
      </div>
    )
  }

  return (
    <ScrollArea className="h-full">
      <div className="space-y-6 p-4">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold">Environment</h2>
            <p className="mt-1 text-muted-foreground">
              Sync dotfiles across worktrees in {project.name}
            </p>
          </div>
          <Button
            variant={envConfig.auto_copy_enabled ? 'default' : 'outline'}
            onClick={handleAutoCopyToggle}
          >
            Auto-Copy: {envConfig.auto_copy_enabled ? 'ON' : 'OFF'}
          </Button>
        </div>

        {/* Manual Sync Card */}
        <Card className="p-4">
          <h3 className="mb-4 flex items-center gap-2 text-lg font-medium">
            <Copy className="h-5 w-5" />
            Manual Sync
          </h3>

          <div className="grid grid-cols-[1fr,auto,1fr] items-end gap-4">
            <WorktreeSelector
              worktrees={worktrees}
              value={effectiveSource}
              onChange={setSelectedSource}
              label="Source"
              placeholder="Select source worktree"
            />

            <ArrowRight className="mb-2 h-5 w-5 text-muted-foreground" />

            <WorktreeSelector
              worktrees={worktrees}
              value={selectedTarget}
              onChange={setSelectedTarget}
              label="Target"
              excludePaths={effectiveSource ? [effectiveSource] : []}
              placeholder="Select target worktree"
            />
          </div>

          <div className="mt-4 flex items-center justify-between">
            <p className="text-sm text-muted-foreground">
              Files: {envConfig.tracked_patterns.join(', ') || 'None configured'}
            </p>
            <Button
              onClick={handleCopyEnvFiles}
              disabled={!effectiveSource || !selectedTarget || isCopying}
            >
              {isCopying ? (
                <>
                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  Copying...
                </>
              ) : (
                <>
                  <Copy className="mr-2 h-4 w-4" />
                  Copy Now
                </>
              )}
            </Button>
          </div>
        </Card>

        {/* Configuration Card */}
        <Card className="p-4">
          <h3 className="mb-4 flex items-center gap-2 text-lg font-medium">
            <Settings2 className="h-5 w-5" />
            Configuration
          </h3>

          <EnvPatternList
            patterns={envConfig.tracked_patterns}
            onPatternsChange={handlePatternsChange}
          />
        </Card>

        {/* Recent Activity Card */}
        <Card className="p-4">
          <h3 className="mb-4 text-lg font-medium">Recent Activity</h3>
          <EnvCopyHistory lastResult={envConfig.last_copy_result} />
        </Card>
      </div>
    </ScrollArea>
  )
}
