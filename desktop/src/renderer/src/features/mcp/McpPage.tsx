import { useCallback } from 'react'
import {
  Storage as ServerIcon,
  PlayArrow as PlayIcon,
  Stop as StopIcon,
  Refresh as RefreshIcon,
  Delete as DeleteIcon,
  ErrorOutline as AlertIcon,
  CheckCircle as SuccessIcon,
  Terminal as TerminalIcon,
  ContentCopy as CopyIcon
} from '@mui/icons-material'
import {
  Button,
  Card,
  CardContent,
  Box,
  Typography,
  Chip,
  Paper,
  Stack,
  Divider,
  IconButton,
  Tooltip
} from '@mui/material'
import { PageHeader } from '@/components/shared/PageHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { ErrorBanner } from '@/components/shared/ErrorBanner'
import { useMcpState } from '@/hooks/useAppState'
import type { McpLogEntry, McpTool } from '@/types/state'

/**
 * rstn-mcp Integration Page.
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

  // Get tools from state
  const tools = mcp?.available_tools ?? []

  // Loading state
  if (isLoading) {
    return <LoadingState message="Connecting to MCP server..." />
  }

  // No project open
  if (!mcp) {
    return (
      <EmptyState
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
    <Box sx={{ height: '100%', overflow: 'auto', p: 3 }}>
      {/* Header */}
      <PageHeader
        title="rstn-mcp"
        description={`Claude Code integration for ${projectName}`}
        icon={<ServerIcon />}
      />

      <Stack spacing={3}>
        {/* Error message */}
        {hasError && mcp.error && <ErrorBanner error={mcp.error} />}

        {/* Server Status Card */}
        <Card variant="outlined" sx={{ borderRadius: 4 }}>
          <CardContent sx={{ p: 3 }}>
            <Stack direction="row" alignItems="center" spacing={1} sx={{ mb: 2 }}>
              <ServerIcon fontSize="small" color="primary" />
              <Typography variant="h6" fontWeight={600}>Worktree MCP Server</Typography>
            </Stack>

            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 2 }}>
              <Stack direction="row" spacing={3} alignItems="center">
                <Stack direction="row" spacing={1} alignItems="center">
                  <Typography variant="body2" color="text.secondary">Status:</Typography>
                  <StatusBadge status={mcp.status} />
                </Stack>

                {mcp.port && (
                  <Stack direction="row" spacing={1} alignItems="center">
                    <Typography variant="body2" color="text.secondary">Port:</Typography>
                    <Chip label={mcp.port} size="small" variant="outlined" sx={{ borderRadius: 1, fontFamily: 'monospace' }} />
                  </Stack>
                )}
              </Stack>

              <Stack direction="row" spacing={1}>
                {isRunning ? (
                  <Button
                    variant="contained"
                    color="error"
                    size="small"
                    onClick={handleStop}
                    startIcon={<StopIcon />}
                    sx={{ borderRadius: 2 }}
                  >
                    Stop Server
                  </Button>
                ) : (
                  <Button
                    variant="contained"
                    size="small"
                    onClick={handleStart}
                    disabled={isStarting}
                    startIcon={isStarting ? <RefreshIcon sx={{ animation: 'spin 2s linear infinite' }} /> : <PlayIcon />}
                    sx={{ borderRadius: 2 }}
                  >
                    {isStarting ? 'Starting...' : 'Start Server'}
                  </Button>
                )}
              </Stack>
            </Box>

            {/* Server info */}
            {isRunning && (
              <Box sx={{ mt: 3, p: 2, bgcolor: 'surfaceContainerLow.main', borderRadius: 2, border: 1, borderColor: 'outlineVariant' }}>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Typography variant="caption" color="text.secondary">Config File:</Typography>
                  <Typography variant="caption" sx={{ fontFamily: 'monospace', bgcolor: 'action.hover', px: 1, py: 0.5, borderRadius: 0.5 }}>
                    {mcp.config_path}
                  </Typography>
                </Stack>
              </Box>
            )}
          </CardContent>
        </Card>

        {/* Available Tools Card */}
        {isRunning && (
          <Card variant="outlined" sx={{ borderRadius: 4 }}>
            <CardContent sx={{ p: 3 }}>
              <Typography variant="subtitle1" fontWeight={600} sx={{ mb: 2 }}>Available Tools</Typography>
              {tools.length === 0 ? (
                <Typography variant="body2" color="text.secondary">No tools available</Typography>
              ) : (
                <Stack spacing={1.5}>
                  {tools.map((tool) => (
                    <Paper key={tool.name} variant="outlined" sx={{ p: 2, bgcolor: 'surfaceContainerLow.main', borderColor: 'outlineVariant' }}>
                      <Typography variant="subtitle2" sx={{ fontFamily: 'monospace', fontWeight: 700, color: 'primary.main', mb: 0.5 }}>
                        {tool.name}
                      </Typography>
                      <Typography variant="body2" sx={{ mb: 1 }}>{tool.description}</Typography>
                      {tool.input_schema && (
                        <Box component="details" sx={{ cursor: 'pointer' }}>
                          <Box component="summary" sx={{ typography: 'caption', color: 'text.secondary', '&:hover': { color: 'text.primary' } }}>
                            Parameters
                          </Box>
                          <Box
                            component="pre"
                            sx={{
                              mt: 1,
                              p: 1.5,
                              bgcolor: 'background.default',
                              borderRadius: 1,
                              typography: 'caption',
                              fontFamily: 'monospace',
                              overflowX: 'auto',
                              border: 1,
                              borderColor: 'outlineVariant'
                            }}
                          >
                            {JSON.stringify(tool.input_schema, null, 2)}
                          </Box>
                        </Box>
                      )}
                    </Paper>
                  ))}
                </Stack>
              )}
            </CardContent>
          </Card>
        )}

        {/* Claude Code Command Card */}
        {isRunning && mcp.config_path && (
          <Card variant="outlined" sx={{ borderRadius: 4 }}>
            <CardContent sx={{ p: 3 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
                <Stack direction="row" spacing={1} alignItems="center">
                  <TerminalIcon fontSize="small" />
                  <Typography variant="subtitle1" fontWeight={600}>Claude Code Command</Typography>
                </Stack>
                <Button
                  variant="outlined"
                  size="small"
                  onClick={handleCopyCommand}
                  startIcon={<CopyIcon />}
                  sx={{ borderRadius: 2 }}
                >
                  Copy
                </Button>
              </Box>
              <Box sx={{ p: 2, bgcolor: 'background.default', borderRadius: 2, border: 1, borderColor: 'outlineVariant' }}>
                <Typography variant="caption" sx={{ fontFamily: 'monospace', wordBreak: 'break-all', color: 'primary.light' }}>
                  claude -p --verbose --output-format stream-json --mcp-config {mcp.config_path} "your prompt here"
                </Typography>
              </Box>
              <Typography variant="caption" color="text.secondary" sx={{ mt: 1.5, display: 'block' }}>
                This command is automatically used when you chat with Claude Code. The --mcp-config flag enables Claude to access your project files.
              </Typography>
            </CardContent>
          </Card>
        )}

        {/* Server Logs Card */}
        <Card variant="outlined" sx={{ borderRadius: 4 }}>
          <CardContent sx={{ p: 3 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
              <Stack direction="row" spacing={1} alignItems="center">
                <Typography variant="subtitle1" fontWeight={600}>Server Logs</Typography>
                {logEntries.length > 0 && (
                  <Chip label={logEntries.length} size="small" sx={{ height: 20, fontSize: '0.65rem' }} />
                )}
              </Stack>
              <Button
                variant="outlined"
                size="small"
                onClick={handleClearLogs}
                disabled={logEntries.length === 0}
                startIcon={<DeleteIcon />}
                sx={{ borderRadius: 2 }}
              >
                Clear
              </Button>
            </Box>

            {logEntries.length === 0 ? (
              <Box sx={{ py: 6, textAlign: 'center', bgcolor: 'surfaceContainerLow.main', borderRadius: 2, border: '1px dashed', borderColor: 'outlineVariant' }}>
                <TerminalIcon sx={{ fontSize: 40, color: 'text.disabled', mb: 1, opacity: 0.5 }} />
                <Typography variant="body2" color="text.secondary">
                  MCP tool calls from Claude Code will appear here when the server is running.
                </Typography>
              </Box>
            ) : (
              <Stack spacing={0.5} sx={{ p: 1.5, bgcolor: 'background.default', borderRadius: 2, border: 1, borderColor: 'outlineVariant', maxHeight: 400, overflowY: 'auto' }}>
                {logEntries.map((entry, index) => (
                  <LogEntryRow key={index} entry={entry} />
                ))}
              </Stack>
            )}
          </CardContent>
        </Card>
      </Stack>
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </Box>
  )
}

function StatusBadge({ status }: { status: string }) {
  switch (status) {
    case 'running':
      return (
        <Chip
          icon={<SuccessIcon sx={{ fontSize: '1rem !important' }} />}
          label="Running"
          size="small"
          sx={{ bgcolor: 'success.dark', color: 'success.contrastText', fontWeight: 600 }}
        />
      )
    case 'starting':
      return (
        <Chip
          icon={<RefreshIcon sx={{ animation: 'spin 2s linear infinite', fontSize: '1rem !important' }} />}
          label="Starting"
          size="small"
          sx={{ bgcolor: 'warning.dark', color: 'warning.contrastText', fontWeight: 600 }}
        />
      )
    case 'error':
      return (
        <Chip
          icon={<AlertIcon sx={{ fontSize: '1rem !important' }} />}
          label="Error"
          size="small"
          color="error"
          sx={{ fontWeight: 600 }}
        />
      )
    default:
      return (
        <Chip
          label="Stopped"
          size="small"
          variant="outlined"
          sx={{ color: 'text.secondary', fontWeight: 600 }}
        />
      )
  }
}

function LogEntryRow({ entry }: { entry: McpLogEntry }) {
  const time = entry.timestamp.split('T')[1]?.slice(0, 8) ?? entry.timestamp
  const isIn = entry.direction === 'in'

  return (
    <Stack
      direction="row"
      spacing={1.5}
      alignItems="flex-start"
      sx={{
        p: 0.75,
        borderRadius: 1,
        '&:hover': { bgcolor: 'action.hover' },
        ...(entry.is_error && { color: 'error.main' })
      }}
    >
      <Typography variant="caption" sx={{ fontFamily: 'monospace', color: 'text.secondary', flexShrink: 0 }}>
        [{time}]
      </Typography>
      <Chip
        label={isIn ? 'IN' : 'OUT'}
        size="small"
        variant={isIn ? 'filled' : 'outlined'}
        sx={{
          height: 18,
          fontSize: '0.6rem',
          fontWeight: 700,
          bgcolor: isIn ? 'primary.dark' : 'transparent',
          color: isIn ? 'primary.contrastText' : 'text.secondary',
          flexShrink: 0
        }}
      />
      <Typography variant="caption" sx={{ fontFamily: 'monospace', fontWeight: 700, flexShrink: 0 }}>
        {entry.method}
      </Typography>
      {entry.tool_name && (
        <Typography variant="caption" sx={{ fontFamily: 'monospace', color: 'primary.main', flexShrink: 0 }}>
          "{entry.tool_name}"
        </Typography>
      )}
      <Typography
        variant="caption"
        sx={{
          fontFamily: 'monospace',
          color: 'text.secondary',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap'
        }}
      >
        {entry.payload}
      </Typography>
    </Stack>
  )
}
