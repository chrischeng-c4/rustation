import { Play, Loader2, CheckCircle, XCircle, Bot, MessageSquare } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import type { JustCommand, TaskStatus } from '@/types/task'

interface TaskCardProps {
  command: JustCommand
  status: TaskStatus
  isActive?: boolean
  isClaudeCode?: boolean
  onRun?: (name: string) => void
}

export function TaskCard({
  command,
  status,
  isActive = false,
  isClaudeCode = false,
  onRun,
}: TaskCardProps) {
  const isRunning = status === 'running'

  return (
    <div
      data-testid={`task-card-${command.name}`}
      className={cn(
        'flex items-center justify-between rounded-lg border p-3 transition-colors',
        // Claude Code special styling
        isClaudeCode && isActive && 'border-violet-500 bg-violet-500/10',
        isClaudeCode && !isActive && 'border-violet-200 bg-violet-50/50 hover:bg-violet-100/50 dark:border-violet-800 dark:bg-violet-950/20 dark:hover:bg-violet-900/30',
        // Regular command styling
        !isClaudeCode && isActive && 'border-primary bg-primary/5',
        !isClaudeCode && !isActive && 'hover:bg-muted/50'
      )}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          {isClaudeCode && <Bot className="h-4 w-4 text-violet-500" />}
          <span className={cn('font-mono text-sm font-medium', isClaudeCode && 'text-violet-700 dark:text-violet-300')}>
            {isClaudeCode ? 'Claude Code' : command.name}
          </span>
          {!isClaudeCode && status === 'success' && <CheckCircle className="h-4 w-4 text-green-500" />}
          {!isClaudeCode && status === 'error' && <XCircle className="h-4 w-4 text-red-500" />}
        </div>
        {command.description && (
          <p className="mt-0.5 truncate text-xs text-muted-foreground">{command.description}</p>
        )}
      </div>

      <Button
        variant={isClaudeCode ? (isActive ? 'default' : 'outline') : (isRunning ? 'secondary' : 'outline')}
        size="sm"
        disabled={!isClaudeCode && isRunning}
        onClick={() => onRun?.(command.name)}
        className={cn('ml-2 shrink-0', isClaudeCode && 'border-violet-300 hover:bg-violet-100 dark:border-violet-700 dark:hover:bg-violet-900')}
      >
        {isClaudeCode ? (
          <MessageSquare className="h-3.5 w-3.5" />
        ) : isRunning ? (
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
        ) : (
          <Play className="h-3.5 w-3.5" />
        )}
      </Button>
    </div>
  )
}
