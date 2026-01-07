# Project Management

Multi-project tabs with Git worktree support for branch-based workflows.

## Purpose

Allow developers to work on multiple projects and branches simultaneously using git worktrees. Each worktree maintains isolated state for tasks, Docker services, and terminal sessions, enabling seamless context switching without losing work.

## Requirements

### Requirement: Multiple Projects
The system SHALL support opening multiple git repositories simultaneously in tabs.

#### Scenario: Open project
- **WHEN** user selects "Open Project" and chooses a directory
- **THEN** detect git root, create project tab, and load worktrees

#### Scenario: Switch project
- **WHEN** user clicks on a project tab
- **THEN** switch active project and display its worktrees

#### Scenario: Close project
- **WHEN** user clicks close button on project tab
- **THEN** remove project from active list and add to recent projects

### Requirement: Recent Projects
The system SHALL maintain a list of recently opened projects for quick access.

#### Scenario: Show recent projects
- **WHEN** user clicks (+) button in project tabs
- **THEN** display dropdown menu with recent projects

#### Scenario: Open recent project
- **WHEN** user selects a project from recent list
- **THEN** open project as new tab

### Requirement: Git Worktree Detection
The system SHALL automatically detect and list all git worktrees for opened projects.

#### Scenario: Detect main worktree
- **WHEN** project is opened
- **THEN** execute `git rev-parse --show-toplevel` to find git root and create main worktree state

#### Scenario: Detect feature worktrees
- **WHEN** project is opened
- **THEN** parse `git worktree list --porcelain` and create worktree state for each

### Requirement: Worktree Tabs
The system SHALL display worktree tabs below project tabs for the active project.

#### Scenario: Display worktrees
- **WHEN** project is active
- **THEN** show tab for each worktree with branch name

#### Scenario: Switch worktree
- **WHEN** user clicks on worktree tab
- **THEN** switch active worktree and preserve isolated state (Tasks, Docker)

### Requirement: Add Worktree
The system SHALL support creating new worktrees from existing or new branches.

#### Scenario: Add worktree from existing branch
- **WHEN** user selects branch from "Add Worktree" dialog
- **THEN** execute `git worktree add <path> <branch>` in sibling directory

#### Scenario: Add worktree with new branch
- **WHEN** user enters new branch name in dialog
- **THEN** execute `git worktree add -b <new-branch> <path>` and create worktree

#### Scenario: Worktree path convention
- **WHEN** creating worktree for branch "feature/auth" from project at `/projects/rustation`
- **THEN** create worktree at `/projects/rustation-feature-auth`

### Requirement: Remove Worktree
The system SHALL support deleting worktrees.

#### Scenario: Remove worktree
- **WHEN** user clicks remove button on worktree tab
- **THEN** execute `git worktree remove <path>` and update worktree list

#### Scenario: Cannot remove main worktree
- **WHEN** user attempts to remove main worktree
- **THEN** prevent removal and show error message

### Requirement: Branch Listing
The system SHALL list available branches for worktree creation.

#### Scenario: List branches
- **WHEN** user opens "Add Worktree" dialog
- **THEN** execute `git branch --list` and display branches without existing worktrees

#### Scenario: Filter branches with worktrees
- **WHEN** displaying branch list
- **THEN** mark or hide branches that already have worktrees

### Requirement: Per-Worktree State Isolation
The system SHALL maintain separate state for Tasks and Docker per worktree.

#### Scenario: Worktree has isolated tasks
- **WHEN** user runs task in worktree A
- **THEN** task output and status are isolated to worktree A

#### Scenario: Worktree has isolated Docker state
- **WHEN** user selects Docker service in worktree A
- **THEN** selection does not affect worktree B

#### Scenario: Switch worktree preserves state
- **WHEN** user switches from worktree A to B and back to A
- **THEN** restore task output and Docker selection from worktree A

### Requirement: Git Status Detection
The system SHALL detect if worktree has uncommitted changes.

#### Scenario: Worktree modified
- **WHEN** worktree has uncommitted changes
- **THEN** display indicator (asterisk or dot) on worktree tab

#### Scenario: Worktree clean
- **WHEN** worktree has no uncommitted changes
- **THEN** display normal worktree tab

## State Structure

```typescript
interface ProjectState {
  id: string
  path: string
  name: string
  worktrees: WorktreeState[]
  active_worktree_index: number
  available_branches: BranchInfo[]
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
  mcp: McpState
  terminal: TerminalState
  explorer: FileExplorerState
  // ... other per-worktree state
}

interface BranchInfo {
  name: string
  has_worktree: boolean
  is_current: boolean
}
```

## Implementation References

- Backend: `packages/core/src/worktree.rs` (git CLI integration)
- UI: `desktop/src/renderer/src/features/projects/`
- State: `packages/core/src/reducer/project.rs`, `packages/core/src/reducer/worktree.rs`
