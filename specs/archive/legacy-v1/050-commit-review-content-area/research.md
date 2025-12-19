# Research: Commit Review in Content Area

**Feature**: 050-commit-review-content-area
**Date**: 2025-12-15
**Status**: Complete

## Overview

This document captures research decisions made for implementing commit review functionality in the Content area of the Worktree TUI, replacing the existing modal dialog approach.

## Research Areas

### 1. TUI State Management Pattern

**Decision**: Use in-memory state fields in `WorktreeView` struct

**Rationale**:
- Existing pattern in rustation TUI codebase
- Session-scoped data (no persistence needed)
- Simple, performant, and Rust-idiomatic
- Aligns with Principle I (Performance-First) and V (Rust-Native)

**Alternatives Considered**:
- **External state management crate** (e.g., redux-rs): Rejected due to unnecessary complexity for session-scoped state
- **Separate CommitReviewState struct**: Considered but kept inline for simplicity (can extract later if needed)

**Implementation**:
```rust
pub struct WorktreeView {
    // Existing fields...

    // Commit review state
    pub commit_groups: Option<Vec<rstn_core::CommitGroup>>,
    pub current_commit_index: usize,
    pub commit_message_input: String,
    pub commit_message_cursor: usize,
    pub commit_warnings: Vec<String>,
    pub commit_sensitive_files: Vec<String>,
    pub commit_validation_error: Option<String>,
}
```

### 2. Event-Driven Architecture for Async Git Operations

**Decision**: Use tokio for async git operations, communicate via `Event` enum

**Rationale**:
- Existing pattern in rustation (already using tokio)
- Non-blocking UI during git commit operations
- Clean separation between UI and git logic
- Aligns with Principle I (Performance-First - "No blocking operations")

**Alternatives Considered**:
- **Synchronous git operations**: Rejected due to UI blocking during commits
- **Separate thread pool**: Rejected in favor of tokio (already in use)

**Implementation**:
```rust
// Events
Event::CommitGroupsReady { groups, warnings, sensitive_files }
Event::CommitGroupCompleted
Event::CommitGroupFailed { error }

// Actions
ViewAction::RunIntelligentCommit
ViewAction::SubmitCommitGroup
```

### 3. Text Input Handling in TUI

**Decision**: Inline character-by-character editing with cursor tracking

**Rationale**:
- Familiar text editing experience for users
- Allows full control over commit message
- Simple implementation using Rust string manipulation
- No external text editor dependency

**Alternatives Considered**:
- **Launch external editor** (e.g., $EDITOR): Rejected due to modal interruption and complexity
- **Use text_input widget**: Evaluated but chose custom implementation for full control

**Implementation**:
```rust
fn handle_char_input(&mut self, c: char) {
    self.commit_message_input.insert(self.commit_message_cursor, c);
    self.commit_message_cursor += c.len_utf8();
    self.commit_validation_error = None; // Clear error on edit
}

fn handle_backspace(&mut self) {
    if self.commit_message_cursor > 0 {
        let idx = self.commit_message_cursor;
        let removed_char = self.commit_message_input.remove(idx - 1);
        self.commit_message_cursor -= removed_char.len_utf8();
        self.commit_validation_error = None;
    }
}
```

### 4. Clipboard Integration

**Decision**: Use `arboard` crate for cross-platform clipboard support

**Rationale**:
- Already in dependencies (Cargo.toml shows `arboard = "3.4"`)
- Pure Rust implementation (aligns with Principle V)
- Cross-platform (macOS, Linux, Windows)
- Simple API for copy operations

**Alternatives Considered**:
- **Manual clipboard implementation**: Rejected due to platform-specific complexity
- **cli-clipboard crate**: Rejected in favor of arboard (more maintained, better cross-platform support)

**Implementation**:
```rust
use arboard::Clipboard;

fn copy_commit_review(&self) -> Result<()> {
    let group = &self.commit_groups.as_ref()[self.current_commit_index];
    let content = format!(
        "Commit Group {}/{}\n\nMessage:\n{}\n\nFiles:\n{}",
        self.current_commit_index + 1,
        self.commit_groups.as_ref().unwrap().len(),
        self.commit_message_input,
        group.files.join("\n")
    );
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(content)?;
    Ok(())
}
```

### 5. Error Handling Strategy

**Decision**: Stop workflow on error, preserve completed commits, display error in Output area

**Rationale**:
- User explicitly chose this approach during clarification (Question 1)
- Prevents cascading failures
- Preserves user work (already committed groups)
- Allows user to fix issue and retry
- Clear error visibility in Output pane

**Alternatives Considered** (rejected during clarification):
- **Continue to next group**: Could hide errors and create inconsistent state
- **Rollback all commits**: Loses user work, potentially destructive
- **Auto-retry**: Could mask underlying issues

**Implementation**:
```rust
Event::CommitGroupFailed { error } => {
    // Stop workflow
    worktree_view.handle_commit_error(error);
    // Stay in review mode, display error in Output
    output_area.append_error(&format!("Commit failed: {}", error));
}
```

### 6. Input Validation Strategy

**Decision**: Client-side validation with inline error display

