# Tasks: Output Redirection Operators

**Input**: Design documents from `/specs/005-output-redirection/`
**Prerequisites**: plan.md (complete), spec.md (complete), research.md (complete), data-model.md (complete), contracts/ (complete)

**Tests**: Tests are included based on specification requirements and constitution testing philosophy.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Foundation + Infrastructure)

**Purpose**: Data structures and parser infrastructure that enables all user stories

- [ ] T001 [P] Create RedirectionType enum in crates/rush/src/executor/mod.rs
- [ ] T002 [P] Create Redirection struct in crates/rush/src/executor/mod.rs
- [ ] T003 [P] Extend Token enum with RedirectOut, RedirectAppend, RedirectIn variants in crates/rush/src/executor/parser.rs
- [ ] T004 Extend PipelineSegment struct with redirections field in crates/rush/src/executor/mod.rs
- [ ] T005 Add Redirection variant to RushError enum in crates/rush/src/error.rs
- [ ] T006 [P] Add validation method to Redirection struct in crates/rush/src/executor/mod.rs
- [ ] T007 [P] Update PipelineSegment::validate to validate redirections in crates/rush/src/executor/mod.rs

**Checkpoint**: Data structures ready - parsing implementation can now begin

---

## Phase 2: Foundational (Parser Infrastructure - BLOCKING)

**Purpose**: Core tokenization and parsing logic that MUST be complete before ANY user story execution

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T008 Implement tokenize_with_redirections helper that detects >, >>, < outside quotes in crates/rush/src/executor/parser.rs
- [ ] T009 Update parse_pipeline to use tokenize_with_redirections in crates/rush/src/executor/parser.rs
- [ ] T010 Implement parse_segment_redirections to extract redirections from token stream in crates/rush/src/executor/parser.rs
- [ ] T011 Update split_into_segments to separate redirections from command args in crates/rush/src/executor/parser.rs
- [ ] T012 Add syntax validation for redirection operator followed by file path in crates/rush/src/executor/parser.rs
- [ ] T013 [P] Add unit tests for tokenizing > operator in crates/rush/tests/unit/redirection_parser_tests.rs
- [ ] T014 [P] Add unit tests for tokenizing >> operator (not two > tokens) in crates/rush/tests/unit/redirection_parser_tests.rs
- [ ] T015 [P] Add unit tests for tokenizing < operator in crates/rush/src/executor/parser.rs
- [ ] T016 [P] Add unit tests for operators inside quotes become Word tokens in crates/rush/tests/unit/redirection_parser_tests.rs
- [ ] T017 [P] Add unit tests for whitespace handling around operators in crates/rush/tests/unit/redirection_parser_tests.rs
- [ ] T018 [P] Add unit tests for Redirection struct validation in crates/rush/tests/unit/redirection_model_tests.rs
- [ ] T019 [P] Add unit tests for PipelineSegment with redirections in crates/rush/tests/unit/redirection_model_tests.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Basic Output Redirection (>) (Priority: P1) 🎯 MVP

**Goal**: Users can redirect command stdout to files using > operator, overwriting file contents

**Independent Test**: Run `echo "test" > output.txt`, verify file contains "test", run `echo "new" > output.txt`, verify file now contains "new" (overwritten)

### Implementation for User Story 1

- [ ] T020 [US1] Implement apply_output_redirection for RedirectionType::Output in crates/rush/src/executor/pipeline.rs
- [ ] T021 [US1] Use File::create for truncate mode in apply_output_redirection in crates/rush/src/executor/pipeline.rs
- [ ] T022 [US1] Convert File to Stdio using Stdio::from in apply_output_redirection in crates/rush/src/executor/pipeline.rs
- [ ] T023 [US1] Update PipelineExecutor::spawn to apply stdout redirections before command.spawn in crates/rush/src/executor/pipeline.rs
- [ ] T024 [US1] Implement error handling for file creation failures with ErrorKind matching in crates/rush/src/executor/pipeline.rs
- [ ] T025 [US1] Map ErrorKind::NotFound to "file not found" message in crates/rush/src/executor/pipeline.rs
- [ ] T026 [US1] Map ErrorKind::PermissionDenied to "permission denied" message in crates/rush/src/executor/pipeline.rs
- [ ] T027 [US1] Map ErrorKind::IsADirectory to "is a directory" message in crates/rush/src/executor/pipeline.rs
- [ ] T028 [P] [US1] Integration test for echo "hello" > file.txt in crates/rush/tests/integration/redirection_tests.rs
- [ ] T029 [P] [US1] Integration test for overwriting existing file in crates/rush/tests/integration/redirection_tests.rs
- [ ] T030 [P] [US1] Integration test for ls -la > listing.txt in crates/rush/tests/integration/redirection_tests.rs
- [ ] T031 [P] [US1] Integration test for permission denied error in crates/rush/tests/integration/redirection_tests.rs
- [ ] T032 [P] [US1] Integration test for is a directory error in crates/rush/tests/integration/redirection_tests.rs
- [ ] T033 [P] [US1] Contract test validating FR-001 through FR-006 in crates/rush/tests/contract/redirection_spec_validation.rs

