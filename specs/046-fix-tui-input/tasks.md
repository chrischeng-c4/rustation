# Tasks: Fix TUI Input Dialog Bug

**Input**: Design documents from `/specs/046-fix-tui-input/`
**Prerequisites**: plan.md (required), spec.md (required), research.md

**Tests**: Tests are REQUIRED - this feature explicitly requires 12+ unit tests, 6+ integration tests, and 3+ E2E tests.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Project**: `crates/rstn/` (rstn crate within rustation monorepo)
- **Source**: `crates/rstn/src/tui/`
- **Tests**: `crates/rstn/tests/`

---

## Phase 1: Setup (Investigation Infrastructure)

**Purpose**: Add debug logging to diagnose the bug

- [x] T001 Add tracing::debug! to handle_key_event() entry point in crates/rstn/src/tui/app.rs:105-110
- [x] T002 [P] Add tracing::debug! to ViewAction::RequestInput handling in crates/rstn/src/tui/app.rs:413-428
- [x] T003 [P] Add tracing::debug! to handle_key_event_in_input_mode() in crates/rstn/src/tui/app.rs:562-620
- [x] T004 Run TUI with RUST_LOG=debug and capture logs when triggering Specify workflow (SKIPPED - found bug via code analysis)

**Checkpoint**: Debug logging in place, ready to analyze bug ✅

---

## Phase 2: Foundational (Bug Analysis)

**Purpose**: Analyze logs and identify root cause - MUST complete before fix implementation

**CRITICAL**: No fix implementation can begin until root cause is identified

- [x] T005 Analyze debug logs to identify where keyboard input is lost (found via code analysis)
- [x] T006 Verify hypothesis 1: Check if input_mode state synchronization is the issue (NOT the cause)
- [x] T007 [P] Verify hypothesis 2: Check if event thread race condition exists (NOT the cause)
- [x] T008 [P] Verify hypothesis 3: Check crossterm raw mode configuration (NOT the cause)
- [x] T009 Document root cause finding in research.md

**ROOT CAUSE**: Layout constraints in input_dialog.rs:172-187 used Constraint::Length(1) for input area regardless of multiline mode. This caused the area.height > 1 check in render_input():246 to fail, preventing TextInput from rendering.

**Checkpoint**: Root cause identified and documented ✅

---

## Phase 3: User Story 1 - Fix Input Dialog Keyboard Input (Priority: P1) MVP

**Goal**: Fix the critical bug blocking keyboard input in the Specify workflow input dialog

**Independent Test**: Launch rstn TUI, navigate to Worktree > SDD Workflow > Specify, press Enter, and verify typed characters appear

### Implementation for User Story 1

- [x] T010 [US1] Implement fix for identified root cause in crates/rstn/src/tui/widgets/input_dialog.rs:172-187
- [x] T011 [US1] Add defensive check for input_mode and input_dialog state consistency in crates/rstn/src/tui/app.rs
- [x] T012 [US1] Add state validation in ViewAction::RequestInput handler in crates/rstn/src/tui/app.rs:413-428
- [ ] T013 [US1] Manual test: Verify typing works in Specify input dialog
- [ ] T014 [US1] Manual test: Verify Backspace deletes characters
- [ ] T015 [US1] Manual test: Verify Alt+Enter submits multiline input
- [ ] T016 [US1] Manual test: Verify Escape cancels input dialog
- [x] T017 [US1] Run existing test suite to verify no regressions: cargo test -p rstn (70 tests pass)

**Checkpoint**: Bug fix complete - Specify workflow keyboard input works

---

## Phase 4: User Story 2 - Unit Tests for Input Handling (Priority: P2)

**Goal**: Add 12+ unit tests for input dialog and keyboard handling to prevent regressions

**Independent Test**: Run `cargo test -p rstn` and verify all new unit tests pass

### Tests for User Story 2

