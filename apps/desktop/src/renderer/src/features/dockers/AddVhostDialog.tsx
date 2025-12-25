import { useState } from 'react'
import { Server, Copy, Check } from 'lucide-react'
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
  DialogTrigger,
} from '@/components/ui/dialog'

interface AddVhostDialogProps {
  serviceId: string
  disabled?: boolean
  onCreateVhost?: (serviceId: string, vhostName: string) => Promise<string>
}

export function AddVhostDialog({
  serviceId,
  disabled,
  onCreateVhost,
}: AddVhostDialogProps) {
  const [open, setOpen] = useState(false)
  const [vhostName, setVhostName] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const [connectionString, setConnectionString] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const handleCreate = async () => {
    if (!vhostName.trim()) {
      setError('Vhost name is required')
      return
    }

    // Validate: alphanumeric, underscores, and hyphens only
    if (!/^[a-zA-Z_][a-zA-Z0-9_-]*$/.test(vhostName)) {
      setError('Vhost name must start with a letter or underscore and contain only alphanumeric characters, underscores, and hyphens')
      return
    }

    setError(null)
    setIsCreating(true)

    try {
      if (onCreateVhost) {
        const connStr = await onCreateVhost(serviceId, vhostName)
        setConnectionString(connStr)
      } else {
        // Mock for now
        setConnectionString(`amqp://guest:guest@localhost:5672/${vhostName}`)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create vhost')
    } finally {
      setIsCreating(false)
    }
  }

  const handleCopy = async () => {
    if (connectionString) {
      await navigator.clipboard.writeText(connectionString)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }

  const handleOpenChange = (isOpen: boolean) => {
    setOpen(isOpen)
    if (!isOpen) {
      // Reset state when closing
      setVhostName('')
      setConnectionString(null)
      setError(null)
      setCopied(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          disabled={disabled}
          onClick={(e) => e.stopPropagation()}
        >
          <Server className="mr-1 h-3.5 w-3.5" />
          Add vhost
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Create Virtual Host</DialogTitle>
          <DialogDescription>
            Create a new virtual host in RabbitMQ
          </DialogDescription>
        </DialogHeader>

        {!connectionString ? (
          <>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="vhostName">Vhost Name</Label>
                <Input
                  id="vhostName"
                  placeholder="my_vhost"
                  value={vhostName}
                  onChange={(e) => setVhostName(e.target.value)}
                  disabled={isCreating}
                />
                {error && (
                  <p className="text-sm text-destructive">{error}</p>
                )}
              </div>
            </div>
            <DialogFooter>
              <Button onClick={handleCreate} disabled={isCreating || !vhostName.trim()}>
                {isCreating ? 'Creating...' : 'Create Vhost'}
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label>Connection String</Label>
                <div className="flex gap-2">
                  <Input
                    value={connectionString}
                    readOnly
                    className="font-mono text-xs"
                  />
                  <Button variant="outline" size="icon" onClick={handleCopy}>
                    {copied ? (
                      <Check className="h-4 w-4 text-green-500" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
                <p className="text-sm text-muted-foreground">
                  Vhost &quot;{vhostName}&quot; created successfully!
                </p>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => handleOpenChange(false)}>
                Close
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}
