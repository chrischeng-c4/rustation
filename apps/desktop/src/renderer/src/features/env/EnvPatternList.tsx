import { useState, useCallback } from 'react'
import { Plus, X, FileText, Folder } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

interface EnvPatternListProps {
  /** Current list of tracked patterns */
  patterns: string[]
  /** Callback when patterns change */
  onPatternsChange: (patterns: string[]) => void
  /** Whether editing is disabled */
  disabled?: boolean
}

/**
 * Editable list of env file patterns (e.g., ".env", ".envrc", ".claude/").
 * Allows adding and removing patterns.
 */
export function EnvPatternList({
  patterns,
  onPatternsChange,
  disabled = false,
}: EnvPatternListProps) {
  const [newPattern, setNewPattern] = useState('')

  const handleAddPattern = useCallback(() => {
    const trimmed = newPattern.trim()
    if (!trimmed) return
    if (patterns.includes(trimmed)) {
      setNewPattern('')
      return
    }
    onPatternsChange([...patterns, trimmed])
    setNewPattern('')
  }, [newPattern, patterns, onPatternsChange])

  const handleRemovePattern = useCallback(
    (pattern: string) => {
      onPatternsChange(patterns.filter((p) => p !== pattern))
    },
    [patterns, onPatternsChange]
  )

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter') {
        e.preventDefault()
        handleAddPattern()
      }
    },
    [handleAddPattern]
  )

  const isDirectory = (pattern: string) => pattern.endsWith('/')

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium">Tracked Patterns</span>
        <span className="text-xs text-muted-foreground">{patterns.length} pattern(s)</span>
      </div>

      {/* Pattern list */}
      <div className="space-y-2">
        {patterns.map((pattern) => (
          <div
            key={pattern}
            className="flex items-center justify-between rounded-md border px-3 py-2"
          >
            <span className="flex items-center gap-2 text-sm">
              {isDirectory(pattern) ? (
                <Folder className="h-4 w-4 text-muted-foreground" />
              ) : (
                <FileText className="h-4 w-4 text-muted-foreground" />
              )}
              <code className="font-mono">{pattern}</code>
            </span>
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 p-0"
              onClick={() => handleRemovePattern(pattern)}
              disabled={disabled}
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        ))}

        {patterns.length === 0 && (
          <div className="rounded-md border border-dashed px-3 py-4 text-center text-sm text-muted-foreground">
            No patterns configured. Add patterns like .env or .claude/
          </div>
        )}
      </div>

      {/* Add new pattern */}
      <div className="flex gap-2">
        <Input
          placeholder="Add pattern (e.g., .env.local)"
          value={newPattern}
          onChange={(e) => setNewPattern(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          className="font-mono"
        />
        <Button
          variant="outline"
          size="icon"
          onClick={handleAddPattern}
          disabled={disabled || !newPattern.trim()}
        >
          <Plus className="h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}
