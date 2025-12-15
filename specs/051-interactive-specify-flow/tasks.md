---
description: "Task list for Interactive Specify Flow implementation"
---

# Tasks: Interactive Specify Flow

**Input**: Design documents from `/specs/051-interactive-specify-flow/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), data-model.md (available)

**Tests**: Tests are NOT explicitly requested in the specification, so test tasks are OPTIONAL and minimal (following existing test patterns from feature 050).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story as defined in the PR strategy (plan.md).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project** (monorepo): `crates/rstn/src/tui/` for TUI code
- All paths are relative to repository root

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Basic structure for specify workflow state

- [ ] T001 Read and understand feature 050 (Commit Review) implementation patterns in crates/rstn/src/tui/
- [ ] T002 Review existing ContentType enum and WorktreeView structure in crates/rstn/src/tui/views/

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and structures that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Add SpecifyState structure to crates/rstn/src/tui/views/worktree.rs (per data-model.md)
- [ ] T004 [P] Add ContentType::SpecifyInput variant to crates/rstn/src/tui/views/mod.rs
- [ ] T005 [P] Add ContentType::SpecifyReview variant to crates/rstn/src/tui/views/mod.rs
- [ ] T006 [P] Add Event::SpecifyGenerationStarted to crates/rstn/src/tui/event.rs
- [ ] T007 [P] Add Event::SpecifyGenerationCompleted to crates/rstn/src/tui/event.rs
- [ ] T008 [P] Add Event::SpecifyGenerationFailed to crates/rstn/src/tui/event.rs
- [ ] T009 [P] Add Event::SpecifySaved to crates/rstn/src/tui/event.rs
- [ ] T010 [P] Add ViewAction::GenerateSpec to action types (app.rs or actions module)
- [ ] T011 [P] Add ViewAction::SaveSpec to action types (app.rs or actions module)
- [ ] T012 Add specify_state field to WorktreeView struct in crates/rstn/src/tui/views/worktree.rs
- [ ] T013 Initialize specify_state in WorktreeView::new() in crates/rstn/src/tui/views/worktree.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Stay in TUI while creating specs (Priority: P1) 🎯 MVP

**Goal**: Users can input feature descriptions directly in the TUI and trigger spec generation without leaving the interface

**Independent Test**: Trigger "Specify" action, enter a description, press Enter - generation starts without shelling out or leaving TUI. Cancel with Esc returns to normal view.

**PR Target**: ~600 lines (foundation + P1 story) - Deliverable: Context-switching eliminated

### Implementation for User Story 1

- [ ] T014 [US1] Add "Specify" action to Commands pane list in crates/rstn/src/tui/views/worktree.rs
- [ ] T015 [US1] Implement start_specify_input() method in crates/rstn/src/tui/views/worktree.rs
- [ ] T016 [US1] Implement cancel_specify() method in crates/rstn/src/tui/views/worktree.rs
- [ ] T017 [US1] Implement handle_specify_input() for text input in crates/rstn/src/tui/views/worktree.rs
- [ ] T018 [US1] Add input validation (validate_input) to SpecifyState in crates/rstn/src/tui/views/worktree.rs
- [ ] T019 [US1] Implement submit_specify_description() method in crates/rstn/src/tui/views/worktree.rs
- [ ] T020 [US1] Add render_specify_input() for input dialog in crates/rstn/src/tui/views/worktree.rs
- [ ] T021 [US1] Update render_content() to handle ContentType::SpecifyInput in crates/rstn/src/tui/views/worktree.rs
- [ ] T022 [US1] Update handle_key_event() to route SpecifyInput keys in crates/rstn/src/tui/views/worktree.rs
- [ ] T023 [US1] Handle input mode key bindings (char, Backspace, Delete, arrows, Home, End, Ctrl+Enter, Enter, Esc) in crates/rstn/src/tui/views/worktree.rs
- [ ] T024 [US1] Implement execute_spec_generation() async function in crates/rstn/src/tui/app.rs
- [ ] T025 [US1] Add shell script integration (tokio::process::Command for create-new-feature.sh) in crates/rstn/src/tui/app.rs
- [ ] T026 [US1] Handle ViewAction::GenerateSpec in app event loop in crates/rstn/src/tui/app.rs
- [ ] T027 [US1] Handle Event::SpecifyGenerationStarted in app event loop in crates/rstn/src/tui/app.rs
- [ ] T028 [US1] Update status bar for "Generating spec..." during generation in crates/rstn/src/tui/views/worktree.rs
- [ ] T029 [US1] Add error handling for empty/whitespace-only input in crates/rstn/src/tui/views/worktree.rs
- [ ] T030 [US1] Add visual feedback for validation errors in render_specify_input() in crates/rstn/src/tui/views/worktree.rs
- [ ] T031 [P] [US1] Add basic unit test for SpecifyState creation and validation in tests/unit/tui_specify_tests.rs
- [ ] T032 [P] [US1] Add unit test for start_specify_input() and cancel_specify() transitions in tests/unit/tui_specify_tests.rs

**Checkpoint**: At this point, User Story 1 should be fully functional - users can input descriptions and trigger generation without leaving TUI

---

## Phase 4: User Story 2 - Review generated specs before saving (Priority: P2)

**Goal**: Users can see the generated spec in the Content area, verify it meets needs, and decide to accept, edit, or discard

**Independent Test**: After generation completes, review screen appears with full spec content. Press Enter to save (spec file created), or Esc to discard (no file created, return to normal view).

**PR Target**: ~400 lines - Deliverable: Quality control prevents bad specs from being saved

### Implementation for User Story 2

- [ ] T033 [US2] Implement load_generated_spec() method in crates/rstn/src/tui/views/worktree.rs
- [ ] T034 [US2] Add render_specify_review() for review mode display in crates/rstn/src/tui/views/worktree.rs
- [ ] T035 [US2] Update render_content() to handle ContentType::SpecifyReview in crates/rstn/src/tui/views/worktree.rs
- [ ] T036 [US2] Implement save_specify_spec() method in crates/rstn/src/tui/views/worktree.rs
- [ ] T037 [US2] Handle review mode key bindings (Enter to save, Esc to cancel) in crates/rstn/src/tui/views/worktree.rs
- [ ] T038 [US2] Display feature number and title at top of review in render_specify_review() in crates/rstn/src/tui/views/worktree.rs
- [ ] T039 [US2] Display action hints ([Enter] Save, [e] Edit, [Esc] Cancel) in render_specify_review() in crates/rstn/src/tui/views/worktree.rs
- [ ] T040 [US2] Parse shell script JSON output to extract feature number and name in crates/rstn/src/tui/app.rs
- [ ] T041 [US2] Read generated spec file content in execute_spec_generation() in crates/rstn/src/tui/app.rs
- [ ] T042 [US2] Handle Event::SpecifyGenerationCompleted in app event loop in crates/rstn/src/tui/app.rs
- [ ] T043 [US2] Handle Event::SpecifyGenerationFailed with error display in crates/rstn/src/tui/app.rs
- [ ] T044 [US2] Implement file write logic in ViewAction::SaveSpec handler in crates/rstn/src/tui/app.rs
- [ ] T045 [US2] Handle Event::SpecifySaved to show success message in crates/rstn/src/tui/app.rs
- [ ] T046 [US2] Update status bar for "Review spec" mode in crates/rstn/src/tui/views/worktree.rs
- [ ] T047 [US2] Clean up specify_state after successful save in crates/rstn/src/tui/views/worktree.rs
- [ ] T048 [US2] Load newly created spec in Content area after save in crates/rstn/src/tui/views/worktree.rs
- [ ] T049 [US2] Add timeout handling (60s) for generation in crates/rstn/src/tui/app.rs
- [ ] T050 [P] [US2] Add unit test for load_generated_spec() and review transitions in tests/unit/tui_specify_tests.rs
- [ ] T051 [P] [US2] Add unit test for save_specify_spec() workflow in tests/unit/tui_specify_tests.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently - full input→generate→review→save workflow complete

---

## Phase 5: User Story 3 - Edit specs inline before saving (Priority: P3)

**Goal**: Users can edit the generated spec inline before saving, eliminating the roundtrip to external editors for minor tweaks

**Independent Test**: From review screen, press 'e' to enter edit mode. Make changes with arrow keys, Home, End. Press Ctrl+S to save (modified spec written), or Esc to cancel edits (return to review with changes discarded).

**PR Target**: ~500 lines - Deliverable: Iteration speed improved for quick adjustments

### Implementation for User Story 3

- [ ] T052 [US3] Add edit_mode flag to SpecifyState in crates/rstn/src/tui/views/worktree.rs
- [ ] T053 [US3] Add edit_cursor and edit_scroll_offset to SpecifyState in crates/rstn/src/tui/views/worktree.rs
- [ ] T054 [US3] Implement toggle_specify_edit_mode() method in crates/rstn/src/tui/views/worktree.rs
- [ ] T055 [US3] Add render_specify_edit() for edit mode display in crates/rstn/src/tui/views/worktree.rs
- [ ] T056 [US3] Show edit mode indicator in title/status in render_specify_edit() in crates/rstn/src/tui/views/worktree.rs
- [ ] T057 [US3] Handle 'e' key in review mode to toggle edit in crates/rstn/src/tui/views/worktree.rs
- [ ] T058 [US3] Handle edit mode key bindings (char input, Backspace, Delete) in crates/rstn/src/tui/views/worktree.rs
- [ ] T059 [US3] Handle cursor movement keys (arrows, Home, End) in edit mode in crates/rstn/src/tui/views/worktree.rs
- [ ] T060 [US3] Handle Ctrl+S to save from edit mode in crates/rstn/src/tui/views/worktree.rs
- [ ] T061 [US3] Handle Enter to save from edit mode in crates/rstn/src/tui/views/worktree.rs
- [ ] T062 [US3] Handle Esc to cancel edit (return to review) in crates/rstn/src/tui/views/worktree.rs
- [ ] T063 [US3] Update render_content() to handle edit mode rendering in crates/rstn/src/tui/views/worktree.rs
- [ ] T064 [US3] Update status bar for "Editing spec" mode in crates/rstn/src/tui/views/worktree.rs
- [ ] T065 [US3] Preserve cursor position during edit operations in crates/rstn/src/tui/views/worktree.rs
- [ ] T066 [US3] Add scroll support for long specs in edit mode in crates/rstn/src/tui/views/worktree.rs
- [ ] T067 [P] [US3] Add unit test for toggle_specify_edit_mode() in tests/unit/tui_specify_tests.rs
- [ ] T068 [P] [US3] Add unit test for edit mode cursor management in tests/unit/tui_specify_tests.rs

**Checkpoint**: All user stories should now be independently functional - complete specify workflow with inline editing

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Production-ready improvements and comprehensive testing

**PR Target**: ~300 lines - Deliverable: Robust error handling and integration testing

- [ ] T069 [P] Enhance error messages for generation failures in crates/rstn/src/tui/app.rs
- [ ] T070 [P] Add retry capability after generation failure in crates/rstn/src/tui/views/worktree.rs
- [ ] T071 [P] Improve validation error messages (show inline in UI) in crates/rstn/src/tui/views/worktree.rs
- [ ] T072 Add comprehensive status bar updates for all modes in crates/rstn/src/tui/views/worktree.rs
- [ ] T073 Add visual transitions between modes (clear feedback) in crates/rstn/src/tui/views/worktree.rs
- [ ] T074 Handle edge case: very long descriptions (>1000 chars) with scrolling in crates/rstn/src/tui/views/worktree.rs
- [ ] T075 Handle edge case: special characters in description (quotes, newlines, unicode) in crates/rstn/src/tui/app.rs
- [ ] T076 Add error handling for missing shell script in crates/rstn/src/tui/app.rs
- [ ] T077 Add error handling for write permission failures in crates/rstn/src/tui/app.rs
- [ ] T078 Add cleanup on timeout (cancel specify state) in crates/rstn/src/tui/views/worktree.rs
- [ ] T079 [P] Add integration test for full workflow (input→generate→review→save) in tests/integration/specify_workflow_test.rs
- [ ] T080 [P] Add integration test for full workflow with edit mode in tests/integration/specify_workflow_test.rs
- [ ] T081 [P] Add integration test for error scenarios (empty input, generation failure) in tests/integration/specify_workflow_test.rs
- [ ] T082 [P] Add integration test for cancel at each stage in tests/integration/specify_workflow_test.rs
- [ ] T083 Manual testing checklist: Trigger from Commands, multi-line input, long descriptions in specs/051-interactive-specify-flow/checklists/manual-test.md
- [ ] T084 Manual testing checklist: Review and edit workflow in specs/051-interactive-specify-flow/checklists/manual-test.md
- [ ] T085 Manual testing checklist: Error scenarios (missing script, permissions) in specs/051-interactive-specify-flow/checklists/manual-test.md
- [ ] T086 Code cleanup and refactoring for consistency with feature 050 patterns
- [ ] T087 Performance validation: Verify <50ms transitions, <16ms input response
- [ ] T088 Update quickstart.md with final implementation notes in specs/051-interactive-specify-flow/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User Story 1: Independent after Foundational
  - User Story 2: Technically depends on US1 (needs generation to work), but can be developed/tested with mock data
  - User Story 3: Depends on US2 (needs review mode), but should be independently testable
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational, but integrates with US1 - Test by completing US1 first or mock the generation result
- **User Story 3 (P3)**: Can start after Foundational, but requires review mode from US2 - Test by completing US1+US2 first

### Within Each User Story

- Implementation before tests (tests follow existing patterns from feature 050)
- Core workflow (input/review/edit) before error handling
- Basic rendering before visual polish
- State management before UI transitions
- Story complete before moving to next priority

### Parallel Opportunities

- Phase 2: All enum variants and events marked [P] can run in parallel (T004-T011)
- Within US1: T031-T032 (tests) can run parallel with implementation
- Within US2: T050-T051 (tests) can run parallel with implementation
- Within US3: T067-T068 (tests) can run parallel with implementation
- Polish phase: All tests (T079-T082) can run in parallel, all manual test tasks (T083-T085) can run in parallel

---

## Parallel Example: Foundational Phase

```bash
# Launch all enum/event additions together:
Task: "Add ContentType::SpecifyInput variant to crates/rstn/src/tui/views/mod.rs"
Task: "Add ContentType::SpecifyReview variant to crates/rstn/src/tui/views/mod.rs"
Task: "Add Event::SpecifyGenerationStarted to crates/rstn/src/tui/event.rs"
Task: "Add Event::SpecifyGenerationCompleted to crates/rstn/src/tui/event.rs"
Task: "Add Event::SpecifyGenerationFailed to crates/rstn/src/tui/event.rs"
Task: "Add Event::SpecifySaved to crates/rstn/src/tui/event.rs"
Task: "Add ViewAction::GenerateSpec to action types"
Task: "Add ViewAction::SaveSpec to action types"
```

## Parallel Example: User Story 1 Tests

```bash
# Launch all US1 tests together:
Task: "Add basic unit test for SpecifyState creation and validation in tests/unit/tui_specify_tests.rs"
Task: "Add unit test for start_specify_input() and cancel_specify() transitions in tests/unit/tui_specify_tests.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (~5 mins - reading existing code)
2. Complete Phase 2: Foundational (~1-2 hours - type additions)
3. Complete Phase 3: User Story 1 (~4-6 hours - input mode + generation)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Create PR #1 (Foundation + US1) - Target: ~600 lines

