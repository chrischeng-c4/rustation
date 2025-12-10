# Tasks: Tilde Expansion

**Input**: Design documents from `/specs/035-tilde-expansion/`
**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Comprehensive unit tests are included as part of this implementation (following existing pattern in rush shell).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Single Rust project workspace: `crates/rush/src/` at repository root
- Tests: Inline in module files following existing pattern

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic module structure

- [ ] T001 Create module file crates/rush/src/executor/tilde.rs with module structure and public API stub
- [ ] T002 Add tilde module declaration in crates/rush/src/executor/mod.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Implement quote/escape state machine in expand_tilde() function in crates/rush/src/executor/tilde.rs
- [ ] T004 Implement word-boundary detection and word accumulation in expand_tilde() in crates/rush/src/executor/tilde.rs
- [ ] T005 Implement parse_tilde_prefix() helper to extract tilde prefix and remainder in crates/rush/src/executor/tilde.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Home Directory Shortcut (Priority: P1) 🎯 MVP

**Goal**: Users can use `~` as a shortcut to their home directory in commands

**Independent Test**: Run `echo ~` in the shell and verify it prints the user's home directory path. Run `cd ~` and verify the shell changes to the home directory.

### Implementation for User Story 1

- [ ] T006 [US1] Implement get_home_dir() helper to retrieve HOME environment variable in crates/rush/src/executor/tilde.rs
- [ ] T007 [US1] Implement expand_tilde_prefix() to handle basic `~` expansion in crates/rush/src/executor/tilde.rs
- [ ] T008 [US1] Implement expand_word() to process individual words and call expand_tilde_prefix() in crates/rush/src/executor/tilde.rs
- [ ] T009 [US1] Add unit tests for basic `~` expansion (test_basic_tilde, test_tilde_with_path) in crates/rush/src/executor/tilde.rs
- [ ] T010 [US1] Add unit tests for quote/escape handling (test_single_quotes, test_double_quotes, test_escaped_tilde) in crates/rush/src/executor/tilde.rs
- [ ] T011 [US1] Add unit tests for edge cases (test_missing_home, test_multiple_tildes, test_tilde_mid_word) in crates/rush/src/executor/tilde.rs

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Working Directory Shortcuts (Priority: P2)

**Goal**: Users can use `~+` for current directory and `~-` for previous directory

**Independent Test**: Change directories twice, then run `echo ~+` (should show current PWD) and `echo ~-` (should show previous OLDPWD).

### Implementation for User Story 2

- [ ] T012 [US2] Extend expand_tilde_prefix() to handle `~+` expansion using PWD environment variable in crates/rush/src/executor/tilde.rs
- [ ] T013 [US2] Extend expand_tilde_prefix() to handle `~-` expansion using OLDPWD environment variable in crates/rush/src/executor/tilde.rs
- [ ] T014 [US2] Add unit tests for `~+` expansion (test_tilde_plus, test_tilde_plus_with_path) in crates/rush/src/executor/tilde.rs
- [ ] T015 [US2] Add unit tests for `~-` expansion (test_tilde_minus, test_tilde_minus_with_path, test_missing_oldpwd) in crates/rush/src/executor/tilde.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Other User's Home Directory (Priority: P3)

**Goal**: Users can use `~username` to reference another user's home directory

**Independent Test**: Run `echo ~root` and verify it expands to root user's home directory path.

### Implementation for User Story 3

- [ ] T016 [US3] Add nix crate dependency to Cargo.toml for safe user database lookup (optional, can defer if size is concern)
- [ ] T017 [US3] Implement get_user_home() helper to lookup user home directory using nix::unistd::User::from_name() in crates/rush/src/executor/tilde.rs
- [ ] T018 [US3] Extend expand_tilde_prefix() to handle `~username` pattern by parsing username and calling get_user_home() in crates/rush/src/executor/tilde.rs
- [ ] T019 [US3] Add unit tests for `~username` expansion (test_tilde_user, test_tilde_user_with_path, test_nonexistent_user) in crates/rush/src/executor/tilde.rs

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Integration

