import { useEffect, useCallback, useState, useMemo } from 'react'
import {
  Refresh as RefreshIcon,
  ErrorOutline as AlertCircleIcon,
  ChevronRight,
  ExpandMore as ChevronDown,
  LockOutlined as LockIcon,
  Dns as ContainerIcon
} from '@mui/icons-material'
import {
  Button,
  Box,
  Typography,
  Chip,
  Paper,
  Divider,
  Stack,
  IconButton,
  Collapse
} from '@mui/material'
import { LogPanel } from '@/components/shared/LogPanel'
import { PageHeader } from '@/components/shared/PageHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { DockerServiceCard } from './DockerServiceCard'
import { PortConflictDialog } from './PortConflictDialog'
import { useDockersState } from '@/hooks/useAppState'
import type { DockerServiceInfo } from '@/types/state'
import { statusLabels } from '@/types/state'

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
        title="Docker Not Available"
        description="Please ensure Docker Desktop or Docker Engine is installed and running on your system."
        action={{
          label: "Retry Connection",
          onClick: handleRetry,
          icon: <RefreshIcon />
        }}
      />
    )
  }

  return (
    <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column' }}>
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
        <Button
          variant="outlined"
          onClick={handleRefreshAll}
          disabled={isRefreshing}
          startIcon={<RefreshIcon sx={{ animation: isRefreshing ? 'spin 2s linear infinite' : 'none' }} />}
        >
          Refresh
        </Button>
      </PageHeader>

      {/* Two-column layout */}
      <Stack direction="row" spacing={3} sx={{ flex: 1, overflow: 'hidden', mt: 1 }}>
        {/* Left: Service List */}
        <Paper
          variant="outlined"
          sx={{
            width: '50%',
            overflow: 'hidden',
            display: 'flex',
            flexDirection: 'column',
            bgcolor: 'surfaceContainerLow.main',
            borderRadius: 4
          }}
        >
          <Box sx={{ px: 2, py: 1.5, borderBottom: 1, borderColor: 'outlineVariant' }}>
            <Typography variant="subtitle2" fontWeight={600}>Services</Typography>
          </Box>

          <Box sx={{ flex: 1, overflowY: 'auto', p: 2 }}>
            <Stack spacing={2}>
              {serviceGroups.map((group) => {
                const isCollapsed = collapsedGroups.has(group.name)
                return (
                  <Paper
                    key={group.name}
                    elevation={0}
                    variant="outlined"
                    sx={{ overflow: 'hidden', borderColor: 'outlineVariant' }}
                  >
                    {/* Group Header */}
                    <Box
                      component="button"
                      onClick={() => toggleGroup(group.name)}
                      sx={{
                        display: 'flex',
                        width: '100%',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        px: 2,
                        py: 1,
                        border: 'none',
                        bgcolor: 'transparent',
                        cursor: 'pointer',
                        textAlign: 'left',
                        '&:hover': { bgcolor: 'action.hover' }
                      }}
                    >
                      <Stack direction="row" spacing={1} alignItems="center">
                        {isCollapsed ? <ChevronRight fontSize="small" /> : <ChevronDown fontSize="small" />}
                        <Typography variant="subtitle2" fontWeight={600}>{group.name}</Typography>
                        <Chip
                          label={`${group.runningCount}/${group.services.length}`}
                          size="small"
                          sx={{ height: 20, fontSize: '0.65rem' }}
                        />
                      </Stack>
                      {!group.isRstnManaged && (
                        <Stack direction="row" spacing={0.5} alignItems="center" sx={{ color: 'text.secondary' }}>
                          <LockIcon sx={{ fontSize: 14 }} />
                          <Typography variant="caption">read-only</Typography>
                        </Stack>
                      )}
                    </Box>

                    {/* Group Services */}
                    <Collapse in={!isCollapsed}>
                      <Divider />
                      <Stack spacing={1.5} sx={{ p: 2 }}>
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
                      </Stack>
                    </Collapse>
                  </Paper>
                )
              })}

              {serviceGroups.length === 0 && !isRefreshing && (
                <EmptyState
                  title="No Services Found"
                  description="No Docker services were detected on your system."
                  action={{
                    label: "Refresh",
                    onClick: handleRefreshAll,
                    icon: <RefreshIcon />
                  }}
                />
              )}
            </Stack>
          </Box>
        </Paper>

        {/* Right: Log Panel */}
        <Box sx={{ width: '50%', overflow: 'hidden' }}>
          <LogPanel
            title={selectedService ? `${selectedService.name} Logs` : 'Logs'}
            logs={logs}
            onRefresh={selectedServiceId ? () => refreshLogs() : undefined}
            isRefreshing={isRefreshingLogs}
            showCopy={true}
            emptyMessage="Click a service to view its logs"
          />
        </Box>
      </Stack>
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </Box>
  )
}
