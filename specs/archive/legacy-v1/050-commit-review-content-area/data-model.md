# Data Model: Commit Review in Content Area

**Feature**: 050-commit-review-content-area
**Date**: 2025-12-15

## Overview

This document defines the data structures and state management for commit review functionality in the Worktree TUI. All state is session-scoped (in-memory only, no persistence).

## Core Entities

### 1. ContentType (Enum Extension)

Represents the type of content displayed in the Content pane.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Spec,
    Plan,
    Tasks,
    CommitReview,  // NEW: Added for commit review mode
}

impl ContentType {
    fn name(&self) -> &'static str {
        match self {
            ContentType::Spec => "Spec",
            ContentType::Plan => "Plan",
            ContentType::Tasks => "Tasks",
            ContentType::CommitReview => "Commit Review",  // NEW
        }
    }
}
```

**Lifecycle**: Immutable enum, created fresh on each content type switch.

**Validation**: Type-safe enum, no runtime validation needed.

### 2. WorktreeView (State Extension)

Extended with commit review state fields.

```rust
pub struct WorktreeView {
    // === EXISTING FIELDS (not modified) ===
    pub feature_info: Option<FeatureInfo>,
    pub worktree_type: WorktreeType,
    pub content_type: ContentType,
    pub focus: WorktreeFocus,
    pub tab_index: usize,
    pub content_scroll: u16,
    pub log_buffer: LogBuffer,
    pub file_tracker: FileChangeTracker,
    pub spec_phases: Vec<(SpecPhase, PhaseStatus)>,
    pub autoflow_state: AutoFlowState,
    pub selected_command: usize,
    pub output_scroll: usize,

    // === NEW FIELDS: Commit Review State ===

    /// Current commit groups being reviewed (None when not in review mode)
    pub commit_groups: Option<Vec<rstn_core::CommitGroup>>,

    /// Index of the currently displayed commit group (0-based)
    pub current_commit_index: usize,

    /// User's edited commit message for current group
    pub commit_message_input: String,

    /// Cursor position in commit message (byte offset, UTF-8 aware)
    pub commit_message_cursor: usize,

    /// Security warnings from commit analysis
    pub commit_warnings: Vec<String>,

    /// Sensitive files detected in commits
    pub commit_sensitive_files: Vec<String>,

    /// Validation error message (None if valid)
    pub commit_validation_error: Option<String>,
}
```

**Field Details**:

- `commit_groups`: `Option<Vec<CommitGroup>>`
  - `None`: Not in commit review mode
  - `Some(vec![...])`: In review mode with N groups
  - **Validation**: Must have at least 1 group when Some

- `current_commit_index`: `usize`
  - Range: `0..commit_groups.len()`
  - **Validation**: Must be < length of commit_groups vector

- `commit_message_input`: `String`
  - UTF-8 string, pre-filled with Claude-generated message
  - **Validation**: Must not be empty or whitespace-only on submit

- `commit_message_cursor`: `usize`
  - Byte offset into commit_message_input
  - Range: `0..=commit_message_input.len()`
  - **Validation**: Must be valid UTF-8 char boundary

- `commit_warnings`: `Vec<String>`
  - Security warnings, file count warnings, etc.
  - **Validation**: None (display-only)

- `commit_sensitive_files`: `Vec<String>`
  - File paths flagged as sensitive (.env, credentials.json, etc.)
  - **Validation**: None (display-only)

- `commit_validation_error`: `Option<String>`
  - Error message to display inline
  - Examples: "Commit message cannot be empty", "Invalid UTF-8"
  - **Validation**: None (display-only)

### 3. CommitGroup (External Type from rstn-core)

Represents a logical group of files to commit together.

```rust
// Defined in rstn-core/src/git/commit.rs
pub struct CommitGroup {
    /// Claude-generated commit message
    pub message: String,

    /// File paths in this group (relative to repo root)
    pub files: Vec<PathBuf>,

