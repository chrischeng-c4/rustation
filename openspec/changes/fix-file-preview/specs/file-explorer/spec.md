## MODIFIED Requirements
### Requirement: File Comments
The system SHALL support adding and viewing comments on files, stored in local SQLite.

#### Scenario: Add file comment
- **WHEN** user enters comment text in the Comments tab and clicks Submit
- **THEN** persist comment to `.rstn/rstn.db` linked to file path (without line number)

#### Scenario: Add inline comment
- **WHEN** user clicks on a line number in the Preview tab and submits text
- **THEN** persist comment to `.rstn/rstn.db` linked to file path AND line number

#### Scenario: View comments
- **WHEN** file has comments
- **THEN** display comment count badge in file list
- **AND** show all comments in Comments tab (threaded)
- **AND** show inline comments in Preview tab (embedded in code)
