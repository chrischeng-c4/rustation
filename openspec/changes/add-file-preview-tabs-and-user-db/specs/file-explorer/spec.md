## ADDED Requirements

### Requirement: File Preview Tabs
The system SHALL provide a tabbed interface for viewing files in the Detail Panel.

#### Scenario: Single click opens preview
- **WHEN** the user single-clicks a file in the file tree
- **THEN** it opens in a "Preview Tab" (italicized title)
- **AND** if a Preview Tab already exists, it is replaced by the new file
- **AND** the new tab becomes active

#### Scenario: Double click pins tab
- **WHEN** the user double-clicks a file OR double-clicks an existing Preview Tab
- **THEN** the tab is converted to a "Pinned Tab" (normal title)
- **AND** it is no longer replaced by subsequent single-clicks

#### Scenario: Tab management
- **WHEN** multiple tabs are open
- **THEN** the user can switch between them by clicking the tab header
- **AND** the user can close specific tabs via a close button

## MODIFIED Requirements

### Requirement: File Selection State
The system SHALL track the list of open tabs and the currently active tab.

#### Scenario: Active tab state
- **WHEN** a tab is selected
- **THEN** the application state updates `active_tab_path`
- **AND** the Detail Panel renders the content of that file
