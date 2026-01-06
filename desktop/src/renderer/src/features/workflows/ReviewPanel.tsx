import { useCallback, useState, useMemo } from 'react'
import {
  Description as FileTextIcon,
  ChatBubbleOutline as MessageSquareIcon,
  CheckCircle as CheckCircleIcon,
  Cancel as XCircleIcon,
  ChevronRight as ChevronRightIcon,
  Add as PlusIcon,
  Refresh as RefreshIcon,
  AssignmentTurnedIn as ClipboardCheckIcon,
  ExpandMore as ExpandMoreIcon
} from '@mui/icons-material'
import {
  Button,
  Card,
  CardContent,
  Box,
  Typography,
  Chip,
  Paper,
  Stack,
  Divider,
  IconButton,
  TextField,
  MenuItem,
  Select,
  FormControl,
  InputLabel,
  Tooltip
} from '@mui/material'
import { PageHeader } from '@/components/shared/PageHeader'
import { WorkflowHeader } from '@/components/shared/WorkflowHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { ErrorBanner } from '@/components/shared/ErrorBanner'
import { useAppState } from '@/hooks/useAppState'
import ReactMarkdown from 'react-markdown'
import type {
  ReviewSession,
  CommentTarget,
  ReviewStatus,
  ReviewContentType,
} from '@/types/state'

// ============================================================================
// Status Configuration
// ============================================================================

const STATUS_CONFIG: Record<
  ReviewStatus,
  { label: string; color: 'default' | 'primary' | 'secondary' | 'success' | 'error' | 'info' | 'warning' }
> = {
  pending: { label: 'Pending', color: 'default' },
  reviewing: { label: 'Reviewing', color: 'info' },
  iterating: { label: 'Iterating', color: 'warning' },
  approved: { label: 'Approved', color: 'success' },
  rejected: { label: 'Rejected', color: 'error' },
}

const CONTENT_TYPE_LABELS: Record<ReviewContentType, string> = {
  Plan: 'Plan',
  Proposal: 'Proposal',
  Code: 'Code',
  Artifact: 'Artifact',
}

// ============================================================================
// Helper Functions
// ============================================================================

function parseMarkdownSections(content: string): { id: string; title: string; level: number }[] {
  const sections: { id: string; title: string; level: number }[] = []
  const lines = content.split('\n')

  lines.forEach((line) => {
    const h1Match = line.match(/^#\s+(.+)$/)
    const h2Match = line.match(/^#{2}\s+(.+)$/)

    if (h1Match) {
      const title = h1Match[1].trim()
      const id = title.toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '')
      sections.push({ id, title, level: 1 })
    } else if (h2Match) {
      const title = h2Match[1].trim()
      const id = title.toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '')
      sections.push({ id, title, level: 2 })
    }
  })

  return sections
}

function getTargetDisplay(target: CommentTarget): string {
  if (target.type === 'document') return 'General'
  if (target.type === 'section') return `Section: ${target.id}`
  if (target.type === 'file') return `File: ${target.path}`
  return 'Unknown'
}

// ============================================================================
// Sub-Components
// ============================================================================

interface ContentViewProps {
  session: ReviewSession
  onSectionClick: (sectionId: string) => void
}

