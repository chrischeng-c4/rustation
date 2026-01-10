import { useCallback, useRef, useEffect, useState } from 'react'
import {
  ChatBubbleOutline as CommentIcon,
  Circle as CircleIcon,
  Add as AddIcon,
  Remove as RemoveIcon,
  QuestionMark as QuestionIcon
} from '@mui/icons-material'
import { FileIcon } from '@/components/shared/FileIcon'
import {
  Box,
  Typography,
  Tooltip,
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

  // Git status color mapping (IDE-style)
  const getGitStatusInfo = (status?: GitFileStatus): { color: string; icon: React.ReactNode; label: string } | null => {
    switch (status) {
      case 'modified':
        return {
          color: '#E3B341', // Yellow/Orange
          icon: <CircleIcon sx={{ fontSize: 8 }} />,
          label: 'Modified'
        }
      case 'added':
        return {
          color: '#81C784', // Green
          icon: <AddIcon sx={{ fontSize: 10, strokeWidth: 2 }} />,
          label: 'Added'
        }
      case 'untracked':
        return {
          color: '#64B5F6', // Blue
          icon: <QuestionIcon sx={{ fontSize: 10 }} />,
          label: 'Untracked'
        }
      case 'deleted':
        return {
          color: '#E57373', // Red
          icon: <RemoveIcon sx={{ fontSize: 10 }} />,
          label: 'Deleted'
        }
      case 'ignored':
        return {
          color: theme.palette.text.disabled,
          icon: null,
          label: 'Ignored'
        }
      default:
        return null
    }
  }

  // Get text color for file name based on git status
  const getNameColor = (status?: GitFileStatus) => {
    const info = getGitStatusInfo(status)
    return info?.color || 'inherit'
  }

  const RowComponent = ({
    index,
    style,
    ariaAttributes,
    entries: rowEntries,
    selectedPath: rowSelectedPath,
    theme: rowTheme,
    getNameColor: rowGetNameColor,
    getGitStatusInfo: rowGetGitStatusInfo,
    handleRowClick: rowHandleRowClick,
    handleRowDoubleClick: rowHandleRowDoubleClick
  }: any) => {
    const entry = rowEntries[index]
    const isSelected = rowSelectedPath === entry.path
    const gitInfo = rowGetGitStatusInfo(entry.git_status)

    return (
      <Box
        style={style}
        onClick={() => rowHandleRowClick(entry.path)}
        onDoubleClick={() => rowHandleRowDoubleClick(entry)}
        {...ariaAttributes}
        sx={{
          display: 'flex',
          alignItems: 'center',
          px: 1.5,
          gap: 0.5,
          cursor: 'pointer',
          bgcolor: isSelected ? 'rgba(208, 188, 255, 0.12)' : 'transparent',
          '&:hover': {
            bgcolor: isSelected ? 'rgba(208, 188, 255, 0.16)' : 'action.hover',
          }
        }}
      >
        {/* Git status indicator - left side */}
        <Box sx={{ width: 16, display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
          {gitInfo?.icon && (
            <Tooltip title={gitInfo.label} placement="left">
              <Box sx={{ color: gitInfo.color, display: 'flex', alignItems: 'center' }}>
                {gitInfo.icon}
              </Box>
            </Tooltip>
          )}
        </Box>

        {/* File/folder icon */}
        <FileIcon filename={entry.name} kind={entry.kind} size={18} />

        {/* File name */}
        <Typography
          variant="body2"
          noWrap
          sx={{
            flex: 1,
            color: rowGetNameColor(entry.git_status),
            fontWeight: isSelected ? 600 : 400,
            fontSize: '0.8125rem'
          }}
        >
          {entry.name}
        </Typography>

        {/* Comment indicator - right side */}
        {entry.comment_count > 0 && (
          <Tooltip title={`${entry.comment_count} comment${entry.comment_count > 1 ? 's' : ''}`}>
            <Box sx={{
              display: 'flex',
              alignItems: 'center',
              gap: 0.25,
              color: 'text.secondary',
              flexShrink: 0
            }}>
              <CommentIcon sx={{ fontSize: 12 }} />
              <Typography sx={{ fontSize: 10 }}>
                {entry.comment_count}
              </Typography>
            </Box>
          </Tooltip>
        )}
      </Box>
    )
  }

  return (
    <Box ref={containerRef} sx={{ height: '100%', width: '100%', overflow: 'hidden' }}>
      {/* Simple header - no columns, just file count */}
      <Box sx={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        px: 1.5,
        py: 1,
        borderBottom: 1,
        borderColor: 'divider',
        bgcolor: 'background.paper',
      }}>
        <Typography variant="caption" fontWeight={600} color="text.secondary">
          Files
        </Typography>
        <Typography variant="caption" color="text.disabled">
          {entries.length} items
        </Typography>
      </Box>

      {height > 0 && (
        <List
          rowComponent={RowComponent}
          rowCount={entries.length}
          rowHeight={32}
          rowProps={{
            entries,
            selectedPath,
            theme,
            getNameColor,
            getGitStatusInfo,
            handleRowClick,
            handleRowDoubleClick
          }}
          style={{ height: height - 36 }}
        />
      )}
    </Box>
  )
}
