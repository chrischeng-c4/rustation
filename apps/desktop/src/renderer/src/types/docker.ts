export type ServiceStatus = "running" | "stopped" | "starting" | "error"

export type ServiceType = "Database" | "MessageBroker" | "Cache" | "Other"

export interface DockerService {
  id: string
  name: string
  image: string
  status: ServiceStatus
  port: number | null
  service_type: ServiceType
  /** Project group (e.g., "tech-platform", "rstn", "pg-bench") */
  project_group: string | null
  /** Whether this container is managed by rstn (rstn-* prefix) */
  is_rstn_managed: boolean
}

export interface DockersState {
  services: DockerService[]
  selectedService: string | null
  logPanelOpen: boolean
}

export const statusColors: Record<ServiceStatus, string> = {
  running: "bg-green-500",
  stopped: "bg-gray-400",
  starting: "bg-yellow-500",
  error: "bg-red-500",
}

export const statusLabels: Record<ServiceStatus, string> = {
  running: "Running",
  stopped: "Stopped",
  starting: "Starting",
  error: "Error",
}
