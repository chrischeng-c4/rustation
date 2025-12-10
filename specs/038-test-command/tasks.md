# Tasks: Extended Test Command

**Input**: Design documents from `/specs/038-test-command/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/test-api.md

**Tests**: Tests are NOT requested in the specification - implementation only.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Source files: `crates/rush/src/`
- Test files: `crates/rush/tests/`
- Builtin modules: `crates/rush/src/executor/builtins/`
- Parser modules: `crates/rush/src/parser/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create basic module structure and error types for extended test command

- [x] T001 Add TestError variants to crates/rush/src/lib.rs (InvalidOperator, TypeMismatch, InvalidPattern, PatternTooLong, FileTestFailed)
- [x] T002 [P] Create test_expr.rs parser module at crates/rush/src/executor/test_expr.rs with module skeleton
- [x] T003 [P] Create test_extended.rs builtin at crates/rush/src/executor/builtins/test_extended.rs with module skeleton
- [x] T004 Register [[  builtin in crates/rush/src/executor/builtins/mod.rs execute_builtin() function

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures, parser infrastructure, and expression evaluation that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 [P] Define Expression enum in crates/rush/src/executor/test_expr.rs (UnaryOp, BinaryOp, LogicalOp, Grouped, Literal variants)
- [x] T006 [P] Define UnaryOperator enum in crates/rush/src/executor/test_expr.rs (Negation, StringEmpty, StringNonEmpty, file operators)
- [x] T007 [P] Define BinaryOperator enum in crates/rush/src/executor/test_expr.rs (string, numeric, pattern operators)
- [x] T008 [P] Define LogicalOperator enum in crates/rush/src/executor/test_expr.rs (And, Or variants)
- [x] T009 Create TestExpression struct in crates/rush/src/executor/test_expr.rs with expression field
- [x] T010 Implement parse_test_expression() function in crates/rush/src/executor/test_expr.rs (tokenize [[ ]] content)
- [x] T011 Implement recursive descent parser with precedence climbing in crates/rush/src/executor/test_expr.rs
- [x] T012 Add TestExpression variant to AST Command enum in crates/rush/src/executor/mod.rs (via builtin dispatch)
- [x] T013 Integrate test expression parsing into main parser in crates/rush/src/executor/builtins/test_extended.rs (recognize [[ keyword)
- [x] T014 Create TestEvaluator struct in crates/rush/src/executor/builtins/test_extended.rs
- [x] T015 Implement evaluate() dispatcher method in crates/rush/src/executor/builtins/test_extended.rs (routes to specific evaluators)
- [x] T016 Implement variable expansion integration in crates/rush/src/executor/builtins/test_extended.rs (no word splitting/globbing)

**Checkpoint**: ✅ **Foundation ready - user story implementation can now begin in parallel**

---

## Phase 3: User Story 1 - Basic Conditional Testing (Priority: P1) 🎯 MVP

**Goal**: Enable users to perform reliable string/numeric comparisons and file tests in shell scripts with clearer syntax and no quoting issues

**Independent Test**: Write if statements with string/numeric comparisons (`[[ "$var" == "value" ]]`, `[[ $a -lt $b ]]`, `[[ -f $file ]]`) and verify they produce correct exit codes

### Implementation for User Story 1

- [x] T017 [P] [US1] Implement evaluate_string_equality() in crates/rush/src/executor/builtins/test_extended.rs (== and != operators)
- [x] T018 [P] [US1] Implement evaluate_string_comparison() in crates/rush/src/executor/builtins/test_extended.rs (< and > lexicographic operators)
- [x] T019 [P] [US1] Implement evaluate_numeric_comparison() in crates/rush/src/executor/builtins/test_extended.rs (-eq, -ne, -lt, -le, -gt, -ge)
- [x] T020 [P] [US1] Implement evaluate_string_test() in crates/rush/src/executor/builtins/test_extended.rs (-z empty, -n non-empty)
- [x] T021 [US1] Implement evaluate_file_test() in crates/rush/src/executor/builtins/test_extended.rs using std::fs (-e, -f, -d, -r, -w, -x, -s)
- [x] T022 [US1] Implement evaluate_unary_op() dispatcher in crates/rush/src/executor/builtins/test_extended.rs (routes unary operators)
- [x] T023 [US1] Implement evaluate_binary_op() dispatcher in crates/rush/src/executor/builtins/test_extended.rs (routes binary operators)
- [x] T024 [US1] Implement evaluate_numeric_comparison() with integer parsing and error handling in crates/rush/src/executor/builtins/test_extended.rs
- [x] T025 [US1] Implement execute() function in crates/rush/src/executor/builtins/test_extended.rs (entry point, returns exit codes 0/1/2)
- [x] T026 [US1] Add CommandExecutor integration in crates/rush/src/executor/builtins/test_extended.rs (variable expansion and evaluation pipeline)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Pattern Matching and Regex (Priority: P2)

**Goal**: Enable users to match strings against glob patterns and regular expressions for validation and filtering

**Independent Test**: Write test cases with pattern matching operators (`[[ $filename == *.txt ]]`, `[[ $email =~ ^[a-z]+@[a-z]+ ]]`) and verify they match/reject expected patterns

### Implementation for User Story 2

- [ ] T027 [P] [US2] Create GlobMatcher module in crates/rush/src/executor/builtins/test_extended.rs with glob_match() function
- [ ] T028 [US2] Implement glob pattern matching algorithm in crates/rush/src/executor/builtins/test_extended.rs (*, ?, [...], [!...])
- [ ] T029 [US2] Add regex crate dependency to crates/rush/Cargo.toml (for POSIX ERE support)
- [ ] T030 [P] [US2] Create RegexMatcher module in crates/rush/src/executor/builtins/test_extended.rs with regex_match() function
- [ ] T031 [US2] Implement regex pattern compilation and matching in crates/rush/src/executor/builtins/test_extended.rs (use regex crate)
- [ ] T032 [US2] Implement BASH_REMATCH array population in crates/rush/src/executor/builtins/test_extended.rs (store captures in VariableManager)
- [ ] T033 [US2] Add pattern length validation (10KB limit) in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T034 [US2] Integrate glob matching into evaluate_binary_op() for == and != in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T035 [US2] Integrate regex matching into evaluate_binary_op() for =~ operator in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T036 [US2] Add error handling for invalid regex patterns in crates/rush/src/executor/builtins/test_extended.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Complex Conditional Logic (Priority: P3)

**Goal**: Enable users to combine multiple test conditions with logical operators for cleaner, more readable scripts

**Independent Test**: Write complex conditional expressions with multiple clauses (`[[ $a -gt 0 && $b -lt 100 ]]`, `[[ ( $x == "foo" || $x == "bar" ) && -f $file ]]`) and verify correct short-circuit evaluation

### Implementation for User Story 3

- [ ] T037 [P] [US3] Implement evaluate_logical_and() in crates/rush/src/executor/builtins/test_extended.rs with short-circuit (stop if left false)
- [ ] T038 [P] [US3] Implement evaluate_logical_or() in crates/rush/src/executor/builtins/test_extended.rs with short-circuit (stop if left true)
- [ ] T039 [P] [US3] Implement evaluate_negation() in crates/rush/src/executor/builtins/test_extended.rs (! operator)
- [ ] T040 [US3] Implement evaluate_grouped() in crates/rush/src/executor/builtins/test_extended.rs (handle parentheses)
- [ ] T041 [US3] Add LogicalOp handling to evaluate() dispatcher in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T042 [US3] Update parser to handle operator precedence correctly in crates/rush/src/parser/test_expr.rs (! > comparisons > && > ||)
- [ ] T043 [US3] Update parser to handle parentheses grouping in crates/rush/src/parser/test_expr.rs
- [ ] T044 [US3] Add parentheses balance validation in crates/rush/src/parser/test_expr.rs

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Edge Cases and Error Handling

**Purpose**: Harden implementation against edge cases identified in spec

- [ ] T045 [P] Add invalid regex pattern error handling in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T046 [P] Add unset variable handling (treat as empty string) in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T047 [P] Add type mismatch error messages (numeric ops on non-numbers) in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T048 [P] Add empty string pattern matching behavior in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T049 [P] Add special character escaping in glob patterns in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T050 Add file test error handling for non-existent files in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T051 Add unicode character support in regex patterns in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T052 Add expression nesting depth limit (32 levels) in crates/rush/src/parser/test_expr.rs

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T053 [P] Add comprehensive error messages for all TestError variants in crates/rush/src/error.rs
- [ ] T054 [P] Add syntax error detection with helpful messages in crates/rush/src/parser/test_expr.rs
- [ ] T055 Add performance optimization for simple expressions (< 1ms target) in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T056 Add performance optimization for regex matching (< 10ms target) in crates/rush/src/executor/builtins/test_extended.rs
- [ ] T057 Run cargo clippy and address any warnings in crates/rush/
- [ ] T058 Run cargo fmt to format code in crates/rush/
- [ ] T059 Update CLAUDE.md with Extended Test Command dependencies (regex crate)
- [ ] T060 Verify build succeeds with cargo build

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Edge Cases (Phase 6)**: Can run in parallel with user stories or after
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Independent of US1 (adds pattern matching)
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Independent of US1/US2 (adds logical operators)

### Within Each User Story

- **Foundation tasks** (Phase 2) MUST complete before any user story
- **Within each story**: Tasks can run in parallel if marked [P]
- **Story complete** before moving to next priority

### Parallel Opportunities

- All Setup tasks (T001-T004) marked [P] can run in parallel
- All Foundational enum definitions (T005-T008) can run in parallel
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- Within User Story 1: T017-T020 can run in parallel (different operators)
- Within User Story 2: T027-T028 (glob) and T030-T031 (regex) can run in parallel
- Within User Story 3: T037-T039 can run in parallel (different logical operators)
- All edge case tasks (T045-T051) can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all operator implementations for User Story 1 together:
Task: "Implement evaluate_string_equality() in crates/rush/src/executor/builtins/test_extended.rs"
Task: "Implement evaluate_string_comparison() in crates/rush/src/executor/builtins/test_extended.rs"
Task: "Implement evaluate_numeric_comparison() in crates/rush/src/executor/builtins/test_extended.rs"
Task: "Implement evaluate_string_test() in crates/rush/src/executor/builtins/test_extended.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T016) - CRITICAL - blocks all stories
3. Complete Phase 3: User Story 1 (T017-T026)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add Edge Cases + Polish → Feature complete

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Basic conditionals)
   - Developer B: User Story 2 (Pattern matching)
   - Developer C: User Story 3 (Complex logic)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files or different functions, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

### Pull Request Strategy

**CRITICAL: Create separate PRs for each user story to keep changes reviewable.**

**PR Size Limits** (see CLAUDE.md for details):
- **Ideal**: 500 lines of changes
- **Maximum**: 1,500 lines
- **Too large**: 3,000+ lines - must split

**Workflow for Multi-Story Features**:
1. Complete Phase 1 (Setup) + Phase 2 (Foundational)
2. Commit and create **PR #1** for foundation (~200 lines)
3. Merge PR #1 to main
4. Create branch for User Story 1 from updated main
5. Complete User Story 1 (Phase 3)
6. Commit and create **PR #2** for US1 only (~800 lines)
7. Merge PR #2 to main (MVP complete!)
8. Create branch for User Story 2 from updated main
9. Complete User Story 2 (Phase 4)
10. Commit and create **PR #3** for US2 only (~500 lines)
11. Merge PR #3 to main
12. Create branch for User Story 3 from updated main
13. Complete User Story 3 (Phase 5)
14. Commit and create **PR #4** for US3 only (~400 lines)
15. Merge PR #4 to main
16. Create branch for Polish from updated main
17. Complete Phase 6 + 7 (Edge Cases + Polish)
18. Commit and create **PR #5** for polish (~100 lines)
19. Merge PR #5 to main (Feature complete!)

**Before Creating PR**:
- Check line count: `git diff --stat main`
- If >1,500 lines, split by user story or logical component
- Each PR should be independently reviewable and mergeable

**Benefits**:
- Faster code review cycles
- Easier to discuss specific changes
- Can merge incrementally (deliver value sooner)
- Simpler rollback if issues found
- Each PR delivers independently testable value
