## ADDED Requirements

### Requirement: Active View Navigation
The system SHALL support navigation between active views within the main content area.

#### Scenario: Supported Views
- **WHEN** the application initializes or navigation is triggered
- **THEN** the system SHALL support the following active views:
  - `tasks`
  - `settings`
  - `dockers`
  - `env`
  - `mcp`
  - `chat`
  - `terminal`
  - `workflows`
  - `explorer`
  - `claude-code`
  - `a2ui`

#### Scenario: View State Persistence
- **WHEN** the user switches views
- **THEN** the active view selection SHALL be persisted in the application state.
