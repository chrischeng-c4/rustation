/**
 * Project tabs component - displays open projects and worktrees at the top of the window.
 *
 * Two-level tab structure with scope-based features (Three-Scope Model):
 * - Top row: Project tabs (git repos) + Docker button (Global scope)
 * - Second row: Worktree sub-tabs (git worktrees) + Env button (Project scope)
 */

import { useState, useCallback } from 'react'
import {
  Close as XIcon,
  Add as PlusIcon,
  FolderOpen as FolderOpenIcon,
  AccountTree as GitBranchIcon,
  History as HistoryIcon,
  Dns as ContainerIcon,
  Code as FileCodeIcon,
  CameraAlt as CameraIcon,
  ExpandMore as ChevronDownIcon,
  CheckCircle as CheckCircleIcon,
} from '@mui/icons-material'
import {
  Button,
  Box,
  Typography,
  Tabs,
  Tab,
  IconButton,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  Stack,
  Tooltip,
  Paper,
  alpha
} from '@mui/material'
import { useActiveProject, useActiveWorktree, useAppState } from '@/hooks/useAppState'
import { NotificationDrawer } from '@/features/notifications'
import { AddWorktreeDialog } from './AddWorktreeDialog'

export function ProjectTabs() {
  const { state } = useAppState()
  const { projects, activeIndex, dispatch } = useActiveProject()
  const { worktrees, activeWorktreeIndex, worktree } = useActiveWorktree()
  const [addWorktreeDialogOpen, setAddWorktreeDialogOpen] = useState(false)
  
  const [projectMenuAnchor, setProjectMenuAnchor] = useState<null | HTMLElement>(null)

  const recentProjects = state?.recent_projects ?? []
  const activeProject = projects[activeIndex]

  const handleOpenProject = async () => {
    setProjectMenuAnchor(null)
    const path = await window.dialogApi.openFolder()
    if (path) {
      await dispatch({ type: 'OpenProject', payload: { path } })
    }
  }

  const handleSwitchProject = (_: any, index: number) => {
    dispatch({ type: 'SwitchProject', payload: { index } })
  }

  const handleCloseProject = (e: React.MouseEvent, index: number) => {
    e.stopPropagation()
    dispatch({ type: 'CloseProject', payload: { index } })
  }

  const handleSwitchWorktree = (_: any, index: number) => {
    dispatch({ type: 'SwitchWorktree', payload: { index } })
  }

  const handleOpenRecentProject = async (path: string) => {
    setProjectMenuAnchor(null)
    await dispatch({ type: 'OpenProject', payload: { path } })
  }

  const handleAddWorktreeFromBranch = useCallback(async (branch: string) => {
    await dispatch({ type: 'AddWorktree', payload: { branch } })
  }, [dispatch])

  const handleAddWorktreeNewBranch = useCallback(async (branch: string) => {
    await dispatch({ type: 'AddWorktreeNewBranch', payload: { branch } })
  }, [dispatch])

  const openProjectPaths = new Set(projects.map(p => p.path))
  const filteredRecentProjects = recentProjects.filter(r => !openProjectPaths.has(r.path))
  const activeView = state?.active_view ?? 'tasks'

  const handleDockerClick = useCallback(async () => {
    await dispatch({ type: 'SetActiveView', payload: { view: 'dockers' } })
  }, [dispatch])

  const handleScreenshot = useCallback(async () => {
    try {
      const result = await window.screenshotApi.capture()
      if (result.success && result.filePath) {
        await dispatch({
          type: 'AddNotification',
          payload: { message: `Screenshot saved: ${result.filePath}`, notification_type: 'success' },
        })
      }
    } catch (error) {
      console.error('Screenshot error:', error)
    }
  }, [dispatch])

  const handleEnvClick = useCallback(async () => {
    await dispatch({ type: 'SetActiveView', payload: { view: 'env' } })
  }, [dispatch])

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', bgcolor: 'surfaceContainerLow.main', borderBottom: 1, borderColor: 'outlineVariant' }}>
      {/* Project Tabs (Top Row) */}
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ px: 1, height: 48 }}>
        <Stack direction="row" alignItems="center" spacing={0.5} sx={{ flex: 1, minWidth: 0 }}>
          {projects.length === 0 ? (
            <Button
              variant="text"
              size="small"
              onClick={handleOpenProject}
              startIcon={<FolderOpenIcon />}
              sx={{ color: 'text.secondary', textTransform: 'none' }}
            >
              Open Project
            </Button>
          ) : (
            <>
              <Tabs
                value={activeIndex}
                onChange={handleSwitchProject}
                variant="scrollable"
                scrollButtons="auto"
                sx={{
                  minHeight: 40,
                  '& .MuiTabs-indicator': { height: 3, borderRadius: '3px 3px 0 0' }
                }}
              >
                {projects.map((project, index) => (
                  <Tab
                    key={project.id}
                    label={
                      <Stack direction="row" alignItems="center" spacing={1}>
                        <Typography variant="caption" fontWeight={600} noWrap sx={{ maxWidth: 120 }}>
                          {project.worktrees.some(wt => wt.is_modified) && (
                            <Box component="span" sx={{ color: 'warning.main', mr: 0.5 }}>*</Box>
                          )}
                          {project.name}
                        </Typography>
                        <IconButton
                          size="small"
                          onClick={(e) => handleCloseProject(e, index)}
                          sx={{ p: 0.25, '&:hover': { bgcolor: 'error.container', color: 'error.main' } }}
                        >
                          <XIcon sx={{ fontSize: 12 }} />
                        </IconButton>
                      </Stack>
                    }
                    sx={{ minHeight: 40, py: 0, textTransform: 'none', px: 2 }}
                  />
                ))}
              </Tabs>
              <IconButton size="small" onClick={(e) => setProjectMenuAnchor(e.currentTarget)} sx={{ ml: 0.5 }}>
                <PlusIcon fontSize="small" />
              </IconButton>
              <Menu
                anchorEl={projectMenuAnchor}
                open={Boolean(projectMenuAnchor)}
                onClose={() => setProjectMenuAnchor(null)}
              >
                <MenuItem onClick={handleOpenProject}>
                  <ListItemIcon><FolderOpenIcon fontSize="small" /></ListItemIcon>
                  <ListItemText primary="Open Project..." />
                </MenuItem>
                {filteredRecentProjects.length > 0 && [
                  <Divider key="divider" />,
                  <Box key="label" sx={{ px: 2, py: 1 }}><Typography variant="caption" fontWeight={700} color="text.secondary">RECENT</Typography></Box>,
                  ...filteredRecentProjects.slice(0, 5).map((recent) => (
                    <MenuItem key={recent.path} onClick={() => handleOpenRecentProject(recent.path)}>
                      <ListItemIcon><HistoryIcon fontSize="small" /></ListItemIcon>
                      <ListItemText primary={recent.path.split('/').pop()} secondary={recent.path} secondaryTypographyProps={{ variant: 'caption', noWrap: true, sx: { maxWidth: 200 } }} />
                    </MenuItem>
                  ))
                ]}
              </Menu>
            </>
          )}
        </Stack>

        <Stack direction="row" alignItems="center" spacing={1} sx={{ pl: 2 }}>
          {import.meta.env.DEV && (
            <Tooltip title="Capture Screenshot">
              <IconButton size="small" onClick={handleScreenshot}><CameraIcon fontSize="small" /></IconButton>
            </Tooltip>
          )}
          <Button
            variant={activeView === 'dockers' ? 'contained' : 'text'}
            color={activeView === 'dockers' ? 'secondary' : 'inherit'}
            size="small"
            onClick={handleDockerClick}
            startIcon={<ContainerIcon />}
            sx={{ height: 32, borderRadius: 2, textTransform: 'none', fontSize: '0.75rem' }}
          >
            Docker
          </Button>
          <NotificationDrawer />
        </Stack>
      </Stack>

      {/* Worktree Sub-Tabs (Second Row) */}
      {worktrees.length > 0 && (
        <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ px: 1.5, height: 36, bgcolor: alpha('#000', 0.1) }}>
          <Stack direction="row" alignItems="center" spacing={1} sx={{ flex: 1, minWidth: 0 }}>
            <GitBranchIcon sx={{ fontSize: 14, color: 'text.secondary' }} />
            <Tabs
              value={activeWorktreeIndex}
              onChange={handleSwitchWorktree}
              variant="scrollable"
              sx={{
                minHeight: 36,
                '& .MuiTabs-indicator': { display: 'none' },
                '& .Mui-selected': { bgcolor: 'action.selected', borderRadius: 1 }
              }}
            >
              {worktrees.map((wt, index) => (
                <Tab
                  key={wt.id}
                  label={
                    <Stack direction="row" alignItems="center" spacing={0.5}>
                      {wt.is_modified && <Box component="span" sx={{ color: 'warning.main' }}>*</Box>}
                      <Typography variant="caption" fontWeight={activeIndex === index ? 700 : 500}>
                        {wt.branch}
                      </Typography>
                      {wt.is_main && <Typography variant="caption" sx={{ opacity: 0.5, fontSize: '0.6rem' }}>(main)</Typography>}
                    </Stack>
                  }
                  sx={{ minHeight: 32, height: 32, px: 1.5, textTransform: 'none', minWidth: 0 }}
                />
              ))}
            </Tabs>
            <IconButton size="small" onClick={() => setAddWorktreeDialogOpen(true)} sx={{ width: 24, height: 24 }}>
              <PlusIcon sx={{ fontSize: 14 }} />
            </IconButton>
          </Stack>

          <Button
            variant={activeView === 'env' ? 'contained' : 'text'}
            color={activeView === 'env' ? 'secondary' : 'inherit'}
            size="small"
            onClick={handleEnvClick}
            startIcon={<FileCodeIcon sx={{ fontSize: 14 }} />}
            sx={{ height: 24, borderRadius: 1, textTransform: 'none', fontSize: '0.7rem', px: 1 }}
          >
            Env
          </Button>
        </Stack>
      )}

      {activeProject && (
        <AddWorktreeDialog
          open={addWorktreeDialogOpen}
          onOpenChange={setAddWorktreeDialogOpen}
          repoPath={activeProject.path}
          onAddFromBranch={handleAddWorktreeFromBranch}
          onAddNewBranch={handleAddWorktreeNewBranch}
        />
      )}
    </Box>
  )
}
