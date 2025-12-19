# Implementation Tasks: Commit Review in Content Area

**Feature**: 050-commit-review-content-area
**Branch**: `050-commit-review-content-area`
**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)
**Generated**: 2025-12-15

## Overview

This document provides a detailed task breakdown for implementing commit review functionality in the Content area of the Worktree TUI. Tasks are organized by implementation phase following the spec's structure.

**Note**: This feature has tightly coupled user stories that form a single coherent workflow. All stories must be implemented together as they depend on each other.

### User Stories Summary

All four user stories are delivered together in one implementation:
- **US1**: See ALL files in commit groups without truncation
- **US2**: Edit commit messages inline in Content area
- **US3**: Copy commit group details easily
- **US4**: Navigate between groups with keyboard shortcuts

### Implementation Strategy

**Single PR Delivery** (~1,000 lines total):
- All changes are tightly coupled and form a complete workflow
- Cannot be split into independent deliverables
- Complete feature testing only possible when all components are integrated

## Phase 1: Core Structure & State

**Goal**: Add enums and state fields to support commit review mode.

**Files Modified**:
- `crates/rstn/src/tui/views/worktree.rs`

### Tasks

- [X] T001 Add `CommitReview` variant to `ContentType` enum in crates/rstn/src/tui/views/worktree.rs
- [X] T002 Update `ContentType::name()` method to return "Commit Review" for new variant in crates/rstn/src/tui/views/worktree.rs
- [X] T003 Add 7 commit review state fields to `WorktreeView` struct in crates/rstn/src/tui/views/worktree.rs (commit_groups, current_commit_index, commit_message_input, commit_message_cursor, commit_warnings, commit_sensitive_files, commit_validation_error)
- [X] T004 Initialize commit review fields in `WorktreeView::new()` method in crates/rstn/src/tui/views/worktree.rs

**Completion Criteria**:
- [ ] Code compiles successfully with new fields
- [ ] No functional changes yet (fields initialized but unused)
- [ ] All existing tests pass

## Phase 2: State Management Methods

**Goal**: Implement methods to manage commit review workflow state.

**Files Modified**:
- `crates/rstn/src/tui/views/worktree.rs`

### Tasks

- [X] T005 Implement `start_commit_review()` method in crates/rstn/src/tui/views/worktree.rs (initialize workflow, check file counts, set content type, auto-focus Content pane)
- [X] T006 Implement `next_commit_group()` method in crates/rstn/src/tui/views/worktree.rs (increment index, load message, return bool)
- [X] T007 Implement `previous_commit_group()` method in crates/rstn/src/tui/views/worktree.rs (decrement index, load message, return bool)
- [X] T008 Implement `cancel_commit_review()` method in crates/rstn/src/tui/views/worktree.rs (clear state, return to Spec view)
- [X] T009 Implement `get_current_commit_message()` method in crates/rstn/src/tui/views/worktree.rs (return cloned message)
- [X] T010 Implement `validate_commit_message()` method in crates/rstn/src/tui/views/worktree.rs (check empty/whitespace, set validation_error)
- [X] T011 Implement helper `load_current_group_message()` private method in crates/rstn/src/tui/views/worktree.rs

**Completion Criteria**:
- [ ] All 6 public methods compile and have correct signatures
- [ ] Unit tests pass for navigation boundaries
- [ ] Unit tests pass for validation (empty/whitespace messages)
- [ ] Unit tests pass for state transitions

## Phase 3: Rendering

**Goal**: Render commit review UI in Content pane.

**Files Modified**:
- `crates/rstn/src/tui/views/worktree.rs`

### Tasks

- [X] T012 Implement `render_commit_review()` method in crates/rstn/src/tui/views/worktree.rs (render group number, message, files, warnings, navigation controls)
- [X] T013 Add inline validation error rendering in `render_commit_review()` in crates/rstn/src/tui/views/worktree.rs (show "⚠ Commit message cannot be empty" when validation fails)
- [X] T014 Add warnings rendering in `render_commit_review()` in crates/rstn/src/tui/views/worktree.rs (show file count warnings, security warnings)
- [X] T015 Update `render_content()` method to dispatch to `render_commit_review()` when content_type is CommitReview in crates/rstn/src/tui/views/worktree.rs
- [X] T016 Update tab title rendering to show "Commit Review" when in CommitReview mode in crates/rstn/src/tui/views/worktree.rs

**Completion Criteria**:
- [ ] UI renders correctly with mock commit groups
- [ ] Group number displays correctly (e.g., "1/6")
- [ ] All files visible without truncation
- [ ] Validation errors display inline
- [ ] Warnings display when present
- [ ] Navigation hints show correct key bindings

## Phase 4: Input Handling

**Goal**: Handle keyboard input for message editing and navigation.

