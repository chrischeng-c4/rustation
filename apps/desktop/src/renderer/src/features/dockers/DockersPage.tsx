import { useEffect, useCallback, useState, useMemo } from 'react'
import { RefreshCw, AlertCircle, ChevronDown, ChevronRight, Lock, Container } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { LogPanel } from '@/components/shared/LogPanel'
import { PageHeader } from '@/components/shared/PageHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { DockerServiceCard } from './DockerServiceCard'
import { PortConflictDialog } from './PortConflictDialog'
import { useDockersState } from '@/hooks/useAppState'
import type { DockerServiceInfo } from '@/types/state'

interface ServiceGroup {
  name: string
  services: DockerServiceInfo[]
  isRstnManaged: boolean
  runningCount: number
}

export function DockersPage() {
  const { dockers, dispatch, isLoading: isStateLoading } = useDockersState()
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set())

  // Derive values from state
  const services = dockers?.services ?? []
  const selectedServiceId = dockers?.selected_service_id ?? null
  const logs = dockers?.logs ?? []
  const isRefreshing = dockers?.is_loading ?? false
  const isRefreshingLogs = dockers?.is_loading_logs ?? false
  const dockerAvailable = dockers?.docker_available ?? null
  const pendingConflict = dockers?.pending_conflict ?? null

  const selectedService = services.find((s) => s.id === selectedServiceId)

  // Group services by project_group
  const serviceGroups = useMemo((): ServiceGroup[] => {
    const groupMap = new Map<string, DockerServiceInfo[]>()

    for (const service of services) {
      const groupName = service.project_group ?? 'other'
      const existing = groupMap.get(groupName) ?? []
      existing.push(service)
      groupMap.set(groupName, existing)
    }

    // Convert to array and sort: rstn first, then alphabetically
    const groups: ServiceGroup[] = []
    for (const [name, groupServices] of groupMap) {
      const isRstnManaged = groupServices.some(s => s.is_rstn_managed)
      const runningCount = groupServices.filter(s => s.status === 'running').length
      groups.push({ name, services: groupServices, isRstnManaged, runningCount })
    }

    return groups.sort((a, b) => {
      // rstn always first
      if (a.name === 'rstn') return -1
      if (b.name === 'rstn') return 1
      // Then alphabetically
      return a.name.localeCompare(b.name)
    })
  }, [services])

  const toggleGroup = useCallback((groupName: string) => {
    setCollapsedGroups(prev => {
      const next = new Set(prev)
      if (next.has(groupName)) {
        next.delete(groupName)
      } else {
        next.add(groupName)
      }
      return next
    })
  }, [])

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

  // Port conflict resolution handlers
  const handleResolveWithPort = useCallback(async (serviceId: string, port: number) => {
    await dispatch({
      type: 'StartDockerServiceWithPort',
      payload: { service_id: serviceId, port }
    })
  }, [dispatch])

  const handleResolveByStoppingContainer = useCallback(async (containerId: string, serviceId: string) => {
    await dispatch({
      type: 'ResolveConflictByStoppingContainer',
      payload: { conflicting_container_id: containerId, service_id: serviceId }
    })
  }, [dispatch])

  const handleCancelConflict = useCallback(async () => {
    await dispatch({ type: 'ClearPortConflict' })
  }, [dispatch])

  // Initial loading state
  if (isStateLoading || dockerAvailable === null) {
    return <LoadingState message="Checking Docker status..." />
  }

  // Docker not available state
  if (dockerAvailable === false) {
    return (
      <EmptyState
        icon={AlertCircle}
        title="Docker Not Available"
        description="Please ensure Docker Desktop or Docker Engine is installed and running on your system."
        action={{
          label: "Retry Connection",
          onClick: handleRetry,
          icon: RefreshCw
        }}
      />
    )
  }

  return (
    <div className="flex h-full flex-col">
      {/* Port Conflict Dialog */}
      <PortConflictDialog
        pendingConflict={pendingConflict}
        onResolveWithPort={handleResolveWithPort}
        onResolveByStoppingContainer={handleResolveByStoppingContainer}
        onCancel={handleCancelConflict}
      />

      {/* Header */}
      <PageHeader
        title="Dockers"
        description="Container management dashboard for shared services"
      >
        <Button variant="outline" onClick={handleRefreshAll} disabled={isRefreshing}>
          <RefreshCw className={`mr-2 h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </PageHeader>

      {/* Two-column layout */}
      <div className="flex flex-1 gap-4 overflow-hidden">
        {/* Left: Service List */}
        <div className="w-1/2 overflow-hidden rounded-lg border">
          <div className="border-b bg-muted/40 px-4 py-2">
            <span className="text-sm font-medium">Services</span>
          </div>
          <ScrollArea className="h-[calc(100%-40px)]">
            <div className="space-y-2 p-4">
              {serviceGroups.map((group) => {
                const isCollapsed = collapsedGroups.has(group.name)
                return (
                  <div key={group.name} className="rounded-lg border">
                    {/* Group Header */}
                    <button
                      className="flex w-full items-center justify-between px-3 py-2 hover:bg-muted/40 transition-colors"
                      onClick={() => toggleGroup(group.name)}
                    >
                      <div className="flex items-center gap-2">
                        {isCollapsed ? (
                          <ChevronRight className="h-4 w-4 text-muted-foreground" />
                        ) : (
                          <ChevronDown className="h-4 w-4 text-muted-foreground" />
                        )}
                        <span className="font-medium">{group.name}</span>
                        <Badge variant="secondary" className="text-xs">
                          {group.runningCount}/{group.services.length}
                        </Badge>
                      </div>
                      {!group.isRstnManaged && (
                        <div className="flex items-center gap-1 text-xs text-muted-foreground">
                          <Lock className="h-3 w-3" />
                          read-only
                        </div>
                      )}
                    </button>

                    {/* Group Services */}
                    {!isCollapsed && (
                      <div className="space-y-2 border-t px-3 py-2">
                        {group.services.map((service) => (
                          <DockerServiceCard
                            key={service.id}
                            service={service}
                            isActive={selectedServiceId === service.id}
                            onSelect={handleViewLogs}
                            onToggle={handleToggle}
                            onRestart={handleRestart}
                            onViewLogs={handleViewLogs}
                            onCreateDb={handleCreateDb}
                            onCreateVhost={handleCreateVhost}
                          />
                        ))}
                      </div>
                    )}
                  </div>
                )
              })}
              {serviceGroups.length === 0 && !isRefreshing && (
                <EmptyState
                  icon={Container}
                  title="No Services Found"
                  description="No Docker services were detected on your system."
                  action={{
                    label: "Refresh",
                    onClick: handleRefreshAll,
                    icon: RefreshCw
                  }}
                />
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
            emptyMessage="Click a service to view its logs"
          />
        </div>
      </div>
    </div>
  )
}