- [ ] T018 [P] [US2] Add test_new_creates_active_input in crates/rstn/src/tui/widgets/input_dialog.rs
- [ ] T019 [P] [US2] Add test_insert_char_forwards_to_input in crates/rstn/src/tui/widgets/input_dialog.rs
- [ ] T020 [P] [US2] Add test_multiline_insert_char in crates/rstn/src/tui/widgets/input_dialog.rs
- [ ] T021 [P] [US2] Add test_cursor_movement_methods in crates/rstn/src/tui/widgets/input_dialog.rs
- [ ] T022 [P] [US2] Add test_backspace_deletes_char in crates/rstn/src/tui/widgets/input_dialog.rs
- [ ] T023 [P] [US2] Add test_multiline_newline_insert in crates/rstn/src/tui/widgets/input_dialog.rs
- [ ] T024 [P] [US2] Add test_request_input_sets_input_mode in crates/rstn/src/tui/app.rs (test module)
- [ ] T025 [P] [US2] Add test_request_input_creates_dialog in crates/rstn/src/tui/app.rs (test module)
- [ ] T026 [P] [US2] Add test_key_event_routes_to_input_mode_handler in crates/rstn/src/tui/app.rs (test module)
- [ ] T027 [P] [US2] Add test_enter_submits_single_line_input in crates/rstn/src/tui/app.rs (test module)
- [ ] T028 [P] [US2] Add test_alt_enter_submits_multiline_input in crates/rstn/src/tui/app.rs (test module)
- [ ] T029 [P] [US2] Add test_escape_cancels_input in crates/rstn/src/tui/app.rs (test module)
- [ ] T030 [US2] Run unit tests and verify 12+ new tests pass: cargo test -p rstn

**Checkpoint**: 12+ unit tests added and passing

---

## Phase 5: User Story 3 - Integration Tests for SDD Workflow (Priority: P2)

**Goal**: Add 6+ integration tests verifying the complete Specify workflow

**Independent Test**: Run `cargo test -p rstn --test sdd_workflow_test` and verify all tests pass

### Tests for User Story 3

- [ ] T031 [US3] Create test file crates/rstn/tests/sdd_workflow_test.rs with test harness setup
- [ ] T032 [P] [US3] Add test_specify_returns_request_input_action in crates/rstn/tests/sdd_workflow_test.rs
- [ ] T033 [P] [US3] Add test_request_input_creates_multiline_dialog in crates/rstn/tests/sdd_workflow_test.rs
- [ ] T034 [P] [US3] Add test_input_dialog_accepts_characters in crates/rstn/tests/sdd_workflow_test.rs
- [ ] T035 [P] [US3] Add test_input_dialog_handles_backspace in crates/rstn/tests/sdd_workflow_test.rs
- [ ] T036 [P] [US3] Add test_input_dialog_submits_on_alt_enter in crates/rstn/tests/sdd_workflow_test.rs
- [ ] T037 [P] [US3] Add test_input_dialog_cancels_on_escape in crates/rstn/tests/sdd_workflow_test.rs
- [ ] T038 [US3] Run integration tests and verify 6+ tests pass: cargo test -p rstn --test sdd_workflow_test

**Checkpoint**: 6+ integration tests added and passing

---

## Phase 6: User Story 4 - E2E Tests with TestBackend (Priority: P3)

**Goal**: Add 3+ E2E tests using ratatui TestBackend for visual verification

**Independent Test**: Run `cargo test -p rstn --test e2e` and verify all E2E tests pass

### Implementation for User Story 4

- [ ] T039 [US4] Create E2E test directory structure crates/rstn/tests/e2e/
- [ ] T040 [US4] Create TuiTestHarness struct in crates/rstn/tests/e2e/mod.rs with TestBackend setup
- [ ] T041 [US4] Implement send_key() method in TuiTestHarness in crates/rstn/tests/e2e/mod.rs
- [ ] T042 [US4] Implement send_text() method in TuiTestHarness in crates/rstn/tests/e2e/mod.rs
- [ ] T043 [US4] Implement render() and buffer_contains() methods in crates/rstn/tests/e2e/mod.rs

### E2E Tests for User Story 4

- [ ] T044 [US4] Create test file crates/rstn/tests/e2e/sdd_workflow_e2e.rs
- [ ] T045 [P] [US4] Add test_input_dialog_renders_in_buffer in crates/rstn/tests/e2e/sdd_workflow_e2e.rs
- [ ] T046 [P] [US4] Add test_typed_characters_appear_in_buffer in crates/rstn/tests/e2e/sdd_workflow_e2e.rs
- [ ] T047 [P] [US4] Add test_escape_removes_dialog_from_buffer in crates/rstn/tests/e2e/sdd_workflow_e2e.rs
- [ ] T048 [US4] Run E2E tests and verify 3+ tests pass: cargo test -p rstn --test e2e

