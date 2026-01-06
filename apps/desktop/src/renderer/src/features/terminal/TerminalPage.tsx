import { useCallback, useEffect, useRef } from 'react'
import { Box, Button, Paper, Stack, Typography } from '@mui/material'
import { Close, OpenInFull, Terminal } from '@mui/icons-material'
import { PageHeader } from '@/components/shared/PageHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { useTerminalState } from '@/hooks/useAppState'

/**
 * Terminal Page - Integrated PTY terminal emulator.
 * Spawns worktree-scoped shell sessions using portable-pty.
 */
export function TerminalPage() {
  const { terminal, worktreePath, projectName, dispatch, isLoading } = useTerminalState()
  const terminalRef = useRef<HTMLDivElement>(null)

  // Default terminal dimensions
  const DEFAULT_COLS = 80
  const DEFAULT_ROWS = 24

  // Spawn terminal session when mounted (if no active session)
  useEffect(() => {
    if (terminal && !terminal.session_id && worktreePath) {
      // Request terminal spawn
      dispatch({
        type: 'SpawnTerminal',
        payload: { cols: DEFAULT_COLS, rows: DEFAULT_ROWS },
      })
    }
  }, [terminal, worktreePath, dispatch])

  const handleSpawn = useCallback(async () => {
    await dispatch({
      type: 'SpawnTerminal',
      payload: { cols: terminal?.cols ?? DEFAULT_COLS, rows: terminal?.rows ?? DEFAULT_ROWS },
    })
  }, [terminal, dispatch])

  const handleKill = useCallback(async () => {
    if (terminal?.session_id) {
      await dispatch({
        type: 'KillTerminal',
        payload: { session_id: terminal.session_id },
      })
    }
  }, [terminal, dispatch])

  const handleResize = useCallback(async () => {
    if (terminal?.session_id) {
      // Expand to full size
      await dispatch({
        type: 'ResizeTerminal',
        payload: {
          session_id: terminal.session_id,
          cols: 120,
          rows: 40,
        },
      })
    }
  }, [terminal, dispatch])

  // Loading state
  if (isLoading) {
    return <LoadingState message="Initializing terminal session..." />
  }

  // No project open
  if (!terminal) {
    return (
      <EmptyState
        icon={Terminal}
        title="No Project Open"
        description="Open a project to access the integrated terminal."
      />
    )
  }

  const hasSession = !!terminal.session_id

  return (
    <Stack sx={{ height: '100%' }}>
      {/* Header */}
      <PageHeader
        title="Terminal"
        description={hasSession ? `Shell session in ${projectName}` : "No active session"}
        icon={<Terminal fontSize="small" />}
      >
        {hasSession ? (
          <>
            <Button
              variant="outline"
              size="sm"
              onClick={handleResize}
            >
              <OpenInFull fontSize="small" sx={{ mr: 1 }} />
              Expand
            </Button>
            <Button
              variant="contained"
              color="error"
              size="small"
              onClick={handleKill}
            >
              <Close fontSize="small" sx={{ mr: 1 }} />
              Kill
            </Button>
          </>
        ) : (
          <Button
            onClick={handleSpawn}
          >
            <Terminal fontSize="small" sx={{ mr: 1 }} />
            Spawn Shell
          </Button>
        )}
      </PageHeader>

      {/* Terminal Area */}
      <Box sx={{ flex: 1, p: 2, pt: 0 }}>
        {hasSession ? (
          <Paper sx={{ height: '100%', bgcolor: '#000', p: 2, fontFamily: 'monospace', fontSize: '0.875rem' }}>
            {/* Terminal Container - xterm.js will attach here */}
            <div
              ref={terminalRef}
              style={{
                height: '100%',
                width: '100%',
                minHeight: `${(terminal.rows ?? DEFAULT_ROWS) * 18}px`,
              }}
            >
              {/* Placeholder until xterm.js is integrated */}
              <Box sx={{ color: '#66bb6a' }}>
                <Typography variant="body2">Session: {terminal.session_id}</Typography>
                <Typography variant="body2">Size: {terminal.cols ?? DEFAULT_COLS}x{terminal.rows ?? DEFAULT_ROWS}</Typography>
                <Typography variant="body2" sx={{ mt: 2, color: 'text.secondary' }}>
                  PTY connected. xterm.js integration pending.
                </Typography>
                <Typography variant="body2" sx={{ mt: 3, color: '#4caf50' }}>
                  $ _
                </Typography>
              </Box>
            </div>
          </Paper>
        ) : (
          <EmptyState
            icon={Terminal}
            title="No Active Session"
            description={`Click "Spawn Shell" to start a new terminal session in ${worktreePath}.`}
            action={{
              label: "Spawn Shell",
              onClick: handleSpawn,
              icon: Terminal
            }}
          />
        )}
      </Box>

      {/* Status Bar */}
      {hasSession && (
        <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ borderTop: 1, borderColor: 'divider', px: 2, py: 1 }}>
          <Typography variant="caption" color="text.secondary">
            Working directory: {worktreePath}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {terminal.cols ?? DEFAULT_COLS} cols x {terminal.rows ?? DEFAULT_ROWS} rows
          </Typography>
        </Stack>
      )}
    </Stack>
  )
}
