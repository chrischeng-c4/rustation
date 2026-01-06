import { useState } from 'react'
import {
  Storage as DatabaseIcon,
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
  Box,
  Typography,
  IconButton,
  Stack,
  InputAdornment
} from '@mui/material'

interface AddDbDialogProps {
  serviceId: string
  serviceName: string
  disabled?: boolean
  onCreateDb?: (serviceId: string, dbName: string) => Promise<string>
}

export function AddDbDialog({
  serviceId,
  serviceName,
  disabled,
  onCreateDb,
}: AddDbDialogProps) {
  const [open, setOpen] = useState(false)
  const [dbName, setDbName] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const [connectionString, setConnectionString] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const handleCreate = async () => {
    if (!dbName.trim()) {
      setError('Database name is required')
      return
    }

    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(dbName)) {
      setError('Database name must start with a letter or underscore and contain only alphanumeric characters and underscores')
      return
    }

    setError(null)
    setIsCreating(true)

    try {
      if (onCreateDb) {
        const connStr = await onCreateDb(serviceId, dbName)
        setConnectionString(connStr)
      } else {
        const mockConnStr = serviceId.includes('postgres')
          ? `postgresql://postgres:postgres@localhost:5432/${dbName}`
          : serviceId.includes('mysql')
          ? `mysql://root:mysql@localhost:3306/${dbName}`
          : `mongodb://localhost:27017/${dbName}`
        setConnectionString(mockConnStr)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create database')
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
      setDbName('')
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
        startIcon={<DatabaseIcon />}
      >
        Add DB
      </Button>

      <Dialog open={open} onClose={() => handleOpenChange(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Database</DialogTitle>
        <DialogContent>
          <DialogContentText sx={{ mb: 3 }}>
            Create a new database in {serviceName}
          </DialogContentText>

          {!connectionString ? (
            <Stack spacing={2} sx={{ mt: 1 }}>
              <TextField
                autoFocus
                label="Database Name"
                placeholder="my_database"
                fullWidth
                value={dbName}
                onChange={(e) => setDbName(e.target.value)}
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
                Database "{dbName}" created successfully!
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
                disabled={isCreating || !dbName.trim()}
              >
                {isCreating ? 'Creating...' : 'Create Database'}
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
