import { useCallback } from 'react'
import { Server, Play, Square, RefreshCw, Trash2, AlertCircle, CheckCircle2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { useMcpState } from '@/hooks/useAppState'
import type { McpLogEntry } from '@/types/state'

/**
 * MCP Inspector Page.
 * Displays MCP server status and traffic log for debugging.
 */
export function McpPage() {
  const { mcp, projectName, dispatch, isLoading } = useMcpState()

  const handleStart = useCallback(async () => {
    await dispatch({ type: 'StartMcpServer' })
  }, [dispatch])

  const handleStop = useCallback(async () => {
    await dispatch({ type: 'StopMcpServer' })
  }, [dispatch])

  const handleClearLogs = useCallback(async () => {
    await dispatch({ type: 'ClearMcpLogs' })
  }, [dispatch])

  // Loading state
  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  // No project open
  if (!mcp) {
    return (
      <div className="flex h-full flex-col items-center justify-center">
        <Server className="h-12 w-12 text-muted-foreground" />
        <h2 className="mt-4 text-xl font-semibold">No Project Open</h2>
        <p className="mt-2 text-muted-foreground">
          Open a project to use the MCP Inspector.
        </p>
      </div>
    )
  }

  const isRunning = mcp.status === 'running'
  const isStarting = mcp.status === 'starting'
  const hasError = mcp.status === 'error'
  const logEntries = mcp.log_entries ?? []

  return (
    <ScrollArea className="h-full">
      <div className="space-y-6 p-4">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold">MCP Inspector</h2>
            <p className="mt-1 text-muted-foreground">
              Monitor MCP server for {projectName}
            </p>
          </div>
        </div>

        {/* Status Card */}
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <Server className="h-5 w-5" />
                <span className="font-medium">MCP Server:</span>
                <StatusBadge status={mcp.status} />
              </div>

              {mcp.port && (
                <span className="text-sm text-muted-foreground">
                  Port: {mcp.port}
                </span>
              )}
            </div>

            <div className="flex gap-2">
              {isRunning ? (
                <Button variant="destructive" size="sm" onClick={handleStop}>
                  <Square className="mr-2 h-4 w-4" />
                  Stop
                </Button>
              ) : (
                <Button
                  size="sm"
                  onClick={handleStart}
                  disabled={isStarting}
                >
                  {isStarting ? (
                    <>
                      <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                      Starting...
                    </>
                  ) : (
                    <>
                      <Play className="mr-2 h-4 w-4" />
                      Start
                    </>
                  )}
                </Button>
              )}
            </div>
          </div>

          {/* Error message */}
          {hasError && mcp.error && (
            <div className="mt-3 flex items-center gap-2 text-sm text-destructive">
              <AlertCircle className="h-4 w-4" />
              {mcp.error}
            </div>
          )}

          {/* Config path hint */}
          {isRunning && mcp.config_path && (
            <p className="mt-3 text-xs text-muted-foreground">
              Claude config: {mcp.config_path}
            </p>
          )}
        </Card>

        {/* Traffic Log Card */}
        <Card className="p-4">
          <div className="flex items-center justify-between mb-4">
            <h3 className="flex items-center gap-2 text-lg font-medium">
              Traffic Log
              {logEntries.length > 0 && (
                <Badge variant="secondary">{logEntries.length}</Badge>
              )}
            </h3>
            <Button
              variant="outline"
              size="sm"
              onClick={handleClearLogs}
              disabled={logEntries.length === 0}
            >
              <Trash2 className="mr-2 h-4 w-4" />
              Clear
            </Button>
          </div>

          {logEntries.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">
              <Server className="mx-auto h-8 w-8 mb-2 opacity-50" />
              <p>No traffic yet</p>
              <p className="text-sm">
                Tool calls from Claude will appear here
              </p>
            </div>
          ) : (
            <div className="space-y-2 font-mono text-sm">
              {logEntries.map((entry, index) => (
                <LogEntryRow key={index} entry={entry} />
              ))}
            </div>
          )}
        </Card>
      </div>
    </ScrollArea>
  )
}

function StatusBadge({ status }: { status: string }) {
  switch (status) {
    case 'running':
      return (
        <Badge className="bg-green-500 hover:bg-green-600">
          <CheckCircle2 className="mr-1 h-3 w-3" />
          Running
        </Badge>
      )
    case 'starting':
      return (
        <Badge className="bg-yellow-500 hover:bg-yellow-600">
          <RefreshCw className="mr-1 h-3 w-3 animate-spin" />
          Starting
        </Badge>
      )
    case 'error':
      return (
        <Badge variant="destructive">
          <AlertCircle className="mr-1 h-3 w-3" />
          Error
        </Badge>
      )
    default:
      return (
        <Badge variant="secondary">
          Stopped
        </Badge>
      )
  }
}

function LogEntryRow({ entry }: { entry: McpLogEntry }) {
  const time = entry.timestamp.split('T')[1]?.slice(0, 8) ?? entry.timestamp
  const isIn = entry.direction === 'in'

  return (
    <div
      className={`flex items-start gap-2 rounded px-2 py-1 ${
        entry.is_error
          ? 'bg-destructive/10 text-destructive'
          : isIn
            ? 'bg-blue-500/10'
            : 'bg-green-500/10'
      }`}
    >
      <span className="text-muted-foreground whitespace-nowrap">[{time}]</span>
      <Badge variant={isIn ? 'default' : 'outline'} className="shrink-0">
        {isIn ? 'IN' : 'OUT'}
      </Badge>
      <span className="font-semibold">{entry.method}</span>
      {entry.tool_name && (
        <span className="text-muted-foreground">"{entry.tool_name}"</span>
      )}
      <span className="truncate text-muted-foreground">{entry.payload}</span>
    </div>
  )
}