**Checkpoint**: At this point, User Story 1 should be fully functional - users can use `>` to redirect output

---

## Phase 4: User Story 2 - Append Output Redirection (>>) (Priority: P1)

**Goal**: Users can append command output to files using >> operator without losing previous content

**Independent Test**: Run `echo "first" > test.txt`, then `echo "second" >> test.txt`, verify file contains both "first" and "second" on separate lines

### Implementation for User Story 2

- [ ] T034 [US2] Implement apply_append_redirection for RedirectionType::Append in crates/rush/src/executor/pipeline.rs
- [ ] T035 [US2] Use OpenOptions with create(true).append(true) for append mode in crates/rush/src/executor/pipeline.rs
- [ ] T036 [US2] Update PipelineExecutor::spawn to handle both Output and Append types in crates/rush/src/executor/pipeline.rs
- [ ] T037 [P] [US2] Integration test for echo "line1" then echo "line2" >> appends correctly in crates/rush/tests/integration/redirection_tests.rs
- [ ] T038 [P] [US2] Integration test for >> creates file if doesn't exist in crates/rush/tests/integration/redirection_tests.rs
- [ ] T039 [P] [US2] Integration test for multiple append commands in sequence in crates/rush/tests/integration/redirection_tests.rs
- [ ] T040 [P] [US2] Integration test for appending to large existing file (no full rewrite) in crates/rush/tests/integration/redirection_tests.rs
- [ ] T041 [P] [US2] Contract test validating FR-007 through FR-011 in crates/rush/tests/contract/redirection_spec_validation.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work - users have full output redirection

---

## Phase 5: User Story 3 - Input Redirection (<) (Priority: P2)

**Goal**: Users can redirect command stdin from files using < operator for batch data processing

**Independent Test**: Create file with `echo -e "line1\nline2" > input.txt`, run `wc -l < input.txt`, verify it outputs "2"

### Implementation for User Story 3

- [ ] T042 [US3] Implement apply_input_redirection for RedirectionType::Input in crates/rush/src/executor/pipeline.rs
- [ ] T043 [US3] Use File::open for read mode in apply_input_redirection in crates/rush/src/executor/pipeline.rs
- [ ] T044 [US3] Update PipelineExecutor::spawn to apply stdin redirections before command.spawn in crates/rush/src/executor/pipeline.rs
- [ ] T045 [US3] Implement error handling for input file not found in crates/rush/src/executor/pipeline.rs
- [ ] T046 [P] [US3] Integration test for cat < input.txt displays file contents in crates/rush/tests/integration/redirection_tests.rs
- [ ] T047 [P] [US3] Integration test for sort < numbers.txt sorts correctly in crates/rush/tests/integration/redirection_tests.rs
- [ ] T048 [P] [US3] Integration test for grep "pattern" < data.txt searches file in crates/rush/tests/integration/redirection_tests.rs
- [ ] T049 [P] [US3] Integration test for file not found error for < operator in crates/rush/tests/integration/redirection_tests.rs
- [ ] T050 [P] [US3] Integration test for < operator position independent (before or after command) in crates/rush/tests/integration/redirection_tests.rs
- [ ] T051 [P] [US3] Contract test validating FR-012 through FR-016 in crates/rush/tests/contract/redirection_spec_validation.rs

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently

---

## Phase 6: User Story 4 - Combined Redirections (Priority: P2)

**Goal**: Users can combine input and output redirections in single commands for complex workflows

**Independent Test**: Create `echo -e "c\nb\na" > unsorted.txt`, run `sort < unsorted.txt > sorted.txt`, verify sorted.txt contains "a\nb\nc"

### Implementation for User Story 4

- [ ] T052 [US4] Implement redirection resolution to find last stdout redirect (Output or Append) in crates/rush/src/executor/pipeline.rs
- [ ] T053 [US4] Implement redirection resolution to find last stdin redirect (Input) in crates/rush/src/executor/pipeline.rs
- [ ] T054 [US4] Update PipelineExecutor::spawn to apply both stdin and stdout redirections in crates/rush/src/executor/pipeline.rs
- [ ] T055 [US4] Implement last-wins logic for multiple output redirections in crates/rush/src/executor/pipeline.rs
- [ ] T056 [P] [US4] Integration test for sort < input.txt > output.txt in crates/rush/tests/integration/combined_tests.rs
- [ ] T057 [P] [US4] Integration test for command < in.txt >> out.txt (input + append) in crates/rush/tests/integration/combined_tests.rs
- [ ] T058 [P] [US4] Integration test for cat < file1.txt | grep "x" > file2.txt (with pipes) in crates/rush/tests/integration/combined_tests.rs
- [ ] T059 [P] [US4] Integration test for echo "data" | tee output.txt > final.txt in crates/rush/tests/integration/combined_tests.rs
- [ ] T060 [P] [US4] Integration test for multiple output redirections (last wins) in crates/rush/tests/integration/combined_tests.rs
- [ ] T061 [P] [US4] Contract test validating FR-017 through FR-020 in crates/rush/tests/contract/redirection_spec_validation.rs

