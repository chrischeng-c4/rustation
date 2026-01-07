# Docker Management

Container management dashboard for development services.

## Purpose

Provide a visual interface for managing Docker containers used in local development. Users can start, stop, restart services, view logs, and create databases/vhosts without using command-line tools.

## Requirements

### Requirement: Service Detection
The system SHALL automatically detect Docker containers with `rstn-` name prefix and display them in the dashboard.

#### Scenario: Container running
- **WHEN** a container named `rstn-postgres` is running
- **THEN** display service card with green status indicator and port number

#### Scenario: Container stopped
- **WHEN** a container is stopped
- **THEN** display service card with gray status indicator

#### Scenario: Docker unavailable
- **WHEN** Docker daemon is not running
- **THEN** display "Docker Not Available" message

### Requirement: Service Control
The system SHALL provide start, stop, and restart controls for each detected service.

#### Scenario: Start service
- **WHEN** user clicks Start button on stopped service
- **THEN** execute `docker start <container>` and update status to Running

#### Scenario: Stop service
- **WHEN** user clicks Stop button on running service
- **THEN** execute `docker stop <container>` and update status to Stopped

#### Scenario: Restart service
- **WHEN** user clicks Restart button
- **THEN** execute `docker restart <container>` and maintain Running status

### Requirement: Log Viewing
The system SHALL provide real-time log streaming for selected containers.

#### Scenario: Select service
- **WHEN** user clicks on a service card
- **THEN** display last 100 lines of container logs in right panel

#### Scenario: Refresh logs
- **WHEN** user clicks Refresh button
- **THEN** fetch latest container logs

#### Scenario: Copy logs
- **WHEN** user clicks Copy button
- **THEN** copy log content to system clipboard

### Requirement: Database Creation
The system SHALL support creating databases in PostgreSQL, MySQL, and MongoDB containers.

#### Scenario: Create PostgreSQL database
- **WHEN** user enters database name in Add DB dialog for PostgreSQL service
- **THEN** execute `docker exec <container> createdb <name>` and return connection string `postgresql://postgres:postgres@localhost:5432/<name>`

#### Scenario: Create MySQL database
- **WHEN** user enters database name for MySQL service
- **THEN** execute SQL `CREATE DATABASE` and return connection string `mysql://root:mysql@localhost:3306/<name>`

#### Scenario: Create MongoDB database
- **WHEN** user enters database name for MongoDB service
- **THEN** return connection string `mongodb://localhost:27017/<name>`

### Requirement: RabbitMQ Vhost Creation
The system SHALL support creating virtual hosts in RabbitMQ containers.

#### Scenario: Create vhost
- **WHEN** user enters vhost name in Add Vhost dialog
- **THEN** execute `rabbitmqadmin declare vhost name=<vhost>` and return AMQP URL

### Requirement: Supported Services
The system SHALL recognize and provide service-specific features for the following container types:

| Service | Image Pattern | Port | Features |
|---------|---------------|------|----------|
| PostgreSQL | `postgres:*` | 5432 | Create DB |
| MySQL | `mysql:*` | 3306 | Create DB |
| MongoDB | `mongo:*` | 27017 | Create DB |
| Redis | `redis:*` | 6379 | - |
| RabbitMQ | `rabbitmq:*-management` | 5672 | Create Vhost |
| NATS | `nats:*` | 4222 | - |

#### Scenario: Detect PostgreSQL
- **WHEN** container image matches `postgres:*` pattern
- **THEN** classify as PostgreSQL and enable "Add DB" button

#### Scenario: Detect RabbitMQ
- **WHEN** container image matches `rabbitmq:*-management` pattern
- **THEN** classify as RabbitMQ and enable "Add Vhost" button

### Requirement: Per-Worktree State Isolation
The system SHALL maintain separate Docker state for each worktree.

#### Scenario: Switch worktree
- **WHEN** user switches to different worktree
- **THEN** preserve selected service and log view for original worktree

#### Scenario: Service selection
- **WHEN** user selects service in worktree A
- **THEN** selection does not affect worktree B

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
  status: ServiceStatus  // Running | Stopped | Starting | Stopping | Error
  port: number | null
  service_type: ServiceType  // PostgreSQL | MySQL | MongoDB | Redis | RabbitMQ | NATS
}
```

## Implementation References

- Backend: `packages/core/src/docker.rs` (Bollard API)
- UI: `desktop/src/renderer/src/features/dockers/`
- State: `packages/core/src/reducer/docker.rs`
