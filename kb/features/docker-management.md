---
title: "Docker Management"
description: "Container management dashboard"
category: feature
status: active
last_updated: 2025-12-26
version: 3.0.0
---

# Docker Management

## Overview

The Docker tab provides a dashboard for managing development containers.

---

## Features

### Service Cards

```
┌──────────────────────────────────────────────────────┐
│ PostgreSQL                              [▶] [↻] [⏹] │
│ postgres:16                                          │
│ ● Running   Port: 5432                              │
│                                        [+ Add DB]    │
└──────────────────────────────────────────────────────┘
```

- **Status indicator**: Running (green), Stopped (gray), Error (red)
- **Controls**: Start, Restart, Stop
- **Quick actions**: Add DB, Add Vhost (service-specific)

### Log Panel

- **Real-time logs** from selected container
- **Tail support** (last N lines)
- **Copy to clipboard**
- **Refresh** on demand

### Database Creation

PostgreSQL/MySQL/MongoDB containers support:
- **Add DB dialog**: Enter database name
- **Returns**: Connection string for immediate use

### RabbitMQ Vhosts

- **Add Vhost dialog**: Enter vhost name
- **Returns**: Connection URL

---

## Supported Services

| Service | Image | Port | Features |
|---------|-------|------|----------|
| PostgreSQL | postgres:16 | 5432 | Create DB |
| MySQL | mysql:8 | 3306 | Create DB |
| MongoDB | mongo:7 | 27017 | Create DB |
| Redis | redis:7 | 6379 | - |
| RabbitMQ | rabbitmq:3-management | 5672 | Create Vhost |
| NATS | nats:latest | 4222 | - |

---

## Actions

| Action | Payload | Description |
|--------|---------|-------------|
| `CheckDockerAvailability` | - | Check if Docker daemon running |
| `SetDockerAvailable` | `{ available: bool }` | Set availability status |
| `RefreshDockerServices` | - | Refresh service list |
| `StartDockerService` | `{ service_id: string }` | Start container |
| `StopDockerService` | `{ service_id: string }` | Stop container |
| `RestartDockerService` | `{ service_id: string }` | Restart container |
| `SelectDockerService` | `{ service_id: string }` | Select for log view |
| `FetchDockerLogs` | `{ service_id, tail }` | Get container logs |
| `CreateDatabase` | `{ service_id, db_name }` | Create DB in container |
| `CreateVhost` | `{ service_id, vhost_name }` | Create RabbitMQ vhost |

---

## Backend Implementation

### Docker API (bollard crate)

```rust
// packages/core/src/docker.rs

pub fn docker_is_available() -> bool
pub fn docker_list_services() -> Vec<DockerService>
pub fn docker_start_service(id: &str) -> Result<()>
pub fn docker_stop_service(id: &str) -> Result<()>
pub fn docker_get_logs(id: &str, tail: u32) -> Vec<String>
pub fn docker_create_database(id: &str, db_name: &str) -> String
pub fn docker_create_vhost(id: &str, vhost_name: &str) -> String
```

### Service Detection

Services are detected by:
1. Container name prefix: `rstn-`
2. Image name matching known service types

---

## UI Components

### DockersPage.tsx

- **Service list**: Cards for each detected service
- **Log panel**: Selected service logs
- **Loading states**: Skeleton while fetching
- **Error states**: Docker not available message

### DockerServiceCard.tsx

- **Status badge** with color
- **Control buttons** (Start/Stop/Restart)
- **Service-specific actions** (Add DB/Vhost)

### AddDbDialog.tsx / AddVhostDialog.tsx

- **Input validation**: Alphanumeric + underscore
- **Loading state**: During creation
- **Result**: Connection string display with copy

---

## State Structure

```typescript
interface DockersState {
  docker_available: boolean | null
  services: DockerServiceInfo[]
  selected_service_id: string | null
  logs: string[]
  is_loading: boolean
  is_loading_logs: boolean
}

interface DockerServiceInfo {
  id: string
  name: string
  image: string
  status: ServiceStatus
  port: number | null
  service_type: ServiceType
}
```

---

## References

- [Architecture Overview](../architecture/00-overview.md)
- [State Topology](../architecture/02-state-topology.md)
