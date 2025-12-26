import { useMemo } from 'react'
import { ChevronDown, GitBranch } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import type { WorktreeState } from '@/types/state'

interface WorktreeSelectorProps {
  /** All available worktrees */
  worktrees: WorktreeState[]
  /** Currently selected worktree path */
  value: string | null
  /** Callback when selection changes */
  onChange: (path: string) => void
  /** Label for the selector */
  label?: string
  /** Paths to exclude from selection (e.g., the source worktree) */
  excludePaths?: string[]
  /** Placeholder text when nothing is selected */
  placeholder?: string
  /** Whether the selector is disabled */
  disabled?: boolean
}

/**
 * Dropdown selector for choosing a worktree.
 * Displays branch name with path as secondary info.
 */
export function WorktreeSelector({
  worktrees,
  value,
  onChange,
  label,
  excludePaths = [],
  placeholder = 'Select worktree',
  disabled = false,
}: WorktreeSelectorProps) {
  const filteredWorktrees = useMemo(
    () => worktrees.filter((w) => !excludePaths.includes(w.path)),
    [worktrees, excludePaths]
  )

  const selectedWorktree = useMemo(
    () => worktrees.find((w) => w.path === value),
    [worktrees, value]
  )

  return (
    <div className="flex flex-col gap-1.5">
      {label && <span className="text-sm font-medium">{label}</span>}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="outline"
            className="w-full justify-between"
            disabled={disabled || filteredWorktrees.length === 0}
          >
            <span className="flex items-center gap-2 truncate">
              <GitBranch className="h-4 w-4 shrink-0" />
              {selectedWorktree ? (
                <span className="truncate">{selectedWorktree.branch}</span>
              ) : (
                <span className="text-muted-foreground">{placeholder}</span>
              )}
            </span>
            <ChevronDown className="h-4 w-4 shrink-0 opacity-50" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-[300px]">
          {filteredWorktrees.map((worktree) => (
            <DropdownMenuItem
              key={worktree.path}
              onClick={() => onChange(worktree.path)}
              className="flex flex-col items-start gap-0.5"
            >
              <span className="flex items-center gap-2 font-medium">
                <GitBranch className="h-3.5 w-3.5" />
                {worktree.branch}
                {worktree.is_main && (
                  <span className="text-xs text-muted-foreground">(main)</span>
                )}
              </span>
              <span className="text-xs text-muted-foreground truncate max-w-[280px]">
                {worktree.path}
              </span>
            </DropdownMenuItem>
          ))}
          {filteredWorktrees.length === 0 && (
            <DropdownMenuItem disabled>No worktrees available</DropdownMenuItem>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}
