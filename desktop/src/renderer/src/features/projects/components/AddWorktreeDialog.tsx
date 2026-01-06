import { useState, useEffect } from 'react'
import {
  AccountTree as GitBranchIcon,
  Add as PlusIcon,
  Refresh as RefreshIcon
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
  Stack,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Paper,
  CircularProgress,
  Divider
} from '@mui/material'

interface BranchInfo {
  name: string
  hasWorktree: boolean
  isCurrent: boolean
}

type DialogMode = 'select' | 'new'

interface AddWorktreeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  repoPath: string
  onAddFromBranch: (branch: string) => Promise<void>
  onAddNewBranch: (branch: string) => Promise<void>
}

export function AddWorktreeDialog({
  open,
  onOpenChange,
  repoPath,
  onAddFromBranch,
  onAddNewBranch,
}: AddWorktreeDialogProps) {
  const [mode, setMode] = useState<DialogMode>('select')
  const [branches, setBranches] = useState<BranchInfo[]>([])
  const [isLoadingBranches, setIsLoadingBranches] = useState(false)
  const [selectedBranch, setSelectedBranch] = useState<string | null>(null)
  const [newBranchName, setNewBranchName] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Load branches when dialog opens
  useEffect(() => {
    if (open && repoPath) {
      loadBranches()
    }
  }, [open, repoPath])

  const loadBranches = async () => {
    setIsLoadingBranches(true)
    setError(null)
    try {
      const branchList = await window.api.worktree.listBranches(repoPath)
      setBranches(branchList)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load branches')
    } finally {
      setIsLoadingBranches(false)
    }
  }

  const handleAddFromBranch = async () => {
    if (!selectedBranch) return

    setIsCreating(true)
    setError(null)
    try {
      await onAddFromBranch(selectedBranch)
      handleClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create worktree')
    } finally {
      setIsCreating(false)
    }
  }

  const handleAddNewBranch = async () => {
    if (!newBranchName.trim()) {
      setError('Branch name is required')
      return
    }

    if (!/^[a-zA-Z0-9._/-]+$/.test(newBranchName)) {
      setError('Invalid branch name. Use only letters, numbers, dots, underscores, slashes, and hyphens.')
      return
    }

    setIsCreating(true)
    setError(null)
    try {
      await onAddNewBranch(newBranchName)
      handleClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create worktree')
    } finally {
      setIsCreating(false)
    }
  }

  const handleClose = () => {
    onOpenChange(false)
    setTimeout(() => {
      setMode('select')
      setSelectedBranch(null)
      setNewBranchName('')
      setError(null)
      setBranches([])
    }, 200)
  }

  const availableBranches = branches.filter(b => !b.hasWorktree)

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>Add Worktree</DialogTitle>
      <DialogContent>
        <DialogContentText sx={{ mb: 2 }}>
          {mode === 'select'
            ? 'Select an existing branch or create a new one'
            : 'Enter a name for the new branch'}
        </DialogContentText>

        {mode === 'select' ? (
          <Stack spacing={3}>
            {isLoadingBranches ? (
              <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}>
                <CircularProgress size={32} />
              </Box>
            ) : (
              <Box>
                <Typography variant="caption" fontWeight={700} color="text.secondary" sx={{ textTransform: 'uppercase', mb: 1, display: 'block' }}>
                  Available Branches
                </Typography>
                {availableBranches.length === 0 ? (
                  <Paper variant="outlined" sx={{ p: 3, textAlign: 'center', bgcolor: 'action.hover' }}>
                    <Typography variant="body2" color="text.secondary">
                      All branches already have worktrees
                    </Typography>
                  </Paper>
                ) : (
                  <Paper variant="outlined" sx={{ maxHeight: 240, overflow: 'auto', bgcolor: 'background.default' }}>
                    <List sx={{ p: 0 }}>
                      {availableBranches.map((branch) => (
                        <ListItem key={branch.name} disablePadding divider>
                          <ListItemButton
                            selected={selectedBranch === branch.name}
                            onClick={() => setSelectedBranch(branch.name)}
                            sx={{ py: 1.5 }}
                          >
                            <ListItemIcon sx={{ minWidth: 36 }}>
                              <GitBranchIcon fontSize="small" color={selectedBranch === branch.name ? 'primary' : 'inherit'} />
                            </ListItemIcon>
                            <ListItemText
                              primary={branch.name}
                              primaryTypographyProps={{ variant: 'body2', fontWeight: selectedBranch === branch.name ? 700 : 400 }}
                              secondary={branch.isCurrent ? '(current)' : null}
                              secondaryTypographyProps={{ variant: 'caption' }}
                            />
                          </ListItemButton>
                        </ListItem>
                      ))}
                    </List>
                  </Paper>
                )}
              </Box>
            )}

            <Button
              variant="outlined"
              fullWidth
              onClick={() => setMode('new')}
              startIcon={<PlusIcon />}
              sx={{ borderRadius: 2 }}
            >
              Create New Branch
            </Button>

            {error && (
              <Typography variant="caption" color="error">{error}</Typography>
            )}
          </Stack>
        ) : (
          <Stack spacing={3}>
            <Box>
              <TextField
                autoFocus
                label="Branch Name"
                placeholder="feature/my-new-feature"
                fullWidth
                value={newBranchName}
                onChange={(e) => setNewBranchName(e.target.value)}
                disabled={isCreating}
                error={!!error}
                helperText={error || "A new branch will be created from the current HEAD"}
              />
            </Box>
          </Stack>
        )}
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 3 }}>
        {mode === 'select' ? (
          <>
            <Button onClick={handleClose}>Cancel</Button>
            <Button
              variant="contained"
              onClick={handleAddFromBranch}
              disabled={!selectedBranch || isCreating}
              startIcon={isCreating && <RefreshIcon sx={{ animation: 'spin 2s linear infinite' }} />}
            >
              {isCreating ? 'Creating...' : 'Add Worktree'}
            </Button>
          </>
        ) : (
          <>
            <Button onClick={() => setMode('select')}>Back</Button>
            <Button
              variant="contained"
              onClick={handleAddNewBranch}
              disabled={!newBranchName.trim() || isCreating}
              startIcon={isCreating && <RefreshIcon sx={{ animation: 'spin 2s linear infinite' }} />}
            >
              {isCreating ? 'Creating...' : 'Create & Add Worktree'}
            </Button>
          </>
        )}
      </DialogActions>
      <style>{`@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
    </Dialog>
  )
}
