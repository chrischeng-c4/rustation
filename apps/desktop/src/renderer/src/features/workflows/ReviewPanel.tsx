import { useCallback, useState, useMemo } from 'react'
import {
  FileText,
  MessageSquare,
  CheckCircle,
  XCircle,
  ChevronRight,
  Plus,
  RefreshCw,
  ClipboardCheck,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Textarea } from '@/components/ui/textarea'
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
  { label: string; color: string; bgClass: string; textClass: string }
> = {
  pending: {
    label: 'Pending',
    color: 'gray',
    bgClass: 'bg-gray-500/10',
    textClass: 'text-gray-700 dark:text-gray-300',
  },
  reviewing: {
    label: 'Reviewing',
    color: 'blue',
    bgClass: 'bg-blue-500/10',
    textClass: 'text-blue-700 dark:text-blue-300',
  },
  iterating: {
    label: 'Iterating',
    color: 'yellow',
    bgClass: 'bg-yellow-500/10',
    textClass: 'text-yellow-700 dark:text-yellow-300',
  },
  approved: {
    label: 'Approved',
    color: 'green',
    bgClass: 'bg-green-500/10',
    textClass: 'text-green-700 dark:text-green-300',
  },
  rejected: {
    label: 'Rejected',
    color: 'red',
    bgClass: 'bg-red-500/10',
    textClass: 'text-red-700 dark:text-red-300',
  },
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

/**
 * Parse markdown content to extract section markers (H1, H2 headings).
 */
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

/**
 * Get display text for comment target.
 */
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
    <div className="flex h-full flex-col">
      {/* Content Header */}
      <div className="border-b bg-muted/40 px-4 py-2 flex items-center justify-between h-10 shrink-0">
        <div className="flex items-center gap-2">
          <FileText className="h-4 w-4 text-muted-foreground" />
          <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            {CONTENT_TYPE_LABELS[session.content.content_type]}
          </span>
          {session.iteration > 1 && (
            <Badge variant="outline" className="text-[10px] h-5 px-1.5 bg-muted/50 border-none">
              Iteration {session.iteration}
            </Badge>
          )}
        </div>
        <Badge variant="secondary" className={`${STATUS_CONFIG[session.status].bgClass} border-none h-5 px-1.5`}>
          <span className={`${STATUS_CONFIG[session.status].textClass} text-[10px]`}>
            {STATUS_CONFIG[session.status].label}
          </span>
        </Badge>
      </div>

      {/* Section Markers */}
      {sections.length > 0 && (
        <div className="border-b bg-muted/20 px-4 py-1.5 shrink-0">
          <div className="flex flex-wrap gap-1">
            {sections.map((section) => (
              <Button
                key={section.id}
                variant="ghost"
                size="sm"
                className="h-5 px-1.5 text-[10px] text-muted-foreground hover:text-foreground"
                onClick={() => onSectionClick(section.id)}
              >
                {section.level === 1 ? '# ' : '## '}
                {section.title}
              </Button>
            ))}
          </div>
        </div>
      )}

      {/* Content Body */}
      <ScrollArea className="flex-1">
        <div className="p-6">
          <div className="prose prose-sm dark:prose-invert max-w-none prose-headings:font-semibold prose-h1:text-xl prose-h2:text-lg prose-h3:text-base">
            <ReactMarkdown>{session.content.content}</ReactMarkdown>
          </div>

          {/* File Changes */}
          {session.content.file_changes.length > 0 && (
            <div className="mt-8 border rounded-lg overflow-hidden bg-muted/5">
              <div className="bg-muted/30 px-4 py-2 border-b">
                <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">File Changes</h4>
              </div>
              <div className="p-4 space-y-3">
                {session.content.file_changes.map((change, idx) => (
                  <div key={idx} className="flex items-start gap-3 text-xs">
                    <Badge
                      variant={
                        change.action === 'create'
                          ? 'default'
                          : change.action === 'modify'
                            ? 'secondary'
                            : 'destructive'
                      }
                      className="mt-0.5 h-5 px-1.5 text-[10px]"
                    >
                      {change.action}
                    </Badge>
                    <div className="flex-1">
                      <code className="text-[11px] font-mono bg-muted px-1 rounded">{change.path}</code>
                      <p className="text-muted-foreground mt-1 leading-relaxed">{change.summary}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </ScrollArea>
    </div>
  )
}

interface CommentsSidebarProps {
  session: ReviewSession
  onAddComment: (target: CommentTarget, content: string) => void
  onResolveComment: (commentId: string) => void
}

function CommentsSidebar({ session, onAddComment, onResolveComment }: CommentsSidebarProps) {
  const [newCommentContent, setNewCommentContent] = useState('')
  const [newCommentTarget, setNewCommentTarget] = useState<CommentTarget>({ type: 'document' })

  const handleAddComment = useCallback(() => {
    if (!newCommentContent.trim()) return
    onAddComment(newCommentTarget, newCommentContent.trim())
    setNewCommentContent('')
    setNewCommentTarget({ type: 'document' })
  }, [newCommentContent, newCommentTarget, onAddComment])

  const unresolvedComments = session.comments.filter((c) => !c.resolved)
  const resolvedComments = session.comments.filter((c) => c.resolved)

  return (
    <div className="flex h-full w-80 flex-col border-l bg-muted/5">
      {/* Sidebar Header */}
      <div className="border-b bg-muted/40 px-4 py-2 flex items-center justify-between h-10 shrink-0">
        <div className="flex items-center gap-2">
          <MessageSquare className="h-4 w-4 text-muted-foreground" />
          <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Comments</span>
        </div>
        <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
          {unresolvedComments.length} active
        </Badge>
      </div>

      {/* Comments List */}
      <ScrollArea className="flex-1">
        <div className="p-4 space-y-4">
          {/* Unresolved Comments */}
          {unresolvedComments.length > 0 && (
            <div className="space-y-3">
              {unresolvedComments.map((comment) => (
                <Card key={comment.id} className="shadow-none border-none bg-background p-3 ring-1 ring-border">
                  <div className="flex items-start justify-between gap-2 mb-2">
                    <Badge variant="outline" className="text-[10px] h-5 px-1 bg-muted/50 border-none">
                      {getTargetDisplay(comment.target)}
                    </Badge>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-5 w-5 hover:bg-green-500/10 hover:text-green-600"
                      onClick={() => onResolveComment(comment.id)}
                      title="Resolve comment"
                    >
                      <CheckCircle className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                  <p className="text-xs text-foreground whitespace-pre-wrap leading-relaxed">{comment.content}</p>
                  <div className="mt-3 pt-2 border-t border-muted/50 text-[10px] text-muted-foreground flex justify-between">
                    <span>{comment.author === 'user' ? 'You' : 'System'}</span>
                    <span>{new Date(comment.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                  </div>
                </Card>
              ))}
            </div>
          )}

          {/* Resolved Comments */}
          {resolvedComments.length > 0 && (
            <div className="space-y-2 opacity-60">
              <div className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50 px-1">Resolved</div>
              {resolvedComments.map((comment) => (
                <Card key={comment.id} className="shadow-none border-none bg-muted/20 p-2">
                  <div className="flex items-center gap-2 mb-1">
                    <Badge variant="outline" className="text-[9px] h-4 px-1 bg-transparent border-muted-foreground/30 text-muted-foreground">
                      {getTargetDisplay(comment.target)}
                    </Badge>
                    <CheckCircle className="h-3 w-3 text-muted-foreground" />
                  </div>
                  <p className="text-[11px] text-muted-foreground line-clamp-2 italic">{comment.content}</p>
                </Card>
              ))}
            </div>
          )}

          {session.comments.length === 0 && (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <div className="h-12 w-12 rounded-full bg-muted/50 flex items-center justify-center mb-3">
                <MessageSquare className="h-6 w-6 text-muted-foreground/50" />
              </div>
              <p className="text-xs font-medium text-muted-foreground">No feedback yet</p>
              <p className="text-[10px] text-muted-foreground/70 mt-1">Add comments to specific sections or the entire document.</p>
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Add Comment Form */}
      <div className="border-t bg-background p-4 space-y-3 shrink-0">
        <Textarea
          value={newCommentContent}
          onChange={(e) => setNewCommentContent(e.target.value)}
          placeholder="Add your feedback..."
          className="min-h-[80px] resize-none text-xs border-muted focus-visible:ring-primary/20"
        />
        <div className="flex items-center gap-2">
          <select
            value={newCommentTarget.type}
            onChange={(e) => {
              const type = e.target.value as 'document' | 'section' | 'file'
              if (type === 'document') {
                setNewCommentTarget({ type: 'document' })
              } else if (type === 'section') {
                setNewCommentTarget({ type: 'section', id: '' })
              } else {
                setNewCommentTarget({ type: 'file', path: '' })
              }
            }}
            className="flex h-8 rounded-md border border-input bg-background px-2 py-1 text-xs outline-none focus:ring-1 focus:ring-primary/20"
          >
            <option value="document">General</option>
            <option value="section">Section</option>
            <option value="file">File</option>
          </select>
          <Button
            size="sm"
            onClick={handleAddComment}
            disabled={!newCommentContent.trim()}
            className="flex-1 h-8"
          >
            <Plus className="mr-1.5 h-3.5 w-3.5" />
            Comment
          </Button>
        </div>
      </div>
    </div>
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
    <div className="border-t bg-background px-6 py-4 shrink-0">
      <div className="flex items-center gap-3">
        <Button
          variant="default"
          size="sm"
          onClick={onApprove}
          disabled={!canApprove}
          className="bg-green-600 hover:bg-green-700 h-9 px-4 gap-2"
        >
          <CheckCircle className="h-4 w-4" />
          Approve & Proceed
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={onRequestChanges}
          disabled={!canRequestChanges}
          className="border-yellow-500/50 text-yellow-700 dark:text-yellow-300 hover:bg-yellow-500/5 h-9 px-4 gap-2"
        >
          <RefreshCw className="h-4 w-4" />
          Request Changes
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={onReject}
          disabled={!canReject}
          className="ml-auto text-muted-foreground hover:text-destructive hover:bg-destructive/5 h-9 px-4 gap-2"
        >
          <XCircle className="h-4 w-4" />
          Reject
        </Button>
      </div>
    </div>
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

  // Handlers
  const handleSessionSelect = useCallback(
    (sessionId: string) => {
      dispatch({ type: 'SetActiveReviewSession', payload: { session_id: sessionId } })
    },
    [dispatch]
  )

  const handleSectionClick = useCallback((sectionId: string) => {
    // Scroll to section in content view
    const element = document.getElementById(sectionId)
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  }, [])

  const handleAddComment = useCallback(
    (target: CommentTarget, content: string) => {
      if (!activeSession) return
      dispatch({
        type: 'AddReviewComment',
        payload: {
          session_id: activeSession.id,
          target,
          content,
        },
      })
    },
    [activeSession, dispatch]
  )

  const handleResolveComment = useCallback(
    (commentId: string) => {
      if (!activeSession) return
      dispatch({
        type: 'ResolveReviewComment',
        payload: {
          session_id: activeSession.id,
          comment_id: commentId,
        },
      })
    },
    [activeSession, dispatch]
  )

  const handleApprove = useCallback(() => {
    if (!activeSession) return
    dispatch({
      type: 'ApproveReview',
      payload: { session_id: activeSession.id },
    })
  }, [activeSession, dispatch])

  const handleRequestChanges = useCallback(() => {
    if (!activeSession) return
    dispatch({
      type: 'SubmitReviewFeedback',
      payload: { session_id: activeSession.id },
    })
  }, [activeSession, dispatch])

  const handleReject = useCallback(() => {
    if (!activeSession) return
    const reason = prompt('Reason for rejection (optional):')
    dispatch({
      type: 'RejectReview',
      payload: {
        session_id: activeSession.id,
        reason: reason || 'Rejected by user',
      },
    })
  }, [activeSession, dispatch])

  // Loading state
  if (isLoading || reviewGate?.is_loading) {
    return <LoadingState message="Loading review sessions..." />
  }

  // Error state
  if (reviewGate?.error) {
    return (
      <div className="flex h-full flex-col">
        <PageHeader title="Review Error" icon={<XCircle className="h-5 w-5 text-red-500" />} />
        <div className="px-4">
          <ErrorBanner error={reviewGate.error} />
        </div>
      </div>
    )
  }

  // No sessions state
  if (allSessions.length === 0) {
    return (
      <div className="flex h-full flex-col">
        <PageHeader title="ReviewGate" description="Human-in-the-loop review mechanism" icon={<ClipboardCheck className="h-5 w-5 text-blue-500" />} />
        <div className="flex-1 px-4 pb-4">
          <EmptyState
            icon={MessageSquare}
            title="No Review Sessions"
            description="Review sessions will appear here when Claude generates artifacts that require your approval."
          />
        </div>
      </div>
    )
  }

  // Multiple sessions - show session selector
  if (allSessions.length > 1 && !activeSession) {
    return (
      <div className="flex h-full flex-col">
        <PageHeader
          title="Review Sessions"
          description={`${allSessions.length} sessions available for review`}
          icon={<ClipboardCheck className="h-5 w-5 text-blue-500" />}
        />

        <ScrollArea className="flex-1 px-4 pb-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {allSessions.map((session) => (
              <Card
                key={session.id}
                className="cursor-pointer hover:border-primary/50 transition-colors shadow-none bg-muted/10"
                onClick={() => handleSessionSelect(session.id)}
              >
                <CardHeader className="p-4">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex items-center gap-2">
                      <FileText className="h-4 w-4 text-muted-foreground" />
                      <CardTitle className="text-sm font-semibold">
                        {CONTENT_TYPE_LABELS[session.content.content_type]}
                      </CardTitle>
                    </div>
                    <Badge className={`${STATUS_CONFIG[session.status].bgClass} border-none h-5 px-1.5 text-[10px]`}>
                      <span className={STATUS_CONFIG[session.status].textClass}>
                        {STATUS_CONFIG[session.status].label}
                      </span>
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent className="p-4 pt-0">
                  <div className="flex items-center justify-between text-[10px] text-muted-foreground">
                    <span>{new Date(session.created_at).toLocaleDateString()} {new Date(session.created_at).toLocaleTimeString()}</span>
                    <div className="flex items-center gap-3">
                      {session.iteration > 1 && <span>Iter {session.iteration}</span>}
                      <span className="flex items-center gap-1">
                        <MessageSquare className="h-3 w-3" />
                        {session.comments.filter((c) => !c.resolved).length}
                      </span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </ScrollArea>
      </div>
    )
  }

  // Active session view
  if (!activeSession) {
    return (
      <div className="flex h-full flex-col">
        <PageHeader title="ReviewGate" icon={<ClipboardCheck className="h-5 w-5 text-blue-500" />} />
        <div className="flex-1 px-4 pb-4">
          <EmptyState icon={MessageSquare} title="No Active Session" description="Please select a session to begin review." />
        </div>
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col">
      <WorkflowHeader
        title={`${CONTENT_TYPE_LABELS[activeSession.content.content_type]} Review`}
        subtitle={`Iteration ${activeSession.iteration} • ${new Date(activeSession.created_at).toLocaleString()}`}
        icon={<ClipboardCheck className="h-4 w-4 text-blue-500" />}
      >
        {allSessions.length > 1 && (
          <select
            value={activeSession.id}
            onChange={(e) => handleSessionSelect(e.target.value)}
            className="h-7 rounded-md border border-input bg-background px-2 py-0 text-[10px] outline-none focus:ring-1 focus:ring-primary/20"
          >
            {allSessions.map((session) => (
              <option key={session.id} value={session.id}>
                {CONTENT_TYPE_LABELS[session.content.content_type]} - Iter {session.iteration}
              </option>
            ))}
          </select>
        )}
      </WorkflowHeader>

      <div className="flex-1 flex overflow-hidden px-4 pb-4 pt-4">
        <div className="flex-1 flex border rounded-lg overflow-hidden bg-background">
          <div className="flex-1 min-w-0">
            <ContentView session={activeSession} onSectionClick={handleSectionClick} />
          </div>
          <CommentsSidebar
            session={activeSession}
            onAddComment={handleAddComment}
            onResolveComment={handleResolveComment}
          />
        </div>
      </div>

      {/* Action Bar */}
      <ActionBar
        session={activeSession}
        onApprove={handleApprove}
        onRequestChanges={handleRequestChanges}
        onReject={handleReject}
      />
    </div>
  )
}
