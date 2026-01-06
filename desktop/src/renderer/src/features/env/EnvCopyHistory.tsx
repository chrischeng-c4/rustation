import { useMemo } from 'react'
import {
  CheckCircle as SuccessIcon,
  ErrorOutline as AlertIcon,
  Description as FileTextIcon,
  AccessTime as ClockIcon
} from '@mui/icons-material'
import { Chip, Box, Typography, Stack, Paper, alpha } from '@mui/material'
import type { EnvCopyResult } from '@/types/state'

interface EnvCopyHistoryProps {
  /** The last copy result (if any) */
  lastResult: EnvCopyResult | null
}

/**
 * Displays the result of the most recent env file copy operation.
 */
export function EnvCopyHistory({ lastResult }: EnvCopyHistoryProps) {
  const formattedTime = useMemo(() => {
    if (!lastResult?.timestamp) return null
    try {
      const date = new Date(lastResult.timestamp)
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    } catch {
      return null
    }
  }, [lastResult?.timestamp])

  if (!lastResult) {
    return (
      <Paper variant="outlined" sx={{ py: 6, textAlign: 'center', bgcolor: 'surfaceContainerLow.main', borderStyle: 'dashed' }}>
        <ClockIcon sx={{ fontSize: 40, color: 'text.disabled', opacity: 0.3, mb: 1 }} />
        <Typography variant="body2" color="text.secondary">No recent copy operations</Typography>
      </Paper>
    )
  }

  const { copied_files, failed_files } = lastResult
  const totalCopied = copied_files.length
  const totalFailed = failed_files.length
  const isEmpty = totalCopied === 0 && totalFailed === 0

  if (isEmpty) {
    return (
      <Paper variant="outlined" sx={{ p: 2, bgcolor: 'surfaceContainerLow.main' }}>
        <Stack direction="row" spacing={1.5} alignItems="center">
          <FileTextIcon fontSize="small" sx={{ color: 'text.secondary' }} />
          <Typography variant="body2" color="text.secondary">
            No files to copy (all patterns already exist in target)
          </Typography>
          {formattedTime && <Typography variant="caption" sx={{ ml: 'auto' }}>{formattedTime}</Typography>}
        </Stack>
      </Paper>
    )
  }

  const isSuccess = totalFailed === 0
  const isPartial = totalCopied > 0 && totalFailed > 0

  return (
    <Stack spacing={2.5}>
      {/* Summary */}
      <Stack direction="row" spacing={1.5} alignItems="center">
        {isSuccess ? (
          <SuccessIcon color="success" fontSize="small" />
        ) : (
          <AlertIcon color="warning" fontSize="small" />
        )}
        <Typography variant="subtitle2" fontWeight={700}>
          {isSuccess
            ? `Copied ${totalCopied} file(s)`
            : isPartial
              ? `Copied ${totalCopied}, failed ${totalFailed}`
              : `Failed to copy ${totalFailed} file(s)`}
        </Typography>
        {formattedTime && (
          <Typography variant="caption" color="text.secondary" sx={{ ml: 'auto' }}>{formattedTime}</Typography>
        )}
      </Stack>

      {/* Copied files */}
      {copied_files.length > 0 && (
        <Box>
          <Typography variant="caption" fontWeight={700} color="text.secondary" sx={{ textTransform: 'uppercase', mb: 1, display: 'block' }}>Copied:</Typography>
          <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1 }}>
            {copied_files.map((file) => (
              <Chip 
                key={file} 
                label={file} 
                size="small" 
                variant="outlined"
                sx={{ height: 20, fontSize: '0.65rem', fontFamily: 'monospace', borderRadius: 0.5, bgcolor: alpha('#fff', 0.05) }} 
              />
            ))}
          </Box>
        </Box>
      )}

      {/* Failed files */}
      {failed_files.length > 0 && (
        <Box>
          <Typography variant="caption" fontWeight={700} color="error" sx={{ textTransform: 'uppercase', mb: 1, display: 'block' }}>Failed:</Typography>
          <Stack spacing={1}>
            {failed_files.map(([file, error]) => (
              <Paper 
                key={file} 
                variant="outlined" 
                sx={{ p: 1, px: 1.5, bgcolor: alpha('#ef5350', 0.05), borderColor: alpha('#ef5350', 0.2) }}
              >
                <Stack direction="row" spacing={1.5} alignItems="center">
                  <Typography variant="caption" sx={{ fontFamily: 'monospace', fontWeight: 700, color: 'error.main' }}>{file}</Typography>
                  <Typography variant="caption" color="error">{error}</Typography>
                </Stack>
              </Paper>
            ))}
          </Stack>
        </Box>
      )}
    </Stack>
  )
}
