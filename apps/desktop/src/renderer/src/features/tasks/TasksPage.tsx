import { useEffect, useCallback } from 'react'
import { RefreshCw, AlertCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { LogPanel } from '@/components/LogPanel'
import { TaskCard } from './TaskCard'
import { ChatPanel } from './ChatPanel'
import { ConstitutionPanel } from './ConstitutionPanel'
import { useTasksState } from '@/hooks/useAppState'
import type { JustCommandInfo } from '@/types/state'
import { cn } from '@/lib/utils'

// Claude Code as a special "command" in the list
const CLAUDE_CODE_COMMAND: JustCommandInfo = {
  name: 'claude-code',
  description: 'Chat with Claude Code AI assistant',
  recipe: '',
}

// Constitution initialization as a special "command"
const CONSTITUTION_INIT_COMMAND: JustCommandInfo = {
  name: 'constitution-init',
  description: 'Initialize project constitution (CESDD)',
  recipe: '',
}

export function TasksPage() {
  const { tasks, projectPath, dispatch, isLoading: isStateLoading } = useTasksState()

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

  // Handle selecting Claude Code (just sets active command, no "run")
  const handleSelectClaudeCode = useCallback(async () => {
    await dispatch({ type: 'SetActiveCommand', payload: { name: 'claude-code' } })
    await dispatch({ type: 'ClearTaskOutput' })
  }, [dispatch])

  // Handle selecting Constitution Init (sets active command and initializes workflow)
  const handleSelectConstitutionInit = useCallback(async () => {
    await dispatch({ type: 'SetActiveCommand', payload: { name: 'constitution-init' } })
    await dispatch({ type: 'ClearTaskOutput' })
    // Initialize workflow state
    await dispatch({ type: 'StartConstitutionWorkflow' })
  }, [dispatch])

  // Check if Claude Code is active
  const isClaudeCodeActive = activeCommand === 'claude-code'
  // Check if Constitution Init is active
  const isConstitutionInitActive = activeCommand === 'constitution-init'

  // Combined command list: Special commands first, then justfile commands
  const displayCommands = [CLAUDE_CODE_COMMAND, CONSTITUTION_INIT_COMMAND, ...commands]

  // No project path means no active project
  if (!projectPath) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        No project selected
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">Tasks</h2>
          <p className="mt-1 text-muted-foreground">Run justfile commands</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={handleRefresh} disabled={isRefreshing}>
            <RefreshCw className={`mr-2 h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Error banner */}
      {error && (
        <div className="mb-4 flex items-center gap-2 rounded-lg border border-amber-200 bg-amber-50 p-3 text-sm text-amber-700">
          <AlertCircle className="h-4 w-4" />
          {error}
        </div>
      )}

      {/* Two-column layout */}
      <div className="flex flex-1 gap-4 overflow-hidden">
        {/* Column 1: Commands List */}
        <div
          className={cn(
            'overflow-hidden rounded-lg border',
            isClaudeCodeActive ? 'w-1/3' : isConstitutionInitActive ? 'w-1/3' : 'w-1/2'
          )}
        >
          <div className="border-b bg-muted/40 px-4 py-2">
            <span className="text-sm font-medium">Commands</span>
          </div>
          <ScrollArea className="h-[calc(100%-40px)]">
            <div className="space-y-2 p-4">
              {displayCommands.map((cmd) => (
                <TaskCard
                  key={cmd.name}
                  command={cmd}
                  status={taskStatuses[cmd.name] || 'idle'}
                  isActive={activeCommand === cmd.name}
                  onRun={
                    cmd.name === 'claude-code'
                      ? handleSelectClaudeCode
                      : cmd.name === 'constitution-init'
                        ? handleSelectConstitutionInit
                        : handleRun
                  }
                  isClaudeCode={cmd.name === 'claude-code'}
                />
              ))}
              {commands.length === 0 && !isRefreshing && (
                <div className="flex flex-col items-center justify-center py-8 text-center">
                  <p className="text-muted-foreground">No justfile found in project</p>
                  <Button variant="outline" className="mt-4" onClick={handleRefresh}>
                    <RefreshCw className="mr-2 h-4 w-4" />
                    Scan
                  </Button>
                </div>
              )}
            </div>
          </ScrollArea>
        </div>

        {/* Column 2: Chat Panel (only when Claude Code active) */}
        {isClaudeCodeActive && (
          <div className="w-2/3 overflow-hidden">
            <ChatPanel />
          </div>
        )}

        {/* Column 2: Constitution Panel (when Constitution Init active) */}
        {isConstitutionInitActive && (
          <div className="w-2/3 overflow-hidden">
            <ConstitutionPanel />
          </div>
        )}

        {/* Column 2: Log Panel (when justfile command active) */}
        {!isClaudeCodeActive && !isConstitutionInitActive && (
          <div className="w-1/2 overflow-hidden">
            <LogPanel
              title={activeCommand ? `just ${activeCommand}` : 'Output'}
              logs={output}
              showCopy={true}
              emptyMessage="Select a command to run"
            />
          </div>
        )}
      </div>
    </div>
  )
}
