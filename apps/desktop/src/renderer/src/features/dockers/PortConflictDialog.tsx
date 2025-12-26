import { useState, useEffect } from 'react'
import { AlertTriangle, Container } from 'lucide-react'
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

interface PendingConflict {
  service_id: string
  conflict: {
    requested_port: number
    conflicting_container: {
      id: string
      name: string
      image: string
      is_rstn_managed: boolean
    }
    suggested_port: number
  }
}

interface PortConflictDialogProps {
  pendingConflict: PendingConflict | null
  onResolveWithPort: (serviceId: string, port: number) => void
  onResolveByStoppingContainer: (containerId: string, serviceId: string) => void
  onCancel: () => void
}

export function PortConflictDialog({
  pendingConflict,
  onResolveWithPort,
  onResolveByStoppingContainer,
  onCancel,
}: PortConflictDialogProps) {
  const [resolution, setResolution] = useState<'alt-port' | 'stop-container'>('alt-port')
  const [customPort, setCustomPort] = useState<string>('')
  const [isResolving, setIsResolving] = useState(false)

  // Reset state when dialog opens with new conflict
  useEffect(() => {
    if (pendingConflict) {
      setResolution('alt-port')
      setCustomPort(String(pendingConflict.conflict.suggested_port))
      setIsResolving(false)
    }
  }, [pendingConflict])

  if (!pendingConflict) return null

  const { service_id, conflict } = pendingConflict
  const { requested_port, conflicting_container, suggested_port } = conflict
  const canStopContainer = conflicting_container.is_rstn_managed

  const handleResolve = async () => {
    setIsResolving(true)
    try {
      if (resolution === 'alt-port') {
        const port = parseInt(customPort, 10)
        if (isNaN(port) || port < 1 || port > 65535) {
          return
        }
        onResolveWithPort(service_id, port)
      } else {
        onResolveByStoppingContainer(conflicting_container.id, service_id)
      }
    } finally {
      setIsResolving(false)
    }
  }

  const isValidPort = () => {
    const port = parseInt(customPort, 10)
    return !isNaN(port) && port >= 1 && port <= 65535
  }

  return (
    <Dialog open={!!pendingConflict} onOpenChange={(open) => !open && onCancel()}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-yellow-500" />
            Port Conflict Detected
          </DialogTitle>
          <DialogDescription>
            Port {requested_port} is already in use. Choose how to resolve this conflict.
          </DialogDescription>
        </DialogHeader>

        {/* Conflicting container info */}
        <div className="rounded-lg border bg-muted/40 p-3">
          <div className="flex items-center gap-2 text-sm font-medium">
            <Container className="h-4 w-4" />
            {conflicting_container.name}
          </div>
          <div className="mt-1 text-xs text-muted-foreground">
            Image: {conflicting_container.image}
          </div>
        </div>

        {/* Resolution options - using simple buttons instead of radio group */}
        <div className="space-y-3">
          {/* Option 1: Use alternative port */}
          <div
            className={`rounded-lg border p-3 cursor-pointer transition-colors ${
              resolution === 'alt-port' ? 'border-primary bg-primary/5' : 'hover:bg-muted/40'
            }`}
            onClick={() => setResolution('alt-port')}
          >
            <Label className="font-medium cursor-pointer">
              Use alternative port
            </Label>
            <div className="mt-2 flex items-center gap-2">
              <Input
                type="number"
                min={1}
                max={65535}
                value={customPort}
                onChange={(e) => setCustomPort(e.target.value)}
                disabled={resolution !== 'alt-port' || isResolving}
                className="w-24"
                onClick={(e) => e.stopPropagation()}
              />
              <span className="text-sm text-muted-foreground">
                (suggested: {suggested_port})
              </span>
            </div>
          </div>

          {/* Option 2: Stop conflicting container */}
          <div
            className={`rounded-lg border p-3 transition-colors ${
              !canStopContainer
                ? 'opacity-50 cursor-not-allowed'
                : resolution === 'stop-container'
                ? 'border-primary bg-primary/5 cursor-pointer'
                : 'hover:bg-muted/40 cursor-pointer'
            }`}
            onClick={() => canStopContainer && setResolution('stop-container')}
          >
            <Label className={`font-medium ${!canStopContainer ? 'text-muted-foreground' : 'cursor-pointer'}`}>
              Stop conflicting container and retry
            </Label>
            {!canStopContainer && (
              <p className="mt-1 text-xs text-muted-foreground">
                This container is not managed by rstn and cannot be stopped here.
              </p>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onCancel} disabled={isResolving}>
            Cancel
          </Button>
          <Button
            onClick={handleResolve}
            disabled={isResolving || (resolution === 'alt-port' && !isValidPort())}
          >
            {isResolving ? 'Resolving...' : 'Continue'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
