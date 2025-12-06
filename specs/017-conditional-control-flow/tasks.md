# Tasks: Conditional Control Flow (if/then/else/elif/fi)

**Input**: Design documents from `/specs/017-conditional-control-flow/`
**Prerequisites**: plan.md (Rust monorepo, reedline REPL), spec.md (5 user stories P1-P5), data-model.md (AST structures), research.md (parser design)

**Tests**: Tests are OPTIONAL. This feature does not explicitly request TDD, but integration tests are included as best practices for shell semantics validation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `crates/rush/src/`, `crates/rush/tests/` (Rust monorepo structure)
- Paths shown below follow the plan.md project structure for the rush shell

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure for conditional control flow

- [ ] T001 Add AST structure types to `crates/rush/src/executor/mod.rs`: `IfBlock`, `ElifClause`, `CompoundList` structs and `Command` enum variant
- [ ] T002 Add reserved keyword types to `crates/rush/src/executor/mod.rs`: `Keyword` enum (If, Then, Elif, Else, Fi) with helper functions
- [ ] T003 [P] Add syntax error type to `crates/rush/src/lib.rs`: `SyntaxError` enum with variants for syntax errors (UnexpectedToken, UnmatchedKeyword, MissingKeyword)
- [ ] T004 Extend `Token` enum in `crates/rush/src/executor/parser.rs` to include `Keyword(Keyword)` variant and `Newline`/`Eof` tokens

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core parser and executor infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 [P] Implement keyword tokenization in `crates/rush/src/executor/parser.rs`: Function to recognize if/then/elif/else/fi as keywords only in command position
- [ ] T006 [P] Implement `expect_keyword()` helper in `crates/rush/src/executor/parser.rs` to validate and consume keyword tokens from token stream
- [ ] T007 Create new file `crates/rush/src/executor/conditional.rs` with module declaration in `mod.rs`
- [ ] T008 Implement `parse_compound_list()` function in `crates/rush/src/executor/conditional.rs` to parse a sequence of commands separated by `;` or newlines
- [ ] T009 [P] Implement `execute_compound_list()` function in `crates/rush/src/executor/mod.rs` or `execute.rs` to execute a sequence of commands and return exit code of last command
- [ ] T010 Modify parser entry point in `crates/rush/src/executor/parser.rs` to detect `if` token at command start and dispatch to `parse_if_clause()` instead of regular pipeline parsing
- [ ] T011 [P] Create unit test file `crates/rush/tests/unit/parser_conditional.rs` with placeholder test structure
- [ ] T012 [P] Create integration test file `crates/rush/tests/integration/conditionals.rs` with placeholder test structure

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Simple Conditional Execution (Priority: P1) 🎯 MVP

**Goal**: Implement basic `if condition; then commands; fi` syntax and execution - the fundamental conditional pattern

**Independent Test**: Can be fully tested by running `if true; then echo "success"; fi` and verifying correct branch execution based on exit status

### Implementation for User Story 1

- [ ] T013 [US1] Implement `parse_if_clause()` skeleton in `crates/rush/src/executor/conditional.rs`: Parse if/then/fi without elif/else support
- [ ] T014 [US1] Implement parsing for if condition (before `then` keyword) in `parse_if_clause()` - must handle single or multiple commands separated by `;` or newlines
- [ ] T015 [US1] Implement parsing for then block (between `then` and `fi` keywords) in `parse_if_clause()` - may be empty
- [ ] T016 [US1] Implement mandatory `fi` keyword validation at end of `parse_if_clause()`
- [ ] T017 [US1] Implement `execute_if_block()` function in `crates/rush/src/executor/conditional.rs` to: (1) execute condition, (2) check exit code 0, (3) execute then_block if true, (4) return exit code of last command in branch
- [ ] T018 [US1] Integrate `Command::If` handling in executor's main command dispatch (likely in `execute.rs` or `execute_compound_list()`) to call `execute_if_block()`
- [ ] T019 [US1] Add unit tests in `crates/rush/tests/unit/parser_conditional.rs` for `parse_if_clause()`: Valid if/then/fi, missing fi error, missing then error, empty then block
- [ ] T020 [US1] [P] Add integration tests in `crates/rush/tests/integration/conditionals.rs` for User Story 1 acceptance scenarios: if true with output, if false with no output, if with command exit status, if with nonexistent command
- [ ] T021 [US1] Verify exit code propagation: `if true; then false; fi; echo $?` should print 1 (exit code of last command in branch)
- [ ] T022 [US1] Test empty then block: `if true; then; fi` should execute and return 0

