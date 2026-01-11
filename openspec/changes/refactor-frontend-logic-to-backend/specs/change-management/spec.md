## ADDED Requirements
### Requirement: Change Proposal Management
The system SHALL manage development changes (CESDD workflow).

#### Scenario: Create proposal
- **WHEN** user starts a new change
- **THEN** create change directory structure in `openspec/changes/`

### Requirement: Context File Management
The system SHALL validate and manage source files added to the change context.

#### Scenario: Validate context file
- **WHEN** user adds a file to context via `ValidateContextFile` action
- **THEN** backend validates file exists and is within project
- **AND** backend validates file type (text vs binary)
- **AND** backend returns validation result (Valid | Error)

#### Scenario: Add validated file
- **WHEN** file is valid
- **THEN** add path to `change.context_files`
