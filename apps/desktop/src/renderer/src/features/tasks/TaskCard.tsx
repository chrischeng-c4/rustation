import { Box, Button, Paper, Stack, Typography } from '@mui/material'
import { CheckCircle, Cancel, PlayArrow } from '@mui/icons-material'
import type { JustCommand, TaskStatus } from '@/types/task'

interface TaskCardProps {
  command: JustCommand
  status: TaskStatus
  isActive?: boolean
  onRun?: (name: string) => void
}

export function TaskCard({ command, status, isActive = false, onRun }: TaskCardProps) {
  const isRunning = status === 'running'

  return (
    <Paper
      variant="outlined"
      data-testid={`task-card-${command.name}`}
      sx={{
        p: 1.5,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        borderColor: isActive ? 'primary.main' : 'divider',
        bgcolor: isActive ? 'action.selected' : 'transparent',
        transition: 'background-color 120ms ease',
        '&:hover': {
          bgcolor: isActive ? 'action.selected' : 'action.hover',
        },
      }}
    >
      <Box sx={{ flex: 1, minWidth: 0 }}>
        <Stack direction="row" spacing={1} alignItems="center">
          <Typography variant="body2" sx={{ fontFamily: 'monospace', fontWeight: 600 }} noWrap>
            {command.name}
          </Typography>
          {status === 'success' && <CheckCircle fontSize="small" color="success" />}
          {status === 'error' && <Cancel fontSize="small" color="error" />}
        </Stack>
        {command.description && (
          <Typography variant="caption" color="text.secondary" noWrap sx={{ mt: 0.5, display: 'block' }}>
            {command.description}
          </Typography>
        )}
      </Box>

      <Button
        variant={isRunning ? 'contained' : 'outlined'}
        size="small"
        disabled={isRunning}
        onClick={() => onRun?.(command.name)}
        sx={{ ml: 2, flexShrink: 0, minWidth: 40 }}
      >
        {isRunning ? '...' : <PlayArrow fontSize="small" />}
      </Button>
    </Paper>
  )
}
