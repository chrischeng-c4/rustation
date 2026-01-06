import { useCallback, useEffect, useRef, useState } from 'react'
import { Box, IconButton, Paper, Stack, Typography } from '@mui/material'
import { CheckCircle, Code, ContentCopy, Refresh } from '@mui/icons-material'

interface LogPanelProps {
  title?: string
  logs: string[]
  onRefresh?: () => void
  isRefreshing?: boolean
  showCopy?: boolean
  emptyMessage?: string
}

export function LogPanel({
  title = 'Output',
  logs,
  onRefresh,
  isRefreshing = false,
  showCopy = true,
  emptyMessage = 'No output',
}: LogPanelProps) {
  const [copied, setCopied] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)

  // Auto-scroll to bottom when logs change
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [logs])

  const handleCopy = useCallback(async () => {
    const text = logs.join('\n')
    await navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }, [logs])

  return (
    <Paper variant="outlined" sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Header */}
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ borderBottom: 1, borderColor: 'divider', px: 2, py: 1 }}>
        <Stack direction="row" alignItems="center" spacing={1}>
          <Code fontSize="small" />
          <Typography variant="subtitle2">{title}</Typography>
        </Stack>
        <Stack direction="row" spacing={0.5}>
          {showCopy && logs.length > 0 && (
            <IconButton size="small" onClick={handleCopy}>
              {copied ? (
                <CheckCircle fontSize="small" sx={{ color: 'success.main' }} />
              ) : (
                <ContentCopy fontSize="small" />
              )}
            </IconButton>
          )}
          {onRefresh && (
            <IconButton size="small" onClick={onRefresh} disabled={isRefreshing}>
              <Refresh fontSize="small" sx={{ animation: isRefreshing ? 'spin 1s linear infinite' : undefined }} />
            </IconButton>
          )}
        </Stack>
      </Stack>

      {/* Content */}
      <Box ref={scrollRef} sx={{ flex: 1, overflow: 'auto', p: 2 }}>
        {logs.length > 0 ? (
          <Box component="pre" sx={{ m: 0, whiteSpace: 'pre-wrap', fontFamily: 'monospace', fontSize: '0.75rem' }}>
            {logs.join('\n')}
          </Box>
        ) : (
          <Typography variant="body2" color="text.secondary">
            {emptyMessage}
          </Typography>
        )}
      </Box>
    </Paper>
  )
}
