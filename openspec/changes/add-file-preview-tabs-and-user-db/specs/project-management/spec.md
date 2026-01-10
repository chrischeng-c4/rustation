## MODIFIED Requirements

### Requirement: Database Persistence
The system SHALL persist structured data to a user-scoped global SQLite database (`~/.rstn/state.db`).

#### Scenario: Data isolation
- **WHEN** data (comments, logs) is written to the database
- **THEN** it MUST include a `project_id` column derived from the project path hash
- **AND** queries MUST filter by this `project_id` to ensure data isolation between projects

#### Scenario: Fresh start
- **WHEN** the application starts with the new database configuration
- **THEN** it SHALL create the new global database if missing
- **AND** it SHALL NOT migrate data from legacy `.rstn/rstn.db` files (legacy data is ignored)
