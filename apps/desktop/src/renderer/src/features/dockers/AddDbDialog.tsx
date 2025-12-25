import { useState } from 'react'
import { Database, Copy, Check } from 'lucide-react'
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

interface AddDbDialogProps {
  serviceId: string
  serviceName: string
  disabled?: boolean
  onCreateDb?: (serviceId: string, dbName: string) => Promise<string>
}

export function AddDbDialog({
  serviceId,
  serviceName,
  disabled,
  onCreateDb,
}: AddDbDialogProps) {
  const [open, setOpen] = useState(false)
  const [dbName, setDbName] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const [connectionString, setConnectionString] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const handleCreate = async () => {
    if (!dbName.trim()) {
      setError('Database name is required')
      return
    }

    // Validate: alphanumeric and underscores only
    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(dbName)) {
      setError('Database name must start with a letter or underscore and contain only alphanumeric characters and underscores')
      return
    }

    setError(null)
    setIsCreating(true)

    try {
      if (onCreateDb) {
        const connStr = await onCreateDb(serviceId, dbName)
        setConnectionString(connStr)
      } else {
        // Mock for now
        const mockConnStr = serviceId.includes('postgres')
          ? `postgresql://postgres:postgres@localhost:5432/${dbName}`
          : serviceId.includes('mysql')
          ? `mysql://root:mysql@localhost:3306/${dbName}`
          : `mongodb://localhost:27017/${dbName}`
        setConnectionString(mockConnStr)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create database')
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
      setDbName('')
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
          <Database className="mr-1 h-3.5 w-3.5" />
          Add DB
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Create Database</DialogTitle>
          <DialogDescription>
            Create a new database in {serviceName}
          </DialogDescription>
        </DialogHeader>

        {!connectionString ? (
          <>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="dbName">Database Name</Label>
                <Input
                  id="dbName"
                  placeholder="my_database"
                  value={dbName}
                  onChange={(e) => setDbName(e.target.value)}
                  disabled={isCreating}
                />
                {error && (
                  <p className="text-sm text-destructive">{error}</p>
                )}
              </div>
            </div>
            <DialogFooter>
              <Button onClick={handleCreate} disabled={isCreating || !dbName.trim()}>
                {isCreating ? 'Creating...' : 'Create Database'}
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
                  Database &quot;{dbName}&quot; created successfully!
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
