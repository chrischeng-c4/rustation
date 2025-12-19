# Feature 050: Commit Review in Content Area

## Overview

Replace the modal dialog for intelligent commit workflow with a dynamic Content area view. This eliminates truncated file lists, enables easy copying, and provides better UX without modal interruptions.

## Problem Statement

Current intelligent commit workflow has three critical issues:

1. **Truncated files list**: Modal dialog description area limited to 3 lines - only shows "Files:" header, actual file list cut off
2. **Status bar clutter**: "Analyzing staged changes..." message stuck in footer even after groups are ready
3. **Cannot copy content**: Modal blocks pane navigation, Shift+Y doesn't capture full content, no way to copy commit details

## Clarifications

### Session 2025-12-15

- Q: If a commit operation fails mid-workflow (e.g., commit group 3 of 6 fails due to a git error), what should happen? → A: Stop workflow, keep reviewed groups (1-2 committed), show error in Output, allow user to fix and retry
- Q: What should happen if a user tries to submit a commit group with an empty or whitespace-only message? → A: Block submission, show inline error message "Commit message cannot be empty", keep focus in review mode
- Q: What is the maximum number of files expected in a single commit group for rendering and performance planning? → A: 50 files maximum - Beyond this, suggest splitting
- Q: Should commit review actions (navigation, edits, submissions) be logged for debugging purposes? → A: Log key events only: review start, group submissions, errors, and workflow completion/cancellation
- Q: What should happen if the user triggers "Intelligent Commit" when no files are staged? → A: Show error message "No staged files to commit" in Output area, don't enter review mode

## User Stories

### As a developer reviewing commit groups
- I want to see ALL files in each commit group without truncation
- So that I can verify which files are being grouped together

### As a developer editing commit messages
- I want to edit commit messages inline in the Content area
- So that I have a familiar editing experience without modal constraints

### As a developer copying commit details
- I want to copy commit group details (message + files) easily
- So that I can share or document what changes are being committed

### As a developer navigating commit groups
- I want to navigate between groups using keyboard shortcuts (n/p)
- So that I can quickly review all groups without mouse interaction

## Requirements

### Functional Requirements

**FR-1: Content Area Display**
- Display commit groups in the Content area instead of modal dialog
- Show group number (e.g., "Commit Group 1/6")
- Show commit message (editable)
- Show ALL files in the group (no truncation)
- Show navigation controls ([Enter] Submit, [n] Next, [p] Previous, [Esc] Cancel)
- Show warnings if any security issues detected

**FR-2: Message Editing**
- Support inline editing of commit message
- Support text input (typing characters)
- Support backspace/delete
- Support cursor movement (arrow keys)
- Pre-fill with Claude-generated message
- **Validation**: Block submission if message is empty or whitespace-only
- **Error feedback**: Display inline error "Commit message cannot be empty" when validation fails

**FR-3: Navigation**
- 'n' key to move to next commit group
- 'p' key to move to previous commit group
- Enter key to submit current group and auto-advance
- Esc key to cancel entire workflow
- Tab key to switch between panes (standard TUI behavior)

**FR-4: Copying**
- 'y' key to copy content when Content pane is focused
- Shift+Y to copy visual view including commit review
- Clipboard contains: group number, message, all files

**FR-5: Status Updates**
- Clear status bar when commit groups are ready
- Show "Review commit 1/6" during review
- Show "Review commit 2/6" after first commit, etc.
- Show "All commits completed" when done

**FR-6: Workflow**
- **Precondition**: If no files are staged, show error "No staged files to commit" in Output area and abort workflow
- On "Intelligent Commit" action:
  1. Show "Analyzing staged changes..." in Output area
  2. When groups ready, switch Content area to CommitReview mode
  3. Auto-focus Content area
  4. User reviews/edits each group
  5. Submit creates commit and moves to next group
  6. After last commit, return to normal view
  7. **Error handling**: If any commit fails, stop the workflow immediately, preserve all successfully committed groups, display error details in Output area, and allow user to fix the issue and retry or cancel

