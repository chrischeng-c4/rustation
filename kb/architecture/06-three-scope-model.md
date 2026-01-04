---
title: "Three-Scope Model"
description: "Feature scoping at Global, Project, and Worktree levels"
category: architecture
status: active
last_updated: 2025-12-26
version: 1.0.0
tags: [architecture, scope, global, project, worktree]
weight: 6
---

# Three-Scope Model

## 1. Overview

rstn features are organized into three scopes based on their data sharing requirements:

| Scope | Level | Data Sharing | UI Location |
|-------|-------|--------------|-------------|
| **Global** | App-wide | Across ALL projects | First bar (right side) |
| **Project** | Per git repo | Across worktrees in same project | Second bar (right side) |
| **Worktree** | Per worktree | Isolated to single worktree | Left sidebar |

### Visual Layout (Scope-in-Bar Pattern)

```
+---------------------------------------------------------------------+
| [project-a] [project-b] [+]                    [Docker]             |  <- Global scope
+---------------------------------------------------------------------+
| [main] [feature/auth] [+]                      [Env]                |  <- Project scope
+----------+----------------------------------------------------------+
| Tasks    |                                                          |
| Settings |              Main Content Area                           |  <- Worktree scope
|          |                                                          |
+----------+----------------------------------------------------------+
```

---

## 2. Scope Definitions

### 2.1 Global Scope

**Characteristics**:
- Data shared across ALL open projects
- Independent of which project/worktree is active
- State stored in `AppState` root level

**Features**:
- **Docker Management**: Container services are global resources
- Future: Global search, app-wide settings

**Why Docker is Global**:
- Docker containers run system-wide, not per-project
- Container names/ports conflict across projects
- Service status should be visible regardless of active project

### 2.2 Project Scope

**Characteristics**:
- Data shared across worktrees within same project
- Changes in one worktree affect sibling worktrees
- State stored in `ProjectState`

**Features**:
- **Env Management**: Copy dotfiles between worktrees
- Future: Git hooks, project-wide settings, shared credentials

**Why Env is Project-level**:
- `.env` files are gitignored but needed in all worktrees
- Worktrees share the same project configuration needs
- Copy-on-create simplifies worktree setup

### 2.3 Worktree Scope

**Characteristics**:
- Data isolated to single worktree
- No sharing between worktrees
- State stored in `WorktreeState`

**Features**:
- **Tasks**: Justfile command execution
- **Settings**: Per-worktree configuration
- Future: MCP server, terminal sessions

---

## 3. State Structure

```
AppState
+-- docker: GlobalDockersState           # Global scope
+-- notifications: Vec<Notification>     # Global scope
+-- projects: Vec<ProjectState>
|   +-- env_config: EnvConfig            # Project scope
|   +-- worktrees: Vec<WorktreeState>
|       +-- tasks: TasksState            # Worktree scope
|       +-- active_view: ActiveView      # Worktree scope
+-- global_settings: GlobalSettings
```

### ActiveView Enum

Tracks which view is currently displayed:

```
ActiveView
+-- Tasks      (Worktree scope)
+-- Settings   (Worktree scope)
+-- Dockers    (Global scope)
+-- Env        (Project scope)
+-- A2UI       (Worktree scope - Experimental)
```

---

## 4. UI Behavior

### Click Actions

| Element | Location | Action |
|---------|----------|--------|
| Docker button | First bar (right) | Switch to DockersPage |
| Env button | Second bar (right) | Switch to EnvPage |
| Tasks tab | Sidebar | Switch to TasksPage |
| Settings tab | Sidebar | Switch to SettingsPage |

### Sidebar Highlighting

- `ActiveView::Tasks` -> Tasks tab highlighted
- `ActiveView::Settings` -> Settings tab highlighted
- `ActiveView::Dockers` -> No sidebar highlight (global view)
- `ActiveView::Env` -> No sidebar highlight (project view)

---

## 5. Extensibility

The scope model allows future features to be added at the appropriate level:

| Future Feature | Scope | Reason |
|----------------|-------|--------|
| Global search | Global | Search across all projects |
| Git hooks | Project | Shared across worktrees |
| Terminal | Worktree | Session-specific |
| MCP server | Worktree | Per-worktree Claude integration |

---

## 6. Implementation Notes

### Adding a Global Feature

1. Add state to `AppState` root level
2. Add button to first bar (right side)
3. Create page component
4. Add to `ActiveView` enum

### Adding a Project Feature

1. Add state to `ProjectState`
2. Add button to second bar (right side)
3. Create page component
4. Add to `ActiveView` enum

### Adding a Worktree Feature

1. Add state to `WorktreeState`
2. Add tab to left sidebar
3. Create page component
4. Add to `ActiveView` enum