**Checkpoint**: All core user stories complete - full redirection functionality working

---

## Phase 7: User Story 5 - Error Handling and Edge Cases (Priority: P3)

**Goal**: Robust error handling prevents data loss and provides clear error messages in edge cases

**Independent Test**: Run error scenarios - `echo "x" > /etc` (directory error), `false > out.txt` (command failure), `cat < file.txt > file.txt` (same file warning)

### Implementation for User Story 5

- [ ] T062 [US5] Add comprehensive error context to all redirection errors with file paths in crates/rush/src/executor/pipeline.rs
- [ ] T063 [US5] Ensure file handles cleaned up properly on command failure using RAII in crates/rush/src/executor/pipeline.rs
- [ ] T064 [US5] Ensure file handles cleaned up on Ctrl+C interruption in crates/rush/src/executor/pipeline.rs
- [ ] T065 [US5] Implement exit code propagation (command exit code, not redirection errors) in crates/rush/src/executor/pipeline.rs
- [ ] T066 [P] [US5] Integration test for redirecting to directory shows "is a directory" error in crates/rush/tests/integration/redirection_tests.rs
- [ ] T067 [P] [US5] Integration test for command failure with redirection propagates exit code in crates/rush/tests/integration/redirection_tests.rs
- [ ] T068 [P] [US5] Integration test for same file read/write race condition behavior in crates/rush/tests/integration/redirection_tests.rs
- [ ] T069 [P] [US5] Integration test for permission denied shows clear error in crates/rush/tests/integration/redirection_tests.rs
- [ ] T070 [P] [US5] Integration test for empty file created when command produces no output in crates/rush/tests/integration/redirection_tests.rs
- [ ] T071 [P] [US5] Integration test for redirecting input from empty file works correctly in crates/rush/tests/integration/redirection_tests.rs
- [ ] T072 [P] [US5] Integration test for operators inside quoted strings become literals in crates/rush/tests/integration/redirection_tests.rs
- [ ] T073 [P] [US5] Contract test validating FR-021 through FR-030 in crates/rush/tests/contract/redirection_spec_validation.rs

**Checkpoint**: All user stories complete with robust error handling

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Performance validation, documentation, and final quality improvements

- [ ] T074 [P] Create performance benchmark for redirection setup overhead in crates/rush/benches/redirection_bench.rs
- [ ] T075 [P] Benchmark baseline command execution without redirections in crates/rush/benches/redirection_bench.rs
- [ ] T076 [P] Benchmark command execution with output redirection in crates/rush/benches/redirection_bench.rs
- [ ] T077 [P] Benchmark command execution with append redirection to large file in crates/rush/benches/redirection_bench.rs
- [ ] T078 [P] Verify redirection overhead is <1ms per constitution requirement in crates/rush/benches/redirection_bench.rs
- [ ] T079 [P] Validate all 286 existing tests still pass (backward compatibility) using cargo test
- [ ] T080 [P] Update CLI.md with redirection operator usage examples in crates/rush/CLI.md
- [ ] T081 [P] Update README.md with redirection feature documentation in crates/rush/README.md
- [ ] T082 [P] Run cargo fmt to format all new code
- [ ] T083 [P] Run cargo clippy and address all warnings
- [ ] T084 [P] Run quickstart.md validation (if exists) in specs/005-output-redirection/quickstart.md
- [ ] T085 Verify all acceptance scenarios from spec.md are tested and passing
- [ ] T086 Create PR description summarizing changes and linking to spec.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1, US2, US3 are independent and can proceed in parallel (if staffed)
  - US4 (Combined) depends on US1 and US3 being complete
  - US5 (Error Handling) can proceed in parallel but benefits from US1-4 being tested first
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1) - Output >**: Can start after Foundational (Phase 2) - No dependencies
- **User Story 2 (P1) - Append >>**: Can start after Foundational (Phase 2) - No dependencies (builds on US1 pattern but independent)
- **User Story 3 (P2) - Input <**: Can start after Foundational (Phase 2) - No dependencies
- **User Story 4 (P2) - Combined**: Requires US1 (Output) and US3 (Input) complete for integration tests
- **User Story 5 (P3) - Error Handling**: Can start after Foundational, benefits from US1-4 being tested

