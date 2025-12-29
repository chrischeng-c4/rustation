import { Play, Loader2, CheckCircle, XCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import type { JustCommand, TaskStatus } from '@/types/task'

interface TaskCardProps {
  command: JustCommand
  status: TaskStatus
  isActive?: boolean
  onRun?: (name: string) => void
}

export function TaskCard({ command, status, isActive = false, onRun }: TaskCardProps) {
  const isRunning = status === 'running'

  return (
    <div
      data-testid={`task-card-${command.name}`}
      className={cn(
        'flex items-center justify-between rounded-lg border p-3 transition-colors',
        isActive && 'border-primary bg-primary/5',
        !isActive && 'hover:bg-muted/50'
      )}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="font-mono text-sm font-medium">{command.name}</span>
          {status === 'success' && <CheckCircle className="h-4 w-4 text-green-500" />}
          {status === 'error' && <XCircle className="h-4 w-4 text-red-500" />}
        </div>
        {command.description && (
          <p className="mt-0.5 truncate text-xs text-muted-foreground">{command.description}</p>
        )}
      </div>

      <Button
        variant={isRunning ? 'secondary' : 'outline'}
        size="sm"
        disabled={isRunning}
        onClick={() => onRun?.(command.name)}
        className="ml-2 shrink-0"
      >
        {isRunning ? (
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
        ) : (
          <Play className="h-3.5 w-3.5" />
        )}
      </Button>
    </div>
  )
}
