import {
  Edit as PencilIcon,
  Delete as Trash2Icon,
  Star as StarIcon
} from '@mui/icons-material'
import {
  Button,
  Card,
  CardContent,
  Typography,
  Box,
  Stack,
  IconButton,
  Chip,
  Paper
} from '@mui/material'
import type { AgentProfile } from '@/types/state'

interface ProfileListProps {
  /** All available profiles */
  profiles: AgentProfile[]
  /** Currently active profile ID */
  activeProfileId?: string
  /** Callback when edit is clicked */
  onEdit: (profile: AgentProfile) => void
  /** Callback when delete is clicked */
  onDelete: (profileId: string) => void
  /** Callback when a profile is selected */
  onSelect: (profileId: string) => void
}

/**
 * List view of all agent profiles with edit/delete actions.
 */
export function ProfileList({
  profiles,
  activeProfileId,
  onEdit,
  onDelete,
  onSelect,
}: ProfileListProps) {
  if (profiles.length === 0) {
    return (
      <Paper variant="outlined" sx={{ py: 6, textAlign: 'center', bgcolor: 'surfaceContainerLow.main', borderStyle: 'dashed' }}>
        <Typography variant="body2" color="text.secondary">
          No profiles available. Create your first custom profile!
        </Typography>
      </Paper>
    )
  }

  const builtinProfiles = profiles.filter((p) => p.is_builtin)
  const customProfiles = profiles.filter((p) => !p.is_builtin)

  const renderProfileItem = (profile: AgentProfile) => {
    const isSelected = activeProfileId === profile.id
    return (
      <Paper
        key={profile.id}
        variant="outlined"
        onClick={() => onSelect(profile.id)}
        sx={{
          p: 2,
          cursor: 'pointer',
          transition: 'all 0.2s',
          borderColor: isSelected ? 'primary.main' : 'outlineVariant',
          bgcolor: isSelected ? 'action.selected' : 'background.paper',
          '&:hover': { borderColor: 'primary.main', bgcolor: 'action.hover' }
        }}
      >
        <Stack direction="row" justifyContent="space-between" alignItems="flex-start">
          <Box sx={{ flex: 1, minWidth: 0 }}>
            <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 0.5 }}>
              {profile.is_builtin && <StarIcon sx={{ fontSize: 16, color: 'warning.main' }} />}
              <Typography variant="subtitle2" fontWeight={isSelected ? 700 : 600}>{profile.name}</Typography>
            </Stack>
            <Typography variant="caption" color="text.secondary" display="block" noWrap sx={{ opacity: 0.8 }}>
              {profile.prompt.split('\n')[0]}
            </Typography>
            {!profile.is_builtin && (
              <Typography variant="caption" sx={{ color: 'text.disabled', mt: 0.5, display: 'block' }}>
                Updated {new Date(profile.updated_at).toLocaleDateString()}
              </Typography>
            )}
          </Box>
          <Stack direction="row" spacing={0.5} alignItems="center">
            {profile.is_builtin ? (
              <Chip label="Built-in" size="small" variant="outlined" sx={{ height: 18, fontSize: '0.6rem', borderRadius: 0.5 }} />
            ) : (
              <>
                <IconButton
                  size="small"
                  onClick={(e) => {
                    e.stopPropagation()
                    onEdit(profile)
                  }}
                >
                  <PencilIcon fontSize="inherit" />
                </IconButton>
                <IconButton
                  size="small"
                  color="error"
                  onClick={(e) => {
                    e.stopPropagation()
                    onDelete(profile.id)
                  }}
                >
                  <Trash2Icon fontSize="inherit" />
                </IconButton>
              </>
            )}
          </Stack>
        </Stack>
      </Paper>
    )
  }

  return (
    <Stack spacing={3}>
      {/* Built-in Profiles */}
      {builtinProfiles.length > 0 && (
        <Box>
          <Typography variant="caption" fontWeight={700} sx={{ textTransform: 'uppercase', letterSpacing: '0.05em', color: 'text.secondary', mb: 1.5, display: 'block' }}>
            Built-in Profiles
          </Typography>
          <Stack spacing={1}>
            {builtinProfiles.map(renderProfileItem)}
          </Stack>
        </Box>
      )}

      {/* Custom Profiles */}
      {customProfiles.length > 0 && (
        <Box>
          <Typography variant="caption" fontWeight={700} sx={{ textTransform: 'uppercase', letterSpacing: '0.05em', color: 'text.secondary', mb: 1.5, display: 'block' }}>
            Custom Profiles
          </Typography>
          <Stack spacing={1}>
            {customProfiles.map(renderProfileItem)}
          </Stack>
        </Box>
      )}
    </Stack>
  )
}
