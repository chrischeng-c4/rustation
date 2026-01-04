import { FileText, Play, Check, X, Clock, Archive, RefreshCw, Rocket, ClipboardCheck, MessageSquare, ThumbsUp, ThumbsDown, ChevronDown, FileCode } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { WorkflowHeader } from '@/components/shared/WorkflowHeader'
import { useAppState } from '@/hooks/useAppState'
import { ContextFilesInput } from './ContextFilesInput'
import type { Change, ReviewSession, ReviewStatus } from '@/types/state'

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

  const statusConfig: Record<ReviewStatus, { label: string; color: string; icon: typeof Clock }> = {
    pending: { label: 'Pending Review', color: 'bg-gray-500/10 text-gray-700', icon: Clock },
    reviewing: { label: 'In Review', color: 'bg-blue-500/10 text-blue-700', icon: ClipboardCheck },
    iterating: { label: 'Iterating', color: 'bg-yellow-500/10 text-yellow-700', icon: RefreshCw },
    approved: { label: 'Approved', color: 'bg-green-500/10 text-green-700', icon: Check },
    rejected: { label: 'Rejected', color: 'bg-red-500/10 text-red-700', icon: X },
  }

  const config = statusConfig[session.status]
  const Icon = config.icon

  return (
    <Badge variant="outline" className={`ml-2 gap-1 ${config.color}`}>
      <Icon className="h-3 w-3" />
      {type === 'proposal' ? 'Proposal' : 'Plan'}: {config.label}
      {session.comments.filter((c) => !c.resolved).length > 0 && (
        <span className="ml-1 flex items-center gap-0.5">
          <MessageSquare className="h-3 w-3" />
          {session.comments.filter((c) => !c.resolved).length}
        </span>
      )}
    </Badge>
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
  // Only show controls if review is in progress (not already approved/rejected)
  if (session.status === 'approved' || session.status === 'rejected') {
    return null
  }

  const unresolvedComments = session.comments.filter((c) => !c.resolved).length

  return (
    <div className="border-t bg-muted/30 p-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <ClipboardCheck className="h-4 w-4" />
          <span>Review required</span>
          {unresolvedComments > 0 && (
            <Badge variant="secondary" className="gap-1">
              <MessageSquare className="h-3 w-3" />
              {unresolvedComments} comment{unresolvedComments > 1 ? 's' : ''}
            </Badge>
          )}
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="outline" onClick={onReject} className="gap-1">
            <ThumbsDown className="h-3 w-3" />
            Reject
          </Button>
          <Button size="sm" onClick={onApprove} className="gap-1 bg-green-600 hover:bg-green-700">
            <ThumbsUp className="h-3 w-3" />
            Approve
          </Button>
        </div>
      </div>
    </div>
  )
}

/**
 * ChangeDetailView - Shows change details, proposal, and plan
 */