### Non-Functional Requirements

**NFR-1: Performance**
- Rendering commit review should be instant (<50ms)
- Navigation between groups should be instant
- **File limit**: Support up to 50 files per commit group; groups exceeding this should trigger a warning suggesting the user split the changes

**NFR-2: Usability**
- All keyboard shortcuts documented in UI
- Visual feedback for current group number
- Clear distinction between commit review mode and normal mode

**NFR-3: Maintainability**
- CommitReview state isolated in WorktreeView
- Event-driven architecture for async commit operations
- Clear separation between rendering and logic

**NFR-4: Observability**
- Log key workflow events for debugging:
  - Review workflow start (with group count)
  - Each successful commit group submission (group number, message)
  - Workflow completion or cancellation
  - Any errors during commit operations
- Use structured logging with context (session ID if available)

## Architecture

### Content Type Addition

Add `CommitReview` variant to `ContentType` enum:

```rust
pub enum ContentType {
    Spec,
    Plan,
    Tasks,
    CommitReview,  // NEW
}
```

### State Management

Add commit review state to `WorktreeView`:

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
    pub commit_validation_error: Option<String>,  // NEW: for inline validation errors
}
```

### Event Flow

```
User triggers "Intelligent Commit"
  ↓
ViewAction::RunIntelligentCommit
  ↓
Spawn async task → rstn_core::git::intelligent_commit()
  ↓
Event::CommitGroupsReady { groups, warnings, sensitive_files }
  ↓
worktree_view.start_commit_review(groups, warnings, sensitive_files)
  ↓
Content area displays CommitReview mode
  ↓
User edits message, presses Enter
  ↓
ViewAction::SubmitCommitGroup
  ↓
Spawn async task → rstn_core::git::commit_group(updated_group)
  ↓
Event::CommitGroupCompleted (success) OR Event::CommitGroupFailed (error)
  ↓
Success: worktree_view.next_commit_group() OR worktree_view.cancel_commit_review()
  ↓
Error: worktree_view.handle_commit_error(), display error in Output, stay in review mode
```

### Rendering

When `content_type == CommitReview`:

```
┌ Commit Review ────────────────────────────────────────┐
│                                                        │
│ Commit Group 1/6                                       │
│                                                        │
│ Message:                                               │
│   feat(050): add commit review in content area        │
│   ⚠ Commit message cannot be empty  (shown if validation fails)
│                                                        │
│ Files:                                                 │
│   - crates/rstn/src/tui/views/worktree.rs            │
│   - crates/rstn/src/tui/app.rs                       │
│   - crates/rstn/src/tui/event.rs                     │
│   - crates/rstn/src/tui/views/mod.rs                 │
│   - specs/050-commit-review-content-area/spec.md     │
│                                                        │
│ [Enter] Submit  [n] Next  [p] Previous  [Esc] Cancel  │
└────────────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Core Structure
1. Add `CommitReview` to `ContentType` enum
2. Add commit review state fields to `WorktreeView` struct
3. Add initialization in `WorktreeView::new()`

### Phase 2: Methods
4. Add `start_commit_review()` method
5. Add `next_commit_group()` method
6. Add `previous_commit_group()` method
7. Add `cancel_commit_review()` method
8. Add `get_current_commit_message()` method

### Phase 3: Rendering
9. Add `render_commit_review()` method
10. Update `render_content()` to handle `CommitReview` case
11. Handle tab title updates for CommitReview mode

### Phase 4: Input Handling
12. Add `handle_commit_review_input()` method
13. Update `handle_key_event()` to route to commit review handler
14. Handle char input, backspace, arrow keys, n, p, Enter, Esc
15. Add message validation on Enter (block if empty/whitespace, set validation_error)

### Phase 5: Events & Actions
15. Add `CommitGroupCompleted` and `CommitGroupFailed` to `Event` enum
16. Add `SubmitCommitGroup` to `ViewAction` enum
17. Update `Event::CommitGroupsReady` handler in app.rs
18. Add `ViewAction::SubmitCommitGroup` handler in app.rs
19. Add `Event::CommitGroupCompleted` handler in app.rs
20. Add `Event::CommitGroupFailed` handler in app.rs (display error, stay in review mode)

