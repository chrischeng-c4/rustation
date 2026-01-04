/**
 * Project tabs component - displays open projects and worktrees at the top of the window.
 *
 * Two-level tab structure with scope-based features (Three-Scope Model):
 * - Top row: Project tabs (git repos) + Docker button (Global scope)
 * - Second row: Worktree sub-tabs (git worktrees) + Env button (Project scope)
 */

import { useState, useCallback } from 'react'
import { X, Plus, FolderOpen, GitBranch, History, Container, FileCode, Camera, Bot } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useActiveProject, useActiveWorktree, useAppState } from '@/hooks/useAppState'
import { NotificationDrawer } from '@/features/notifications'
import { cn } from '@/lib/utils'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { AddWorktreeDialog } from './AddWorktreeDialog'

export function ProjectTabs() {
  const { state } = useAppState()
  const { projects, activeIndex, dispatch } = useActiveProject()
  const { worktrees, activeWorktreeIndex, worktree } = useActiveWorktree()
  const [addWorktreeDialogOpen, setAddWorktreeDialogOpen] = useState(false)

  const recentProjects = state?.recent_projects ?? []
  const activeProject = projects[activeIndex]

  const handleOpenProject = async () => {
    const path = await window.dialogApi.openFolder()
    if (path) {
      await dispatch({ type: 'OpenProject', payload: { path } })
    }
  }

  const handleSwitchProject = async (index: number) => {
    await dispatch({ type: 'SwitchProject', payload: { index } })
  }

  const handleCloseProject = async (e: React.MouseEvent, index: number) => {
    e.stopPropagation()
    await dispatch({ type: 'CloseProject', payload: { index } })
  }

  const handleSwitchWorktree = async (index: number) => {
    await dispatch({ type: 'SwitchWorktree', payload: { index } })
  }

  const handleOpenRecentProject = async (path: string) => {
    await dispatch({ type: 'OpenProject', payload: { path } })
  }

  const handleAddWorktreeFromBranch = useCallback(async (branch: string) => {
    await dispatch({ type: 'AddWorktree', payload: { branch } })
  }, [dispatch])

  const handleAddWorktreeNewBranch = useCallback(async (branch: string) => {
    await dispatch({ type: 'AddWorktreeNewBranch', payload: { branch } })
  }, [dispatch])

  // Check if worktree has unsaved changes
  const getWorktreeModified = (wt: typeof worktree) => {
    return wt?.is_modified ?? false
  }

  // Filter out already open projects from recent list
  const openProjectPaths = new Set(projects.map(p => p.path))
  const filteredRecentProjects = recentProjects.filter(r => !openProjectPaths.has(r.path))

  // Get active view from state
  const activeView = state?.active_view ?? 'tasks'

  const handleDockerClick = useCallback(async () => {
    await dispatch({ type: 'SetActiveView', payload: { view: 'dockers' } })
  }, [dispatch])

  const handleScreenshot = useCallback(async () => {
    console.log('Screenshot button clicked!')
    try {
      const result = await window.screenshotApi.capture()
      console.log('Screenshot result:', result)

      if (result.success && result.filePath) {
        // Show success notification
        await dispatch({
          type: 'AddNotification',
          payload: {
            message: `Screenshot saved and copied to clipboard: ${result.filePath}`,
            notification_type: 'success',
          },
        })
      } else {
        // Show error notification
        await dispatch({
          type: 'AddNotification',
          payload: {
            message: `Screenshot failed: ${result.error || 'Unknown error'}`,
            notification_type: 'error',
          },
        })
      }
    } catch (error) {
      console.error('Screenshot error:', error)
      await dispatch({
        type: 'AddNotification',
        payload: {
          message: `Screenshot error: ${error}`,
          notification_type: 'error',
        },
      })
    }
  }, [dispatch])

  const handleEnvClick = useCallback(async () => {
    await dispatch({ type: 'SetActiveView', payload: { view: 'env' } })
  }, [dispatch])

  const handleAgentRulesClick = useCallback(async () => {
    await dispatch({ type: 'SetActiveView', payload: { view: 'agent_rules' } })
  }, [dispatch])

  return (
    <div className="flex flex-col border-b bg-muted/30" data-testid="project-tabs">
      {/* Project Tabs (Top Row) - Global Features on Right */}
      <div className="flex items-center justify-between px-2 py-1 min-h-[40px]">
        {/* Left side: Project tabs */}
        <div className="flex items-center gap-1">
          {projects.length === 0 ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleOpenProject}
              className="gap-2 text-muted-foreground"
            >
              <FolderOpen className="h-4 w-4" />
              Open Project
            </Button>
          ) : (
            <>
              {projects.map((project, index) => {
                // Check if any worktree in this project is modified
                const hasModifiedWorktree = project.worktrees.some(wt => wt.is_modified)

                return (
                  <div
                    key={project.id}
                    onClick={() => handleSwitchProject(index)}
                    className={cn(
                      'flex items-center gap-2 px-3 py-1.5 rounded-md cursor-pointer transition-colors',
                      'hover:bg-accent group',
                      index === activeIndex
                        ? 'bg-background border shadow-sm'
                        : 'text-muted-foreground'
                    )}
                  >
                    <span className="text-sm truncate max-w-[120px]">
                      {hasModifiedWorktree && <span className="text-yellow-500 mr-1">*</span>}
                      {project.name}
                    </span>
                    <button
                      onClick={(e) => handleCloseProject(e, index)}
                      className="opacity-0 group-hover:opacity-100 hover:bg-destructive/20 rounded p-0.5 transition-opacity"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  </div>
                )
              })}
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 text-muted-foreground"
                  >
                    <Plus className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start">
                  <DropdownMenuItem onClick={handleOpenProject}>
                    <FolderOpen className="h-4 w-4 mr-2" />
                    Open Project...
                  </DropdownMenuItem>
                  {filteredRecentProjects.length > 0 && (
                    <>
                      <DropdownMenuSeparator />
                      <DropdownMenuLabel className="flex items-center gap-2">
                        <History className="h-3 w-3" />
                        Recent Projects
                      </DropdownMenuLabel>
                      {filteredRecentProjects.slice(0, 5).map((recent) => (
                        <DropdownMenuItem
                          key={recent.path}
                          onClick={() => handleOpenRecentProject(recent.path)}
                        >
                          <span className="truncate max-w-[200px]">
                            {recent.path.split('/').pop()}
                          </span>
                        </DropdownMenuItem>
                      ))}
                    </>
                  )}
                </DropdownMenuContent>
              </DropdownMenu>
            </>
          )}
        </div>

        {/* Right side: Global features (Screenshot, Docker, Notifications) */}
        <div className="flex items-center gap-1">
          {/* Screenshot button (dev mode only) */}
          {import.meta.env.DEV && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleScreenshot}
              className="gap-1.5 text-sm h-7"
              title="Capture screenshot"
            >
              <Camera className="h-3.5 w-3.5" />
              Screenshot
            </Button>
          )}

          <Button
            variant={activeView === 'dockers' ? 'secondary' : 'ghost'}
            size="sm"
            onClick={handleDockerClick}
            className="gap-1.5 text-sm h-7"
          >
            <Container className="h-3.5 w-3.5" />
            Docker
          </Button>
          <NotificationDrawer />
        </div>
      </div>

      {/* Worktree Sub-Tabs (Second Row) - Project Features on Right */}
      {worktrees.length > 0 && (
        <div className="flex items-center justify-between px-2 py-1 border-t border-border/50 bg-muted/20">
          {/* Left side: Worktree tabs */}
          <div className="flex items-center gap-1">
            <GitBranch className="h-3.5 w-3.5 text-muted-foreground mr-1" />
            {worktrees.map((wt, index) => (
              <div
                key={wt.id}
                onClick={() => handleSwitchWorktree(index)}
                className={cn(
                  'flex items-center gap-1 px-2 py-1 rounded text-xs cursor-pointer transition-colors',
                  'hover:bg-accent',
                  index === activeWorktreeIndex
                    ? 'bg-background border shadow-sm font-medium'
                    : 'text-muted-foreground'
                )}
              >
                {getWorktreeModified(wt) && <span className="text-yellow-500">*</span>}
                <span className="truncate max-w-[100px]">{wt.branch}</span>
                {wt.is_main && (
                  <span className="text-[10px] text-muted-foreground/70">(main)</span>
                )}
              </div>
            ))}
            {/* Add Worktree Button */}
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6 text-muted-foreground ml-1"
              onClick={() => setAddWorktreeDialogOpen(true)}
            >
              <Plus className="h-3.5 w-3.5" />
            </Button>
          </div>

          {/* Right side: Project features (Env, Agent Rules) */}
          <div className="flex items-center gap-1">
            <Button
              variant={activeView === 'env' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={handleEnvClick}
              className="gap-1.5 text-xs h-6"
            >
              <FileCode className="h-3.5 w-3.5" />
              Env
            </Button>
            <Button
              variant={activeView === 'agent_rules' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={handleAgentRulesClick}
              className="gap-1.5 text-xs h-6"
            >
              <Bot className="h-3.5 w-3.5" />
              Agent Rules
            </Button>
          </div>
        </div>
      )}

      {/* Add Worktree Dialog */}
      {activeProject && (
        <AddWorktreeDialog
          open={addWorktreeDialogOpen}
          onOpenChange={setAddWorktreeDialogOpen}
          repoPath={activeProject.path}
          onAddFromBranch={handleAddWorktreeFromBranch}
          onAddNewBranch={handleAddWorktreeNewBranch}
        />
      )}
    </div>
  )
}
