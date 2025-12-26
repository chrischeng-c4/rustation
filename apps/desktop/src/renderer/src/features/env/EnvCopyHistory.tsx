import { useMemo } from 'react'
import { CheckCircle2, AlertCircle, FileText, Clock } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import type { EnvCopyResult } from '@/types/state'

interface EnvCopyHistoryProps {
  /** The last copy result (if any) */
  lastResult: EnvCopyResult | null
}

/**
 * Displays the result of the most recent env file copy operation.
 * Shows copied files and any failures.
 */
export function EnvCopyHistory({ lastResult }: EnvCopyHistoryProps) {
  const formattedTime = useMemo(() => {
    if (!lastResult?.timestamp) return null
    try {
      const date = new Date(lastResult.timestamp)
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    } catch {
      return null
    }
  }, [lastResult?.timestamp])

  if (!lastResult) {
    return (
      <div className="rounded-md border border-dashed px-4 py-6 text-center">
        <Clock className="mx-auto h-8 w-8 text-muted-foreground/50" />
        <p className="mt-2 text-sm text-muted-foreground">No recent copy operations</p>
      </div>
    )
  }

  const { copied_files, failed_files } = lastResult
  const totalCopied = copied_files.length
  const totalFailed = failed_files.length
  const isEmpty = totalCopied === 0 && totalFailed === 0

  if (isEmpty) {
    return (
      <div className="rounded-md border px-4 py-4">
        <div className="flex items-center gap-2 text-muted-foreground">
          <FileText className="h-4 w-4" />
          <span className="text-sm">No files to copy (all patterns already exist in target)</span>
          {formattedTime && (
            <span className="ml-auto text-xs">{formattedTime}</span>
          )}
        </div>
      </div>
    )
  }

  const isSuccess = totalFailed === 0
  const isPartial = totalCopied > 0 && totalFailed > 0

  return (
    <div className="space-y-3">
      {/* Summary */}
      <div className="flex items-center gap-2">
        {isSuccess ? (
          <CheckCircle2 className="h-4 w-4 text-green-500" />
        ) : (
          <AlertCircle className="h-4 w-4 text-yellow-500" />
        )}
        <span className="text-sm font-medium">
          {isSuccess
            ? `Copied ${totalCopied} file(s)`
            : isPartial
              ? `Copied ${totalCopied}, failed ${totalFailed}`
              : `Failed to copy ${totalFailed} file(s)`}
        </span>
        {formattedTime && (
          <span className="ml-auto text-xs text-muted-foreground">{formattedTime}</span>
        )}
      </div>

      {/* Copied files */}
      {copied_files.length > 0 && (
        <div className="space-y-1">
          <span className="text-xs font-medium text-muted-foreground">Copied:</span>
          <div className="flex flex-wrap gap-1">
            {copied_files.map((file) => (
              <Badge key={file} variant="secondary" className="font-mono text-xs">
                {file}
              </Badge>
            ))}
          </div>
        </div>
      )}

      {/* Failed files */}
      {failed_files.length > 0 && (
        <div className="space-y-1">
          <span className="text-xs font-medium text-red-500">Failed:</span>
          <div className="space-y-1">
            {failed_files.map(([file, error]) => (
              <div
                key={file}
                className="flex items-start gap-2 rounded border border-red-200 bg-red-50 px-2 py-1 text-xs dark:border-red-900 dark:bg-red-950"
              >
                <code className="font-mono">{file}</code>
                <span className="text-red-600 dark:text-red-400">{error}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
