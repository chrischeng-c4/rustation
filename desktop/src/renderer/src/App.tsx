import { SyntheticEvent, useCallback, useEffect, useState } from 'react'
import { Box, Button, CircularProgress, Stack, Typography } from '@mui/material'
import {
  FolderOpen,
} from '@mui/icons-material'
import { useAppState, useActiveWorktree } from '@/hooks/useAppState'
import type { ActiveView } from '@/types/state'
import { DockersPage } from '@/features/dockers/DockersPage'
import { TasksPage } from '@/features/tasks/TasksPage'
import { ExplorerPage } from '@/features/explorer/ExplorerPage'
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
import { Sidebar } from '@/components/layout/Sidebar'
import { ProjectTabs } from '@/features/projects/ProjectTabs'
import { WorktreeTabs } from '@/features/worktrees'

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
      case 'explorer':
        return <ExplorerPage />
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

      {/* Level 1: Project Tabs (includes GlobalIconBar) */}
      <ProjectTabs />

      {/* Level 2: Worktree Tabs (only shown when project is active) */}
      <WorktreeTabs />

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
            <Sidebar />

            <Box sx={{ flex: 1, overflow: 'auto', p: 3 }}>{renderContent()}</Box>
          </Box>
        ) : (
          /* No project open - show NoProjectView for worktree-scope views */
          <NoProjectView />
        )}

      </Box>
    </Box>
  )
}

export default App
