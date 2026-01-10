## ADDED Requirements
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
