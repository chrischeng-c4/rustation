import { useState } from 'react'
import {
  Dns as ServerIcon,
  ContentCopy as CopyIcon,
  Check as CheckIcon
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
  IconButton,
  Stack,
  InputAdornment
} from '@mui/material'

interface AddVhostDialogProps {
  serviceId: string
  disabled?: boolean
  onCreateVhost?: (serviceId: string, vhostName: string) => Promise<string>
}

export function AddVhostDialog({
  serviceId,
  disabled,
  onCreateVhost,
}: AddVhostDialogProps) {
  const [open, setOpen] = useState(false)
  const [vhostName, setVhostName] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const [connectionString, setConnectionString] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const handleCreate = async () => {
    if (!vhostName.trim()) {
      setError('Vhost name is required')
      return
    }

    if (!/^[a-zA-Z_][a-zA-Z0-9_-]*$/.test(vhostName)) {
      setError('Vhost name must start with a letter or underscore and contain only alphanumeric characters, underscores, and hyphens')
      return
    }

    setError(null)
    setIsCreating(true)

    try {
      if (onCreateVhost) {
        const connStr = await onCreateVhost(serviceId, vhostName)
        setConnectionString(connStr)
      } else {
        setConnectionString(`amqp://guest:guest@localhost:5672/${vhostName}`)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create vhost')
    } finally {
      setIsCreating(false)
    }
  }

  const handleCopy = async () => {
    if (connectionString) {
      await navigator.clipboard.writeText(connectionString)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }

  const handleOpenChange = (isOpen: boolean) => {
    setOpen(isOpen)
    if (!isOpen) {
      setVhostName('')
      setConnectionString(null)
      setError(null)
      setCopied(false)
    }
  }

  return (
    <>
      <Button
        variant="text"
        size="small"
        disabled={disabled}
        onClick={(e) => {
          e.stopPropagation()
          handleOpenChange(true)
        }}
        startIcon={<ServerIcon />}
      >
        Add vhost
      </Button>

      <Dialog open={open} onClose={() => handleOpenChange(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Virtual Host</DialogTitle>
        <DialogContent>
          <DialogContentText sx={{ mb: 3 }}>
            Create a new virtual host in RabbitMQ
          </DialogContentText>

          {!connectionString ? (
            <Stack spacing={2} sx={{ mt: 1 }}>
              <TextField
                autoFocus
                label="Vhost Name"
                placeholder="my_vhost"
                fullWidth
                value={vhostName}
                onChange={(e) => setVhostName(e.target.value)}
                disabled={isCreating}
                error={!!error}
                helperText={error}
              />
            </Stack>
          ) : (
            <Stack spacing={2} sx={{ mt: 1 }}>
              <TextField
                label="Connection String"
                value={connectionString}
                fullWidth
                readOnly
                InputProps={{
                  readOnly: true,
                  endAdornment: (
                    <InputAdornment position="end">
                      <IconButton onClick={handleCopy} edge="end">
                        {copied ? <CheckIcon color="success" /> : <CopyIcon />}
                      </IconButton>
                    </InputAdornment>
                  ),
                  sx: { fontFamily: 'monospace', fontSize: '0.8rem' }
                }}
              />
              <Typography variant="body2" color="success.main">
                Vhost "{vhostName}" created successfully!
              </Typography>
            </Stack>
          )}
        </DialogContent>
        <DialogActions sx={{ px: 3, pb: 3 }}>
          {!connectionString ? (
            <>
              <Button onClick={() => handleOpenChange(false)}>Cancel</Button>
              <Button
                variant="contained"
                onClick={handleCreate}
                disabled={isCreating || !vhostName.trim()}
              >
                {isCreating ? 'Creating...' : 'Create Vhost'}
              </Button>
            </>
          ) : (
            <Button variant="contained" onClick={() => handleOpenChange(false)}>Close</Button>
          )}
        </DialogActions>
      </Dialog>
    </>
  )
}
