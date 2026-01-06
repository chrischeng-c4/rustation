import {
  Check as CheckIcon,
  ExpandMore as ChevronDownIcon,
  Star as StarIcon
} from '@mui/icons-material'
import {
  Button,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  Box,
  Typography
} from '@mui/material'
import { useState } from 'react'
import type { AgentProfile } from '@/types/state'

interface ProfileSelectorProps {
  profiles: AgentProfile[]
  activeProfileId?: string
  onSelect: (profileId: string | undefined) => void
  disabled?: boolean
}

/**
 * Dropdown selector for choosing an agent profile.
 */
export function ProfileSelector({
  profiles,
  activeProfileId,
  onSelect,
  disabled,
}: ProfileSelectorProps) {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null)
  const open = Boolean(anchorEl)

  const activeProfile = profiles.find((p) => p.id === activeProfileId)
  const builtinProfiles = profiles.filter((p) => p.is_builtin)
  const customProfiles = profiles.filter((p) => !p.is_builtin)

  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget)
  }

  const handleClose = () => {
    setAnchorEl(null)
  }

  const handleItemClick = (profileId: string | undefined) => {
    onSelect(profileId)
    handleClose()
  }

  return (
    <Box>
      <Button
        variant="outlined"
        disabled={disabled}
        fullWidth
        onClick={handleClick}
        endIcon={<ChevronDownIcon />}
        sx={{ justifyContent: 'space-between', px: 2, height: 48, borderRadius: 2 }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', overflow: 'hidden' }}>
          {activeProfile ? (
            <>
              {activeProfile.is_builtin && (
                <StarIcon sx={{ fontSize: 16, color: 'warning.main', mr: 1 }} />
              )}
              <Typography variant="body2" fontWeight={600} noWrap>{activeProfile.name}</Typography>
            </>
          ) : (
            <Typography variant="body2" color="text.secondary">Select a profile...</Typography>
          )}
        </Box>
      </Button>

      <Menu
        anchorEl={anchorEl}
        open={open}
        onClose={handleClose}
        PaperProps={{ sx: { width: 300, mt: 1 } }}
      >
        {/* Built-in Profiles */}
        {builtinProfiles.length > 0 && [
          <Box key="label-builtin" sx={{ px: 2, py: 1 }}><Typography variant="caption" fontWeight={700} color="text.secondary">BUILT-IN PROFILES</Typography></Box>,
          ...builtinProfiles.map((profile) => (
            <MenuItem key={profile.id} onClick={() => handleItemClick(profile.id)}>
              <ListItemIcon><StarIcon fontSize="small" sx={{ color: 'warning.main' }} /></ListItemIcon>
              <ListItemText primary={profile.name} primaryTypographyProps={{ variant: 'body2' }} />
              {activeProfileId === profile.id && <CheckIcon fontSize="small" color="primary" />}
            </MenuItem>
          ))
        ]}

        {/* Custom Profiles */}
        {customProfiles.length > 0 && [
          builtinProfiles.length > 0 && <Divider key="divider" sx={{ my: 1 }} />,
          <Box key="label-custom" sx={{ px: 2, py: 1 }}><Typography variant="caption" fontWeight={700} color="text.secondary">CUSTOM PROFILES</Typography></Box>,
          ...customProfiles.map((profile) => (
            <MenuItem key={profile.id} onClick={() => handleItemClick(profile.id)}>
              <ListItemText primary={profile.name} inset={false} sx={{ pl: 4 }} primaryTypographyProps={{ variant: 'body2' }} />
              {activeProfileId === profile.id && <CheckIcon fontSize="small" color="primary" />}
            </MenuItem>
          ))
        ]}

        {/* None Option */}
        <Divider sx={{ my: 1 }} />
        <MenuItem onClick={() => handleItemClick(undefined)}>
          <ListItemText primary="None (use CLAUDE.md)" sx={{ pl: 4, color: 'text.secondary' }} primaryTypographyProps={{ variant: 'body2' }} />
          {!activeProfileId && <CheckIcon fontSize="small" color="primary" />}
        </MenuItem>
      </Menu>
    </Box>
  )
}
