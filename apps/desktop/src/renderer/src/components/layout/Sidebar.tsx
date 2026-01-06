import { useCallback } from 'react'
import { 
  ListAlt as TasksIcon, 
  Settings as SettingsIcon, 
  Storage as ServerIcon, 
  Chat as ChatIcon, 
  Terminal as TerminalIcon, 
  AccountTree as WorkflowIcon,
  SmartToy as ClaudeIcon,
  Code as A2UIIcon,
  FolderOpen as ExplorerIcon
} from '@mui/icons-material'
import { 
  Box, 
  List, 
  ListItem, 
  ListItemButton, 
  ListItemIcon, 
  ListItemText, 
  Divider,
  Tooltip 
} from '@mui/material'
import { useAppState } from '@/hooks/useAppState'
import type { ActiveView } from '@/types/state'

const NAV_ITEMS = [
  { value: 'explorer', label: 'Explorer', icon: <ExplorerIcon /> },
  { value: 'workflows', label: 'Flows', icon: <WorkflowIcon /> },
  { value: 'claude-code', label: 'Claude', icon: <ClaudeIcon color="secondary" /> },
  { value: 'tasks', label: 'Tasks', icon: <TasksIcon /> },
  { value: 'mcp', label: 'rstn', icon: <ServerIcon /> },
  { value: 'chat', label: 'Chat', icon: <ChatIcon /> },
  { value: 'a2ui', label: 'A2UI', icon: <A2UIIcon color="secondary" /> },
  { value: 'terminal', label: 'Term', icon: <TerminalIcon /> },
] as const

const BOTTOM_ITEMS = [
  { value: 'settings', label: 'Settings', icon: <SettingsIcon /> },
] as const

export function Sidebar() {
  const { state, dispatch } = useAppState()
  const activeView = state?.active_view ?? 'tasks'

  const handleNavClick = useCallback(
    (view: string) => {
      dispatch({ type: 'SetActiveView', payload: { view: view as ActiveView } })
    },
    [dispatch]
  )

  const renderNavItem = (item: { value: string; label: string; icon: React.ReactNode }) => {
    const isSelected = activeView === item.value
    
    return (
      <ListItem key={item.value} disablePadding sx={{ display: 'block' }}>
        <Tooltip title={item.label} placement="right">
          <ListItemButton
            selected={isSelected}
            onClick={() => handleNavClick(item.value)}
            sx={{
              minHeight: 56,
              justifyContent: 'center',
              px: 2.5,
              flexDirection: 'column',
              gap: 0.5,
              '&.Mui-selected': {
                color: 'primary.main',
                '& .MuiListItemIcon-root': {
                  color: 'primary.main',
                },
              },
            }}
          >
            <ListItemIcon
              sx={{
                minWidth: 0,
                mr: 0,
                justifyContent: 'center',
                color: isSelected ? 'primary.main' : 'text.secondary',
              }}
            >
              {item.icon}
            </ListItemIcon>
            <ListItemText 
              primary={item.label} 
              primaryTypographyProps={{ 
                variant: 'caption', 
                sx: { fontSize: '0.65rem', fontWeight: isSelected ? 600 : 400 } 
              }} 
            />
          </ListItemButton>
        </Tooltip>
      </ListItem>
    )
  }

  return (
    <Box
      component="nav"
      sx={{
        width: 72,
        flexShrink: 0,
        bgcolor: 'background.paper',
        borderRight: 1,
        borderColor: 'divider',
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        overflowY: 'auto',
        overflowX: 'hidden',
      }}
    >
      <List sx={{ pt: 1 }}>
        {NAV_ITEMS.map(renderNavItem)}
      </List>
      
      <Box sx={{ flexGrow: 1 }} />
      
      <Divider />
      <List>
        {BOTTOM_ITEMS.map(renderNavItem)}
      </List>
    </Box>
  )
}
