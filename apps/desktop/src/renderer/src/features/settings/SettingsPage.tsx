import { useCallback, useEffect } from 'react'
import { Sun, Moon, Monitor, FolderOpen } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useSettingsState } from '@/hooks/useAppState'
import type { Theme } from '@/types/state'

/**
 * Apply theme to the document root element.
 * Uses Tailwind's dark mode class strategy.
 */
function applyTheme(theme: Theme) {
  const root = document.documentElement

  if (theme === 'dark') {
    root.classList.add('dark')
  } else if (theme === 'light') {
    root.classList.remove('dark')
  } else {
    // System preference
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    if (prefersDark) {
      root.classList.add('dark')
    } else {
      root.classList.remove('dark')
    }
  }
}

/**
 * Settings Page - Global and Worktree configuration.
 */
export function SettingsPage() {
  const { settings, dispatch, isLoading } = useSettingsState()

  // Apply theme on mount and when it changes
  useEffect(() => {
    if (settings?.theme) {
      applyTheme(settings.theme)
    }
  }, [settings?.theme])

  // Listen for system theme changes when "system" is selected
  useEffect(() => {
    if (settings?.theme !== 'system') return

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = () => applyTheme('system')

    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [settings?.theme])

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
      <div className="flex h-full items-center justify-center">
        <p className="text-muted-foreground">Loading settings...</p>
      </div>
    )
  }

  const currentTheme = settings.theme

  return (
    <ScrollArea className="h-full">
      <div className="space-y-6 p-4">
        {/* Header */}
        <div>
          <h2 className="text-2xl font-semibold">Settings</h2>
          <p className="mt-1 text-muted-foreground">Configure application preferences</p>
        </div>

        {/* Appearance Card */}
        <Card className="p-4">
          <h3 className="mb-4 text-lg font-medium">Appearance</h3>

          <div className="space-y-4">
            <div>
              <Label className="text-sm font-medium">Theme</Label>
              <p className="text-xs text-muted-foreground mb-3">
                Choose how the application looks
              </p>

              <div className="flex gap-2">
                <Button
                  variant={currentTheme === 'system' ? 'default' : 'outline'}
                  className="flex-1 gap-2"
                  onClick={() => handleThemeChange('system')}
                >
                  <Monitor className="h-4 w-4" />
                  System
                </Button>
                <Button
                  variant={currentTheme === 'light' ? 'default' : 'outline'}
                  className="flex-1 gap-2"
                  onClick={() => handleThemeChange('light')}
                >
                  <Sun className="h-4 w-4" />
                  Light
                </Button>
                <Button
                  variant={currentTheme === 'dark' ? 'default' : 'outline'}
                  className="flex-1 gap-2"
                  onClick={() => handleThemeChange('dark')}
                >
                  <Moon className="h-4 w-4" />
                  Dark
                </Button>
              </div>
            </div>
          </div>
        </Card>

        {/* Projects Card */}
        <Card className="p-4">
          <h3 className="mb-4 text-lg font-medium">Projects</h3>

          <div className="space-y-4">
            <div>
              <Label htmlFor="default-path" className="text-sm font-medium">
                Default Project Path
              </Label>
              <p className="text-xs text-muted-foreground mb-2">
                Starting directory when opening new projects
              </p>

              <div className="flex gap-2">
                <Input
                  id="default-path"
                  value={settings.default_project_path ?? ''}
                  readOnly
                  placeholder="No default path set"
                  className="font-mono text-sm"
                />
                <Button variant="outline" onClick={handleBrowseProjectPath}>
                  <FolderOpen className="h-4 w-4 mr-2" />
                  Browse
                </Button>
                {settings.default_project_path && (
                  <Button variant="ghost" onClick={handleClearProjectPath}>
                    Clear
                  </Button>
                )}
              </div>
            </div>
          </div>
        </Card>

        {/* About Card */}
        <Card className="p-4">
          <h3 className="mb-4 text-lg font-medium">About</h3>

          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Application</span>
              <span>rstn</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Version</span>
              <span className="font-mono">0.1.0</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Framework</span>
              <span>Electron + React + napi-rs</span>
            </div>
          </div>
        </Card>
      </div>
    </ScrollArea>
  )
}