### Within Each User Story

- Implementation tasks before integration tests
- Contract tests validate all requirements for that story
- Each story should be independently completable and testable

### Parallel Opportunities

- **Phase 1**: T001, T002, T003, T006, T007 can all run in parallel
- **Phase 2**: T013, T014, T015, T016, T017, T018, T019 can run in parallel after T008-T012 complete
- **Phase 3 (US1)**: T028-T033 can run in parallel after implementation
- **Phase 4 (US2)**: T037-T041 can run in parallel after implementation
- **Phase 5 (US3)**: T046-T051 can run in parallel after implementation
- **Phase 6 (US4)**: T056-T061 can run in parallel after implementation
- **Phase 7 (US5)**: T066-T073 can run in parallel after implementation
- **Phase 8**: T074-T084 can mostly run in parallel

---

## Parallel Example: User Story 1

```bash
# After implementation tasks T020-T027 complete, launch all tests in parallel:
Task: "Integration test for echo 'hello' > file.txt"
Task: "Integration test for overwriting existing file"
Task: "Integration test for ls -la > listing.txt"
Task: "Integration test for permission denied error"
Task: "Integration test for is a directory error"
Task: "Contract test validating FR-001 through FR-006"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Basic Output >)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready - users can now redirect output to files

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 (Output >) → Test independently → **MVP DELIVERED**
3. Add User Story 2 (Append >>) → Test independently → Both output operators work
4. Add User Story 3 (Input <) → Test independently → Full I/O redirection
5. Add User Story 4 (Combined) → Test independently → Complex workflows enabled
6. Add User Story 5 (Error Handling) → Test independently → Production-ready
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Output >)
   - Developer B: User Story 2 (Append >>)
   - Developer C: User Story 3 (Input <)
3. After US1 and US3 complete:
   - Developer A or B: User Story 4 (Combined)
4. Developer C: User Story 5 (Error Handling) in parallel
5. Team: Phase 8 (Polish) together

---

## Pull Request Strategy

**CRITICAL: Create separate PRs for each user story to keep changes reviewable.**

**PR Size Limits** (from CLAUDE.md):
- **Ideal**: 500 lines of changes
- **Maximum**: 1,500 lines
- **Too large**: 3,000+ lines - must split

**Planned PRs** (from plan.md deployment strategy):

```
PR #1: Foundation + Setup (Phase 1 + Phase 2)
  - Data structures (Redirection, RedirectionType, Token extensions)
  - Parser infrastructure (tokenization, no execution yet)
  - Unit tests for data structures and parsing
  - Target: ~400-500 lines ✅

PR #2: User Story 1 - Basic Output Redirection (>) [P1]
  - Implement > operator execution
  - PipelineExecutor integration
  - File creation with truncate mode
  - Integration tests and error handling
  - Target: ~600-800 lines ✅

PR #3: User Story 2 - Append Output Redirection (>>) [P1]
  - Implement >> operator execution
  - Append mode file opening
  - Integration tests for append behavior
  - Target: ~300-400 lines ✅

PR #4: User Story 3 + 4 - Input and Combined Redirections [P2]
  - Implement < operator execution
  - Combined redirection support
  - Integration with pipes
  - Multiple redirections (last wins)
  - Integration tests
  - Target: ~700-900 lines ✅

PR #5: User Story 5 + Polish - Error Handling and Edge Cases [P3]
  - Comprehensive error messages
  - Edge case handling
  - Contract tests validating all spec requirements
  - Performance benchmarks
  - Documentation updates
  - Target: ~400-500 lines ✅
```

**Workflow**:
1. Complete Phase 1 + Phase 2 (Foundation)
2. Commit and create **PR #1** for foundation
3. Merge PR #1 to main
4. Create branch for User Story 1 from updated main
5. Complete User Story 1 (Phase 3)
6. Commit and create **PR #2** for US1 only
7. Merge PR #2 to main
8. Repeat for each remaining user story

**Before Creating PR**:
- Check line count: `git diff --stat main | tail -1`
- If >1,500 lines, split by user story or component
- Ensure all tests pass: `cargo test`
- Ensure formatting: `cargo fmt --check`
- Ensure no clippy warnings: `cargo clippy`

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD)
- Commit after each logical group of tasks
- Stop at any checkpoint to validate story independently
- Constitution requirement: <1ms redirection overhead, <5ms total command overhead
- Backward compatibility: All 286 existing tests must continue passing

---

**Task Generation Complete**: 86 tasks across 8 phases, organized by 5 user stories with clear dependencies and parallel opportunities. Ready for implementation via `/speckit.implement`.
