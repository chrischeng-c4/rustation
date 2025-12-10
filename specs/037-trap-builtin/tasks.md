# Tasks: Trap Builtin

**Input**: Design documents from `/specs/037-trap-builtin/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/trap-api.md

**Tests**: Comprehensive test coverage included for all user stories (unit + integration tests)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: rush is a Rust monorepo crate at `crates/rush/`
- Source files: `crates/rush/src/`
- Test files: `crates/rush/tests/`
- Builtin modules: `crates/rush/src/executor/builtins/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create basic module structure for trap builtin

- [x] T001 Create trap.rs module skeleton at crates/rush/src/executor/builtins/trap.rs
- [x] T002 Register trap builtin in crates/rush/src/executor/builtins/mod.rs execute_builtin() function
- [x] T003 Add module declaration for trap in crates/rush/src/executor/builtins/mod.rs (pub mod trap)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core error types, data structures, and signal parsing that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Define TrapError enum variants in crates/rush/src/error.rs (InvalidSignal, UncatchableSignal, DuplicateTrap, EmptyCommand)
- [x] T005 Implement Display trait for TrapError in crates/rush/src/error.rs with user-friendly error messages per contracts/trap-api.md
- [x] T006 [P] Create TrapRegistry struct in crates/rush/src/executor/builtins/trap.rs with HashMap<Signal, String> and Option<String> for EXIT
- [x] T007 [P] Create SignalSpec enum in crates/rush/src/executor/builtins/trap.rs (Name, Number, Pseudo variants)
- [x] T008 Implement SignalSpec::parse() function in crates/rush/src/executor/builtins/trap.rs (case-insensitive, supports SIG prefix, numbers, EXIT)
- [x] T009 Implement SignalSpec::to_signal() function in crates/rush/src/executor/builtins/trap.rs (validates catchable signals, rejects SIGKILL/SIGSTOP)
- [x] T010 Add trap_registry field to CommandExecutor struct in crates/rush/src/executor/execute.rs
- [x] T011 Implement TrapRegistry::new() constructor in crates/rush/src/executor/builtins/trap.rs
- [x] T012 Add trap_registry() and trap_registry_mut() accessor methods to CommandExecutor in crates/rush/src/executor/execute.rs

**Checkpoint**: ✅ **Foundation ready - user story implementation can now begin in parallel**

---

## Phase 3: User Story 1 - Register Cleanup Handlers (Priority: P1) 🎯 MVP

**Goal**: Enable users to register signal handlers for cleanup operations (removing temp files, logging) when receiving interruption signals like SIGINT or SIGTERM

**Independent Test**: Register a trap handler that creates a marker file, send SIGINT to shell, verify marker file was created before termination

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T013 [P] [US1] Unit test for signal parsing (valid names INT/SIGINT/int/2) in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T014 [P] [US1] Unit test for invalid signal rejection (SIGFOO, negative numbers, 999) in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T015 [P] [US1] Unit test for uncatchable signal rejection (SIGKILL/9, SIGSTOP/19) in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T016 [P] [US1] Unit test for trap registration success in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T017 [P] [US1] Unit test for duplicate trap error (FR-006) in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T018 [US1] Integration test for SIGINT handler execution in crates/rush/tests/integration/trap_tests.rs
- [ ] T019 [US1] Integration test for SIGTERM handler execution in crates/rush/tests/integration/trap_tests.rs
- [ ] T020 [US1] Integration test for EXIT pseudo-signal on normal termination in crates/rush/tests/integration/trap_tests.rs

### Implementation for User Story 1

- [ ] T021 [US1] Implement TrapRegistry::register() method in crates/rush/src/executor/builtins/trap.rs (validate signal, check duplicates, insert handler)
- [ ] T022 [US1] Implement TrapRegistry::register_exit() method in crates/rush/src/executor/builtins/trap.rs for EXIT pseudo-signal
- [ ] T023 [US1] Implement trap builtin execute() function skeleton in crates/rush/src/executor/builtins/trap.rs (argument parsing)
- [ ] T024 [US1] Implement trap registration path in execute() function (parse command, parse signals, call register for each signal)
- [ ] T025 [US1] Add multiple signal support in execute() function (trap 'cmd' INT TERM QUIT all-or-nothing registration per FR-013)
- [ ] T026 [US1] Implement signal handler setup using nix::sys::signal::sigaction in crates/rush/src/executor/builtins/trap.rs
- [ ] T027 [US1] Create signal delivery mechanism (atomic flag pattern from research.md) in crates/rush/src/executor/execute.rs
- [ ] T028 [US1] Implement TrapRegistry::get() method in crates/rush/src/executor/builtins/trap.rs to retrieve handler command
- [ ] T029 [US1] Implement CommandExecutor::execute_trap() method in crates/rush/src/executor/execute.rs (execute handler command when signal received)
- [ ] T030 [US1] Implement Drop trait for CommandExecutor in crates/rush/src/executor/execute.rs to execute EXIT handler on shell termination
- [ ] T031 [US1] Add error handling for invalid signals with clear messages per contracts/trap-api.md
- [ ] T032 [US1] Add error handling for duplicate traps with helpful suggestion per contracts/trap-api.md
- [ ] T033 [US1] Add validation for empty command strings (error when registering, OK for clearing)

