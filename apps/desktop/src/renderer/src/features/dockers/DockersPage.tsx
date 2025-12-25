import { useState, useEffect, useCallback } from 'react'
import { RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { DockerServiceCard } from './DockerServiceCard'
import { DockerLogSheet } from './DockerLogSheet'
import type { DockerService } from '@/types/docker'

// Mock data for development (will be replaced with napi-rs calls)
const MOCK_SERVICES: DockerService[] = [
  { id: 'rstn-postgres', name: 'PostgreSQL', image: 'postgres:16-alpine', status: 'running', port: 5432, service_type: 'Database' },
  { id: 'rstn-mysql', name: 'MySQL', image: 'mysql:8', status: 'stopped', port: 3306, service_type: 'Database' },
  { id: 'rstn-mongodb', name: 'MongoDB', image: 'mongo:7', status: 'stopped', port: 27017, service_type: 'Database' },
  { id: 'rstn-redis', name: 'Redis', image: 'redis:7-alpine', status: 'running', port: 6379, service_type: 'Cache' },
  { id: 'rstn-rabbitmq', name: 'RabbitMQ', image: 'rabbitmq:3-management', status: 'stopped', port: 5672, service_type: 'MessageBroker' },
  { id: 'rstn-nats', name: 'NATS', image: 'nats:latest', status: 'stopped', port: 4222, service_type: 'Other' },
]

export function DockersPage() {
  const [services, setServices] = useState<DockerService[]>([])
  const [selectedServiceId, setSelectedServiceId] = useState<string | null>(null)
  const [logPanelOpen, setLogPanelOpen] = useState(false)
  const [logs, setLogs] = useState<string[]>([])
  const [isRefreshing, setIsRefreshing] = useState(false)

  const selectedService = services.find((s) => s.id === selectedServiceId)

  // Load initial services
  useEffect(() => {
    loadServices()
  }, [])

  const loadServices = useCallback(async () => {
    console.log('[FE] loadServices called')
    try {
      // TODO: Replace with napi-rs call
      // const services = await window.api.docker.listServices()
      setServices(MOCK_SERVICES)
    } catch (error) {
      console.warn('[FE] Failed to load services:', error)
      setServices(MOCK_SERVICES)
    }
  }, [])

  const handleToggle = useCallback(async (id: string) => {
    console.log('[FE] handleToggle called:', id)
    try {
      // TODO: Replace with napi-rs call
      // await window.api.docker.toggleService(id)
      setServices((prev) =>
        prev.map((s) =>
          s.id === id
            ? { ...s, status: s.status === 'running' ? 'stopped' : 'running' }
            : s
        )
      )
    } catch (error) {
      console.error('[FE] toggleService failed:', error)
    }
  }, [])

  const handleRestart = useCallback(async (id: string) => {
    console.log('[FE] handleRestart called:', id)
    try {
      // TODO: Replace with napi-rs call
      setServices((prev) =>
        prev.map((s) => (s.id === id ? { ...s, status: 'starting' } : s))
      )
      setTimeout(() => {
        setServices((prev) =>
          prev.map((s) => (s.id === id ? { ...s, status: 'running' } : s))
        )
      }, 500)
    } catch (error) {
      console.error('[FE] restartService failed:', error)
    }
  }, [])

  const handleViewLogs = useCallback(async (id: string) => {
    setSelectedServiceId(id)
    setLogPanelOpen(true)
    await refreshLogs(id)
  }, [])

  const refreshLogs = useCallback(async (id?: string) => {
    const serviceId = id || selectedServiceId
    if (!serviceId) return

    setIsRefreshing(true)
    try {
      // TODO: Replace with napi-rs call
      // const logs = await window.api.docker.getLogs(serviceId, 100)
      setLogs([
        `[${new Date().toISOString()}] Container started`,
        '[INFO] Initializing...',
        '[INFO] Ready to accept connections',
      ])
    } catch (error) {
      console.error('[FE] getLogs failed:', error)
    } finally {
      setIsRefreshing(false)
    }
  }, [selectedServiceId])

  const handleCloseLogPanel = useCallback(() => {
    setLogPanelOpen(false)
    setSelectedServiceId(null)
    setLogs([])
  }, [])

  const handleRefreshAll = useCallback(async () => {
    setIsRefreshing(true)
    try {
      await loadServices()
    } finally {
      setIsRefreshing(false)
    }
  }, [loadServices])

  const handleCreateDb = useCallback(async (serviceId: string, dbName: string): Promise<string> => {
    console.log('[FE] handleCreateDb called:', serviceId, dbName)
    // TODO: Replace with napi-rs call
    // const connectionString = await window.api.docker.createDatabase(serviceId, dbName)
    // Mock implementation
    if (serviceId === 'rstn-postgres') {
      return `postgresql://postgres:postgres@localhost:5432/${dbName}`
    } else if (serviceId === 'rstn-mysql') {
      return `mysql://root:mysql@localhost:3306/${dbName}`
    } else {
      return `mongodb://localhost:27017/${dbName}`
    }
  }, [])

  const handleCreateVhost = useCallback(async (serviceId: string, vhostName: string): Promise<string> => {
    console.log('[FE] handleCreateVhost called:', serviceId, vhostName)
    // TODO: Replace with napi-rs call
    // const connectionString = await window.api.docker.createVhost(serviceId, vhostName)
    return `amqp://guest:guest@localhost:5672/${vhostName}`
  }, [])

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">Dockers</h2>
          <p className="mt-1 text-muted-foreground">Container management dashboard</p>
        </div>
        <Button variant="outline" onClick={handleRefreshAll} disabled={isRefreshing}>
          <RefreshCw className={`mr-2 h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Service Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {services.map((service) => (
          <DockerServiceCard
            key={service.id}
            service={service}
            onToggle={handleToggle}
            onRestart={handleRestart}
            onViewLogs={handleViewLogs}
            onCreateDb={handleCreateDb}
            onCreateVhost={handleCreateVhost}
          />
        ))}
      </div>

      {/* Empty State */}
      {services.length === 0 && (
        <div className="flex h-64 flex-col items-center justify-center rounded-lg border border-dashed">
          <p className="text-muted-foreground">No Docker services configured</p>
          <Button variant="outline" className="mt-4">
            Add Service
          </Button>
        </div>
      )}

      {/* Log Panel */}
      <DockerLogSheet
        open={logPanelOpen}
        serviceName={selectedService?.name ?? ''}
        logs={logs}
        onClose={handleCloseLogPanel}
        onRefresh={() => refreshLogs()}
      />
    </div>
  )
}
