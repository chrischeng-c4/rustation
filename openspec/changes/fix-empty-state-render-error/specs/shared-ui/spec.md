## ADDED Requirements
### Requirement: EmptyState Component
The system SHALL provide a shared `EmptyState` component for displaying placeholder content when no data is available.

#### Scenario: Rendering with Icon
- **WHEN** the `EmptyState` component is rendered with an `icon` prop
- **THEN** the `icon` prop MUST be a valid `ReactNode` (e.g., a JSX Element `<Icon />`), NOT a component function.
- **AND** the component SHALL render the icon within a styled container.

#### Scenario: Rendering with Action
- **WHEN** the `EmptyState` component is rendered with an `action` prop containing an `icon`
- **THEN** the `action.icon` prop MUST be a valid `ReactNode` (e.g., `<Icon />`) to be passed to the underlying Button.