    /// Optional description/reasoning for grouping
    pub description: Option<String>,
}
```

**Validation**:
- `message`: Must not be empty
- `files`: Must have at least 1 file, max 50 files (warning threshold)
- `description`: Optional, no validation

### 4. Event (Enum Extension)

Extended with commit review events.

```rust
#[derive(Debug, Clone)]
pub enum Event {
    // === EXISTING EVENTS (not modified) ===
    // ... (other events)

    // === NEW EVENTS: Commit Review ===

    /// Commit groups are ready to review
    CommitGroupsReady {
        groups: Vec<rstn_core::CommitGroup>,
        warnings: Vec<String>,
        sensitive_files: Vec<String>,
    },

    /// A commit group was successfully committed
    CommitGroupCompleted,

    /// A commit group failed to commit
    CommitGroupFailed {
        error: String,
    },

    /// Intelligent commit analysis failed (e.g., no staged files)
    IntelligentCommitFailed {
        error: String,
    },
}
```

**Event Flow**:
```
User triggers "Intelligent Commit"
  ↓
ViewAction::RunIntelligentCommit
  ↓
Spawn async: rstn_core::git::intelligent_commit()
  ↓
Event::IntelligentCommitFailed (if no staged files) OR
Event::CommitGroupsReady (if groups generated)
  ↓
User edits message, presses Enter
  ↓
ViewAction::SubmitCommitGroup
  ↓
Spawn async: rstn_core::git::commit_group()
  ↓
Event::CommitGroupCompleted (success) OR
Event::CommitGroupFailed (error)
```

### 5. ViewAction (Enum Extension)

Extended with commit review actions.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewAction {
    // === EXISTING ACTIONS (not modified) ===
    // ... (other actions)

    // === NEW ACTIONS: Commit Review ===

    /// Trigger intelligent commit workflow
    RunIntelligentCommit,

    /// Submit the current commit group and advance
    SubmitCommitGroup,
}
```

**Action Triggers**:
- `RunIntelligentCommit`: Triggered by user selecting "Intelligent Commit" from Git actions
- `SubmitCommitGroup`: Triggered by pressing Enter in commit review mode (after validation)

## State Transitions

### Commit Review State Machine

```
┌─────────────────┐
│   Normal View   │
│ (content_type   │
│  != CommitReview│
└────────┬────────┘
         │
         │ ViewAction::RunIntelligentCommit
         │
         ▼
┌─────────────────┐
│   Analyzing     │──► Event::IntelligentCommitFailed ──► Return to Normal
│ (async operation│
└────────┬────────┘
         │
         │ Event::CommitGroupsReady
         │
         ▼
┌─────────────────┐
│  Review Group   │◄──┐
│    (editing)    │   │ 'n' (next) or 'p' (previous)
└────────┬────────┘   │
         │            │
         │ Enter key  │
         │ (validated)│
         ▼            │
┌─────────────────┐   │
│   Committing    │   │
│ (async operation│   │
└────────┬────────┘   │
         │            │
         │ Event::CommitGroupCompleted
         │            │
         ├────────────┘ More groups?
         │
         │ Last group committed
         │
         ▼
┌─────────────────┐
│   Normal View   │
│ (review complete│
└─────────────────┘

Error paths:
  Event::CommitGroupFailed ──► Stay in Review Group, show error
  Esc key ──────────────────► Return to Normal (cancel workflow)
```

### Initialization

```rust
impl WorktreeView {
    pub fn new(...) -> Self {
        Self {
            // ... existing fields

            // Commit review fields initialized to "not in review mode"
            commit_groups: None,
            current_commit_index: 0,
            commit_message_input: String::new(),
            commit_message_cursor: 0,
            commit_warnings: Vec::new(),
            commit_sensitive_files: Vec::new(),
            commit_validation_error: None,
        }
    }
}
```

### State Transitions (Methods)

