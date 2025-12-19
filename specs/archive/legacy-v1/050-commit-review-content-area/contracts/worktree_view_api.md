# WorktreeView Internal API Contract

**Feature**: 050-commit-review-content-area
**Type**: Internal Rust API
**Date**: 2025-12-15

## Overview

This document defines the internal API contract for commit review functionality in `WorktreeView`. These are public methods that other components (app.rs, event handlers) will call.

## Public Methods

### 1. start_commit_review

**Purpose**: Initialize commit review workflow with analyzed commit groups.

**Signature**:
```rust
pub fn start_commit_review(
    &mut self,
    groups: Vec<rstn_core::CommitGroup>,
    warnings: Vec<String>,
    sensitive_files: Vec<String>,
)
```

**Parameters**:
- `groups`: Vector of commit groups from intelligent commit analysis
- `warnings`: Security warnings and file count warnings
- `sensitive_files`: Paths to files flagged as sensitive

**Preconditions**:
- `groups` must not be empty (will panic in debug mode)
- Each group must have at least 1 file

**Postconditions**:
- `self.content_type == ContentType::CommitReview`
- `self.focus == WorktreeFocus::Content`
- `self.current_commit_index == 0`
- `self.commit_message_input` contains first group's message
- `self.commit_message_cursor` at end of message

**Side Effects**:
- Switches to CommitReview content mode
- Auto-focuses Content pane
- Generates file count warnings (>50 files)

**Example**:
```rust
worktree_view.start_commit_review(
    vec![
        CommitGroup {
            message: "feat(050): add commit review".to_string(),
            files: vec![PathBuf::from("worktree.rs")],
            description: None,
        },
    ],
    vec!["Sensitive file: .env".to_string()],
    vec![".env".to_string()],
);
```

---

### 2. next_commit_group

**Purpose**: Navigate to the next commit group in the sequence.

**Signature**:
```rust
pub fn next_commit_group(&mut self) -> bool
```

**Parameters**: None

**Returns**:
- `true`: Successfully moved to next group
- `false`: Already at last group (no change)

**Preconditions**:
- Must be in commit review mode (`commit_groups.is_some()`)

**Postconditions**:
- If `true`: `current_commit_index` incremented by 1
- If `true`: `commit_message_input` loaded with new group's message
- If `true`: `commit_validation_error` cleared
- If `false`: State unchanged

**Side Effects**:
- Updates input field with next group's message
- Resets cursor to end of message
- Clears any validation errors

**Example**:
```rust
if worktree_view.next_commit_group() {
    // Moved to next group
} else {
    // Already at last group
}
```

---

### 3. previous_commit_group

**Purpose**: Navigate to the previous commit group in the sequence.

**Signature**:
```rust
pub fn previous_commit_group(&mut self) -> bool
```

**Parameters**: None

**Returns**:
- `true`: Successfully moved to previous group
- `false`: Already at first group (no change)

**Preconditions**:
- Must be in commit review mode (`commit_groups.is_some()`)

**Postconditions**:
- If `true`: `current_commit_index` decremented by 1
- If `true`: `commit_message_input` loaded with prev group's message
- If `true`: `commit_validation_error` cleared
- If `false`: State unchanged

**Side Effects**:
- Updates input field with previous group's message
- Resets cursor to end of message
- Clears any validation errors

**Example**:
```rust
if worktree_view.previous_commit_group() {
    // Moved to previous group
} else {
    // Already at first group
}
```

---

### 4. cancel_commit_review

**Purpose**: Cancel commit review workflow and return to normal view.

**Signature**:
```rust
pub fn cancel_commit_review(&mut self)
```

**Parameters**: None

**Returns**: None

**Preconditions**: None (safe to call even if not in review mode)

**Postconditions**:
- `self.commit_groups == None`
- `self.current_commit_index == 0`
- `self.commit_message_input` cleared
- `self.commit_validation_error == None`
- `self.content_type == ContentType::Spec`

**Side Effects**:
- Abandons all uncommitted groups
- Clears all commit review state
- Switches back to Spec view

**Example**:
```rust
// User pressed Esc
worktree_view.cancel_commit_review();
```

---

### 5. get_current_commit_message

**Purpose**: Retrieve the user-edited commit message for the current group.

**Signature**:
```rust
pub fn get_current_commit_message(&self) -> String
```

**Parameters**: None

**Returns**: Cloned copy of `commit_message_input`

**Preconditions**: None

**Postconditions**: None (read-only)

**Side Effects**: None (immutable borrow)

**Example**:
```rust
let message = worktree_view.get_current_commit_message();
// Use message for git commit
```

---

### 6. validate_commit_message

**Purpose**: Validate the current commit message before submission.

**Signature**:
```rust
pub fn validate_commit_message(&mut self) -> bool
```

**Parameters**: None

**Returns**:
- `true`: Message is valid (can submit)
- `false`: Message is invalid (shows error)

**Preconditions**: None

**Postconditions**:
- If `false`: `commit_validation_error` set to error message
- If `true`: `commit_validation_error == None`

**Side Effects**:
- Sets or clears validation error field

**Validation Rules**:
- Message must not be empty after trimming whitespace
- Future: Could add conventional commit format check

**Example**:
```rust
if worktree_view.validate_commit_message() {
    return Some(ViewAction::SubmitCommitGroup);
} else {
    // Error shown inline, stay in edit mode
}
```

---

### 7. handle_commit_review_input

**Purpose**: Handle keyboard input during commit review mode.

**Signature**:
```rust
pub fn handle_commit_review_input(&mut self, key: KeyEvent) -> Option<ViewAction>
```