export function ChangeDetailView({ change }: ChangeDetailViewProps) {
  const { state, dispatch } = useAppState()

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

  return (
    <div className="h-full flex flex-col">
      <WorkflowHeader
        title={change.name}
        subtitle={change.intent}
        status={change.status}
        statusColor={STATUS_CONFIG[change.status as ChangeStatus]?.color || "bg-gray-500"}
      >
        {/* Review Status Badges */}
        <div className="flex flex-wrap gap-2">
          {proposalReviewSession && <ReviewStatusBadge session={proposalReviewSession} type="proposal" />}
          {planReviewSession && <ReviewStatusBadge session={planReviewSession} type="plan" />}
        </div>
      </WorkflowHeader>

      <div className="flex-1 overflow-hidden p-4">
        {/* Context Files Section */}
        {worktree?.path && (
          <Collapsible className="mb-4">
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-md border bg-muted/30 px-3 py-2 text-sm hover:bg-muted/50 transition-colors [&[data-state=open]>svg]:rotate-180">
              <div className="flex items-center gap-2">
                <FileCode className="h-4 w-4" />
                <span>Context Files</span>
                {(change.context_files?.length ?? 0) > 0 && (
                  <Badge variant="secondary" className="ml-2">
                    {change.context_files.length}
                  </Badge>
                )}
              </div>
              <ChevronDown className="h-4 w-4 transition-transform duration-200" />
            </CollapsibleTrigger>
            <CollapsibleContent className="mt-2 rounded-md border p-3">
              <ContextFilesInput
                changeId={change.id}
                files={change.context_files ?? []}
                projectRoot={worktree.path}
              />
            </CollapsibleContent>
          </Collapsible>
        )}

        <Tabs defaultValue="proposal" className="h-full">
          <TabsList className="mb-4">
            <TabsTrigger value="proposal" className="gap-1">
              <FileText className="h-4 w-4" />
              Proposal
              {proposalReviewSession?.status === 'reviewing' && (
                <span className="ml-1 h-2 w-2 rounded-full bg-blue-500" />
              )}
            </TabsTrigger>
            <TabsTrigger value="plan" className="gap-1">
              <FileText className="h-4 w-4" />
              Plan
              {planReviewSession?.status === 'reviewing' && (
                <span className="ml-1 h-2 w-2 rounded-full bg-blue-500" />
              )}
            </TabsTrigger>
            <TabsTrigger value="implementation" className="gap-1">
              <Rocket className="h-4 w-4" />
              Implementation
            </TabsTrigger>
          </TabsList>

          <TabsContent value="proposal" className="h-[calc(100%-50px)]">
            {isPlanning && change.streaming_output ? (
              <ScrollArea className="h-full rounded-md border p-4">
                <div className="flex items-center gap-2 mb-2 text-yellow-600">
                  <Clock className="h-4 w-4 animate-spin" />
                  <span className="text-sm font-medium">Generating...</span>
                </div>
                <pre className="whitespace-pre-wrap text-sm">{change.streaming_output}</pre>
              </ScrollArea>
            ) : hasProposal ? (
              <div className="flex h-full flex-col rounded-md border">
                <ScrollArea className="flex-1 p-4">
                  <pre className="whitespace-pre-wrap text-sm">{change.proposal}</pre>
                </ScrollArea>
                {/* Inline Review Controls */}
                {proposalReviewSession && (
                  <InlineReviewControls
                    session={proposalReviewSession}
                    onApprove={handleApproveProposalReview}
                    onReject={handleRejectProposalReview}
                  />
                )}
              </div>
            ) : (
              <div className="flex h-full flex-col items-center justify-center gap-4">
                <FileText className="h-12 w-12 text-muted-foreground" />
                <p className="text-muted-foreground">No proposal generated yet</p>
                {canGenerateProposal && (
                  <Button onClick={handleGenerateProposal}>
                    <Play className="mr-2 h-4 w-4" />
                    Generate Proposal
                  </Button>
                )}
              </div>
            )}
          </TabsContent>

          <TabsContent value="plan" className="h-[calc(100%-50px)]">
            {isPlanning && !hasProposal && change.streaming_output ? (
              <ScrollArea className="h-full rounded-md border p-4">
                <div className="flex items-center gap-2 mb-2 text-yellow-600">
                  <Clock className="h-4 w-4 animate-spin" />
                  <span className="text-sm font-medium">Generating plan...</span>
                </div>
                <pre className="whitespace-pre-wrap text-sm">{change.streaming_output}</pre>
              </ScrollArea>
            ) : hasPlan ? (
              <div className="flex h-full flex-col rounded-md border">
                <ScrollArea className="flex-1 p-4">
                  <pre className="whitespace-pre-wrap text-sm">{change.plan}</pre>
                </ScrollArea>
                {/* Inline Review Controls */}
                {planReviewSession && (
                  <InlineReviewControls
                    session={planReviewSession}
                    onApprove={handleApprovePlanReview}
                    onReject={handleRejectPlanReview}
                  />
                )}
              </div>
            ) : (
              <div className="flex h-full flex-col items-center justify-center gap-4">
                <FileText className="h-12 w-12 text-muted-foreground" />
                <p className="text-muted-foreground">
                  {hasProposal ? 'No plan generated yet' : 'Generate a proposal first'}
                </p>
                {canGeneratePlan && (
                  <Button onClick={handleGeneratePlan}>
                    <Play className="mr-2 h-4 w-4" />
                    Generate Plan
                  </Button>
                )}
              </div>
            )}
          </TabsContent>

          <TabsContent value="implementation" className="h-[calc(100%-50px)]">
            {isImplementing && change.streaming_output ? (
              <ScrollArea className="h-full rounded-md border p-4">
                <div className="flex items-center gap-2 mb-2 text-blue-600">
                  <Rocket className="h-4 w-4 animate-pulse" />
                  <span className="text-sm font-medium">Implementing...</span>
                </div>
                <pre className="whitespace-pre-wrap text-sm">{change.streaming_output}</pre>
              </ScrollArea>
            ) : change.status === 'done' ? (
              <ScrollArea className="h-full rounded-md border p-4">
                <div className="flex items-center gap-2 mb-2 text-green-600">
                  <Check className="h-4 w-4" />
                  <span className="text-sm font-medium">Implementation Complete</span>
                </div>
                <pre className="whitespace-pre-wrap text-sm">{change.streaming_output}</pre>
              </ScrollArea>
            ) : change.status === 'failed' ? (
              <ScrollArea className="h-full rounded-md border p-4">
                <div className="flex items-center gap-2 mb-2 text-red-600">
                  <X className="h-4 w-4" />
                  <span className="text-sm font-medium">Implementation Failed</span>
                </div>
                <pre className="whitespace-pre-wrap text-sm">{change.streaming_output}</pre>
              </ScrollArea>
            ) : (
              <div className="flex h-full flex-col items-center justify-center gap-4">
                <Rocket className="h-12 w-12 text-muted-foreground" />
                <p className="text-muted-foreground">
                  {hasPlan ? 'Ready to implement' : 'Generate a plan first'}
                </p>
                {canExecute && (
                  <Button onClick={handleExecutePlan} className="bg-blue-600 hover:bg-blue-700">
                    <Rocket className="mr-2 h-4 w-4" />
                    Execute Plan
                  </Button>
                )}
              </div>
            )}
          </TabsContent>
        </Tabs>

        {/* Action Buttons */}
        <div className="mt-4 flex gap-2 border-t pt-4">
          {canApprove && (
            <Button onClick={handleApprovePlan} className="bg-green-600 hover:bg-green-700">
              <Check className="mr-2 h-4 w-4" />
              Approve Plan
            </Button>
          )}
          {canExecute && (
            <Button onClick={handleExecutePlan} className="bg-blue-600 hover:bg-blue-700">
              <Rocket className="mr-2 h-4 w-4" />
              Execute Plan
            </Button>
          )}
          {isImplementing && (
            <Badge variant="secondary" className="px-3 py-1">
              <Rocket className="mr-2 h-4 w-4 animate-pulse" />
              Implementing...
            </Badge>
          )}
          {canSyncAndArchive && (
            <>
              <Button onClick={handleSyncContext} variant="outline">
                <RefreshCw className="mr-2 h-4 w-4" />
                Sync to Context
              </Button>
              <Button onClick={handleArchive} className="bg-blue-600 hover:bg-blue-700">
                <Archive className="mr-2 h-4 w-4" />
                Archive
              </Button>
            </>
          )}
          {isArchived && (
            <Badge variant="secondary" className="px-3 py-1">
              <Archive className="mr-2 h-4 w-4" />
              Archived
            </Badge>
          )}
          {canCancel && (
            <Button variant="destructive" onClick={handleCancelChange}>
              <X className="mr-2 h-4 w-4" />
              Cancel Change
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}
