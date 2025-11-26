# Tasks: cd Builtin Command

**Input**: Design documents from `/specs/008-cd-builtin/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Unit tests included inline with implementation (Rust convention)

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Rust monorepo**: `crates/rush/src/` for source, `crates/rush/tests/` for tests

---

## Phase 1: Setup

**Purpose**: Project initialization and module structure

- [x] T001 Create cd builtin file at `crates/rush/src/executor/builtins/cd.rs`
- [x] T002 Add `pub mod cd;` to `crates/rush/src/executor/builtins/mod.rs`
- [x] T003 Register `"cd"` in `execute_builtin()` match statement in `crates/rush/src/executor/builtins/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core helper functions that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Implement `expand_tilde(path: &str, home: &str) -> String` helper function in `crates/rush/src/executor/builtins/cd.rs`
- [x] T005 Implement `validate_path(path: &Path) -> Result<(), String>` helper function in `crates/rush/src/executor/builtins/cd.rs`
- [x] T006 Implement `change_directory(target: &str, executor: &mut CommandExecutor) -> Result<i32>` core function in `crates/rush/src/executor/builtins/cd.rs`
- [x] T007 [P] Add unit tests for `expand_tilde()` in `crates/rush/src/executor/builtins/cd.rs`
- [x] T008 [P] Add unit tests for `validate_path()` in `crates/rush/src/executor/builtins/cd.rs`

**Checkpoint**: Foundation ready - helper functions exist and are tested

---

## Phase 3: User Story 1 + 2 - Basic Navigation + Tilde Expansion (Priority: P1) 🎯 MVP

**Goal**: Users can navigate with `cd /path`, `cd relative`, `cd`, `cd ~`, `cd ~/path`

**Independent Test**: Run `cd /tmp` and verify cwd changes. Run `cd ~` and verify home navigation.

### Implementation for User Stories 1 & 2

- [x] T009 [US1] Implement `execute()` function signature matching other builtins in `crates/rush/src/executor/builtins/cd.rs`
- [x] T010 [US1] Handle `cd` with no arguments (change to HOME) in `crates/rush/src/executor/builtins/cd.rs`
- [x] T011 [US1] Handle absolute path argument in `crates/rush/src/executor/builtins/cd.rs`
- [x] T012 [US1] Handle relative path argument in `crates/rush/src/executor/builtins/cd.rs`
- [x] T013 [US2] Integrate tilde expansion for `~` and `~/path` arguments in `crates/rush/src/executor/builtins/cd.rs`
- [x] T014 [US1] Update PWD environment variable after successful cd in `crates/rush/src/executor/builtins/cd.rs`
- [x] T015 [P] [US1] Add unit test for `cd /tmp` (absolute path) in `crates/rush/src/executor/builtins/cd.rs`
- [x] T016 [P] [US1] Add unit test for `cd` with no args (HOME) in `crates/rush/src/executor/builtins/cd.rs`
- [x] T017 [P] [US2] Add unit test for `cd ~` in `crates/rush/src/executor/builtins/cd.rs`
- [x] T018 [P] [US2] Add unit test for `cd ~/subdir` in `crates/rush/src/executor/builtins/cd.rs`
- [x] T019 [US1] Add integration test for basic cd in `crates/rush/tests/feature_test.rs`
- [x] T020 [US1] Verify `cargo test` passes and `cd /tmp && pwd` works in manual testing

**Checkpoint**: User Stories 1 & 2 complete - basic navigation and tilde expansion work

---

## Phase 4: User Story 3 - Previous Directory Navigation (Priority: P2)

**Goal**: Users can use `cd -` to return to previous directory

**Independent Test**: Run `cd /tmp`, `cd /var`, `cd -` and verify return to `/tmp` with path printed.

### Implementation for User Story 3

- [x] T021 [US3] Save current directory to OLDPWD before each successful cd in `crates/rush/src/executor/builtins/cd.rs`
- [x] T022 [US3] Handle `cd -` argument (change to OLDPWD) in `crates/rush/src/executor/builtins/cd.rs`
- [x] T023 [US3] Print new directory when `cd -` is used in `crates/rush/src/executor/builtins/cd.rs`
- [x] T024 [US3] Handle error when OLDPWD not set in `crates/rush/src/executor/builtins/cd.rs`
- [x] T025 [P] [US3] Add unit test for `cd -` basic functionality in `crates/rush/src/executor/builtins/cd.rs`
- [x] T026 [P] [US3] Add unit test for `cd -` when OLDPWD not set in `crates/rush/src/executor/builtins/cd.rs`
- [x] T027 [US3] Add integration test for cd - toggle in `crates/rush/tests/feature_test.rs`

**Checkpoint**: User Story 3 complete - `cd -` works correctly

---

## Phase 5: User Story 4 - CDPATH Search (Priority: P3)

**Goal**: Users can configure CDPATH for quick navigation to project directories

