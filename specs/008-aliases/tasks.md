# Implementation Tasks: Feature 008 - Shell Aliases

**Feature**: Shell Aliases
**Branch**: `008-aliases`
**Status**: Ready for Implementation

## Overview

Implement shell aliases supporting:
- Create: `alias ll='ls -la'`
- View: `alias` (all) or `alias ll` (specific)
- Delete: `unalias ll`
- Expansion: `ll` → `ls -la`
- Persistence: Save/load from ~/.config/rush/aliases

## User Story Summary

| ID | Story | Priority |
|----|-------|----------|
| US1 | Create Simple Aliases | P1 |
| US2 | View Existing Aliases | P1 |
| US3 | Delete Aliases | P1 |
| US4 | Persist Across Sessions | P2 |
| US5 | Aliases with Arguments | P2 |

---

## Phase 1: Core Alias Infrastructure (2-3 hours)

- [ ] T001 [US1] Create `executor/aliases.rs` module with AliasManager struct
- [ ] T002 [US1] Implement AliasManager with HashMap<String, String> storage
- [ ] T003 [US1] Implement add_alias(name, value) method
- [ ] T004 [US2] Implement get_alias(name) -> Option<&str> method
- [ ] T005 [US2] Implement list_aliases() -> Vec<(String, String)> method
- [ ] T006 [US3] Implement remove_alias(name) -> bool method
- [ ] T007 [US5] Implement expand_alias(input) -> String for command expansion
- [ ] T008 [US1] Add circular alias detection (max 10 levels)
- [ ] T009 [US1] Write unit tests for AliasManager

---

## Phase 2: Builtin Commands (1-2 hours)

- [ ] T010 [US1/US2] Create `executor/builtins/alias.rs` module
- [ ] T011 [US1] Parse `alias name='value'` syntax
- [ ] T012 [US1] Handle alias with = but no quotes: `alias name=value`
- [ ] T013 [US2] Implement `alias` with no args to list all aliases
- [ ] T014 [US2] Implement `alias name` to show specific alias
- [ ] T015 [US3] Create `executor/builtins/unalias.rs` module
- [ ] T016 [US3] Implement unalias command to remove alias
- [ ] T017 [US3] Handle error for non-existent alias
- [ ] T018 [US1-US3] Register alias and unalias in builtins/mod.rs
- [ ] T019 [US1-US3] Write unit tests for alias/unalias builtins

---

## Phase 3: Integration & Expansion (1 hour)

- [ ] T020 [US5] Integrate AliasManager into CommandExecutor
- [ ] T021 [US5] Modify execute() to expand aliases before parsing
- [ ] T022 [US5] Handle argument passing (append args to expanded alias)
- [ ] T023 [US5] Test alias expansion with various commands
- [ ] T024 [US1] Ensure aliases take priority over external commands

---

## Phase 4: Persistence (1-2 hours)

- [ ] T025 [US4] Define alias file path: ~/.config/rush/aliases
- [ ] T026 [US4] Implement save_aliases() to write aliases to file
- [ ] T027 [US4] Implement load_aliases() to read aliases from file
- [ ] T028 [US4] Call load_aliases() on shell startup (Repl::new)
- [ ] T029 [US4] Call save_aliases() after alias/unalias commands
- [ ] T030 [US4] Handle missing or corrupted alias file gracefully
- [ ] T031 [US4] Write tests for persistence

---

## Phase 5: Polish (30 min)

- [ ] T032 [US1-US5] Validate alias names (alphanumeric + underscore)
- [ ] T033 [US1-US5] Error handling for invalid alias syntax
- [ ] T034 [US1-US5] Run full test suite
- [ ] T035 [US1-US5] Run clippy

---

**Total Tasks**: 35
**Estimated Duration**: 4-6 hours

## MVP Scope

**Minimum Viable (T001-T024)**: In-memory aliases without persistence
- alias/unalias commands
- Alias expansion
- ~3-4 hours

**Full Feature (T001-T035)**: Complete with persistence
- All MVP plus save/load
- ~4-6 hours
