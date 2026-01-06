import React, { useState, useCallback } from 'react'
import {
  Description as FileTextIcon,
  Add as PlusIcon,
  Info as InfoIcon,
  Edit as PencilIcon,
  Delete as Trash2Icon,
  Star as StarIcon,
  ExpandMore as ExpandMoreIcon
} from '@mui/icons-material'
import {
  Button,
  Card,
  CardContent,
  Box,
  Typography,
  Paper,
  Stack,
  Divider,
  IconButton,
  Tooltip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  TextField,
  Chip,
  alpha
} from '@mui/material'
import { PageHeader } from '@/components/shared/PageHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { useAppState } from '@/hooks/useAppState'
import type { ConstitutionPreset } from '@/types/state'

interface PresetEditorDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  preset?: ConstitutionPreset
  onSave: (name: string, prompt: string) => void
}

function PresetEditorDialog({ open, onOpenChange, preset, onSave }: PresetEditorDialogProps) {
  const [name, setName] = useState('')
  const [prompt, setPrompt] = useState('')
  const [errors, setErrors] = useState<{ name?: string; prompt?: string }>({})

  const isEditing = !!preset

  // Reset form when dialog opens/closes or preset changes
  React.useEffect(() => {
    if (open) {
      setName(preset?.name || '')
      setPrompt(preset?.prompt || '')
      setErrors({})
    } else {
      setName('')
      setPrompt('')
      setErrors({})
    }
  }, [open, preset])

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
      <DialogTitle>{isEditing ? 'Edit Preset' : 'Create New Preset'}</DialogTitle>
      <DialogContent>
        <DialogContentText sx={{ mb: 3 }}>
          {isEditing
            ? 'Update the preset name and system prompt.'
            : 'Create a custom constitution preset with specific instructions.'}
        </DialogContentText>

        <Stack spacing={3} sx={{ mt: 1 }}>
          <TextField
            label="Preset Name"
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
            placeholder="Describe the AI's role and rules..."
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
          {isEditing ? 'Save Changes' : 'Create Preset'}
        </Button>
      </DialogActions>
    </Dialog>
  )
}

/**
 * Constitution Presets Management Panel.
 */
