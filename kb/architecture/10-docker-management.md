---
title: "Docker Container Management"
description: "Architecture for the Dockers tab - SaaS service container management"
category: architecture
status: draft
last_updated: 2025-12-24
version: 1.0.0
tags: [docker, containers, saas, tui]
weight: 10
---

# Docker Container Management

## 1. Overview

The Dockers tab provides a centralized interface for managing Docker containers used in local development. It supports:

- **Built-in SaaS services**: PostgreSQL, Redis, MongoDB, RabbitMQ, MySQL
- **External container monitoring**: View logs of any running Docker container
- **Service lifecycle management**: Start, stop, configure services
- **Data persistence**: Volume mounts at `~/.rstn/docker-data/{service}/`

### Tab Structure

```
[1] Workflows │ [2] Dockers │ [3] Settings
```

The TUI has three primary tabs:
1. **Workflows**: Main development interface
2. **Dockers**: Container management (this tab)
3. **Settings**: Application configuration

## 2. Architecture

### 2.1 Layout

```
┌────────────────────────────────────────────────────────────┐
│  [1] Workflows │ [2] Dockers (active) │ [3] Settings       │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌─────────────────┐  ┌────────────────────────────────┐  │
│  │ Services        │  │ Info / Logs                    │  │
│  │                 │  │                                │  │
│  │ 🟢 PostgreSQL   │  │  Container: rstn-postgres      │  │
│  │ 🟢 Redis        │  │  Status: Running               │  │
│  │ 🔴 MongoDB      │  │  Port: 5432                    │  │
│  │ ⚪ RabbitMQ     │  │  Image: postgres:16            │  │
│  │ ⚪ MySQL        │  │                                │  │
│  │ ─────────────── │  │  ─────────────────────────     │  │
│  │ Other Running:  │  │  [Logs]                        │  │
│  │ 🟢 my-app       │  │  2024-12-24 10:00:01 Ready     │  │
│  │ 🟢 nginx        │  │  2024-12-24 10:00:00 Started   │  │
│  │                 │  │                                │  │
│  └─────────────────┘  └────────────────────────────────┘  │
│                                                            │
├────────────────────────────────────────────────────────────┤
│ Status: PostgreSQL running on port 5432                    │
├────────────────────────────────────────────────────────────┤
│ j/k select │ Enter actions │ l logs │ c copy │ r refresh  │
└────────────────────────────────────────────────────────────┘
```

### 2.2 Component Hierarchy

```
DockerView
├── ServiceListPanel (left, 30%)
│   ├── Built-in Services Section
│   │   └── ServiceItem[] (with status indicator)
│   └── External Containers Section
│       └── ContainerItem[] (with status indicator)
│
├── InfoPanel (right, 70%)
│   ├── ContainerInfo (name, status, port, image)
│   ├── LogViewer (streaming logs)
│   └── ActionMenu (when open)
│
├── StatusBar
│   └── Current service status message
│
└── Footer
    └── Keybinding hints
```

## 3. State Model

### 3.1 Type Definitions

```
DockerServiceType (enum):
  - POSTGRES
  - REDIS
  - MONGODB
  - RABBITMQ
  - MYSQL

ServiceStatus (enum):
  - STOPPED   (⚪) - Container not running
  - STARTING  (🟡) - Container is starting up
  - RUNNING   (🟢) - Container healthy and running
  - ERROR     (🔴) - Container failed or unhealthy

PanelMode (enum):
  - INFO      - Show container info
  - LOGS      - Show streaming logs
  - ACTIONS   - Show action menu

DockerAction (enum):
  - RUN
  - STOP
  - ADD_DATABASE
  - ADD_USER
  - COPY_CONNECTION
```

### 3.2 State Structure

```
DockerService:
  service_type: DockerServiceType
  container_name: str           # e.g., "rstn-postgres"
  status: ServiceStatus
  port: int | None              # Exposed host port
  version: str                  # e.g., "16" for postgres
  connection_string: str | None # For copy functionality
  data_path: str                # Volume mount path

ExternalContainer:
  name: str                     # Container name
  image: str                    # Image name
  status: str                   # Docker status string
  ports: list[str]              # Port mappings

DockerViewState:
  # Built-in services (always shown, 5 items)
  services: list[DockerService]

  # Other running containers (log-only)
  external_containers: list[ExternalContainer]

  # Selection state
  selected_index: int
  in_external_section: bool     # True = selecting external containers

  # Right panel state
  panel_mode: PanelMode
  log_content: list[str]        # Container logs (max 1000 lines)
  log_follow: bool              # Auto-scroll to new logs

  # Action menu state
  action_menu_open: bool
  action_menu_items: list[DockerAction]
  action_menu_selected: int

  # Status
  status_message: str
  docker_available: bool        # False if Docker not installed
```

