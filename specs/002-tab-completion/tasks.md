# Tasks: Tab Completion

**Input**: Design documents from `/specs/002-tab-completion/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included as they align with constitution testing philosophy and Rust best practices.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

This is a single Rust project in a monorepo workspace. All paths relative to repository root:
- Source: `crates/rush/src/`
- Tests: `crates/rush/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create completion module structure and testing infrastructure

- [X] T001 Create completion module directory at crates/rush/src/completion/
- [X] T002 [P] Create unit test directory at crates/rush/tests/unit/completion/
- [X] T003 [P] Create integration test file at crates/rush/tests/integration/completion_tests.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core completion infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Create completion module exports in crates/rush/src/completion/mod.rs
- [X] T005 Re-export reedline types (Completer, Suggestion, Span) in crates/rush/src/completion/mod.rs
- [X] T006 Create CompletionRegistry struct skeleton in crates/rush/src/completion/mod.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Command Name Completion (Priority: P1) 🎯 MVP

**Goal**: Enable command name auto-completion from PATH executables when user presses Tab

**Independent Test**: Type `gi<TAB>` in rush and verify it completes to `git`. Type `ca<TAB>` and verify menu shows cat, cargo, cal. This works standalone without path or flag completion.

### Implementation for User Story 1

- [ ] T007 [P] [US1] Create CommandCompleter struct in crates/rush/src/completion/command.rs with cache field
- [ ] T008 [P] [US1] Implement CommandCompleter::new() constructor with platform-specific case sensitivity in crates/rush/src/completion/command.rs
- [ ] T009 [US1] Implement scan_path() method to enumerate PATH directories and find executables in crates/rush/src/completion/command.rs
- [ ] T010 [US1] Implement ensure_cache_loaded() method for lazy cache initialization in crates/rush/src/completion/command.rs
- [ ] T011 [US1] Implement prefix matching logic with case-sensitive/insensitive support in crates/rush/src/completion/command.rs
- [ ] T012 [US1] Implement Completer trait for CommandCompleter with complete() method in crates/rush/src/completion/command.rs
- [ ] T013 [US1] Add logic to limit completions to 50 items and return "too many matches" message in crates/rush/src/completion/command.rs
- [ ] T014 [US1] Export CommandCompleter from crates/rush/src/completion/mod.rs
- [ ] T015 [US1] Integrate CommandCompleter with REPL in crates/rush/src/repl/mod.rs using Reedline::with_completer()
- [ ] T016 [P] [US1] Write unit tests for single match scenario in crates/rush/tests/unit/completion/command_tests.rs
- [ ] T017 [P] [US1] Write unit tests for multiple matches scenario in crates/rush/tests/unit/completion/command_tests.rs
- [ ] T018 [P] [US1] Write unit tests for no matches scenario in crates/rush/tests/unit/completion/command_tests.rs
- [ ] T019 [P] [US1] Write unit tests for too many matches (>50) scenario in crates/rush/tests/unit/completion/command_tests.rs
- [ ] T020 [P] [US1] Write unit tests for case-insensitive matching on macOS in crates/rush/tests/unit/completion/command_tests.rs
- [ ] T021 [US1] Write integration test for command completion in REPL in crates/rush/tests/integration/completion_tests.rs

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Command completion works in rush shell.

---

## Phase 4: User Story 2 - File and Directory Path Completion (Priority: P2)

**Goal**: Enable file and directory path auto-completion when user presses Tab on file arguments

**Independent Test**: Type `ls src/r<TAB>` and verify it completes to `src/repl/`. Type `cat README<TAB>` and verify it completes to `README.md`. This works independently of command completion.

### Implementation for User Story 2