**Checkpoint**: At this point, User Story 1 (basic if/then/fi) should be fully functional and independently testable

---

## Phase 4: User Story 2 - Conditional with Alternative (Priority: P2)

**Goal**: Add `else` clause support to handle both success and failure cases

**Independent Test**: Can be fully tested by running `if false; then echo "yes"; else echo "no"; fi` and verifying else branch executes

### Implementation for User Story 2

- [ ] T023 [US2] Extend `IfBlock` struct (already in T001) to add `else_block: Option<CompoundList>` field - mark as already added in T001
- [ ] T024 [US2] Implement `parse_else_part()` helper function in `crates/rush/src/executor/conditional.rs` to parse optional else block and return (elif_clauses, else_block)
- [ ] T025 [US2] Modify `parse_if_clause()` in `crates/rush/src/executor/conditional.rs` to call `parse_else_part()` and populate `else_block` field of IfBlock
- [ ] T026 [US2] Implement else block parsing in `parse_else_part()`: Recognize `else` keyword, parse commands until `fi`, set as `Option::Some`
- [ ] T027 [US2] Extend `execute_if_block()` in `crates/rush/src/executor/conditional.rs` to check else_block: if no conditions matched and else_block is Some, execute it and return its exit code
- [ ] T028 [US2] Add unit tests in `crates/rush/tests/unit/parser_conditional.rs` for `parse_else_part()`: Valid else, empty else block, no else (returns None)
- [ ] T029 [US2] [P] Add integration tests in `crates/rush/tests/integration/conditionals.rs` for User Story 2 acceptance scenarios: if true with else (then executes), if false with else (else executes), multiline if/then/else/fi
- [ ] T030 [US2] Verify exit code: `if false; then echo "yes"; else echo "no"; fi; echo $?` should print "no" and then 0

**Checkpoint**: At this point, User Stories 1 AND 2 (if/then/else/fi) should work independently

---

## Phase 5: User Story 3 - Multiple Condition Branches (Priority: P3)

**Goal**: Add support for multiple `elif` clauses to handle multi-way branching without deep nesting

**Independent Test**: Can be fully tested by running `if false; then echo "1"; elif true; then echo "2"; fi` and verifying elif branch executes

### Implementation for User Story 3

