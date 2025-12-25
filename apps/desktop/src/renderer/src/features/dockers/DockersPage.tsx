import { useEffect, useCallback } from 'react'
import { RefreshCw, AlertCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { LogPanel } from '@/components/LogPanel'
import { DockerServiceCard } from './DockerServiceCard'
import { useDockersState } from '@/hooks/useAppState'

export function DockersPage() {
  const { dockers, dispatch, isLoading: isStateLoading } = useDockersState()

  // Derive values from state
  const services = dockers?.services ?? []
  const selectedServiceId = dockers?.selected_service_id ?? null
  const logs = dockers?.logs ?? []
  const isRefreshing = dockers?.is_loading ?? false
  const isRefreshingLogs = dockers?.is_loading_logs ?? false
  const dockerAvailable = dockers?.docker_available ?? null

  const selectedService = services.find((s) => s.id === selectedServiceId)

  // Check Docker availability and load services on mount
  useEffect(() => {
    dispatch({ type: 'CheckDockerAvailability' })
    dispatch({ type: 'RefreshDockerServices' })
  }, [dispatch])

  const handleToggle = useCallback(async (id: string) => {
    const service = services.find((s) => s.id === id)
    if (!service) return

    if (service.status === 'running') {
      await dispatch({ type: 'StopDockerService', payload: { service_id: id } })
    } else {
      await dispatch({ type: 'StartDockerService', payload: { service_id: id } })
    }
  }, [services, dispatch])

  const handleRestart = useCallback(async (id: string) => {
    await dispatch({ type: 'RestartDockerService', payload: { service_id: id } })
  }, [dispatch])

  const handleViewLogs = useCallback(async (id: string) => {
    await dispatch({ type: 'SelectDockerService', payload: { service_id: id } })
    await dispatch({ type: 'FetchDockerLogs', payload: { service_id: id, tail: 100 } })
  }, [dispatch])

  const refreshLogs = useCallback(async () => {
    if (!selectedServiceId) return
    await dispatch({ type: 'FetchDockerLogs', payload: { service_id: selectedServiceId, tail: 100 } })
  }, [selectedServiceId, dispatch])

  const handleRefreshAll = useCallback(async () => {
    await dispatch({ type: 'RefreshDockerServices' })
  }, [dispatch])

  const handleRetry = useCallback(async () => {
    await dispatch({ type: 'CheckDockerAvailability' })
    await dispatch({ type: 'RefreshDockerServices' })
  }, [dispatch])

  // CreateDb and CreateVhost still use legacy API for now (they return connection strings)
  const handleCreateDb = useCallback(async (serviceId: string, dbName: string): Promise<string> => {
    const connectionString = await window.api.docker.createDatabase(serviceId, dbName)
    return connectionString
  }, [])

  const handleCreateVhost = useCallback(async (serviceId: string, vhostName: string): Promise<string> => {
    const connectionString = await window.api.docker.createVhost(serviceId, vhostName)
    return connectionString
  }, [])

  // Initial loading state
  if (isStateLoading || dockerAvailable === null) {
    return (
      <div className="flex h-full items-center justify-center">
        <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  // Docker not available state
  if (dockerAvailable === false) {
    return (
      <div className="flex h-full flex-col items-center justify-center">
        <AlertCircle className="h-12 w-12 text-muted-foreground" />
        <h2 className="mt-4 text-xl font-semibold">Docker Not Available</h2>
        <p className="mt-2 text-muted-foreground">
          Please ensure Docker is installed and running.
        </p>
        <Button variant="outline" className="mt-4" onClick={handleRetry}>
          <RefreshCw className="mr-2 h-4 w-4" />
          Retry
        </Button>
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">Dockers</h2>
          <p className="mt-1 text-muted-foreground">Container management dashboard</p>
        </div>
        <Button variant="outline" onClick={handleRefreshAll} disabled={isRefreshing}>
          <RefreshCw className={`mr-2 h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Two-column layout */}
      <div className="flex flex-1 gap-4 overflow-hidden">
        {/* Left: Service List */}
        <div className="w-1/2 overflow-hidden rounded-lg border">
          <div className="border-b bg-muted/40 px-4 py-2">
            <span className="text-sm font-medium">Services</span>
          </div>
          <ScrollArea className="h-[calc(100%-40px)]">
            <div className="space-y-3 p-4">
              {services.map((service) => (
                <DockerServiceCard
                  key={service.id}
                  service={service}
                  isActive={selectedServiceId === service.id}
                  onToggle={handleToggle}
                  onRestart={handleRestart}
                  onViewLogs={handleViewLogs}
                  onCreateDb={handleCreateDb}
                  onCreateVhost={handleCreateVhost}
                />
              ))}
              {services.length === 0 && !isRefreshing && (
                <div className="flex flex-col items-center justify-center py-8 text-center">
                  <p className="text-muted-foreground">No Docker services found</p>
                  <Button variant="outline" className="mt-4" onClick={handleRefreshAll}>
                    <RefreshCw className="mr-2 h-4 w-4" />
                    Refresh
                  </Button>
                </div>
              )}
            </div>
          </ScrollArea>
        </div>

        {/* Right: Log Panel */}
        <div className="w-1/2 overflow-hidden">
          <LogPanel
            title={selectedService ? `${selectedService.name} Logs` : 'Logs'}
            logs={logs}
            onRefresh={selectedServiceId ? () => refreshLogs() : undefined}
            isRefreshing={isRefreshingLogs}
            showCopy={true}
            emptyMessage="Select a service and click Logs to view output"
          />
        </div>
      </div>
    </div>
  )
}
