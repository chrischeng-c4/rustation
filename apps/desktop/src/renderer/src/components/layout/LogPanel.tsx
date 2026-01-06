import { useState } from 'react'
import { alpha } from '@mui/material/styles'
import { Box, Chip, IconButton, Paper, Stack, Typography } from '@mui/material'
import {
  ChevronRight,
  ExpandMore,
  Close,
  DeleteOutline,
  Download,
} from '@mui/icons-material'
import { useAppState } from '@/hooks/useAppState'

type LogPanelType = 'actions' | 'errors' | 'info' | 'debug' | 'metrics'

interface DevLog {
  id: string
  timestamp: string
  source: string
  log_type: string
  summary: string
  data: unknown
}

interface LogEntryProps {
  log: DevLog
  isExpanded: boolean
  onToggleExpand: () => void
}

function LogEntry({ log, isExpanded, onToggleExpand }: LogEntryProps) {
  const getLogTypeColor = (type: string) => {
    switch (type) {
      case 'action':
        return '#1e88e5'
      case 'error':
        return '#d32f2f'
      case 'info':
        return '#546e7a'
      case 'state':
        return '#2e7d32'
      case 'claude':
        return '#8e24aa'
      default:
        return '#546e7a'
    }
  }

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    return date.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
  }

  return (
    <Box sx={{ borderBottom: 1, borderColor: 'divider', p: 1, '&:hover': { bgcolor: 'action.hover' } }}>
      <Stack
        direction="row"
        spacing={1}
        alignItems="flex-start"
        onClick={onToggleExpand}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault()
            onToggleExpand()
          }
        }}
        sx={{ cursor: 'pointer' }}
      >
        {isExpanded ? (
          <ExpandMore fontSize="small" sx={{ mt: 0.5, color: 'text.secondary' }} />
        ) : (
          <ChevronRight fontSize="small" sx={{ mt: 0.5, color: 'text.secondary' }} />
        )}
        <Box sx={{ flex: 1 }}>
          <Stack direction="row" spacing={1} alignItems="center">
            <Chip
              label={log.log_type}
              size="small"
              sx={{
                textTransform: 'uppercase',
                fontSize: '0.6rem',
                color: getLogTypeColor(log.log_type),
                bgcolor: alpha(getLogTypeColor(log.log_type), 0.12),
                borderColor: alpha(getLogTypeColor(log.log_type), 0.24),
                borderWidth: 1,
                borderStyle: 'solid',
              }}
            />
            <Chip
              label={log.source}
              size="small"
              sx={{ textTransform: 'uppercase', fontSize: '0.6rem' }}
            />
            <Typography variant="caption" color="text.secondary">
              {formatTimestamp(log.timestamp)}
            </Typography>
          </Stack>
          <Typography variant="body2" fontWeight={500} sx={{ mt: 0.5 }}>
            {log.summary}
          </Typography>
        </Box>
      </Stack>
      {isExpanded && (
        <Box sx={{ ml: 4, mt: 1, borderRadius: 1, bgcolor: 'action.hover', p: 1 }}>
          <Box component="pre" sx={{ m: 0, fontSize: '0.65rem', color: 'text.secondary', overflowX: 'auto' }}>
            {JSON.stringify(log.data, null, 2)}
          </Box>
        </Box>
      )}
    </Box>
  )
}

export function LogPanel() {
  const { state, dispatch } = useAppState()
  const [expandedLogs, setExpandedLogs] = useState<Set<string>>(new Set())

  if (!state) return null

  const { ui_layout, dev_logs } = state
  const activePanel = ui_layout?.active_panel
  const panelExpanded = ui_layout?.panel_expanded ?? false
  const panelWidth = ui_layout?.panel_width ?? 300

  if (!panelExpanded || !activePanel) return null

  // Filter logs based on active panel type
  const filteredLogs = dev_logs?.filter((log: DevLog) => {
    switch (activePanel) {
      case 'actions':
        return log.log_type === 'action'
      case 'errors':
        return log.log_type === 'error'
      case 'info':
        return log.log_type === 'info'
      case 'debug':
        return true // All logs
      case 'metrics':
        return false // Placeholder
      default:
        return false
    }
  }) ?? []

  const handleClose = () => {
    dispatch({ type: 'CloseLogPanel' })
  }

  const handleClearAll = () => {
    dispatch({ type: 'ClearDevLogs' })
    setExpandedLogs(new Set())
  }

  const handleExport = () => {
    const data = JSON.stringify(filteredLogs, null, 2)
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${activePanel}-logs-${Date.now()}.json`
    a.click()
    URL.revokeObjectURL(url)
  }

  const toggleExpand = (logId: string) => {
    setExpandedLogs((prev) => {
      const next = new Set(prev)
      if (next.has(logId)) {
        next.delete(logId)
      } else {
        next.add(logId)
      }
      return next
    })
  }

  const getPanelTitle = () => {
    switch (activePanel) {
      case 'actions':
        return 'Actions Log'
      case 'errors':
        return 'Error Log'
      case 'info':
        return 'Info Log'
      case 'debug':
        return 'Debug Log'
      case 'metrics':
        return 'Performance Metrics'
      default:
        return 'Log'
    }
  }

  return (
    <Paper
      square
      sx={{
        display: 'flex',
        flexDirection: 'column',
        borderLeft: 1,
        borderColor: 'divider',
        width: panelWidth,
      }}
    >
      {/* Header */}
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ borderBottom: 1, borderColor: 'divider', px: 2, py: 1.5 }}>
        <Stack direction="row" alignItems="center" spacing={1}>
          <Typography variant="subtitle2">{getPanelTitle()}</Typography>
          <Chip label={filteredLogs.length} size="small" />
        </Stack>
        <IconButton size="small" onClick={handleClose} title="Close panel">
          <Close fontSize="small" />
        </IconButton>
      </Stack>

      {/* Content */}
      <Box sx={{ flex: 1, overflow: 'auto' }}>
        {filteredLogs.length === 0 ? (
          <Stack alignItems="center" justifyContent="center" sx={{ height: '100%', p: 4, textAlign: 'center' }}>
            <Typography variant="body2" color="text.secondary">
              No logs to display
            </Typography>
          </Stack>
        ) : (
          <Box>
            {filteredLogs.map((log: DevLog) => (
              <LogEntry
                key={log.id}
                log={log}
                isExpanded={expandedLogs.has(log.id)}
                onToggleExpand={() => toggleExpand(log.id)}
              />
            ))}
          </Box>
        )}
      </Box>

      {/* Footer */}
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ borderTop: 1, borderColor: 'divider', px: 2, py: 1 }}>
        <IconButton
          size="small"
          onClick={handleClearAll}
          disabled={filteredLogs.length === 0}
          title="Clear All"
        >
          <DeleteOutline fontSize="small" />
        </IconButton>
        <IconButton
          size="small"
          onClick={handleExport}
          disabled={filteredLogs.length === 0}
          title="Export"
        >
          <Download fontSize="small" />
        </IconButton>
      </Stack>
    </Paper>
  )
}
