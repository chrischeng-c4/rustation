import { useState } from 'react'
import { Play, Square, RotateCw, FileText, Copy, Check } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from '@/components/ui/card'
import { cn } from '@/lib/utils'
import type { DockerServiceInfo } from '@/types/state'
import { statusColors, statusLabels } from '@/types/state'
import { AddDbDialog } from './AddDbDialog'
import { AddVhostDialog } from './AddVhostDialog'

// Connection string templates with default credentials
function getConnectionString(service: DockerServiceInfo): string {
  const host = 'localhost'
  const port = service.port

  switch (service.id) {
    case 'rstn-postgres':
      return `postgresql://postgres:postgres@${host}:${port}/postgres`
    case 'rstn-mysql':
      return `mysql://root:mysql@${host}:${port}`
    case 'rstn-mongodb':
      return `mongodb://${host}:${port}`
    case 'rstn-redis':
      return `redis://${host}:${port}`
    case 'rstn-rabbitmq':
      return `amqp://guest:guest@${host}:${port}`
    case 'rstn-nats':
      return `nats://${host}:${port}`
    default:
      return `${host}:${port}`
  }
}

interface DockerServiceCardProps {
  service: DockerServiceInfo
  isActive?: boolean
  onSelect?: (id: string) => void
  onToggle?: (id: string) => void
  onRestart?: (id: string) => void
  onViewLogs?: (id: string) => void
  onCreateDb?: (serviceId: string, dbName: string) => Promise<string>
  onCreateVhost?: (serviceId: string, vhostName: string) => Promise<string>
}

export function DockerServiceCard({
  service,
  isActive = false,
  onSelect,
  onToggle,
  onRestart,
  onViewLogs,
  onCreateDb,
  onCreateVhost,
}: DockerServiceCardProps) {
  const [copied, setCopied] = useState(false)
  const isRunning = service.status === 'running'
  const isStarting = service.status === 'starting'
  const isRstnManaged = service.is_rstn_managed
  const canControl = !isStarting && isRstnManaged

  const handleCopyConnectionString = async () => {
    const connectionString = getConnectionString(service)
    await navigator.clipboard.writeText(connectionString)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Card
      className={cn(
        'w-full cursor-pointer transition-colors hover:border-muted-foreground/50',
        isActive && 'border-primary bg-primary/5'
      )}
      onClick={() => onSelect?.(service.id)}
    >
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">{service.name}</CardTitle>
          <div className="flex items-center gap-2">
            <span
              className={`h-2.5 w-2.5 rounded-full ${statusColors[service.status]}`}
            />
            <Badge variant={isRunning ? 'default' : 'secondary'}>
              {statusLabels[service.status]}
            </Badge>
          </div>
        </div>
        <CardDescription className="font-mono text-xs">
          {service.image}
        </CardDescription>
      </CardHeader>

      <CardContent className="pb-3">
        {service.port && (
          <p className="text-sm text-muted-foreground">
            Port: <span className="font-mono">{service.port}</span>
          </p>
        )}
      </CardContent>

      <CardFooter className="flex flex-wrap gap-2">
        <Button
          variant={isRunning ? 'destructive' : 'default'}
          size="sm"
          disabled={!canControl}
          onClick={(e) => {
            e.stopPropagation()
            onToggle?.(service.id)
          }}
        >
          {isRunning ? (
            <>
              <Square className="mr-1 h-3.5 w-3.5" />
              Stop
            </>
          ) : (
            <>
              <Play className="mr-1 h-3.5 w-3.5" />
              Start
            </>
          )}
        </Button>

        <Button
          variant="outline"
          size="sm"
          disabled={!canControl || !isRunning}
          onClick={(e) => {
            e.stopPropagation()
            onRestart?.(service.id)
          }}
        >
          <RotateCw className="mr-1 h-3.5 w-3.5" />
          Restart
        </Button>

        <Button
          variant="ghost"
          size="sm"
          onClick={(e) => {
            e.stopPropagation()
            onViewLogs?.(service.id)
          }}
        >
          <FileText className="mr-1 h-3.5 w-3.5" />
          Logs
        </Button>

        {/* Conditional Add DB button for databases */}
        {service.service_type === 'Database' && (
          <AddDbDialog
            serviceId={service.id}
            serviceName={service.name}
            disabled={!isRunning}
            onCreateDb={onCreateDb}
          />
        )}

        {/* Conditional Add vhost button for RabbitMQ */}
        {service.service_type === 'MessageBroker' && (
          <AddVhostDialog
            serviceId={service.id}
            disabled={!isRunning}
            onCreateVhost={onCreateVhost}
          />
        )}

        <Button
          variant="ghost"
          size="sm"
          onClick={(e) => {
            e.stopPropagation()
            handleCopyConnectionString()
          }}
          className="ml-auto"
        >
          {copied ? (
            <>
              <Check className="mr-1 h-3.5 w-3.5 text-green-500" />
              Copied
            </>
          ) : (
            <>
              <Copy className="mr-1 h-3.5 w-3.5" />
              Copy URL
            </>
          )}
        </Button>
      </CardFooter>
    </Card>
  )
}
