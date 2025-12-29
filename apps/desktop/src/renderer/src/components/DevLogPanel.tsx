import { useState, useCallback, useMemo } from 'react'
import { Bug, Trash2, ChevronDown, ChevronRight, Copy, Check, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useAppState } from '@/hooks/useAppState'
import type { DevLog, DevLogSource, DevLogType } from '@/types/state'
import { cn } from '@/lib/utils'

/**
 * DevLogPanel - Right-side panel for displaying development logs
 *
 * Features:
 * - Collapsible entries (collapsed = summary, expanded = beautiful JSON)
 * - Source and type badges with colors
 * - Clear all logs button
 * - Dev mode only
 */
export function DevLogPanel() {
  const { state, dispatch } = useAppState()
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set())
  const [isOpen, setIsOpen] = useState(true)

  const devLogs = state?.dev_logs ?? []

  const toggleExpand = useCallback((id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }, [])

  const handleClear = useCallback(async () => {
    await dispatch({ type: 'ClearDevLogs' })
  }, [dispatch])

  const handleClose = useCallback(() => {
    setIsOpen(false)
  }, [])

  if (!isOpen) {
    return (
      <Button
        variant="ghost"
        size="sm"
        onClick={() => setIsOpen(true)}
        className="fixed right-2 top-2 z-50"
        title="Open Dev Logs"
      >
        <Bug className="h-4 w-4" />
      </Button>
    )
  }

  return (
    <div className="flex h-full w-80 flex-col border-l bg-background">
      {/* Header */}
      <div className="flex items-center justify-between border-b bg-muted/40 px-3 py-2">
        <div className="flex items-center gap-2">
          <Bug className="h-4 w-4 text-orange-500" />
          <span className="text-sm font-medium">Dev Logs</span>
          <span className="text-xs text-muted-foreground">({devLogs.length})</span>
        </div>
        <div className="flex gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            onClick={handleClear}
            disabled={devLogs.length === 0}
            title="Clear all logs"
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            onClick={handleClose}
            title="Close panel"
          >
            <X className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      {/* Log Entries */}
      <ScrollArea className="flex-1">
        {devLogs.length === 0 ? (
          <div className="flex h-32 items-center justify-center text-sm text-muted-foreground">
            No dev logs yet
          </div>
        ) : (
          <div className="space-y-1 p-2">
            {devLogs.map((log) => (
              <DevLogEntry
                key={log.id}
                log={log}
                isExpanded={expandedIds.has(log.id)}
                onToggle={() => toggleExpand(log.id)}
              />
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  )
}

interface DevLogEntryProps {
  log: DevLog
  isExpanded: boolean
  onToggle: () => void
}

function DevLogEntry({ log, isExpanded, onToggle }: DevLogEntryProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = useCallback(
    async (e: React.MouseEvent) => {
      e.stopPropagation()
      await navigator.clipboard.writeText(JSON.stringify(log.data, null, 2))
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    },
    [log.data]
  )

  const timestamp = useMemo(() => {
    return new Date(log.timestamp).toLocaleTimeString()
  }, [log.timestamp])

  return (
    <div
      className={cn(
        'rounded border bg-card text-card-foreground',
        isExpanded && 'ring-1 ring-primary/20'
      )}
    >
      {/* Collapsed Header (always visible) */}
      <div
        className="flex cursor-pointer items-center gap-2 px-2 py-1.5 hover:bg-muted/50"
        onClick={onToggle}
      >
        {isExpanded ? (
          <ChevronDown className="h-3.5 w-3.5 flex-shrink-0 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-3.5 w-3.5 flex-shrink-0 text-muted-foreground" />
        )}

        <SourceBadge source={log.source} />
        <TypeBadge logType={log.log_type} />

        <span className="flex-1 truncate text-xs">{log.summary}</span>

        <span className="text-[10px] text-muted-foreground">{timestamp}</span>
      </div>

      {/* Expanded Content */}
      {isExpanded && (
        <div className="border-t bg-muted/30 p-2">
          <div className="flex items-center justify-between pb-1">
            <span className="text-[10px] font-medium text-muted-foreground">DATA</span>
            <Button
              variant="ghost"
              size="icon"
              className="h-5 w-5"
              onClick={handleCopy}
              title="Copy JSON"
            >
              {copied ? (
                <Check className="h-3 w-3 text-green-500" />
              ) : (
                <Copy className="h-3 w-3" />
              )}
            </Button>
          </div>
          <pre className="max-h-60 overflow-auto rounded bg-muted p-2 font-mono text-[10px]">
            {JSON.stringify(log.data, null, 2)}
          </pre>
        </div>
      )}
    </div>
  )
}

function SourceBadge({ source }: { source: DevLogSource }) {
  const config = {
    rust: { label: 'RS', className: 'bg-orange-500/20 text-orange-600' },
    frontend: { label: 'FE', className: 'bg-blue-500/20 text-blue-600' },
    claude: { label: 'CL', className: 'bg-purple-500/20 text-purple-600' },
    ipc: { label: 'IPC', className: 'bg-gray-500/20 text-gray-600' },
  }[source]

  return (
    <span
      className={cn(
        'flex-shrink-0 rounded px-1 py-0.5 text-[9px] font-semibold uppercase',
        config.className
      )}
    >
      {config.label}
    </span>
  )
}

function TypeBadge({ logType }: { logType: DevLogType }) {
  const config = {
    action: { label: 'ACT', className: 'bg-green-500/20 text-green-600' },
    state: { label: 'STA', className: 'bg-cyan-500/20 text-cyan-600' },
    claude: { label: 'CLU', className: 'bg-purple-500/20 text-purple-600' },
    error: { label: 'ERR', className: 'bg-red-500/20 text-red-600' },
    info: { label: 'INF', className: 'bg-gray-500/20 text-gray-600' },
  }[logType]

  return (
    <span
      className={cn(
        'flex-shrink-0 rounded px-1 py-0.5 text-[9px] font-semibold uppercase',
        config.className
      )}
    >
      {config.label}
    </span>
  )
}
