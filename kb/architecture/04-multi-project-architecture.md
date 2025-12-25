---
title: "Multi-Project Architecture"
description: "Support for multiple projects with tabs, workspace management, and per-project state"
category: architecture
status: active
last_updated: 2025-12-25
version: 1.0.0
tags: [architecture, workspace, multi-project, tabs]
weight: 4
---

# Multi-Project Architecture

This document defines how rstn supports multiple projects simultaneously.

## 1. Overview

rstn supports opening multiple projects in tabs, similar to browser tabs or VS Code workspaces.

### UI Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  [*proj-1] [proj-2] [proj-3] [+]            ← Project Tabs     │
├──────────┬──────────────────────────────────────────────────────┤
│ [Task]   │  cmd1  │  填空 arg                [exe]             │
│ [Docker] │  cmd2  │─────────────────────────────────────────── │
│[Settings]│  ...   │  log output                                │
│          │        │                                            │
└──────────┴────────┴────────────────────────────────────────────┘
   Sidebar   Commands        Right Panel (Args + Log)
```

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Project** | A folder with project files (justfile, docker-compose, etc.) |
| **Project Tab** | Top-level tab representing an open project |
| **Feature Tab** | Sidebar tab within a project (Task, Docker, Settings) |
| **Active Project** | Currently focused project (receives keyboard input) |

---

## 2. State Structure

### AppState (Root)

```
AppState
├── projects: Vec<ProjectState>     # All open projects
├── active_project_index: usize     # Which project is focused
├── global_settings: GlobalSettings # App-wide settings
└── recent_projects: Vec<RecentProject>  # For "Open Recent"
```

### ProjectState (Per-Project)

```
ProjectState
├── id: String                      # Unique identifier
├── path: PathBuf                   # "/Users/chris/my-project"
├── name: String                    # "my-project" (folder name)
├── is_modified: bool               # Show "*" indicator
├── active_tab: FeatureTab          # Task | Docker | Settings
├── tasks: TasksState               # Justfile commands & output
└── dockers: DockersState           # Docker services for this project
```

### Hierarchy Diagram

```mermaid
classDiagram
    class AppState {
        +Vec~ProjectState~ projects
        +usize active_project_index
        +GlobalSettings global_settings
        +Vec~RecentProject~ recent_projects
    }

    class ProjectState {
        +String id
        +PathBuf path
        +String name
        +bool is_modified
        +FeatureTab active_tab
        +TasksState tasks
        +DockersState dockers
    }

    class TasksState {
        +Vec~JustCommand~ commands
        +HashMap~String, TaskStatus~ task_statuses
        +Option~String~ active_command
        +Vec~String~ output
        +bool is_loading
    }

    class DockersState {
        +Option~bool~ docker_available
        +Vec~DockerServiceInfo~ services
        +Option~String~ selected_service_id
        +Vec~String~ logs
    }

    AppState "1" *-- "0..*" ProjectState
    ProjectState *-- TasksState
    ProjectState *-- DockersState
```

---

## 3. Actions

### Project Management Actions

| Action | Payload | Description |
|--------|---------|-------------|
| `OpenProject` | `{ path: String }` | Open a folder as project |
| `CloseProject` | `{ index: usize }` | Close a project tab |
| `SwitchProject` | `{ index: usize }` | Focus a different project |
| `ScanProject` | `{ index: usize }` | Re-scan justfile, docker-compose |

### Per-Project Actions

All existing actions (e.g., `RefreshDockerServices`, `RunJustCommand`) now operate on the **active project**.

```
dispatch({ type: 'RunJustCommand', payload: { name: 'test', cwd: '.' } })
// Runs in: projects[active_project_index].path
```

---

## 4. UI Behavior

### Project Tabs

- **Click tab**: Switch to that project
- **Click [+]**: Open folder dialog to add project
- **Middle-click / Click X**: Close project tab
- **Asterisk (*)**: Indicates unsaved changes or running tasks

### Opening Projects

1. **Menu**: File > Open Folder
2. **Drag & Drop**: Drag folder onto window
3. **Recent**: File > Open Recent

### Project Detection

When opening a folder, rstn scans for:

| File | Creates |
|------|---------|
| `justfile` or `Justfile` | TasksState with parsed commands |
| `docker-compose.yml` | (Future) Project-specific Docker config |

---

## 5. Data Flow

### Opening a Project

```mermaid
sequenceDiagram
    participant User
    participant React
    participant IPC
    participant Rust

    User->>React: Click [+] tab
    React->>React: Open folder dialog
    User->>React: Select folder
    React->>IPC: dispatch(OpenProject { path })
    IPC->>Rust: state_dispatch(action_json)
    Rust->>Rust: Create ProjectState
    Rust->>Rust: Scan for justfile
    Rust->>Rust: Add to projects[]
    Rust->>IPC: notify_state_update()
    IPC->>React: state:update event
    React->>React: Re-render with new tab
```

### Running a Task

```mermaid
sequenceDiagram
    participant User
    participant React
    participant Rust

    User->>React: Click "Run" on cmd
    React->>Rust: dispatch(RunJustCommand { name, cwd: "." })
    Note over Rust: cwd resolved to projects[active].path
    Rust->>Rust: Execute: just {name} in project dir
    Rust->>Rust: Update projects[active].tasks.output
    Rust->>React: state:update
    React->>React: Show output in log panel
```

---

## 6. Persistence

### Session State

On app close, save to `~/.rstn/session.json`:

```json
{
  "open_projects": [
    "/Users/chris/project-a",
    "/Users/chris/project-b"
  ],
  "active_project_index": 0,
  "recent_projects": [
    { "path": "/Users/chris/old-project", "last_opened": "2025-12-20" }
  ]
}
```

### On Startup

1. Load `session.json`
2. Re-open previously open projects
3. Restore active project index

---

## 7. Implementation Phases

### Phase 1: Core Multi-Project (Current)

- [ ] Update `AppState` with `projects: Vec<ProjectState>`
- [ ] Add project management actions
- [ ] UI: Project tabs at top
- [ ] Justfile path resolved per-project

### Phase 2: Enhanced Tasks

- [ ] Parse justfile arguments `{{arg}}`
- [ ] Argument form before execution
- [ ] Task history per project

### Phase 3: Project-Specific Docker (Future)

- [ ] Detect `docker-compose.yml` in project
- [ ] Show project's containers vs global containers

### Phase 4: Session Persistence (Future)

- [ ] Save/restore open projects
- [ ] Recent projects menu
