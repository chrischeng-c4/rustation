import { useEffect, useCallback } from 'react'
import { RefreshCw, ListTodo } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { LogPanel } from '@/components/shared/LogPanel'
import { PageHeader } from '@/components/shared/PageHeader'
import { EmptyState } from '@/components/shared/EmptyState'
import { ErrorBanner } from '@/components/shared/ErrorBanner'
import { TaskCard } from './TaskCard'
import { useTasksState } from '@/hooks/useAppState'

/**
 * TasksPage - Simple justfile command runner.
 *
 * This page is for fire-and-forget task execution.
 * For guided, stateful workflows, use the Workflows tab.
 */
export function TasksPage() {
  const { tasks, projectPath, dispatch } = useTasksState()

  // Derive values from state
  const commands = tasks?.commands ?? []
  const taskStatuses = tasks?.task_statuses ?? {}
  const output = tasks?.output ?? []
  const activeCommand = tasks?.active_command ?? null
  const isRefreshing = tasks?.is_loading ?? false
  const error = tasks?.error ?? null

  // Build justfile path from project path
  const justfilePath = projectPath ? `${projectPath}/justfile` : null

  // Load commands when project changes
  useEffect(() => {
    if (justfilePath) {
      dispatch({ type: 'LoadJustfileCommands', payload: { path: justfilePath } })
    }
  }, [dispatch, justfilePath])

  const handleRun = useCallback(
    async (name: string) => {
      if (projectPath) {
        await dispatch({ type: 'RunJustCommand', payload: { name, cwd: projectPath } })
      }
    },
    [dispatch, projectPath]
  )

  const handleRefresh = useCallback(async () => {
    if (justfilePath) {
      await dispatch({ type: 'ClearTaskOutput' })
      await dispatch({ type: 'SetActiveCommand', payload: { name: null } })
      await dispatch({ type: 'LoadJustfileCommands', payload: { path: justfilePath } })
    }
  }, [dispatch, justfilePath])

  // No project path means no active project
  if (!projectPath) {
    return (
      <EmptyState
        icon={ListTodo}
        title="No Project Selected"
        description="Please select an open project from the tabs above to manage its tasks."
      />
    )
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <PageHeader
        title="Tasks"
        description="Run justfile commands for the current worktree"
      >
        <Button variant="outline" onClick={handleRefresh} disabled={isRefreshing}>
          <RefreshCw className={`mr-2 h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </PageHeader>

      {/* Error banner */}
      {error && <ErrorBanner error={error} />}

      {/* Two-column layout */}
      <div className="flex flex-1 gap-4 overflow-hidden">
        {/* Column 1: Commands List */}
        <div className="w-1/2 overflow-hidden rounded-lg border">
          <div className="border-b bg-muted/40 px-4 py-2">
            <span className="text-sm font-medium">Commands</span>
          </div>
          <ScrollArea className="h-[calc(100%-40px)]">
            <div className="space-y-2 p-4">
              {commands.map((cmd) => (
                <TaskCard
                  key={cmd.name}
                  command={cmd}
                  status={taskStatuses[cmd.name] || 'idle'}
                  isActive={activeCommand === cmd.name}
                  onRun={handleRun}
                />
              ))}
              {commands.length === 0 && !isRefreshing && (
                <EmptyState
                  icon={RefreshCw}
                  title="No Commands"
                  description="No justfile found in project root."
                  action={{
                    label: "Scan Again",
                    onClick: handleRefresh,
                    icon: RefreshCw
                  }}
                />
              )}
            </div>
          </ScrollArea>
        </div>

        {/* Column 2: Log Panel */}
        <div className="w-1/2 overflow-hidden">
          <LogPanel
            title={activeCommand ? `just ${activeCommand}` : 'Output'}
            logs={output}
            showCopy={true}
            emptyMessage="Select a command to run"
          />
        </div>
      </div>
    </div>
  )
}
