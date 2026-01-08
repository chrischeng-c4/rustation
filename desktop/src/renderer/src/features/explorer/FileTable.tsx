import { useCallback, useRef, useEffect, useState } from 'react'
import { 
  InsertDriveFileOutlined as File, 
  Folder as Folder, 
  ChatBubbleOutline as MessageSquare 
} from '@mui/icons-material'
import { 
  Box,
  Typography,
  Chip,
  useTheme
} from '@mui/material'
import { List } from 'react-window'
import { useActiveWorktree } from '@/hooks/useAppState'
import type { FileEntry, GitFileStatus } from '@/types/state'

export function FileTable() {
  const { worktree, dispatch } = useActiveWorktree()
  const theme = useTheme()
  const explorer = worktree?.explorer
  const entries = explorer?.entries ?? []
  const selectedPath = explorer?.selected_path

  const containerRef = useRef<HTMLDivElement>(null)
  const [height, setHeight] = useState(0)

  useEffect(() => {
    if (!containerRef.current) return

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setHeight(entry.contentRect.height)
      }
    })

    observer.observe(containerRef.current)
    return () => observer.disconnect()
  }, [])

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
      case 'ignored': return theme.palette.text.disabled
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

  const RowComponent = ({
    index,
    style,
    ariaAttributes,
    entries: rowEntries,
    selectedPath: rowSelectedPath,
    theme: rowTheme,
    getGitColor: rowGetGitColor,
    formatSize: rowFormatSize,
    handleRowClick: rowHandleRowClick,
    handleRowDoubleClick: rowHandleRowDoubleClick
  }: any) => {
    const entry = rowEntries[index]
    const isSelected = rowSelectedPath === entry.path

    return (
      <Box
        style={style}
        onClick={() => rowHandleRowClick(entry.path)}
        onDoubleClick={() => rowHandleRowDoubleClick(entry)}
        {...ariaAttributes}
        sx={{
          display: 'flex',
          alignItems: 'center',
          px: 2,
          cursor: 'pointer',
          borderBottom: 1,
          borderColor: 'divider',
          bgcolor: isSelected ? 'rgba(208, 188, 255, 0.12)' : 'transparent',
          '&:hover': {
            bgcolor: isSelected ? 'rgba(208, 188, 255, 0.16)' : 'action.hover',
          }
        }}
      >
        <Box sx={{ width: '40%', display: 'flex', alignItems: 'center', gap: 1.5, overflow: 'hidden' }}>
          {entry.kind === 'directory' ? (
            <Folder sx={{ fontSize: 18, color: rowTheme.palette.primary.light, flexShrink: 0 }} />
          ) : (
            <File sx={{ fontSize: 18, color: 'text.secondary', flexShrink: 0 }} />
          )}
          <Typography
            variant="body2"
            noWrap
            sx={{
              color: rowGetGitColor(entry.git_status),
              fontWeight: isSelected ? 600 : 400
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
              borderColor: 'divider',
              flexShrink: 0
            }}>
              <MessageSquare sx={{ fontSize: 10, color: 'text.secondary' }} />
              <Typography sx={{ fontSize: 10, color: 'text.secondary' }}>
                {entry.comment_count}
              </Typography>
            </Box>
          )}
        </Box>

        <Box sx={{ width: '15%', px: 1 }}>
          <Typography variant="caption" color="text.secondary" noWrap>
            {entry.kind === 'file' ? rowFormatSize(entry.size) : '--'}
          </Typography>
        </Box>

        <Box sx={{ width: '20%', px: 1 }}>
          {entry.git_status && entry.git_status !== 'clean' && (
            <Chip
              label={entry.git_status}
              size="small"
              variant="outlined"
              sx={{
                height: 18,
                fontSize: '9px',
                textTransform: 'uppercase',
                color: rowGetGitColor(entry.git_status),
                borderColor: rowGetGitColor(entry.git_status),
                opacity: 0.8
              }}
            />
          )}
        </Box>

        <Box sx={{ width: '25%', textAlign: 'right' }}>
          <Typography variant="caption" color="text.secondary" sx={{ fontSize: 10 }}>
            {new Date(entry.updated_at).toLocaleDateString()}
          </Typography>
        </Box>
      </Box>
    )
  }

  return (
    <Box ref={containerRef} sx={{ height: '100%', width: '100%', overflow: 'hidden' }}>
      <Box sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        px: 2, 
        py: 1.5, 
        borderBottom: 1, 
        borderColor: 'divider',
        bgcolor: 'background.paper',
        position: 'sticky',
        top: 0,
        zIndex: 1
      }}>
        <Typography variant="caption" fontWeight={700} sx={{ width: '40%' }}>Name</Typography>
        <Typography variant="caption" fontWeight={700} sx={{ width: '15%', px: 1 }}>Size</Typography>
        <Typography variant="caption" fontWeight={700} sx={{ width: '20%', px: 1 }}>Status</Typography>
        <Typography variant="caption" fontWeight={700} sx={{ width: '25%', textAlign: 'right' }}>Modified</Typography>
      </Box>
      
      {height > 0 && (
        <List
          rowComponent={RowComponent}
          rowCount={entries.length}
          rowHeight={40}
          rowProps={{
            entries,
            selectedPath,
            theme,
            getGitColor,
            formatSize,
            handleRowClick,
            handleRowDoubleClick
          }}
          style={{ height: height - 40 }}
        />
      )}
    </Box>
  )
}
