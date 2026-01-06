import { useState } from 'react'
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
  Box
} from '@mui/material'

interface NewChangeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSubmit: (intent: string) => void
}

/**
 * NewChangeDialog - Dialog to create a new change from intent
 */
export function NewChangeDialog({ open, onOpenChange, onSubmit }: NewChangeDialogProps) {
  const [intent, setIntent] = useState('')

  const handleSubmit = () => {
    if (intent.trim()) {
      onSubmit(intent.trim())
      setIntent('')
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && e.metaKey) {
      handleSubmit()
    }
  }

  return (
    <Dialog 
      open={open} 
      onClose={() => onOpenChange(false)}
      maxWidth="sm"
      fullWidth
    >
      <DialogTitle>Create New Change</DialogTitle>
      <DialogContent>
        <DialogContentText sx={{ mb: 3 }}>
          Describe what you want to accomplish. This will be used to generate a proposal and plan.
        </DialogContentText>

        <Stack spacing={2} sx={{ mt: 1 }}>
          <TextField
            autoFocus
            label="Intent"
            placeholder="e.g., Add user authentication with OAuth2 support"
            multiline
            rows={4}
            fullWidth
            value={intent}
            onChange={(e) => setIntent(e.target.value)}
            onKeyDown={handleKeyDown}
            helperText="Be specific about what you want to build. Press Cmd+Enter to submit."
          />
        </Stack>
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 3 }}>
        <Button onClick={() => onOpenChange(false)}>
          Cancel
        </Button>
        <Button 
          variant="contained" 
          onClick={handleSubmit} 
          disabled={!intent.trim()}
          sx={{ borderRadius: 2 }}
        >
          Create Change
        </Button>
      </DialogActions>
    </Dialog>
  )
}
