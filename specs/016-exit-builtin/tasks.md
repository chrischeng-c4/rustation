# Tasks: Exit Builtin Command

**Input**: Design documents from `/specs/016-exit-builtin/`
**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included as part of implementation (unit tests in module, integration tests in test file).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Source**: `crates/rush/src/`
- **Tests**: `crates/rush/src/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Module creation and registration

- [x] T001 Create exit builtin module file at `crates/rush/src/executor/builtins/exit.rs`
- [x] T002 Add `pub mod exit;` to `crates/rush/src/executor/builtins/mod.rs`
- [x] T003 Add `exit` dispatch entry to `execute_builtin()` match in `crates/rush/src/executor/builtins/mod.rs`

**Checkpoint**: Module registered - implementation can now begin

---

## Phase 2: User Story 1 + 2 - Basic Exit Functionality (Priority: P1)

**Goal**: Exit shell with default status (last command's exit code) or explicit exit code

**Independent Test**: Type `exit` in shell and verify shell terminates; type `exit 42` and verify exit code 42

**Note**: US1 and US2 are combined because they share 95% of implementation - just different argument handling.

### Implementation for User Stories 1 & 2

- [x] T004 [US1] Implement `execute()` function skeleton in `crates/rush/src/executor/builtins/exit.rs`
- [x] T005 [US1] Add ExitRequest error variant to `crates/rush/src/lib.rs` (error module)
- [x] T006 [US1] Implement no-argument case: exit with last exit code in `crates/rush/src/executor/builtins/exit.rs`
- [x] T007 [US2] Implement single-argument case: parse and exit with explicit code in `crates/rush/src/executor/builtins/exit.rs`
- [x] T008 [US1] Handle ExitRequest in REPL for clean shell termination in `crates/rush/src/repl/mod.rs`
- [x] T009 [US1] Handle ExitRequest in main.rs for single-command mode in `crates/rush/src/main.rs`
- [x] T010 [P] [US1] Add unit test: test_exit_no_args in `crates/rush/src/executor/builtins/exit.rs`
- [x] T011 [P] [US2] Add unit test: test_exit_with_code in `crates/rush/src/executor/builtins/exit.rs`

**Checkpoint**: Basic exit functionality working - shell can be terminated

---

## Phase 3: User Story 3 - Exit Code Validation (Priority: P2)

**Goal**: Mask exit codes to 0-255 range per POSIX specification

**Independent Test**: Run `exit 256` and verify it wraps to 0; run `exit -1` and verify it wraps to 255

### Implementation for User Story 3

- [x] T012 [US3] Implement exit code masking `(value & 0xFF)` in `crates/rush/src/executor/builtins/exit.rs`
- [x] T013 [P] [US3] Add unit test: test_exit_code_wrapping_256 in `crates/rush/src/executor/builtins/exit.rs`
- [x] T014 [P] [US3] Add unit test: test_exit_code_negative in `crates/rush/src/executor/builtins/exit.rs`

**Checkpoint**: Exit code validation working - POSIX compliant

---

## Phase 4: User Story 4 - Error Handling (Priority: P2)

**Goal**: Clear error messages for invalid arguments, shell does not exit on error

**Independent Test**: Run `exit abc` and verify error message and shell continues

### Implementation for User Story 4

- [x] T015 [US4] Implement non-numeric argument error handling in `crates/rush/src/executor/builtins/exit.rs`
- [x] T016 [US4] Implement too-many-arguments error handling in `crates/rush/src/executor/builtins/exit.rs`
- [x] T017 [P] [US4] Add unit test: test_exit_non_numeric_arg in `crates/rush/src/executor/builtins/exit.rs`
- [x] T018 [P] [US4] Add unit test: test_exit_too_many_args in `crates/rush/src/executor/builtins/exit.rs`

**Checkpoint**: Error handling complete - invalid arguments handled gracefully

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Verification and cleanup

- [x] T019 Run `cargo clippy` and fix any warnings in exit.rs
- [x] T020 Run `cargo test -p rush` and verify all tests pass
- [x] T021 Run `cargo build --release` and verify clean build

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **US1+2 (Phase 2)**: Depends on Setup completion
- **US3 (Phase 3)**: Depends on Phase 2 completion (exit code logic exists)
- **US4 (Phase 4)**: Depends on Phase 2 completion (execute function exists)
- **Polish (Phase 5)**: Depends on all user stories complete

### Within Each Phase

- T001 → T002 → T003 (sequential - module must exist before export/registration)
- T004 → T005 → T006 → T007 (execution skeleton before specifics)
- T008 → T009 (executor before main loop)

### Parallel Opportunities

- T010, T011 can run in parallel (different test cases)
- T013, T014 can run in parallel (different test scenarios)
- T017, T018 can run in parallel (different test scenarios)
- Phase 3 and Phase 4 can run in parallel after Phase 2 completes

---

## Parallel Example: User Story 3 & 4 Tests

```bash
# After implementation complete, run tests in parallel:
Task: "Add unit test: test_exit_code_wrapping_256"
Task: "Add unit test: test_exit_code_negative"
Task: "Add unit test: test_exit_non_numeric_arg"
Task: "Add unit test: test_exit_too_many_args"
```

---

## Implementation Strategy

### MVP First (User Stories 1+2)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: US1+2 Implementation (T004-T011)
3. **STOP and VALIDATE**: Test exit command works
4. Create PR for MVP

### Incremental Delivery

1. Setup + US1+2 → Basic exit working → PR #1 (MVP)
2. If time permits in same PR:
   - Add US3 (exit code validation) → POSIX compliant
   - Add US4 (error handling) → Robust

### Single PR Strategy

Given the small feature size (~200 lines), all phases will be in a single PR:
1. Complete all phases T001-T021
2. Verify all tests pass
3. Create single PR with complete implementation

---

## Notes

- [P] tasks = different files or independent test cases
- [Story] label maps task to specific user story
- US1 and US2 combined because they share implementation
- Exit uses error propagation (ExitRequest) to signal termination
- Commit after each logical group of tasks
- Total: 21 tasks

### Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| Setup | T001-T003 | Module creation and registration |
| US1+2 | T004-T011 | Basic exit with default/explicit code |
| US3 | T012-T014 | Exit code validation (POSIX) |
| US4 | T015-T018 | Error handling |
| Polish | T019-T021 | Verification |

**Total Tasks**: 21
**Estimated Lines**: ~200
**PR Strategy**: Single PR (well under 500 line ideal)
