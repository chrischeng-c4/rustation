# File Explorer

Comprehensive file management system with Git integration and project annotations.

## Purpose

Enable developers to browse, navigate, and manage project files with Git status indicators, file previews, and collaborative comments. Provide a file management experience integrated with development workflow context.

## Requirements

### Requirement: Directory Navigation
The system SHALL provide advanced navigation with history support (back/forward).

#### Scenario: Navigate to directory
- **WHEN** user double-clicks a directory in file list
- **THEN** load and display directory contents, add current path to back stack

#### Scenario: Navigate back
- **WHEN** user clicks Back button or presses Backspace
- **THEN** navigate to previous directory from history stack

#### Scenario: Navigate forward
- **WHEN** user clicks Forward button
- **THEN** navigate to next directory from forward stack

#### Scenario: Navigate to parent
- **WHEN** user clicks parent directory in breadcrumbs or presses Backspace
- **THEN** navigate to parent directory

### Requirement: File List Display
The system SHALL display file entries with Git status indicators and metadata.

#### Scenario: Display files with Git status
- **WHEN** viewing a git repository directory
- **THEN** display files with color-coded status:
  - Green: Added
  - Yellow: Modified
  - Gray: Ignored
  - Red: Deleted
  - Blue: Untracked

#### Scenario: Display file metadata
- **WHEN** rendering file list
- **THEN** show name, size, permissions, last modified date, and comment count

#### Scenario: Sort files
- **WHEN** user clicks column header
- **THEN** sort entries by that column (name, size, date)

### Requirement: File Selection
The system SHALL support single file selection with preview.

#### Scenario: Select file
- **WHEN** user clicks on a file
- **THEN** highlight the file and load preview in detail panel

#### Scenario: Select different file
- **WHEN** user clicks on another file while one is selected
- **THEN** update selection and load new preview

### Requirement: File Operations
The system SHALL support creating, renaming, and deleting files.

#### Scenario: Create new file
- **WHEN** user selects "New File" from context menu
- **THEN** create empty file with prompted name in current directory

#### Scenario: Create new folder
- **WHEN** user selects "New Folder" from context menu
- **THEN** create directory with prompted name

#### Scenario: Rename file
- **WHEN** user presses F2 or selects "Rename" from context menu
- **THEN** show inline input to rename file

#### Scenario: Delete file
- **WHEN** user presses Delete or selects "Delete" from context menu
- **THEN** move file to system trash (not permanent deletion)

### Requirement: Context Menu
The system SHALL provide file-specific context menu actions.

#### Scenario: Open context menu
- **WHEN** user right-clicks on a file
- **THEN** display context menu with:
  - Open in OS (Finder/Explorer)
  - Copy Path
  - Copy Relative Path
  - Rename (F2)
  - Delete (Del)
  - New File
  - New Folder

#### Scenario: Reveal in OS
- **WHEN** user selects "Open in OS"
- **THEN** open system file manager at file location

### Requirement: Detail Panel
The system SHALL provide multi-tab detail view for selected files.

#### Scenario: Info tab
- **WHEN** user selects file and views Info tab
- **THEN** display full path, size, dates, and permissions (rwx-rwx-rwx)

#### Scenario: Preview tab for text files
- **WHEN** user selects text file and views Preview tab
- **THEN** display syntax-highlighted content (read-only)

#### Scenario: Preview tab for images
- **WHEN** user selects image file and views Preview tab
- **THEN** display image thumbnail

#### Scenario: Comments tab
- **WHEN** user views Comments tab
- **THEN** display threaded discussion UI for file-specific comments

### Requirement: File Comments
The system SHALL support adding and viewing comments on files, stored in local SQLite.

#### Scenario: Add comment
- **WHEN** user enters comment text and clicks Submit
- **THEN** persist comment to `.rstn/rstn.db` linked to file path

#### Scenario: View comments
- **WHEN** file has comments
- **THEN** display comment count badge in file list and show comments in detail panel

### Requirement: Breadcrumb Navigation
The system SHALL display clickable path breadcrumbs for current directory.

#### Scenario: Click breadcrumb segment
- **WHEN** user clicks on path segment in breadcrumbs
- **THEN** navigate to that directory

### Requirement: Git Integration
The system SHALL integrate with Git to show file status.

#### Scenario: Refresh Git status
- **WHEN** directory is loaded and is part of git repository
- **THEN** execute `git status --porcelain` asynchronously and populate status map

#### Scenario: Untracked files
- **WHEN** file is not tracked by git
- **THEN** display with "Untracked" status (blue)

### Requirement: Performance
The system SHALL handle directories with 10,000+ files efficiently.

#### Scenario: Load large directory
- **WHEN** directory contains more than 1000 files
- **THEN** use virtual scrolling to render only visible entries

### Requirement: Security
The system SHALL prevent path traversal attacks in file operations.

#### Scenario: Validate file path
- **WHEN** performing any file operation
- **THEN** validate path is within project root, reject if outside

## State Structure

```rust
pub struct FileExplorerState {
    pub current_path: String,
    pub entries: Vec<FileEntry>,
    pub selected_path: Option<String>,
    pub sort_config: SortConfig,
    pub filter_query: String,
    pub history: NavigationHistory,
    pub git_status: HashMap<String, GitFileStatus>,
    pub clipboard: Option<FileClipboard>,
    pub is_loading: bool,
    pub preview: Option<FilePreview>,
}

pub struct NavigationHistory {
    pub back_stack: Vec<String>,
    pub forward_stack: Vec<String>,
}

pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub kind: FileKind,  // File | Directory | Symlink
    pub size: u64,
    pub permissions: String,
    pub updated_at: String,
    pub comment_count: usize,
    pub git_status: Option<GitFileStatus>,
}

pub enum GitFileStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Ignored,
    Clean,
}
```

## Implementation References

- Backend: `packages/core/src/explorer/mod.rs`
- UI: `desktop/src/renderer/src/features/explorer/`
- State: `packages/core/src/reducer/explorer.rs`
- Database: `.rstn/rstn.db` (SQLite)
