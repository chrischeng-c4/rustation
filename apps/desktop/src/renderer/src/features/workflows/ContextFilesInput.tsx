import * as React from 'react'
import { useState, useCallback } from 'react'
import { Plus, X, Eye, FileCode, AlertCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { SourceCodeViewer } from '@/components/shared/SourceCodeViewer'
import { useAppState } from '@/hooks/useAppState'

interface ContextFilesInputProps {
  /** Change ID to manage context files for */
  changeId: string
  /** Current list of context file paths */
  files: string[]
  /** Project root for file reading */
  projectRoot: string
}

/**
 * Component for managing source file selection for Claude context injection.
 * Allows adding/removing file paths that will be included when generating proposals/plans.
 */
export function ContextFilesInput({
  changeId,
  files,
  projectRoot,
}: ContextFilesInputProps): React.ReactElement {
  const { dispatch } = useAppState()
  const [inputValue, setInputValue] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isValidating, setIsValidating] = useState(false)
  const [previewPath, setPreviewPath] = useState<string | null>(null)

  const handleAddFile = useCallback(async () => {
    const path = inputValue.trim()
    if (!path) return

    // Check if already added
    if (files.includes(path)) {
      setError('File already added')
      return
    }

    setError(null)
    setIsValidating(true)

    try {
      // Build absolute path if relative
      const absPath = path.startsWith('/')
        ? path
        : `${projectRoot}/${path}`

      // Validate by attempting to read the file
      await window.api.file.read(absPath, projectRoot)

      // File exists and is readable - add it
      await dispatch({
        type: 'AddContextFile',
        change_id: changeId,
        path: path,
      })

      setInputValue('')
      setError(null)
    } catch (err) {
      const errorStr = err instanceof Error ? err.message : String(err)
      // Parse error code
      const colonIndex = errorStr.indexOf(':')
      const code = colonIndex > 0 ? errorStr.substring(0, colonIndex).trim() : 'UNKNOWN'

      switch (code) {
        case 'FILE_NOT_FOUND':
          setError(`File not found: ${path}`)
          break
        case 'SECURITY_VIOLATION':
          setError('File is outside project scope')
          break
        case 'FILE_TOO_LARGE':
          setError('File too large (max 10MB)')
          break
        case 'NOT_UTF8':
          setError('File is not text (binary files not supported)')
          break
        default:
          setError(`Cannot read file: ${path}`)
      }
    } finally {
      setIsValidating(false)
    }
  }, [inputValue, files, projectRoot, changeId, dispatch])

  const handleRemoveFile = useCallback(async (path: string) => {
    await dispatch({
      type: 'RemoveContextFile',
      change_id: changeId,
      path: path,
    })
  }, [changeId, dispatch])

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleAddFile()
    }
  }, [handleAddFile])

  return (
    <div className="space-y-3">
      {/* Input for adding files */}
      <div className="flex gap-2">
        <Input
          placeholder="Enter file path (e.g., src/lib/utils.ts)"
          value={inputValue}
          onChange={(e) => {
            setInputValue(e.target.value)
            setError(null)
          }}
          onKeyDown={handleKeyDown}
          className="font-mono text-sm"
        />
        <Button
          size="sm"
          onClick={handleAddFile}
          disabled={!inputValue.trim() || isValidating}
        >
          {isValidating ? (
            <span className="animate-pulse">...</span>
          ) : (
            <>
              <Plus className="h-4 w-4 mr-1" />
              Add
            </>
          )}
        </Button>
      </div>

      {/* Error message */}
      {error && (
        <Alert variant="destructive" className="py-2">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription className="text-sm">{error}</AlertDescription>
        </Alert>
      )}

      {/* List of added files */}
      {files.length > 0 ? (
        <div className="space-y-2">
          {files.map((path) => (
            <div
              key={path}
              className="flex items-center gap-2 p-2 rounded-md bg-muted/50 border"
            >
              <FileCode className="h-4 w-4 text-muted-foreground flex-shrink-0" />
              <span className="flex-1 font-mono text-sm truncate" title={path}>
                {path}
              </span>

              {/* Preview button */}
              <Sheet
                open={previewPath === path}
                onOpenChange={(open) => setPreviewPath(open ? path : null)}
              >
                <SheetTrigger asChild>
                  <Button variant="ghost" size="icon" className="h-7 w-7">
                    <Eye className="h-4 w-4" />
                  </Button>
                </SheetTrigger>
                <SheetContent side="right" className="w-[600px] sm:max-w-[600px]">
                  <SheetHeader>
                    <SheetTitle className="flex items-center gap-2">
                      <FileCode className="h-5 w-5" />
                      <span className="font-mono text-sm">{path}</span>
                    </SheetTitle>
                    <SheetDescription>
                      Preview of source file content
                    </SheetDescription>
                  </SheetHeader>
                  <div className="mt-4">
                    <SourceCodeViewer
                      path={path}
                      projectRoot={projectRoot}
                      maxHeight="calc(100vh - 200px)"
                    />
                  </div>
                </SheetContent>
              </Sheet>

              {/* Remove button */}
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 text-muted-foreground hover:text-destructive"
                onClick={() => handleRemoveFile(path)}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          ))}
        </div>
      ) : (
        <p className="text-sm text-muted-foreground py-2">
          No files selected. Add source files to include as context for Claude.
        </p>
      )}

      {/* File count badge */}
      {files.length > 0 && (
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Badge variant="secondary" className="font-mono">
            {files.length} file{files.length !== 1 ? 's' : ''}
          </Badge>
          <span>will be included as context</span>
        </div>
      )}
    </div>
  )
}

export default ContextFilesInput