function ContentView({ session, onSectionClick }: ContentViewProps) {
  const sections = useMemo(() => parseMarkdownSections(session.content.content), [session.content.content])

  return (
    <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column' }}>
      {/* Content Header */}
      <Box sx={{ borderBottom: 1, borderColor: 'outlineVariant', bgcolor: 'surfaceContainerLow.main', px: 2, py: 1, display: 'flex', alignItems: 'center', justifyContent: 'space-between', height: 48 }}>
        <Stack direction="row" spacing={1.5} alignItems="center">
          <FileTextIcon fontSize="small" sx={{ color: 'text.secondary' }} />
          <Typography variant="caption" fontWeight={700} sx={{ textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            {CONTENT_TYPE_LABELS[session.content.content_type]}
          </Typography>
          {session.iteration > 1 && (
            <Chip label={`Iteration ${session.iteration}`} size="small" sx={{ height: 20, fontSize: '0.65rem', borderRadius: 0.5, bgcolor: 'secondaryContainer.main' }} />
          )}
        </Stack>
        <Chip 
          label={STATUS_CONFIG[session.status].label} 
          size="small" 
          color={STATUS_CONFIG[session.status].color as any}
          sx={{ height: 20, fontSize: '0.65rem', fontWeight: 700 }}
        />
      </Box>

      {/* Section Markers */}
      {sections.length > 0 && (
        <Box sx={{ borderBottom: 1, borderColor: 'outlineVariant', bgcolor: 'background.default', px: 2, py: 0.75 }}>
          <Stack direction="row" spacing={1} sx={{ flexWrap: 'wrap', gap: 1 }}>
            {sections.map((section) => (
              <Button
                key={section.id}
                variant="text"
                size="small"
                onClick={() => onSectionClick(section.id)}
                sx={{ 
                  height: 24, 
                  fontSize: '0.65rem', 
                  px: 1, 
                  minWidth: 0, 
                  color: 'text.secondary',
                  '&:hover': { color: 'primary.main' }
                }}
              >
                {section.level === 1 ? '# ' : '## '}{section.title}
              </Button>
            ))}
          </Stack>
        </Box>
      )}

      {/* Content Body */}
      <Box sx={{ flex: 1, overflow: 'auto', p: 4 }}>
        <Typography component="div" variant="body2" sx={{ '& h1, & h2, & h3': { mt: 3, mb: 1.5, fontWeight: 600 }, '& ul, & ol': { pl: 2 } }}>
          <ReactMarkdown>{session.content.content}</ReactMarkdown>
        </Typography>

        {/* File Changes */}
        {session.content.file_changes.length > 0 && (
          <Box sx={{ mt: 6, border: 1, borderColor: 'outlineVariant', borderRadius: 2, overflow: 'hidden', bgcolor: 'surfaceContainerLow.main' }}>
            <Box sx={{ bgcolor: 'action.hover', px: 2, py: 1, borderBottom: 1, borderColor: 'outlineVariant' }}>
              <Typography variant="caption" fontWeight={700} sx={{ textTransform: 'uppercase' }}>File Changes</Typography>
            </Box>
            <Stack spacing={2} sx={{ p: 2 }}>
              {session.content.file_changes.map((change, idx) => (
                <Stack key={idx} direction="row" spacing={2} alignItems="flex-start">
                  <Chip 
                    label={change.action} 
                    size="small" 
                    variant="filled"
                    color={change.action === 'create' ? 'success' : change.action === 'modify' ? 'info' : 'error'}
                    sx={{ height: 18, fontSize: '0.6rem', fontWeight: 700, borderRadius: 0.5, mt: 0.25 }}
                  />
                  <Box>
                    <Typography variant="caption" sx={{ fontFamily: 'monospace', fontWeight: 600, color: 'primary.main' }}>{change.path}</Typography>
                    <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5, fontSize: '0.75rem' }}>{change.summary}</Typography>
                  </Box>
                </Stack>
              ))}
            </Stack>
          </Box>
        )}
      </Box>
    </Box>
  )
}

interface CommentsSidebarProps {
  session: ReviewSession
  onAddComment: (target: CommentTarget, content: string) => void
  onResolveComment: (commentId: string) => void
}

