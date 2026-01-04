import { RefreshCw } from 'lucide-react'
import { cn } from '@/lib/utils'

interface LoadingStateProps {
  message?: string
  className?: string
}

/**
 * LoadingState - Standardized loading indicator for pages or sections.
 */
export function LoadingState({
  message = "Loading...",
  className,
}: LoadingStateProps) {
  return (
    <div
      className={cn(
        "flex h-full w-full flex-col items-center justify-center gap-4",
        className
      )}
    >
      <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
      <p className="text-sm font-medium text-muted-foreground">{message}</p>
    </div>
  )
}
