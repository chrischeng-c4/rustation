import { useCallback, useEffect, useState } from 'react'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ListTodo, Settings, RefreshCw, FolderOpen, Server, MessageSquare, TerminalSquare, Workflow } from 'lucide-react'
import { DockersPage } from '@/features/dockers/DockersPage'
import { TasksPage } from '@/features/tasks/TasksPage'
import { EnvPage } from '@/features/env'
import { AgentRulesPage } from '@/features/agent-rules'
import { SettingsPage } from '@/features/settings'
import { McpPage } from '@/features/mcp'
import { ChatPage } from '@/features/chat'
import { TerminalPage } from '@/features/terminal'
import { WorkflowsPage } from '@/features/workflows'
import { Toaster } from '@/features/notifications'
import { CommandPalette } from '@/components/command-palette'
import { DevLogPanel } from '@/components/DevLogPanel'
import { useActiveWorktree, useAppState } from '@/hooks/useAppState'
import { ProjectTabs } from '@/components/ProjectTabs'
import { Button } from '@/components/ui/button'
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
    <div className="flex h-full flex-col items-center justify-center gap-4 text-muted-foreground">
      <FolderOpen className="h-16 w-16" />
      <h2 className="text-xl font-medium">No Project Open</h2>
      <p className="text-sm">Open a project folder to get started</p>
      <Button onClick={handleOpenProject} className="mt-4">
        <FolderOpen className="mr-2 h-4 w-4" />
        Open Project
      </Button>
    </div>
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
    (view: string) => {
      dispatch({ type: 'SetActiveView', payload: { view: view as ActiveView } })
    },
    [dispatch]
  )

  // Show loading while state is initializing
  if (isLoading || !state) {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
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
      case 'agent_rules':
        return <AgentRulesPage />
      case 'mcp':
        return <McpPage />
      case 'chat':
        return <ChatPage />
      case 'terminal':
        return <TerminalPage />
      case 'workflows':
        return <WorkflowsPage />
      default:
        return <TasksPage />
    }
  }

  // Determine if sidebar items should be highlighted
  // Only highlight for worktree-scope views (tasks, settings, mcp, chat, terminal)
  const getSidebarValue = () => {
    if (activeView === 'tasks' || activeView === 'settings' || activeView === 'mcp' || activeView === 'chat' || activeView === 'terminal' || activeView === 'workflows') {
      return activeView
    }
    // For global/project scope views (dockers, env), don't highlight sidebar
    return ''
  }

  return (
    <div className="flex h-screen flex-col bg-background">
      {/* Command Palette (Cmd+K / Ctrl+K) */}
      <CommandPalette open={commandPaletteOpen} onOpenChange={setCommandPaletteOpen} />

      {/* Toast Notifications (fixed overlay) */}
      <Toaster />

      {/* Project Tabs (Top) */}
      <ProjectTabs />

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Global scope views (Docker) work without a project */}
        {activeView === 'dockers' ? (
          <div className="flex-1 overflow-auto p-6">
            <DockersPage />
          </div>
        ) : worktree ? (
          /* Sidebar + Content when project is open */
          <Tabs
            value={getSidebarValue()}
            onValueChange={handleSidebarChange}
            orientation="vertical"
            className="flex h-full w-full"
          >
            {/* Sidebar: Only worktree-scope features */}
            <TabsList className="flex h-full w-16 flex-col items-center gap-2 rounded-none border-r bg-muted/40 p-2">
              <TabsTrigger
                value="workflows"
                className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <Workflow className="h-5 w-5" />
                <span className="text-[10px]">Flows</span>
              </TabsTrigger>
              <TabsTrigger
                value="tasks"
                className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <ListTodo className="h-5 w-5" />
                <span className="text-[10px]">Tasks</span>
              </TabsTrigger>
              <TabsTrigger
                value="mcp"
                className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <Server className="h-5 w-5" />
                <span className="text-[10px]">rstn</span>
              </TabsTrigger>
              <TabsTrigger
                value="chat"
                className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <MessageSquare className="h-5 w-5" />
                <span className="text-[10px]">Chat</span>
              </TabsTrigger>
              <TabsTrigger
                value="terminal"
                className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <TerminalSquare className="h-5 w-5" />
                <span className="text-[10px]">Term</span>
              </TabsTrigger>
              <TabsTrigger
                value="settings"
                className="mt-auto flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <Settings className="h-5 w-5" />
                <span className="text-[10px]">Settings</span>
              </TabsTrigger>
            </TabsList>

            {/* Main Content - renders based on activeView */}
            <div className="flex-1 overflow-auto p-6">{renderContent()}</div>
          </Tabs>
        ) : (
          /* No project open - show NoProjectView for worktree-scope views */
          <NoProjectView />
        )}

        {/* Dev Log Panel (right side, dev mode only) */}
        {IS_DEV && <DevLogPanel />}
      </div>
    </div>
  )
}

export default App
