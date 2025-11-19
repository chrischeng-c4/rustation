# Tasks: Pipe Operator Support

**Input**: Design documents from `/specs/004-pipes/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Contract tests are explicitly defined in the specification and included in this task list.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `crates/rush/src/`, `crates/rush/tests/` at repository root
- Paths below follow rush shell monorepo structure

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add development dependencies and prepare test infrastructure

- [x] T001 Add criterion benchmark dependency to crates/rush/Cargo.toml dev-dependencies
- [x] T002 [P] Add tempfile test dependency to crates/rush/Cargo.toml dev-dependencies
- [x] T003 [P] Create benches directory at crates/rush/benches/ for performance tests
- [x] T004 [P] Create contract test directory at crates/rush/tests/contract/

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures and parser modifications that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Data Structures

- [x] T005 [P] Define Pipeline struct in crates/rush/src/executor/mod.rs with segments and raw_input fields
- [x] T006 [P] Define PipelineSegment struct in crates/rush/src/executor/mod.rs with program, args, and index fields
- [x] T007 [P] Implement Pipeline::new(), len(), is_empty(), and validate() methods in crates/rush/src/executor/mod.rs
- [x] T008 [P] Implement PipelineSegment::new(), validate(), is_first(), and is_last() methods in crates/rush/src/executor/mod.rs
- [x] T009 Export Pipeline and PipelineSegment types from crates/rush/src/executor/mod.rs module

### Parser Extensions

- [x] T010 Define Token enum (Word, Pipe) in crates/rush/src/executor/parser.rs
- [x] T011 Implement tokenize_with_pipes() function in crates/rush/src/executor/parser.rs to detect pipes outside quotes
- [x] T012 Implement split_into_segments() function in crates/rush/src/executor/parser.rs to split tokens at pipe boundaries
- [x] T013 Implement parse_pipeline() public function in crates/rush/src/executor/parser.rs that returns Pipeline
- [x] T014 Add validation for malformed pipelines (empty before/after pipe, double pipes) in crates/rush/src/executor/parser.rs

### Parser Unit Tests

- [x] T015 [P] Create parser unit test file at crates/rush/tests/unit/pipe_parser_tests.rs
- [x] T016 [P] Write test_parse_single_command in crates/rush/tests/unit/pipe_parser_tests.rs
- [x] T017 [P] Write test_parse_two_command_pipeline in crates/rush/tests/unit/pipe_parser_tests.rs
- [x] T018 [P] Write test_parse_multi_command_pipeline in crates/rush/tests/unit/pipe_parser_tests.rs
- [x] T019 [P] Write test_parse_pipe_in_quotes in crates/rush/tests/unit/pipe_parser_tests.rs (verify pipe inside quotes is literal)
- [x] T020 [P] Write test_parse_empty_before_pipe in crates/rush/tests/unit/pipe_parser_tests.rs (verify error)
- [x] T021 [P] Write test_parse_empty_after_pipe in crates/rush/tests/unit/pipe_parser_tests.rs (verify error)
- [x] T022 [P] Write test_parse_double_pipe in crates/rush/tests/unit/pipe_parser_tests.rs (verify error)
- [x] T023 Run cargo test to verify all parser tests pass

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Basic Two-Command Pipeline (Priority: P1) 🎯 MVP

**Goal**: Enable users to connect two commands using the pipe operator where stdout of first becomes stdin of second

**Independent Test**: Run `echo "hello world" | grep hello` and verify output shows "hello world"

### Contract Tests for User Story 1

- [ ] T024 [P] [US1] Copy specs/004-pipes/contracts/us1_basic_two_command_pipeline.rs to crates/rush/tests/contract/
- [ ] T025 [P] [US1] Update contract test imports in crates/rush/tests/contract/us1_basic_two_command_pipeline.rs to match rush module structure
- [ ] T026 [US1] Run cargo test us1_ to verify contract tests compile (expect failures before implementation)

### Core Pipeline Executor

- [ ] T027 [US1] Create crates/rush/src/executor/pipeline.rs file with PipelineExecutor struct
- [ ] T028 [US1] Implement PipelineExecutor::new() and execute() in crates/rush/src/executor/pipeline.rs
- [ ] T029 [US1] Implement execute_single() private method in crates/rush/src/executor/pipeline.rs for single-command optimization
- [ ] T030 [US1] Define PipelineExecution internal struct in crates/rush/src/executor/pipeline.rs with children and pipeline fields
- [ ] T031 [US1] Implement PipelineExecution::spawn() in crates/rush/src/executor/pipeline.rs for two-command pipelines
  - First command: stdin from terminal, stdout to Stdio::piped()
  - Second command: stdin from first command's stdout, stdout to terminal
  - Both commands: stderr to terminal
- [ ] T032 [US1] Implement PipelineExecution::wait_all() in crates/rush/src/executor/pipeline.rs to wait for all processes and return last exit code
- [ ] T033 [US1] Add tracing debug/info logs in crates/rush/src/executor/pipeline.rs for spawn and completion events
- [ ] T034 [US1] Export PipelineExecutor from crates/rush/src/executor/mod.rs module

### Integration with REPL

- [ ] T035 [US1] Refactor crates/rush/src/executor/execute.rs to use PipelineExecutor instead of direct Command execution
- [x] T036 [US1] Update crates/rush/src/repl/mod.rs to call parse_pipeline() and PipelineExecutor::execute()
- [x] T037 [US1] Verify backward compatibility by running cargo test (all existing tests should still pass)

### Integration Tests for User Story 1

- [x] T038 [P] [US1] Create integration test file at crates/rush/tests/integration/pipe_tests.rs
- [x] T039 [P] [US1] Write test_ls_pipe_grep in crates/rush/tests/integration/pipe_tests.rs
- [x] T040 [P] [US1] Write test_echo_pipe_wc in crates/rush/tests/integration/pipe_tests.rs
- [x] T041 [P] [US1] Write test_cat_pipe_head in crates/rush/tests/integration/pipe_tests.rs
- [x] T042 [P] [US1] Write test_empty_output_pipe in crates/rush/tests/integration/pipe_tests.rs
- [x] T043 [P] [US1] Write test_date_pipe_cat in crates/rush/tests/integration/pipe_tests.rs
- [x] T044 [US1] Run cargo test us1_ to verify all US1 contract tests pass
- [x] T045 [US1] Run cargo test pipe_tests to verify all US1 integration tests pass

### Documentation Updates

- [ ] T046 [P] [US1] Update crates/rush/KNOWN_ISSUES.md to move pipes from "What Doesn't Work Yet" to "What Works"
- [ ] T047 [P] [US1] Add rustdoc comments to Pipeline, PipelineSegment, PipelineExecutor in crates/rush/src/executor/ files

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Users can run basic two-command pipelines like `ls | grep txt`.

---

## Phase 4: User Story 2 - Multi-Command Pipeline Chain (Priority: P2)

**Goal**: Enable users to chain three or more commands together with proper I/O chaining

**Independent Test**: Run `echo -e "apple\nbanana\napple" | sort | uniq` and verify deduplicated sorted output

### Contract Tests for User Story 2

- [x] T048 [P] [US2] Copy specs/004-pipes/contracts/us2_multi_command_pipeline.rs to crates/rush/tests/contract/
- [x] T049 [P] [US2] Update contract test imports in crates/rush/tests/contract/us2_multi_command_pipeline.rs
- [x] T050 [US2] Run cargo test us2_ to verify contract tests compile (expect failures before implementation)

### Extend Pipeline Executor

- [x] T051 [US2] Extend PipelineExecution::spawn() in crates/rush/src/executor/pipeline.rs to handle 3+ commands
  - First command: stdin from terminal, stdout to pipe
  - Middle commands: stdin from previous stdout, stdout to pipe
  - Last command: stdin from previous stdout, stdout to terminal
  - All commands: stderr to terminal
- [x] T052 [US2] Update execute_pipeline() in crates/rush/src/executor/pipeline.rs to support multi-command chains
- [x] T053 [US2] Add logging for each segment in multi-command pipelines in crates/rush/src/executor/pipeline.rs

### Integration Tests for User Story 2

- [x] T054 [P] [US2] Write test_cat_pipe_grep_pipe_wc in crates/rush/tests/integration/pipe_tests.rs
- [x] T055 [P] [US2] Write test_ls_pipe_grep_pipe_head in crates/rush/tests/integration/pipe_tests.rs
- [x] T056 [P] [US2] Write test_echo_pipe_sort_pipe_tail in crates/rush/tests/integration/pipe_tests.rs
- [x] T057 [P] [US2] Write test_five_command_pipeline in crates/rush/tests/integration/pipe_tests.rs
- [x] T058 [P] [US2] Write test_long_pipeline_20_commands in crates/rush/tests/integration/pipe_tests.rs (stress test)
- [x] T059 [US2] Run cargo test us2_ to verify all US2 contract tests pass
- [x] T060 [US2] Run cargo test pipe_tests to verify all US2 integration tests pass

### Performance Validation

- [x] T061 [P] [US2] Create benchmark file at crates/rush/benches/pipeline_bench.rs
- [x] T062 [P] [US2] Write benchmark_parse_pipeline in crates/rush/benches/pipeline_bench.rs
- [x] T063 [P] [US2] Write benchmark_execute_two_command_pipeline in crates/rush/benches/pipeline_bench.rs
- [x] T064 [P] [US2] Write benchmark_execute_five_command_pipeline in crates/rush/benches/pipeline_bench.rs
- [x] T065 [P] [US2] Write benchmark_concurrent_execution in crates/rush/benches/pipeline_bench.rs to verify concurrent vs sequential timing
- [x] T066 [US2] Run cargo bench and verify parse time <1ms and execution overhead <5ms (constitution requirements)

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Users can run complex multi-command pipelines like `cat file | grep error | wc -l`.

---

## Phase 5: User Story 3 - Pipeline Error Handling (Priority: P3)

**Goal**: Provide clear error messages when commands in a pipeline fail, matching standard shell error semantics

**Independent Test**: Run `ls /nonexistent | grep foo` and verify appropriate error message appears

### Contract Tests for User Story 3

- [x] T067 [P] [US3] Copy specs/004-pipes/contracts/us3_error_handling.rs to crates/rush/tests/contract/
- [x] T068 [P] [US3] Update contract test imports in crates/rush/tests/contract/us3_error_handling.rs
- [x] T069 [US3] Run cargo test us3_ to verify contract tests compile (expect failures before implementation)

### Enhanced Error Handling

- [x] T070 [US3] Improve error messages in PipelineExecution::spawn() in crates/rush/src/executor/pipeline.rs to indicate which command failed
- [x] T071 [US3] Add command position (index) to spawn failure errors in crates/rush/src/executor/pipeline.rs
- [x] T072 [US3] Enhance wait_all() error messages in crates/rush/src/executor/pipeline.rs to show which command failed during execution
- [x] T073 [US3] Add validation for command-not-found vs permission-denied errors in crates/rush/src/executor/pipeline.rs

### Integration Tests for User Story 3

- [x] T074 [P] [US3] Write test_first_command_fails in crates/rush/tests/integration/pipe_tests.rs
- [x] T075 [P] [US3] Write test_second_command_fails in crates/rush/tests/integration/pipe_tests.rs
- [x] T076 [P] [US3] Write test_middle_command_fails in crates/rush/tests/integration/pipe_tests.rs
- [x] T077 [P] [US3] Write test_grep_no_matches in crates/rush/tests/integration/pipe_tests.rs (verify exit code 1, no error message)
- [x] T078 [P] [US3] Write test_broken_pipe in crates/rush/tests/integration/pipe_tests.rs (yes | head -1)
- [x] T079 [US3] Run cargo test us3_ to verify all US3 contract tests pass
- [x] T080 [US3] Run cargo test pipe_tests to verify all US3 integration tests pass

**Checkpoint**: Error handling is now comprehensive. Users receive clear, actionable error messages when pipelines fail.

---

## Phase 6: User Story 4 - Pipeline Exit Code Handling (Priority: P4)

**Goal**: Pipeline exit code follows Unix semantics (last command's exit code)

**Independent Test**: Run `echo test | false; echo $?` and verify exit code is 1 (from false command)

### Contract Tests for User Story 4

- [x] T081 [P] [US4] Copy specs/004-pipes/contracts/us4_exit_codes.rs to crates/rush/tests/contract/
- [x] T082 [P] [US4] Update contract test imports in crates/rush/tests/contract/us4_exit_codes.rs
- [x] T083 [US4] Run cargo test us4_ to verify contract tests compile (expect pass if US1-US3 implemented correctly)

### Exit Code Validation

- [x] T084 [US4] Verify wait_all() in crates/rush/src/executor/pipeline.rs returns last command's exit code
- [x] T085 [US4] Add integration test test_exit_code_propagation in crates/rush/tests/integration/pipe_tests.rs
- [x] T086 [US4] Run cargo test us4_ to verify all US4 contract tests pass
- [x] T087 [US4] Run cargo test to verify all pipeline tests pass end-to-end

**Checkpoint**: All user stories (US1-US4) are now complete and independently functional. Pipeline exit codes follow Unix conventions.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, final validation, and cross-story improvements

### Documentation

- [x] T088 [P] Update crates/rush/CLI.md to add "Pipe Operator" section with examples
- [x] T089 [P] Update crates/rush/README.md to add pipe operator to feature list
- [x] T090 [P] Update crates/rush/TEST_COVERAGE.md to document pipe test statistics
- [x] T091 [P] Verify specs/004-pipes/quickstart.md examples work correctly by running each command

### Final Validation

- [x] T092 Run cargo clippy --all-targets --all-features and address all warnings
- [x] T093 Run cargo fmt --check to verify code formatting
- [x] T094 Run cargo test --all to verify all tests pass (159+ tests expected)
- [x] T095 Run cargo bench to generate performance baseline report
- [x] T096 Verify all contract tests pass: cargo test --test us1_basic_two_command_pipeline --test us2_multi_command_pipeline --test us3_error_handling --test us4_exit_codes
- [x] T097 Manual smoke test: Run rush and test `ls | grep rs`, `cat Cargo.toml | head -10`, `echo test | grep test`

### Performance and Quality Gates

- [x] T098 Verify parse time <1ms (check benchmark results from T066)
- [x] T099 Verify execution overhead <5ms (check benchmark results from T066)
- [x] T100 Verify zero regressions in existing tests (all 159+ tests pass)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3 → P4)
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Extends US1 executor but independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Enhances error handling, independently testable
- **User Story 4 (P4)**: Can start after Foundational (Phase 2) - Validates exit codes, independently testable

**Note**: US2, US3, and US4 can technically be implemented in parallel after US1 is complete, but sequentially is recommended for this feature size.

### Within Each User Story

- Contract tests → Implementation → Integration tests → Run tests to validate
- Models/data structures before executor logic
- Core implementation before validation
- Story complete before moving to next priority

### Parallel Opportunities

- **Phase 1**: All tasks (T001-T004) marked [P] can run in parallel
- **Phase 2 Data Structures**: T005-T008 can run in parallel
- **Phase 2 Parser Tests**: T016-T022 can run in parallel (after T015 creates file)
- **Within US1**: Contract tests (T024-T025), integration tests (T039-T043), docs (T046-T047) can run in parallel
- **Within US2**: Contract tests (T048-T049), integration tests (T054-T058), benchmarks (T061-T065) can run in parallel
- **Within US3**: Contract tests (T067-T068), integration tests (T074-T078) can run in parallel
- **Within US4**: Contract tests (T081-T082) can run in parallel
- **Phase 7**: Documentation tasks (T088-T091) can run in parallel

---

## Parallel Example: User Story 1

```bash
# After T023 (foundation complete), launch US1 tasks in parallel:

