---
title: "Docker Container Management (GUI)"
description: "Architecture for the Dockers tab in Tauri GUI"
category: architecture
status: draft
last_updated: 2025-12-24
version: 2.0.0
tags: [docker, tauri, react, gui]
weight: 10
---

# Docker Container Management (GUI)

## 1. Overview
The Dockers tab in the GUI provides a visual dashboard for managing development infrastructure. It moves from a text-based list to a **Service Grid** with real-time feedback.

---

## 2. Visual Specification (GUI)

### 2.1 Service Grid
- **Card View**: Each service (Postgres, Redis, etc.) is represented by a Card.
- **Header**: Service Name + Icon + Status Badge (Green/Red/Yellow).
- **Body**: Current port, connection string (truncated), volume path.
- **Actions**: Floating or bottom-aligned buttons (Start, Stop, Config, Logs).

### 2.2 Log Console (Slide-over)
- Clicking "Logs" opens a side panel or a bottom sheet.
- Uses a virtualized list for performance with 10k+ log lines.
- Feature: "Tail" mode (auto-scroll) with a toggle.

### 2.3 Connection Helper
- One-click "Copy Connection String".
- Tooltip showing the full URI.

---

## 3. Backend Integration (Rust)

### 3.1 Docker Client
Uses the `bollard` library for native async communication with the Docker socket (Unix) or Named Pipe (Windows).

### 3.2 Command Interface
```rust
#[tauri::command]
async fn toggle_service(service: DockerServiceType, state: State<'_, AppState>) -> Result<ServiceStatus, Error>;

#[tauri::command]
async fn get_container_logs(name: String, tail: usize) -> Result<Vec<String>, Error>;
```

### 3.3 Event Streaming
Status changes are emitted globally.
```rust
// Emitted whenever a container changes state (e.g., via external CLI)
window.emit("docker:status-change", Payload { name: "rstn-postgres", status: "running" });
```

---

## 4. State Model (Sync)

### 4.1 Data Structure
```typescript
interface DockerService {
  id: string;
  name: string;
  image: string;
  status: 'running' | 'stopped' | 'starting' | 'error';
  port: number;
  connectionString: string;
}
```

---

## 5. UI Components (React)

### 5.1 `DockerServiceCard`
- **Visuals**: Uses `shadcn/ui` Card component.
- **Interactions**:
    - Hover: Show advanced actions (Add User, Add DB).
    - Click: Open log view.

### 5.2 `DockerLogViewer`
- **Engine**: Virtualized list.
- **Controls**: Search/Filter logs, Clear, Follow toggle.

---

## 6. Implementation Reference (GUI)

- **Frontend**: `src/features/docker/`
    - `DockerDashboard.tsx` (Main container)
    - `components/ServiceCard.tsx`
    - `components/LogDrawer.tsx`
- **Backend**: `src-tauri/src/docker/`
    - `manager.rs` (Bollard wrapper)
    - `commands.rs` (Tauri Command handlers)