import { useCallback, useState, useEffect, useMemo } from 'react'
import {
  ChatBubbleOutline as CommentIcon,
  Circle as CircleIcon,
  Add as AddIcon,
  Remove as RemoveIcon,
  QuestionMark as QuestionIcon,
  ChevronRight as ChevronRightIcon,
  ExpandMore as ExpandMoreIcon,
  Home as HomeIcon
} from '@mui/icons-material'
import { FileIcon } from '@/components/shared/FileIcon'
import {
  Box,
  Typography,
  Tooltip,
  useTheme,
  IconButton,
  Collapse,
  CircularProgress
} from '@mui/material'
import { useActiveWorktree } from '@/hooks/useAppState'
import type { FileEntry, GitFileStatus } from '@/types/state'

/** Cached directory entries for expanded folders */
interface DirectoryCache {
  entries: FileEntry[]
  isLoading: boolean
  error?: string
}


export function FileTreeView() {
  const { worktree, dispatch } = useActiveWorktree()
  const theme = useTheme()
  const explorer = worktree?.explorer
  const rootPath = worktree?.path ?? ''
  const currentPath = explorer?.current_path ?? rootPath
  const entries = explorer?.entries ?? []
  const selectedPath = explorer?.selected_path

  // Track expanded folders and their cached entries
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set())
  const [directoryCache, setDirectoryCache] = useState<Map<string, DirectoryCache>>(new Map())
  const [loadingPaths, setLoadingPaths] = useState<Set<string>>(new Set())

  // Initialize: expand current path and cache its entries
  useEffect(() => {
    if (currentPath && entries.length > 0) {
      setDirectoryCache(prev => {
        const next = new Map(prev)
        next.set(currentPath, { entries, isLoading: false })
        return next
      })
      // Auto-expand current path
      setExpandedPaths(prev => new Set([...prev, currentPath]))
    }
  }, [currentPath, entries])

  // Load directory entries for a path
  const loadDirectory = useCallback(async (path: string) => {
    if (loadingPaths.has(path)) return

    setLoadingPaths(prev => new Set([...prev, path]))
    setDirectoryCache(prev => {
      const next = new Map(prev)
      next.set(path, { entries: [], isLoading: true })
      return next
    })

    try {
      // Use the backend to load directory without mutating state
      const result = await window.explorerApi.listDirectory(path, rootPath)
      if (result) {
        // Convert API response to FileEntry format
        const entries: FileEntry[] = result.map((e) => ({
          name: e.name,
          path: e.path,
          kind: e.kind as 'file' | 'directory' | 'symlink',
          size: e.size,
          permissions: e.permissions,
          updated_at: e.updated_at,
          comment_count: e.comment_count,
          git_status: e.git_status as GitFileStatus | undefined
        }))
        setDirectoryCache(prev => {
          const next = new Map(prev)
          next.set(path, { entries, isLoading: false })
          return next
        })
      }
    } catch (error) {
      setDirectoryCache(prev => {
        const next = new Map(prev)
        next.set(path, { entries: [], isLoading: false, error: String(error) })
        return next
      })
    } finally {
      setLoadingPaths(prev => {
        const next = new Set(prev)
        next.delete(path)
        return next
      })
    }
  }, [loadingPaths, rootPath])

  // Toggle folder expansion (for row click)
  const toggleExpand = useCallback((path: string) => {
    setExpandedPaths(prev => {
      const next = new Set(prev)
      if (next.has(path)) {
        next.delete(path)
      } else {
        next.add(path)
        // Load directory if not cached
        if (!directoryCache.has(path)) {
          loadDirectory(path)
        }
      }
      return next
    })
  }, [directoryCache, loadDirectory])

  // Toggle folder expansion (for arrow button click, stops propagation)
  const handleToggleExpand = useCallback((path: string, e: React.MouseEvent) => {
    e.stopPropagation()
    toggleExpand(path)
  }, [toggleExpand])

  // Select file/folder - for files, also open in tab
  const handleSelect = useCallback((entry: FileEntry) => {
    dispatch({ type: 'SelectFile', payload: { path: entry.path } })
    // For files, also open in a tab (preview mode)
    if (entry.kind === 'file') {
      dispatch({ type: 'OpenFileTab', payload: { path: entry.path } })
    }
  }, [dispatch])

  // Pin a file tab (double-click behavior)
  const handlePinTab = useCallback((entry: FileEntry) => {
    if (entry.kind === 'file') {
      dispatch({ type: 'PinTab', payload: { path: entry.path } })
    }
  }, [dispatch])

  // Navigate to root
  const handleGoToRoot = useCallback(() => {
    dispatch({ type: 'ExploreDir', payload: { path: rootPath } })
  }, [dispatch, rootPath])

  // Git status helpers
  const getGitStatusInfo = useCallback((status?: GitFileStatus): { color: string; icon: React.ReactNode; label: string } | null => {
    switch (status) {
      case 'modified':
        return {
          color: '#E3B341',
          icon: <CircleIcon sx={{ fontSize: 6 }} />,
          label: 'Modified'
        }
      case 'added':
        return {
          color: '#81C784',
          icon: <AddIcon sx={{ fontSize: 8 }} />,
          label: 'Added'
        }
      case 'untracked':
        return {
          color: '#64B5F6',
          icon: <QuestionIcon sx={{ fontSize: 8 }} />,
          label: 'Untracked'
        }
      case 'deleted':
        return {
          color: '#E57373',
          icon: <RemoveIcon sx={{ fontSize: 8 }} />,
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
  }, [theme.palette.text.disabled])

  const getNameColor = useCallback((status?: GitFileStatus) => {
    const info = getGitStatusInfo(status)
    return info?.color || 'inherit'
  }, [getGitStatusInfo])

  // Build tree structure from root
  const rootEntries = useMemo(() => {
    return directoryCache.get(currentPath)?.entries ?? entries
  }, [directoryCache, currentPath, entries])

  // Render a single tree item
  const renderTreeItem = useCallback((entry: FileEntry, depth: number) => {
    const isDirectory = entry.kind === 'directory'
    const isExpanded = expandedPaths.has(entry.path)
    const isSelected = selectedPath === entry.path
    const gitInfo = getGitStatusInfo(entry.git_status)
    const cached = directoryCache.get(entry.path)
    const isLoading = loadingPaths.has(entry.path)
    const childEntries = cached?.entries ?? []
    const indent = depth * 16

    return (
      <Box key={entry.path}>
        {/* Tree item row */}
        <Box
          onClick={() => {
            handleSelect(entry)
            // For directories, also toggle expansion
            if (isDirectory) {
              toggleExpand(entry.path)
            }
          }}
          onDoubleClick={() => handlePinTab(entry)}
          sx={{
            display: 'flex',
            alignItems: 'center',
            height: 24,
            pl: `${indent + 4}px`,
            pr: 1,
            gap: 0.25,
            cursor: 'pointer',
            bgcolor: isSelected ? 'rgba(208, 188, 255, 0.12)' : 'transparent',
            '&:hover': {
              bgcolor: isSelected ? 'rgba(208, 188, 255, 0.16)' : 'action.hover',
            }
          }}
        >
          {/* Expand/collapse arrow for directories */}
          <Box
            sx={{
              width: 16,
              height: 16,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0
            }}
          >
            {isDirectory && (
              <IconButton
                size="small"
                onClick={(e) => handleToggleExpand(entry.path, e)}
                sx={{ p: 0, width: 16, height: 16 }}
              >
                {isLoading ? (
                  <CircularProgress size={10} />
                ) : isExpanded ? (
                  <ExpandMoreIcon sx={{ fontSize: 14 }} />
                ) : (
                  <ChevronRightIcon sx={{ fontSize: 14 }} />
                )}
              </IconButton>
            )}
          </Box>

          {/* Git status indicator */}
          <Box sx={{ width: 12, display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
            {gitInfo?.icon && (
              <Tooltip title={gitInfo.label} placement="left">
                <Box sx={{ color: gitInfo.color, display: 'flex', alignItems: 'center' }}>
                  {gitInfo.icon}
                </Box>
              </Tooltip>
            )}
          </Box>

          {/* File/folder icon */}
          <FileIcon filename={entry.name} kind={entry.kind} isOpen={isExpanded} size={16} />

          {/* File name */}
          <Typography
            variant="body2"
            noWrap
            sx={{
              flex: 1,
              ml: 0.5,
              color: getNameColor(entry.git_status),
              fontWeight: isSelected ? 600 : 400,
              fontSize: '0.75rem'
            }}
          >
            {entry.name}
          </Typography>

          {/* Comment indicator */}
          {entry.comment_count > 0 && (
            <Tooltip title={`${entry.comment_count} comment${entry.comment_count > 1 ? 's' : ''}`}>
              <Box sx={{
                display: 'flex',
                alignItems: 'center',
                gap: 0.25,
                color: 'text.secondary',
                flexShrink: 0
              }}>
                <CommentIcon sx={{ fontSize: 10 }} />
                <Typography sx={{ fontSize: 9 }}>
                  {entry.comment_count}
                </Typography>
              </Box>
            </Tooltip>
          )}
        </Box>

        {/* Children (expanded) */}
        {isDirectory && (
          <Collapse in={isExpanded} timeout={150}>
            {childEntries.map(child => renderTreeItem(child, depth + 1))}
          </Collapse>
        )}
      </Box>
    )
  }, [
    expandedPaths,
    selectedPath,
    directoryCache,
    loadingPaths,
    getGitStatusInfo,
    getNameColor,
    handleSelect,
    handlePinTab,
    handleToggleExpand,
    toggleExpand,
    theme.palette.primary.light
  ])

  // Sort entries: directories first, then alphabetically
  const sortedEntries = useMemo(() => {
    return [...rootEntries].sort((a, b) => {
      if (a.kind === 'directory' && b.kind !== 'directory') return -1
      if (a.kind !== 'directory' && b.kind === 'directory') return 1
      return a.name.localeCompare(b.name)
    })
  }, [rootEntries])

  // Get project name from root path
  const projectName = useMemo(() => {
    return rootPath.split('/').pop() || 'Project'
  }, [rootPath])

  return (
    <Box sx={{ height: '100%', width: '100%', overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
      {/* Header with project name */}
      <Box sx={{
        display: 'flex',
        alignItems: 'center',
        gap: 1,
        px: 1,
        py: 0.75,
        borderBottom: 1,
        borderColor: 'divider',
        bgcolor: 'background.paper',
      }}>
        <IconButton size="small" onClick={handleGoToRoot} sx={{ p: 0.25 }}>
          <HomeIcon sx={{ fontSize: 14 }} />
        </IconButton>
        <Typography variant="caption" fontWeight={600} noWrap sx={{ flex: 1 }}>
          {projectName}
        </Typography>
        <Typography variant="caption" color="text.disabled">
          {rootEntries.length} items
        </Typography>
      </Box>

      {/* Tree content */}
      <Box sx={{ flex: 1, overflow: 'auto', py: 0.5 }}>
        {sortedEntries.map(entry => renderTreeItem(entry, 0))}
      </Box>
    </Box>
  )
}
