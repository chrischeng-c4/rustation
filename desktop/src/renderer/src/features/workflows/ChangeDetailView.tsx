import {
  Description as FileTextIcon,
  PlayArrow as PlayIcon,
  CheckCircle as CheckIcon,
  Cancel as XIcon,
  AccessTime as ClockIcon,
  Archive as ArchiveIcon,
  Refresh as RefreshIcon,
  RocketLaunch as RocketIcon,
  AssignmentTurnedIn as ClipboardCheckIcon,
  ChatBubbleOutline as MessageSquareIcon,
  ThumbUpOutlined as ThumbsUpIcon,
  ThumbDownOutlined as ThumbsDownIcon,
  ExpandMore as ChevronDownIcon,
  Code as FileCodeIcon
} from '@mui/icons-material'
import {
  Button,
  Chip,
  Box,
  Typography,
  Tabs,
  Tab,
  Paper,
  Stack,
  Divider,
  IconButton,
  Collapse,
  alpha
} from '@mui/material'
import { WorkflowHeader } from '@/components/shared/WorkflowHeader'
import { useAppState } from '@/hooks/useAppState'
import { ContextFilesInput } from './ContextFilesInput'
import type { Change, ReviewSession, ReviewStatus, ChangeStatus } from '@/types/state'
import { useState } from 'react'

interface ChangeDetailViewProps {
  change: Change
}

/**
 * Get review session by ID from state
 */
function getReviewSession(
  sessions: Record<string, ReviewSession> | undefined,
  sessionId: string | null | undefined
): ReviewSession | null {
  if (!sessions || !sessionId) return null
  return sessions[sessionId] ?? null
}

/**
 * ReviewStatusBadge - Shows review status with icon
 */
function ReviewStatusBadge({ session, type }: { session: ReviewSession | null; type: 'proposal' | 'plan' }) {
  if (!session) return null

  const statusConfig: Record<ReviewStatus, { label: string; color: 'default' | 'info' | 'warning' | 'success' | 'error'; icon: any }> = {
    pending: { label: 'Pending Review', color: 'default', icon: ClockIcon },
    reviewing: { label: 'In Review', color: 'info', icon: ClipboardCheckIcon },
    iterating: { label: 'Iterating', color: 'warning', icon: RefreshIcon },
    approved: { label: 'Approved', color: 'success', icon: CheckIcon },
    rejected: { label: 'Rejected', color: 'error', icon: XIcon },
  }

  const config = statusConfig[session.status]
  const Icon = config.icon

  return (
    <Chip
      icon={<Icon sx={{ fontSize: '0.8rem !important' }} />}
      label={`${type === 'proposal' ? 'Proposal' : 'Plan'}: ${config.label}`}
      size="small"
      color={config.color as any}
      variant="outlined"
      sx={{ 
        height: 24, 
        fontSize: '0.65rem', 
        fontWeight: 700,
        bgcolor: alpha(config.color === 'default' ? '#000' : '#fff', 0.05)
      }}
    />
  )
}

/**
 * InlineReviewControls - Review action buttons shown inline in content tabs
 */
interface InlineReviewControlsProps {
  session: ReviewSession
  onApprove: () => void
  onReject: () => void
}

