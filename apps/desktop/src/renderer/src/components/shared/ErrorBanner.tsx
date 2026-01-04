import { AlertCircle } from 'lucide-react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { cn } from '@/lib/utils'

interface ErrorBannerProps {
  error: string
  className?: string
}

/**
 * ErrorBanner - Standardized error display for features.
 */
export function ErrorBanner({ error, className }: ErrorBannerProps) {
  return (
    <Alert variant="destructive" className={cn("mb-4", className)}>
      <AlertCircle className="h-4 w-4" />
      <AlertDescription className="text-sm">
        {error}
      </AlertDescription>
    </Alert>
  )
}