**Rationale**:
- User explicitly chose this approach during clarification (Question 2)
- Immediate feedback without round-trip to git
- Prevents bad commits proactively
- Consistent with git's validation rules

**Implementation**:
```rust
fn validate_commit_message(&mut self) -> bool {
    let trimmed = self.commit_message_input.trim();
    if trimmed.is_empty() {
        self.commit_validation_error = Some("Commit message cannot be empty".to_string());
        return false;
    }
    self.commit_validation_error = None;
    true
}

// On Enter key:
if self.validate_commit_message() {
    return Some(ViewAction::SubmitCommitGroup);
}
// else: stay in edit mode, show inline error
```

### 7. Performance Optimization for Large File Lists

**Decision**: Support up to 50 files per group, warn if exceeded

**Rationale**:
- User explicitly chose this limit during clarification (Question 3)
- Balances rendering performance with practical use cases
- 50 files × ~100 chars path = ~5KB rendering data (minimal)
- Encourages good commit hygiene (focused commits)

**Alternatives Considered**:
- **Unlimited files**: Could degrade rendering performance
- **Pagination**: Added complexity for rare edge case
- **Virtualization**: Overkill for 50-item limit

**Implementation**:
```rust
fn start_commit_review(&mut self, groups: Vec<CommitGroup>, warnings: Vec<String>, sensitive_files: Vec<String>) {
    // Check file count in each group
    for (i, group) in groups.iter().enumerate() {
        if group.files.len() > 50 {
            self.commit_warnings.push(format!(
                "Group {} has {} files (>50). Consider splitting this commit.",
                i + 1,
                group.files.len()
            ));
        }
    }

    self.commit_groups = Some(groups);
    self.current_commit_index = 0;
    self.commit_warnings.extend(warnings);
    self.commit_sensitive_files = sensitive_files;
    self.content_type = ContentType::CommitReview;
}
```

### 8. Logging Strategy

**Decision**: Log key workflow events only (start, submissions, errors, completion)

**Rationale**:
- User explicitly chose this approach during clarification (Question 4)
- Balances debuggability with log noise
- Structured logging with context
- Sufficient for troubleshooting without overwhelming logs

**Implementation**:
```rust
use tracing::{info, error, warn};

// Workflow start
info!(
    session_id = ?self.session_id,
    group_count = groups.len(),
    "Starting commit review workflow"
);

// Group submission
info!(
    session_id = ?self.session_id,
    group_index = self.current_commit_index,
    message = %self.commit_message_input,
    "Submitting commit group"
);

// Error
error!(
    session_id = ?self.session_id,
    group_index = self.current_commit_index,
    error = %err,
    "Commit group failed"
);

// Completion
info!(
    session_id = ?self.session_id,
    total_groups = self.commit_groups.as_ref().unwrap().len(),
    "Commit review workflow completed"
);
```

### 9. Edge Case Handling: No Staged Files

**Decision**: Show error message in Output area, don't enter review mode

**Rationale**:
- User explicitly chose this approach during clarification (Question 5)
- Consistent with git behavior (can't commit nothing)
- Clear error feedback
- Prevents entering invalid state

**Implementation**:
```rust
// In intelligent_commit() handler
let status = run_git_status()?;
if status.staged_files.is_empty() {
    return Err(anyhow!("No staged files to commit"));
}

// In event handler
Event::IntelligentCommitFailed { error } => {
    output_area.append_error(&error);
    // Don't enter commit review mode
}
```

## Best Practices Applied

### Rust TUI Development
- **Separation of concerns**: Rendering logic separate from state management
- **Immutability where possible**: Use `&self` for rendering, `&mut self` only for state changes
- **Error handling**: Propagate errors via `Result<>`, display in UI
- **Performance**: Pre-compute layout constraints, avoid allocations in render loop

### Event-Driven Architecture
- **Single event loop**: All async operations communicate via Event enum
- **Idempotent event handlers**: Events can be replayed without side effects
- **Clear state transitions**: Each event moves state machine forward

### Accessibility
- **Keyboard-first**: All operations accessible via keyboard
- **Visual hierarchy**: Clear focus indicators, group numbers, navigation hints
- **Error visibility**: Inline validation errors, clear error messages in Output

## Dependencies Summary

All dependencies are existing in the project:
- `ratatui = "0.29"` - TUI framework (existing)
- `crossterm = "0.28"` - Terminal I/O (existing)
- `arboard = "3.4"` - Clipboard integration (existing)
- `tokio` - Async runtime (existing)
- `rstn-core` - Git operations (existing)
- `tracing` - Structured logging (existing)

**No new dependencies required**.

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Message editing conflicts with existing key bindings | Only handle input when `content_type == CommitReview` |
| Commit failures leave UI in inconsistent state | Error events return to normal view or stay in review mode, show error in Output |
| Users accidentally cancel after reviewing many groups | Esc key clearly labeled as "Cancel" - future enhancement could add confirmation |
| Large file lists degrade rendering | 50-file limit with warning, encourage splitting |
| Async commit operations delay feedback | Show "Committing..." status, event-driven completion notification |

## Research Completion

All technical unknowns have been resolved. No "NEEDS CLARIFICATION" items remaining.

**Ready to proceed to Phase 1: Design & Contracts**.
