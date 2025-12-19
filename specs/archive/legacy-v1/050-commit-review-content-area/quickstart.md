# Quick Start: Commit Review in Content Area

**Feature**: 050-commit-review-content-area
**For**: Developers implementing this feature
**Estimated Reading Time**: 5 minutes

## What is This Feature?

Commit Review in Content Area replaces the modal dialog for intelligent commit workflows with an inline Content pane view. Users can review commit groups, edit messages, navigate between groups, and submit commits—all without modal interruptions.

## Key Concepts

### 1. Content Types

The WorktreeView has four content types:
- `Spec` - Display specification
- `Plan` - Display implementation plan
- `Tasks` - Display task breakdown
- **`CommitReview`** ← NEW - Review commit groups

### 2. Commit Review State

When in CommitReview mode, WorktreeView holds:
- **commit_groups**: List of commit groups to review
- **current_commit_index**: Which group is being reviewed (0-based)
- **commit_message_input**: User's edited message
- **commit_message_cursor**: Cursor position in message
- **commit_validation_error**: Inline validation error (if any)

### 3. Workflow

```
User → "Intelligent Commit" → Analyze staged files → Generate groups
  ↓
Switch to CommitReview mode → Show first group
  ↓
User edits message, presses Enter → Validate → Submit → Next group
  ↓
Repeat until all groups committed → Return to normal view
```

## Implementation Overview

### Files to Modify

1. **`crates/rstn/src/tui/views/worktree.rs`** (PRIMARY)
   - Add `CommitReview` to `ContentType` enum
   - Add 7 state fields to `WorktreeView` struct
   - Implement 8 public methods (see contracts/)
   - Add rendering for commit review UI
   - Add input handling for commit review mode

2. **`crates/rstn/src/tui/app.rs`**
   - Handle `Event::CommitGroupsReady`
   - Handle `Event::CommitGroupCompleted`
   - Handle `Event::CommitGroupFailed`
   - Handle `Event::IntelligentCommitFailed`
   - Handle `ViewAction::SubmitCommitGroup`

3. **`crates/rstn/src/tui/event.rs`**
   - Add 4 new events (see above)

4. **`crates/rstn/src/tui/views/mod.rs`**
   - Add `SubmitCommitGroup` to `ViewAction` enum

5. **`crates/rstn-core/src/git/commit.rs`**
   - Implement `commit_group(group: CommitGroup)` function
   - Handle git commit errors

### Core Methods to Implement

See [`contracts/worktree_view_api.md`](./contracts/worktree_view_api.md) for detailed API contracts.

**Essential 8 methods**:
1. `start_commit_review()` - Initialize review workflow
2. `next_commit_group()` - Navigate to next group
3. `previous_commit_group()` - Navigate to previous group
4. `cancel_commit_review()` - Exit review mode
5. `get_current_commit_message()` - Get edited message
6. `validate_commit_message()` - Check message validity
7. `handle_commit_review_input()` - Handle keyboard input
8. `render_commit_review()` - Render UI

### Data Model

See [`data-model.md`](./data-model.md) for complete entity definitions.

**Key structs**:
```rust
pub struct WorktreeView {
    // ... existing fields

    pub commit_groups: Option<Vec<rstn_core::CommitGroup>>,
    pub current_commit_index: usize,
    pub commit_message_input: String,
    pub commit_message_cursor: usize,
    pub commit_warnings: Vec<String>,
    pub commit_sensitive_files: Vec<String>,
    pub commit_validation_error: Option<String>,
}

pub enum ContentType {
    Spec,
    Plan,
    Tasks,
    CommitReview,  // NEW
}
```

## Step-by-Step Implementation

### Phase 1: Add Enums and State (30 min)

1. Add `CommitReview` to `ContentType` enum
2. Add 7 commit review fields to `WorktreeView` struct
3. Initialize fields in `WorktreeView::new()`
4. Update `ContentType::name()` method

**Test**: Compile successfully, no functional changes yet.

### Phase 2: Implement State Management Methods (1 hour)

1. Implement `start_commit_review()`
2. Implement `next_commit_group()`
3. Implement `previous_commit_group()`
4. Implement `cancel_commit_review()`
5. Implement `get_current_commit_message()`
6. Implement `validate_commit_message()`

**Test**: Unit tests for navigation, validation, state transitions.

### Phase 3: Implement Rendering (1 hour)

1. Create `render_commit_review()` method
2. Update `render_content()` to dispatch to `render_commit_review()`
3. Handle tab title for CommitReview mode
4. Style validation errors, warnings

**Test**: Manual test with mock data, verify UI renders correctly.

### Phase 4: Implement Input Handling (1 hour)

1. Create `handle_commit_review_input()` method
2. Handle character input (insert at cursor)
3. Handle backspace/delete
4. Handle cursor movement (arrows, Home, End)
5. Handle navigation (n, p keys)
6. Handle submission (Enter with validation)
7. Handle cancel (Esc key)

**Test**: Manual test all keyboard shortcuts work.

### Phase 5: Add Events and Wire Up (1 hour)

