# Architecture Design

## Context
The application manages multiple Git repositories ("Projects"), each with potentially multiple checkout directories ("Worktrees"). Users need to switch contexts frequently but also share certain resources.

## Goals
- **Clarity**: User must always know which Project and Worktree they are in.
- **Efficiency**: "Global" tools like Docker shouldn't require switching projects to view.
- **Consistency**: "Project" settings (like `.env` syncing) should apply to all worktrees in that project.

## Hierarchy Definition

| Level | Entity | Scope | Components |
|-------|--------|-------|------------|
| 0 | **Global** | App-wide | Copy (📋), Screenshot (📸), Download (📥), Notifications (🔔), Logs (📊), Docker (🐳), Settings (⚙️) |
| 1 | **Project** | Git Repo | Environment Management (Sync) |
| 2 | **Worktree** | Directory | Tasks, Terminal, File Explorer, Chat, MCP |

## Decisions
- **Decision**: **Global Utilities as Icon Buttons**.
  - **Why**: Space-efficient design. Always-accessible utilities are used frequently but don't need tab space.
  - **Why Icons**: Visual recognition is faster than reading text labels. Common utilities have universal icons.
  - **Utilities**: 📋 Copy (screen→clipboard), 📸 Screenshot (save file), 📥 Download, 🔔 Notifications, 📊 Logs, 🐳 Docker, ⚙️ Settings
- **Decision**: **Docker is Global**.
  - **Why**: Containers (postgres, redis) often serve multiple projects or persist across branch switches. Binding them to a worktree hides them unnecessarily.
- **Decision**: **Env is Project-Scoped**.
  - **Why**: `.env` files are typically committed (templates) or shared (secrets) across branches of the same repo. Managing them per-worktree is tedious; managing them per-project allows "Sync to all worktrees".
- **Decision**: **Tab UI with Left-Right Split**.
  - **Why**: Tabs are a standard metaphor for parallel contexts. Left side = selections (dynamic), Right side = features (fixed).

## Design Philosophy

**rstn is an opinionated development workflow tool.** We provide what we believe is the best development experience, and users follow this proven approach.

- **No Agent Rules in UI**: Agent/AI configuration belongs in Settings, not in the main workflow. rstn defines the optimal workflow; users configure AI preferences separately.
- **Curated Feature Set**: Only features that serve the core workflow (multi-project, multi-worktree development) are included.
- **Opinionated Defaults**: Pre-configured with sensible defaults based on best practices.

## Risks
- **Complexity**: Nested tabs can clutter the UI.
  - *Mitigation*: Use distinct visual styles for Level 1 (Project) vs Level 2 (Worktree) tabs. Use left-right split in tabs (selections on left, features on right).