**Checkpoint**: At this point, User Story 1 should be fully functional - users can register signal handlers and handlers execute on signal delivery

---

## Phase 4: User Story 2 - Inspect Active Traps (Priority: P2)

**Goal**: Enable users to view all currently registered trap handlers for debugging complex scripts

**Independent Test**: Register several traps (INT, TERM, EXIT), run `trap` with no arguments, verify all registered handlers listed with format "trap -- 'command' SIGNAL"

### Tests for User Story 2

- [ ] T034 [P] [US2] Unit test for empty trap listing in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T035 [P] [US2] Unit test for single trap listing format in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T036 [P] [US2] Unit test for multiple trap listing format in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T037 [P] [US2] Unit test for EXIT trap inclusion in listing in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T038 [US2] Integration test for trap listing output format in crates/rush/tests/integration/trap_tests.rs

### Implementation for User Story 2

- [ ] T039 [US2] Implement TrapRegistry::list() method in crates/rush/src/executor/builtins/trap.rs (return Vec<(Signal, &String)> sorted by signal number)
- [ ] T040 [US2] Implement trap listing path in execute() function for no-argument case in crates/rush/src/executor/builtins/trap.rs
- [ ] T041 [US2] Format trap listing output as "trap -- 'command' SIGNAL" per contracts/trap-api.md in crates/rush/src/executor/builtins/trap.rs
- [ ] T042 [US2] Handle empty trap registry case (no output) in execute() function
- [ ] T043 [US2] Ensure EXIT trap is listed last in output in crates/rush/src/executor/builtins/trap.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently - users can register traps and list active traps

---

## Phase 5: User Story 3 - Clear Trap Handlers (Priority: P3)

**Goal**: Enable users to remove trap handlers to restore default signal behavior or disable cleanup that's no longer needed

**Independent Test**: Set a trap for SIGINT, clear it with `trap "" INT`, send SIGINT, verify default behavior (script termination) without handler execution

### Tests for User Story 3

- [ ] T044 [P] [US3] Unit test for clearing existing trap in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T045 [P] [US3] Unit test for clearing non-existent trap (idempotent) in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T046 [P] [US3] Unit test for clearing EXIT trap in crates/rush/src/executor/builtins/trap.rs tests module
- [ ] T047 [US3] Integration test for default signal behavior after clearing in crates/rush/tests/integration/trap_tests.rs
- [ ] T048 [US3] Integration test for re-registering after clearing in crates/rush/tests/integration/trap_tests.rs

### Implementation for User Story 3

- [ ] T049 [US3] Implement TrapRegistry::clear() method in crates/rush/src/executor/builtins/trap.rs (remove handler, restore default)
- [ ] T050 [US3] Implement TrapRegistry::clear_exit() method in crates/rush/src/executor/builtins/trap.rs for EXIT pseudo-signal
- [ ] T051 [US3] Implement trap clearing path in execute() function for empty command "" in crates/rush/src/executor/builtins/trap.rs
- [ ] T052 [US3] Add signal handler restoration to default behavior in clear() method using nix::sys::signal
- [ ] T053 [US3] Ensure clearing succeeds silently for non-existent traps (idempotent per contracts/trap-api.md)

**Checkpoint**: All user stories should now be independently functional - register, list, and clear operations all work

---

## Phase 6: Edge Cases & Validation

**Purpose**: Handle all edge cases from spec.md and contracts/trap-api.md

