import { useEffect, useState } from 'react'
import { Plus, FileText, CheckCircle, Clock, XCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useAppState } from '@/hooks/useAppState'
import { NewChangeDialog } from './NewChangeDialog'
import { ChangeDetailView } from './ChangeDetailView'
import type { Change, ChangeStatus } from '@/types/state'

/**
 * Status badge colors and labels
 */
const STATUS_CONFIG: Record<ChangeStatus, { color: string; label: string; icon: typeof Clock }> = {
  proposed: { color: 'bg-blue-500', label: 'Proposed', icon: FileText },
  planning: { color: 'bg-yellow-500', label: 'Planning', icon: Clock },
  planned: { color: 'bg-purple-500', label: 'Planned', icon: FileText },
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
          isSelected ? 'border-primary bg-accent' : ''
        }`}
        onClick={() => handleSelectChange(change.id)}
      >
        <CardHeader className="p-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm font-medium">{change.name}</CardTitle>
            <Badge variant="secondary" className={`${config.color} text-white`}>
              <Icon className="mr-1 h-3 w-3" />
              {config.label}
            </Badge>
          </div>
          <CardDescription className="line-clamp-2 text-xs">{change.intent}</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  return (
    <div className="flex h-full gap-4">
      {/* Change List (Left) */}
      <div className="w-72 flex-shrink-0">
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-lg font-semibold">Changes</h2>
          <Button size="sm" onClick={() => setIsDialogOpen(true)}>
            <Plus className="mr-1 h-4 w-4" />
            New
          </Button>
        </div>

        {isLoading ? (
          <div className="flex h-32 items-center justify-center text-muted-foreground">
            Loading changes...
          </div>
        ) : changes.length === 0 ? (
          <Card className="border-dashed">
            <CardContent className="flex flex-col items-center justify-center py-8 text-center">
              <FileText className="mb-2 h-8 w-8 text-muted-foreground" />
              <p className="text-sm text-muted-foreground">No changes yet</p>
              <p className="text-xs text-muted-foreground">
                Create a change to start managing your features
              </p>
            </CardContent>
          </Card>
        ) : (
          <ScrollArea className="h-[calc(100vh-200px)]">
            <div className="space-y-2 pr-2">
              {changes.map(renderChangeCard)}
            </div>
          </ScrollArea>
        )}
      </div>

      {/* Change Detail (Right) */}
      <div className="flex-1">
        {selectedChange ? (
          <ChangeDetailView change={selectedChange} />
        ) : (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            Select a change to view details
          </div>
        )}
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