# Parallel batch 1: Contract tests
Task T024: Copy contract test file
Task T025: Update imports

# Sequential: T026 (verify compilation)

# Sequential: T027-T034 (core implementation - dependent chain)

# Sequential: T035-T037 (REPL integration)

# Parallel batch 2: Integration tests (after implementation)
Task T038: Create integration test file
Task T039: Write test_ls_pipe_grep
Task T040: Write test_echo_pipe_wc
Task T041: Write test_cat_pipe_head
Task T042: Write test_empty_output_pipe
Task T043: Write test_date_pipe_cat

# Sequential: T044-T045 (run tests)

# Parallel batch 3: Documentation
Task T046: Update KNOWN_ISSUES.md
Task T047: Add rustdoc comments
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T023) - **CRITICAL - blocks all stories**
3. Complete Phase 3: User Story 1 (T024-T047)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Create PR #1 (Foundation + US1) and merge
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational (T001-T023) → Foundation ready
2. Add User Story 1 (T024-T047) → Test independently → **Deploy/Demo (MVP!)**
3. Add User Story 2 (T048-T066) → Test independently → Deploy/Demo
4. Add User Story 3 (T067-T080) → Test independently → Deploy/Demo
5. Add User Story 4 (T081-T087) → Test independently → Deploy/Demo
6. Polish (T088-T100) → Final validation → Deploy
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T023)
2. Once Foundational is done:
   - Developer A: User Story 1 (T024-T047)
   - After US1 complete:
     - Developer A: User Story 2 (T048-T066)
     - Developer B: User Story 3 (T067-T080)
     - Developer C: User Story 4 (T081-T087)