- [ ] T054 [P] Add validation for real-time signal boundaries (RTMIN, RTMAX, RTMIN+N, RTMAX-N) in SignalSpec::parse()
- [ ] T055 [P] Add error handling for empty signal list (trap 'cmd' with no signals) in execute() function
- [ ] T056 Add handler execution failure handling (command not found, syntax error) with logging in CommandExecutor::execute_trap()
- [ ] T057 Test EXIT handler timing (executes after TERM handler if both present) in crates/rush/tests/integration/trap_tests.rs
- [ ] T058 Test signal handler execution failures don't crash shell in crates/rush/tests/integration/trap_tests.rs

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, help text, and performance validation

- [ ] T059 [P] Add comprehensive help text for trap --help in crates/rush/src/executor/builtins/trap.rs
- [ ] T060 [P] Add doc comments for all public functions in crates/rush/src/executor/builtins/trap.rs
- [ ] T061 [P] Add usage examples to module-level documentation in crates/rush/src/executor/builtins/trap.rs
- [ ] T062 Run cargo clippy --all-targets and fix all warnings
- [ ] T063 Run cargo fmt to ensure consistent formatting
- [ ] T064 Validate performance targets (handler execution <100ms, listing <5s) per SC-002 and SC-005
- [ ] T065 [P] Add quickstart.md examples to integration test suite in crates/rush/tests/integration/trap_tests.rs
- [ ] T066 Final code review and cleanup

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Edge Cases (Phase 6)**: Can start after US1 completes, but benefits from all stories being done
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - No dependencies on US1 (independent listing)
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - No dependencies on US1/US2 (independent clearing)

**All user stories are independently testable and can be developed in parallel after Foundational phase**

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Registry methods before execute() function integration
- Core functionality before error handling
- Unit tests before integration tests
- Story complete before moving to next priority

### Parallel Opportunities

**Phase 1 (Setup)**: All 3 tasks can run sequentially (same files)

**Phase 2 (Foundational)**:
- T006 (TrapRegistry) and T007 (SignalSpec) can run in parallel
- T004 (TrapError) and T005 (Display) sequential (same file)
- T008 and T009 sequential (SignalSpec methods depend on T007)
- T010, T011, T012 sequential (depend on T006)

**Phase 3 (US1) - Tests**:
- T013-T017 (unit tests) all parallelizable
- T018-T020 (integration tests) all parallelizable

**Phase 3 (US1) - Implementation**:
- T021 and T022 can run in parallel (different methods)
- T031-T033 (error handling) all parallelizable

**Phase 4 (US2) - Tests**:
- T034-T037 (unit tests) all parallelizable

**Phase 4 (US2) - Implementation**:
- All tasks sequential (modify same execute() function)

**Phase 5 (US3) - Tests**:
- T044-T046 (unit tests) all parallelizable

**Phase 5 (US3) - Implementation**:
- T049 and T050 can run in parallel (different methods)

**Phase 6 (Edge Cases)**:
- T054 and T055 can run in parallel (different validations)

**Phase 7 (Polish)**:
- T059, T060, T061, T065 all parallelizable (documentation tasks)
- T062, T063 sequential (formatting/linting)

---

## Parallel Example: User Story 1 Tests

```bash
# Launch all unit tests for User Story 1 together:
Task: "Unit test for signal parsing (valid names INT/SIGINT/int/2)"
Task: "Unit test for invalid signal rejection (SIGFOO, negative numbers, 999)"
Task: "Unit test for uncatchable signal rejection (SIGKILL/9, SIGSTOP/19)"
Task: "Unit test for trap registration success"
Task: "Unit test for duplicate trap error (FR-006)"

# Launch all integration tests for User Story 1 together:
Task: "Integration test for SIGINT handler execution"
Task: "Integration test for SIGTERM handler execution"
Task: "Integration test for EXIT pseudo-signal on normal termination"
```

## Parallel Example: User Story 1 Implementation

