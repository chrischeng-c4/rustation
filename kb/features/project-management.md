---
title: "Project Management"
description: "Multi-project and worktree support"
category: feature
status: active
last_updated: 2025-12-26
version: 3.0.0
---

# Project Management

## Overview

rustation supports multiple projects open simultaneously, with git worktree integration for branch-based workflows.

---

## Features

### Project Tabs

```
┌────────────────────────────────────────────────────────┐
│  [*rustation] [my-app] [lib-utils] [+]    ← Projects   │
├────────────────────────────────────────────────────────┤
│  [main] [feature/auth] [bugfix/123] [+]   ← Worktrees  │
└────────────────────────────────────────────────────────┘
```

- **Open multiple projects** in tabs
- **Switch** between projects instantly
- **Close** projects (state preserved in recent)
- **Recent projects** dropdown in (+) menu

### Git Worktrees

Each project can have multiple worktrees:
- **Main worktree** (always present)
- **Feature worktrees** (created from branches)
- Each worktree has isolated Tasks/Docker state

---

## Actions

### Project Actions

| Action | Payload | Description |
|--------|---------|-------------|
| `OpenProject` | `{ path: string }` | Open folder as project |
| `CloseProject` | `{ index: number }` | Close project tab |
| `SwitchProject` | `{ index: number }` | Switch to project |

### Worktree Actions

| Action | Payload | Description |
|--------|---------|-------------|
| `SwitchWorktree` | `{ index: number }` | Switch worktree tab |
| `RefreshWorktrees` | - | Re-scan git worktree list |
| `AddWorktree` | `{ branch: string }` | Create from existing branch |
| `AddWorktreeNewBranch` | `{ branch: string }` | Create with new branch |
| `RemoveWorktree` | `{ worktree_path: string }` | Delete worktree |

---

## Git Integration

### Worktree Detection

On `OpenProject`:
1. Detect git root via `git rev-parse --show-toplevel`
2. Parse `git worktree list --porcelain`
3. Create `WorktreeState` for each worktree

### Add Worktree

Worktree path convention (sibling directory):
```
/projects/rustation/           ← Main repo
/projects/rustation-feature/   ← Feature worktree
/projects/rustation-bugfix/    ← Bugfix worktree
```

### Branch Listing

`worktree_list_branches()` returns:
```rust
pub struct BranchInfo {
    pub name: String,       // e.g., "feature/auth"
    pub has_worktree: bool, // Already has a worktree?
    pub is_current: bool,   // Current branch?
}
```

---

## UI Components

### ProjectTabs.tsx

- **Project row**: Tab per open project
- **Worktree row**: Tab per worktree (when project open)
- **Add button**: Dropdown menu for new project/worktree

### AddWorktreeDialog.tsx

- **Branch list**: Available branches without worktrees
- **Create new**: Enter new branch name
- **Actions**: Cancel / Add Worktree

---

## State Structure

```typescript
interface ProjectState {
  id: string
  path: string
  name: string
  worktrees: WorktreeState[]
  active_worktree_index: number
}

interface WorktreeState {
  id: string
  path: string
  branch: string
  is_main: boolean
  is_modified: boolean
  active_tab: FeatureTab
  tasks: TasksState
  dockers: DockersState
}
```

---

## References

- [Architecture Overview](../architecture/00-overview.md)
- [State Topology](../architecture/02-state-topology.md)