**Result**: Context-switching eliminated, users can input and generate specs in TUI

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → **PR #1** (MVP!)
3. Add User Story 2 → Test independently → **PR #2**
4. Add User Story 3 → Test independently → **PR #3**
5. Add Polish → Complete testing → **PR #4**
6. Each PR adds value without breaking previous functionality

### Sequential Development (Single Developer)

**Estimated Timeline**:
- Phase 1-2: 2-3 hours (setup + foundation)
- Phase 3 (US1): 6-8 hours (input + generation)
- Phase 4 (US2): 4-6 hours (review + save)
- Phase 5 (US3): 5-7 hours (edit mode)
- Phase 6: 3-4 hours (polish + testing)

**Total**: ~20-28 hours for complete feature

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each logical group of tasks
- Stop at any checkpoint to validate story independently
- Follow feature 050 patterns closely for consistency
- All file paths are absolute from repository root

### Pull Request Strategy

**CRITICAL: Create separate PRs for each user story to keep changes reviewable.**

**PR Size Limits** (from CLAUDE.md and plan.md):
- **Ideal**: 500 lines of changes
- **Maximum**: 1,500 lines
- **Too large**: 3,000+ lines - must split

**Workflow for This Feature**:

1. **PR #1: Foundation + User Story 1** (~600 lines)
   - Phase 1: Setup
   - Phase 2: Foundational
   - Phase 3: User Story 1 (Stay in TUI)
   - Merge to main

