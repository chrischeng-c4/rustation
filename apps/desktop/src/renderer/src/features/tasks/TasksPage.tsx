import { useEffect, useCallback } from 'react'
import { Box, Button, Paper, Stack, Typography } from '@mui/material'
import { ListAlt, Refresh } from '@mui/icons-material'
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
        icon={ListAlt}
        title="No Project Selected"
        description="Please select an open project from the tabs above to manage its tasks."
      />
    )
  }

  return (
    <Stack sx={{ height: '100%' }}>
      {/* Header */}
      <PageHeader
        title="Tasks"
        description="Run justfile commands for the current worktree"
      >
        <Button variant="outlined" onClick={handleRefresh} disabled={isRefreshing}>
          <Refresh fontSize="small" sx={{ mr: 1, animation: isRefreshing ? 'spin 1s linear infinite' : undefined }} />
          Refresh
        </Button>
      </PageHeader>

      {/* Error banner */}
      {error && <ErrorBanner error={error} />}

      {/* Two-column layout */}
      <Stack direction="row" spacing={2} sx={{ flex: 1, overflow: 'hidden' }}>
        {/* Column 1: Commands List */}
        <Paper variant="outlined" sx={{ width: '50%', overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          <Box sx={{ px: 2, py: 1, borderBottom: 1, borderColor: 'divider' }}>
            <Typography variant="subtitle2">Commands</Typography>
          </Box>
          <Box sx={{ flex: 1, overflow: 'auto' }}>
            <Stack spacing={2} sx={{ p: 2 }}>
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
                  icon={Refresh}
                  title="No Commands"
                  description="No justfile found in project root."
                  action={{
                    label: "Scan Again",
                    onClick: handleRefresh,
                    icon: Refresh
                  }}
                />
              )}
            </Stack>
          </Box>
        </Paper>

        {/* Column 2: Log Panel */}
        <Box sx={{ width: '50%', overflow: 'hidden' }}>
          <LogPanel
            title={activeCommand ? `just ${activeCommand}` : 'Output'}
            logs={output}
            showCopy={true}
            emptyMessage="Select a command to run"
          />
        </Box>
      </Stack>
    </Stack>
  )
}
