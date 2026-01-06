import * as React from 'react'
import { useEffect, useState } from 'react'
import { Alert, Box, CircularProgress, Paper, Stack, Typography } from '@mui/material'
import { Code, ErrorOutline } from '@mui/icons-material'

interface SourceCodeViewerProps {
  /** Absolute or relative path to the file */
  path: string
  /** Project root for security validation */
  projectRoot: string
  /** Optional: Show line numbers (default: true) */
  showLineNumbers?: boolean
  /** Optional: Maximum height with scroll */
  maxHeight?: string
  /** Optional: Callback when file cannot be read */
  onError?: (error: string) => void
}

type ViewerState =
  | { status: 'loading' }
  | { status: 'success'; content: string }
  | { status: 'error'; message: string }

/**
 * Parse error code from Rust error message format "CODE: message"
 */
function parseErrorMessage(error: string): { code: string; message: string } {
  const colonIndex = error.indexOf(':')
  if (colonIndex > 0) {
    const code = error.substring(0, colonIndex).trim()
    const message = error.substring(colonIndex + 1).trim()
    return { code, message }
  }
  return { code: 'UNKNOWN', message: error }
}

/**
 * Get user-friendly error message based on error code
 */
function getFriendlyErrorMessage(code: string, path: string): string {
  switch (code) {
    case 'FILE_NOT_FOUND':
      return `File not found: ${path}`
    case 'SECURITY_VIOLATION':
      return 'Access denied: File is outside project scope'
    case 'FILE_TOO_LARGE':
      return 'File too large to display (max 10MB)'
    case 'NOT_UTF8':
      return 'Cannot display: File is not UTF-8 text'
    case 'PERMISSION_DENIED':
      return 'Permission denied: Cannot read file'
    default:
      return `Error reading file: ${path}`
  }
}

/**
 * Component for viewing source code files with basic styling.
 * Uses the secure file reader API (window.api.file.read).
 */
export function SourceCodeViewer({
  path,
  projectRoot,
  showLineNumbers = true,
  maxHeight = '400px',
  onError,
}: SourceCodeViewerProps): React.ReactElement {
  const [state, setState] = useState<ViewerState>({ status: 'loading' })

  useEffect(() => {
    let cancelled = false

    async function loadFile(): Promise<void> {
      setState({ status: 'loading' })

      try {
        // Build absolute path if relative
        const absPath = path.startsWith('/')
          ? path
          : `${projectRoot}/${path}`

        const content = await window.api.file.read(absPath, projectRoot)

        if (!cancelled) {
          setState({ status: 'success', content })
        }
      } catch (error) {
        if (!cancelled) {
          const errorStr = error instanceof Error ? error.message : String(error)
          const { code } = parseErrorMessage(errorStr)
          const friendlyMessage = getFriendlyErrorMessage(code, path)

          setState({ status: 'error', message: friendlyMessage })
          onError?.(friendlyMessage)
        }
      }
    }

    loadFile()

    return () => {
      cancelled = true
    }
  }, [path, projectRoot, onError])

  if (state.status === 'loading') {
    return (
      <Stack direction="row" alignItems="center" justifyContent="center" spacing={1} sx={{ py: 4 }}>
        <CircularProgress size={20} />
        <Typography variant="body2" color="text.secondary">
          Loading file...
        </Typography>
      </Stack>
    )
  }

  if (state.status === 'error') {
    return (
      <Alert severity="error" icon={<ErrorOutline fontSize="small" />}>
        <Typography variant="body2">{state.message}</Typography>
      </Alert>
    )
  }

  const lines = state.content.split('\n')

  return (
    <Paper variant="outlined" sx={{ bgcolor: 'action.hover' }}>
      <Stack direction="row" alignItems="center" spacing={1} sx={{ px: 2, py: 1, borderBottom: 1, borderColor: 'divider' }}>
        <Code fontSize="small" />
        <Typography variant="caption" sx={{ fontFamily: 'monospace' }}>
          {path}
        </Typography>
      </Stack>
      <Box sx={{ maxHeight, overflow: 'auto' }}>
        <Box component="pre" sx={{ m: 0, p: 2, fontSize: '0.875rem', fontFamily: 'monospace', overflowX: 'auto' }}>
          <Box component="code">
            {showLineNumbers ? (
              <Box component="table" sx={{ width: '100%', borderCollapse: 'collapse' }}>
                <Box component="tbody">
                  {lines.map((line, index) => (
                    <Box
                      component="tr"
                      key={index}
                      sx={{ '&:hover': { bgcolor: 'action.selected' } }}
                    >
                      <Box
                        component="td"
                        sx={{
                          pr: 2,
                          textAlign: 'right',
                          color: 'text.secondary',
                          userSelect: 'none',
                          whiteSpace: 'nowrap',
                          width: '1%',
                        }}
                      >
                        {index + 1}
                      </Box>
                      <Box component="td" sx={{ whiteSpace: 'pre' }}>
                        {line}
                      </Box>
                    </Box>
                  ))}
                </Box>
              </Box>
            ) : (
              state.content
            )}
          </Box>
        </Box>
      </Box>
    </Paper>
  )
}

export default SourceCodeViewer