1. Add 4 new events to `Event` enum (event.rs)
2. Add `SubmitCommitGroup` to `ViewAction` (views/mod.rs)
3. Handle `Event::CommitGroupsReady` in app.rs
4. Handle `Event::CommitGroupCompleted` in app.rs
5. Handle `Event::CommitGroupFailed` in app.rs
6. Handle `ViewAction::SubmitCommitGroup` in app.rs

**Test**: End-to-end workflow with mock git operations.

### Phase 6: Implement Git Integration (30 min)

1. Add `commit_group()` function in rstn-core/src/git/commit.rs
2. Execute git commit with group's files and message
3. Handle errors (return Result)

**Test**: Integration test with real git repository.

### Phase 7: Add Logging (15 min)

1. Add `tracing::info!()` for workflow start
2. Add `tracing::info!()` for each group submission
3. Add `tracing::error!()` for commit failures
4. Add `tracing::info!()` for workflow completion

**Test**: Verify logs appear in `~/.rustation/logs/rstn.log`.

### Phase 8: Polish and Test (1 hour)

1. Test with 1 group, 10 groups, edge cases
2. Test validation errors
3. Test navigation boundaries
4. Test copying (y and Shift+Y)
5. Test status bar updates
6. Fix any bugs found

## Common Pitfalls

### 1. Cursor Position and UTF-8

**Problem**: Cursor position breaks on emoji or multibyte characters.

**Solution**: Always use `str::is_char_boundary()` to validate cursor position:
```rust
if !self.commit_message_input.is_char_boundary(cursor) {
    // Invalid! Adjust cursor to nearest boundary
}
```

### 2. Index Out of Bounds

**Problem**: `current_commit_index` exceeds `commit_groups.len()`.

**Solution**: Always check bounds in `next_commit_group()`:
```rust
if self.current_commit_index + 1 < groups.len() {
    self.current_commit_index += 1;
    return true;
}
false  // No more groups
```

### 3. Validation State Inconsistency

**Problem**: Validation error remains visible after user edits message.

**Solution**: Clear validation error on any character input:
```rust
fn handle_char_input(&mut self, c: char) {
    self.commit_message_input.insert(self.commit_message_cursor, c);
    self.commit_validation_error = None;  // Clear error
}
```

### 4. Modal vs. Non-Modal Confusion

**Problem**: Users try to Tab to other panes but input is captured.

**Solution**: Only capture input when `focus == WorktreeFocus::Content`:
```rust
if self.focus == WorktreeFocus::Content && self.content_type == ContentType::CommitReview {
    return self.handle_commit_review_input(key);
}
```

## Testing Checklist

- [ ] Workflow with 1 commit group
- [ ] Workflow with 10 commit groups
- [ ] Empty commit message validation
- [ ] Whitespace-only message validation
- [ ] Navigate to next group (n key)
- [ ] Navigate to previous group (p key)
- [ ] Submit group (Enter key)
- [ ] Cancel workflow (Esc key)
- [ ] Copy content (Tab to Content, y key)
- [ ] Commit failure mid-workflow
- [ ] No staged files error
- [ ] >50 files warning
- [ ] UTF-8 characters in message
- [ ] Cursor movement with arrows
- [ ] Backspace at start of message
- [ ] Delete at end of message

## Performance Benchmarks

Ensure these targets are met:

```rust
#[bench]
fn bench_render_commit_review(b: &mut Bencher) {
    // Target: <50ms
    b.iter(|| {
        worktree_view.render_commit_review(&mut frame, area);
    });
}

#[bench]
fn bench_handle_char_input(b: &mut Bencher) {
    // Target: <16ms (60 FPS)
    b.iter(|| {
        worktree_view.handle_commit_review_input(KeyEvent::from(KeyCode::Char('a')));
    });
}
```

## Debugging Tips

### Enable Tracing

```bash
RUST_LOG=debug cargo run
tail -f ~/.rustation/logs/rstn.log
```

### Check State in Debugger

Set breakpoint in `handle_commit_review_input()`, inspect:
- `self.content_type` (should be `CommitReview`)
- `self.current_commit_index` (should be < groups.len())
- `self.commit_message_cursor` (should be <= input.len())

### Verify Events

Add debug logging to event handlers:
```rust
Event::CommitGroupsReady { groups, .. } => {
    eprintln!("DEBUG: Received {} groups", groups.len());
    // ...
}
```

## Resources

- [Specification](./spec.md) - Full feature requirements
- [Data Model](./data-model.md) - Entity definitions
- [API Contracts](./contracts/worktree_view_api.md) - Method contracts
- [Research](./research.md) - Technology decisions
- [Ratatui Docs](https://ratatui.rs/) - TUI framework
- [Crossterm Docs](https://docs.rs/crossterm/) - Terminal I/O

## Next Steps

After implementing this feature:

1. **Run tests**: `cargo test --package rstn`
2. **Manual testing**: `cargo run --bin rstn` → Navigate to Worktree → Trigger "Intelligent Commit"
3. **Create PR**: Follow deployment strategy in plan.md
4. **Update CHANGELOG**: Document new feature in appropriate section

## Questions?

Refer to:
- **API questions**: See `contracts/worktree_view_api.md`
- **Data questions**: See `data-model.md`
- **Design questions**: See `research.md`
- **Requirements questions**: See `spec.md`

Good luck! 🚀
