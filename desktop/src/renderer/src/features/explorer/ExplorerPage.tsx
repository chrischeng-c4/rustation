import { useEffect } from 'react'
import { FolderOpen } from '@mui/icons-material'
import { Box, Paper } from '@mui/material'
import { PageHeader } from '@/components/shared/PageHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { useActiveWorktree } from '@/hooks/useAppState'
import { FileTreeView } from './FileTreeView'
import { FileTabs } from './FileTabs'
import { DetailPanel } from './DetailPanel'

export function ExplorerPage() {
  const { worktree, dispatch, isLoading } = useActiveWorktree()

  const explorer = worktree?.explorer
  const currentPath = explorer?.current_path

  // Initial load - trigger when we have a path but no entries loaded yet
  useEffect(() => {
    if (currentPath && explorer?.entries.length === 0 && !explorer?.is_loading) {
      dispatch({ type: 'ExploreDir', payload: { path: currentPath } })
    }
  }, [currentPath, explorer?.entries.length, explorer?.is_loading, dispatch])

  if (!worktree) {
    return (
      <EmptyState
        title="No Worktree Selected"
        description="Please select a project worktree to explore files."
      />
    )
  }

  if (isLoading || !explorer) {
    return <LoadingState message="Loading file explorer..." />
  }

  return (
    <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column', overflow: 'hidden', p: 3 }}>
      <PageHeader
        title="File Explorer"
        description="Browse files, view metadata, and manage comments"
        icon={<FolderOpen />}
      />

      <Box sx={{ flex: 1, display: 'flex', gap: 2, minHeight: 0 }}>
        {/* File Tree Panel (Sidebar) - VSCode style */}
        <Paper
          variant="outlined"
          sx={{
            width: 280,
            flexShrink: 0,
            overflow: 'hidden',
            borderRadius: 2,
            bgcolor: 'background.paper'
          }}
        >
          <FileTreeView />
        </Paper>

        {/* Detail/Preview Panel (Main Content) with Tabs */}
        <Paper
          variant="outlined"
          sx={{
            flex: 1,
            minWidth: 0,
            overflow: 'hidden',
            borderRadius: 2,
            bgcolor: 'background.paper',
            display: 'flex',
            flexDirection: 'column'
          }}
        >
          <FileTabs />
          <Box sx={{ flex: 1, minHeight: 0, overflow: 'hidden' }}>
            <DetailPanel />
          </Box>
        </Paper>
      </Box>
    </Box>
  )
}