- [ ] T022 [P] [US2] Create PathCompleter struct in crates/rush/src/completion/path.rs with case_sensitive field
- [ ] T023 [P] [US2] Implement PathCompleter::new() constructor with platform-specific case sensitivity in crates/rush/src/completion/path.rs
- [ ] T024 [US2] Implement extract_partial_path() helper to parse cursor position and extract path fragment in crates/rush/src/completion/path.rs
- [ ] T025 [US2] Implement split_path_and_prefix() helper to separate parent directory from prefix in crates/rush/src/completion/path.rs
- [ ] T026 [US2] Implement list_directory_entries() method to scan filesystem and return matching entries in crates/rush/src/completion/path.rs
- [ ] T027 [US2] Add logic to show hidden files only if prefix starts with '.' in crates/rush/src/completion/path.rs
- [ ] T028 [US2] Add logic to append '/' to directories and quote paths with spaces in crates/rush/src/completion/path.rs
- [ ] T029 [US2] Implement Completer trait for PathCompleter with complete() method in crates/rush/src/completion/path.rs
- [ ] T030 [US2] Add logic to handle tilde expansion (~/) in crates/rush/src/completion/path.rs
- [ ] T031 [US2] Add logic to handle absolute paths (starting with /) in crates/rush/src/completion/path.rs
- [ ] T032 [US2] Add logic to limit path completions to 50 items in crates/rush/src/completion/path.rs
- [ ] T033 [US2] Export PathCompleter from crates/rush/src/completion/mod.rs
- [ ] T034 [US2] Create CompletionContext struct to parse line and determine completion type in crates/rush/src/completion/mod.rs
- [ ] T035 [US2] Update CompletionRegistry to route to CommandCompleter or PathCompleter based on context in crates/rush/src/completion/mod.rs
- [ ] T036 [US2] Update REPL integration to use CompletionRegistry instead of CommandCompleter in crates/rush/src/repl/mod.rs
- [ ] T037 [P] [US2] Write unit tests for directory completion in crates/rush/tests/unit/completion/path_tests.rs
- [ ] T038 [P] [US2] Write unit tests for file completion in crates/rush/tests/unit/completion/path_tests.rs
- [ ] T039 [P] [US2] Write unit tests for hidden file completion in crates/rush/tests/unit/completion/path_tests.rs
- [ ] T040 [P] [US2] Write unit tests for paths with spaces (quoting) in crates/rush/tests/unit/completion/path_tests.rs
- [ ] T041 [P] [US2] Write unit tests for tilde expansion in crates/rush/tests/unit/completion/path_tests.rs
- [ ] T042 [P] [US2] Write unit tests for absolute paths in crates/rush/tests/unit/completion/path_tests.rs
- [ ] T043 [US2] Write integration test for path completion in REPL in crates/rush/tests/integration/completion_tests.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Users can complete both commands and paths.

---

## Phase 5: User Story 3 - Flag Completion for Common Commands (Priority: P3)

**Goal**: Enable flag/option auto-completion for common commands (git, cargo, ls) when user presses Tab on flags

**Independent Test**: Type `git --<TAB>` and verify menu shows git flags with descriptions. Type `ls -<TAB>` and verify menu shows ls flags. This works independently of command/path completion.

### Implementation for User Story 3

- [ ] T044 [P] [US3] Create FlagDefinition struct in crates/rush/src/completion/flag.rs
- [ ] T045 [P] [US3] Create FlagCompleter struct in crates/rush/src/completion/flag.rs
- [ ] T046 [US3] Add lazy_static dependency to Cargo.toml for static flag registry
- [ ] T047 [US3] Create FLAG_REGISTRY with lazy_static in crates/rush/src/completion/flag.rs
- [ ] T048 [US3] Add git flags to FLAG_REGISTRY (--version, --help, --verbose, etc.) in crates/rush/src/completion/flag.rs
- [ ] T049 [US3] Add cargo flags to FLAG_REGISTRY (--version, --help, --verbose, etc.) in crates/rush/src/completion/flag.rs
- [ ] T050 [US3] Add ls flags to FLAG_REGISTRY (-l, -a, -h, -r, -t, etc.) in crates/rush/src/completion/flag.rs
- [ ] T051 [US3] Add cd, cat, echo, grep, find flags to FLAG_REGISTRY in crates/rush/src/completion/flag.rs
- [ ] T052 [US3] Implement FlagCompleter::new() constructor in crates/rush/src/completion/flag.rs
- [ ] T053 [US3] Implement extract_command_and_flag() helper to parse command name and partial flag in crates/rush/src/completion/flag.rs
- [ ] T054 [US3] Implement flag matching logic (both long and short flags) in crates/rush/src/completion/flag.rs
- [ ] T055 [US3] Implement Completer trait for FlagCompleter with complete() method in crates/rush/src/completion/flag.rs
- [ ] T056 [US3] Add logic to include descriptions in Suggestion struct in crates/rush/src/completion/flag.rs
- [ ] T057 [US3] Add logic to include short alternatives in Suggestion.extra field in crates/rush/src/completion/flag.rs
- [ ] T058 [US3] Export FlagCompleter from crates/rush/src/completion/mod.rs
- [ ] T059 [US3] Update CompletionContext to detect flag completion (tokens starting with -) in crates/rush/src/completion/mod.rs
- [ ] T060 [US3] Update CompletionRegistry to route to FlagCompleter when appropriate in crates/rush/src/completion/mod.rs
- [ ] T061 [P] [US3] Write unit tests for git flag completion in crates/rush/tests/unit/completion/flag_tests.rs
- [ ] T062 [P] [US3] Write unit tests for cargo flag completion in crates/rush/tests/unit/completion/flag_tests.rs
- [ ] T063 [P] [US3] Write unit tests for ls flag completion in crates/rush/tests/unit/completion/flag_tests.rs
- [ ] T064 [P] [US3] Write unit tests for unknown command (no flags) in crates/rush/tests/unit/completion/flag_tests.rs
- [ ] T065 [P] [US3] Write unit tests for short flag matching in crates/rush/tests/unit/completion/flag_tests.rs
- [ ] T066 [US3] Write integration test for flag completion in REPL in crates/rush/tests/integration/completion_tests.rs

