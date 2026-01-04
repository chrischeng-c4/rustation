import { useEffect, useState } from 'react'
import { Plus, CheckCircle, Clock, XCircle, GitBranch } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { PageHeader } from '@/components/shared/PageHeader'
import { WorkflowHeader } from '@/components/shared/WorkflowHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { useAppState } from '@/hooks/useAppState'
import { NewChangeDialog } from './NewChangeDialog'
import { ChangeDetailView } from './ChangeDetailView'
import type { Change, ChangeStatus } from '@/types/state'

/**
 * Status badge colors and labels
 */
const STATUS_CONFIG: Record<ChangeStatus, { color: string; label: string; icon: any }> = {
  proposed: { color: 'bg-blue-500', label: 'Proposed', icon: GitBranch },
  planning: { color: 'bg-yellow-500', label: 'Planning', icon: Clock },
  planned: { color: 'bg-purple-500', label: 'Planned', icon: GitBranch },
  implementing: { color: 'bg-orange-500', label: 'Implementing', icon: Clock },
  testing: { color: 'bg-cyan-500', label: 'Testing', icon: Clock },
  done: { color: 'bg-green-500', label: 'Done', icon: CheckCircle },
  archived: { color: 'bg-gray-500', label: 'Archived', icon: CheckCircle },
  cancelled: { color: 'bg-red-500', label: 'Cancelled', icon: XCircle },
  failed: { color: 'bg-red-600', label: 'Failed', icon: XCircle },
}

/**
 * ChangeManagementPanel - CESDD Phase 2 Change Management
 *
 * Allows users to:
 * 1. Create new changes from intent
 * 2. Generate proposals and plans using Claude
 * 3. Track change lifecycle
 */
export function ChangeManagementPanel() {
  const { state, dispatch } = useAppState()
  const [isDialogOpen, setIsDialogOpen] = useState(false)

  // Get current worktree's changes
  const activeProject = state?.projects?.[state?.active_project_index ?? 0]
  const activeWorktree = activeProject?.worktrees?.[activeProject?.active_worktree_index ?? 0]
  const changesState = activeWorktree?.changes
  const changes = changesState?.changes ?? []
  const selectedChangeId = changesState?.selected_change_id
  const selectedChange = changes.find((c) => c.id === selectedChangeId)
  const isLoading = changesState?.is_loading ?? false

  // Refresh changes on mount
  useEffect(() => {
    dispatch({ type: 'RefreshChanges' })
  }, [dispatch])

  const handleCreateChange = async (intent: string) => {
    dispatch({ type: 'CreateChange', payload: { intent } })
    setIsDialogOpen(false)
  }

  const handleSelectChange = (changeId: string) => {
    dispatch({ type: 'SelectChange', payload: { change_id: changeId } })
  }

  const renderChangeCard = (change: Change) => {
    const config = STATUS_CONFIG[change.status]
    const Icon = config.icon
    const isSelected = change.id === selectedChangeId

    return (
      <Card
        key={change.id}
        className={`cursor-pointer transition-colors hover:bg-accent ${
          isSelected ? 'border-primary bg-accent shadow-sm' : ''
        }`}
        onClick={() => handleSelectChange(change.id)}
      >
        <CardHeader className="p-3">
          <div className="flex items-center justify-between gap-2">
            <CardTitle className="text-sm font-medium truncate">{change.name}</CardTitle>
            <Badge variant="secondary" className={`${config.color} text-white shrink-0 h-5 px-1.5`}>
              <Icon className="mr-1 h-3 w-3" />
              {config.label}
            </Badge>
          </div>
          <CardDescription className="line-clamp-2 text-[10px] leading-tight mt-1">{change.intent}</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  // Empty state
  if (!isLoading && changes.length === 0) {
    return (
      <div className="flex h-full flex-col">
        <PageHeader
          title="Change Management"
          description="Manage features with proposal and plan generation"
          icon={<GitBranch className="h-5 w-5 text-blue-500" />}
        />
        <div className="flex-1 px-4 pb-4">
          <EmptyState
            icon={GitBranch}
            title="Start Change Management"
            description="Create changes to manage features with AI-powered proposals and plans."
            action={{
              label: "Create First Change",
              onClick: () => setIsDialogOpen(true),
              icon: Plus
            }}
          />
        </div>

        <NewChangeDialog
          open={isDialogOpen}
          onOpenChange={setIsDialogOpen}
          onSubmit={handleCreateChange}
        />
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col">
      <WorkflowHeader
        title="Change Management"
        subtitle={`${changes.length} active changes in current worktree`}
        icon={<GitBranch className="h-4 w-4 text-blue-500" />}
      >
        <Button size="sm" onClick={() => setIsDialogOpen(true)} className="h-8 gap-1">
          <Plus className="h-3.5 w-3.5" />
          New Change
        </Button>
      </WorkflowHeader>

      <div className="flex flex-1 gap-4 overflow-hidden px-4 pb-4 pt-4">
        {/* Change List (Left) */}
        <div className="w-64 flex-shrink-0 flex flex-col border rounded-lg bg-muted/10">
          <div className="p-3 border-b bg-muted/30">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Active Changes</h3>
          </div>
          {isLoading ? (
            <div className="flex-1 flex items-center justify-center">
              <LoadingState message="" />
            </div>
          ) : (
            <ScrollArea className="flex-1">
              <div className="p-2 space-y-2">
                {changes.map(renderChangeCard)}
              </div>
            </ScrollArea>
          )}
        </div>

        {/* Change Detail (Right) */}
        <div className="flex-1 border rounded-lg overflow-hidden flex flex-col bg-background">
          {selectedChange ? (
            <ChangeDetailView change={selectedChange} />
          ) : (
            <EmptyState
              icon={GitBranch}
              title="No Change Selected"
              description="Select a change from the list on the left to view its details, proposal, and plan."
            />
          )}
        </div>
      </div>

      {/* New Change Dialog */}
      <NewChangeDialog
        open={isDialogOpen}
        onOpenChange={setIsDialogOpen}
        onSubmit={handleCreateChange}
      />
    </div>
  )
}
