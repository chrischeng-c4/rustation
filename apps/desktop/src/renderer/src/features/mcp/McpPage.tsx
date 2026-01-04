import { useCallback } from 'react'
import { Server, Play, Square, RefreshCw, Trash2, AlertCircle, CheckCircle2, Terminal, Copy } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { PageHeader } from '@/components/shared/PageHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { ErrorBanner } from '@/components/shared/ErrorBanner'
import { useMcpState } from '@/hooks/useAppState'
import type { McpLogEntry, McpTool } from '@/types/state'

/**
 * rstn-mcp Integration Page.
 * Shows MCP server status, Claude Code command, and server logs.
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

  const handleCopyCommand = useCallback(() => {
    if (!mcp?.config_path) return
    const command = `claude -p --verbose --output-format stream-json --mcp-config ${mcp.config_path} "your prompt here"`
    navigator.clipboard.writeText(command)
  }, [mcp?.config_path])

  // Get tools from state (populated automatically when server starts)
  const tools = mcp?.available_tools ?? []

  // Loading state
  if (isLoading) {
    return <LoadingState message="Connecting to MCP server..." />
  }

  // No project open
  if (!mcp) {
    return (
      <EmptyState
        icon={Server}
        title="No Project Open"
        description="Open a project to enable MCP integration with Claude Code."
      />
    )
  }

  const isRunning = mcp.status === 'running'
  const isStarting = mcp.status === 'starting'
  const hasError = mcp.status === 'error'
  const logEntries = mcp.log_entries ?? []

  return (
    <ScrollArea className="h-full">
      <div className="p-4">
        {/* Header */}
        <PageHeader
          title="rstn-mcp"
          description={`Claude Code integration for ${projectName}`}
        />

        <div className="space-y-6">
          {/* Error message */}
          {hasError && mcp.error && <ErrorBanner error={mcp.error} />}

          {/* Server Status Card */}
          <Card className="p-4">
            <h3 className="mb-3 flex items-center gap-2 text-lg font-medium">
              <Server className="h-5 w-5" />
              Worktree MCP Server
            </h3>

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">Status:</span>
                  <StatusBadge status={mcp.status} />
                </div>

                {mcp.port && (
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium">Port:</span>
                    <Badge variant="outline">{mcp.port}</Badge>
                  </div>
                )}
              </div>

              <div className="flex gap-2">
                {isRunning ? (
                  <Button variant="destructive" size="sm" onClick={handleStop}>
                    <Square className="mr-2 h-4 w-4" />
                    Stop Server
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
                        Start Server
                      </>
                    )}
                  </Button>
                )}
              </div>
            </div>

            {/* Server info */}
            {isRunning && (
              <div className="mt-4 space-y-2 rounded-md bg-muted/50 p-3 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">Config File:</span>
                  <code className="text-xs">{mcp.config_path}</code>
                </div>
              </div>
            )}
          </Card>

          {/* Available Tools Card */}
          {isRunning && (
            <Card className="p-4">
              <h3 className="mb-4 text-lg font-medium">Available Tools</h3>
              {tools.length === 0 ? (
                <p className="text-sm text-muted-foreground">No tools available</p>
              ) : (
                <div className="space-y-3">
                  {tools.map((tool) => (
                    <Card key={tool.name} className="p-3 bg-muted/30">
                      <div className="space-y-2">
                        <div className="flex items-center gap-2">
                          <code className="text-sm font-mono font-semibold">{tool.name}</code>
                        </div>
                        <p className="text-xs text-muted-foreground">{tool.description}</p>
                        {tool.input_schema && (
                          <details className="text-xs">
                            <summary className="cursor-pointer text-muted-foreground hover:text-foreground">
                              Parameters
                            </summary>
                            <pre className="mt-2 overflow-x-auto rounded bg-muted p-2 font-mono text-xs">
                              {JSON.stringify(tool.input_schema, null, 2)}
                            </pre>
                          </details>
                        )}
                      </div>
                    </Card>
                  ))}
                </div>
              )}
            </Card>
          )}

          {/* Claude Code Command Card */}
          {isRunning && mcp.config_path && (
            <Card className="p-4">
              <div className="mb-3 flex items-center justify-between">
                <h3 className="flex items-center gap-2 text-lg font-medium">
                  <Terminal className="h-5 w-5" />
                  Claude Code Command
                </h3>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleCopyCommand}
                >
                  <Copy className="mr-2 h-4 w-4" />
                  Copy
                </Button>
              </div>
              <div className="rounded-md bg-muted p-3">
                <code className="text-xs break-all">
                  claude -p --verbose --output-format stream-json --mcp-config {mcp.config_path} "your prompt here"
                </code>
              </div>
              <p className="mt-2 text-xs text-muted-foreground">
                This command is automatically used when you chat with Claude Code. The --mcp-config flag enables Claude to access your project files.
              </p>
            </Card>
          )}

          {/* Server Logs Card */}
          <Card className="p-4">
            <div className="flex items-center justify-between mb-4">
              <h3 className="flex items-center gap-2 text-lg font-medium">
                Server Logs
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
                Clear Logs
              </Button>
            </div>

            {logEntries.length === 0 ? (
              <EmptyState
                icon={Terminal}
                title="No Activity Yet"
                description="MCP tool calls from Claude Code will appear here when the server is running."
                className="py-12"
              />
            ) : (
              <div className="space-y-2 font-mono text-sm">
                {logEntries.map((entry, index) => (
                  <LogEntryRow key={index} entry={entry} />
                ))}
              </div>
            )}
          </Card>
        </div>
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