**Files Modified**:
- `crates/rstn/src/tui/views/worktree.rs`

### Tasks

- [X] T017 Implement `handle_commit_review_input()` method skeleton in crates/rstn/src/tui/views/worktree.rs
- [X] T018 Add character input handling (insert at cursor) in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs
- [X] T019 Add backspace handling (delete before cursor) in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs
- [X] T020 Add delete key handling (delete after cursor) in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs
- [X] T021 Add arrow key handling (Left/Right cursor movement) in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs
- [X] T022 Add Home/End key handling (cursor to start/end) in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs
- [X] T023 Add 'n' key handling (next group) in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs
- [X] T024 Add 'p' key handling (previous group) in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs
- [X] T025 Add Enter key handling (validate and submit) in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs (call validate_commit_message, return ViewAction::SubmitCommitGroup if valid)
- [X] T026 Add Esc key handling (cancel workflow) in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs
- [X] T027 Update `handle_key_event()` to route to `handle_commit_review_input()` when in CommitReview mode in crates/rstn/src/tui/views/worktree.rs

**Completion Criteria**:
- [ ] Character input works correctly
- [ ] Backspace/delete work at all cursor positions
- [ ] Arrow keys move cursor correctly
- [ ] Home/End move to start/end
- [ ] n/p navigate between groups
- [ ] Enter validates and submits (or shows error)
- [ ] Esc cancels workflow
- [ ] UTF-8 cursor handling works (test with emoji)

## Phase 5: Events & Actions

**Goal**: Add events and actions for commit workflow, wire up handlers.

**Files Modified**:
- `crates/rstn/src/tui/event.rs`
- `crates/rstn/src/tui/views/mod.rs`
- `crates/rstn/src/tui/app.rs`

### Tasks

