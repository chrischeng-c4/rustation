import * as React from 'react'
import { useState, useCallback } from 'react'
import {
  Add as PlusIcon,
  Close as XIcon,
  Visibility as EyeIcon,
  Code as FileCodeIcon,
  ErrorOutline as AlertCircleIcon
} from '@mui/icons-material'
import {
  Button,
  TextField,
  Chip,
  Box,
  Typography,
  Drawer,
  IconButton,
  Stack,
  Alert,
  Paper,
  Divider,
  CircularProgress
} from '@mui/material'
import { SourceCodeViewer } from '@/components/shared/SourceCodeViewer'
import { useAppState } from '@/hooks/useAppState'

interface ContextFilesInputProps {
  changeId: string
  files: string[]
  projectRoot: string
}

/**
 * Component for managing source file selection for Claude context injection.
 */
export function ContextFilesInput({
  changeId,
  files,
  projectRoot,
}: ContextFilesInputProps): React.ReactElement {
  const { dispatch } = useAppState()
  const [inputValue, setInputValue] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isValidating, setIsValidating] = useState(false)
  const [previewPath, setPreviewPath] = useState<string | null>(null)

  const handleAddFile = useCallback(async () => {
    const path = inputValue.trim()
    if (!path) return

    if (files.includes(path)) {
      setError('File already added')
      return
    }

    setError(null)
    setIsValidating(true)

    try {
      const absPath = path.startsWith('/') ? path : `${projectRoot}/${path}`
      await window.api.file.read(absPath, projectRoot)

      await dispatch({
        type: 'AddContextFile',
        change_id: changeId,
        path: path,
      })

      setInputValue('')
      setError(null)
    } catch (err) {
      const errorStr = err instanceof Error ? err.message : String(err)
      const colonIndex = errorStr.indexOf(':')
      const code = colonIndex > 0 ? errorStr.substring(0, colonIndex).trim() : 'UNKNOWN'

      switch (code) {
        case 'FILE_NOT_FOUND': setError(`File not found: ${path}`); break
        case 'SECURITY_VIOLATION': setError('File is outside project scope'); break
        case 'FILE_TOO_LARGE': setError('File too large (max 10MB)'); break
        case 'NOT_UTF8': setError('File is not text (binary files not supported)'); break
        default: setError(`Cannot read file: ${path}`)
      }
    } finally {
      setIsValidating(false)
    }
  }, [inputValue, files, projectRoot, changeId, dispatch])

  const handleRemoveFile = useCallback(async (path: string) => {
    await dispatch({
      type: 'RemoveContextFile',
      change_id: changeId,
      path: path,
    })
  }, [changeId, dispatch])

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleAddFile()
    }
  }, [handleAddFile])

  return (
    <Stack spacing={2.5}>
      {/* Input area */}
      <Stack direction="row" spacing={1}>
        <TextField
          placeholder="Enter file path (e.g., src/lib/utils.ts)"
          value={inputValue}
          onChange={(e) => {
            setInputValue(e.target.value)
            setError(null)
          }}
          onKeyDown={handleKeyDown}
          fullWidth
          size="small"
          InputProps={{ sx: { fontFamily: 'monospace', fontSize: '0.85rem' } }}
        />
        <Button
          variant="contained"
          size="small"
          onClick={handleAddFile}
          disabled={!inputValue.trim() || isValidating}
          startIcon={isValidating ? <CircularProgress size={16} /> : <PlusIcon />}
          sx={{ borderRadius: 1.5, px: 2, minWidth: 80 }}
        >
          Add
        </Button>
      </Stack>

      {error && (
        <Alert severity="error" sx={{ py: 0, borderRadius: 1 }}>
          {error}
        </Alert>
      )}

      {/* List of added files */}
      {files.length > 0 ? (
        <Stack spacing={1}>
          {files.map((path) => (
            <Paper
              key={path}
              variant="outlined"
              sx={{
                display: 'flex',
                alignItems: 'center',
                gap: 1.5,
                p: 1,
                px: 1.5,
                bgcolor: 'background.paper',
                borderColor: 'outlineVariant'
              }}
            >
              <FileCodeIcon fontSize="small" sx={{ color: 'text.secondary' }} />
              <Typography variant="caption" sx={{ flex: 1, fontFamily: 'monospace', color: 'primary.main', fontWeight: 600 }} noWrap>
                {path}
              </Typography>

              <Stack direction="row" spacing={0.5}>
                <Tooltip title="Preview file">
                  <IconButton size="small" onClick={() => setPreviewPath(path)}>
                    <EyeIcon fontSize="inherit" />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Remove">
                  <IconButton size="small" onClick={() => handleRemoveFile(path)} sx={{ '&:hover': { color: 'error.main' } }}>
                    <XIcon fontSize="inherit" />
                  </IconButton>
                </Tooltip>
              </Stack>
            </Paper>
          ))}
        </Stack>
      ) : (
        <Typography variant="body2" color="text.secondary" sx={{ py: 1, fontStyle: 'italic' }}>
          No files selected. Add source files to include as context for Claude.
        </Typography>
      )}

      {files.length > 0 && (
        <Stack direction="row" spacing={1} alignItems="center">
          <Chip label={`${files.length} files`} size="small" sx={{ height: 20, fontSize: '0.6rem', fontWeight: 700, borderRadius: 0.5 }} />
          <Typography variant="caption" color="text.secondary">will be included as context</Typography>
        </Stack>
      )}

      {/* Preview Drawer */}
      <Drawer
        anchor="right"
        open={!!previewPath}
        onClose={() => setPreviewPath(null)}
        PaperProps={{ sx: { width: 600, bgcolor: 'background.default' } }}
      >
        <Box sx={{ p: 3, height: '100%', display: 'flex', flexDirection: 'column' }}>
          <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
            <Stack direction="row" spacing={1.5} alignItems="center">
              <FileCodeIcon color="primary" />
              <Box>
                <Typography variant="subtitle1" fontWeight={700} sx={{ fontFamily: 'monospace' }}>{previewPath}</Typography>
                <Typography variant="caption" color="text.secondary">Source File Preview</Typography>
              </Box>
            </Stack>
            <IconButton onClick={() => setPreviewPath(null)}><XIcon /></IconButton>
          </Stack>
          <Divider sx={{ mb: 3 }} />
          <Box sx={{ flex: 1, overflow: 'hidden' }}>
            {previewPath && (
              <SourceCodeViewer
                path={previewPath}
                projectRoot={projectRoot}
                maxHeight="100%"
              />
            )}
          </Box>
        </Box>
      </Drawer>
    </Stack>
  )
}

export default ContextFilesInput