## 4. State Machines

### 4.1 Service Lifecycle FSM

```mermaid
stateDiagram-v2
    [*] --> Stopped

    Stopped --> Starting: DockerRunRequested
    note right of Starting
        1. Pull image if needed
        2. Create volume directory
        3. Start container
        4. Wait for health check
    end note

    Starting --> Running: DockerContainerStarted
    Starting --> Error: DockerStartFailed

    Running --> Stopping: DockerStopRequested
    Stopping --> Stopped: DockerContainerStopped
    Running --> Error: DockerHealthCheckFailed

    Error --> Starting: DockerRunRequested (retry)
    Error --> Stopped: DockerStopRequested (cleanup)

    note right of Running
        Container is healthy
        Port is accessible
        Logs streaming available
    end note
```

### 4.2 Action Menu FSM

```mermaid
stateDiagram-v2
    [*] --> ServiceList

    ServiceList --> ActionMenu: Enter key
    note right of ActionMenu
        Show available actions:
        - Run/Stop (toggle)
        - Add Database (if applicable)
        - Add User (if applicable)
        - Copy Connection String
    end note

    ActionMenu --> ServiceList: Esc key
    ActionMenu --> ExecutingAction: Enter (select action)

    ExecutingAction --> ServiceList: ActionCompleted
    ExecutingAction --> ServiceList: ActionFailed

    state ActionMenu {
        [*] --> SelectAction
        SelectAction --> SelectAction: j/k (navigate actions)
    }
```

### 4.3 Panel Mode FSM

```mermaid
stateDiagram-v2
    [*] --> Info

    Info --> Logs: l key (toggle)
    Logs --> Info: l key (toggle)

    Info --> ActionMenu: Enter key
    Logs --> ActionMenu: Enter key

    ActionMenu --> Info: Esc key
    ActionMenu --> Info: ActionCompleted
```

## 5. Message Protocol

### 5.1 UI → Effect Messages

| Message | Payload | Purpose |
|---------|---------|---------|
| `DockerRefreshRequested` | - | Refresh all container statuses |
| `DockerRunRequested` | `service_type: DockerServiceType` | Start a built-in service |
| `DockerStopRequested` | `service_type: DockerServiceType` | Stop a built-in service |
| `DockerLogsRequested` | `container_name: str` | Start streaming logs |
| `DockerLogsStopRequested` | `container_name: str` | Stop streaming logs |
| `DockerActionRequested` | `service_type, action, params` | Execute service action |

### 5.2 Effect → Reducer Messages

| Message | Payload | Purpose |
|---------|---------|---------|
| `DockerContainersUpdated` | `services, external` | Update container list |
| `DockerContainerStarted` | `service_type, port, connection_string` | Container is ready |
| `DockerContainerStopped` | `service_type` | Container stopped |
| `DockerStartFailed` | `service_type, error` | Start failed |
| `DockerLogsReceived` | `container_name, lines: list[str]` | New log lines |
| `DockerActionCompleted` | `service_type, action, result` | Action succeeded |
| `DockerActionFailed` | `service_type, action, error` | Action failed |
| `DockerNotAvailable` | `error` | Docker not installed/running |

## 6. Keybindings

### 6.1 Docker View Context

| Key | Action | Description |
|-----|--------|-------------|
| `j` / `↓` | SelectNext | Move selection down |
| `k` / `↑` | SelectPrev | Move selection up |
| `Enter` | OpenActionMenu | Show available actions |
| `Esc` | CloseActionMenu | Close action menu |
| `l` | ToggleLogs | Toggle log view |
| `c` | CopyConnection | Copy connection string to clipboard |
| `r` | Refresh | Refresh container statuses |

### 6.2 Action Menu Context

| Key | Action | Description |
|-----|--------|-------------|
| `j` / `↓` | SelectNextAction | Move to next action |
| `k` / `↑` | SelectPrevAction | Move to previous action |
| `Enter` | ExecuteAction | Execute selected action |
| `Esc` | CloseMenu | Return to service list |

## 7. Built-in Services

### 7.1 Service Configuration