**Purpose**: Integrate tilde expansion into the shell's expansion pipeline

- [ ] T020 Modify crates/rush/src/executor/execute.rs to call expand_tilde() after brace expansion (insert at line ~141)
- [ ] T021 Update execute.rs to pass tilded_line to expand_variables() instead of braced_line
- [ ] T022 Run full test suite with `cargo test` to verify integration
- [ ] T023 Fix any integration issues or test failures

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements and validation

- [ ] T024 Manual testing with interactive shell - test all user scenarios from spec.md
- [ ] T025 Manual testing of edge cases (empty HOME, special characters in paths, etc.)
- [ ] T026 Performance validation - verify expansion completes in <1ms
- [ ] T027 [P] Code cleanup and ensure consistent error handling
- [ ] T028 [P] Add documentation comments to public functions in tilde.rs
- [ ] T029 Update specs/features.json to mark feature 035 as "complete"
- [ ] T030 Bump version to v0.35.0 in Cargo.toml

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3, 4, 5)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if desired)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Integration (Phase 6)**: Depends on all desired user stories being complete
- **Polish (Phase 7)**: Depends on integration being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Independent of US1, but builds on same infrastructure
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Independent of US1/US2, but builds on same infrastructure

### Within Each User Story

- Core implementation before tests (though tests can be written first in TDD style)
- All tests for a story can run in parallel after implementation

### Parallel Opportunities

- All Setup tasks (T001, T002) can run in parallel
- All Foundational tasks (T003, T004, T005) can run in sequence (they modify the same file)
- Once Foundational phase completes:
  - US1 implementation (T006-T011) can be done as a unit
  - US2 implementation (T012-T015) can be done independently after US1
  - US3 implementation (T016-T019) can be done independently after US2
- All Polish tasks marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# After foundation is complete, implement US1 core logic:
# T006: get_home_dir() helper
# T007: expand_tilde_prefix() for basic ~
# T008: expand_word() to process words

# Then write all US1 tests together:
# T009: basic expansion tests
# T010: quote/escape tests
# T011: edge case tests
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T005) - CRITICAL
3. Complete Phase 3: User Story 1 (T006-T011)
4. Complete Phase 6: Integration (T020-T023)
5. Complete Phase 7: Polish (T024-T030)
6. **STOP and VALIDATE**: Test basic `~` expansion independently
7. Deploy/merge if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Can merge to main (MVP!)
3. Add User Story 2 → Test independently → Can merge to main
4. Add User Story 3 → Test independently → Can merge to main
5. Each story adds value without breaking previous stories

### Single PR Strategy (Recommended for this feature)

Since the entire feature is ~400 lines total (well under 500 line ideal limit), implement all user stories together in a single PR:

1. Complete Phase 1-7 in sequence
2. All three user stories (P1, P2, P3) in one implementation
3. Create single PR with complete feature
4. Merge to main after review

**Rationale**: Feature is cohesive and small, all stories share the same infrastructure, breaking into multiple PRs would create unnecessary intermediate states.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Follow existing patterns from expansion.rs and glob.rs

### Pull Request Strategy

**Single PR Approach** (Recommended):

Given the small size (~405 lines total), create one PR containing the complete feature:

**PR #1: Implement tilde expansion (035-tilde-expansion → main)**
- Add crates/rush/src/executor/tilde.rs (~300 lines)
- Modify crates/rush/src/executor/execute.rs (+2 lines)
- Modify crates/rush/src/executor/mod.rs (+1 line)
- Tests: Comprehensive unit tests in tilde.rs (~100 lines)
- Total: ~403 lines ✅ (within 500 line ideal limit)

**Before Creating PR**:
- Check line count: `git diff --stat main`
- Expected: ~305 insertions (within ideal 500 line limit)
- Commit message: `feat(035): implement tilde expansion`

**Benefits**:
- Single cohesive review
- All three user stories work together
- No intermediate states with partial functionality
- Simpler merge and rollback if needed