**Independent Test**: Set `CDPATH=/projects`, create `/projects/myapp`, run `cd myapp` and verify navigation.

### Implementation for User Story 4

- [x] T028 [US4] Implement `search_cdpath(name: &str, cdpath: &str) -> Option<PathBuf>` in `crates/rush/src/executor/builtins/cd.rs`
- [x] T029 [US4] Integrate CDPATH search for relative paths not in cwd in `crates/rush/src/executor/builtins/cd.rs`
- [x] T030 [US4] Ensure local directory takes precedence over CDPATH in `crates/rush/src/executor/builtins/cd.rs`
- [x] T031 [US4] Print resolved path when CDPATH match is used in `crates/rush/src/executor/builtins/cd.rs`
- [x] T032 [P] [US4] Add unit test for CDPATH search in `crates/rush/src/executor/builtins/cd.rs`
- [x] T033 [P] [US4] Add unit test for local directory precedence in `crates/rush/src/executor/builtins/cd.rs`
- [x] T034 [US4] Add integration test for CDPATH in `crates/rush/tests/feature_test.rs`

**Checkpoint**: User Story 4 complete - CDPATH search works

---

## Phase 6: Polish & Edge Cases

**Purpose**: Handle edge cases, improve error messages, final validation

- [x] T035 [P] Add test for nonexistent directory error message in `crates/rush/src/executor/builtins/cd.rs`
- [x] T036 [P] Add test for "not a directory" error (file target) in `crates/rush/src/executor/builtins/cd.rs`
- [x] T037 [P] Add test for HOME not set error in `crates/rush/src/executor/builtins/cd.rs`
- [x] T038 Handle empty string argument (treat as `cd`) in `crates/rush/src/executor/builtins/cd.rs`
- [x] T039 Run `cargo clippy` and fix any warnings
- [x] T040 Run `cargo fmt` to ensure consistent formatting
- [x] T041 Run full test suite `cargo test -p rush` and verify all tests pass
- [x] T042 Manual validation: test all acceptance scenarios from spec.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **US1+US2 (Phase 3)**: Depends on Foundational - MVP milestone 🎯
- **US3 (Phase 4)**: Depends on US1+US2 (needs OLDPWD tracking)
- **US4 (Phase 5)**: Depends on Foundational only (can parallel with US3)
- **Polish (Phase 6)**: Depends on all user stories complete

### User Story Dependencies

```
Setup (Phase 1)
    ↓
Foundational (Phase 2) - Helper functions
    ↓
    ├── US1+US2 (Phase 3) - Basic cd + Tilde [P1] 🎯 MVP
    │       ↓
    │       └── US3 (Phase 4) - cd - [P2]
    │
    └── US4 (Phase 5) - CDPATH [P3] (can parallel with US3 after Foundation)
            ↓
        Polish (Phase 6)
```

### Parallel Opportunities

**Phase 2 (Foundational)**:
- T007 and T008 can run in parallel (independent test files)

**Phase 3 (US1+US2)**:
- T015, T016, T017, T018 all parallel (independent tests)

**Phase 4 (US3)**:
- T025, T026 parallel (independent tests)

**Phase 5 (US4)**:
- T032, T033 parallel (independent tests)
- US4 can run in parallel with US3 (after Foundational complete)

**Phase 6 (Polish)**:
- T035, T036, T037 all parallel (independent tests)

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T008) - **CRITICAL**
3. Complete Phase 3: US1+US2 (T009-T020)
4. **STOP and VALIDATE**: Test `cd /tmp`, `cd ~`, `cd ~/Documents`
5. Commit as PR #1: Basic cd + Tilde (~400 lines)

### Incremental Delivery

1. **PR #1**: Setup + Foundational + US1+US2 → **MVP: basic cd works**
2. **PR #2**: US3 (cd -) → **Previous directory navigation**
3. **PR #3**: US4 (CDPATH) → **Power user feature**
4. **PR #4**: Polish → **Edge cases and final validation**

### PR Size Validation

Before each PR:
```bash
git diff --stat main
```

Expected sizes:
- PR #1: ~400 lines (US1+US2 + Foundation)
- PR #2: ~150 lines (US3 cd -)
- PR #3: ~200 lines (US4 CDPATH)
- PR #4: ~100 lines (Polish)

---

## Summary

| Phase | Tasks | Parallel | Story |
|-------|-------|----------|-------|
| Setup | T001-T003 | 0 | - |
| Foundational | T004-T008 | 2 | - |
| US1+US2 | T009-T020 | 4 | MVP 🎯 |
| US3 | T021-T027 | 2 | cd - |
| US4 | T028-T034 | 2 | CDPATH |
| Polish | T035-T042 | 3 | - |
| **Total** | **42 tasks** | **13 parallel** | |

**MVP Scope**: Phases 1-3 (T001-T020) = 20 tasks
**Full Feature**: All 42 tasks