3. Stories complete and integrate independently

**Recommended**: Sequential implementation (US1 → US2 → US3 → US4) for this feature due to natural progression and manageable scope.

---

## Pull Request Strategy

**CRITICAL: Create separate PRs for each logical increment to keep changes reviewable.**

**PR Size Limits** (from CLAUDE.md):
- **Ideal**: 500 lines of changes
- **Maximum**: 1,500 lines
- **Too large**: 3,000+ lines - must split

### Recommended PR Sequence

**PR #1: Foundation + Setup** (~300 lines)
- Phase 1: Setup (T001-T004)
- Phase 2: Foundational (T005-T023)
- Deliverables: Data structures, parser extensions, parser unit tests
- **Validation**: Run cargo test (parser tests pass, no execution yet)

**PR #2: User Story 1 (P1) - Basic Two-Command Pipeline** (~800 lines)
- Phase 3: User Story 1 (T024-T047)
- Deliverables: PipelineExecutor, REPL integration, contract tests, integration tests, docs
- **Validation**: Run cargo test us1_ and cargo test pipe_tests
- **Delivers MVP**: Users can run `ls | grep foo`

**PR #3: User Story 2 (P2) - Multi-Command Chains** (~400 lines)
- Phase 4: User Story 2 (T048-T066)
- Deliverables: Multi-command support, benchmarks, integration tests
- **Validation**: Run cargo test us2_ and cargo bench
- **Delivers**: Complex pipelines like `cat | grep | wc`

