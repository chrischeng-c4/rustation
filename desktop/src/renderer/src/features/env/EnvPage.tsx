import { useState, useCallback } from 'react'
import {
  ContentCopy as CopyIcon,
  Refresh as RefreshIcon,
  Settings as SettingsIcon,
  ArrowForward as ArrowRightIcon
} from '@mui/icons-material'
import {
  Button,
  Card,
  CardContent,
  Box,
  Typography,
  Stack,
  Paper,
  Divider,
  CircularProgress
} from '@mui/material'
import { EnvPatternList } from './EnvPatternList'
import { WorktreeSelector } from './WorktreeSelector'
import { EnvCopyHistory } from './EnvCopyHistory'
import { useEnvState } from '@/hooks/useAppState'
import { PageHeader } from '@/components/shared/PageHeader'

/**
 * Environment Management Page.
 */
export function EnvPage() {
  const { envConfig, project, worktrees, dispatch, isLoading } = useEnvState()

  // Local UI state for source/target selection
  const [selectedSource, setSelectedSource] = useState<string | null>(null)
  const [selectedTarget, setSelectedTarget] = useState<string | null>(null)
  const [isCopying, setIsCopying] = useState(false)

  // Use configured source or first worktree as default
  const effectiveSource =
    selectedSource ?? envConfig?.source_worktree ?? worktrees[0]?.path ?? null

  const handleAutoCopyToggle = useCallback(async () => {
    if (!envConfig) return
    await dispatch({
      type: 'SetEnvAutoCopy',
      payload: { enabled: !envConfig.auto_copy_enabled },
    })
  }, [envConfig, dispatch])

  const handlePatternsChange = useCallback(
    async (patterns: string[]) => {
      await dispatch({
        type: 'SetEnvTrackedPatterns',
        payload: { patterns },
      })
    },
    [dispatch]
  )

  const handleCopyEnvFiles = useCallback(async () => {
    if (!effectiveSource || !selectedTarget) return

    setIsCopying(true)
    try {
      await dispatch({
        type: 'CopyEnvFiles',
        payload: {
          from_worktree_path: effectiveSource,
          to_worktree_path: selectedTarget,
        },
      })
    } finally {
      setIsCopying(false)
    }
  }, [effectiveSource, selectedTarget, dispatch])

  // Loading state
  if (isLoading) {
    return (
      <Box sx={{ display: 'flex', height: '100%', alignItems: 'center', justifyContent: 'center' }}>
        <CircularProgress color="primary" />
      </Box>
    )
  }

  // No project open
  if (!project || !envConfig) {
    return (
      <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', p: 3 }}>
        <SettingsIcon sx={{ fontSize: 64, color: 'text.disabled', mb: 2, opacity: 0.5 }} />
        <Typography variant="h5" fontWeight={600}>No Project Open</Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
          Open a project to manage environment files.
        </Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100%', overflow: 'auto', p: 3 }}>
      <Stack spacing={3}>
        {/* Header */}
        <PageHeader
          title="Environment"
          description={`Sync dotfiles across worktrees in ${project.name}`}
          icon={<SettingsIcon />}
        >
          <Button
            variant={envConfig.auto_copy_enabled ? 'contained' : 'outlined'}
            onClick={handleAutoCopyToggle}
            sx={{ borderRadius: 2 }}
          >
            Auto-Copy: {envConfig.auto_copy_enabled ? 'ON' : 'OFF'}
          </Button>
        </PageHeader>

        {/* Manual Sync Card */}
        <Card variant="outlined" sx={{ borderRadius: 4 }}>
          <CardContent sx={{ p: 3 }}>
            <Stack direction="row" alignItems="center" spacing={1} sx={{ mb: 3 }}>
              <CopyIcon fontSize="small" color="primary" />
              <Typography variant="h6" fontWeight={600}>Manual Sync</Typography>
            </Stack>

            <Stack direction="row" spacing={3} alignItems="flex-end" sx={{ mb: 3 }}>
              <Box sx={{ flex: 1 }}>
                <WorktreeSelector
                  worktrees={worktrees}
                  value={effectiveSource}
                  onChange={setSelectedSource}
                  label="Source"
                  placeholder="Select source worktree"
                />
              </Box>

              <Box sx={{ pb: 1 }}>
                <ArrowRightIcon sx={{ color: 'text.disabled' }} />
              </Box>

              <Box sx={{ flex: 1 }}>
                <WorktreeSelector
                  worktrees={worktrees}
                  value={selectedTarget}
                  onChange={setSelectedTarget}
                  label="Target"
                  excludePaths={effectiveSource ? [effectiveSource] : []}
                  placeholder="Select target worktree"
                />
              </Box>
            </Stack>

            <Divider sx={{ my: 2 }} />

            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <Box>
                <Typography variant="caption" color="text.secondary" display="block">Tracked Files:</Typography>
                <Typography variant="body2" fontWeight={500}>
                  {envConfig.tracked_patterns.join(', ') || 'None configured'}
                </Typography>
              </Box>
              <Button
                variant="contained"
                onClick={handleCopyEnvFiles}
                disabled={!effectiveSource || !selectedTarget || isCopying}
                startIcon={isCopying ? <RefreshIcon sx={{ animation: 'spin 2s linear infinite' }} /> : <CopyIcon />}
                sx={{ borderRadius: 2 }}
              >
                {isCopying ? 'Copying...' : 'Copy Now'}
              </Button>
            </Box>
          </CardContent>
        </Card>

        {/* Configuration Card */}
        <Card variant="outlined" sx={{ borderRadius: 4 }}>
          <CardContent sx={{ p: 3 }}>
            <Stack direction="row" alignItems="center" spacing={1} sx={{ mb: 2 }}>
              <SettingsIcon fontSize="small" color="primary" />
              <Typography variant="h6" fontWeight={600}>Configuration</Typography>
            </Stack>

            <EnvPatternList
              patterns={envConfig.tracked_patterns}
              onPatternsChange={handlePatternsChange}
            />
          </CardContent>
        </Card>

        {/* Recent Activity Card */}
        <Card variant="outlined" sx={{ borderRadius: 4 }}>
          <CardContent sx={{ p: 3 }}>
            <Typography variant="h6" fontWeight={600} sx={{ mb: 2 }}>Recent Activity</Typography>
            <EnvCopyHistory lastResult={envConfig.last_copy_result} />
          </CardContent>
        </Card>
      </Stack>
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </Box>
  )
}
