import { useState } from 'react'
import {
  PlayArrow as PlayIcon,
  Stop as StopIcon,
  Refresh as RotateCwIcon,
  Description as FileTextIcon,
  ContentCopy as CopyIcon,
  Check as CheckIcon
} from '@mui/icons-material'
import {
  Button,
  Badge,
  Card,
  CardHeader,
  CardContent,
  CardActions,
  Typography,
  Box,
  IconButton,
  Tooltip,
  Chip
} from '@mui/material'
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

  // Map status labels to M3 colors
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return 'success'
      case 'starting': return 'warning'
      case 'error': return 'error'
      default: return 'default'
    }
  }

  return (
    <Card
      variant="outlined"
      sx={{
        width: '100%',
        cursor: 'pointer',
        transition: (theme) => theme.transitions.create(['border-color', 'background-color']),
        borderColor: isActive ? 'primary.main' : 'outlineVariant',
        bgcolor: isActive ? 'secondaryContainer.main' : 'background.paper',
        '&:hover': {
          borderColor: 'primary.main',
          bgcolor: isActive ? 'secondaryContainer.main' : 'action.hover',
        },
      }}
      onClick={() => onSelect?.(service.id)}
    >
      <CardHeader
        sx={{ pb: 1 }}
        title={
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <Typography variant="subtitle1" fontWeight={600}>{service.name}</Typography>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <Box
                sx={{
                  width: 8,
                  height: 8,
                  borderRadius: '50%',
                  bgcolor: isRunning ? 'success.main' : isStarting ? 'warning.main' : 'text.disabled'
                }}
              />
              <Chip
                label={statusLabels[service.status]}
                size="small"
                color={getStatusColor(service.status) as any}
                variant={isRunning ? 'filled' : 'outlined'}
                sx={{ fontSize: '0.65rem', height: 20 }}
              />
            </Box>
          </Box>
        }
        subheader={
          <Typography variant="caption" sx={{ fontFamily: 'monospace', color: 'onSurfaceVariant.main' }}>
            {service.image}
          </Typography>
        }
      />

      <CardContent sx={{ pb: 1 }}>
        {service.port && (
          <Typography variant="body2" sx={{ color: 'onSurfaceVariant.main' }}>
            Port: <Box component="span" sx={{ fontFamily: 'monospace', fontWeight: 600 }}>{service.port}</Box>
          </Typography>
        )}
      </CardContent>

      <CardActions sx={{ px: 2, pb: 2, pt: 0, flexWrap: 'wrap', gap: 1 }}>
        <Button
          variant="contained"
          size="small"
          color={isRunning ? 'error' : 'primary'}
          disabled={!canControl}
          onClick={(e) => {
            e.stopPropagation()
            onToggle?.(service.id)
          }}
          startIcon={isRunning ? <StopIcon /> : <PlayIcon />}
          sx={{ borderRadius: 2 }}
        >
          {isRunning ? 'Stop' : 'Start'}
        </Button>

        <Button
          variant="outlined"
          size="small"
          disabled={!canControl || !isRunning}
          onClick={(e) => {
            e.stopPropagation()
            onRestart?.(service.id)
          }}
          startIcon={<RotateCwIcon />}
          sx={{ borderRadius: 2 }}
        >
          Restart
        </Button>

        <Button
          variant="text"
          size="small"
          onClick={(e) => {
            e.stopPropagation()
            onViewLogs?.(service.id)
          }}
          startIcon={<FileTextIcon />}
        >
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

        <Box sx={{ ml: 'auto' }}>
          <Tooltip title="Copy Connection URL">
            <Button
              size="small"
              variant="text"
              onClick={(e) => {
                e.stopPropagation()
                handleCopyConnectionString()
              }}
              startIcon={copied ? <CheckIcon color="success" /> : <CopyIcon />}
            >
              {copied ? 'Copied' : 'Copy URL'}
            </Button>
          </Tooltip>
        </Box>
      </CardActions>
    </Card>
  )
}
