import { useEffect, useState } from 'react'
import {
  Add as PlusIcon,
  CheckCircle as CheckCircleIcon,
  AccessTime as ClockIcon,
  Cancel as XCircleIcon,
  AccountTree as GitIcon
} from '@mui/icons-material'
import {
  Button,
  Card,
  CardContent,
  Box,
  Typography,
  Chip,
  Paper,
  Stack,
  Divider,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  ListItemIcon
} from '@mui/material'
import { PageHeader } from '@/components/shared/PageHeader'
import { WorkflowHeader } from '@/components/shared/WorkflowHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { useAppState } from '@/hooks/useAppState'
import { NewChangeDialog } from './NewChangeDialog'
import { ChangeDetailView } from './ChangeDetailView'
import type { Change, ChangeStatus } from '@/types/state'

/**
 * Status badge colors and labels
 */
const STATUS_CONFIG: Record<ChangeStatus, { color: 'info' | 'warning' | 'secondary' | 'success' | 'error' | 'default'; label: string; icon: any }> = {
  proposed: { color: 'info', label: 'Proposed', icon: GitIcon },
  planning: { color: 'warning', label: 'Planning', icon: ClockIcon },
  planned: { color: 'secondary', label: 'Planned', icon: GitIcon },
  implementing: { color: 'warning', label: 'Implementing', icon: ClockIcon },
  testing: { color: 'info', label: 'Testing', icon: ClockIcon },
  done: { color: 'success', label: 'Done', icon: CheckCircleIcon },
  archived: { color: 'default', label: 'Archived', icon: CheckCircleIcon },
  cancelled: { color: 'error', label: 'Cancelled', icon: XCircleIcon },
  failed: { color: 'error', label: 'Failed', icon: XCircleIcon },
}

/**
 * ChangeManagementPanel - CESDD Phase 2 Change Management
 */
export function ChangeManagementPanel() {
  const { state, dispatch } = useAppState()
  const [isDialogOpen, setIsDialogOpen] = useState(false)

  // Get current worktree's changes
  const activeProject = state?.projects?.[state?.active_project_index ?? 0]
  const activeWorktree = activeProject?.worktrees?.[activeProject?.active_worktree_index ?? 0]
  const changesState = activeWorktree?.changes
  const changes = changesState?.changes ?? []
  const selectedChangeId = changesState?.selected_change_id
  const selectedChange = changes.find((c) => c.id === selectedChangeId)
  const isLoading = changesState?.is_loading ?? false

  // Refresh changes on mount
  useEffect(() => {
    dispatch({ type: 'RefreshChanges' })
  }, [dispatch])

  const handleCreateChange = async (intent: string) => {
    dispatch({ type: 'CreateChange', payload: { intent } })
    setIsDialogOpen(false)
  }

  const handleSelectChange = (changeId: string) => {
    dispatch({ type: 'SelectChange', payload: { change_id: changeId } })
  }

  const renderChangeItem = (change: Change) => {
    const config = STATUS_CONFIG[change.status]
    const isSelected = change.id === selectedChangeId

    return (
      <ListItem key={change.id} disablePadding sx={{ mb: 1 }}>
        <ListItemButton
          selected={isSelected}
          onClick={() => handleSelectChange(change.id)}
          sx={{
            borderRadius: 2,
            p: 2,
            border: 1,
            borderColor: isSelected ? 'primary.main' : 'outlineVariant',
            bgcolor: isSelected ? 'secondaryContainer.main' : 'background.paper',
            flexDirection: 'column',
            alignItems: 'stretch',
            '&.Mui-selected': {
              bgcolor: 'secondaryContainer.main',
              color: 'onSecondaryContainer.main',
              '&:hover': { bgcolor: 'secondaryContainer.main', filter: 'brightness(0.95)' }
            }
          }}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
            <Typography variant="subtitle2" fontWeight={700} sx={{ flex: 1, minWidth: 0 }} noWrap>
              {change.name}
            </Typography>
            <Chip
              label={config.label}
              size="small"
              color={config.color as any}
              sx={{ height: 18, fontSize: '0.6rem', fontWeight: 700, borderRadius: 0.5, ml: 1 }}
            />
          </Box>
          <Typography variant="caption" sx={{ color: isSelected ? 'inherit' : 'text.secondary', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>
            {change.intent}
          </Typography>
        </ListItemButton>
      </ListItem>
    )
  }

  // Empty state
  if (!isLoading && changes.length === 0) {
    return (
      <Stack sx={{ height: '100%' }}>
        <PageHeader
          title="Change Management"
          description="Manage features with proposal and plan generation"
          icon={<GitIcon />}
        />
        <Box sx={{ flex: 1, p: 3 }}>
          <EmptyState
            title="Start Change Management"
            description="Create changes to manage features with AI-powered proposals and plans."
            action={{
              label: "Create First Change",
              onClick: () => setIsDialogOpen(true),
              icon: <PlusIcon />
            }}
          />
        </Box>

        <NewChangeDialog
          open={isDialogOpen}
          onOpenChange={setIsDialogOpen}
          onSubmit={handleCreateChange}
        />
      </Stack>
    )
  }

  return (
    <Stack sx={{ height: '100%' }}>
      <WorkflowHeader
        title="Change Management"
        subtitle={`${changes.length} active changes in current worktree`}
        icon={<GitIcon />}
      >
        <Button
          variant="contained"
          size="small"
          onClick={() => setIsDialogOpen(true)}
          startIcon={<PlusIcon />}
          sx={{ borderRadius: 2 }}
        >
          New Change
        </Button>
      </WorkflowHeader>

      <Stack direction="row" spacing={3} sx={{ flex: 1, overflow: 'hidden', p: 3 }}>
        {/* Change List (Left) */}
        <Paper
          variant="outlined"
          sx={{
            width: 280,
            flexShrink: 0,
            display: 'flex',
            flexDirection: 'column',
            bgcolor: 'surfaceContainerLow.main',
            borderRadius: 4,
            overflow: 'hidden'
          }}
        >
          <Box sx={{ px: 2, py: 1.5, borderBottom: 1, borderColor: 'outlineVariant' }}>
            <Typography variant="caption" fontWeight={700} sx={{ textTransform: 'uppercase', letterSpacing: '0.05em', color: 'text.secondary' }}>Active Changes</Typography>
          </Box>
          <Box sx={{ flex: 1, overflow: 'auto', p: 1.5 }}>
            {isLoading ? (
              <LoadingState message="" />
            ) : (
              <List sx={{ p: 0 }}>
                {changes.map(renderChangeItem)}
              </List>
            )}
          </Box>
        </Paper>

        {/* Change Detail (Right) */}
        <Paper
          variant="outlined"
          sx={{
            flex: 1,
            borderRadius: 4,
            overflow: 'hidden',
            display: 'flex',
            flexDirection: 'column',
            bgcolor: 'background.paper'
          }}
        >
          {selectedChange ? (
            <ChangeDetailView change={selectedChange} />
          ) : (
            <EmptyState
              title="No Change Selected"
              description="Select a change from the list on the left to view its details, proposal, and plan."
            />
          )}
        </Paper>
      </Stack>

      {/* New Change Dialog */}
      <NewChangeDialog
        open={isDialogOpen}
        onOpenChange={setIsDialogOpen}
        onSubmit={handleCreateChange}
      />
    </Stack>
  )
}
