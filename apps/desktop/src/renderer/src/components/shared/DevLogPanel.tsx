import { useState, useCallback, useMemo } from 'react'
import { Box, IconButton, Paper, Stack, Typography } from '@mui/material'
import { alpha } from '@mui/material/styles'
import {
  BugReport,
  CheckCircle,
  ChevronRight,
  Close,
  ContentCopy,
  DeleteOutline,
  ExpandMore,
} from '@mui/icons-material'
import { useAppState } from '@/hooks/useAppState'
import type { DevLog, DevLogSource, DevLogType } from '@/types/state'

/**
 * DevLogPanel - Right-side panel for displaying development logs
 *
 * Features:
 * - Collapsible entries (collapsed = summary, expanded = beautiful JSON)
 * - Source and type badges with colors
 * - Clear all logs button
 * - Dev mode only
 */
export function DevLogPanel() {
  const { state, dispatch } = useAppState()
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set())
  const [isOpen, setIsOpen] = useState(true)

  const devLogs = state?.dev_logs ?? []

  const toggleExpand = useCallback((id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }, [])

  const handleClear = useCallback(async () => {
    await dispatch({ type: 'ClearDevLogs' })
  }, [dispatch])

  const handleClose = useCallback(() => {
    setIsOpen(false)
  }, [])

  if (!isOpen) {
    return (
      <IconButton
        size="small"
        onClick={() => setIsOpen(true)}
        sx={{ position: 'fixed', right: 8, top: 8, zIndex: 50 }}
        title="Open Dev Logs"
      >
        <BugReport fontSize="small" />
      </IconButton>
    )
  }

  return (
    <Box sx={{ display: 'flex', height: '100%', width: 288, flexDirection: 'column', borderLeft: 1, borderColor: 'divider' }}>
      {/* Header */}
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ borderBottom: 1, borderColor: 'divider', px: 2, py: 1 }}>
        <Stack direction="row" alignItems="center" spacing={1}>
          <BugReport fontSize="small" sx={{ color: 'warning.main' }} />
          <Typography variant="subtitle2">Dev Logs</Typography>
          <Typography variant="caption" color="text.secondary">
            ({devLogs.length})
          </Typography>
        </Stack>
        <Stack direction="row" spacing={0.5}>
          <IconButton
            size="small"
            onClick={handleClear}
            disabled={devLogs.length === 0}
            title="Clear all logs"
          >
            <DeleteOutline fontSize="small" />
          </IconButton>
          <IconButton size="small" onClick={handleClose} title="Close panel">
            <Close fontSize="small" />
          </IconButton>
        </Stack>
      </Stack>

      {/* Log Entries */}
      <Box sx={{ flex: 1, overflow: 'auto' }}>
        {devLogs.length === 0 ? (
          <Stack alignItems="center" justifyContent="center" sx={{ height: 128 }}>
            <Typography variant="body2" color="text.secondary">
              No dev logs yet
            </Typography>
          </Stack>
        ) : (
          <Stack spacing={1} sx={{ p: 2 }}>
            {devLogs.map((log) => (
              <DevLogEntry
                key={log.id}
                log={log}
                isExpanded={expandedIds.has(log.id)}
                onToggle={() => toggleExpand(log.id)}
              />
            ))}
          </Stack>
        )}
      </Box>
    </Box>
  )
}

interface DevLogEntryProps {
  log: DevLog
  isExpanded: boolean
  onToggle: () => void
}

function DevLogEntry({ log, isExpanded, onToggle }: DevLogEntryProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = useCallback(
    async (e: React.MouseEvent) => {
      e.stopPropagation()
      await navigator.clipboard.writeText(JSON.stringify(log.data, null, 2))
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    },
    [log.data]
  )

  const timestamp = useMemo(() => {
    return new Date(log.timestamp).toLocaleTimeString()
  }, [log.timestamp])

  return (
    <Paper
      variant="outlined"
      sx={{
        borderColor: isExpanded ? 'primary.main' : 'divider',
        boxShadow: isExpanded ? 2 : 0,
      }}
    >
      {/* Collapsed Header (always visible) */}
      <Stack
        direction="row"
        alignItems="center"
        spacing={1}
        onClick={onToggle}
        sx={{ cursor: 'pointer', px: 1.5, py: 1, '&:hover': { bgcolor: 'action.hover' } }}
      >
        {isExpanded ? (
          <ExpandMore fontSize="small" sx={{ color: 'text.secondary' }} />
        ) : (
          <ChevronRight fontSize="small" sx={{ color: 'text.secondary' }} />
        )}

        <SourceBadge source={log.source} />
        <TypeBadge logType={log.log_type} />

        <Typography variant="caption" sx={{ flex: 1 }} noWrap>
          {log.summary}
        </Typography>

        <Typography variant="caption" color="text.secondary">
          {timestamp}
        </Typography>
      </Stack>

      {/* Expanded Content */}
      {isExpanded && (
        <Box sx={{ borderTop: 1, borderColor: 'divider', bgcolor: 'action.hover', p: 1.5 }}>
          <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ pb: 1 }}>
            <Typography variant="overline" color="text.secondary">
              Data
            </Typography>
            <IconButton size="small" onClick={handleCopy} title="Copy JSON">
              {copied ? (
                <CheckCircle fontSize="small" sx={{ color: 'success.main' }} />
              ) : (
                <ContentCopy fontSize="small" />
              )}
            </IconButton>
          </Stack>
          <Box
            component="pre"
            sx={{
              m: 0,
              maxHeight: 240,
              overflow: 'auto',
              borderRadius: 1,
              bgcolor: 'background.default',
              p: 1,
              fontFamily: 'monospace',
              fontSize: '0.625rem',
            }}
          >
            {JSON.stringify(log.data, null, 2)}
          </Box>
        </Box>
      )}
    </Paper>
  )
}

function SourceBadge({ source }: { source: DevLogSource }) {
  const config = {
    rust: { label: 'RS', color: '#ef6c00' },
    frontend: { label: 'FE', color: '#1e88e5' },
    claude: { label: 'CL', color: '#8e24aa' },
    ipc: { label: 'IPC', color: '#546e7a' },
  }[source]

  return (
    <Box
      component="span"
      sx={{
        flexShrink: 0,
        borderRadius: 0.5,
        px: 0.75,
        py: 0.25,
        fontSize: '0.55rem',
        fontWeight: 700,
        textTransform: 'uppercase',
        color: config.color,
        bgcolor: alpha(config.color, 0.16),
      }}
    >
      {config.label}
    </Box>
  )
}

function TypeBadge({ logType }: { logType: DevLogType }) {
  const config = {
    action: { label: 'ACT', color: '#2e7d32' },
    state: { label: 'STA', color: '#00838f' },
    claude: { label: 'CLU', color: '#8e24aa' },
    error: { label: 'ERR', color: '#d32f2f' },
    info: { label: 'INF', color: '#546e7a' },
  }[logType]

  return (
    <Box
      component="span"
      sx={{
        flexShrink: 0,
        borderRadius: 0.5,
        px: 0.75,
        py: 0.25,
        fontSize: '0.55rem',
        fontWeight: 700,
        textTransform: 'uppercase',
        color: config.color,
        bgcolor: alpha(config.color, 0.16),
      }}
    >
      {config.label}
    </Box>
  )
}
