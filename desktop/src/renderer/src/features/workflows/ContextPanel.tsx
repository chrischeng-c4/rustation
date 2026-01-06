import { useCallback, useEffect, useState } from 'react'
import {
  Description as FileTextIcon,
  Refresh as RefreshIcon,
  AccessTime as ClockIcon,
  AutoAwesome as WandIcon,
  Book as BookOpenIcon,
  Add as PlusIcon
} from '@mui/icons-material'
import {
  Button,
  Card,
  CardContent,
  Box,
  Typography,
  Tabs,
  Tab,
  Chip,
  Paper,
  Stack,
  IconButton,
  Tooltip,
  Divider
} from '@mui/material'
import { PageHeader } from '@/components/shared/PageHeader'
import { WorkflowHeader } from '@/components/shared/WorkflowHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { ErrorBanner } from '@/components/shared/ErrorBanner'
import { useAppState } from '@/hooks/useAppState'
import ReactMarkdown from 'react-markdown'

/**
 * Context viewing panel for Living Context Layer.
 */
export function ContextPanel() {
  const { state, dispatch, isLoading } = useAppState()
  const [activeTab, setActiveTab] = useState<string | null>(null)

  const activeProject = state?.projects?.[state?.active_project_index ?? 0]
  const worktree = activeProject?.worktrees?.[activeProject?.active_worktree_index ?? 0]
  const context = worktree?.context
  const contextFiles = context?.files ?? []
  const isInitialized = context?.is_initialized
  const lastRefreshed = context?.last_refreshed

  // AI Generation state
  const isGenerating = context?.is_generating ?? false
  const generationOutput = context?.generation_output ?? ''
  const generationError = context?.generation_error

  // Context Sync state
  const isSyncing = context?.is_syncing ?? false
  const syncOutput = context?.sync_output ?? ''
  const syncError = context?.sync_error

  // Initial tab selection
  useEffect(() => {
    if (contextFiles.length > 0 && !activeTab) {
      setActiveTab(contextFiles[0].name)
    }
  }, [contextFiles, activeTab])

  // Check constitution on mount
  useEffect(() => {
    dispatch({ type: 'RefreshContext' })
  }, [dispatch])

  const handleInitialize = useCallback(async () => {
    await dispatch({ type: 'InitializeContext' })
  }, [dispatch])

  const handleGenerateContext = useCallback(async () => {
    await dispatch({ type: 'GenerateContext' })
  }, [dispatch])

  const handleRefresh = useCallback(async () => {
    await dispatch({ type: 'RefreshContext' })
  }, [dispatch])

  const activeFile = contextFiles.find(f => f.name === activeTab)

  // Loading state
  if (isLoading || context?.is_loading) {
    return <LoadingState message="Loading project context..." />
  }

  // AI Generation in progress
  if (isGenerating) {
    return (
      <Stack sx={{ height: '100%' }}>
        <WorkflowHeader
          title="Generating Context"
          subtitle="Analyzing codebase"
          icon={<RefreshIcon sx={{ animation: 'spin 2s linear infinite' }} color="primary" />}
        />
        <Box sx={{ flex: 1, p: 3, pt: 0, overflow: 'auto' }}>
          <Stack spacing={2}>
            <Card variant="outlined" sx={{ bgcolor: 'background.default' }}>
              <CardContent>
                <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap', color: 'text.secondary' }}>
                  {generationOutput || 'Analyzing codebase...'}
                </Typography>
              </CardContent>
            </Card>
            {generationError && <ErrorBanner error={generationError} />}
          </Stack>
        </Box>
        <style>{`@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
      </Stack>
    )
  }

  // Context Sync in progress
  if (isSyncing) {
    return (
      <Stack sx={{ height: '100%' }}>
        <WorkflowHeader
          title="Syncing Context"
          subtitle="Extracting latest changes"
          icon={<RefreshIcon sx={{ animation: 'spin 2s linear infinite' }} color="success" />}
        />
        <Box sx={{ flex: 1, p: 3, pt: 0, overflow: 'auto' }}>
          <Stack spacing={2}>
            <Card variant="outlined" sx={{ bgcolor: 'background.default' }}>
              <CardContent>
                <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap', color: 'text.secondary' }}>
                  {syncOutput || 'Extracting context updates...'}
                </Typography>
              </CardContent>
            </Card>
            {syncError && <ErrorBanner error={syncError} />}
          </Stack>
        </Box>
        <style>{`@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
      </Stack>
    )
  }

  // Context not initialized
  if (!isInitialized) {
    return (
      <Stack sx={{ height: '100%' }}>
        <PageHeader
          title="Living Context"
          description="Source of truth for project knowledge"
          icon={<BookOpenIcon />}
        />
        <Box sx={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', p: 3 }}>
          <Paper variant="outlined" sx={{ maxWidth: 480, width: '100%', p: 4, bgcolor: 'surfaceContainerHigh.main' }}>
            <Stack spacing={3}>
              <Box>
                <Typography variant="h6" fontWeight={600} gutterBottom align="center">Initialize Living Context</Typography>
                <Typography variant="body2" color="text.secondary" align="center">
                  Living context provides AI with project knowledge including tech stack, architecture, and recent changes.
                </Typography>
              </Box>

              <Box>
                <Button variant="contained" fullWidth onClick={handleGenerateContext} startIcon={<WandIcon />}>
                  Generate with AI
                </Button>
                <Typography variant="caption" display="block" align="center" color="text.secondary" sx={{ mt: 1 }}>
                  Analyzes codebase and generates comprehensive context files
                </Typography>
              </Box>

              <Divider>
                <Typography variant="caption" color="text.secondary">OR</Typography>
              </Divider>

              <Box>
                <Button variant="outlined" fullWidth onClick={handleInitialize} startIcon={<FileTextIcon />}>
                  Use Templates
                </Button>
                <Typography variant="caption" display="block" align="center" color="text.secondary" sx={{ mt: 1 }}>
                  Create empty template files to fill in manually
                </Typography>
              </Box>
            </Stack>
          </Paper>
        </Box>
      </Stack>
    )
  }

  // Context exists - show files
  return (
    <Stack sx={{ height: '100%' }}>
      <WorkflowHeader
        title="Living Context"
        subtitle="Project memory and architectural source of truth"
        icon={<BookOpenIcon />}
        status={`${contextFiles.length} files`}
        statusColor="primary"
      >
        <Stack direction="row" spacing={1} alignItems="center">
          {lastRefreshed && (
            <Chip
              icon={<ClockIcon sx={{ fontSize: '0.8rem !important' }} />}
              label={new Date(lastRefreshed).toLocaleTimeString()}
              size="small"
              variant="outlined"
              sx={{ height: 24, fontSize: '0.65rem' }}
            />
          )}
          <Button
            variant="outlined"
            size="small"
            onClick={handleGenerateContext}
            startIcon={<WandIcon />}
            sx={{ height: 32, borderRadius: 2 }}
          >
            AI Refresh
          </Button>
          <Tooltip title="Refresh from disk">
            <IconButton size="small" onClick={handleRefresh} sx={{ bgcolor: 'action.hover' }}>
              <RefreshIcon fontSize="small" />
            </IconButton>
          </Tooltip>
        </Stack>
      </WorkflowHeader>

      <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', p: 3, pt: 2 }}>
        {contextFiles.length === 0 ? (
          <EmptyState
            title="No Context Files"
            description="No context files found in .rstn/context/ directory."
            action={{
              label: "Initialize Templates",
              onClick: handleInitialize,
              icon: <PlusIcon />
            }}
          />
        ) : (
          <Paper variant="outlined" sx={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', borderRadius: 4 }}>
            <Box sx={{ borderBottom: 1, borderColor: 'divider', bgcolor: 'surfaceContainerLow.main' }}>
              <Tabs
                value={activeTab}
                onChange={(_, v) => setActiveTab(v)}
                variant="scrollable"
                scrollButtons="auto"
                sx={{ minHeight: 48 }}
              >
                {contextFiles.map((file) => (
                  <Tab
                    key={file.name}
                    value={file.name}
                    label={formatContextName(file.name)}
                    icon={<FileTextIcon sx={{ fontSize: 16 }} />}
                    iconPosition="start"
                    sx={{ minHeight: 48, textTransform: 'none', fontSize: '0.75rem', fontWeight: 600 }}
                  />
                ))}
              </Tabs>
            </Box>

            <Box sx={{ flex: 1, overflow: 'auto', p: 4 }}>
              {activeFile && (
                <Box>
                  {/* File metadata */}
                  <Stack direction="row" spacing={3} sx={{ mb: 4 }}>
                    <Chip
                      label={activeFile.context_type}
                      size="small"
                      sx={{ borderRadius: 1, height: 20, fontSize: '0.6rem', fontWeight: 700, bgcolor: 'secondaryContainer.main' }}
                    />
                    {activeFile.last_updated && (
                      <Stack direction="row" alignItems="center" spacing={0.5} sx={{ color: 'text.secondary' }}>
                        <ClockIcon sx={{ fontSize: 14 }} />
                        <Typography variant="caption">Updated {formatDate(activeFile.last_updated)}</Typography>
                      </Stack>
                    )}
                    <Typography variant="caption" color="text.secondary">~{activeFile.token_estimate} tokens</Typography>
                  </Stack>

                  {/* File content */}
                  <Typography component="div" variant="body2" sx={{ '& h1, & h2, & h3': { mt: 3, mb: 1.5, fontWeight: 600 }, '& ul, & ol': { pl: 2 }, '& pre': { bgcolor: 'action.hover', p: 2, borderRadius: 2, overflow: 'auto', border: 1, borderColor: 'outlineVariant' } }}>
                    <ReactMarkdown>{activeFile.content}</ReactMarkdown>
                  </Typography>
                </Box>
              )}
            </Box>
          </Paper>
        )}
      </Box>
    </Stack>
  )
}

/** Format context file name for display */
function formatContextName(name: string): string {
  return name
    .replace(/-/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase())
}

/** Format ISO date string for display */
function formatDate(isoDate: string): string {
  try {
    return new Date(isoDate).toLocaleDateString()
  } catch {
    return isoDate
  }
}

