import { useCallback, useMemo } from 'react'
import { Code as FileCode } from '@mui/icons-material'
import { Box, Typography } from '@mui/material'
import { useActiveWorktree } from '@/hooks/useAppState'
import { SourceCodeViewer, type CommentData } from '@/components/shared/SourceCodeViewer'

// Helper to determine if path is a file (has extension in filename)
// This is a heuristic - it may not be 100% accurate for all edge cases
const isFilePath = (path: string): boolean => {
  const name = path.split('/').pop() ?? ''
  if (!name) return false

  // Common dotfiles without extensions that are files
  const knownDotfiles = ['.gitignore', '.gitattributes', '.editorconfig', '.prettierignore', '.npmrc', '.nvmrc', '.env']
  if (knownDotfiles.includes(name)) return true

  // Handle dotfiles with extensions like .eslintrc.json
  if (name.startsWith('.')) {
    const afterFirstDot = name.slice(1)
    return afterFirstDot.includes('.')
  }

  // Regular files have at least one dot (extension)
  return name.includes('.')
}

export function DetailPanel() {
  const { worktree, dispatch } = useActiveWorktree()
  const explorer = worktree?.explorer
  // Use active_tab_path for displaying content (from tabs), fallback to selected_path
  const activeTabPath = explorer?.active_tab_path
  const selectedPath = activeTabPath ?? explorer?.selected_path
  // First try to find in entries (root level), otherwise derive from path
  const entryFromState = explorer?.entries.find((e) => e.path === selectedPath)
  const selectedEntry = entryFromState ?? (selectedPath ? {
    path: selectedPath,
    kind: isFilePath(selectedPath) ? 'file' as const : 'directory' as const,
    name: selectedPath.split('/').pop() ?? '',
  } : undefined)
  const comments = explorer?.selected_comments ?? []

  // Filter to only inline comments (with line numbers)
  const inlineComments = useMemo(() => {
    return comments
      .filter((c) => c.line_number !== null && c.line_number !== undefined)
      .map((c) => c as CommentData)
  }, [comments])

  // Handle inline comment addition
  const handleAddInlineComment = useCallback(
    async (lineNumber: number, content: string) => {
      if (!selectedPath) return

      console.log('[DetailPanel] Adding comment:', { lineNumber, content, path: selectedPath })

      await dispatch({
        type: 'AddFileComment',
        payload: { path: selectedPath, content, line_number: lineNumber },
      })

      console.log('[DetailPanel] Comment dispatch completed')
    },
    [selectedPath, dispatch]
  )

  if (!selectedPath) {
    return (
      <Box
        sx={{
          display: 'flex',
          height: '100%',
          alignItems: 'center',
          justifyContent: 'center',
          p: 4,
          textAlign: 'center',
          color: 'text.secondary',
        }}
      >
        <Box>
          <FileCode sx={{ fontSize: 48, mb: 2, opacity: 0.1 }} />
          <Typography variant="body2">Select a file to preview</Typography>
        </Box>
      </Box>
    )
  }

  // At this point selectedPath is defined, so we can determine file kind
  const isFile = selectedEntry?.kind === 'file' || isFilePath(selectedPath)
  const rootPath = worktree?.path ?? ''

  return (
    <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column', overflow: 'hidden' }}>
      {/* File content */}
      <Box sx={{ flex: 1, minHeight: 0, overflow: 'hidden' }}>
        {isFile ? (
          <SourceCodeViewer
            path={selectedPath}
            projectRoot={rootPath}
            comments={inlineComments}
            onAddComment={handleAddInlineComment}
          />
        ) : (
          <Box
            sx={{
              display: 'flex',
              height: '100%',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'text.secondary',
            }}
          >
            <Typography variant="body2">Preview only available for files</Typography>
          </Box>
        )}
      </Box>
    </Box>
  )
}