**Checkpoint**: All user stories should now be independently functional. Users have complete tab completion for commands, paths, and flags.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and overall quality

- [ ] T067 [P] Add tracing/logging for completion performance metrics in crates/rush/src/completion/mod.rs
- [ ] T068 [P] Add error handling for permission denied in PATH scanning in crates/rush/src/completion/command.rs
- [ ] T069 [P] Add error handling for permission denied in directory scanning in crates/rush/src/completion/path.rs
- [ ] T070 [P] Optimize PATH cache memory usage in crates/rush/src/completion/command.rs
- [ ] T071 [P] Add documentation comments to all public APIs in crates/rush/src/completion/
- [ ] T072 [P] Update README.md to mention tab completion feature in crates/rush/README.md
- [ ] T073 [P] Update KNOWN_ISSUES.md to remove tab completion from planned features in crates/rush/KNOWN_ISSUES.md
- [ ] T074 Run cargo clippy and address warnings in completion module
- [ ] T075 Run cargo fmt on completion module
- [ ] T076 Run performance benchmarks per quickstart.md and verify <100ms target
- [ ] T077 Manual testing checklist from quickstart.md (commands, paths, flags, edge cases)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Requires CompletionRegistry refactor but is independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Requires CompletionRegistry but is independently testable

### Within Each User Story

- Unit tests can run in parallel (marked with [P])
- Implementation tasks follow logical order (struct → methods → trait impl → integration)
- Integration tests come last after REPL integration
- Each story checkpoint validates independence

### Parallel Opportunities

**Setup Phase**:
- T002 and T003 can run in parallel (different directories)

**User Story 1 (within story)**:
- T007 and T008 can run in parallel (initial setup)
- T016, T017, T018, T019, T020 can run in parallel (all unit tests, different test functions)

**User Story 2 (within story)**:
- T022 and T023 can run in parallel (initial setup)
- T037-T042 can run in parallel (all unit tests, different test functions)

**User Story 3 (within story)**:
- T044 and T045 can run in parallel (struct definitions)
- T048-T051 can run in parallel (adding different command flags to registry)
- T061-T065 can run in parallel (all unit tests, different test functions)

**Polish Phase**:
- T067, T068, T069, T070, T071, T072, T073 can all run in parallel (different files)

**User Stories (if multiple developers)**:
- After Phase 2 completes, User Stories 1, 2, and 3 can be developed in parallel by different developers
- NOTE: US2 and US3 both modify CompletionRegistry, so coordination needed for those specific tasks

---

## Parallel Example: User Story 1

```bash
# Launch all unit tests for User Story 1 together:
Task: T016 - "Write unit tests for single match scenario"
Task: T017 - "Write unit tests for multiple matches scenario"
Task: T018 - "Write unit tests for no matches scenario"
Task: T019 - "Write unit tests for too many matches scenario"
Task: T020 - "Write unit tests for case-insensitive matching"

# These can all be written simultaneously in different test functions
```

---

## Parallel Example: User Story 2

