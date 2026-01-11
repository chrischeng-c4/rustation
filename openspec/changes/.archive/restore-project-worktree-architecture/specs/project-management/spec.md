# Project Management Spec

## MODIFIED Requirements

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

### Requirement: Worktree Tabs
The system SHALL display worktree tabs below project tabs for the active project.

#### Scenario: Display worktrees
- **WHEN** project is active
- **THEN** show tab for each worktree with branch name in a secondary navigation bar

#### Scenario: Switch worktree
- **WHEN** user clicks on worktree tab
- **THEN** switch active worktree and preserve isolated state (Tasks, Docker)
- **NOTE**: Docker is technically global, but "isolated state" here refers to Worktree-specific tools (Tasks, Terminal).

### Requirement: Hierarchy Visualization
The system SHALL visually distinguish between Project scope and Worktree scope.

#### Scenario: Project Level Actions
- **WHEN** user is in a Project context
- **THEN** "Environment" and "Rules" options are visible/active

#### Scenario: Worktree Level Actions
- **WHEN** user is in a Worktree context
- **THEN** "Tasks", "Terminal", "Explorer" options are visible/active

## REMOVED Requirements
### Requirement: Per-Worktree State Isolation (Docker)
**Reason**: Docker management is now Global (Cross-Project).
**Migration**: Move Docker state to `AppState.docker`.

## ADDED Requirements

### Requirement: Global Docker Access
The system SHALL provide access to Docker management regardless of the active project.

#### Scenario: Access Docker
- **WHEN** user clicks the global Docker icon/tab
- **THEN** display all running containers across the system
