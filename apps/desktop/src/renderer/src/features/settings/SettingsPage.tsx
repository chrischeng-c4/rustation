import { useCallback } from 'react'
import { Box, Button, Paper, Stack, TextField, Typography } from '@mui/material'
import { Brightness4, Brightness7, DesktopWindows, FolderOpen } from '@mui/icons-material'
import { useSettingsState } from '@/hooks/useAppState'
import type { Theme } from '@/types/state'

/**
 * Settings Page - Global and Worktree configuration.
 */
export function SettingsPage() {
  const { settings, dispatch, isLoading } = useSettingsState()

  const handleThemeChange = useCallback(
    async (theme: Theme) => {
      await dispatch({ type: 'SetTheme', payload: { theme } })
    },
    [dispatch]
  )

  const handleBrowseProjectPath = useCallback(async () => {
    const path = await window.dialogApi.openFolder()
    if (path) {
      await dispatch({ type: 'SetProjectPath', payload: { path } })
    }
  }, [dispatch])

  const handleClearProjectPath = useCallback(async () => {
    await dispatch({ type: 'SetProjectPath', payload: { path: null } })
  }, [dispatch])

  if (isLoading || !settings) {
    return (
      <Stack alignItems="center" justifyContent="center" sx={{ height: '100%' }}>
        <Typography variant="body2" color="text.secondary">
          Loading settings...
        </Typography>
      </Stack>
    )
  }

  const currentTheme = settings.theme

  return (
    <Box sx={{ height: '100%', overflow: 'auto' }}>
      <Stack spacing={3} sx={{ p: 3 }}>
        {/* Header */}
        <Box>
          <Typography variant="h5" fontWeight={600}>Settings</Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
            Configure application preferences
          </Typography>
        </Box>

        {/* Appearance Card */}
        <Paper variant="outlined" sx={{ p: 3 }}>
          <Typography variant="h6" fontWeight={600} sx={{ mb: 2 }}>
            Appearance
          </Typography>

          <Box>
            <Typography variant="subtitle2">Theme</Typography>
            <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 2 }}>
              Choose how the application looks
            </Typography>

            <Stack direction="row" spacing={2}>
              <Button
                variant={currentTheme === 'system' ? 'contained' : 'outlined'}
                onClick={() => handleThemeChange('system')}
                startIcon={<DesktopWindows fontSize="small" />}
                sx={{ flex: 1 }}
              >
                System
              </Button>
              <Button
                variant={currentTheme === 'light' ? 'contained' : 'outlined'}
                onClick={() => handleThemeChange('light')}
                startIcon={<Brightness7 fontSize="small" />}
                sx={{ flex: 1 }}
              >
                Light
              </Button>
              <Button
                variant={currentTheme === 'dark' ? 'contained' : 'outlined'}
                onClick={() => handleThemeChange('dark')}
                startIcon={<Brightness4 fontSize="small" />}
                sx={{ flex: 1 }}
              >
                Dark
              </Button>
            </Stack>
          </Box>
        </Paper>

        {/* Projects Card */}
        <Paper variant="outlined" sx={{ p: 3 }}>
          <Typography variant="h6" fontWeight={600} sx={{ mb: 2 }}>
            Projects
          </Typography>

          <Box>
            <Typography variant="subtitle2">Default Project Path</Typography>
            <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 1.5 }}>
              Starting directory when opening new projects
            </Typography>

            <Stack direction="row" spacing={2}>
              <TextField
                id="default-path"
                value={settings.default_project_path ?? ''}
                placeholder="No default path set"
                size="small"
                fullWidth
                InputProps={{ readOnly: true }}
              />
              <Button variant="outlined" onClick={handleBrowseProjectPath} startIcon={<FolderOpen fontSize="small" />}>
                Browse
              </Button>
              {settings.default_project_path && (
                <Button variant="text" onClick={handleClearProjectPath}>
                  Clear
                </Button>
              )}
            </Stack>
          </Box>
        </Paper>

        {/* About Card */}
        <Paper variant="outlined" sx={{ p: 3 }}>
          <Typography variant="h6" fontWeight={600} sx={{ mb: 2 }}>
            About
          </Typography>

          <Stack spacing={1}>
            <Stack direction="row" justifyContent="space-between">
              <Typography variant="body2" color="text.secondary">Application</Typography>
              <Typography variant="body2">rstn</Typography>
            </Stack>
            <Stack direction="row" justifyContent="space-between">
              <Typography variant="body2" color="text.secondary">Version</Typography>
              <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>0.1.0</Typography>
            </Stack>
            <Stack direction="row" justifyContent="space-between">
              <Typography variant="body2" color="text.secondary">Framework</Typography>
              <Typography variant="body2">Electron + React + napi-rs</Typography>
            </Stack>
          </Stack>
        </Paper>
      </Stack>
    </Box>
  )
}