```rust
impl WorktreeView {
    /// Start commit review workflow
    pub fn start_commit_review(
        &mut self,
        groups: Vec<CommitGroup>,
        warnings: Vec<String>,
        sensitive_files: Vec<String>,
    ) {
        // Validate: at least one group
        assert!(!groups.is_empty(), "Must have at least one commit group");

        // Check file count warnings
        for (i, group) in groups.iter().enumerate() {
            if group.files.len() > 50 {
                self.commit_warnings.push(format!(
                    "Group {} has {} files (>50). Consider splitting.",
                    i + 1,
                    group.files.len()
                ));
            }
        }

        // Initialize first group
        let first_message = groups[0].message.clone();

        self.commit_groups = Some(groups);
        self.current_commit_index = 0;
        self.commit_message_input = first_message.clone();
        self.commit_message_cursor = first_message.len();  // Cursor at end
        self.commit_warnings.extend(warnings);
        self.commit_sensitive_files = sensitive_files;
        self.commit_validation_error = None;
        self.content_type = ContentType::CommitReview;
        self.focus = WorktreeFocus::Content;  // Auto-focus Content
    }

    /// Move to next commit group
    pub fn next_commit_group(&mut self) -> bool {
        if let Some(groups) = &self.commit_groups {
            if self.current_commit_index + 1 < groups.len() {
                self.current_commit_index += 1;
                self.load_current_group_message();
                return true;
            }
        }
        false  // No more groups
    }

    /// Move to previous commit group
    pub fn previous_commit_group(&mut self) -> bool {
        if self.current_commit_index > 0 {
            self.current_commit_index -= 1;
            self.load_current_group_message();
            return true;
        }
        false  // Already at first group
    }

    /// Load message from current group into input
    fn load_current_group_message(&mut self) {
        if let Some(groups) = &self.commit_groups {
            let message = groups[self.current_commit_index].message.clone();
            self.commit_message_input = message.clone();
            self.commit_message_cursor = message.len();
            self.commit_validation_error = None;
        }
    }

    /// Cancel commit review workflow
    pub fn cancel_commit_review(&mut self) {
        self.commit_groups = None;
        self.current_commit_index = 0;
        self.commit_message_input.clear();
        self.commit_message_cursor = 0;
        self.commit_warnings.clear();
        self.commit_sensitive_files.clear();
        self.commit_validation_error = None;
        self.content_type = ContentType::Spec;  // Return to Spec view
    }

    /// Get current commit message (with user edits)
    pub fn get_current_commit_message(&self) -> String {
        self.commit_message_input.clone()
    }

    /// Validate commit message
    pub fn validate_commit_message(&mut self) -> bool {
        let trimmed = self.commit_message_input.trim();
        if trimmed.is_empty() {
            self.commit_validation_error = Some("Commit message cannot be empty".to_string());
            return false;
        }
        self.commit_validation_error = None;
        true
    }
}
```

## Validation Rules

### Commit Message Validation

```rust
fn validate_commit_message(message: &str) -> Result<(), String> {
    let trimmed = message.trim();

    if trimmed.is_empty() {
        return Err("Commit message cannot be empty".to_string());
    }

    // Future: Could add more validations
    // - Conventional commit format check
    // - Message length limits
    // - Profanity filter

    Ok(())
}
```

### Cursor Position Validation

```rust
fn validate_cursor_position(input: &str, cursor: usize) -> Result<(), String> {
    if cursor > input.len() {
        return Err("Cursor position exceeds input length".to_string());
    }

    // Ensure cursor is at valid UTF-8 char boundary
    if !input.is_char_boundary(cursor) {
        return Err("Cursor not at valid character boundary".to_string());
    }

    Ok(())
}
```

### File Count Validation

```rust
fn check_file_count(group: &CommitGroup) -> Option<String> {
    if group.files.len() > 50 {
        return Some(format!(
            "Group has {} files (>50). Consider splitting this commit.",
            group.files.len()
        ));
    }
    None
}
```

## Memory Management

### Memory Estimates

```rust
// Per commit group
sizeof(CommitGroup) ≈ 24 bytes (Vec header)
+ N files × ~100 bytes/path ≈ 5,000 bytes (50 files)
+ message ~100 bytes
≈ 5,124 bytes per group

// Per review session (10 groups)
≈ 51,240 bytes (~50 KB)

// Additional UI state
commit_message_input: ~100 bytes
warnings: ~1 KB
Total session: ~52 KB
```