**PR #4: User Stories 3 & 4 (P3-P4) - Error Handling + Exit Codes** (~500 lines)
- Phase 5: User Story 3 (T067-T080)
- Phase 6: User Story 4 (T081-T087)
- Phase 7: Polish (T088-T100)
- Deliverables: Enhanced errors, exit code validation, documentation, final validation
- **Validation**: Run cargo test --all and cargo clippy
- **Delivers**: Production-ready pipe operator

### Before Creating Each PR

```bash
# Check line count
git diff --stat main

# If >1,500 lines, split further by task groups
```

---

## Notes

- [P] tasks = different files, no dependencies within phase
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- **Total tasks**: 100 tasks organized into 7 phases
- **Estimated time**: 10-14 hours (from plan.md)
- **Contract tests**: 17+ tests across 4 files validating all acceptance scenarios
- **Integration tests**: 20+ tests for real command execution
- **Unit tests**: 8+ parser tests for edge cases

### Success Metrics

- All 15 functional requirements (FR-001 to FR-015) implemented
- All 4 user stories (US1-US4) tested and validated
- All 17 acceptance scenarios pass contract tests
- Parse time <1ms, execution overhead <5ms (constitution requirements)
- Zero clippy warnings, all tests passing
- Backward compatible (existing rush features unaffected)

**Ready to implement!** 🚀
