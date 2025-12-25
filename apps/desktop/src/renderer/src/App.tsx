import { useCallback } from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ListTodo, Container, Settings, RefreshCw, FolderOpen } from 'lucide-react'
import { DockersPage } from '@/features/dockers/DockersPage'
import { TasksPage } from '@/features/tasks/TasksPage'
import { useActiveProject, useAppState } from '@/hooks/useAppState'
import { ProjectTabs } from '@/components/ProjectTabs'
import { Button } from '@/components/ui/button'
import type { FeatureTab } from '@/types/state'

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

function App(): JSX.Element {
  const { state, isLoading } = useAppState()
  const { project, dispatch } = useActiveProject()

  const activeTab = project?.active_tab ?? 'tasks'

  const handleTabChange = useCallback(
    (tab: string) => {
      dispatch({ type: 'SetFeatureTab', payload: { tab: tab as FeatureTab } })
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

  return (
    <div className="flex h-screen flex-col bg-background">
      {/* Project Tabs (Top) */}
      <ProjectTabs />

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {project ? (
          /* Sidebar + Content when project is open */
          <Tabs
            value={activeTab}
            onValueChange={handleTabChange}
            orientation="vertical"
            className="flex h-full w-full"
          >
            <TabsList className="flex h-full w-16 flex-col items-center gap-2 rounded-none border-r bg-muted/40 p-2">
              <TabsTrigger
                value="tasks"
                className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <ListTodo className="h-5 w-5" />
                <span className="text-[10px]">Tasks</span>
              </TabsTrigger>
              <TabsTrigger
                value="dockers"
                className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <Container className="h-5 w-5" />
                <span className="text-[10px]">Docker</span>
              </TabsTrigger>
              <TabsTrigger
                value="settings"
                className="mt-auto flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
              >
                <Settings className="h-5 w-5" />
                <span className="text-[10px]">Settings</span>
              </TabsTrigger>
            </TabsList>

            {/* Main Content */}
            <div className="flex-1 overflow-auto p-6">
              <TabsContent value="tasks" className="m-0 h-full">
                <TasksPage />
              </TabsContent>
              <TabsContent value="dockers" className="m-0 h-full">
                <DockersPage />
              </TabsContent>
              <TabsContent value="settings" className="m-0 h-full">
                <div className="flex h-full items-center justify-center text-muted-foreground">
                  Settings - Coming Soon
                </div>
              </TabsContent>
            </div>
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
