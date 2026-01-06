import { useCallback } from 'react'
import { 
  InsertDriveFileOutlined as File, 
  Folder as Folder, 
  ChatBubbleOutline as MessageSquare 
} from '@mui/icons-material'
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableContainer, 
  TableHead, 
  TableRow,
  Box,
  Typography,
  Chip
} from '@mui/material'
import { useActiveWorktree } from '@/hooks/useAppState'
import type { FileEntry, GitFileStatus } from '@/types/state'

export function FileTable() {
  const { worktree, dispatch } = useActiveWorktree()
  const explorer = worktree?.explorer
  const entries = explorer?.entries ?? []
  const selectedPath = explorer?.selected_path

  const handleRowClick = useCallback((path: string) => {
    dispatch({ type: 'SelectFile', payload: { path } })
  }, [dispatch])

  const handleRowDoubleClick = useCallback((entry: FileEntry) => {
    if (entry.kind === 'directory') {
      dispatch({ type: 'ExploreDir', payload: { path: entry.path } })
    }
  }, [dispatch])

  const getGitColor = (status?: GitFileStatus) => {
    switch (status) {
      case 'modified': return '#E3B341' // M3 Yellow
      case 'added': return '#81C784'    // M3 Green
      case 'untracked': return '#64B5F6' // M3 Blue
      case 'deleted': return '#E57373'   // M3 Red
      case 'ignored': return 'rgba(255,255,255,0.38)'
      default: return 'inherit'
    }
  }

  const formatSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  return (
    <TableContainer sx={{ height: '100%', overflow: 'auto' }}>
      <Table stickyHeader size="small" aria-label="file table">
        <TableHead>
          <TableRow>
            <TableCell sx={{ width: '40%', py: 1.5 }}>Name</TableCell>
            <TableCell sx={{ width: '15%' }}>Size</TableCell>
            <TableCell sx={{ width: '20%' }}>Status</TableCell>
            <TableCell align="right" sx={{ width: '25%' }}>Modified</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {entries.map((entry) => {
            const isSelected = selectedPath === entry.path
            return (
              <TableRow 
                key={entry.path}
                hover
                selected={isSelected}
                onClick={() => handleRowClick(entry.path)}
                onDoubleClick={() => handleRowDoubleClick(entry)}
                sx={{ 
                  cursor: 'pointer',
                  '&.Mui-selected': {
                    bgcolor: 'rgba(208, 188, 255, 0.12)', // PrimaryContainer alpha
                  },
                  '&.Mui-selected:hover': {
                    bgcolor: 'rgba(208, 188, 255, 0.16)',
                  }
                }}
              >
                <TableCell sx={{ py: 1 }}>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
                    {entry.kind === 'directory' ? (
                      <Folder sx={{ fontSize: 18, color: '#D0BCFF' }} />
                    ) : (
                      <File sx={{ fontSize: 18, color: 'text.secondary' }} />
                    )}
                    <Typography 
                      variant="body2" 
                      noWrap 
                      sx={{ 
                        color: getGitColor(entry.git_status),
                        fontWeight: isSelected ? 500 : 400
                      }}
                    >
                      {entry.name}
                    </Typography>
                    {entry.comment_count > 0 && (
                      <Box sx={{ 
                        display: 'flex', 
                        alignItems: 'center', 
                        gap: 0.5, 
                        px: 0.5, 
                        borderRadius: 1, 
                        bgcolor: 'action.selected',
                        border: 1,
                        borderColor: 'divider'
                      }}>
                        <MessageSquare sx={{ fontSize: 10, color: 'text.secondary' }} />
                        <Typography sx={{ fontSize: 10, color: 'text.secondary' }}>
                          {entry.comment_count}
                        </Typography>
                      </Box>
                    )}
                  </Box>
                </TableCell>
                <TableCell sx={{ py: 1 }}>
                  <Typography variant="caption" color="text.secondary">
                    {entry.kind === 'file' ? formatSize(entry.size) : '--'}
                  </Typography>
                </TableCell>
                <TableCell sx={{ py: 1 }}>
                  {entry.git_status && entry.git_status !== 'clean' && (
                    <Chip 
                      label={entry.git_status} 
                      size="small" 
                      variant="outlined"
                      sx={{ 
                        height: 18, 
                        fontSize: '9px', 
                        textTransform: 'uppercase',
                        color: getGitColor(entry.git_status),
                        borderColor: getGitColor(entry.git_status),
                        opacity: 0.8
                      }} 
                    />
                  )}
                </TableCell>
                <TableCell align="right" sx={{ py: 1 }}>
                  <Typography variant="caption" color="text.secondary" sx={{ fontSize: 10 }}>
                    {new Date(entry.updated_at).toLocaleDateString()}
                  </Typography>
                </TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </TableContainer>
  )
}
