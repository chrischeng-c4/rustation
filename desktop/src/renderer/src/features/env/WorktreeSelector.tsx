import { useMemo, useState } from 'react'
import {
  ExpandMore as ChevronDownIcon,
  AccountTree as GitBranchIcon
} from '@mui/icons-material'
import {
  Button,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Typography,
  Box,
  Stack,
  Divider,
  alpha
} from '@mui/material'
import type { WorktreeState } from '@/types/state'

interface WorktreeSelectorProps {
  /** All available worktrees */
  worktrees: WorktreeState[]
  /** Currently selected worktree path */
  value: string | null
  /** Callback when selection changes */
  onChange: (path: string) => void
  /** Label for the selector */
  label?: string
  /** Paths to exclude from selection (e.g., the source worktree) */
  excludePaths?: string[]
  /** Placeholder text when nothing is selected */
  placeholder?: string
  /** Whether the selector is disabled */
  disabled?: boolean
}

/**
 * Dropdown selector for choosing a worktree.
 */
export function WorktreeSelector({
  worktrees,
  value,
  onChange,
  label,
  excludePaths = [],
  placeholder = 'Select worktree',
  disabled = false,
}: WorktreeSelectorProps) {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null)
  const open = Boolean(anchorEl)

  const filteredWorktrees = useMemo(
    () => worktrees.filter((w) => !excludePaths.includes(w.path)),
    [worktrees, excludePaths]
  )

  const selectedWorktree = useMemo(
    () => worktrees.find((w) => w.path === value),
    [worktrees, value]
  )

  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget)
  }

  const handleClose = () => {
    setAnchorEl(null)
  }

  const handleItemClick = (path: string) => {
    onChange(path)
    handleClose()
  }

  return (
    <Stack spacing={1}>
      {label && <Typography variant="caption" fontWeight={700} color="text.secondary" sx={{ textTransform: 'uppercase' }}>{label}</Typography>}
      
      <Button
        variant="outlined"
        fullWidth
        disabled={disabled || filteredWorktrees.length === 0}
        onClick={handleClick}
        endIcon={<ChevronDownIcon />}
        sx={{ 
          justifyContent: 'space-between', 
          height: 48, 
          px: 2, 
          borderRadius: 2,
          borderColor: 'outlineVariant',
          bgcolor: 'background.paper',
          textTransform: 'none'
        }}
      >
        <Stack direction="row" spacing={1.5} alignItems="center" sx={{ minWidth: 0 }}>
          <GitBranchIcon fontSize="small" color={selectedWorktree ? 'primary' : 'inherit'} />
          <Typography variant="body2" fontWeight={selectedWorktree ? 700 : 400} noWrap sx={{ color: selectedWorktree ? 'text.primary' : 'text.disabled' }}>
            {selectedWorktree ? selectedWorktree.branch : placeholder}
          </Typography>
        </Stack>
      </Button>

      <Menu
        anchorEl={anchorEl}
        open={open}
        onClose={handleClose}
        PaperProps={{ sx: { width: 320, mt: 1 } }}
      >
        {filteredWorktrees.map((worktree) => (
          <MenuItem 
            key={worktree.path} 
            onClick={() => handleItemClick(worktree.path)}
            selected={value === worktree.path}
            sx={{ py: 1.5 }}
          >
            <ListItemIcon>
              <GitBranchIcon fontSize="small" color={value === worktree.path ? 'primary' : 'inherit'} />
            </ListItemIcon>
            <ListItemText
              primary={
                <Stack direction="row" spacing={1} alignItems="center">
                  <Typography variant="body2" fontWeight={value === worktree.path ? 700 : 500}>{worktree.branch}</Typography>
                  {worktree.is_main && <Typography variant="caption" sx={{ opacity: 0.5, fontSize: '0.65rem' }}>(main)</Typography>}
                </Stack>
              }
              secondary={worktree.path}
              secondaryTypographyProps={{ variant: 'caption', noWrap: true, sx: { maxWidth: 240, display: 'block' } }}
            />
          </MenuItem>
        ))}
        {filteredWorktrees.length === 0 && (
          <MenuItem disabled>
            <Typography variant="body2" color="text.secondary">No worktrees available</Typography>
          </MenuItem>
        )}
      </Menu>
    </Stack>
  )
}
