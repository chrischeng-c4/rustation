import { useState, useEffect } from 'react'
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
import type { AgentProfile } from '@/types/state'

interface ProfileEditorDialogProps {
  /** Whether the dialog is open */
  open: boolean
  /** Callback when dialog should close */
  onOpenChange: (open: boolean) => void
  /** Profile being edited (undefined for new profile) */
  profile?: AgentProfile
  /** Callback when save is clicked */
  onSave: (name: string, prompt: string) => void
}

/**
 * Dialog for creating or editing an agent profile.
 */
export function ProfileEditorDialog({
  open,
  onOpenChange,
  profile,
  onSave,
}: ProfileEditorDialogProps) {
  const [name, setName] = useState('')
  const [prompt, setPrompt] = useState('')
  const [errors, setErrors] = useState<{ name?: string; prompt?: string }>({})

  const isEditing = !!profile

  // Reset form when dialog opens/closes or profile changes
  useEffect(() => {
    if (open) {
      setName(profile?.name || '')
      setPrompt(profile?.prompt || '')
      setErrors({})
    } else {
      setName('')
      setPrompt('')
      setErrors({})
    }
  }, [open, profile])

  const validate = () => {
    const newErrors: { name?: string; prompt?: string } = {}

    if (!name.trim()) {
      newErrors.name = 'Name is required'
    }

    if (!prompt.trim()) {
      newErrors.prompt = 'Prompt is required'
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSave = () => {
    if (validate()) {
      onSave(name.trim(), prompt.trim())
      onOpenChange(false)
    }
  }

  return (
    <Dialog open={open} onClose={() => onOpenChange(false)} maxWidth="md" fullWidth>
      <DialogTitle>{isEditing ? 'Edit Profile' : 'Create New Profile'}</DialogTitle>
      <DialogContent>
        <DialogContentText sx={{ mb: 3 }}>
          {isEditing
            ? 'Update the profile name and system prompt.'
            : 'Create a custom agent profile with specific instructions.'}
        </DialogContentText>

        <Stack spacing={3} sx={{ mt: 1 }}>
          <TextField
            label="Profile Name"
            placeholder="e.g. Rust Expert, Code Reviewer"
            fullWidth
            required
            value={name}
            onChange={(e) => setName(e.target.value)}
            error={!!errors.name}
            helperText={errors.name}
          />

          <TextField
            label="System Prompt"
            placeholder="Describe the agent's role and rules..."
            multiline
            rows={12}
            fullWidth
            required
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            error={!!errors.prompt}
            helperText={errors.prompt || `${prompt.length.toLocaleString()} characters`}
            InputProps={{ sx: { fontFamily: 'monospace', fontSize: '0.85rem' } }}
          />
        </Stack>
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 3 }}>
        <Button onClick={() => onOpenChange(false)}>Cancel</Button>
        <Button variant="contained" onClick={handleSave} sx={{ borderRadius: 2 }}>
          {isEditing ? 'Save Changes' : 'Create Profile'}
        </Button>
      </DialogActions>
    </Dialog>
  )
}