```bash
# Launch all unit tests for User Story 2 together:
Task: T037 - "Write unit tests for directory completion"
Task: T038 - "Write unit tests for file completion"
Task: T039 - "Write unit tests for hidden file completion"
Task: T040 - "Write unit tests for paths with spaces"
Task: T041 - "Write unit tests for tilde expansion"
Task: T042 - "Write unit tests for absolute paths"

# These can all be written simultaneously in different test functions
```

---

## Parallel Example: User Story 3

```bash
# Add flag definitions in parallel:
Task: T048 - "Add git flags to FLAG_REGISTRY"
Task: T049 - "Add cargo flags to FLAG_REGISTRY"
Task: T050 - "Add ls flags to FLAG_REGISTRY"
Task: T051 - "Add cd, cat, echo, grep, find flags"

# Launch all unit tests together:
Task: T061 - "Write unit tests for git flag completion"
Task: T062 - "Write unit tests for cargo flag completion"
Task: T063 - "Write unit tests for ls flag completion"
Task: T064 - "Write unit tests for unknown command"
Task: T065 - "Write unit tests for short flag matching"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T006)
3. Complete Phase 3: User Story 1 (T007-T021)
4. **STOP and VALIDATE**:
   - Run `cargo test -p rush completion::command`
   - Run `cargo build -p rush --release`
   - Test manually: `./target/release/rush`, type `gi<TAB>`, verify completion
5. Deploy/demo if ready - **Users now have command completion!**

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → **Deploy/Demo (MVP!)**
   - Users can now complete command names
3. Add User Story 2 → Test independently → **Deploy/Demo**
   - Users can now complete commands AND paths
4. Add User Story 3 → Test independently → **Deploy/Demo**
   - Users have full completion: commands, paths, and flags
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T006)
2. Once Foundational is done:
   - Developer A: User Story 1 (T007-T021) - Command completion
   - Developer B: User Story 2 (T022-T043) - Path completion
   - Developer C: User Story 3 (T044-T066) - Flag completion
3. Coordination needed: US2 and US3 both modify CompletionRegistry
   - Solution: US2 creates CompletionContext and routing, US3 adds flag route
   - Or: Do US1 + US2 first, then US3 (sequential)

### Recommended Approach for Single Developer

Priority order for maximum value:

1. **Phase 1-2**: Setup + Foundational (T001-T006) - ~30 minutes
2. **Phase 3 (US1)**: Command completion (T007-T021) - ~3-4 hours
   - **Checkpoint: MVP complete** - ship it!
3. **Phase 4 (US2)**: Path completion (T022-T043) - ~4-5 hours
   - **Checkpoint: v0.2 ready** - ship it!
4. **Phase 5 (US3)**: Flag completion (T044-T066) - ~3-4 hours
   - **Checkpoint: Full feature complete** - ship it!
5. **Phase 6**: Polish (T067-T077) - ~2 hours

**Total estimated time**: 12-15 hours for all three user stories

---

## Task Count Summary

- **Phase 1 (Setup)**: 3 tasks
- **Phase 2 (Foundational)**: 3 tasks
- **Phase 3 (US1 - Command Completion)**: 15 tasks
- **Phase 4 (US2 - Path Completion)**: 22 tasks
- **Phase 5 (US3 - Flag Completion)**: 23 tasks
- **Phase 6 (Polish)**: 11 tasks

**Total**: 77 tasks

**Parallel opportunities**:
- 22 tasks marked [P] can run in parallel with other tasks
- All 3 user stories can be developed in parallel after foundational phase

---

## Notes

- [P] tasks = different files, no dependencies - can run in parallel
- [Story] label (US1/US2/US3) maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Stop at any checkpoint to validate story independently before continuing
- MVP = Just User Story 1 (command completion) - provides immediate value
- Recommended: Ship US1 first, get user feedback, then add US2 and US3
- Constitution compliance verified in plan.md - all 5 principles aligned

---

## Validation Checklist

✅ All tasks follow format: `- [ ] [ID] [P?] [Story?] Description with file path`
✅ Tasks organized by user story (Phase 3, 4, 5 = US1, US2, US3)
✅ Each user story has independent test criteria
✅ Dependencies clearly documented
✅ Parallel opportunities identified ([P] markers)
✅ MVP scope defined (User Story 1)
✅ File paths are absolute and specific
✅ No vague tasks - all actionable
✅ Tests included per Rust/constitution best practices
✅ Incremental delivery strategy documented
