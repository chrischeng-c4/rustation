---
title: "State Topology"
description: "AppState tree structure"
category: architecture
status: active
last_updated: 2025-12-26
version: 3.0.0
---

# State Topology

## AppState Tree

```
AppState
├── version: String
├── projects: Vec<ProjectState>
│   └── ProjectState
│       ├── id: String
│       ├── path: String
│       ├── name: String
│       ├── worktrees: Vec<WorktreeState>
│       │   └── WorktreeState
│       │       ├── id: String
│       │       ├── path: String
│       │       ├── branch: String
│       │       ├── is_main: bool
│       │       ├── is_modified: bool
│       │       ├── active_tab: FeatureTab
│       │       ├── mcp: McpState
│       │       ├── tasks: TasksState
│       │       │   ├── commands: Vec<JustCommandInfo>
│       │       │   ├── task_statuses: HashMap<String, TaskStatus>
│       │       │   ├── active_command: Option<String>
│       │       │   ├── output: Vec<String>
│       │       │   └── is_loading: bool
│       │       └── dockers: DockersState
│       │           ├── docker_available: Option<bool>
│       │           ├── services: Vec<DockerServiceInfo>
│       │           ├── selected_service_id: Option<String>
│       │           ├── logs: Vec<String>
│       │           └── is_loading: bool
│       └── active_worktree_index: usize
├── active_project_index: usize
├── global_settings: GlobalSettings
│   ├── theme: Theme
│   └── default_project_path: Option<String>
├── recent_projects: Vec<RecentProject>
└── error: Option<AppError>
```

---

## Key Types

### FeatureTab
```rust
pub enum FeatureTab {
    Tasks,
    Dockers,
    Settings,
}
```

### ServiceStatus (Docker)
```rust
pub enum ServiceStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error,
}
```

### TaskStatus
```rust
pub enum TaskStatus {
    Idle,
    Running,
    Success,
    Error,
}
```

---

## State Access Patterns

### From React (useAppState hooks)

```typescript
// Get active project
const { projects, activeIndex } = useActiveProject()
const project = projects[activeIndex]

// Get active worktree
const { worktrees, activeWorktreeIndex } = useActiveWorktree()
const worktree = worktrees[activeWorktreeIndex]

// Get Docker state for current worktree
const { dockers } = useDockersState()
const services = dockers?.services ?? []

// Get Tasks state for current worktree
const { tasks } = useTasksState()
const commands = tasks?.commands ?? []
```

---

## State Isolation

Each **worktree** maintains isolated state for:
- Tasks (justfile commands, output)
- Docker (selected service, logs)
- Active tab

This allows switching worktrees without losing context.

---

## References

- [State-First Principle](01-state-first.md)
- [Persistence](03-persistence.md)
