import { useState, useEffect } from 'react'
import {
  WarningAmber as AlertTriangleIcon,
  Dns as ContainerIcon
} from '@mui/icons-material'
import {
  Button,
  TextField,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  Typography,
  Stack,
  Box,
  Paper,
  alpha
} from '@mui/material'

interface PendingConflict {
  service_id: string
  conflict: {
    requested_port: number
    conflicting_container: {
      id: string
      name: string
      image: string
      is_rstn_managed: boolean
    }
    suggested_port: number
  }
}

interface PortConflictDialogProps {
  pendingConflict: PendingConflict | null
  onResolveWithPort: (serviceId: string, port: number) => void
  onResolveByStoppingContainer: (containerId: string, serviceId: string) => void
  onCancel: () => void
}

export function PortConflictDialog({
  pendingConflict,
  onResolveWithPort,
  onResolveByStoppingContainer,
  onCancel,
}: PortConflictDialogProps) {
  const [resolution, setResolution] = useState<'alt-port' | 'stop-container'>('alt-port')
  const [customPort, setCustomPort] = useState<string>('')
  const [isResolving, setIsResolving] = useState(false)

  // Reset state when dialog opens with new conflict
  useEffect(() => {
    if (pendingConflict) {
      setResolution('alt-port')
      setCustomPort(String(pendingConflict.conflict.suggested_port))
      setIsResolving(false)
    }
  }, [pendingConflict])

  if (!pendingConflict) return null

  const { service_id, conflict } = pendingConflict
  const { requested_port, conflicting_container, suggested_port } = conflict
  const canStopContainer = conflicting_container.is_rstn_managed

  const handleResolve = async () => {
    setIsResolving(true)
    try {
      if (resolution === 'alt-port') {
        const port = parseInt(customPort, 10)
        if (isNaN(port) || port < 1 || port > 65535) {
          return
        }
        onResolveWithPort(service_id, port)
      } else {
        onResolveByStoppingContainer(conflicting_container.id, service_id)
      }
    } finally {
      setIsResolving(false)
    }
  }

  const isValidPort = () => {
    const port = parseInt(customPort, 10)
    return !isNaN(port) && port >= 1 && port <= 65535
  }

  return (
    <Dialog open={!!pendingConflict} onClose={onCancel} maxWidth="sm" fullWidth>
      <DialogTitle>
        <Stack direction="row" spacing={1.5} alignItems="center">
          <AlertTriangleIcon color="warning" />
          <Typography variant="h6">Port Conflict Detected</Typography>
        </Stack>
      </DialogTitle>
      <DialogContent>
        <DialogContentText sx={{ mb: 3 }}>
          Port <strong>{requested_port}</strong> is already in use. Choose how to resolve this conflict.
        </DialogContentText>

        <Stack spacing={3}>
          {/* Conflicting container info */}
          <Paper variant="outlined" sx={{ p: 2, bgcolor: 'surfaceContainerLow.main', borderRadius: 2 }}>
            <Stack direction="row" spacing={1.5} alignItems="center">
              <ContainerIcon fontSize="small" color="primary" />
              <Box>
                <Typography variant="body2" fontWeight={700}>{conflicting_container.name}</Typography>
                <Typography variant="caption" color="text.secondary">Image: {conflicting_container.image}</Typography>
              </Box>
            </Stack>
          </Paper>

          {/* Resolution options */}
          <Stack spacing={2}>
            {/* Option 1: Use alternative port */}
            <Paper
              variant="outlined"
              onClick={() => setResolution('alt-port')}
              sx={{
                p: 2,
                cursor: 'pointer',
                transition: 'all 0.2s',
                borderColor: resolution === 'alt-port' ? 'primary.main' : 'outlineVariant',
                bgcolor: resolution === 'alt-port' ? alpha('#D0BCFF', 0.05) : 'background.paper',
                '&:hover': { bgcolor: 'action.hover' }
              }}
            >
              <Typography variant="subtitle2" fontWeight={700}>Use alternative port</Typography>
              <Stack direction="row" spacing={2} alignItems="center" sx={{ mt: 1.5 }}>
                <TextField
                  size="small"
                  type="number"
                  label="Port"
                  value={customPort}
                  onChange={(e) => setCustomPort(e.target.value)}
                  disabled={resolution !== 'alt-port' || isResolving}
                  onClick={(e) => e.stopPropagation()}
                  sx={{ width: 120 }}
                />
                <Typography variant="caption" color="text.secondary">(suggested: {suggested_port})</Typography>
              </Stack>
            </Paper>

            {/* Option 2: Stop conflicting container */}
            <Paper
              variant="outlined"
              onClick={() => canStopContainer && setResolution('stop-container')}
              sx={{
                p: 2,
                cursor: canStopContainer ? 'pointer' : 'not-allowed',
                opacity: canStopContainer ? 1 : 0.6,
                transition: 'all 0.2s',
                borderColor: resolution === 'stop-container' ? 'primary.main' : 'outlineVariant',
                bgcolor: resolution === 'stop-container' ? alpha('#D0BCFF', 0.05) : 'background.paper',
                '&:hover': { bgcolor: canStopContainer ? 'action.hover' : 'background.paper' }
              }}
            >
              <Typography variant="subtitle2" fontWeight={700}>Stop conflicting container and retry</Typography>
              {!canStopContainer && (
                <Typography variant="caption" color="error" sx={{ display: 'block', mt: 0.5 }}>
                  This container is not managed by rstn and cannot be stopped here.
                </Typography>
              )}
            </Paper>
          </Stack>
        </Stack>
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 3 }}>
        <Button onClick={onCancel} disabled={isResolving}>Cancel</Button>
        <Button
          variant="contained"
          onClick={handleResolve}
          disabled={isResolving || (resolution === 'alt-port' && !isValidPort())}
          sx={{ borderRadius: 2, px: 3 }}
        >
          {isResolving ? 'Resolving...' : 'Continue'}
        </Button>
      </DialogActions>
    </Dialog>
  )
}
