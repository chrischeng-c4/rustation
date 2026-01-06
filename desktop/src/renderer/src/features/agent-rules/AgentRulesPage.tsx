import { useState, useCallback } from 'react'
import {
  SmartToy as BotIcon,
  Refresh as RefreshIcon,
  WarningAmber as AlertTriangleIcon,
  Info as InfoIcon,
  Add as PlusIcon
} from '@mui/icons-material'
import {
  Button,
  Card,
  CardContent,
  Box,
  Typography,
  Stack,
  Paper,
  Divider,
  CircularProgress,
  alpha
} from '@mui/material'
import { useAgentRulesState } from '@/hooks/useAppState'
import { ProfileSelector } from './ProfileSelector'
import { ProfileList } from './ProfileList'
import { ProfileEditorDialog } from './ProfileEditorDialog'
import { PageHeader } from '@/components/shared/PageHeader'
import type { AgentProfile } from '@/types/state'

/**
 * Agent Rules Management Page.
 */
export function AgentRulesPage() {
  const { agentRulesConfig, project, dispatch, isLoading } = useAgentRulesState()

  // Dialog state
  const [isEditorOpen, setIsEditorOpen] = useState(false)
  const [editingProfile, setEditingProfile] = useState<AgentProfile | undefined>()

  // Handlers
  const handleToggle = useCallback(async () => {
    if (!agentRulesConfig) return
    await dispatch({
      type: 'SetAgentRulesEnabled',
      payload: { enabled: !agentRulesConfig.enabled },
    })
  }, [agentRulesConfig, dispatch])

  const handleSelectProfile = useCallback(
    async (profileId: string | undefined) => {
      await dispatch({
        type: 'SelectAgentProfile',
        payload: { profile_id: profileId },
      })
    },
    [dispatch],
  )

  const handleCreateProfile = useCallback(() => {
    setEditingProfile(undefined)
    setIsEditorOpen(true)
  }, [])

  const handleEditProfile = useCallback((profile: AgentProfile) => {
    setEditingProfile(profile)
    setIsEditorOpen(true)
  }, [])

  const handleDeleteProfile = useCallback(
    async (profileId: string) => {
      if (confirm('Are you sure you want to delete this profile?')) {
        await dispatch({
          type: 'DeleteAgentProfile',
          payload: { id: profileId },
        })
      }
    },
    [dispatch],
  )

  const handleSaveProfile = useCallback(
    async (name: string, prompt: string) => {
      if (editingProfile) {
        await dispatch({
          type: 'UpdateAgentProfile',
          payload: {
            id: editingProfile.id,
            name,
            prompt,
          },
        })
      } else {
        await dispatch({
          type: 'CreateAgentProfile',
          payload: { name, prompt },
        })
      }
    },
    [editingProfile, dispatch],
  )

  // Loading state
  if (isLoading) {
    return (
      <Box sx={{ display: 'flex', height: '100%', alignItems: 'center', justifyContent: 'center' }}>
        <CircularProgress />
      </Box>
    )
  }

  // No project open
  if (!project || !agentRulesConfig) {
    return (
      <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', p: 3 }}>
        <BotIcon sx={{ fontSize: 64, color: 'text.disabled', mb: 2, opacity: 0.5 }} />
        <Typography variant="h5" fontWeight={600}>No Project Open</Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
          Open a project to customize Claude Code behavior.
        </Typography>
      </Box>
    )
  }

  const isEnabled = agentRulesConfig.enabled
  const activeProfile = agentRulesConfig.profiles.find(
    (p) => p.id === agentRulesConfig.active_profile_id,
  )

  return (
    <Box sx={{ height: '100%', overflow: 'auto', p: 3 }}>
      <Stack spacing={3}>
        {/* Header */}
        <PageHeader
          title="Agent Rules"
          description={`Custom AI behavior for ${project.name}`}
          icon={<BotIcon />}
        >
          <Button
            variant={isEnabled ? 'contained' : 'outlined'}
            onClick={handleToggle}
            startIcon={<BotIcon />}
            sx={{ borderRadius: 2 }}
          >
            {isEnabled ? 'Enabled' : 'Disabled'}
          </Button>
        </PageHeader>

        {/* Warning Card */}
        {isEnabled && activeProfile && (
          <Paper variant="outlined" sx={{ p: 2, bgcolor: alpha('#f9a825', 0.1), borderColor: 'warning.main', borderRadius: 2 }}>
            <Stack direction="row" spacing={2}>
              <AlertTriangleIcon color="warning" />
              <Box>
                <Typography variant="subtitle2" fontWeight={700} color="warning.main">Custom Rules Active</Typography>
                <Typography variant="body2" sx={{ color: 'warning.light', mt: 0.5 }}>
                  Profile <strong>{activeProfile.name}</strong> will <strong>replace</strong> the
                  default CLAUDE.md instructions.
                </Typography>
              </Box>
            </Stack>
          </Paper>
        )}

        {/* Profile Selection */}
        <Card variant="outlined" sx={{ borderRadius: 4 }}>
          <CardContent sx={{ p: 3 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2.5 }}>
              <Typography variant="h6" fontWeight={600}>Active Profile</Typography>
              <Button 
                variant="contained" 
                size="small" 
                onClick={handleCreateProfile} 
                startIcon={<PlusIcon />}
                sx={{ borderRadius: 2 }}
              >
                New Profile
              </Button>
            </Box>

            <ProfileSelector
              profiles={agentRulesConfig.profiles}
              activeProfileId={agentRulesConfig.active_profile_id}
              onSelect={handleSelectProfile}
              disabled={!isEnabled}
            />

            {!isEnabled && (
              <Typography variant="caption" sx={{ color: 'text.secondary', mt: 1.5, display: 'block' }}>
                Enable agent rules to select a profile
              </Typography>
            )}
          </CardContent>
        </Card>

        {/* Active Profile Preview */}
        {activeProfile && (
          <Card variant="outlined" sx={{ borderRadius: 4 }}>
            <CardContent sx={{ p: 3 }}>
              <Typography variant="h6" fontWeight={600} sx={{ mb: 2 }}>Profile Preview</Typography>
              <Stack spacing={1.5}>
                <Stack direction="row" justifyContent="space-between">
                  <Typography variant="body2" color="text.secondary">Name:</Typography>
                  <Typography variant="body2" fontWeight={600}>{activeProfile.name}</Typography>
                </Stack>
                {activeProfile.is_builtin && (
                  <Stack direction="row" justifyContent="space-between">
                    <Typography variant="body2" color="text.secondary">Type:</Typography>
                    <Typography variant="body2" sx={{ color: 'warning.main' }}>⭐ Built-in</Typography>
                  </Stack>
                )}
                <Stack direction="row" justifyContent="space-between">
                  <Typography variant="body2" color="text.secondary">Updated:</Typography>
                  <Typography variant="body2">{new Date(activeProfile.updated_at).toLocaleString()}</Typography>
                </Stack>
                <Box sx={{ mt: 1 }}>
                  <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 1 }}>Prompt:</Typography>
                  <Paper variant="outlined" sx={{ p: 2, bgcolor: 'background.default', borderRadius: 2 }}>
                    <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap', maxHeight: 200, overflow: 'auto' }}>
                      {activeProfile.prompt}
                    </Typography>
                  </Paper>
                </Box>
              </Stack>
            </CardContent>
          </Card>
        )}

        {/* Profile List */}
        <Card variant="outlined" sx={{ borderRadius: 4 }}>
          <CardContent sx={{ p: 3 }}>
            <Typography variant="h6" fontWeight={600} sx={{ mb: 3 }}>All Profiles</Typography>
            <ProfileList
              profiles={agentRulesConfig.profiles}
              activeProfileId={agentRulesConfig.active_profile_id}
              onEdit={handleEditProfile}
              onDelete={handleDeleteProfile}
              onSelect={handleSelectProfile}
            />
          </CardContent>
        </Card>

        {/* Info Card */}
        <Paper variant="outlined" sx={{ p: 2.5, bgcolor: 'surfaceContainerLow.main', borderRadius: 3 }}>
          <Stack direction="row" spacing={2}>
            <InfoIcon color="primary" />
            <Box>
              <Typography variant="subtitle2" fontWeight={700} gutterBottom>How Agent Rules Work</Typography>
              <Box component="ul" sx={{ m: 0, pl: 2, typography: 'caption', color: 'text.secondary', '& li': { mb: 0.5 } }}>
                <li>Select a profile to customize Claude Code's system prompt</li>
                <li>Built-in profiles (⭐) provide expert templates</li>
                <li>Create custom profiles to define your own coding standards</li>
                <li>Built-in profiles cannot be edited or deleted</li>
              </Box>
            </Box>
          </Stack>
        </Paper>
      </Stack>

      <ProfileEditorDialog
        open={isEditorOpen}
        onOpenChange={setIsEditorOpen}
        profile={editingProfile}
        onSave={handleSaveProfile}
      />
    </Box>
  )
}
