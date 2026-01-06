import { useEffect, useCallback } from 'react'
import { 
  ArrowBack as ArrowLeft, 
  ArrowForward as ArrowRight, 
  ArrowUpward as ArrowUp,
  FolderOpen 
} from '@mui/icons-material'
import { Box, IconButton, Divider } from '@mui/material'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui/resizable'
import { PageHeader } from '@/components/shared/PageHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { useActiveWorktree } from '@/hooks/useAppState'
import { PathBreadcrumbs } from './PathBreadcrumbs'
import { FileTable } from './FileTable'
import { DetailPanel } from './DetailPanel'

export function ExplorerPage() {
  const { worktree, dispatch, isLoading } = useActiveWorktree()
  
  const explorer = worktree?.explorer
  const currentPath = explorer?.current_path
  const canGoBack = (explorer?.history.back_stack.length ?? 0) > 0
  const canGoForward = (explorer?.history.forward_stack.length ?? 0) > 0

  // Initial load
  useEffect(() => {
    if (worktree?.path && !currentPath) {
      dispatch({ type: 'ExploreDir', payload: { path: worktree.path } })
    }
  }, [worktree?.path, currentPath, dispatch])

  const handleNavigateBack = useCallback(() => {
    dispatch({ type: 'NavigateBack' })
  }, [dispatch])

  const handleNavigateForward = useCallback(() => {
    dispatch({ type: 'NavigateForward' })
  }, [dispatch])

  const handleNavigateUp = useCallback(() => {
    dispatch({ type: 'NavigateUp' })
  }, [dispatch])

  const handlePathClick = useCallback((path: string) => {
    dispatch({ type: 'ExploreDir', payload: { path } })
  }, [dispatch])

  if (isLoading || !explorer) {
    return <LoadingState message="Loading file explorer..." />
  }

  if (!worktree) {
    return (
      <EmptyState 
        icon={FolderOpen} 
        title="No Worktree Selected" 
        description="Please select a project worktree to explore files." 
      />
    )
  }

  return (
    <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column', overflow: 'hidden' }}>
      <PageHeader
        title="File Explorer"
        description="Browse files, view metadata, and manage comments"
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
          <IconButton 
            size="small"
            disabled={!canGoBack} 
            onClick={handleNavigateBack}
          >
            <ArrowLeft fontSize="small" />
          </IconButton>
          <IconButton 
            size="small"
            disabled={!canGoForward} 
            onClick={handleNavigateForward}
          >
            <ArrowRight fontSize="small" />
          </IconButton>
          <IconButton 
            size="small"
            onClick={handleNavigateUp}
          >
            <ArrowUp fontSize="small" />
          </IconButton>
        </Box>
      </PageHeader>

      <Box sx={{ px: 2, py: 1, borderBottom: 1, borderColor: 'divider', bgcolor: 'background.paper', opacity: 0.8 }}>
        <PathBreadcrumbs 
          currentPath={currentPath ?? ''} 
          rootPath={worktree.path} 
          onNavigate={handlePathClick} 
        />
      </Box>

      <Box sx={{ flex: 1, minHeight: 0 }}>
        <ResizablePanelGroup direction="horizontal">
          <ResizablePanel defaultSize={60} minSize={30}>
            <FileTable />
          </ResizablePanel>
          
          <ResizableHandle withHandle />
          
          <ResizablePanel defaultSize={40} minSize={20}>
            <DetailPanel />
          </ResizablePanel>
        </ResizablePanelGroup>
      </Box>
    </Box>
  )
}