**Checkpoint**: E2E test infrastructure complete with 3+ tests passing

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and validation

- [ ] T049 Run full test suite and verify all tests pass: cargo test -p rstn
- [ ] T050 [P] Run clippy and fix any warnings: cargo clippy -p rstn --all-targets
- [ ] T051 [P] Run format check: cargo fmt -p rstn --check
- [ ] T052 Final manual test: Complete Specify workflow end-to-end
- [ ] T053 Update research.md with final root cause documentation
- [ ] T054 Verify test counts meet requirements (12+ unit, 6+ integration, 3+ E2E)

**Checkpoint**: All quality gates passed, feature ready for PR

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS bug fix
- **User Story 1 (Phase 3)**: Depends on Foundational - Bug fix (MVP)
- **User Story 2 (Phase 4)**: Depends on US1 - Unit tests
- **User Story 3 (Phase 5)**: Depends on US1 - Integration tests
- **User Story 4 (Phase 6)**: Depends on US1 - E2E tests
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Phase 2 (root cause identified) - MVP
- **User Story 2 (P2)**: Can start after US1 complete, tests the fix
- **User Story 3 (P2)**: Can start after US1 complete, parallel with US2
- **User Story 4 (P3)**: Can start after US1 complete, parallel with US2/US3

### Within Each User Story

- Investigation before fix implementation
- Fix before tests (for US1)
- Test harness before test cases (for US3, US4)
- All tests must pass before checkpoint

### Parallel Opportunities

- **Phase 1**: T002, T003 can run in parallel (different code sections)
- **Phase 2**: T007, T008 can run in parallel (different hypotheses)
- **Phase 4**: T018-T029 can run in parallel (different test functions)
- **Phase 5**: T032-T037 can run in parallel (different test functions)
- **Phase 6**: T045-T047 can run in parallel (different test functions)
- **US2, US3, US4** can run in parallel after US1 complete

---

## Parallel Example: User Story 2 (Unit Tests)

```bash
# Launch all unit tests for input_dialog.rs together:
Task: "Add test_new_creates_active_input in crates/rstn/src/tui/widgets/input_dialog.rs"
Task: "Add test_insert_char_forwards_to_input in crates/rstn/src/tui/widgets/input_dialog.rs"
Task: "Add test_multiline_insert_char in crates/rstn/src/tui/widgets/input_dialog.rs"
Task: "Add test_cursor_movement_methods in crates/rstn/src/tui/widgets/input_dialog.rs"

# Launch all unit tests for app.rs together:
Task: "Add test_request_input_sets_input_mode in crates/rstn/src/tui/app.rs"
Task: "Add test_request_input_creates_dialog in crates/rstn/src/tui/app.rs"
Task: "Add test_key_event_routes_to_input_mode_handler in crates/rstn/src/tui/app.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (add debug logging)
2. Complete Phase 2: Foundational (identify root cause)
3. Complete Phase 3: User Story 1 (fix bug)
4. **STOP and VALIDATE**: Test Specify workflow manually
5. Create PR #1 for bug fix

### Incremental Delivery

1. Complete Setup + Foundational + US1 → Bug fixed (MVP!)
2. Add US2 (Unit Tests) → Regression protection → PR #2
3. Add US3 (Integration Tests) → Workflow coverage → PR #3
4. Add US4 (E2E Tests) → Visual verification → PR #4

### PR Strategy

| PR | Content | Estimated Lines |
|----|---------|-----------------|
| PR #1 | Bug fix (US1) | ≤ 400 |
| PR #2 | Unit tests (US2) | ≤ 500 |
| PR #3 | Integration tests (US3) | ≤ 400 |
| PR #4 | E2E tests (US4) | ≤ 500 |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Total: 54 tasks across 7 phases
- Test counts: 12 unit (T018-T029), 6 integration (T032-T037), 3 E2E (T045-T047) = 21 total

### Success Criteria Mapping

| Success Criteria | Tasks |
|------------------|-------|
| SC-001: Users can type in dialog | T010-T016 |
| SC-002: Existing tests pass | T017, T049 |
| SC-003: 12+ unit tests | T018-T030 |
| SC-004: 6+ integration tests | T031-T038 |
| SC-005: 3+ E2E tests | T039-T048 |
| SC-006: No regressions | T049-T051 |
| SC-007: Instantaneous feedback | T010-T013 |