function InlineReviewControls({ session, onApprove, onReject }: InlineReviewControlsProps) {
  if (session.status === 'approved' || session.status === 'rejected') {
    return null
  }

  const unresolvedComments = session.comments.filter((c) => !c.resolved).length

  return (
    <Paper elevation={0} sx={{ borderTop: 1, borderColor: 'outlineVariant', bgcolor: 'action.hover', p: 2 }}>
      <Stack direction="row" alignItems="center" justifyContent="space-between">
        <Stack direction="row" alignItems="center" spacing={1.5}>
          <ClipboardCheckIcon fontSize="small" color="primary" />
          <Typography variant="body2" fontWeight={600}>Review required</Typography>
          {unresolvedComments > 0 && (
            <Chip 
              icon={<MessageSquareIcon sx={{ fontSize: '0.8rem !important' }} />}
              label={`${unresolvedComments} active`} 
              size="small" 
              color="secondary"
              sx={{ height: 20, fontSize: '0.6rem' }} 
            />
          )}
        </Stack>
        <Stack direction="row" spacing={1}>
          <Button 
            size="small" 
            variant="outlined" 
            color="error" 
            onClick={onReject} 
            startIcon={<ThumbsDownIcon />}
            sx={{ borderRadius: 1.5 }}
          >
            Reject
          </Button>
          <Button 
            size="small" 
            variant="contained" 
            color="success" 
            onClick={onApprove} 
            startIcon={<ThumbsUpIcon />}
            sx={{ borderRadius: 1.5 }}
          >
            Approve
          </Button>
        </Stack>
      </Stack>
    </Paper>
  )
}

/**
 * ChangeDetailView - Shows change details, proposal, and plan
 */
