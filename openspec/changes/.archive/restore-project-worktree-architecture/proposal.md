# Change: Restore Project/Worktree Architecture

## Why
The application requires a robust hierarchical structure to support multi-project workflows effectively. The current UI/UX needs to strictly reflect the "Global -> Project -> Worktree" hierarchy to avoid user confusion and ensure state isolation where appropriate (Tasks, Terminal) while allowing shared context where needed (Docker, Env).

## What Changes
- **UI/UX**: Implement a strict Two-Tier Tab System.
  - **Level 1 (Top)**: Project Tabs (representing distinct Git Repositories).
  - **Level 2 (Sub)**: Worktree Tabs (representing Git Worktrees/Branches within a project).
- **Scope Redefinition**:
  - **Docker**: Explicitly Global (Cross-Project). accessible from anywhere, but visually distinct (e.g., a dedicated "Docker" mode or always available panel).
  - **Environment (Env)**: Project-Scoped (Cross-Worktree). Managed at the project level, shared/syncable across worktrees.
- **State Management**: Ensure `ActiveView` and navigation logic respect this hierarchy.

## Impact
- **Affected Specs**:
  - `project-management` (Modified)
  - `environment-management` (Added)
- **Affected Code**:
  - Frontend: `ProjectTabs`, `Sidebar`, `AppLayout`.
  - Backend: `EnvConfig` logic, `AppState` navigation handlers.
