import { useCallback } from 'react'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ListTodo, Settings, RefreshCw, FolderOpen, FileCode } from 'lucide-react'
import { DockersPage } from '@/features/dockers/DockersPage'
import { TasksPage } from '@/features/tasks/TasksPage'
import { useActiveWorktree, useAppState } from '@/hooks/useAppState'
import { ProjectTabs } from '@/components/ProjectTabs'
import { Button } from '@/components/ui/button'
import type { ActiveView } from '@/types/state'

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

/**
 * Placeholder EnvPage component.
 * TODO: Move to features/env/EnvPage.tsx
 */
function EnvPagePlaceholder() {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 text-muted-foreground">
      <FileCode className="h-16 w-16" />
      <h2 className="text-xl font-medium">Environment Files</h2>
      <p className="text-sm">Manage dotfiles across worktrees</p>
      <p className="text-xs text-muted-foreground/60">Coming Soon</p>
    </div>
  )
}

function App() {
  const { state, isLoading, dispatch } = useAppState()
  const { worktree } = useActiveWorktree()

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
        return (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            Settings - Coming Soon
          </div>
        )
      case 'dockers':
        return <DockersPage />
      case 'env':
        return <EnvPagePlaceholder />
      default:
        return <TasksPage />
    }
  }

  // Determine if sidebar items should be highlighted
  // Only highlight for worktree-scope views (tasks, settings)
  const getSidebarValue = () => {
    if (activeView === 'tasks' || activeView === 'settings') {
      return activeView
    }
    // For global/project scope views (dockers, env), don't highlight sidebar
    return ''
  }

  return (
    <div className="flex h-screen flex-col bg-background">
      {/* Project Tabs (Top) */}
      <ProjectTabs />

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {worktree ? (
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
                value="tasks"
                className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <ListTodo className="h-5 w-5" />
                <span className="text-[10px]">Tasks</span>
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
          /* No project open */
          <NoProjectView />
        )}
      </div>
    </div>
  )
}

export default App
