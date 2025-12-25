---
title: "MCP Server Architecture (GUI)"
description: "Embedded MCP HTTP server for Claude Code integration in Tauri"
category: architecture
status: implemented
last_updated: 2025-12-24
version: 2.0.0
tags: [mcp, http, tauri, gui]
weight: 11
---

# MCP Server Architecture (GUI)

## 1. Overview
In the GUI version, the MCP (Model Context Protocol) server remains a critical bridge between Claude Code and the application. It is embedded within the **Tauri Backend (Rust/Python)** process.

### Key Changes for GUI:
1.  **Backend Integration**: The server is started and managed by the Tauri Rust backend.
2.  **State Access**: Instead of accessing TUI-specific view state, it accesses the unified `AppState` in Rust.
3.  **Unified Control**: Tools like `rstn_report_status` trigger Tauri Events that the React frontend listens to (e.g., showing an input modal).

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────┐
│                 Tauri Backend (Rust/Python)              │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────────┐              │
│  │ Claude Code  │───▶│  MCP HTTP Server │              │
│  │   (client)   │    │   (FastAPI)      │              │
│  └──────────────┘    └────────┬─────────┘              │
│                               │                         │
│              ┌────────────────┴────────────────┐       │
│              ▼                                 ▼       │
│     ┌─────────────────┐            ┌──────────────┐   │
│     │ Read-only Tools │            │ Action Tools │   │
│     │ (AppState)      │            │ (Commands)   │   │
│     └─────────────────┘            └──────┬───────┘   │
│                                           ▼           │
│                              ┌─────────────────────┐  │
│                              │ Tauri Event Emit    │  │
│                              │   → React UI        │  │
│                              └─────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 3. Tool Categories (GUI)

### Data Plane (Read-Only)
- `rstn_get_app_state`: Returns the `AppState` currently synced between Rust and React.
- `rstn_read_spec`: Unchanged (reads from file system).

### Control Plane (Actions)
- `rstn_report_status`:
    - **TUI**: Opened a dialog in terminal.
    - **GUI**: Emits a `ui:modal:open` event with the prompt. React shows a beautiful Shadcn modal.
- `rstn_complete_task`:
    - **TUI**: Updated terminal text.
    - **GUI**: Updates `AppState`. React re-renders with a progress bar animation.

---

## 4. Message Flow Diagrams

### 4.1 MCP Tool Invocation Sequence

```mermaid
sequenceDiagram
    participant Claude as Claude Code
    participant MCP as MCP Server (HTTP)
    participant Rust as Rust Backend
    participant Event as Tauri Event Bus
    participant React as React Frontend

    Claude->>MCP: POST /tools/call (rstn_get_app_state)
    MCP->>Rust: Read AppState
    Rust-->>MCP: AppState JSON
    MCP-->>Claude: Tool result

    Claude->>MCP: POST /tools/call (rstn_report_status)
    MCP->>Rust: Dispatch action
    Rust->>Event: emit("ui:modal:open", payload)
    Event-->>React: Modal event
    React->>React: Show input modal
    MCP-->>Claude: Tool result (success)
```

### 4.2 MCP Server Lifecycle FSM

```mermaid
stateDiagram-v2
    [*] --> Stopped: App starts

    Stopped --> Starting: StartMcpServer action
    Starting --> Running: Server bound to port
    Starting --> Error: Port conflict / bind error

    Running --> Stopping: StopMcpServer action
    Running --> Error: Server crash

    Stopping --> Stopped: Cleanup complete
    Error --> Stopped: Retry / Reset

    note right of Running: Listening on dynamic port
    note right of Error: Show error in UI
```

### 4.3 Tool Categories Flow

```mermaid
flowchart LR
    subgraph Claude["Claude Code"]
        C1[Tool request]
    end

    subgraph MCP["MCP Server"]
        direction TB
        M1[Receive request]
        M2{Tool type?}
    end

    subgraph DataPlane["Data Plane (Read)"]
        D1[rstn_get_app_state]
        D2[rstn_read_spec]
    end

    subgraph ControlPlane["Control Plane (Write)"]
        A1[rstn_report_status]
        A2[rstn_complete_task]
    end

    subgraph Backend["Rust Backend"]
        B1[Read AppState]
        B2[Dispatch Action]
        B3[Emit Tauri Event]
    end

    C1 --> M1 --> M2
    M2 -->|Read| DataPlane --> B1
    M2 -->|Write| ControlPlane --> B2 --> B3

    style DataPlane fill:#90EE90
    style ControlPlane fill:#FFB6C1
```

### Summary Flow
1.  **Claude Code** calls an MCP tool (e.g., `rstn_report_status`).
2.  **MCP Server** (FastAPI) receives the call.
3.  **MCP Server** dispatches a message to the Rust core or directly emits a Tauri event.
4.  **Tauri Event Bus** broadcasts the change.
5.  **React Frontend** receives the event and updates the view (e.g., "User Input Required").

---

## 5. Implementation Reference (GUI)

- **Backend Logic**: `src-tauri/src/mcp/` (Python/FastAPI remains, managed by Rust `Command`).
- **Communication**: Tauri `Window::emit` for broadcasting tool side-effects to the frontend.
- **Config**: Same dynamic `mcp-session.json` approach.