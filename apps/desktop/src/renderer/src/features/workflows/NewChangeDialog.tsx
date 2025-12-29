import { useState } from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'

interface NewChangeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSubmit: (intent: string) => void
}

/**
 * NewChangeDialog - Dialog to create a new change from intent
 */
export function NewChangeDialog({ open, onOpenChange, onSubmit }: NewChangeDialogProps) {
  const [intent, setIntent] = useState('')

  const handleSubmit = () => {
    if (intent.trim()) {
      onSubmit(intent.trim())
      setIntent('')
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && e.metaKey) {
      handleSubmit()
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Create New Change</DialogTitle>
          <DialogDescription>
            Describe what you want to accomplish. This will be used to generate a proposal and plan.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="intent">Intent</Label>
            <Textarea
              id="intent"
              placeholder="e.g., Add user authentication with OAuth2 support"
              value={intent}
              onChange={(e) => setIntent(e.target.value)}
              onKeyDown={handleKeyDown}
              rows={4}
              className="resize-none"
            />
            <p className="text-xs text-muted-foreground">
              Be specific about what you want to build. Press Cmd+Enter to submit.
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!intent.trim()}>
            Create Change
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
