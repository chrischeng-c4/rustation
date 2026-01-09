## ADDED Requirements
### Requirement: Session Restoration
The system SHALL restore the user's last active project and view upon application startup.

#### Scenario: Restore last project
- **WHEN** user launches the application
- **AND** a project was active in the previous session
- **THEN** automatically load that project and its last active worktree

### Requirement: Balanced Layout
The system SHALL prioritize file content visibility over file list navigation in the visual hierarchy.

#### Scenario: File list width
- **WHEN** viewing the file explorer
- **THEN** the file list (tree) SHALL occupy a minor portion of the width (sidebar) or be collapsible

#### Scenario: Preview area width
- **WHEN** a file is selected
- **THEN** the preview/detail panel SHALL occupy the majority of the available width (main content)

## MODIFIED Requirements
### Requirement: File Comments
The system SHALL support adding and viewing comments on files, stored in local SQLite.

#### Scenario: Add comment
- **WHEN** user enters comment text and clicks Submit
- **THEN** dispatch `AddFileComment` action
- **AND** persist comment to `.rstn/rstn.db` linked to file path
- **AND** update the UI to show the new comment immediately

#### Scenario: View comments
- **WHEN** file has comments
- **THEN** display comment count badge in file list
- **AND** display threaded discussion UI in the detail panel

## REMOVED Requirements
### Requirement: Directory Navigation
**Reason**: Merged into general navigation, no specific requirement change but clarifying flow.
**Migration**: None needed, logic remains, just UI focus shift.
(Actually, I will not remove this, as it's still valid. I will just leave it alone.)
