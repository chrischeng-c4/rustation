import { useState, useCallback } from 'react'
import {
  Add as PlusIcon,
  Close as XIcon,
  Description as FileTextIcon,
  Folder as FolderIcon
} from '@mui/icons-material'
import {
  Button,
  TextField,
  Box,
  Typography,
  Stack,
  IconButton,
  Paper,
  Divider,
  alpha
} from '@mui/material'

interface EnvPatternListProps {
  /** Current list of tracked patterns */
  patterns: string[]
  /** Callback when patterns change */
  onPatternsChange: (patterns: string[]) => void
  /** Whether editing is disabled */
  disabled?: boolean
}

/**
 * Editable list of env file patterns.
 */
export function EnvPatternList({
  patterns,
  onPatternsChange,
  disabled = false,
}: EnvPatternListProps) {
  const [newPattern, setNewPattern] = useState('')

  const handleAddPattern = useCallback(() => {
    const trimmed = newPattern.trim()
    if (!trimmed) return
    if (patterns.includes(trimmed)) {
      setNewPattern('')
      return
    }
    onPatternsChange([...patterns, trimmed])
    setNewPattern('')
  }, [newPattern, patterns, onPatternsChange])

  const handleRemovePattern = useCallback(
    (pattern: string) => {
      onPatternsChange(patterns.filter((p) => p !== pattern))
    },
    [patterns, onPatternsChange]
  )

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter') {
        e.preventDefault()
        handleAddPattern()
      }
    },
    [handleAddPattern]
  )

  const isDirectory = (pattern: string) => pattern.endsWith('/')

  return (
    <Stack spacing={2}>
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <Typography variant="subtitle2" fontWeight={600}>Tracked Patterns</Typography>
        <Typography variant="caption" color="text.secondary">{patterns.length} pattern(s)</Typography>
      </Box>

      {/* Pattern list */}
      <Stack spacing={1}>
        {patterns.map((pattern) => (
          <Paper
            key={pattern}
            variant="outlined"
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              px: 1.5,
              py: 0.75,
              bgcolor: 'background.default',
              borderColor: 'outlineVariant'
            }}
          >
            <Stack direction="row" spacing={1.5} alignItems="center">
              {isDirectory(pattern) ? (
                <FolderIcon fontSize="small" sx={{ color: 'text.secondary' }} />
              ) : (
                <FileTextIcon fontSize="small" sx={{ color: 'text.secondary' }} />
              )}
              <Typography variant="body2" sx={{ fontFamily: 'monospace', fontWeight: 600 }}>{pattern}</Typography>
            </Stack>
            <IconButton
              size="small"
              onClick={() => handleRemovePattern(pattern)}
              disabled={disabled}
              sx={{ '&:hover': { color: 'error.main' } }}
            >
              <XIcon fontSize="inherit" />
            </IconButton>
          </Paper>
        ))}

        {patterns.length === 0 && (
          <Paper
            variant="outlined"
            sx={{
              py: 4,
              textAlign: 'center',
              bgcolor: 'surfaceContainerLow.main',
              borderStyle: 'dashed',
              borderColor: 'outlineVariant'
            }}
          >
            <Typography variant="body2" color="text.secondary">
              No patterns configured. Add patterns like .env or .claude/
            </Typography>
          </Paper>
        )}
      </Stack>

      <Divider sx={{ my: 1 }} />

      {/* Add new pattern */}
      <Stack direction="row" spacing={1}>
        <TextField
          placeholder="Add pattern (e.g., .env.local)"
          value={newPattern}
          onChange={(e) => setNewPattern(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          size="small"
          fullWidth
          InputProps={{ sx: { fontFamily: 'monospace', fontSize: '0.85rem' } }}
        />
        <Button
          variant="contained"
          onClick={handleAddPattern}
          disabled={disabled || !newPattern.trim()}
          startIcon={<PlusIcon />}
          sx={{ borderRadius: 1.5, px: 2, minWidth: 80 }}
        >
          Add
        </Button>
      </Stack>
    </Stack>
  )
}
