import { useCallback, useEffect, useRef } from 'react'
import {
  TerminalSquare,
  RefreshCw,
  X,
  Maximize2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
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
        icon={TerminalSquare}
        title="No Project Open"
        description="Open a project to access the integrated terminal."
      />
    )
  }

  const hasSession = !!terminal.session_id

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <PageHeader
        title="Terminal"
        description={hasSession ? `Shell session in ${projectName}` : "No active session"}
        icon={<TerminalSquare className="h-5 w-5" />}
      >
        {hasSession ? (
          <>
            <Button
              variant="outline"
              size="sm"
              onClick={handleResize}
            >
              <Maximize2 className="mr-2 h-4 w-4" />
              Expand
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={handleKill}
            >
              <X className="mr-2 h-4 w-4" />
              Kill
            </Button>
          </>
        ) : (
          <Button
            onClick={handleSpawn}
          >
            <TerminalSquare className="mr-2 h-4 w-4" />
            Spawn Shell
          </Button>
        )}
      </PageHeader>

      {/* Terminal Area */}
      <div className="flex-1 p-4 pt-0">
        {hasSession ? (
          <Card className="h-full bg-black p-2 font-mono text-sm">
            {/* Terminal Container - xterm.js will attach here */}
            <div
              ref={terminalRef}
              className="h-full w-full"
              style={{
                minHeight: `${(terminal.rows ?? DEFAULT_ROWS) * 18}px`,
              }}
            >
              {/* Placeholder until xterm.js is integrated */}
              <div className="text-green-400">
                <p>Session: {terminal.session_id}</p>
                <p>Size: {terminal.cols ?? DEFAULT_COLS}x{terminal.rows ?? DEFAULT_ROWS}</p>
                <p className="mt-2 text-muted-foreground">
                  PTY connected. xterm.js integration pending.
                </p>
                <p className="mt-4 text-green-500">$ _</p>
              </div>
            </div>
          </Card>
        ) : (
          <EmptyState
            icon={TerminalSquare}
            title="No Active Session"
            description={`Click "Spawn Shell" to start a new terminal session in ${worktreePath}.`}
            action={{
              label: "Spawn Shell",
              onClick: handleSpawn,
              icon: TerminalSquare
            }}
          />
        )}
      </div>

      {/* Status Bar */}
      {hasSession && (
        <div className="border-t px-4 py-2 text-xs text-muted-foreground flex items-center justify-between">
          <span>
            Working directory: {worktreePath}
          </span>
          <span>
            {terminal.cols ?? DEFAULT_COLS} cols x {terminal.rows ?? DEFAULT_ROWS} rows
          </span>
        </div>
      )}
    </div>
  )
}
