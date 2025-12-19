# Tasks: TUI Input Keybindings

**Input**: Design documents from `/specs/048-tui-input-keybindings/`
**Prerequisites**: plan.md, spec.md

**Tests**: Update existing tests to use new keybindings. No new tests required.

**Organization**: This is a focused refactoring task. All user stories share the same implementation since they're all aspects of the same keybinding change.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Repository root**: `/Users/chrischeng/projects/rustation/`
- **rstn crate**: `crates/rstn/`

---

## Phase 1: Setup

**Purpose**: No setup required - existing Rust workspace project

> **SKIP**: Project already initialized with all dependencies (crossterm already has KeyModifiers::SHIFT).

---

## Phase 2: Implementation - Keybinding Change (All User Stories)

**Purpose**: Change Alt+Enter → Shift+Enter for newline, Enter → Submit

**Goal**: Users can submit with Enter and create newlines with Shift+Enter in multiline input dialogs

**Independent Test**: Run `cargo build && ./target/debug/rstn`, open a multiline input dialog, verify Enter submits and Shift+Enter creates newline

### Core Implementation

- [x] T001 [US1] Update key handling logic in `crates/rstn/src/tui/app.rs:654-672` - swap Alt+Enter to Shift+Enter for newline, Enter for submit
- [x] T002 [US2] Update help text in `crates/rstn/src/tui/widgets/input_dialog.rs:225-240` - change "Alt+Enter" to "Enter" for submit, add "Shift+Enter" for newline

**Checkpoint**: Core keybinding change complete

---

## Phase 3: Test Updates

**Purpose**: Update existing tests to use new keybindings

### Test File Updates

- [x] T003 [P] [US3] Update test `test_alt_enter_submits_multiline_input` in `crates/rstn/src/tui/app.rs` - renamed to `test_enter_submits_multiline_input`, added `test_shift_enter_creates_newline_in_multiline_input`
- [x] T004 [P] [US3] Update test `test_input_dialog_submits_on_alt_enter` in `crates/rstn/tests/sdd_workflow_test.rs` - renamed to `test_input_dialog_submits_on_enter`
- [x] T005 [P] [US3] Update test `test_alt_enter_submits_and_clears_dialog` in `crates/rstn/tests/e2e_tests/sdd_workflow_e2e.rs` - renamed to `test_enter_submits_and_clears_dialog`, fixed `test_multiline_input_with_newlines` to use Shift+Enter

**Checkpoint**: All tests updated to reflect new keybindings

---

## Phase 4: Verification & Polish

**Purpose**: Verify all tests pass and behavior works correctly

- [x] T006 Run `cargo test -p rstn` and verify all tests pass (87 tests passed)
- [ ] T007 Manual verification: Build and test in WezTerm terminal

---

## Dependencies & Execution Order

### Task Dependencies

```
T001 (app.rs key handling) → T002 (help text) → T003, T004, T005 (tests in parallel) → T006, T007 (verification)
```

### Parallel Opportunities

```bash
# After T002 completes, launch all test updates together:
Task T003: "Update test in crates/rstn/src/tui/app.rs"
Task T004: "Update test in crates/rstn/tests/sdd_workflow_test.rs"
Task T005: "Update test in crates/rstn/tests/e2e_tests/sdd_workflow_e2e.rs"
```

---

## Implementation Strategy

### Single PR Approach

This is a small, focused change (~50 lines). All tasks can be completed in a single PR:

1. Complete T001-T002 (core implementation)
2. Complete T003-T005 (test updates, parallel)
3. Complete T006-T007 (verification)
4. Create PR with all changes

### Estimated Effort

| Phase | Tasks | Estimated Lines |
|-------|-------|-----------------|
| Phase 2 (Implementation) | 2 tasks | ~20 lines |
| Phase 3 (Tests) | 3 tasks | ~30 lines |
| Phase 4 (Verification) | 2 tasks | 0 lines (verification only) |
| **Total** | **7 tasks** | **~50 lines** |

---

## Notes

- No new dependencies required (crossterm KeyModifiers::SHIFT already available)
- All 3 user stories (US1: Enter submit, US2: Shift+Enter newline, US3: No WezTerm conflict) are implemented together
- Single PR appropriate due to small scope
