import { useState, useEffect } from 'react'
import { GitBranch, Plus, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'

interface BranchInfo {
  name: string
  hasWorktree: boolean
  isCurrent: boolean
}

type DialogMode = 'select' | 'new'

interface AddWorktreeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  repoPath: string
  onAddFromBranch: (branch: string) => Promise<void>
  onAddNewBranch: (branch: string) => Promise<void>
}

export function AddWorktreeDialog({
  open,
  onOpenChange,
  repoPath,
  onAddFromBranch,
  onAddNewBranch,
}: AddWorktreeDialogProps) {
  const [mode, setMode] = useState<DialogMode>('select')
  const [branches, setBranches] = useState<BranchInfo[]>([])
  const [isLoadingBranches, setIsLoadingBranches] = useState(false)
  const [selectedBranch, setSelectedBranch] = useState<string | null>(null)
  const [newBranchName, setNewBranchName] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Load branches when dialog opens
  useEffect(() => {
    if (open && repoPath) {
      loadBranches()
    }
  }, [open, repoPath])

  const loadBranches = async () => {
    setIsLoadingBranches(true)
    setError(null)
    try {
      const branchList = await window.api.worktree.listBranches(repoPath)
      setBranches(branchList)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load branches')
    } finally {
      setIsLoadingBranches(false)
    }
  }

  const handleAddFromBranch = async () => {
    if (!selectedBranch) return

    setIsCreating(true)
    setError(null)
    try {
      await onAddFromBranch(selectedBranch)
      handleClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create worktree')
    } finally {
      setIsCreating(false)
    }
  }

  const handleAddNewBranch = async () => {
    if (!newBranchName.trim()) {
      setError('Branch name is required')
      return
    }

    // Validate branch name (basic validation)
    if (!/^[a-zA-Z0-9._/-]+$/.test(newBranchName)) {
      setError('Invalid branch name. Use only letters, numbers, dots, underscores, slashes, and hyphens.')
      return
    }

    setIsCreating(true)
    setError(null)
    try {
      await onAddNewBranch(newBranchName)
      handleClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create worktree')
    } finally {
      setIsCreating(false)
    }
  }

  const handleClose = () => {
    onOpenChange(false)
    // Reset state after close animation
    setTimeout(() => {
      setMode('select')
      setSelectedBranch(null)
      setNewBranchName('')
      setError(null)
      setBranches([])
    }, 200)
  }

  // Filter out branches that already have worktrees
  const availableBranches = branches.filter(b => !b.hasWorktree)

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Add Worktree</DialogTitle>
          <DialogDescription>
            {mode === 'select'
              ? 'Select an existing branch or create a new one'
              : 'Enter a name for the new branch'}
          </DialogDescription>
        </DialogHeader>

        {mode === 'select' ? (
          <>
            {isLoadingBranches ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : (
              <div className="grid gap-4 py-4">
                {/* Branch List */}
                <div className="grid gap-2">
                  <Label>Available Branches</Label>
                  {availableBranches.length === 0 ? (
                    <p className="text-sm text-muted-foreground py-4 text-center">
                      All branches already have worktrees
                    </p>
                  ) : (
                    <ScrollArea className="h-[200px] rounded-md border">
                      <div className="p-2 space-y-1">
                        {availableBranches.map((branch) => (
                          <button
                            key={branch.name}
                            onClick={() => setSelectedBranch(branch.name)}
                            className={cn(
                              'w-full flex items-center gap-2 px-3 py-2 rounded-md text-sm text-left',
                              'hover:bg-accent transition-colors',
                              selectedBranch === branch.name && 'bg-accent'
                            )}
                          >
                            <GitBranch className="h-4 w-4 text-muted-foreground shrink-0" />
                            <span className="truncate">{branch.name}</span>
                            {branch.isCurrent && (
                              <span className="text-xs text-muted-foreground">(current)</span>
                            )}
                          </button>
                        ))}
                      </div>
                    </ScrollArea>
                  )}
                </div>

                {/* Create New Branch Option */}
                <Button
                  variant="outline"
                  className="w-full"
                  onClick={() => setMode('new')}
                >
                  <Plus className="h-4 w-4 mr-2" />
                  Create New Branch
                </Button>

                {error && (
                  <p className="text-sm text-destructive">{error}</p>
                )}
              </div>
            )}
            <DialogFooter>
              <Button variant="outline" onClick={handleClose}>
                Cancel
              </Button>
              <Button
                onClick={handleAddFromBranch}
                disabled={!selectedBranch || isCreating}
              >
                {isCreating ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Creating...
                  </>
                ) : (
                  'Add Worktree'
                )}
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="branchName">Branch Name</Label>
                <Input
                  id="branchName"
                  placeholder="feature/my-new-feature"
                  value={newBranchName}
                  onChange={(e) => setNewBranchName(e.target.value)}
                  disabled={isCreating}
                />
                <p className="text-xs text-muted-foreground">
                  A new branch will be created from the current HEAD
                </p>
              </div>
              {error && (
                <p className="text-sm text-destructive">{error}</p>
              )}
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setMode('select')}>
                Back
              </Button>
              <Button
                onClick={handleAddNewBranch}
                disabled={!newBranchName.trim() || isCreating}
              >
                {isCreating ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Creating...
                  </>
                ) : (
                  'Create & Add Worktree'
                )}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}