- [X] T028 [P] Add `CommitGroupCompleted` event to `Event` enum in crates/rstn/src/tui/event.rs
- [X] T029 [P] Add `CommitGroupFailed { error: String }` event to `Event` enum in crates/rstn/src/tui/event.rs
- [X] T030 [P] Add `IntelligentCommitFailed { error: String }` event to `Event` enum in crates/rstn/src/tui/event.rs
- [X] T031 Add `SubmitCommitGroup` to `ViewAction` enum in crates/rstn/src/tui/views/mod.rs
- [X] T032 Implement `Event::CommitGroupsReady` handler in crates/rstn/src/tui/app.rs (call worktree_view.start_commit_review())
- [X] T033 Implement `Event::CommitGroupCompleted` handler in crates/rstn/src/tui/app.rs (call worktree_view.next_commit_group(), if false then cancel_commit_review and show "All commits completed")
- [X] T034 Implement `Event::CommitGroupFailed` handler in crates/rstn/src/tui/app.rs (display error in Output, stay in review mode)
- [X] T035 Implement `Event::IntelligentCommitFailed` handler in crates/rstn/src/tui/app.rs (display error in Output, don't enter review mode)
- [X] T036 Implement `ViewAction::SubmitCommitGroup` handler in crates/rstn/src/tui/app.rs (spawn async task to call rstn_core::git::commit_group())

**Completion Criteria**:
- [ ] All events compile and are wired up
- [ ] Event handlers correctly update worktree_view state
- [ ] Async commit operations don't block UI
- [ ] Error events display in Output area
- [ ] Success events advance workflow

## Phase 6: Git Integration

**Goal**: Implement git commit functionality for commit groups.

**Files Modified**:
- `crates/rstn-core/src/git/commit.rs`

### Tasks

- [X] T037 Implement `commit_group()` function in crates/rstn-core/src/git/commit.rs (takes CommitGroup, executes git commit with files and message)
- [X] T038 Add error handling for git commit failures in `commit_group()` in crates/rstn-core/src/git/commit.rs (return Result<(), Error>)
- [X] T039 Add precondition check in `commit_group()` for empty file list in crates/rstn-core/src/git/commit.rs
- [X] T040 Add precondition check in `commit_group()` for empty message in crates/rstn-core/src/git/commit.rs

**Completion Criteria**:
- [ ] commit_group() successfully creates commits
- [ ] Errors are properly propagated via Result
- [ ] Integration test with real git repo passes
- [ ] Commit message formatting is correct
- [ ] All files in group are committed

## Phase 7: Logging

**Goal**: Add structured logging for commit review workflow.

**Files Modified**:
- `crates/rstn/src/tui/views/worktree.rs`
- `crates/rstn-core/src/git/commit.rs`

### Tasks

- [X] T041 [P] Add logging for workflow start in `start_commit_review()` in crates/rstn/src/tui/views/worktree.rs (log session_id, group_count)
- [X] T042 [P] Add logging for group submission in ViewAction::SubmitCommitGroup handler in crates/rstn/src/tui/app.rs (log session_id, group_index, message)
- [X] T043 [P] Add logging for commit success in Event::CommitGroupCompleted handler in crates/rstn/src/tui/app.rs (log session_id, group_index)
- [X] T044 [P] Add logging for commit failure in Event::CommitGroupFailed handler in crates/rstn/src/tui/app.rs (log session_id, group_index, error)
- [X] T045 [P] Add logging for workflow completion in `cancel_commit_review()` in crates/rstn/src/tui/views/worktree.rs (log session_id, completion_reason)

**Completion Criteria**:
- [ ] Logs appear in ~/.rustation/logs/rstn.log
- [ ] Key events are logged (start, submit, complete, cancel, error)
- [ ] Log entries have structured context (session_id, group_index)
- [ ] No excessive logging (no keystroke logging)

## Phase 8: Clipboard Integration

**Goal**: Enable copying commit review content to clipboard.

**Files Modified**:
- `crates/rstn/src/tui/views/worktree.rs`

### Tasks

- [X] T046 Implement `copy_commit_review()` method in crates/rstn/src/tui/views/worktree.rs (format group number, message, files; use arboard to copy)
- [X] T047 Add 'y' key handling for copying when focused on Content pane in `handle_commit_review_input()` in crates/rstn/src/tui/views/worktree.rs
- [X] T048 Add error handling for clipboard failures in `copy_commit_review()` in crates/rstn/src/tui/views/worktree.rs

**Completion Criteria**:
- [ ] 'y' key copies content to clipboard
- [ ] Clipboard contains group number, message, and all files
- [ ] Formatting is readable when pasted
- [ ] Clipboard errors are handled gracefully

## Phase 9: Status Bar Updates

**Goal**: Update status bar to show commit review progress.

**Files Modified**:
- `crates/rstn/src/tui/app.rs`
- `crates/rstn/src/tui/views/worktree.rs`

### Tasks

- [X] T049 Clear status bar in Event::CommitGroupsReady handler in crates/rstn/src/tui/app.rs
- [X] T050 Update status bar to show "Review commit N/M" when in CommitReview mode in crates/rstn/src/tui/app.rs or worktree.rs rendering
- [X] T051 Update status bar after each commit (increment N) in Event::CommitGroupCompleted handler in crates/rstn/src/tui/app.rs
- [X] T052 Show "All commits completed" message after last commit in Event::CommitGroupCompleted handler in crates/rstn/src/tui/app.rs

**Completion Criteria**:
- [ ] Status bar shows current progress (e.g., "Review commit 2/6")
- [ ] Progress updates after each successful commit
- [ ] "All commits completed" shows after last commit
- [ ] No stale "Analyzing..." messages

## Phase 10: Testing & Polish

**Goal**: Comprehensive testing and bug fixes.

**Files Modified**:
- `tests/commit_review_test.rs` (new file)
- Various files for bug fixes

### Tasks

- [ ] T053 Create integration test file tests/commit_review_test.rs
- [ ] T054 Write unit test for `start_commit_review()` in tests/commit_review_test.rs (verify state initialization)
- [ ] T055 Write unit test for `next_commit_group()` in tests/commit_review_test.rs (verify index increment, boundary check)
- [ ] T056 Write unit test for `previous_commit_group()` in tests/commit_review_test.rs (verify index decrement, boundary check)
- [ ] T057 Write unit test for `validate_commit_message()` in tests/commit_review_test.rs (empty, whitespace, valid)
- [ ] T058 Write unit test for character input handling in tests/commit_review_test.rs (insert at cursor, UTF-8 chars)
- [ ] T059 Write unit test for backspace/delete in tests/commit_review_test.rs (at start, middle, end)
- [ ] T060 Write unit test for cursor movement in tests/commit_review_test.rs (arrows, Home, End)
- [ ] T061 Write integration test for full workflow in tests/commit_review_test.rs (start → edit → submit → next → complete)
- [ ] T062 Write integration test for error handling in tests/commit_review_test.rs (commit failure mid-workflow)
- [ ] T063 Write integration test for validation errors in tests/commit_review_test.rs (empty message blocks submission)
- [ ] T064 Write integration test for >50 files warning in tests/commit_review_test.rs
- [ ] T065 Manual test: Full workflow with 6 groups
- [ ] T066 Manual test: Message editing (typing, backspace, arrows)
- [ ] T067 Manual test: Navigation (n/p keys)
- [ ] T068 Manual test: Copying (Tab to Content, press y)
- [ ] T069 Manual test: Validation (empty message shows error)
- [ ] T070 Manual test: Error recovery (commit failure preserves previous commits)
- [ ] T071 Manual test: No staged files error
- [ ] T072 Manual test: UTF-8 characters in message (emoji, international text)
- [X] T073 Run cargo clippy --all-targets and fix warnings
- [X] T074 Run cargo fmt to format code
- [X] T075 Run cargo test --package rstn and ensure all tests pass

**Completion Criteria**:
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All manual test scenarios pass
- [ ] No clippy warnings
- [ ] Code is properly formatted
- [ ] Performance benchmarks meet targets (<50ms render, <16ms input)

## Dependencies

### Phase Dependencies

```
Phase 1 (Core Structure)
  ↓
Phase 2 (State Management)
  ↓
Phase 3 (Rendering) ←──┐
  ↓                     │
Phase 4 (Input)         │
  ↓                     │
Phase 5 (Events) ───────┘
  ↓
Phase 6 (Git Integration)
  ↓
Phase 7 (Logging) ← Can run in parallel with Phase 8, 9
Phase 8 (Clipboard) ← Can run in parallel with Phase 7, 9
Phase 9 (Status Bar) ← Can run in parallel with Phase 7, 8
  ↓
Phase 10 (Testing & Polish)
```

### Critical Path

**Must complete in order**:
1. Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6

**Can parallelize**:
- Phase 7 (Logging)
- Phase 8 (Clipboard)
- Phase 9 (Status Bar)

**Final**:
- Phase 10 (Testing) - requires all previous phases

### File-Level Dependencies

**crates/rstn/src/tui/views/worktree.rs**:
- T001-T004 must complete before T005-T011
- T005-T011 must complete before T012-T016
- T012-T016 must complete before T017-T027

**Events require actions**:
- T031 (SubmitCommitGroup action) must exist before T036 (handler)

**Git integration blocks testing**:
- T037-T040 (commit_group) must complete before T061-T062 (integration tests)

## Parallel Execution Opportunities

### By File

**Independent files** (can be edited in parallel):
- `crates/rstn/src/tui/event.rs` (T028-T030)
- `crates/rstn/src/tui/views/mod.rs` (T031)
- `crates/rstn-core/src/git/commit.rs` (T037-T040)

**Dependent files** (must be sequential):
- `crates/rstn/src/tui/views/worktree.rs` (T001-T027, T041, T045-T048)
- `crates/rstn/src/tui/app.rs` (T032-T036, T042-T044, T049-T052) - depends on events existing

### By Task Type

**Parallelizable logging tasks** (T041-T045):
- All logging tasks marked with [P] can be done in parallel after their respective methods exist

**Parallelizable event tasks** (T028-T030):
- All new events can be added simultaneously to event.rs

## Task Summary

**Total Tasks**: 75
- **Phase 1** (Core Structure): 4 tasks
- **Phase 2** (State Management): 7 tasks
- **Phase 3** (Rendering): 5 tasks
- **Phase 4** (Input Handling): 11 tasks
- **Phase 5** (Events & Actions): 9 tasks
- **Phase 6** (Git Integration): 4 tasks
- **Phase 7** (Logging): 5 tasks (4 parallelizable)
- **Phase 8** (Clipboard): 3 tasks
- **Phase 9** (Status Bar): 4 tasks
- **Phase 10** (Testing & Polish): 23 tasks

**Parallelizable Tasks**: 8 (marked with [P])

**Estimated Effort**:
- Core implementation (Phases 1-6): ~4-5 hours
- Logging + Clipboard + Status (Phases 7-9): ~1-2 hours (can parallelize)
- Testing & Polish (Phase 10): ~2-3 hours
- **Total**: ~8-10 hours of focused development

## Implementation Workflow

### Recommended Approach

1. **Start**: Begin with Phase 1, commit after each phase
2. **Core Development**: Phases 1-6 sequentially (~4-5 hours)
3. **Parallel Work**: Phases 7-9 can be done together (~1-2 hours)
4. **Testing**: Phase 10 systematically (~2-3 hours)
5. **Review**: Code review, address feedback
6. **Merge**: Single PR to main

### Checkpoints

**After Phase 2**: State management methods compile and unit tests pass
**After Phase 4**: Full input handling works, manual testing possible
**After Phase 6**: End-to-end workflow functional
**After Phase 9**: All features complete, ready for comprehensive testing
**After Phase 10**: All tests pass, ready for PR

## Notes

- **No new dependencies**: All required crates already in Cargo.toml
- **Single PR**: All changes delivered together (~1,000 lines)
- **Constitution compliance**: All gates pass, no violations
- **Performance targets**: <50ms rendering, <16ms input response
- **Logging location**: ~/.rustation/logs/rstn.log

## Next Steps

After completing all tasks:

1. Run full test suite: `cargo test --package rstn`
2. Run clippy: `cargo clippy --all-targets`
3. Format code: `cargo fmt`
4. Manual testing with real git repo
5. Create PR using template from plan.md
6. Update CHANGELOG.md
