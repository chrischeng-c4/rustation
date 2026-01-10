import { useCallback, useMemo } from 'react'
import { Box, Tab, Tabs, IconButton, Typography, Tooltip } from '@mui/material'
import { Close as CloseIcon } from '@mui/icons-material'
import { FileIcon } from '@/components/shared/FileIcon'
import { useActiveWorktree } from '@/hooks/useAppState'

export function FileTabs() {
  const { worktree, dispatch } = useActiveWorktree()
  const explorer = worktree?.explorer
  const tabs = explorer?.tabs ?? []
  const activeTabPath = explorer?.active_tab_path

  // Get filename from path
  const getFileName = useCallback((path: string) => {
    return path.split('/').pop() ?? path
  }, [])

  // Handle tab switch
  const handleTabChange = useCallback((_: React.SyntheticEvent, newValue: string) => {
    dispatch({ type: 'SwitchTab', payload: { path: newValue } })
  }, [dispatch])

  // Handle tab close
  const handleCloseTab = useCallback((path: string, e: React.MouseEvent) => {
    e.stopPropagation()
    dispatch({ type: 'CloseTab', payload: { path } })
  }, [dispatch])

  // Handle double-click to pin
  const handleDoubleClick = useCallback((path: string) => {
    dispatch({ type: 'PinTab', payload: { path } })
  }, [dispatch])

  // Sort tabs: pinned tabs first, then preview tabs
  const sortedTabs = useMemo(() => {
    const pinned = tabs.filter(t => t.is_pinned)
    const preview = tabs.filter(t => !t.is_pinned)
    return [...pinned, ...preview]
  }, [tabs])

  if (tabs.length === 0) {
    return null
  }

  return (
    <Box
      sx={{
        borderBottom: 1,
        borderColor: 'divider',
        bgcolor: 'background.paper',
        minHeight: 36,
      }}
    >
      <Tabs
        value={activeTabPath ?? false}
        onChange={handleTabChange}
        variant="scrollable"
        scrollButtons="auto"
        sx={{
          minHeight: 36,
          '& .MuiTabs-indicator': {
            height: 2,
          },
          '& .MuiTab-root': {
            minHeight: 36,
            py: 0,
            px: 1.5,
            textTransform: 'none',
          },
        }}
      >
        {sortedTabs.map((tab) => (
          <Tab
            key={tab.path}
            value={tab.path}
            onDoubleClick={() => handleDoubleClick(tab.path)}
            label={
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                <FileIcon filename={getFileName(tab.path)} kind="file" size={14} />
                <Tooltip title={tab.path} placement="bottom">
                  <Typography
                    variant="body2"
                    sx={{
                      fontSize: '0.75rem',
                      fontStyle: tab.is_pinned ? 'normal' : 'italic',
                      maxWidth: 120,
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {getFileName(tab.path)}
                  </Typography>
                </Tooltip>
                <IconButton
                  size="small"
                  onClick={(e) => handleCloseTab(tab.path, e)}
                  sx={{
                    p: 0.25,
                    ml: 0.5,
                    opacity: 0.6,
                    '&:hover': { opacity: 1 },
                  }}
                >
                  <CloseIcon sx={{ fontSize: 12 }} />
                </IconButton>
              </Box>
            }
            sx={{
              '&.Mui-selected': {
                bgcolor: 'action.selected',
              },
            }}
          />
        ))}
      </Tabs>
    </Box>
  )
}
