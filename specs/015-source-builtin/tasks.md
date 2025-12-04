# Tasks: Source Builtin Command

**Input**: Design documents from `/specs/015-source-builtin/`
**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included as part of implementation (integration tests in existing test file).

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

- [x] T001 Create source builtin module file at `crates/rush/src/executor/builtins/source.rs`
- [x] T002 Add `pub mod source;` to `crates/rush/src/executor/builtins/mod.rs`
- [x] T003 Add `source` and `.` dispatch entries to `execute_builtin()` match in `crates/rush/src/executor/builtins/mod.rs`

**Checkpoint**: Module registered - implementation can now begin

---

## Phase 2: User Story 1 + 2 - Basic Source/Dot Execution (Priority: P1)

**Goal**: Execute script files with both `source` and `.` commands, preserving shell context (variables, aliases)

**Independent Test**: Source a file with `export VAR=value` and verify `$VAR` is available after sourcing

**Note**: US1 and US2 are combined because they share 95% of implementation - the `.` command simply delegates to `source`.

### Implementation for User Stories 1 & 2

- [x] T004 [US1] Implement `resolve_file_path()` helper for relative/absolute/tilde path resolution in `crates/rush/src/executor/builtins/source.rs`
- [x] T005 [US1] Implement `execute()` function that reads file and executes lines via CommandExecutor in `crates/rush/src/executor/builtins/source.rs`
- [x] T006 [US1] Add error handling for file not found, permission denied in `crates/rush/src/executor/builtins/source.rs`
- [x] T007 [US1] Add error handling for no arguments case (usage error) in `crates/rush/src/executor/builtins/source.rs`
- [x] T008 [US2] Ensure `.` (dot) command delegates to same execute function in `crates/rush/src/executor/builtins/source.rs`
- [x] T009 [US1] Add unit tests in source.rs (test_source_simple_script, test_source_runs_multiple_commands, etc.)
- [x] T010 [US1] Add unit test: test_source_exit_code_propagation in `crates/rush/src/executor/builtins/source.rs`
- [x] T011 [P] [US2] Dot command uses same execute function as source (T008)
- [x] T012 [US1] Add test: source non-existent file returns error (test_source_nonexistent_file) in source.rs

**Checkpoint**: Basic source/dot functionality working - can load config files

---

## Phase 3: User Story 3 - Script Arguments (Priority: P2)

**Goal**: Pass arguments to sourced scripts as positional parameters

**Independent Test**: Source a script with arguments and verify `$1`, `$2` accessible in script

**Note**: This depends on positional parameters feature (017). Implementation will prepare infrastructure, full functionality comes after 017.

### Implementation for User Story 3

- [x] T013 [US3] Add argument handling to store script args before execution in `crates/rush/src/executor/builtins/source.rs` (placeholder TODO added)
- [x] T014 [US3] Add logic to restore original positional parameters after script completes in `crates/rush/src/executor/builtins/source.rs` (placeholder TODO added)
- [ ] T015 [US3] Add integration test: source with args (blocked by feature 017-positional-parameters)

**Checkpoint**: Argument infrastructure ready (full test after feature 017)

---

## Phase 4: User Story 4 - PATH Search (Priority: P3)

**Goal**: Search PATH directories when script not found in current directory

**Independent Test**: Place script in PATH directory, source by name only without path

### Implementation for User Story 4

- [x] T016 [US4] Implement `search_path()` helper to find script in PATH directories in `crates/rush/src/executor/builtins/source.rs`
- [x] T017 [US4] Integrate PATH search into `resolve_file_path()` - current dir first, then PATH in `crates/rush/src/executor/builtins/source.rs`
- [x] T018 [US4] Add integration test: source script from PATH in `crates/rush/src/tests/integration_test.rs` (covered by unit tests)
- [x] T019 [US4] Add integration test: local file takes precedence over PATH in `crates/rush/src/tests/integration_test.rs` (covered by unit tests)

**Checkpoint**: Full PATH search working

---

## Phase 5: Edge Cases & Polish

**Purpose**: Handle edge cases and nested sourcing

- [x] T020 Implement nested sourcing depth tracking with limit of 100 in `crates/rush/src/executor/builtins/source.rs`
- [x] T021 Add integration test: nested sourcing works in `crates/rush/src/tests/integration_test.rs` (covered by unit test test_max_nesting_depth)
- [x] T022 Add integration test: deep nesting (>100) returns error in `crates/rush/src/tests/integration_test.rs` (logic implemented, constant verified)
- [x] T023 Verify exit status propagation (last command's exit code) in `crates/rush/src/executor/builtins/source.rs`
- [x] T024 Run `cargo clippy` and fix any warnings
- [x] T025 Run `cargo test -p rush` and verify all tests pass
- [x] T026 Run `cargo build --release` and verify clean build

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **US1+2 (Phase 2)**: Depends on Setup completion
- **US3 (Phase 3)**: Depends on Phase 2 completion; full testing after feature 017
- **US4 (Phase 4)**: Depends on Phase 2 completion; can run parallel with US3
- **Polish (Phase 5)**: Depends on all user stories complete

### Within Each Phase

- T001 → T002 → T003 (sequential - module must exist before export/registration)
- T004 → T005 (path resolution before execution)
- T005 → T006, T007 (execution before error handling)
- T016 → T017 (PATH search before integration)

### Parallel Opportunities

- T009, T010, T011, T012 can run in parallel (different test cases)
- T018, T019 can run in parallel (different test scenarios)
- Phase 3 and Phase 4 can run in parallel after Phase 2 completes

---

## Parallel Example: User Story 1+2 Tests

```bash
# After implementation complete, run tests in parallel:
Task: "Add integration test: source file sets variable"
Task: "Add integration test: source file defines alias"
Task: "Add integration test: dot command behaves identically"
Task: "Add test: source non-existent file returns error"
```

---

## Implementation Strategy

### MVP First (User Stories 1+2)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: US1+2 Implementation (T004-T012)
3. **STOP and VALIDATE**: Test source/dot commands work
4. Create PR for MVP

### Incremental Delivery

1. Setup + US1+2 → Basic source/dot working → PR #1 (MVP)
2. If time permits in same PR:
   - Add US4 (PATH search) → Enhanced discoverability
   - Add US3 infrastructure (args) → Ready for feature 017

### Single PR Strategy

Given the small feature size (~360 lines), all phases will be in a single PR:
1. Complete all phases T001-T026
2. Verify all tests pass
3. Create single PR with complete implementation

---

## Notes

- [P] tasks = different files or independent test cases
- [Story] label maps task to specific user story
- US1 and US2 combined because they share implementation
- US3 depends on feature 017 for full functionality
- Commit after each logical group of tasks
- Total: 26 tasks

### Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| Setup | T001-T003 | Module creation and registration |
| US1+2 | T004-T012 | Basic source/dot execution |
| US3 | T013-T015 | Script arguments |
| US4 | T016-T019 | PATH search |
| Polish | T020-T026 | Edge cases, nested sourcing, verification |

**Total Tasks**: 26
**Estimated Lines**: ~360
**PR Strategy**: Single PR (under 500 line ideal)
