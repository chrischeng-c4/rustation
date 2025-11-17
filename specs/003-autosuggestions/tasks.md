# Tasks: History-Based Autosuggestions

**Input**: Design documents from `/specs/003-autosuggestions/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included for this feature to validate autosuggestion behavior.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `crates/rush/src/`, `crates/rush/tests/`
- Paths follow Rust monorepo structure from plan.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure (already complete - skip to Phase 2)

- [X] T001 Project structure already exists in crates/rush/
- [X] T002 Dependencies already configured (reedline with Hinter trait support)
- [X] T003 Linting and formatting tools already configured (clippy, rustfmt)

**Status**: ✅ Setup complete (existing project)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Create new module file crates/rush/src/repl/suggest.rs with module documentation
- [ ] T005 [P] Add suggest module declaration in crates/rush/src/repl/mod.rs
- [ ] T006 [P] Create test file crates/rush/tests/unit/suggest_tests.rs with test module setup
- [ ] T007 [P] Create integration test file crates/rush/tests/integration/autosuggestions_tests.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Basic Inline Suggestion Display (Priority: P1) 🎯 MVP

**Goal**: Display inline suggestions from command history as grayed-out text after cursor when user types

**Independent Test**: Type "git s" (assuming "git status" in history) and verify grayed "tatus" appears immediately after cursor

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T008 [P] [US1] Write unit test for RushHinter::new() in tests/unit/suggest_tests.rs
- [ ] T009 [P] [US1] Write unit test for prefix matching logic in tests/unit/suggest_tests.rs
- [ ] T010 [P] [US1] Write unit test for most recent match selection in tests/unit/suggest_tests.rs
- [ ] T011 [P] [US1] Write unit test for cursor position check (pos == line.len()) in tests/unit/suggest_tests.rs
- [ ] T012 [P] [US1] Write unit test for empty input handling in tests/unit/suggest_tests.rs
- [ ] T013 [P] [US1] Write unit test for no matching history in tests/unit/suggest_tests.rs
- [ ] T014 [P] [US1] Write integration test for real-time suggestion updates in tests/integration/autosuggestions_tests.rs

### Implementation for User Story 1

- [ ] T015 [US1] Define RushHinter struct in crates/rush/src/repl/suggest.rs
- [ ] T016 [US1] Implement RushHinter::new() constructor in crates/rush/src/repl/suggest.rs
- [ ] T017 [US1] Implement find_suggestion() helper method with prefix matching logic in crates/rush/src/repl/suggest.rs
- [ ] T018 [US1] Implement reedline::Hinter trait for RushHinter in crates/rush/src/repl/suggest.rs
- [ ] T019 [US1] Add cursor position validation (pos == line.len()) in hint() method in crates/rush/src/repl/suggest.rs
- [ ] T020 [US1] Add empty input check in hint() method in crates/rush/src/repl/suggest.rs
- [ ] T021 [US1] Add reverse chronological history iteration in find_suggestion() in crates/rush/src/repl/suggest.rs
- [ ] T022 [US1] Integrate RushHinter into Repl::with_config() in crates/rush/src/repl/mod.rs using .with_hinter()
- [ ] T023 [US1] Run cargo test to verify all US1 tests pass
- [ ] T024 [US1] Run cargo clippy to check for warnings
- [ ] T025 [US1] Test manually: Start rush, type "git s", verify grayed suggestion appears

**Checkpoint**: At this point, User Story 1 should be fully functional - suggestions display inline

---

## Phase 4: User Story 2 - Accept Suggestion with Arrow Key (Priority: P2)

**Goal**: Allow users to accept displayed suggestions by pressing Right Arrow key

**Independent Test**: Type "git s", see suggestion, press Right Arrow, verify buffer becomes "git status"

### Tests for User Story 2

- [ ] T026 [P] [US2] Write integration test for Right Arrow acceptance in tests/integration/autosuggestions_tests.rs
- [ ] T027 [P] [US2] Write integration test for accept then continue typing in tests/integration/autosuggestions_tests.rs
- [ ] T028 [P] [US2] Write integration test for Right Arrow when no suggestion in tests/integration/autosuggestions_tests.rs

### Implementation for User Story 2

- [ ] T029 [US2] Review existing keybindings in crates/rush/src/repl/mod.rs
- [ ] T030 [US2] Verify EditCommand::AcceptHint is available in reedline (from research.md)
- [ ] T031 [US2] Add Right Arrow keybinding for AcceptHint in Repl::with_config() in crates/rush/src/repl/mod.rs
- [ ] T032 [US2] Configure conditional behavior (cursor at end + suggestion exists) in crates/rush/src/repl/mod.rs
- [ ] T033 [US2] Run cargo test to verify all US2 tests pass
- [ ] T034 [US2] Run cargo clippy to check for warnings
- [ ] T035 [US2] Test manually: Type "git s", press Right Arrow, verify buffer becomes "git status"

**Checkpoint**: At this point, User Stories 1 AND 2 should both work - suggestions can be accepted

---

## Phase 5: User Story 3 - Accept Partial Suggestion (Priority: P3)

**Goal**: Allow users to accept suggestions word-by-word using Alt+Right Arrow

**Independent Test**: Type "git", see suggestion " commit -m 'message'", press Alt+Right, verify only " commit" accepted

### Tests for User Story 3

- [ ] T036 [P] [US3] Write integration test for Alt+Right Arrow word acceptance in tests/integration/autosuggestions_tests.rs
- [ ] T037 [P] [US3] Write integration test for accept word-by-word in tests/integration/autosuggestions_tests.rs
- [ ] T038 [P] [US3] Write integration test for single-word suggestion handling in tests/integration/autosuggestions_tests.rs

### Implementation for User Story 3

- [ ] T039 [US3] Verify EditCommand::AcceptHintWord is available in reedline (from research.md)
- [ ] T040 [US3] Add Alt+Right Arrow keybinding for AcceptHintWord in Repl::with_config() in crates/rush/src/repl/mod.rs
- [ ] T041 [US3] Configure word boundary detection (reedline handles this) in crates/rush/src/repl/mod.rs
- [ ] T042 [US3] Run cargo test to verify all US3 tests pass
- [ ] T043 [US3] Run cargo clippy to check for warnings
- [ ] T044 [US3] Test manually: Type "git", press Alt+Right multiple times, verify word-by-word acceptance

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and production readiness

- [ ] T045 [P] Add comprehensive doc comments to RushHinter in crates/rush/src/repl/suggest.rs
- [ ] T046 [P] Add examples to doc comments showing usage in crates/rush/src/repl/suggest.rs
- [ ] T047 [P] Run rustdoc to verify documentation completeness
- [ ] T048 [P] Add edge case test for empty history in tests/unit/suggest_tests.rs
- [ ] T049 [P] Add edge case test for special characters in tests/unit/suggest_tests.rs
- [ ] T050 [P] Add edge case test for very long suggestions in tests/unit/suggest_tests.rs
- [ ] T051 [P] Add performance benchmark test for 10k history entries in tests/integration/autosuggestions_tests.rs
- [ ] T052 Run complete test suite: cargo test -p rush
- [ ] T053 Run clippy with all features: cargo clippy -p rush --all-targets --all-features
- [ ] T054 Format code: cargo fmt -p rush
- [ ] T055 Update README.md to document autosuggestions feature
- [ ] T056 Update KNOWN_ISSUES.md to mark autosuggestions as implemented
- [ ] T057 Update crates/rush/KNOWN_ISSUES.md roadmap section
- [ ] T058 Run manual testing from specs/003-autosuggestions/quickstart.md (all 15 test scenarios)
- [ ] T059 Verify performance: latency <50ms for 10k history entries
- [ ] T060 Build release binary: cargo build --release -p rush
- [ ] T061 Install and smoke test: Install to ~/.local/bin and verify functionality

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: ✅ Already complete - no dependencies
- **Foundational (Phase 2)**: Depends on Setup (already complete) - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User stories can proceed in parallel (if staffed) after Phase 2
  - Or sequentially in priority order: P1 (US1) → P2 (US2) → P3 (US3)
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Integrates with US1 (acceptance requires suggestions)
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Extends US2 (word-by-word acceptance)

### Within Each User Story

- Tests before implementation (TDD approach)
- Core implementation (RushHinter) before integration (REPL)
- Manual testing after automated tests pass
- Story complete before moving to next priority

### Parallel Opportunities

- All Foundational tasks (T004-T007) can run in parallel
- All test writing tasks within a story can run in parallel
- Once Foundational complete, all user stories can start in parallel (if team capacity allows)
- All Polish tasks marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all test tasks for User Story 1 together:
Task: "Write unit test for RushHinter::new() in tests/unit/suggest_tests.rs"
Task: "Write unit test for prefix matching logic in tests/unit/suggest_tests.rs"
Task: "Write unit test for most recent match selection in tests/unit/suggest_tests.rs"
# ... all test tasks T008-T014 can run together

# After tests written, implementation can proceed sequentially:
Task: "Define RushHinter struct" (T015)
Task: "Implement new() constructor" (T016)
Task: "Implement find_suggestion()" (T017)
# ... continue with T018-T025
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (✅ Already done)
2. Complete Phase 2: Foundational (T004-T007)
3. Complete Phase 3: User Story 1 (T008-T025)
4. **STOP and VALIDATE**: Test User Story 1 independently using quickstart.md
5. Create PR #1 and merge (MVP complete!)

**Deliverable**: Users can see autosuggestions from history

### Incremental Delivery

1. Complete Foundation (Phase 2) → Merge to main
2. Add User Story 1 (Phase 3) → Test independently → Create PR #1 → Merge (MVP!)
3. Add User Story 2 (Phase 4) → Test independently → Create PR #2 → Merge
4. Add User Story 3 (Phase 5) → Test independently → Create PR #3 → Merge
5. Add Polish (Phase 6) → Create PR #4 → Merge
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Foundation together (Phase 2)
2. Once Foundation done:
   - Developer A: User Story 1 (Phase 3)
   - Developer B: User Story 2 (Phase 4) - can start writing tests
   - Developer C: User Story 3 (Phase 5) - can start writing tests
3. Stories complete and integrate independently
4. Each developer creates their own PR per story

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
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
2. Commit and create **PR #1** for foundation (if needed)
3. Merge PR #1 to main
4. Complete User Story 1 (Phase 3)
5. Commit and create **PR #2** for US1 only (~800 lines target)
6. Merge PR #2 to main
7. Complete User Story 2 (Phase 4)
8. Commit and create **PR #3** for US2 only (~600 lines target)
9. Merge PR #3 to main
10. Complete User Story 3 (Phase 5)
11. Commit and create **PR #4** for US3 only (~700 lines target)
12. Merge PR #4 to main
13. Complete Polish (Phase 6)
14. Commit and create **PR #5** for polish (~400 lines target)
15. Merge PR #5 to main

**Before Creating PR**:
- Check line count: `git diff --stat main`
- If >1,500 lines, split by user story or logical component
- Each PR should be independently reviewable and mergeable

**Benefits**:
- Faster code review cycles
- Easier to discuss specific changes
- Can merge incrementally (deliver value sooner)
- Simpler rollback if issues found
