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
  Tooltip,
  useTheme
} from '@mui/material'
import { useAppState } from '@/hooks/useAppState'
import type { ActiveView } from '@/types/state'

const NAV_ITEMS = [
  { value: 'explorer', label: 'Explorer', icon: <ExplorerIcon /> },
  { value: 'workflows', label: 'Flows', icon: <WorkflowIcon /> },
  { value: 'claude-code', label: 'Claude', icon: <ClaudeIcon /> },
  { value: 'tasks', label: 'Tasks', icon: <TasksIcon /> },
  { value: 'mcp', label: 'rstn', icon: <ServerIcon /> },
  { value: 'chat', label: 'Chat', icon: <ChatIcon /> },
  { value: 'a2ui', label: 'A2UI', icon: <A2UIIcon /> },
  { value: 'terminal', label: 'Term', icon: <TerminalIcon /> },
] as const

const BOTTOM_ITEMS = [
  { value: 'settings', label: 'Settings', icon: <SettingsIcon /> },
] as const

export function Sidebar() {
  const { state, dispatch } = useAppState()
  const activeView = state?.active_view ?? 'tasks'
  const theme = useTheme()

  const handleNavClick = useCallback(
    (view: string) => {
      dispatch({ type: 'SetActiveView', payload: { view: view as ActiveView } })
    },
    [dispatch]
  )

  const renderNavItem = (item: { value: string; label: string; icon: React.ReactNode }) => {
    const isSelected = activeView === item.value

    return (
      <ListItem key={item.value} disablePadding sx={{ display: 'block', mb: 1.5 }}>
        <Tooltip title={item.label} placement="right">
          <ListItemButton
            selected={isSelected}
            onClick={() => handleNavClick(item.value)}
            sx={{
              minHeight: 56,
              justifyContent: 'center',
              px: 1,
              flexDirection: 'column',
              backgroundColor: 'transparent !important', // Handle selection via inner box
              gap: 0.5,
            }}
          >
            {/* Active Indicator Pill */}
            <Box
              sx={{
                width: 56,
                height: 32,
                borderRadius: 4, // 16px (Pill shape)
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                bgcolor: isSelected ? 'secondary.container' : 'transparent',
                color: isSelected ? 'onSecondaryContainer' : 'onSurfaceVariant',
                transition: theme.transitions.create(['background-color', 'color']),
                '& .MuiSvgIcon-root': {
                  fontSize: 24,
                  color: 'inherit',
                },
              }}
            >
              {item.icon}
            </Box>

            {/* Label */}
            <ListItemText
              primary={item.label}
              primaryTypographyProps={{
                variant: 'caption',
                sx: {
                  fontSize: '0.75rem', // 12px
                  fontWeight: 500,
                  textAlign: 'center',
                  color: 'onSurfaceVariant',
                }
              }}
              sx={{ m: 0 }}
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
        width: 80, // M3 Navigation Rail width
        flexShrink: 0,
        bgcolor: 'surface.main', // Should be Surface
        borderRight: 1,
        borderColor: 'outlineVariant',
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        overflowY: 'auto',
        overflowX: 'hidden',
        pt: 2,
      }}
    >
      <List>
        {NAV_ITEMS.map(renderNavItem)}
      </List>

      <Box sx={{ flexGrow: 1 }} />

      <Divider sx={{ mx: 2, my: 1 }} />
      <List>
        {BOTTOM_ITEMS.map(renderNavItem)}
      </List>
    </Box>
  )
}
