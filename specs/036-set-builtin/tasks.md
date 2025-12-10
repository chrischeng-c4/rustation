# Tasks: Set Builtin for Shell Options

**Input**: Design documents from `/specs/036-set-builtin/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

**Tests**: Comprehensive unit tests are included as part of this implementation (following existing pattern in rush shell).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- Single Rust project workspace: `crates/rush/src/` at repository root
- Tests: Inline in module files following existing pattern

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic module structure

- [ ] T001 Create ShellOptions struct in crates/rush/src/executor/execute.rs with errexit, xtrace, pipefail fields
- [ ] T002 Add shell_options field to CommandExecutor struct in crates/rush/src/executor/execute.rs
- [ ] T003 Add conditional_depth field to CommandExecutor struct in crates/rush/src/executor/execute.rs
- [ ] T004 Initialize shell_options and conditional_depth in CommandExecutor::new() in crates/rush/src/executor/execute.rs
- [ ] T005 Add pub mod set; declaration in crates/rush/src/executor/builtins/mod.rs (if not already present)
- [ ] T006 Add "set" case to execute_builtin() match in crates/rush/src/executor/builtins/mod.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core option parsing infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T007 Implement parse_option_flags() helper function in crates/rush/src/executor/builtins/set.rs to parse -e, +e, -x, +x syntax
- [ ] T008 Implement parse_long_option() helper function in crates/rush/src/executor/builtins/set.rs to parse -o/+o syntax
- [ ] T009 Extend set::execute() to detect and route option flags vs variable assignments in crates/rush/src/executor/builtins/set.rs
- [ ] T010 Add unit tests for option flag parsing (test_parse_short_form, test_parse_long_form, test_combined_flags) in crates/rush/src/executor/builtins/set.rs
- [ ] T011 Add unit tests for invalid option handling (test_invalid_option, test_unknown_long_option) in crates/rush/src/executor/builtins/set.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Exit on Error (-e) (Priority: P1) 🎯 MVP

**Goal**: Users can enable errexit mode to exit immediately when any command fails

**Independent Test**: Run `set -e` followed by `false` and verify the shell exits with code 1. Run `set -e; if false; then echo "unreached"; fi; echo "reached"` and verify it prints "reached" (conditional exception).

### Implementation for User Story 1

- [ ] T012 [US1] Implement set_errexit() method in crates/rush/src/executor/builtins/set.rs to set/unset errexit flag
- [ ] T013 [US1] Modify execute() in crates/rush/src/executor/execute.rs to check errexit after command execution
- [ ] T014 [US1] Add conditional context check (conditional_depth == 0) before exit in crates/rush/src/executor/execute.rs
- [ ] T015 [US1] Implement increment/decrement of conditional_depth in if/while/until conditions in crates/rush/src/executor/conditional.rs
- [ ] T016 [US1] Implement increment/decrement of conditional_depth in while loop conditions in crates/rush/src/executor/while_loop.rs
- [ ] T017 [US1] Implement increment/decrement of conditional_depth in for loop execution (if needed) in crates/rush/src/executor/for_loop.rs
- [ ] T018 [US1] Add unit tests for errexit basic behavior (test_errexit_exits_on_error, test_errexit_continues_on_success) in crates/rush/src/executor/builtins/set.rs
- [ ] T019 [US1] Add unit tests for errexit with conditionals (test_errexit_if_condition, test_errexit_while_condition, test_errexit_logical_and, test_errexit_logical_or) in crates/rush/src/executor/builtins/set.rs
- [ ] T020 [US1] Add unit tests for errexit with negation (test_errexit_negated_command) in crates/rush/src/executor/builtins/set.rs
- [ ] T021 [US1] Add integration test script for errexit in tests/ directory or as doc comment

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Command Tracing (-x) (Priority: P2)

**Goal**: Users can enable xtrace mode to see each command before it executes with all expansions resolved

**Independent Test**: Run `set -x` followed by `echo $HOME` and verify stderr shows `+ echo /home/user` (with actual HOME value) before stdout shows the result.

### Implementation for User Story 2

- [ ] T022 [US2] Implement set_xtrace() method in crates/rush/src/executor/builtins/set.rs to set/unset xtrace flag
- [ ] T023 [US2] Add xtrace check before command execution in execute() in crates/rush/src/executor/execute.rs
- [ ] T024 [US2] Implement format_command_for_trace() helper to show expanded command in crates/rush/src/executor/execute.rs
- [ ] T025 [US2] Print trace to stderr with `+ ` prefix using eprintln! in crates/rush/src/executor/execute.rs
- [ ] T026 [US2] Add unit tests for xtrace output (test_xtrace_simple_command, test_xtrace_with_variables) in crates/rush/src/executor/builtins/set.rs
- [ ] T027 [US2] Add unit tests for xtrace with pipelines (test_xtrace_pipeline) in crates/rush/src/executor/builtins/set.rs
- [ ] T028 [US2] Add integration test for xtrace output format in tests/ directory or as doc comment

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Pipeline Failure (-o pipefail) (Priority: P2)

**Goal**: Users can enable pipefail mode so pipelines fail if any command fails, not just the last command

**Independent Test**: Run `set -o pipefail` followed by `false | true` and verify the pipeline returns exit code 1 (from false), not 0 (from true).

### Implementation for User Story 3

- [ ] T029 [US3] Implement set_pipefail() method in crates/rush/src/executor/builtins/set.rs to set/unset pipefail flag
- [ ] T030 [US3] Modify execute_pipeline() in crates/rush/src/executor/pipeline.rs to collect all command exit codes
- [ ] T031 [US3] Add pipefail check in pipeline.rs to return first non-zero exit code when enabled
- [ ] T032 [US3] Ensure pipeline returns last exit code when pipefail disabled (default behavior) in crates/rush/src/executor/pipeline.rs
- [ ] T033 [US3] Add access to shell_options from PipelineExecutor (may need to pass reference or add to struct) in crates/rush/src/executor/pipeline.rs
- [ ] T034 [US3] Add unit tests for pipefail behavior (test_pipefail_middle_failure, test_pipefail_all_success, test_pipefail_disabled) in crates/rush/src/executor/builtins/set.rs
- [ ] T035 [US3] Add integration test for pipefail with errexit combination in tests/ directory or as doc comment

**Checkpoint**: All core features (P1, P2) should now be independently functional

---

## Phase 6: User Story 4 - Query Option Status (Priority: P3)

**Goal**: Users can query current option state to verify configuration

**Independent Test**: Run `set -o` without arguments and verify it prints all available options with their current on/off status.

### Implementation for User Story 4

- [ ] T036 [US4] Implement list_options() method in crates/rush/src/executor/builtins/set.rs to return all options with status
- [ ] T037 [US4] Implement format_option_list() to format options as "option on/off" lines in crates/rush/src/executor/builtins/set.rs
- [ ] T038 [US4] Handle `set -o` (no args) to print option list in set::execute() in crates/rush/src/executor/builtins/set.rs
- [ ] T039 [US4] Implement format_option_commands() to generate set commands for recreation in crates/rush/src/executor/builtins/set.rs
- [ ] T040 [US4] Handle `set +o` (no args) to print recreation commands in set::execute() in crates/rush/src/executor/builtins/set.rs
- [ ] T041 [US4] Add unit tests for query output (test_set_o_lists_options, test_set_plus_o_shows_commands) in crates/rush/src/executor/builtins/set.rs

**Checkpoint**: All user stories should now be independently functional

---

## Phase 7: Integration & Edge Cases

**Purpose**: Ensure all options work together and handle edge cases

- [ ] T042 Test combined options `set -ex` (both errexit and xtrace enabled) in crates/rush/src/executor/builtins/set.rs
- [ ] T043 Test mixed enable/disable `set -e +x` in crates/rush/src/executor/builtins/set.rs
- [ ] T044 Test option persistence across commands (set -e persists until set +e) in crates/rush/src/executor/builtins/set.rs
- [ ] T045 Test errexit with pipefail combination (pipeline failure triggers exit) in integration tests
- [ ] T046 Test xtrace with stderr redirection (trace output follows stderr) in integration tests
- [ ] T047 Verify conditional_depth correctly increments/decrements in nested structures in crates/rush/src/executor/builtins/set.rs
- [ ] T048 Run full test suite with `cargo test` to verify no regressions

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements, documentation, and validation

- [ ] T049 [P] Add documentation comments to ShellOptions struct in crates/rush/src/executor/execute.rs
- [ ] T050 [P] Add documentation comments to public methods in set.rs (parse_option_flags, set_errexit, etc.)
- [ ] T051 [P] Verify all clippy warnings addressed with `cargo clippy`
- [ ] T052 [P] Verify formatting with `cargo fmt --check`
- [ ] T053 Manual testing in interactive shell - test all user scenarios from spec.md
- [ ] T054 Performance validation - verify option checks add <1ms overhead per command
- [ ] T055 Update specs/features.json to mark feature 036 as "complete"
- [ ] T056 Bump version to v0.36.0 in Cargo.toml
- [ ] T057 Update CLAUDE.md with set builtin usage examples (optional)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3, 4, 5, 6)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if desired)
  - Or sequentially in priority order (P1 → P2 → P2 → P3)
- **Integration (Phase 7)**: Depends on all desired user stories being complete
- **Polish (Phase 8)**: Depends on integration being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Independent of US1
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - Independent of US1/US2 but needs PipelineExecutor access
- **User Story 4 (P3)**: Can start after Foundational (Phase 2) - Independent of US1/US2/US3

### Within Each User Story

- Core implementation before tests (though tests can be written first in TDD style)
- All tests for a story can run in parallel after implementation
- Conditional depth tracking (US1 T015-T017) should be done together

### Parallel Opportunities

- All Setup tasks (T001-T006) can run in sequence (they modify same/related files)
- All Foundational tasks (T007-T011) can run in sequence (same file)
- Once Foundational phase completes:
  - US1 implementation (T012-T021) can be done as a unit
  - US2 implementation (T022-T028) can be done independently after US1
  - US3 implementation (T029-T035) can be done independently after US1/US2
  - US4 implementation (T036-T041) can be done independently after US1/US2/US3
- All Polish tasks marked [P] can run in parallel

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T011) - CRITICAL
3. Complete Phase 3: User Story 1 (T012-T021)
4. Complete Phase 7: Integration (T042-T048) - Limited to errexit testing
5. Complete Phase 8: Polish (T049-T057)
6. **STOP and VALIDATE**: Test errexit mode independently
7. Deploy/merge if ready (MVP with just -e option!)

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 (errexit) → Test independently → Can merge to main (MVP!)
3. Add User Story 2 (xtrace) → Test independently → Can merge to main
4. Add User Story 3 (pipefail) → Test independently → Can merge to main
5. Add User Story 4 (query) → Test independently → Can merge to main
6. Each story adds value without breaking previous stories

### All Features in Single PR (Recommended)

Based on plan.md deployment strategy, implement all user stories together in 5 sequential PRs:

**PR #1: Foundation** (T001-T011)
- Sets up ShellOptions infrastructure
- Adds option parsing framework
- No user-visible changes yet
- ~100 lines

**PR #2: Errexit** (T012-T021)
- Implements User Story 1 (P1)
- First user-facing feature
- ~100 lines

**PR #3: Xtrace** (T022-T028)
- Implements User Story 2 (P2)
- ~80 lines

**PR #4: Pipefail** (T029-T035)
- Implements User Story 3 (P2)
- ~70 lines

**PR #5: Query + Polish** (T036-T057)
- Implements User Story 4 (P3)
- Integration testing
- Polish and documentation
- ~100 lines

**Total**: ~450 lines across 5 PRs, all within 500-line ideal limit per PR ✅

---

## Notes

- [P] tasks = different files or independent, can run in parallel
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Follow existing patterns from other builtins (cd, alias, export)
- All shell option checks must be O(1) boolean flag access
- Conditional depth tracking is critical for errexit correctness
- Xtrace output goes to stderr, not stdout

### Pull Request Strategy

Follow the 5 PR strategy from plan.md:

1. **PR #1**: Foundation (T001-T011) → main
2. **PR #2**: Errexit (T012-T021) → main
3. **PR #3**: Xtrace (T022-T028) → main  
4. **PR #4**: Pipefail (T029-T035) → main
5. **PR #5**: Query + Polish (T036-T057) → main

Each PR is independently reviewable and testable.

**Before Creating Each PR**:
- Check line count: `git diff --stat main`
- Expected: Each PR ≤ 500 lines ✅
- Commit message: `feat(036): [description]` format