**Conclusion**: Memory footprint is negligible. No special memory management needed.

### Cleanup

```rust
impl Drop for WorktreeView {
    fn drop(&mut self) {
        // Rust automatically cleans up all Vec and String allocations
        // No manual cleanup needed
    }
}
```

## Relationships

### Entity Relationship Diagram

```
┌─────────────────┐
│  WorktreeView   │
└────────┬────────┘
         │
         │ 1:N (owns)
         │
         ▼
┌─────────────────┐
│  CommitGroup    │──────┐
│  (from rstn-core│      │ 1:N (owns)
└─────────────────┘      │
                         ▼
                 ┌──────────────┐
                 │   PathBuf    │
                 │  (file path) │
                 └──────────────┘

┌─────────────────┐
│     Event       │──────► triggers ──────┐
└─────────────────┘                       │
                                          ▼
┌─────────────────┐                ┌─────────────┐
│   ViewAction    │──────────────► │ State Change│
└─────────────────┘                └─────────────┘
```

## Invariants

### Must-Hold Invariants

1. **Commit groups validity**: If `commit_groups.is_some()`, the vector must not be empty
2. **Index bounds**: `current_commit_index < commit_groups.unwrap().len()`
3. **Cursor bounds**: `commit_message_cursor <= commit_message_input.len()`
4. **UTF-8 boundary**: `commit_message_cursor` must be at valid char boundary
5. **Content type consistency**: If `content_type == CommitReview`, then `commit_groups.is_some()`

### Validation Assertions

```rust
#[cfg(debug_assertions)]
fn check_invariants(&self) {
    if self.content_type == ContentType::CommitReview {
        assert!(self.commit_groups.is_some(), "Commit review mode requires groups");

        if let Some(groups) = &self.commit_groups {
            assert!(!groups.is_empty(), "Commit groups cannot be empty");
            assert!(self.current_commit_index < groups.len(), "Index out of bounds");
        }
    }

    assert!(
        self.commit_message_cursor <= self.commit_message_input.len(),
        "Cursor exceeds message length"
    );

    assert!(
        self.commit_message_input.is_char_boundary(self.commit_message_cursor),
        "Cursor not at char boundary"
    );
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_commit_review() {
        let mut view = WorktreeView::new(...);
        let groups = vec![
            CommitGroup {
                message: "feat: add feature".to_string(),
                files: vec![PathBuf::from("file1.rs")],
                description: None,
            },
        ];

        view.start_commit_review(groups.clone(), vec![], vec![]);

        assert_eq!(view.content_type, ContentType::CommitReview);
        assert_eq!(view.current_commit_index, 0);
        assert_eq!(view.commit_message_input, "feat: add feature");
    }

    #[test]
    fn test_validate_empty_message() {
        let mut view = WorktreeView::new(...);
        view.commit_message_input = "   ".to_string();

        assert!(!view.validate_commit_message());
        assert!(view.commit_validation_error.is_some());
    }

    #[test]
    fn test_next_previous_navigation() {
        let mut view = WorktreeView::new(...);
        let groups = vec![
            CommitGroup { message: "msg1".to_string(), files: vec![PathBuf::from("f1")], description: None },
            CommitGroup { message: "msg2".to_string(), files: vec![PathBuf::from("f2")], description: None },
        ];

        view.start_commit_review(groups, vec![], vec![]);

        assert_eq!(view.current_commit_index, 0);
        assert!(view.next_commit_group());
        assert_eq!(view.current_commit_index, 1);
        assert!(view.previous_commit_group());
        assert_eq!(view.current_commit_index, 0);
    }
}
```

## Summary

This data model provides:
- ✅ Clean state management for commit review workflow
- ✅ Type-safe enums for content types and events
- ✅ Clear validation rules and invariants
- ✅ Minimal memory footprint (~52 KB per session)
- ✅ Well-defined state transitions
- ✅ Testable business logic separated from UI

All entities align with Rust best practices and the rush shell constitution principles.
