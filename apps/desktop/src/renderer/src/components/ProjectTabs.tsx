/**
 * Project tabs component - displays open projects at the top of the window.
 */

import { X, Plus, FolderOpen } from 'lucide-react'
import { Button } from './ui/button'
import { useActiveProject } from '../hooks/useAppState'
import { cn } from '@/lib/utils'

export function ProjectTabs() {
  const { projects, activeIndex, dispatch } = useActiveProject()

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

  return (
    <div className="flex items-center gap-1 bg-muted/30 border-b px-2 py-1 min-h-[40px]">
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
          {projects.map((project, index) => (
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
                {project.is_modified && <span className="text-yellow-500 mr-1">*</span>}
                {project.name}
              </span>
              <button
                onClick={(e) => handleCloseProject(e, index)}
                className="opacity-0 group-hover:opacity-100 hover:bg-destructive/20 rounded p-0.5 transition-opacity"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          ))}
          <Button
            variant="ghost"
            size="icon"
            onClick={handleOpenProject}
            className="h-7 w-7 text-muted-foreground"
          >
            <Plus className="h-4 w-4" />
          </Button>
        </>
      )}
    </div>
  )
}