| Service | Image | Port | Container Name |
|---------|-------|------|----------------|
| PostgreSQL | `postgres:16` | 5432 | `rstn-postgres` |
| Redis | `redis:7` | 6379 | `rstn-redis` |
| MongoDB | `mongo:7` | 27017 | `rstn-mongodb` |
| RabbitMQ | `rabbitmq:3-management` | 5672, 15672 | `rstn-rabbitmq` |
| MySQL | `mysql:8` | 3306 | `rstn-mysql` |

### 7.2 Default Credentials

All services use default credentials for local development:
- **Username**: `rstn`
- **Password**: `rstn`
- **Database**: `rstn` (where applicable)

### 7.3 Connection Strings

| Service | Connection String Format |
|---------|-------------------------|
| PostgreSQL | `postgresql://rstn:rstn@localhost:{port}/rstn` |
| Redis | `redis://localhost:{port}` |
| MongoDB | `mongodb://rstn:rstn@localhost:{port}` |
| RabbitMQ | `amqp://rstn:rstn@localhost:{port}` |
| MySQL | `mysql://rstn:rstn@localhost:{port}/rstn` |

### 7.4 Data Persistence

Data is persisted to `~/.rstn/docker-data/{service}/`:

```
~/.rstn/
└── docker-data/
    ├── postgres/      # PostgreSQL data files
    ├── redis/         # Redis RDB/AOF files
    ├── mongodb/       # MongoDB data files
    ├── rabbitmq/      # RabbitMQ data
    └── mysql/         # MySQL data files
```

## 8. Service Actions

### 8.1 Available Actions Per Service

| Service | Run | Stop | Add DB | Add User | Copy Conn |
|---------|-----|------|--------|----------|-----------|
| PostgreSQL | ✓ | ✓ | ✓ | ✓ | ✓ |
| MySQL | ✓ | ✓ | ✓ | ✓ | ✓ |
| MongoDB | ✓ | ✓ | ✓ | ✓ | ✓ |
| Redis | ✓ | ✓ | - | - | ✓ |
| RabbitMQ | ✓ | ✓ | - | ✓ | ✓ |

### 8.2 Action Details

**Run**: Start the container with:
- Volume mount for data persistence
- Port mapping (auto-assign if default in use)
- Environment variables for credentials
- Health check wait

**Stop**: Stop and remove the container (data persisted in volume)

**Add Database**: Create a new database
- Input: database name
- Executes `CREATE DATABASE` or equivalent

**Add User**: Create a new user with database access
- Input: username, password
- Executes `CREATE USER` with appropriate grants

**Copy Connection**: Copy connection string to clipboard

## 9. External Containers

### 9.1 Discovery

All running Docker containers are discovered via `docker ps`. External containers are those not managed by rstn (not prefixed with `rstn-`).

### 9.2 Capabilities

External containers have limited functionality:
- View in service list with status indicator
- View logs (read-only)
- No management actions (run/stop/configure)

## 10. Error Handling

### 10.1 Docker Not Available

If Docker is not installed or not running:
- Show error message in status bar
- Disable all service actions
- Display installation instructions in info panel

### 10.2 Port Conflicts

If default port is in use:
1. Attempt to find next available port
2. Update connection string with new port
3. Show notice in status bar

### 10.3 Container Failures

On container start failure:
- Set status to ERROR
- Display error message in info panel
- Allow retry via Run action

## 11. Implementation Reference

### 11.1 Files to Create

| File | Purpose |
|------|---------|
| `rstn/state/docker.py` | DockerViewState, DockerService types |
| `rstn/domain/docker/service.py` | Docker service management |
| `rstn/domain/docker/commands.py` | Docker CLI wrapper |
| `rstn/reduce/docker.py` | Docker reducers |
| `rstn/tui/render/docker.py` | Docker view rendering |

### 11.2 Files to Modify

| File | Change |
|------|--------|
| `rstn/state/types.py` | Add `ViewType.DOCKER` |
| `rstn/state/app_state.py` | Add `docker_view: DockerViewState` |
| `rstn/msg/__init__.py` | Add Docker message types |
| `rstn/reduce/__init__.py` | Add Docker reducer dispatch |
| `rstn/tui/render/views.py` | Add Docker view rendering |

## 12. Testing Strategy

### 12.1 State Tests

- Round-trip serialization of DockerViewState
- Service status transitions
- Action menu state transitions

### 12.2 Integration Tests

- Docker command execution (with mock)
- Container lifecycle management
- Log streaming

### 12.3 Manual Testing

- Start/stop each service type
- Verify data persistence across restarts
- Test port conflict handling
- Verify external container discovery