- [ ] T031 [US3] Add `elif_clauses: Vec<ElifClause>` field to `IfBlock` struct (already in T001) - mark as already added
- [ ] T032 [US3] Extend `parse_else_part()` in `crates/rush/src/executor/conditional.rs` to recognize and parse `elif` keyword and its condition/then block
- [ ] T033 [US3] Implement elif parsing recursion: `elif condition; then block` followed by another else_part (may have more elif or final else)
- [ ] T034 [US3] Validate during parsing that elif appears only as part of else_part (not before first condition error case covered in edge tests)
- [ ] T035 [US3] Extend `execute_if_block()` in `crates/rush/src/executor/conditional.rs` to iterate through `elif_clauses` after condition fails: execute each elif condition until one returns 0, execute its then_block, stop (short-circuit)
- [ ] T036 [US3] Implement short-circuit evaluation: after first successful condition (if or elif), do NOT evaluate remaining elif clauses, only execute matched branch then_block
- [ ] T037 [US3] Add unit tests in `crates/rush/tests/unit/parser_conditional.rs` for elif parsing: Single elif, multiple elif, elif before else, elif with empty block
- [ ] T038 [US3] [P] Add integration tests in `crates/rush/tests/integration/conditionals.rs` for User Story 3 acceptance scenarios: if false/elif true/fi, if false/elif false/else/fi, multiple elif branches, first-match-wins (if true/elif true/fi executes if block only)
- [ ] T039 [US3] Verify short-circuit behavior: `if false; then echo "1"; elif echo "2"; then echo "3"; fi` should print "2" once (condition executed) and "3" (then block), not "2" twice
- [ ] T040 [US3] Test exit code with elif: `if false; then false; elif true; then false; fi; echo $?` should print 1 (exit code from elif's then block)

**Checkpoint**: All user stories 1-3 (if/then/else/elif/fi) should now be independently functional

---

## Phase 6: User Story 4 - Nested Conditionals (Priority: P4)

**Goal**: Support if statements inside other if statements for complex decision trees

**Independent Test**: Can be fully tested by running `if true; then if true; then echo "nested"; fi; fi` and verifying "nested" is printed

### Implementation for User Story 4

- [ ] T041 [US4] Verify `Command` enum already includes `Command::If(Box<IfBlock>)` variant (from T001) - this enables nesting
- [ ] T042 [US4] Verify `parse_compound_list()` (from T008) correctly delegates to `parse_if_clause()` when it encounters `if` token - enables recursive parsing
- [ ] T043 [US4] Test recursive parsing: Can `parse_if_clause()` be called from within `parse_compound_list()` which is part of another `parse_if_clause()`? Verify this works recursively
- [ ] T044 [US4] Add depth tracking to parser (optional): Add `depth` counter in `parse_if_clause()` to prevent stack overflow for deeply nested constructs; if depth exceeds 100, return error
- [ ] T045 [US4] Verify executor: `execute_compound_list()` already handles `Command::If` variants, so execution of nested if should work automatically
- [ ] T046 [US4] Add unit tests in `crates/rush/tests/unit/parser_conditional.rs` for nested parsing: 2-level nesting, 3-level nesting, deeply nested (5+ levels)
- [ ] T047 [US4] [P] Add integration tests in `crates/rush/tests/integration/conditionals.rs` for User Story 4 acceptance scenarios: nested if/then/fi (inner executes, outer continues), nested with both branches, 3+ level nesting with elif combinations
- [ ] T048 [US4] Test complex nesting: `if true; then if false; then echo "a"; else echo "b"; fi; echo "c"; fi` should print "b" and "c"
- [ ] T049 [US4] Verify exit code through nesting: `if true; then if false; then false; else false; fi; fi; echo $?` should print 1

**Checkpoint**: At this point, all user stories 1-4 (if/then/else/elif/fi with nesting) should work independently

---

## Phase 7: User Story 5 - Multiline Interactive Entry (Priority: P5)

**Goal**: Support continuation prompts in interactive REPL mode for readable multiline conditional blocks

**Independent Test**: Can be fully tested by entering `if true; then` and verifying shell displays continuation prompt rather than executing immediately

### Implementation for User Story 5

- [ ] T050 [US5] Create validator function in new file `crates/rush/src/repl/validator.rs` to count open/close keywords (if/elif/for/while/case vs fi/done/esac)
- [ ] T051 [US5] Implement `count_control_flow_depth()` function in `crates/rush/src/repl/validator.rs` to scan input for if/then/elif/else/fi and return net depth (positive = waiting for close)
- [ ] T052 [US5] Implement `is_incomplete_construct()` function in `crates/rush/src/repl/validator.rs` to detect if input ends with incomplete if/elif/else/fi structure
- [ ] T053 [US5] Modify REPL input loop in `crates/rush/src/repl/mod.rs` to detect incomplete conditionals: after parsing each line, check if depth > 0
- [ ] T054 [US5] Implement input buffering in REPL: Create buffer to accumulate lines until conditional is complete (depth == 0)
- [ ] T055 [US5] Implement continuation prompt display in `crates/rush/src/repl/mod.rs`: Display `> ` prompt when depth > 0 (instead of normal `$ ` prompt)
- [ ] T056 [US5] Ensure consistent prompt for all nesting levels: Use same `> ` prompt regardless of depth (per research decision)
- [ ] T057 [US5] Add logic to handle errors during multiline entry: If syntax error occurs on line N, display error and allow user to recover or start over
- [ ] T058 [US5] [P] Create interactive test scenarios in `crates/rush/tests/integration/multiline.rs` for User Story 5: Interactive if/then/fi entry, interactive if/then/else/fi, interactive with errors
- [ ] T059 [US5] Test interactive scenario 1: Enter `if true; then` → Display `> `, Enter `echo "hello"` → Display `> `, Enter `fi` → Execute construct
- [ ] T060 [US5] Test interactive scenario 2: Enter `if true; then` → Display `> `, Enter `fi fi` → Display syntax error and return to normal prompt
- [ ] T061 [US5] Verify exit code in interactive: After multiline if/then/false/fi entry, `echo $?` should show 1

**Checkpoint**: User Story 5 (multiline interactive entry) complete - all user stories 1-5 fully functional

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements, refinements, and validation across all user stories

- [ ] T062 [P] Document API in code comments: Add doc comments to AST structures (IfBlock, ElifClause, CompoundList) explaining purpose and usage
- [ ] T063 [P] Document parser functions: Add doc comments to `parse_if_clause()`, `parse_else_part()`, `parse_compound_list()` with examples
- [ ] T064 [P] Error message clarity: Verify all `SyntaxError` messages match format from research: "syntax error near unexpected token 'X', expected Y"
- [ ] T065 Run full test suite: `cargo test -p rush` to ensure no regressions in existing functionality
- [ ] T066 Run conditional-specific tests: `cargo test -p rush conditionals` to verify all user story acceptance tests pass
- [ ] T067 [P] Performance validation: Time `cargo run -p rush -- -c 'if true; then if true; then if true; then echo nested; fi; fi; fi'` - should complete in <10ms (per plan)
- [ ] T068 Run quickstart.md validation: Manually test all quickstart scenarios to ensure they work as documented
- [ ] T069 Verify against bash reference: Compare behavior with bash for edge cases: `if true; then; fi`, `if true; then false; fi; echo $?`, etc.
- [ ] T070 [P] Code cleanup: Review code for consistency with rush codebase style, remove debug prints, ensure no TODO markers left

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-7)**: All depend on Foundational phase completion
  - User stories can proceed in parallel (US1, US2, US3, US4, US5 independently) OR sequentially in priority order
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - MVP, no dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational - Independent, can integrate with US1 tests
- **User Story 3 (P3)**: Can start after Foundational - Independent, can integrate with US1/US2 tests
- **User Story 4 (P4)**: Can start after Foundational - Independent, requires recursive parsing/execution (already in foundational)
- **User Story 5 (P5)**: Can start after Foundational - Independent, works with output from all other stories