### Phase 6: Git Integration
20. Add `commit_group()` function to `rstn-core/src/git/commit.rs`
21. Handle commit errors and report to Output area

### Phase 7: Testing & Polish
22. Test full workflow end-to-end
23. Test message editing (typing, backspace, arrows)
24. Test navigation (n/p keys)
25. Test copying (y and Shift+Y)
26. Test status bar updates
27. Test error handling

## Files to Modify

1. `/Users/chrischeng/projects/rustation/crates/rstn/src/tui/views/worktree.rs`
   - ContentType enum + state fields + methods + rendering + input handling

2. `/Users/chrischeng/projects/rustation/crates/rstn/src/tui/app.rs`
   - Event handlers for CommitGroupsReady, SubmitCommitGroup, CommitGroupCompleted

3. `/Users/chrischeng/projects/rustation/crates/rstn/src/tui/event.rs`
   - Add CommitGroupCompleted event

4. `/Users/chrischeng/projects/rustation/crates/rstn/src/tui/views/mod.rs`
   - Add SubmitCommitGroup action

5. `/Users/chrischeng/projects/rustation/crates/rstn-core/src/git/commit.rs`
   - Add commit_group() function

## Success Criteria

### Must Have
- ✅ All files visible in commit review (no truncation)
- ✅ Can edit commit message inline
- ✅ Can navigate between groups (n/p keys)
- ✅ Can submit groups and auto-advance (Enter key)
- ✅ Can cancel workflow (Esc key)
- ✅ Can copy commit details (Tab to Content, press y)
- ✅ Status bar shows progress (e.g., "Review commit 2/6")
- ✅ Output area shows commit success messages

### Should Have
- ✅ Warning display if security issues detected
- ✅ Cursor position tracking in message editing
- ✅ Visual feedback for current group number

### Nice to Have
- Scroll support if file list exceeds screen height
- Syntax highlighting for commit message
- Diff preview for files in current group

## Testing Strategy

### Unit Tests
- `ContentType::name()` returns correct name for CommitReview
- Navigation methods update state correctly
- Message editing updates input string correctly

### Integration Tests
- Full workflow from trigger to completion
- Error handling when commit fails
- State cleanup when canceling

### Manual Tests
1. Run intelligent commit with 6 groups
2. Verify Content area shows group 1/6 with all files
3. Edit message, verify typing works
4. Press 'n', verify moves to group 2/6
5. Press 'p', verify moves back to group 1/6
6. Press Enter, verify commit created and moves to group 2/6
7. Tab to Content area, press 'y', verify clipboard has content
8. Press Esc, verify returns to normal view

## Risks & Mitigation

**Risk: Message editing conflicts with existing key bindings**
- Mitigation: Only handle input when `content_type == CommitReview`

**Risk: Commit failures leave UI in inconsistent state**
- Mitigation: Error events return to normal view, show error in Output

**Risk: Users accidentally cancel after reviewing many groups**
- Mitigation: Could add confirmation dialog (future enhancement)

## Rollout Plan

1. Implement core structure and methods
2. Test rendering with mock data
3. Implement event handling
4. Test with real intelligent commit workflow
5. Document keyboard shortcuts in UI
6. Merge to main branch
7. Update CHANGELOG.md

## Dependencies

- Existing intelligent commit infrastructure (already working)
- rstn-core CommitGroup type
- Event-driven architecture in TUI
- Clipboard integration (arboard crate)

## Metrics

- Number of commits created via intelligent commit
- Average time spent reviewing commit groups
- Frequency of Esc (cancel) vs Enter (submit)
- Copy action usage during commit review

## Future Enhancements

- Show diff preview for files in current group
- Allow editing individual file assignments (move file to different group)
- Remember last message edit for undo/redo
- Export commit groups to script for batch execution
- Integration with git hooks for pre-commit checks
