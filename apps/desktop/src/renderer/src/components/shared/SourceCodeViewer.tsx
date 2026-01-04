import * as React from 'react'
import { useEffect, useState } from 'react'
import { AlertCircle, FileCode, Loader2 } from 'lucide-react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { ScrollArea } from '@/components/ui/scroll-area'

interface SourceCodeViewerProps {
  /** Absolute or relative path to the file */
  path: string
  /** Project root for security validation */
  projectRoot: string
  /** Optional: Show line numbers (default: true) */
  showLineNumbers?: boolean
  /** Optional: Maximum height with scroll */
  maxHeight?: string
  /** Optional: Callback when file cannot be read */
  onError?: (error: string) => void
}

type ViewerState =
  | { status: 'loading' }
  | { status: 'success'; content: string }
  | { status: 'error'; message: string }

/**
 * Parse error code from Rust error message format "CODE: message"
 */
function parseErrorMessage(error: string): { code: string; message: string } {
  const colonIndex = error.indexOf(':')
  if (colonIndex > 0) {
    const code = error.substring(0, colonIndex).trim()
    const message = error.substring(colonIndex + 1).trim()
    return { code, message }
  }
  return { code: 'UNKNOWN', message: error }
}

/**
 * Get user-friendly error message based on error code
 */
function getFriendlyErrorMessage(code: string, path: string): string {
  switch (code) {
    case 'FILE_NOT_FOUND':
      return `File not found: ${path}`
    case 'SECURITY_VIOLATION':
      return 'Access denied: File is outside project scope'
    case 'FILE_TOO_LARGE':
      return 'File too large to display (max 10MB)'
    case 'NOT_UTF8':
      return 'Cannot display: File is not UTF-8 text'
    case 'PERMISSION_DENIED':
      return 'Permission denied: Cannot read file'
    default:
      return `Error reading file: ${path}`
  }
}

/**
 * Component for viewing source code files with basic styling.
 * Uses the secure file reader API (window.api.file.read).
 */
export function SourceCodeViewer({
  path,
  projectRoot,
  showLineNumbers = true,
  maxHeight = '400px',
  onError,
}: SourceCodeViewerProps): React.ReactElement {
  const [state, setState] = useState<ViewerState>({ status: 'loading' })

  useEffect(() => {
    let cancelled = false

    async function loadFile(): Promise<void> {
      setState({ status: 'loading' })

      try {
        // Build absolute path if relative
        const absPath = path.startsWith('/')
          ? path
          : `${projectRoot}/${path}`

        const content = await window.api.file.read(absPath, projectRoot)

        if (!cancelled) {
          setState({ status: 'success', content })
        }
      } catch (error) {
        if (!cancelled) {
          const errorStr = error instanceof Error ? error.message : String(error)
          const { code } = parseErrorMessage(errorStr)
          const friendlyMessage = getFriendlyErrorMessage(code, path)

          setState({ status: 'error', message: friendlyMessage })
          onError?.(friendlyMessage)
        }
      }
    }

    loadFile()

    return () => {
      cancelled = true
    }
  }, [path, projectRoot, onError])

  if (state.status === 'loading') {
    return (
      <div className="flex items-center justify-center py-8 text-muted-foreground">
        <Loader2 className="h-5 w-5 animate-spin mr-2" />
        <span>Loading file...</span>
      </div>
    )
  }

  if (state.status === 'error') {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>{state.message}</AlertDescription>
      </Alert>
    )
  }

  const lines = state.content.split('\n')

  return (
    <div className="rounded-md border bg-muted/50">
      <div className="flex items-center gap-2 px-3 py-2 border-b bg-muted/30 text-sm text-muted-foreground">
        <FileCode className="h-4 w-4" />
        <span className="font-mono text-xs">{path}</span>
      </div>
      <ScrollArea style={{ maxHeight }} className="p-0">
        <pre className="p-3 text-sm font-mono overflow-x-auto">
          <code>
            {showLineNumbers ? (
              <table className="border-collapse w-full">
                <tbody>
                  {lines.map((line, index) => (
                    <tr key={index} className="hover:bg-muted/50">
                      <td className="pr-4 text-right text-muted-foreground select-none w-[1%] whitespace-nowrap">
                        {index + 1}
                      </td>
                      <td className="whitespace-pre">{line}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              state.content
            )}
          </code>
        </pre>
      </ScrollArea>
    </div>
  )
}

export default SourceCodeViewer
