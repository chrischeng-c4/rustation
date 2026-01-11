# Shared UI Components

Reusable UI components used across rustation's desktop application.

## Purpose

Provide a consistent library of shared React components that follow Material Design 3 principles and ensure consistent UX patterns throughout the application.
## Requirements
### Requirement: EmptyState Component
The system SHALL provide a shared `EmptyState` component for displaying placeholder content when no data is available.

#### Scenario: Rendering with Icon
- **WHEN** the `EmptyState` component is rendered with an `icon` prop
- **THEN** the `icon` prop MUST be a valid `ReactNode` (e.g., a JSX Element `<Icon />`), NOT a component function.
- **AND** the component SHALL render the icon within a styled container.

#### Scenario: Rendering with Action
- **WHEN** the `EmptyState` component is rendered with an `action` prop containing an `icon`
- **THEN** the `action.icon` prop MUST be a valid `ReactNode` (e.g., `<Icon />`) to be passed to the underlying Button.

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

### Requirement: Global Theme Density
The system SHALL use a compact Material Design 3 theme configuration to maximize information density via MUI's `defaultProps` pattern.

#### Scenario: Component Default Size
- **WHEN** UI components are rendered
- **THEN** they SHALL use `size: 'small'` as default for: Button, IconButton, TextField, Select, Chip, Table

#### Scenario: Dense Lists and Menus
- **WHEN** lists or menus are rendered
- **THEN** they SHALL use `dense: true` as default for: List, MenuItem

#### Scenario: Compact Toolbar
- **WHEN** toolbars are rendered
- **THEN** they SHALL use `variant: 'dense'` as default

### Requirement: Source Code Viewer
The system SHALL provide a reusable component for viewing source code with syntax highlighting and interaction capabilities.

#### Scenario: Syntax Highlighting
- **WHEN** displaying a supported file type (e.g., .rs, .ts, .json)
- **THEN** render content with color-coded syntax highlighting

#### Scenario: Line Numbers
- **WHEN** rendered with `showLineNumbers=true`
- **THEN** display line numbers in a separate gutter

#### Scenario: Inline Comment Display
- **WHEN** provided with a list of comments containing line numbers
- **THEN** render comment blocks immediately below the corresponding code line
- **AND** adjust layout to prevent obscuring code

#### Scenario: Inline Comment Creation
- **WHEN** `onAddComment` callback is provided
- **THEN** allow user to click on line numbers or gutter to trigger comment creation UI

