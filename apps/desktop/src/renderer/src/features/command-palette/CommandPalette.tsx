import { useCallback, useEffect, useMemo, useState } from 'react'
import { Command } from 'cmdk'
import {
  FolderOpen,
  GitBranch,
  Play,
  Container,
  Settings,
  FileCode,
  ListTodo,
  Sun,
  Moon,
  Monitor,
  Server,
  MessageSquare,
  TerminalSquare,
} from 'lucide-react'
import { useAppState, useActiveProject, useActiveWorktree } from '@/hooks/useAppState'
import type { Theme } from '@/types/state'
import './command-palette.css'

interface CommandPaletteProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

/**
 * Global Command Palette (Cmd+K / Ctrl+K)
 * Provides quick navigation and action execution.
 */
export function CommandPalette({ open, onOpenChange }: CommandPaletteProps) {
  const { state, dispatch } = useAppState()
  const { projects, activeIndex } = useActiveProject()
  const { worktree, worktrees } = useActiveWorktree()
  const [search, setSearch] = useState('')

  // Get tasks from active worktree
  const tasks = worktree?.tasks?.commands ?? []

  // Reset search when closing
  useEffect(() => {
    if (!open) {
      setSearch('')
    }
  }, [open])

  // Handle project switch
  const handleSwitchProject = useCallback(
    async (index: number) => {
      await dispatch({ type: 'SwitchProject', payload: { index } })
      onOpenChange(false)
    },
    [dispatch, onOpenChange]
  )

  // Handle worktree switch
  const handleSwitchWorktree = useCallback(
    async (index: number) => {
      await dispatch({ type: 'SwitchWorktree', payload: { index } })
      onOpenChange(false)
    },
    [dispatch, onOpenChange]
  )

  // Handle view change
  const handleSetView = useCallback(
    async (view: 'tasks' | 'dockers' | 'settings' | 'env' | 'mcp' | 'chat' | 'terminal') => {
      await dispatch({ type: 'SetActiveView', payload: { view } })
      onOpenChange(false)
    },
    [dispatch, onOpenChange]
  )

  // Handle run task
  const handleRunTask = useCallback(
    async (taskName: string) => {
      if (!worktree) return
      await dispatch({
        type: 'RunJustCommand',
        payload: { name: taskName, cwd: worktree.path },
      })
      await dispatch({ type: 'SetActiveView', payload: { view: 'tasks' } })
      onOpenChange(false)
    },
    [worktree, dispatch, onOpenChange]
  )

  // Handle theme change
  const handleSetTheme = useCallback(
    async (theme: Theme) => {
      await dispatch({ type: 'SetTheme', payload: { theme } })
      onOpenChange(false)
    },
    [dispatch, onOpenChange]
  )

  // Memoize filtered items for performance
  const hasProjects = projects.length > 0
  const hasWorktrees = worktrees.length > 0
  const hasTasks = tasks.length > 0

  return (
    <Command.Dialog
      open={open}
      onOpenChange={onOpenChange}
      label="Command Palette"
      className="command-palette"
    >
      <Command.Input
        value={search}
        onValueChange={setSearch}
        placeholder="Type a command or search..."
        className="command-input"
      />
      <Command.List className="command-list">
        <Command.Empty className="command-empty">No results found.</Command.Empty>

        {/* Projects */}
        {hasProjects && (
          <Command.Group heading="Projects" className="command-group">
            {projects.map((project, index) => (
              <Command.Item
                key={project.id}
                value={`project ${project.name}`}
                onSelect={() => handleSwitchProject(index)}
                className="command-item"
              >
                <FolderOpen className="command-icon" />
                <span>{project.name}</span>
                {index === activeIndex && (
                  <span className="command-badge">Active</span>
                )}
              </Command.Item>
            ))}
          </Command.Group>
        )}

        {/* Worktrees */}
        {hasWorktrees && (
          <Command.Group heading="Worktrees" className="command-group">
            {worktrees.map((wt, index) => (
              <Command.Item
                key={wt.id}
                value={`worktree ${wt.branch}`}
                onSelect={() => handleSwitchWorktree(index)}
                className="command-item"
              >
                <GitBranch className="command-icon" />
                <span>{wt.branch}</span>
                {wt.is_main && <span className="command-badge">main</span>}
              </Command.Item>
            ))}
          </Command.Group>
        )}

        {/* Tasks */}
        {hasTasks && (
          <Command.Group heading="Run Task" className="command-group">
            {tasks.slice(0, 10).map((task) => (
              <Command.Item
                key={task.name}
                value={`run task ${task.name} ${task.description ?? ''}`}
                onSelect={() => handleRunTask(task.name)}
                className="command-item"
              >
                <Play className="command-icon" />
                <span>just {task.name}</span>
                {task.description && (
                  <span className="command-description">{task.description}</span>
                )}
              </Command.Item>
            ))}
          </Command.Group>
        )}

        {/* Views */}
        <Command.Group heading="Views" className="command-group">
          <Command.Item
            value="view tasks"
            onSelect={() => handleSetView('tasks')}
            className="command-item"
          >
            <ListTodo className="command-icon" />
            <span>Tasks</span>
          </Command.Item>
          <Command.Item
            value="view rstn-mcp integration server"
            onSelect={() => handleSetView('mcp')}
            className="command-item"
          >
            <Server className="command-icon" />
            <span>rstn-mcp Integration</span>
          </Command.Item>
          <Command.Item
            value="view chat claude ai"
            onSelect={() => handleSetView('chat')}
            className="command-item"
          >
            <MessageSquare className="command-icon" />
            <span>Chat</span>
          </Command.Item>
          <Command.Item
            value="view terminal shell pty"
            onSelect={() => handleSetView('terminal')}
            className="command-item"
          >
            <TerminalSquare className="command-icon" />
            <span>Terminal</span>
          </Command.Item>
          <Command.Item
            value="view docker containers"
            onSelect={() => handleSetView('dockers')}
            className="command-item"
          >
            <Container className="command-icon" />
            <span>Docker</span>
          </Command.Item>
          <Command.Item
            value="view environment env"
            onSelect={() => handleSetView('env')}
            className="command-item"
          >
            <FileCode className="command-icon" />
            <span>Environment</span>
          </Command.Item>
          <Command.Item
            value="view settings preferences"
            onSelect={() => handleSetView('settings')}
            className="command-item"
          >
            <Settings className="command-icon" />
            <span>Settings</span>
          </Command.Item>
        </Command.Group>

        {/* Theme */}
        <Command.Group heading="Theme" className="command-group">
          <Command.Item
            value="theme system auto"
            onSelect={() => handleSetTheme('system')}
            className="command-item"
          >
            <Monitor className="command-icon" />
            <span>System Theme</span>
          </Command.Item>
          <Command.Item
            value="theme light"
            onSelect={() => handleSetTheme('light')}
            className="command-item"
          >
            <Sun className="command-icon" />
            <span>Light Theme</span>
          </Command.Item>
          <Command.Item
            value="theme dark"
            onSelect={() => handleSetTheme('dark')}
            className="command-item"
          >
            <Moon className="command-icon" />
            <span>Dark Theme</span>
          </Command.Item>
        </Command.Group>
      </Command.List>
    </Command.Dialog>
  )
}