export function ChangeDetailView({ change }: ChangeDetailViewProps) {
  const { state, dispatch } = useAppState()
  const [activeTab, setActiveTab] = useState('proposal')
  const [contextOpen, setContextOpen] = useState(false)

  // Get review sessions linked to this change
  const activeProject = state?.projects?.[state?.active_project_index ?? 0]
  const worktree = activeProject?.worktrees?.[activeProject?.active_worktree_index ?? 0]
  const reviewSessions = worktree?.tasks?.review_gate?.sessions

  const proposalReviewSession = getReviewSession(reviewSessions, change.proposal_review_session_id)
  const planReviewSession = getReviewSession(reviewSessions, change.plan_review_session_id)

  const handleGenerateProposal = () => {
    dispatch({ type: 'GenerateProposal', payload: { change_id: change.id } })
  }

  const handleGeneratePlan = () => {
    dispatch({ type: 'GeneratePlan', payload: { change_id: change.id } })
  }

  const handleApprovePlan = () => {
    dispatch({ type: 'ApprovePlan', payload: { change_id: change.id } })
  }

  const handleCancelChange = () => {
    dispatch({ type: 'CancelChange', payload: { change_id: change.id } })
  }

  const handleSyncContext = () => {
    dispatch({ type: 'SyncContext', payload: { change_id: change.id } })
  }

  const handleArchive = () => {
    dispatch({ type: 'ArchiveChange', payload: { change_id: change.id } })
  }

  const handleExecutePlan = () => {
    dispatch({ type: 'ExecutePlan', payload: { change_id: change.id } })
  }

  // Review action handlers
  const handleApproveProposalReview = () => {
    if (proposalReviewSession) {
      dispatch({ type: 'ApproveReview', payload: { session_id: proposalReviewSession.id } })
    }
  }

  const handleRejectProposalReview = () => {
    if (proposalReviewSession) {
      dispatch({ type: 'RejectReview', payload: { session_id: proposalReviewSession.id, reason: 'Rejected by user' } })
    }
  }

  const handleApprovePlanReview = () => {
    if (planReviewSession) {
      dispatch({ type: 'ApproveReview', payload: { session_id: planReviewSession.id } })
    }
  }

  const handleRejectPlanReview = () => {
    if (planReviewSession) {
      dispatch({ type: 'RejectReview', payload: { session_id: planReviewSession.id, reason: 'Rejected by user' } })
    }
  }

  const isPlanning = change.status === 'planning'
  const isImplementing = change.status === 'implementing'
  const hasProposal = !!change.proposal
  const hasPlan = !!change.plan
  const canGenerateProposal = change.status === 'proposed' && !hasProposal
  const canGeneratePlan = hasProposal && !hasPlan && change.status !== 'planning'
  const canApprove = change.status === 'planned'
  const canExecute = change.status === 'planned'
  const canCancel = !['done', 'archived', 'cancelled', 'implementing'].includes(change.status)
  const canSyncAndArchive = change.status === 'done'
  const isArchived = change.status === 'archived'

  const STATUS_COLORS: Record<string, 'info' | 'warning' | 'secondary' | 'success' | 'error' | 'default'> = {
    proposed: 'info',
    planning: 'warning',
    planned: 'secondary',
    implementing: 'warning',
    testing: 'info',
    done: 'success',
    archived: 'default',
    cancelled: 'error',
    failed: 'error',
  }

  return (
    <Stack sx={{ height: '100%' }}>
      <WorkflowHeader
        title={change.name}
        subtitle={change.intent}
        status={change.status}
        statusColor={STATUS_COLORS[change.status] || "default"}
      >
        {/* Review Status Badges */}
        <Stack direction="row" spacing={1}>
          {proposalReviewSession && <ReviewStatusBadge session={proposalReviewSession} type="proposal" />}
          {planReviewSession && <ReviewStatusBadge session={planReviewSession} type="plan" />}
        </Stack>
      </WorkflowHeader>

      <Box sx={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column', p: 3 }}>
        {/* Context Files Section */}
        {worktree?.path && (
          <Box sx={{ mb: 2 }}>
            <Paper 
              variant="outlined" 
              sx={{ 
                p: 1.5, 
                px: 2, 
                display: 'flex', 
                alignItems: 'center', 
                justifyContent: 'space-between',
                cursor: 'pointer',
                bgcolor: 'surfaceContainerLow.main',
                '&:hover': { bgcolor: 'action.hover' }
              }}
              onClick={() => setContextOpen(!contextOpen)}
            >
              <Stack direction="row" spacing={1.5} alignItems="center">
                <FileCodeIcon fontSize="small" sx={{ color: 'text.secondary' }} />
                <Typography variant="body2" fontWeight={600}>Context Files</Typography>
                {(change.context_files?.length ?? 0) > 0 && (
                  <Chip label={change.context_files.length} size="small" sx={{ height: 18, fontSize: '0.6rem' }} />
                )}
              </Stack>
              <ChevronDownIcon sx={{ fontSize: 18, transform: contextOpen ? 'rotate(180deg)' : 'none', transition: 'transform 0.2s' }} />
            </Paper>
            <Collapse in={contextOpen}>
              <Paper variant="outlined" sx={{ mt: 1, p: 2, borderStyle: 'dashed' }}>
                <ContextFilesInput
                  changeId={change.id}
                  files={change.context_files ?? []}
                  projectRoot={worktree.path}
                />
              </Paper>
            </Collapse>
          </Box>
        )}

        <Paper variant="outlined" sx={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', borderRadius: 4 }}>
          <Box sx={{ borderBottom: 1, borderColor: 'divider', bgcolor: 'surfaceContainerLow.main' }}>
            <Tabs value={activeTab} onChange={(_, v) => setActiveTab(v)}>
              <Tab 
                value="proposal" 
                label={
                  <Stack direction="row" alignItems="center" spacing={1}>
                    <FileTextIcon sx={{ fontSize: 16 }} />
                    <Typography variant="caption" fontWeight={700}>Proposal</Typography>
                    {proposalReviewSession?.status === 'reviewing' && (
                      <Box sx={{ width: 6, height: 6, borderRadius: '50%', bgcolor: 'info.main' }} />
                    )}
                  </Stack>
                }
                sx={{ textTransform: 'none', minHeight: 48 }}
              />
              <Tab 
                value="plan" 
                label={
                  <Stack direction="row" alignItems="center" spacing={1}>
                    <FileTextIcon sx={{ fontSize: 16 }} />
                    <Typography variant="caption" fontWeight={700}>Plan</Typography>
                    {planReviewSession?.status === 'reviewing' && (
                      <Box sx={{ width: 6, height: 6, borderRadius: '50%', bgcolor: 'info.main' }} />
                    )}
                  </Stack>
                }
                sx={{ textTransform: 'none', minHeight: 48 }}
              />
              <Tab 
                value="implementation" 
                label={
                  <Stack direction="row" alignItems="center" spacing={1}>
                    <RocketIcon sx={{ fontSize: 16 }} />
                    <Typography variant="caption" fontWeight={700}>Implementation</Typography>
                  </Stack>
                }
                sx={{ textTransform: 'none', minHeight: 48 }}
              />
            </Tabs>
          </Box>

          <Box sx={{ flex: 1, overflow: 'auto', p: 0, display: 'flex', flexDirection: 'column' }}>
            {activeTab === 'proposal' && (
              <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                {isPlanning && change.streaming_output ? (
                  <Box sx={{ p: 3 }}>
                    <Stack direction="row" spacing={1} sx={{ mb: 2, color: 'warning.main' }}>
                      <RefreshIcon fontSize="small" sx={{ animation: 'spin 2s linear infinite' }} />
                      <Typography variant="caption" fontWeight={700}>Generating proposal...</Typography>
                    </Stack>
                    <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                      {change.streaming_output}
                    </Typography>
                  </Box>
                ) : hasProposal ? (
                  <>
                    <Box sx={{ flex: 1, p: 3 }}>
                      <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                        {change.proposal}
                      </Typography>
                    </Box>
                    {proposalReviewSession && (
                      <InlineReviewControls
                        session={proposalReviewSession}
                        onApprove={handleApproveProposalReview}
                        onReject={handleRejectProposalReview}
                      />
                    )}
                  </>
                ) : (
                  <Stack alignItems="center" justifyContent="center" sx={{ flex: 1, p: 4 }}>
                    <FileTextIcon sx={{ fontSize: 48, color: 'text.disabled', mb: 2, opacity: 0.5 }} />
                    <Typography variant="body2" color="text.secondary" gutterBottom>No proposal generated yet</Typography>
                    {canGenerateProposal && (
                      <Button variant="contained" size="small" onClick={handleGenerateProposal} startIcon={<PlayIcon />} sx={{ mt: 1, borderRadius: 2 }}>
                        Generate Proposal
                      </Button>
                    )}
                  </Stack>
                )}
              </Box>
            )}

            {activeTab === 'plan' && (
              <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                {isPlanning && !hasProposal && change.streaming_output ? (
                  <Box sx={{ p: 3 }}>
                    <Stack direction="row" spacing={1} sx={{ mb: 2, color: 'warning.main' }}>
                      <RefreshIcon fontSize="small" sx={{ animation: 'spin 2s linear infinite' }} />
                      <Typography variant="caption" fontWeight={700}>Generating plan...</Typography>
                    </Stack>
                    <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                      {change.streaming_output}
                    </Typography>
                  </Box>
                ) : hasPlan ? (
                  <>
                    <Box sx={{ flex: 1, p: 3 }}>
                      <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                        {change.plan}
                      </Typography>
                    </Box>
                    {planReviewSession && (
                      <InlineReviewControls
                        session={planReviewSession}
                        onApprove={handleApprovePlanReview}
                        onReject={handleRejectPlanReview}
                      />
                    )}
                  </>
                ) : (
                  <Stack alignItems="center" justifyContent="center" sx={{ flex: 1, p: 4 }}>
                    <FileTextIcon sx={{ fontSize: 48, color: 'text.disabled', mb: 2, opacity: 0.5 }} />
                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      {hasProposal ? 'No plan generated yet' : 'Generate a proposal first'}
                    </Typography>
                    {canGeneratePlan && (
                      <Button variant="contained" size="small" onClick={handleGeneratePlan} startIcon={<PlayIcon />} sx={{ mt: 1, borderRadius: 2 }}>
                        Generate Plan
                      </Button>
                    )}
                  </Stack>
                )}
              </Box>
            )}

            {activeTab === 'implementation' && (
              <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                {isImplementing && change.streaming_output ? (
                  <Box sx={{ p: 3 }}>
                    <Stack direction="row" spacing={1} sx={{ mb: 2, color: 'info.main' }}>
                      <RocketIcon fontSize="small" sx={{ animation: 'pulse 1.5s infinite' }} />
                      <Typography variant="caption" fontWeight={700}>Implementing...</Typography>
                    </Stack>
                    <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                      {change.streaming_output}
                    </Typography>
                  </Box>
                ) : change.status === 'done' ? (
                  <Box sx={{ p: 3 }}>
                    <Stack direction="row" spacing={1} sx={{ mb: 2, color: 'success.main' }}>
                      <CheckIcon fontSize="small" />
                      <Typography variant="caption" fontWeight={700}>Implementation Complete</Typography>
                    </Stack>
                    <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                      {change.streaming_output}
                    </Typography>
                  </Box>
                ) : change.status === 'failed' ? (
                  <Box sx={{ p: 3 }}>
                    <Stack direction="row" spacing={1} sx={{ mb: 2, color: 'error.main' }}>
                      <XIcon fontSize="small" />
                      <Typography variant="caption" fontWeight={700}>Implementation Failed</Typography>
                    </Stack>
                    <Typography component="pre" variant="caption" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                      {change.streaming_output}
                    </Typography>
                  </Box>
                ) : (
                  <Stack alignItems="center" justifyContent="center" sx={{ flex: 1, p: 4 }}>
                    <RocketIcon sx={{ fontSize: 48, color: 'text.disabled', mb: 2, opacity: 0.5 }} />
                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      {hasPlan ? 'Ready to implement' : 'Generate a plan first'}
                    </Typography>
                    {canExecute && (
                      <Button variant="contained" size="small" onClick={handleExecutePlan} startIcon={<RocketIcon />} sx={{ mt: 1, borderRadius: 2 }}>
                        Execute Plan
                      </Button>
                    )}
                  </Stack>
                )}
              </Box>
            )}
          </Box>
        </Paper>

        {/* Action Buttons Bar */}
        <Stack direction="row" spacing={2} sx={{ mt: 3, pt: 2, borderTop: 1, borderColor: 'outlineVariant' }}>
          {canApprove && (
            <Button variant="contained" color="success" onClick={handleApprovePlan} startIcon={<CheckIcon />} sx={{ borderRadius: 2 }}>
              Approve Plan
            </Button>
          )}
          {canExecute && (
            <Button variant="contained" color="primary" onClick={handleExecutePlan} startIcon={<RocketIcon />} sx={{ borderRadius: 2 }}>
              Execute Plan
            </Button>
          )}
          {isImplementing && (
            <Chip icon={<RocketIcon sx={{ animation: 'pulse 1.5s infinite' }} />} label="Implementing..." color="warning" variant="filled" sx={{ borderRadius: 1.5 }} />
          )}
          {canSyncAndArchive && (
            <>
              <Button variant="outlined" color="primary" onClick={handleSyncContext} startIcon={<RefreshIcon />} sx={{ borderRadius: 2 }}>
                Sync to Context
              </Button>
              <Button variant="contained" color="primary" onClick={handleArchive} startIcon={<ArchiveIcon />} sx={{ borderRadius: 2 }}>
                Archive
              </Button>
            </>
          )}
          {isArchived && (
            <Chip icon={<ArchiveIcon />} label="Archived" color="default" sx={{ borderRadius: 1.5 }} />
          )}
          <Box sx={{ ml: 'auto' }}>
            {canCancel && (
              <Button variant="text" color="error" onClick={handleCancelChange} startIcon={<XIcon />}>
                Cancel Change
              </Button>
            )}
          </Box>
        </Stack>
      </Box>
      <style>{`
        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
        @keyframes pulse { 0% { opacity: 1; } 50% { opacity: 0.5; } 100% { opacity: 1; } }
      `}</style>
    </Stack>
  )
}