**Parameters**:
- `key`: crossterm keyboard event

**Returns**:
- `Some(ViewAction)`: Action to dispatch
- `None`: Input handled locally, no action

**Supported Keys**:
- `Char(c)`: Insert character at cursor
- `Backspace`: Delete character before cursor
- `Delete`: Delete character after cursor
- `Left`: Move cursor left
- `Right`: Move cursor right
- `Home`: Move cursor to start
- `End`: Move cursor to end
- `Enter`: Validate and submit (if valid)
- `'n'`: Move to next group
- `'p'`: Move to previous group
- `Esc`: Cancel workflow
- `Tab`: Switch pane focus (standard TUI behavior)

**Preconditions**:
- Must be called only when `content_type == CommitReview`

**Postconditions**:
- Cursor position remains valid (char boundary)
- Input string remains valid UTF-8

**Side Effects**:
- Modifies `commit_message_input` and `commit_message_cursor`
- Clears validation errors on edit
- May trigger ViewAction

**Example**:
```rust
match key.code {
    KeyCode::Enter => {
        if let Some(action) = worktree_view.handle_commit_review_input(key) {
            return action; // SubmitCommitGroup
        }
    }
    KeyCode::Char(c) => {
        worktree_view.handle_commit_review_input(key);
    }
    // ... etc
}
```

---

### 8. render_commit_review

**Purpose**: Render the commit review UI in the Content pane.

**Signature**:
```rust
pub fn render_commit_review(&self, frame: &mut Frame, area: Rect)
```

**Parameters**:
- `frame`: ratatui Frame for rendering
- `area`: Rect defining render area

**Returns**: None

**Preconditions**:
- Must be called only when `content_type == CommitReview`
- `commit_groups` must be `Some`

**Postconditions**: None (render-only)

**Side Effects**:
- Draws to terminal framebuffer (via ratatui)

**Rendered Elements**:
```
┌ Commit Review ─────────────────────────┐
│                                         │
│ Commit Group 1/6                        │
│                                         │
│ Message:                                │
│   feat(050): add commit review         │
│   ⚠ Commit message cannot be empty     │ (if validation error)
│                                         │
│ Files:                                  │
│   - crates/rstn/src/tui/worktree.rs   │
│   - crates/rstn/src/tui/event.rs      │
│                                         │
│ Warnings:                               │ (if any)
│   ⚠ Group has 75 files (>50)           │
│                                         │
│ [Enter] Submit  [n] Next  [p] Previous │
│ [Esc] Cancel                            │
└─────────────────────────────────────────┘
```

**Example**:
```rust
if worktree_view.content_type == ContentType::CommitReview {
    worktree_view.render_commit_review(frame, content_area);
}
```

---

## Event Integration

### Events Consumed

```rust
// In app.rs event handler
match event {
    Event::CommitGroupsReady { groups, warnings, sensitive_files } => {
        worktree_view.start_commit_review(groups, warnings, sensitive_files);
    }

    Event::CommitGroupCompleted => {
        if !worktree_view.next_commit_group() {
            // Last group completed
            worktree_view.cancel_commit_review();
            output.append_success("All commits completed");
        }
    }

    Event::CommitGroupFailed { error } => {
        // Stay in review mode, show error
        output.append_error(&format!("Commit failed: {}", error));
        // User can fix message and retry
    }

    Event::IntelligentCommitFailed { error } => {
        // Don't enter review mode
        output.append_error(&format!("Intelligent commit failed: {}", error));
    }
}
```

### Actions Produced

```rust
// From handle_commit_review_input
match key.code {
    KeyCode::Enter => {
        if self.validate_commit_message() {
            Some(ViewAction::SubmitCommitGroup)
        } else {
            None // Stay in edit mode with error
        }
    }
    KeyCode::Esc => {
        self.cancel_commit_review();
        None
    }
    // ... other keys handled internally
}
```

## Error Handling

### Validation Errors

Handled inline via `commit_validation_error` field:
```rust
pub commit_validation_error: Option<String>
```

Displayed in UI below message input.

### Commit Errors

Propagated via `Event::CommitGroupFailed`:
```rust
Event::CommitGroupFailed { error: String }
```

Displayed in Output pane, user stays in review mode to fix and retry.

### Panic Conditions (Debug Only)

```rust
#[cfg(debug_assertions)]
{
    assert!(!groups.is_empty(), "Commit groups cannot be empty");
    assert!(cursor <= input.len(), "Cursor out of bounds");
}
```

Release builds will not panic; errors logged and handled gracefully.

## Thread Safety

All methods are `&mut self`, single-threaded TUI context only.

Async git operations use tokio channels to send events back to main thread.

## Performance Guarantees

- **Rendering**: <50ms for groups up to 50 files
- **Input handling**: <16ms response time (60 FPS)
- **Navigation**: Instant (O(1) index update)
- **Validation**: O(n) where n = message length (typically <100 chars)

## Testing Checklist

- [ ] `start_commit_review` with 1 group
- [ ] `start_commit_review` with 10 groups
- [ ] `start_commit_review` with >50 files (warning generated)
- [ ] `next_commit_group` boundary conditions
- [ ] `previous_commit_group` boundary conditions
- [ ] `cancel_commit_review` clears state
- [ ] `validate_commit_message` with empty message
- [ ] `validate_commit_message` with whitespace-only message
- [ ] `handle_commit_review_input` with all key types
- [ ] UTF-8 cursor handling (emoji, multibyte chars)
- [ ] `render_commit_review` with validation errors
- [ ] `render_commit_review` with warnings

## Version History

- **v1.0** (2025-12-15): Initial API definition for feature 050