```bash
# After tests are written and failing, these can run in parallel:
Task: "Implement TrapRegistry::register() method"
Task: "Implement TrapRegistry::register_exit() method"

# Later, all error handling tasks in parallel:
Task: "Add error handling for invalid signals with clear messages"
Task: "Add error handling for duplicate traps with helpful suggestion"
Task: "Add validation for empty command strings"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T012) - CRITICAL blocking phase
3. Complete Phase 3: User Story 1 (T013-T033)
4. **STOP and VALIDATE**: Test User Story 1 independently
   - Register trap handlers for INT, TERM, EXIT
   - Send signals and verify handlers execute
   - Verify error handling for SIGKILL, duplicates, invalid signals
5. Deploy/demo if ready

**Estimated work**: ~800 lines (per plan.md PR #1 estimate)

### Incremental Delivery

1. **Foundation** (T001-T012) → Build foundation
   - Test: Compile successfully, TrapRegistry instantiates
2. **MVP: US1** (T013-T033) → Add core trap functionality
   - Test: `trap 'echo handled' INT` works, Ctrl+C executes handler
   - Deploy/Demo: MVP signal handling complete
3. **US2** (T034-T043) → Add listing capability
   - Test: `trap` displays all registered handlers
   - Deploy/Demo: Debugging support added
4. **US3** (T044-T053) → Add clearing capability
   - Test: `trap "" INT` clears handler, default behavior restored
   - Deploy/Demo: Full trap lifecycle management
5. **Polish** (T054-T066) → Edge cases and documentation
   - Test: All quickstart.md examples work, all edge cases handled
   - Deploy/Demo: Production-ready

Each increment adds value without breaking previous functionality.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T012)
2. Once Foundational is done:
   - **Developer A**: User Story 1 (T013-T033) - 21 tasks
   - **Developer B**: User Story 2 (T034-T043) - 10 tasks (can start in parallel)
   - **Developer C**: User Story 3 (T044-T053) - 10 tasks (can start in parallel)
3. Stories complete and integrate independently
4. Team collaborates on Edge Cases + Polish (T054-T066)

**Note**: While US2 and US3 CAN start in parallel, sequential delivery (P1→P2→P3) provides better incremental value.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD approach)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Use `cargo test` frequently to validate changes
- Use `cargo clippy` to catch potential issues early

### Pull Request Strategy

**CRITICAL: Create separate PRs for each user story to keep changes reviewable.**

**PR Size Limits** (from CLAUDE.md and plan.md):
- **Ideal**: 500 lines
- **Maximum**: 1,500 lines
- **Too large**: 3,000+ lines - must split

**Workflow for Trap Builtin**:

1. **PR #1: Foundation + US1 (Register Cleanup Handlers)** (~800 lines)
   - Phase 1 (Setup): T001-T003
   - Phase 2 (Foundational): T004-T012
   - Phase 3 (US1): T013-T033
   - Merge to main → Delivers MVP signal handling

2. **PR #2: US2 (Inspect Active Traps)** (~300 lines)
   - Phase 4 (US2): T034-T043
   - Merge to main → Delivers debugging capability

3. **PR #3: US3 (Clear Trap Handlers)** (~200 lines)
   - Phase 5 (US3): T044-T053
   - Merge to main → Delivers dynamic trap management

4. **PR #4: Edge Cases + Documentation** (~150 lines)
   - Phase 6 (Edge Cases): T054-T058
   - Phase 7 (Polish): T059-T066
   - Merge to main → Production-ready

**Before Creating PR**:
```bash
git diff --stat main  # Check line count
cargo test            # All tests pass
cargo clippy          # No warnings
cargo fmt --check     # Formatted correctly
```

**Benefits**:
- Each PR delivers standalone value
- Faster review cycles (<1,500 lines each)
- Can merge incrementally (deliver MVP sooner)
- Independent validation per user story

---

## Task Summary

**Total Tasks**: 66

**By Phase**:
- Phase 1 (Setup): 3 tasks
- Phase 2 (Foundational): 9 tasks (BLOCKS all stories)
- Phase 3 (US1 - MVP): 21 tasks (8 tests + 13 implementation)
- Phase 4 (US2): 10 tasks (5 tests + 5 implementation)
- Phase 5 (US3): 10 tasks (5 tests + 5 implementation)
- Phase 6 (Edge Cases): 5 tasks
- Phase 7 (Polish): 8 tasks

**Parallel Opportunities**:
- Foundational phase: 2 parallel groups
- US1 tests: 5 unit tests + 3 integration tests (8 parallel)
- US2 tests: 4 unit tests + 1 integration test (5 parallel)
- US3 tests: 3 unit tests + 2 integration tests (5 parallel)
- Polish: 4 documentation tasks (parallel)

**Independent Test Criteria**:
- **US1**: Register trap → send signal → verify handler executes
- **US2**: Register traps → run `trap` → verify listing format
- **US3**: Clear trap → send signal → verify default behavior

**Suggested MVP**: Phase 1 + Phase 2 + Phase 3 (User Story 1 only) = 33 tasks, ~800 lines