function CommentsSidebar({ session, onAddComment, onResolveComment }: CommentsSidebarProps) {
  const [newCommentContent, setNewCommentContent] = useState('')
  const [newCommentTargetType, setNewCommentTargetType] = useState<'document' | 'section' | 'file'>('document')

  const handleAddComment = useCallback(() => {
    if (!newCommentContent.trim()) return
    
    let target: CommentTarget = { type: 'document' }
    if (newCommentTargetType === 'section') target = { type: 'section', id: '' }
    else if (newCommentTargetType === 'file') target = { type: 'file', path: '' }
    
    onAddComment(target, newCommentContent.trim())
    setNewCommentContent('')
    setNewCommentTargetType('document')
  }, [newCommentContent, newCommentTargetType, onAddComment])

  const unresolvedComments = session.comments.filter((c) => !c.resolved)
  const resolvedComments = session.comments.filter((c) => c.resolved)

  return (
    <Box sx={{ display: 'flex', height: '100%', width: 320, flexDirection: 'column', borderLeft: 1, borderColor: 'outlineVariant', bgcolor: 'surfaceContainerLow.main' }}>
      {/* Sidebar Header */}
      <Box sx={{ borderBottom: 1, borderColor: 'outlineVariant', bgcolor: 'surfaceContainerLow.main', px: 2, py: 1, display: 'flex', alignItems: 'center', justifyContent: 'space-between', height: 48 }}>
        <Stack direction="row" spacing={1} alignItems="center">
          <MessageSquareIcon fontSize="small" sx={{ color: 'text.secondary' }} />
          <Typography variant="caption" fontWeight={700} sx={{ textTransform: 'uppercase' }}>Comments</Typography>
        </Stack>
        <Chip label={`${unresolvedComments.length} active`} size="small" sx={{ height: 20, fontSize: '0.65rem' }} />
      </Box>

      {/* Comments List */}
      <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
        <Stack spacing={2}>
          {/* Unresolved Comments */}
          {unresolvedComments.map((comment) => (
            <Card key={comment.id} elevation={0} sx={{ border: 1, borderColor: 'outlineVariant', bgcolor: 'background.paper' }}>
              <CardContent sx={{ p: 2 }}>
                <Box sx={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', mb: 1 }}>
                  <Chip 
                    label={getTargetDisplay(comment.target)} 
                    size="small" 
                    variant="outlined" 
                    sx={{ height: 18, fontSize: '0.6rem', borderRadius: 0.5 }} 
                  />
                  <IconButton 
                    size="small" 
                    onClick={() => onResolveComment(comment.id)}
                    sx={{ p: 0.25, '&:hover': { color: 'success.main' } }}
                  >
                    <CheckCircleIcon sx={{ fontSize: 16 }} />
                  </IconButton>
                </Box>
                <Typography variant="body2" sx={{ fontSize: '0.8rem' }}>{comment.content}</Typography>
                <Box sx={{ mt: 1.5, pt: 1, borderTop: 1, borderColor: 'action.hover', display: 'flex', justifyContent: 'space-between' }}>
                  <Typography variant="caption" color="text.secondary">{comment.author === 'user' ? 'You' : 'System'}</Typography>
                  <Typography variant="caption" color="text.secondary">{new Date(comment.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</Typography>
                </Box>
              </CardContent>
            </Card>
          ))}

          {/* Resolved Divider */}
          {resolvedComments.length > 0 && (
            <Divider sx={{ my: 1 }}>
              <Typography variant="caption" color="text.disabled" sx={{ fontWeight: 700, textTransform: 'uppercase' }}>Resolved</Typography>
            </Divider>
          )}

          {/* Resolved Comments */}
          {resolvedComments.map((comment) => (
            <Box key={comment.id} sx={{ opacity: 0.5, p: 1 }}>
              <Stack direction="row" spacing={1} alignItems="center">
                <CheckCircleIcon sx={{ fontSize: 14, color: 'success.main' }} />
                <Typography variant="caption" fontWeight={600}>{getTargetDisplay(comment.target)}</Typography>
              </Stack>
              <Typography variant="caption" sx={{ fontStyle: 'italic', display: 'block', mt: 0.5 }}>{comment.content}</Typography>
            </Box>
          ))}

          {session.comments.length === 0 && (
            <Box sx={{ py: 8, textAlign: 'center' }}>
              <MessageSquareIcon sx={{ fontSize: 48, color: 'text.disabled', opacity: 0.3, mb: 1 }} />
              <Typography variant="caption" display="block" color="text.secondary">No feedback yet</Typography>
            </Box>
          )}
        </Stack>
      </Box>

      {/* Add Comment Form */}
      <Box sx={{ p: 2, borderTop: 1, borderColor: 'outlineVariant', bgcolor: 'background.paper' }}>
        <TextField
          fullWidth
          multiline
          rows={3}
          placeholder="Add your feedback..."
          value={newCommentContent}
          onChange={(e) => setNewCommentContent(e.target.value)}
          variant="outlined"
          size="small"
          sx={{ mb: 1.5, '& .MuiInputBase-root': { fontSize: '0.8rem' } }}
        />
        <Stack direction="row" spacing={1}>
          <FormControl fullWidth size="small">
            <Select
              value={newCommentTargetType}
              onChange={(e) => setNewCommentTargetType(e.target.value as any)}
              sx={{ fontSize: '0.75rem', height: 32 }}
            >
              <MenuItem value="document">General</MenuItem>
              <MenuItem value="section">Section</MenuItem>
              <MenuItem value="file">File</MenuItem>
            </Select>
          </FormControl>
          <Button
            variant="contained"
            size="small"
            onClick={handleAddComment}
            disabled={!newCommentContent.trim()}
            startIcon={<PlusIcon />}
            sx={{ flexShrink: 0, height: 32, borderRadius: 1.5 }}
          >
            Comment
          </Button>
        </Stack>
      </Box>
    </Box>
  )
}

interface ActionBarProps {
  session: ReviewSession
  onApprove: () => void
  onRequestChanges: () => void
  onReject: () => void
}

function ActionBar({ session, onApprove, onRequestChanges, onReject }: ActionBarProps) {
  const canApprove = session.status === 'reviewing'
  const canRequestChanges = session.status === 'reviewing'
  const canReject = session.status === 'reviewing'

  return (
    <Box sx={{ borderTop: 1, borderColor: 'outlineVariant', bgcolor: 'surfaceContainer.main', px: 3, py: 2 }}>
      <Stack direction="row" spacing={2} alignItems="center">
        <Button
          variant="contained"
          color="success"
          onClick={onApprove}
          disabled={!canApprove}
          startIcon={<CheckCircleIcon />}
          sx={{ borderRadius: 2, px: 3 }}
        >
          Approve & Proceed
        </Button>
        <Button
          variant="outlined"
          color="warning"
          onClick={onRequestChanges}
          disabled={!canRequestChanges}
          startIcon={<RefreshIcon />}
          sx={{ borderRadius: 2, px: 3 }}
        >
          Request Changes
        </Button>
        <Box sx={{ ml: 'auto' }}>
          <Button
            variant="text"
            color="error"
            onClick={onReject}
            disabled={!canReject}
            startIcon={<XCircleIcon />}
          >
            Reject
          </Button>
        </Box>
      </Stack>
    </Box>
  )
}

// ============================================================================
// Main Component
// ============================================================================

export function ReviewPanel() {
  const { state, dispatch, isLoading } = useAppState()

  const activeProject = state?.projects?.[state?.active_project_index ?? 0]
  const worktree = activeProject?.worktrees?.[activeProject?.active_worktree_index ?? 0]
  const reviewGate = worktree?.tasks?.review_gate

  const activeSession = useMemo(() => {
    if (!reviewGate?.active_session_id || !reviewGate.sessions) return null
    return reviewGate.sessions[reviewGate.active_session_id] ?? null
  }, [reviewGate])

  const allSessions = useMemo(() => {
    if (!reviewGate?.sessions) return []
    return Object.values(reviewGate.sessions).sort(
      (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    )
  }, [reviewGate])

  const handleSessionSelect = useCallback((sessionId: string) => {
    dispatch({ type: 'SetActiveReviewSession', payload: { session_id: sessionId } })
  }, [dispatch])

  const handleSectionClick = useCallback((sectionId: string) => {
    const element = document.getElementById(sectionId)
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  }, [])

  const handleAddComment = useCallback((target: CommentTarget, content: string) => {
    if (!activeSession) return
    dispatch({
      type: 'AddReviewComment',
      payload: { session_id: activeSession.id, target, content },
    })
  }, [activeSession, dispatch])

  const handleResolveComment = useCallback((commentId: string) => {
    if (!activeSession) return
    dispatch({
      type: 'ResolveReviewComment',
      payload: { session_id: activeSession.id, comment_id: commentId },
    })
  }, [activeSession, dispatch])

  const handleApprove = useCallback(() => {
    if (!activeSession) return
    dispatch({ type: 'ApproveReview', payload: { session_id: activeSession.id } })
  }, [activeSession, dispatch])

  const handleRequestChanges = useCallback(() => {
    if (!activeSession) return
    dispatch({ type: 'SubmitReviewFeedback', payload: { session_id: activeSession.id } })
  }, [activeSession, dispatch])

  const handleReject = useCallback(() => {
    if (!activeSession) return
    const reason = prompt('Reason for rejection (optional):')
    dispatch({
      type: 'RejectReview', payload: { session_id: activeSession.id, reason: reason || 'Rejected by user' },
    })
  }, [activeSession, dispatch])

  if (isLoading || reviewGate?.is_loading) {
    return <LoadingState message="Loading review sessions..." />
  }

  if (reviewGate?.error) {
    return (
      <Stack sx={{ height: '100%' }}>
        <PageHeader title="Review Error" icon={<XCircleIcon color="error" />} />
        <Box sx={{ p: 3 }}><ErrorBanner error={reviewGate.error} /></Box>
      </Stack>
    )
  }

  if (allSessions.length === 0) {
    return (
      <Stack sx={{ height: '100%' }}>
        <PageHeader title="ReviewGate" description="Human-in-the-loop review mechanism" icon={<ClipboardCheckIcon />} />
        <Box sx={{ flex: 1, p: 3 }}>
          <EmptyState title="No Review Sessions" description="Review sessions will appear here when Claude generates artifacts that require your approval." />
        </Box>
      </Stack>
    )
  }

  if (!activeSession) {
    return (
      <Stack sx={{ height: '100%' }}>
        <PageHeader title="Review Sessions" description={`${allSessions.length} sessions available`} icon={<ClipboardCheckIcon />} />
        <Box sx={{ flex: 1, p: 3, overflow: 'auto' }}>
          <Stack spacing={2} direction="row" sx={{ flexWrap: 'wrap' }}>
            {allSessions.map((session) => (
              <Card 
                key={session.id} 
                variant="outlined" 
                onClick={() => handleSessionSelect(session.id)}
                sx={{ width: 300, cursor: 'pointer', '&:hover': { borderColor: 'primary.main', bgcolor: 'action.hover' } }}
              >
                <CardContent>
                  <Stack direction="row" justifyContent="space-between" alignItems="flex-start" sx={{ mb: 2 }}>
                    <Typography variant="subtitle2" fontWeight={700}>{CONTENT_TYPE_LABELS[session.content.content_type]}</Typography>
                    <Chip label={STATUS_CONFIG[session.status].label} size="small" color={STATUS_CONFIG[session.status].color as any} sx={{ height: 18, fontSize: '0.6rem' }} />
                  </Stack>
                  <Typography variant="caption" color="text.secondary" display="block">
                    {new Date(session.created_at).toLocaleString()}
                  </Typography>
                  <Stack direction="row" spacing={2} sx={{ mt: 2 }}>
                    <Typography variant="caption">Iter: {session.iteration}</Typography>
                    <Stack direction="row" alignItems="center" spacing={0.5}>
                      <MessageSquareIcon sx={{ fontSize: 12 }} />
                      <Typography variant="caption">{session.comments.filter(c => !c.resolved).length}</Typography>
                    </Stack>
                  </Stack>
                </CardContent>
              </Card>
            ))}
          </Stack>
        </Box>
      </Stack>
    )
  }

  return (
    <Stack sx={{ height: '100%' }}>
      <WorkflowHeader
        title={`${CONTENT_TYPE_LABELS[activeSession.content.content_type]} Review`}
        subtitle={`Iteration ${activeSession.iteration} • ${new Date(activeSession.created_at).toLocaleString()}`}
        icon={<ClipboardCheckIcon />}
      >
        {allSessions.length > 1 && (
          <FormControl variant="standard" size="small" sx={{ minWidth: 120 }}>
            <Select
              value={activeSession.id}
              onChange={(e) => handleSessionSelect(e.target.value)}
              sx={{ fontSize: '0.7rem' }}
            >
              {allSessions.map((s) => (
                <MenuItem key={s.id} value={s.id} sx={{ fontSize: '0.7rem' }}>
                  {CONTENT_TYPE_LABELS[s.content.content_type]} - Iter {s.iteration}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        )}
      </WorkflowHeader>

      <Box sx={{ flex: 1, overflow: 'hidden', p: 3, pt: 2 }}>
        <Paper variant="outlined" sx={{ display: 'flex', height: '100%', borderRadius: 4, overflow: 'hidden' }}>
          <Box sx={{ flex: 1, minWidth: 0 }}>
            <ContentView session={activeSession} onSectionClick={handleSectionClick} />
          </Box>
          <CommentsSidebar
            session={activeSession}
            onAddComment={handleAddComment}
            onResolveComment={handleResolveComment}
          />
        </Paper>
      </Box>

      <ActionBar
        session={activeSession}
        onApprove={handleApprove}
        onRequestChanges={handleRequestChanges}
        onReject={handleReject}
      />
    </Stack>
  )
}
