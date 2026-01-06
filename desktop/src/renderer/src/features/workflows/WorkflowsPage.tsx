import { useState } from 'react'
import {
  Description as DescriptionIcon,
  MenuBook as BookIcon,
  AccountTree as GitIcon,
  ChevronRight
} from '@mui/icons-material'
import {
  Box,
  Typography,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Paper,
  Stack,
  useTheme
} from '@mui/material'
import { EmptyState } from '@/components/shared/EmptyState'
import { ConstitutionPanel } from './ConstitutionPanel'
import { ChangeManagementPanel } from './ChangeManagementPanel'
import { ContextPanel } from './ContextPanel'

/**
 * Available workflow definitions.
 */
const WORKFLOWS = [
  {
    id: 'constitution-management',
    name: 'Constitution Management',
    description: 'Initialize or update project constitution for AI-assisted development',
    icon: <DescriptionIcon />,
  },
  {
    id: 'context-management',
    name: 'Context Management',
    description: 'View and manage project context - tech stack, architecture, recent changes',
    icon: <BookIcon />,
  },
  {
    id: 'change-management',
    name: 'Change Management',
    description: 'Create and manage changes with proposal, plan generation, and review',
    icon: <GitIcon />,
  },
]

/**
 * WorkflowsPage - State machine driven guided workflows.
 */
export function WorkflowsPage() {
  const [selectedWorkflow, setSelectedWorkflow] = useState<string | null>('constitution-management')
  const theme = useTheme()

  const renderWorkflowPanel = () => {
    switch (selectedWorkflow) {
      case 'constitution-management':
        return <ConstitutionPanel />
      case 'context-management':
        return <ContextPanel />
      case 'change-management':
        return <ChangeManagementPanel />
      default:
        // Use a generic icon for empty state
        return (
          <EmptyState
            title="Select a Workflow"
            description="Choose a workflow from the list on the left to begin."
          />
        )
    }
  }

  return (
    <Box sx={{ display: 'flex', height: '100%', gap: 3 }}>
      {/* Workflow List (Left Column) - M3 Navigation Drawer style */}
      <Paper
        elevation={0}
        sx={{
          width: 320,
          flexShrink: 0,
          bgcolor: 'surfaceContainerLow.main', // M3 Surface Container Low
          borderRadius: 4,
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column'
        }}
      >
        <Box sx={{ p: 3, pb: 2 }}>
          <Typography variant="h6" fontWeight={500}>
            Workflows
          </Typography>
        </Box>

        <List sx={{ px: 2, pb: 2, overflowY: 'auto' }}>
          {WORKFLOWS.map((workflow) => {
            const isSelected = selectedWorkflow === workflow.id

            return (
              <ListItem key={workflow.id} disablePadding sx={{ mb: 1 }}>
                <ListItemButton
                  selected={isSelected}
                  onClick={() => setSelectedWorkflow(workflow.id)}
                  sx={{
                    borderRadius: 4, // M3 shape
                    minHeight: 88, // Allow height for 3 lines if needed, generally tall
                    alignItems: 'flex-start',
                    gap: 2,
                    py: 2,
                    // Active state colors
                    '&.Mui-selected': {
                      bgcolor: 'secondary.container',
                      color: 'onSecondaryContainer',
                      '&:hover': {
                        bgcolor: 'secondary.container', // maintain color on hover
                        filter: 'brightness(0.95)',
                      },
                      '& .MuiListItemIcon-root': {
                        color: 'onSecondaryContainer',
                      },
                      '& .MuiTypography-body2': {
                        color: 'onSecondaryContainer', // Description color
                        opacity: 0.8,
                      }
                    },
                  }}
                >
                  <ListItemIcon
                    sx={{
                      minWidth: 0,
                      mt: 0.5,
                      color: isSelected ? 'inherit' : 'onSurfaceVariant',
                    }}
                  >
                    {workflow.icon}
                  </ListItemIcon>

                  <ListItemText
                    primary={
                      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                        <Typography variant="subtitle2" fontWeight={600}>
                          {workflow.name}
                        </Typography>
                        {isSelected && <ChevronRight sx={{ fontSize: 16, ml: 1, opacity: 0.5 }} />}
                      </Box>
                    }
                    secondary={workflow.description}
                    primaryTypographyProps={{ component: 'div' }}
                    secondaryTypographyProps={{
                      variant: 'body2',
                      sx: {
                        display: '-webkit-box',
                        WebkitLineClamp: 2,
                        WebkitBoxOrient: 'vertical',
                        overflow: 'hidden',
                        mt: 0.5,
                        color: isSelected ? 'inherit' : 'onSurfaceVariant',
                      }
                    }}
                  />
                </ListItemButton>
              </ListItem>
            )
          })}
        </List>
      </Paper>

      {/* Workflow Execution Panel (Right Column) */}
      <Paper
        elevation={0}
        sx={{
          flex: 1,
          bgcolor: 'background.paper', // Surface
          borderRadius: 4,
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
          border: 1,
          borderColor: 'outlineVariant',
        }}
      >
        {renderWorkflowPanel()}
      </Paper>
    </Box>
  )
}