### Within Each User Story

- Parser tasks before executor tasks (parsing must happen before execution)
- Unit tests before integration tests (unit validates components, integration validates scenarios)
- For US2: else parsing before else execution
- For US3: elif parsing before elif execution and short-circuit logic
- For US4: No new parser tasks; tests validate recursive behavior already enabled
- For US5: Validator functions before REPL integration before interactive tests

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (Tasks 1-4)
- All Foundational tasks marked [P] can run in parallel (Tasks 5-6, 11-12)
- Once Foundational phase completes, User Stories 1-5 can be worked in parallel by different developers
- Within Phase 3: T019-T020 parser and integration tests can run in parallel
- Within Phase 4: T024 parser implementation and T028-T029 tests can start together
- Within Phase 8: All [P] tasks can run in parallel

---

## Parallel Example: User Story 1 MVP

```bash
# After Foundational phase (T001-T012) completes:

# Launch parser implementation and tests together:
# Task T013: parse_if_clause skeleton
# Task T019: unit tests for parser (will initially fail)

# Launch executor implementation and tests together:
# Task T017: execute_if_block
# Task T020: integration tests (will initially fail)

# Iterate: tests fail → implement → tests pass

# Once all T013-T022 pass: User Story 1 is complete and independently testable
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T012)
3. Complete Phase 3: User Story 1 (T013-T022)
4. **STOP and VALIDATE**: Run `cargo test -p rush conditionals` - all US1 tests pass
5. Test manually: `cargo run -p rush -- -c 'if true; then echo success; fi'`
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 (if/then/fi) → Test independently → Validate
3. Add User Story 2 (if/then/else/fi) → Test independently → Validate
4. Add User Story 3 (if/then/elif/else/fi) → Test independently → Validate
5. Add User Story 4 (nested if) → Test independently → Validate
6. Add User Story 5 (interactive multiline) → Test independently → Validate
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (if/then/fi)
   - Developer B: User Story 2 (else clause) + Story 3 (elif) sequentially
   - Developer C: User Story 4 (nesting) + Story 5 (multiline) sequentially
3. Stories complete and integrate independently
4. Team does Polish phase together

---

## Notes

- [P] tasks = different files, no dependencies on same-phase tasks
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Verify tests fail before implementing (test-driven approach recommended)
- Commit after each user story phase completes (Phase 3 commit, Phase 4 commit, etc.)
- Stop at any checkpoint to validate story independently
- For Phase 3 (US1) to complete MVP, do T001-T012 (Setup+Foundational), then T013-T022

### Pull Request Strategy

**CRITICAL: Create separate PRs for each user story to keep changes reviewable.**

**PR Size Limits** (from CLAUDE.md):
- **Ideal**: ≤ 500 lines
- **Maximum**: ≤ 1,500 lines
- **Too large**: 3,000+ lines (must split)

**Workflow for Multi-Story Feature**:

1. Complete Phase 1 (Setup) + Phase 2 (Foundational): T001-T012
2. Commit: `git commit -m "feat(017): add conditional control flow AST and parser foundation"`
3. Create **PR #1** for foundation (estimated 500-800 lines)
4. Merge PR #1 to main

5. Complete Phase 3 (User Story 1): T013-T022
6. Commit: `git commit -m "feat(017): implement simple if/then/fi (US1)"`
7. Create **PR #2** for US1 only (estimated 800-1,200 lines including tests)
8. Merge PR #2 to main

9. Complete Phase 4 (User Story 2): T023-T030
10. Commit: `git commit -m "feat(017): add else clause support (US2)"`
11. Create **PR #3** for US2 only (estimated 400-700 lines)
12. Merge PR #3 to main

13. Repeat for US3 (elif), US4 (nesting), US5 (multiline)

14. Phase 8 (Polish): Final polish and documentation
15. Create **PR #6** for polish (estimated 200-400 lines)
16. Merge PR #6 to main

**Before Creating PR**:
- Check line count: `git diff --stat main`
- If >1,500 lines, split by user story or logical component
- Each PR should be independently reviewable and mergeable

**Benefits**:
- Faster code review cycles
- Easier to discuss specific changes
- Can merge incrementally (deliver value sooner)
- Simpler rollback if issues found
- Each story can be validated independently

---

## Task Completion Checklist (Meta)

After completing all tasks:

- [ ] All Phase 1 tasks complete (Setup)
- [ ] All Phase 2 tasks complete (Foundational)
- [ ] All Phase 3 tasks complete (US1) - MVP milestone
- [ ] All Phase 4 tasks complete (US2)
- [ ] All Phase 5 tasks complete (US3)
- [ ] All Phase 6 tasks complete (US4)
- [ ] All Phase 7 tasks complete (US5)
- [ ] All Phase 8 tasks complete (Polish)
- [ ] `cargo test -p rush` passes with no regressions
- [ ] All acceptance scenarios from spec.md verified
- [ ] All success criteria from spec.md met
- [ ] Quickstart.md scenarios validated
- [ ] Performance targets met (<10ms for nested conditionals, <100ms startup)
- [ ] Edge cases from spec.md tested
- [ ] Interactive multiline tested manually in REPL
- [ ] Bash reference compatibility verified for sample scripts
