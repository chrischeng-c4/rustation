import { SyntheticEvent, useCallback, useEffect, useState } from 'react'
import { Box, Button, CircularProgress, Stack, Tab, Tabs, Typography } from '@mui/material'
import {
  AccountTree,
  Chat,
  Code,
  FolderOpen,
  ListAlt,
  Psychology,
  Settings,
  Storage,
} from '@mui/icons-material'
import { DockersPage } from '@/features/dockers/DockersPage'
import { TasksPage } from '@/features/tasks/TasksPage'
import { EnvPage } from '@/features/env'
import { SettingsPage } from '@/features/settings'
import { McpPage } from '@/features/mcp'
import { ChatPage } from '@/features/chat'
import { TerminalPage } from '@/features/terminal'
import { WorkflowsPage } from '@/features/workflows'
import { ClaudeCodePage } from '@/features/claude-code/ClaudeCodePage'
import { A2UIPage } from '@/features/a2ui/A2UIPage'
import { Toaster } from '@/features/notifications'
import { CommandPalette } from '@/features/command-palette'
import { DevLogPanel } from '@/components/shared/DevLogPanel'
import { RightIconBar } from '@/components/layout/RightIconBar'
import { LogPanel } from '@/components/layout/LogPanel'
import { useActiveWorktree, useAppState } from '@/hooks/useAppState'
import { ProjectTabs } from '@/features/projects/components/ProjectTabs'
import type { ActiveView } from '@/types/state'

// Dev mode check - only show DevLogPanel in development
const IS_DEV = import.meta.env.DEV

function NoProjectView() {
  const { dispatch } = useAppState()

  const handleOpenProject = async () => {
    const path = await window.dialogApi.openFolder()
    if (path) {
      await dispatch({ type: 'OpenProject', payload: { path } })
    }
  }

  return (
    <Stack
      sx={{
        height: '100%',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 2,
        color: 'text.secondary',
      }}
    >
      <FolderOpen sx={{ fontSize: 64 }} />
      <Typography variant="h6" fontWeight={600}>
        No Project Open
      </Typography>
      <Typography variant="body2">Open a project folder to get started</Typography>
      <Button variant="contained" onClick={handleOpenProject} startIcon={<FolderOpen />}>
        Open Project
      </Button>
    </Stack>
  )
}

function App() {
  const { state, isLoading, dispatch } = useAppState()
  const { worktree } = useActiveWorktree()
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)

  // Global keyboard shortcut: Cmd+K / Ctrl+K
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        setCommandPaletteOpen((open) => !open)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  // Use global active_view from state
  const activeView = state?.active_view ?? 'tasks'

  const handleSidebarChange = useCallback(
    (_event: SyntheticEvent, view: string) => {
      dispatch({ type: 'SetActiveView', payload: { view: view as ActiveView } })
    },
    [dispatch]
  )

  // Show loading while state is initializing
  if (isLoading || !state) {
    return (
      <Stack sx={{ height: '100vh', alignItems: 'center', justifyContent: 'center' }}>
        <CircularProgress size={32} />
      </Stack>
    )
  }

  // Render content based on active view
  const renderContent = () => {
    switch (activeView) {
      case 'tasks':
        return <TasksPage />
      case 'settings':
        return <SettingsPage />
      case 'dockers':
        return <DockersPage />
      case 'env':
        return <EnvPage />
      case 'mcp':
        return <McpPage />
      case 'chat':
        return <ChatPage />
      case 'terminal':
        return <TerminalPage />
      case 'workflows':
        return <WorkflowsPage />
      case 'claude-code':
        return <ClaudeCodePage />
      case 'a2ui':
        return <A2UIPage />
      default:
        return <TasksPage />
    }
  }

  // Determine if sidebar items should be highlighted
  // Only highlight for worktree-scope views (tasks, settings, mcp, chat, terminal, a2ui)
  const getSidebarValue = () => {
    if (activeView === 'tasks' || activeView === 'settings' || activeView === 'mcp' || activeView === 'chat' || activeView === 'terminal' || activeView === 'workflows' || activeView === 'claude-code' || activeView === 'a2ui') {
      return activeView
    }
    // For global/project scope views (dockers, env), don't highlight sidebar
    return ''
  }

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100vh', bgcolor: 'background.default' }}>
      {/* Command Palette (Cmd+K / Ctrl+K) */}
      <CommandPalette open={commandPaletteOpen} onOpenChange={setCommandPaletteOpen} />

      {/* Toast Notifications (fixed overlay) */}
      <Toaster />

      {/* Project Tabs (Top) */}
      <ProjectTabs />

      {/* Main Content */}
      <Box sx={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Global scope views (Docker) work without a project */}
        {activeView === 'dockers' ? (
          <Box sx={{ flex: 1, overflow: 'auto', p: 3 }}>
            <DockersPage />
          </Box>
        ) : worktree ? (
          /* Sidebar + Content when project is open */
          <Box sx={{ display: 'flex', width: '100%', height: '100%' }}>
            <Tabs
              orientation="vertical"
              value={getSidebarValue()}
              onChange={handleSidebarChange}
              variant="scrollable"
              sx={{
                minWidth: 72,
                borderRight: 1,
                borderColor: 'divider',
                bgcolor: 'background.paper',
                py: 1,
              }}
            >
              <Tab
                value="workflows"
                icon={<AccountTree fontSize="small" />}
                iconPosition="top"
                label="Flows"
                sx={{ minHeight: 64, minWidth: 64, fontSize: '0.65rem' }}
              />
              <Tab
                value="claude-code"
                icon={<Psychology fontSize="small" />}
                iconPosition="top"
                label="Claude"
                sx={{ minHeight: 64, minWidth: 64, fontSize: '0.65rem' }}
              />
              <Tab
                value="tasks"
                icon={<ListAlt fontSize="small" />}
                iconPosition="top"
                label="Tasks"
                sx={{ minHeight: 64, minWidth: 64, fontSize: '0.65rem' }}
              />
              <Tab
                value="mcp"
                icon={<Storage fontSize="small" />}
                iconPosition="top"
                label="rstn"
                sx={{ minHeight: 64, minWidth: 64, fontSize: '0.65rem' }}
              />
              <Tab
                value="chat"
                icon={<Chat fontSize="small" />}
                iconPosition="top"
                label="Chat"
                sx={{ minHeight: 64, minWidth: 64, fontSize: '0.65rem' }}
              />
              <Tab
                value="a2ui"
                icon={<Code fontSize="small" />}
                iconPosition="top"
                label="A2UI"
                sx={{ minHeight: 64, minWidth: 64, fontSize: '0.65rem' }}
              />
              <Tab
                value="terminal"
                icon={<Code fontSize="small" />}
                iconPosition="top"
                label="Term"
                sx={{ minHeight: 64, minWidth: 64, fontSize: '0.65rem' }}
              />
              <Tab
                value="settings"
                icon={<Settings fontSize="small" />}
                iconPosition="top"
                label="Settings"
                sx={{ minHeight: 64, minWidth: 64, fontSize: '0.65rem', mt: 'auto' }}
              />
            </Tabs>

            <Box sx={{ flex: 1, overflow: 'auto', p: 3 }}>{renderContent()}</Box>
          </Box>
        ) : (
          /* No project open - show NoProjectView for worktree-scope views */
          <NoProjectView />
        )}

        {/* Right Icon Bar & Log Panel */}
        <LogPanel />
        <RightIconBar />

        {/* Old Dev Log Panel (fallback, dev mode only) - can be removed after testing */}
        {/* {IS_DEV && <DevLogPanel />} */}
      </Box>
    </Box>
  )
}

export default App
