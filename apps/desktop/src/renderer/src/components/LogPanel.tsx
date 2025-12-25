import { useCallback, useEffect, useRef } from 'react'
import { Terminal, Copy, RefreshCw, Check } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useState } from 'react'

interface LogPanelProps {
  title?: string
  logs: string[]
  onRefresh?: () => void
  isRefreshing?: boolean
  showCopy?: boolean
  emptyMessage?: string
}

export function LogPanel({
  title = 'Output',
  logs,
  onRefresh,
  isRefreshing = false,
  showCopy = true,
  emptyMessage = 'No output',
}: LogPanelProps) {
  const [copied, setCopied] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)

  // Auto-scroll to bottom when logs change
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [logs])

  const handleCopy = useCallback(async () => {
    const text = logs.join('\n')
    await navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }, [logs])

  return (
    <div className="flex h-full flex-col rounded-lg border">
      {/* Header */}
      <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
        <div className="flex items-center gap-2">
          <Terminal className="h-4 w-4" />
          <span className="text-sm font-medium">{title}</span>
        </div>
        <div className="flex gap-1">
          {showCopy && logs.length > 0 && (
            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={handleCopy}>
              {copied ? (
                <Check className="h-3.5 w-3.5 text-green-500" />
              ) : (
                <Copy className="h-3.5 w-3.5" />
              )}
            </Button>
          )}
          {onRefresh && (
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={onRefresh}
              disabled={isRefreshing}
            >
              <RefreshCw className={`h-3.5 w-3.5 ${isRefreshing ? 'animate-spin' : ''}`} />
            </Button>
          )}
        </div>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1 p-4" ref={scrollRef}>
        {logs.length > 0 ? (
          <pre className="whitespace-pre-wrap font-mono text-xs">{logs.join('\n')}</pre>
        ) : (
          <p className="text-sm text-muted-foreground">{emptyMessage}</p>
        )}
      </ScrollArea>
    </div>
  )
}