export function PresetsPanel() {
  const { state, dispatch, isLoading } = useAppState()

  // Dialog state
  const [isEditorOpen, setIsEditorOpen] = useState(false)
  const [editingPreset, setEditingPreset] = useState<ConstitutionPreset | undefined>()

  // Get presets config from active worktree
  const activeProject = state.projects[state.active_project_index]
  const activeWorktree = activeProject?.worktrees[activeProject?.active_worktree_index ?? 0]
  const presetsConfig = activeWorktree?.tasks?.constitution_presets

  // Handlers
  const handleSelectPreset = useCallback(
    async (presetId: string | null) => {
      await dispatch({
        type: 'SelectConstitutionPreset',
        payload: { preset_id: presetId },
      })
    },
    [dispatch],
  )

  const handleCreatePreset = useCallback(() => {
    setEditingPreset(undefined)
    setIsEditorOpen(true)
  }, [])

  const handleEditPreset = useCallback((preset: ConstitutionPreset) => {
    setEditingPreset(preset)
    setIsEditorOpen(true)
  }, [])

  const handleDeletePreset = useCallback(
    async (presetId: string) => {
      if (confirm('Are you sure you want to delete this preset?')) {
        await dispatch({
          type: 'DeleteConstitutionPreset',
          payload: { id: presetId },
        })
      }
    },
    [dispatch],
  )

  const handleSavePreset = useCallback(
    async (name: string, prompt: string) => {
      if (editingPreset) {
        await dispatch({
          type: 'UpdateConstitutionPreset',
          payload: {
            id: editingPreset.id,
            name,
            prompt,
          },
        })
      } else {
        await dispatch({
          type: 'CreateConstitutionPreset',
          payload: { name, prompt },
        })
      }
    },
    [editingPreset, dispatch],
  )

  // Loading state
  if (isLoading) {
    return <LoadingState />
  }

  // No presets config
  if (!presetsConfig) {
    return (
      <EmptyState
        title="No Worktree Active"
        description="Open a worktree to manage constitution presets"
      />
    )
  }

  const activePreset = presetsConfig.presets.find((p) => p.id === presetsConfig.active_preset_id)
  const builtinPresets = presetsConfig.presets.filter((p) => p.is_builtin)
  const customPresets = presetsConfig.presets.filter((p) => !p.is_builtin)

  return (
    <Box sx={{ height: '100%', overflow: 'auto', p: 3 }}>
      <PageHeader
        title="Constitution Presets"
        description="Full system prompt replacement mode"
        icon={<FileTextIcon />}
      />

      <Stack spacing={4}>
        {/* Active Preset */}
        {activePreset && (
          <Paper variant="outlined" sx={{ p: 3, bgcolor: 'primary.container', borderColor: 'primary.main', borderRadius: 4 }}>
            <Stack direction="row" justifyContent="space-between" alignItems="flex-start" sx={{ mb: 2 }}>
              <Box>
                <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 0.5 }}>
                  {activePreset.is_builtin && <StarIcon sx={{ fontSize: 18, color: 'warning.main' }} />}
                  <Typography variant="h6" fontWeight={700} color="primary.contrastText">{activePreset.name}</Typography>
                </Stack>
                <Typography variant="caption" sx={{ color: 'primary.contrastText', opacity: 0.8 }}>Active Preset</Typography>
              </Box>
              <Button
                variant="outlined"
                size="small"
                onClick={() => handleSelectPreset(null)}
                sx={{ borderRadius: 2, color: 'primary.contrastText', borderColor: 'primary.contrastText' }}
              >
                Deactivate
              </Button>
            </Stack>
            <Box sx={{ mt: 2, p: 2, bgcolor: alpha('#000', 0.2), borderRadius: 2 }}>
              <Typography variant="caption" display="block" sx={{ mb: 1, color: 'primary.contrastText', fontWeight: 700, opacity: 0.7 }}>SYSTEM PROMPT:</Typography>
              <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap', color: 'primary.contrastText', maxHeight: 150, overflow: 'auto' }}>
                {activePreset.prompt}
              </Typography>
            </Box>
          </Paper>
        )}

        <Button
          variant="contained"
          fullWidth
          onClick={handleCreatePreset}
          startIcon={<PlusIcon />}
          sx={{ borderRadius: 2, py: 1.5 }}
        >
          Create New Preset
        </Button>

        {/* Built-in Presets */}
        {builtinPresets.length > 0 && (
          <Box>
            <Typography variant="caption" fontWeight={700} sx={{ textTransform: 'uppercase', letterSpacing: '0.05em', color: 'text.secondary', mb: 1.5, display: 'block' }}>
              Built-in Presets
            </Typography>
            <Stack spacing={1.5}>
              {builtinPresets.map((preset) => (
                <Paper
                  key={preset.id}
                  variant="outlined"
                  onClick={() => handleSelectPreset(preset.id)}
                  sx={{
                    p: 2,
                    cursor: 'pointer',
                    transition: 'all 0.2s',
                    borderColor: presetsConfig.active_preset_id === preset.id ? 'primary.main' : 'outlineVariant',
                    bgcolor: presetsConfig.active_preset_id === preset.id ? 'action.selected' : 'background.paper',
                    '&:hover': { borderColor: 'primary.main', bgcolor: 'action.hover' }
                  }}
                >
                  <Stack direction="row" justifyContent="space-between" alignItems="flex-start">
                    <Box sx={{ flex: 1, minWidth: 0 }}>
                      <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 0.5 }}>
                        <StarIcon sx={{ fontSize: 16, color: 'warning.main' }} />
                        <Typography variant="subtitle2" fontWeight={700}>{preset.name}</Typography>
                      </Stack>
                      <Typography variant="caption" color="text.secondary" noWrap display="block" sx={{ opacity: 0.8 }}>
                        {preset.prompt.split('\n')[0]}
                      </Typography>
                    </Box>
                    <Chip label="Built-in" size="small" variant="outlined" sx={{ height: 18, fontSize: '0.6rem', borderRadius: 0.5 }} />
                  </Stack>
                </Paper>
              ))}
            </Stack>
          </Box>
        )}

        {/* Custom Presets */}
        <Box>
          <Typography variant="caption" fontWeight={700} sx={{ textTransform: 'uppercase', letterSpacing: '0.05em', color: 'text.secondary', mb: 1.5, display: 'block' }}>
            Custom Presets
          </Typography>
          {customPresets.length === 0 ? (
            <Paper variant="outlined" sx={{ p: 4, textAlign: 'center', bgcolor: 'surfaceContainerLow.main', borderStyle: 'dashed' }}>
              <Typography variant="body2" color="text.secondary">No custom presets yet. Create your first preset!</Typography>
            </Paper>
          ) : (
            <Stack spacing={1.5}>
              {customPresets.map((preset) => (
                <Paper
                  key={preset.id}
                  variant="outlined"
                  onClick={() => handleSelectPreset(preset.id)}
                  sx={{
                    p: 2,
                    cursor: 'pointer',
                    transition: 'all 0.2s',
                    borderColor: presetsConfig.active_preset_id === preset.id ? 'primary.main' : 'outlineVariant',
                    bgcolor: presetsConfig.active_preset_id === preset.id ? 'action.selected' : 'background.paper',
                    '&:hover': { borderColor: 'primary.main', bgcolor: 'action.hover' }
                  }}
                >
                  <Stack direction="row" justifyContent="space-between" alignItems="flex-start">
                    <Box sx={{ flex: 1, minWidth: 0 }}>
                      <Typography variant="subtitle2" fontWeight={700} sx={{ mb: 0.5 }}>{preset.name}</Typography>
                      <Typography variant="caption" color="text.secondary" noWrap display="block" sx={{ opacity: 0.8 }}>
                        {preset.prompt.split('\n')[0] || 'No description'}
                      </Typography>
                      <Typography variant="caption" sx={{ color: 'text.disabled', mt: 0.5, display: 'block' }}>
                        Updated {new Date(preset.updated_at).toLocaleDateString()}
                      </Typography>
                    </Box>
                    <Stack direction="row" spacing={0.5}>
                      <IconButton
                        size="small"
                        onClick={(e) => {
                          e.stopPropagation()
                          handleEditPreset(preset)
                        }}
                      >
                        <PencilIcon fontSize="inherit" />
                      </IconButton>
                      <IconButton
                        size="small"
                        color="error"
                        onClick={(e) => {
                          e.stopPropagation()
                          handleDeletePreset(preset.id)
                        }}
                      >
                        <Trash2Icon fontSize="inherit" />
                      </IconButton>
                    </Stack>
                  </Stack>
                </Paper>
              ))}
            </Stack>
          )}
        </Box>

        {/* Info Card */}
        <Paper variant="outlined" sx={{ p: 2.5, bgcolor: 'surfaceContainerLow.main', borderRadius: 3 }}>
          <Stack direction="row" spacing={2}>
            <InfoIcon color="primary" />
            <Box>
              <Typography variant="subtitle2" fontWeight={700} gutterBottom>How Presets Work</Typography>
              <Box component="ul" sx={{ m: 0, pl: 2, typography: 'caption', color: 'text.secondary', '& li': { mb: 0.5 } }}>
                <li>Presets <strong>replace</strong> the entire system prompt (not append)</li>
                <li>Built-in presets (⭐) provide templates for common roles</li>
                <li>Create custom presets to define your own coding standards</li>
                <li>Only one preset can be active at a time</li>
                <li>Built-in presets cannot be edited or deleted</li>
              </Box>
            </Box>
          </Stack>
        </Paper>
      </Stack>

      <PresetEditorDialog
        open={isEditorOpen}
        onOpenChange={setIsEditorOpen}
        preset={editingPreset}
        onSave={handleSavePreset}
      />
    </Box>
  )
}
