# Environment Management Spec

## ADDED Requirements

### Requirement: Cross-Worktree Environment Sync
The system SHALL manage environment variables at the Project level and support syncing them to all worktrees.

#### Scenario: View Env Config
- **WHEN** user selects "Environment" for a Project
- **THEN** display the tracked patterns (e.g., `.env`, `.envrc`)

#### Scenario: Sync Env Files
- **WHEN** user triggers "Sync Environment"
- **THEN** copy tracked environment files from the Source Worktree (Main) to all other worktrees

#### Scenario: Auto-Sync
- **WHEN** a new worktree is created
- **AND** `auto_copy_enabled` is true
- **THEN** automatically copy environment files to the new worktree

### Requirement: Source Selection
The system SHALL allow selecting which worktree acts as the "Source of Truth" for environment files.

#### Scenario: Set Source
- **WHEN** user selects a worktree as Source
- **THEN** use that worktree's `.env` files for future sync operations
