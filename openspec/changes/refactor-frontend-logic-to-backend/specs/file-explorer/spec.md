## ADDED Requirements
### Requirement: Directory Expansion
The system SHALL track the expansion state of directories in the file explorer tree.

#### Scenario: Expand directory
- **WHEN** user clicks expand icon on a directory
- **THEN** dispatch `ExpandDirectory` action
- **AND** backend adds path to `expanded_paths`
- **AND** backend reads directory contents if not in cache
- **AND** frontend updates to show children

#### Scenario: Collapse directory
- **WHEN** user clicks collapse icon on a directory
- **THEN** dispatch `CollapseDirectory` action
- **AND** backend removes path from `expanded_paths`
- **AND** frontend updates to hide children

## MODIFIED Requirements
### Requirement: Directory Navigation
The system SHALL provide file tree navigation.

#### Scenario: Tree structure
- **WHEN** explorer is loaded
- **THEN** display root directory contents
- **AND** display contents of any paths in `expanded_paths`
- **AND** maintain scroll position

## REMOVED Requirements
### Requirement: File List Display
**Reason**: Replaced by Directory Expansion/Tree View logic which is more accurate to the implementation.
**Migration**: Use the new `directory_cache` and `expanded_paths` for rendering.