2. **PR #2: User Story 2** (~400 lines)
   - Phase 4: User Story 2 (Review specs)
   - Merge to main

3. **PR #3: User Story 3** (~500 lines)
   - Phase 5: User Story 3 (Inline editing)
   - Merge to main

4. **PR #4: Polish** (~300 lines)
   - Phase 6: Polish & Cross-Cutting Concerns
   - Merge to main

**Before Creating PR**:
- Check line count: `git diff --stat main`
- Run tests: `cargo test --package rstn`
- Run clippy: `cargo clippy --all-targets --all-features`
- Run formatter: `cargo fmt`
- Manual testing per quickstart.md

**Benefits**:
- Faster code review cycles
- Easier to discuss specific changes
- Can merge incrementally (deliver value sooner)
- Simpler rollback if issues found
- Each PR is independently testable

---

## Task Summary

**Total Tasks**: 88
- Phase 1 (Setup): 2 tasks
- Phase 2 (Foundational): 11 tasks
- Phase 3 (User Story 1): 19 tasks
- Phase 4 (User Story 2): 19 tasks
- Phase 5 (User Story 3): 17 tasks
- Phase 6 (Polish): 20 tasks

**Parallel Tasks**: 25 (marked with [P])

**User Story Breakdown**:
- US1: 19 tasks (input mode + generation)
- US2: 19 tasks (review mode + save)
- US3: 17 tasks (edit mode)

**Testing Tasks**: 12 (unit tests + integration tests + manual tests)

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 (User Story 1) = 32 tasks
**Estimated MVP Time**: ~10-13 hours (single developer)

**Pattern Reference**: Feature 050 (Commit Review) - all tasks follow the same architectural patterns
