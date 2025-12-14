---
description: "Task breakdown for Enhanced Worktree View implementation"
---

# Tasks: Enhanced Worktree View with Tabs and Comprehensive Logging

**Input**: Design documents from `/specs/049-enhanced-worktree-view/`
**Prerequisites**: plan.md (technical architecture), spec.md (user stories), data-model.md (entities), contracts/ (module interfaces), research.md (decisions)

**Tests**: Tests are optional for this feature (not explicitly requested in spec). Tasks focus on implementation and manual testing per quickstart.md scenarios.

**Organization**: Tasks grouped by user story to enable independent implementation and testing. Aligned with 5-PR strategy from plan.md.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

Project structure (from plan.md):
- **TUI module**: `crates/rstn/src/tui/`
- **Logging module**: `crates/rstn/src/tui/logging/`
- **Tests**: `tests/tui/`

---

## Phase 1: Setup - Logging Infrastructure (PR #1: Foundation)

**Purpose**: Create logging module with core types and data structures

**Deliverable**: Internal logging types ready for integration (~400 lines)

- [ ] T001 Create logging module directory at crates/rstn/src/tui/logging/
- [ ] T002 [P] Create LogCategory enum in crates/rstn/src/tui/logging/entry.rs with icon() and color() methods
- [ ] T003 [P] Create LogEntry struct in crates/rstn/src/tui/logging/entry.rs with timestamp, category, content fields
- [ ] T004 [P] Implement LogEntry::new() constructor and format_timestamp() method in crates/rstn/src/tui/logging/entry.rs
- [ ] T005 [P] Create LogBuffer struct with VecDeque in crates/rstn/src/tui/logging/buffer.rs
- [ ] T006 [P] Implement LogBuffer::new(), push(), entries(), and len() methods in crates/rstn/src/tui/logging/buffer.rs
- [ ] T007 [P] Create FileChangeTracker struct with HashMap in crates/rstn/src/tui/logging/file_tracker.rs
- [ ] T008 [P] Implement FileChangeTracker::new() and check_files() methods in crates/rstn/src/tui/logging/file_tracker.rs
- [ ] T009 Create logging module exports in crates/rstn/src/tui/logging/mod.rs
- [ ] T010 Export logging module from crates/rstn/src/tui/mod.rs
- [ ] T011 Write unit test for LogBuffer circular buffer (1000-entry limit) in tests/tui/logging_tests.rs
- [ ] T012 [P] Write unit test for FileChangeTracker mtime detection in tests/tui/logging_tests.rs
- [ ] T013 Run cargo test to verify logging module tests pass
- [ ] T014 Run cargo clippy on logging module and fix any warnings
- [ ] T015 Build project and verify no compilation errors with cargo build -p rstn

**Checkpoint**: Logging module compiled and tested. Ready for integration into WorktreeView.

---

## Phase 2: User Story 1 - Quick Navigation Between Spec Documents (Priority: P1) 🎯 MVP

**Goal**: Add visual tab navigation for switching between spec.md, plan.md, and tasks.md

**Independent Test**: Open TUI on feature branch, press Left/Right arrows to switch tabs, verify content changes and tab highlighting works

**Deliverable**: Tab-based navigation in Content panel (~300 lines, PR #2)

### Implementation for User Story 1

- [ ] T016 [US1] Import ratatui Tabs widget in crates/rstn/src/tui/views/worktree.rs
- [ ] T017 [US1] Modify render_content() to split area into tab bar (3 lines) and content area in crates/rstn/src/tui/views/worktree.rs
- [ ] T018 [US1] Render Tabs widget with ["Spec", "Plan", "Tasks"] labels in crates/rstn/src/tui/views/worktree.rs
- [ ] T019 [US1] Map ContentType enum to tab selection index (0=Spec, 1=Plan, 2=Tasks) in crates/rstn/src/tui/views/worktree.rs
- [ ] T020 [US1] Apply yellow + bold highlight style to selected tab in crates/rstn/src/tui/views/worktree.rs
- [ ] T021 [US1] Add Left arrow key handler to cycle tabs backward in handle_key() in crates/rstn/src/tui/views/worktree.rs
- [ ] T022 [US1] Add Right arrow key handler to cycle tabs forward in handle_key() in crates/rstn/src/tui/views/worktree.rs
- [ ] T023 [US1] Verify 's' key still works for backward compatibility in crates/rstn/src/tui/views/worktree.rs
- [ ] T024 [US1] Reset content_scroll to 0 when switching tabs in crates/rstn/src/tui/views/worktree.rs
- [ ] T025 [US1] Build and test tab navigation with cargo build -p rstn && just install-dev
- [ ] T026 [US1] Manual test: Launch TUI, verify tabs visible and highlighted correctly
- [ ] T027 [US1] Manual test: Press Right arrow 3 times, verify Spec→Plan→Tasks→Spec cycle
- [ ] T028 [US1] Manual test: Press Left arrow, verify backward cycling works
- [ ] T029 [US1] Manual test: Verify tabs hidden on main branch (no feature detected)
- [ ] T030 [US1] Run cargo clippy and fix any warnings in worktree.rs changes

**Checkpoint**: Tab navigation complete and independently testable. Users can switch between spec/plan/tasks.

---

## Phase 3: User Story 2 - Comprehensive Activity Logging (Priority: P1)

**Goal**: Display timestamped, color-coded log of all commands (slash commands, Claude streams, shell scripts)

**Independent Test**: Run slash command, observe log entry with timestamp, icon, and color. Run multiple commands, verify all appear in chronological order.

**Deliverable**: Comprehensive logging in Output panel (~500 lines, PR #3)

### Implementation for User Story 2

- [ ] T031 [US2] Add log_buffer field of type LogBuffer to WorktreeView struct in crates/rstn/src/tui/views/worktree.rs
- [ ] T032 [US2] Initialize log_buffer in WorktreeView::new() in crates/rstn/src/tui/views/worktree.rs
- [ ] T033 [P] [US2] Replace output_lines Vec with log_buffer usage in crates/rstn/src/tui/views/worktree.rs
- [ ] T034 [P] [US2] Implement log() method to create LogEntry and push to buffer in crates/rstn/src/tui/views/worktree.rs
- [ ] T035 [P] [US2] Implement log_slash_command() helper method in crates/rstn/src/tui/views/worktree.rs
- [ ] T036 [P] [US2] Implement log_file_change() helper method in crates/rstn/src/tui/views/worktree.rs
- [ ] T037 [P] [US2] Implement log_shell_command() helper method in crates/rstn/src/tui/views/worktree.rs
- [ ] T038 [P] [US2] Add FileChanged event variant to Event enum in crates/rstn/src/tui/event.rs
- [ ] T039 [P] [US2] Add SlashCommandExecuted event variant to Event enum in crates/rstn/src/tui/event.rs
- [ ] T040 [US2] Modify render_output() to iterate log_buffer.entries() instead of output_lines in crates/rstn/src/tui/views/worktree.rs
- [ ] T041 [US2] Format log entries with timestamp, icon, and color in render_output() in crates/rstn/src/tui/views/worktree.rs
- [ ] T042 [US2] Implement style_for_category() to map LogCategory to Color in crates/rstn/src/tui/views/worktree.rs
- [ ] T043 [US2] Update output_scroll to work with log_buffer in crates/rstn/src/tui/views/worktree.rs
- [ ] T044 [US2] Update clear_output() to call log_buffer.clear() in crates/rstn/src/tui/views/worktree.rs (if method exists)
- [ ] T045 [US2] Call log_slash_command() in handle_action() for RunSpecPhase in crates/rstn/src/tui/app.rs
- [ ] T046 [US2] Call log() for ClaudeStream events in handle_claude_stream() in crates/rstn/src/tui/app.rs
- [ ] T047 [US2] Call log_shell_command() in handle_action() for RunCommand in crates/rstn/src/tui/app.rs
- [ ] T048 [US2] Add FileChanged event handling in handle_event() in crates/rstn/src/tui/app.rs
- [ ] T049 [US2] Build and test logging integration with cargo build -p rstn && just install-dev
- [ ] T050 [US2] Manual test: Run /speckit.specify, verify ⚡ slash command log entry appears
- [ ] T051 [US2] Manual test: Observe Claude streaming, verify 🤖 entries appear in real-time
- [ ] T052 [US2] Manual test: Run git status, verify 🔧 shell output logged
- [ ] T053 [US2] Manual test: Verify colors correct (cyan/white/yellow/green/dark gray)
- [ ] T054 [US2] Manual test: Generate 50+ log lines, verify scrolling works
- [ ] T055 [US2] Run cargo clippy and fix warnings in worktree.rs and app.rs changes

**Checkpoint**: Comprehensive logging complete. All command types appear in timestamped, color-coded log.

---

## Phase 4: User Story 3 - Automatic File Change Detection (Priority: P2)

**Goal**: Detect when spec.md, plan.md, or tasks.md are modified externally and reload content automatically

**Independent Test**: Edit spec.md in VS Code, save, verify TUI shows updated content within 2 seconds and logs file change

**Deliverable**: Polling-based file watching (~200 lines, PR #4)

### Implementation for User Story 3

- [ ] T056 [P] [US3] Add file_tracker field of type FileChangeTracker to WorktreeView in crates/rstn/src/tui/views/worktree.rs
- [ ] T057 [P] [US3] Add last_file_check_tick field (u64) to WorktreeView in crates/rstn/src/tui/views/worktree.rs
- [ ] T058 [US3] Initialize file_tracker and last_file_check_tick in WorktreeView::new() in crates/rstn/src/tui/views/worktree.rs
- [ ] T059 [US3] Implement check_file_changes() method to poll spec/plan/tasks files in crates/rstn/src/tui/views/worktree.rs
- [ ] T060 [US3] Call file_tracker.check_files() for spec.md, plan.md, tasks.md in check_file_changes() in crates/rstn/src/tui/views/worktree.rs
- [ ] T061 [US3] Reload file content when change detected in check_file_changes() in crates/rstn/src/tui/views/worktree.rs
- [ ] T062 [US3] Call log_file_change() for each changed file in check_file_changes() in crates/rstn/src/tui/views/worktree.rs
- [ ] T063 [US3] Modify tick() method to call check_file_changes() every 10 ticks in crates/rstn/src/tui/views/worktree.rs
- [ ] T064 [US3] Add tick count check (tick_count - last_file_check_tick >= 10) in tick() in crates/rstn/src/tui/views/worktree.rs
- [ ] T065 [US3] Update last_file_check_tick after checking in tick() in crates/rstn/src/tui/views/worktree.rs
- [ ] T066 [US3] Handle file deletion gracefully (show "No file found" message) in check_file_changes() in crates/rstn/src/tui/views/worktree.rs
- [ ] T067 [US3] Stop file watching when switching away from feature branch in crates/rstn/src/tui/views/worktree.rs
- [ ] T068 [US3] Build and test file watching with cargo build -p rstn && just install-dev
- [ ] T069 [US3] Manual test: Edit spec.md externally, verify content updates within 2 seconds
- [ ] T070 [US3] Manual test: Edit plan.md and tasks.md, verify both detected
- [ ] T071 [US3] Manual test: Verify 📝 file change log entries appear
- [ ] T072 [US3] Manual test: Delete tasks.md, verify "No file found" shown
- [ ] T073 [US3] Manual test: Rapid edits (auto-save), verify no log spam (max 1/sec)
- [ ] T074 [US3] Manual test: Switch to main branch, verify file watching stops
- [ ] T075 [US3] Run cargo clippy and fix warnings in worktree.rs changes

**Checkpoint**: File change detection complete. External edits reload automatically.

---

## Phase 5: User Story 4 - Managed Log History (Priority: P3)

**Goal**: Maintain 1000-line rolling buffer to prevent memory growth during long sessions

**Independent Test**: Generate >1000 log entries, verify buffer caps at 1000 and oldest entries evicted

**Deliverable**: Buffer validation and performance testing (~100 lines, PR #5)

### Implementation for User Story 4

- [ ] T076 [P] [US4] Verify LogBuffer capacity is 1000 in crates/rstn/src/tui/logging/buffer.rs
- [ ] T077 [P] [US4] Verify push() evicts oldest entry when full in crates/rstn/src/tui/logging/buffer.rs
- [ ] T078 [P] [US4] Write integration test to generate 1500 entries in tests/tui/logging_tests.rs
- [ ] T079 [US4] Verify test confirms buffer stays at 1000 entries in tests/tui/logging_tests.rs
- [ ] T080 [US4] Build and run buffer limit test with cargo test
- [ ] T081 [US4] Manual test: Run many commands to generate >1000 log lines
- [ ] T082 [US4] Manual test: Scroll to top, verify oldest 500+ entries removed
- [ ] T083 [US4] Manual test: Observe memory usage, verify stable ~200KB for buffer
- [ ] T084 [US4] Manual test: Generate 2000 entries, verify TUI remains responsive (>30 FPS)
- [ ] T085 [US4] Manual test: Verify no lag when scrolling through full buffer
- [ ] T086 [US4] Manual test: Verify auto-scroll still works with full buffer
- [ ] T087 [US4] Run performance benchmark per quickstart.md scenarios
- [ ] T088 [US4] Run cargo clippy --all-targets --all-features and fix warnings
- [ ] T089 [US4] Run cargo build --release and verify optimization works

**Checkpoint**: Buffer management validated. Performance targets met.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final integration, documentation, and validation

- [ ] T090 [P] Run all quickstart.md integration test scenarios manually
- [ ] T091 [P] Verify Scenario 1 (Tab Navigation) passes all checks
- [ ] T092 [P] Verify Scenario 2 (Comprehensive Logging) passes all checks
- [ ] T093 [P] Verify Scenario 3 (File Change Detection) passes all checks
- [ ] T094 [P] Verify Scenario 4 (Log Buffer Management) passes all checks
- [ ] T095 [P] Verify Scenario 5 (Integration Test) passes all checks
- [ ] T096 [P] Verify Scenario 6 (Edge Cases) passes all checks
- [ ] T097 Update CLAUDE.md with new Active Technologies entry for 049
- [ ] T098 Run cargo fmt to format all modified files
- [ ] T099 Run cargo test to ensure all tests pass
- [ ] T100 Run cargo clippy and ensure no warnings
- [ ] T101 Build release binary with cargo build --release -p rstn
- [ ] T102 Test release binary to verify performance
- [ ] T103 Validate all 4 user stories work together end-to-end
- [ ] T104 Verify backward compatibility (existing 's' key, clear command)
- [ ] T105 Verify constitution compliance (Performance, Zero-Config, etc.)
- [ ] T106 Document any known limitations or future enhancements

**Checkpoint**: Feature complete, tested, documented, ready for production.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - start immediately
- **Phase 2 (US1)**: Depends on Phase 1 completion (logging module must exist)
- **Phase 3 (US2)**: Depends on Phase 1 completion (logging module must exist)
- **Phase 4 (US3)**: Depends on Phase 1 completion (logging module must exist)
- **Phase 5 (US4)**: Depends on Phase 1 completion (logging module must exist)
- **Phase 6 (Polish)**: Depends on completion of all desired user stories

### User Story Dependencies

- **US1 (Tab Navigation)**: Independent - no dependencies on other stories
- **US2 (Comprehensive Logging)**: Independent - no dependencies on other stories
- **US3 (File Change Detection)**: Independent - works without tabs or comprehensive logging
- **US4 (Buffer Management)**: Built into LogBuffer from Phase 1 - just needs validation

**All user stories are independently testable and deliverable.**

### Within Each User Story

- Setup (logging module) before US integration
- Data structures (LogBuffer, FileChangeTracker) before usage
- Event types before event handling
- Helper methods before calling them
- Build before manual testing

### Parallel Opportunities

**Phase 1 (Setup)** - High parallelization:
- T002-T004: LogEntry and LogCategory (parallel, different concepts)
- T005-T006: LogBuffer (parallel with LogEntry)
- T007-T008: FileChangeTracker (parallel with LogBuffer)
- T011-T012: Unit tests (parallel, different test files)

**Phase 2 (US1)** - Sequential (same file modifications):
- Tab rendering tasks must be sequential (all modify worktree.rs)
- Manual tests sequential (require previous tasks complete)

**Phase 3 (US2)** - Moderate parallelization:
- T031-T037: Field additions and helper methods (can parallelize some)
- T038-T039: Event variants (parallel, different event types)
- T045-T047: Event handling (different handlers, can parallelize)

**Phase 4 (US3)** - Sequential (same file):
- File tracker integration tasks sequential (all modify worktree.rs)

**Phase 5 (US4)** - High parallelization:
- T076-T079: Tests can run in parallel
- T090-T096: Integration scenarios can run in parallel

**Across User Stories**:
- After Phase 1, US1-US4 can proceed in parallel if team capacity allows
- Recommended sequence: US1 → US2 → US3 → US4 (priority order)

---

## Parallel Example: Phase 1 (Logging Module)

```bash
# Launch all entity creation tasks together:
Task: "Create LogCategory enum in crates/rstn/src/tui/logging/entry.rs"
Task: "Create LogEntry struct in crates/rstn/src/tui/logging/entry.rs"
Task: "Create LogBuffer struct with VecDeque in crates/rstn/src/tui/logging/buffer.rs"
Task: "Create FileChangeTracker struct in crates/rstn/src/tui/logging/file_tracker.rs"

# Then launch all test tasks together:
Task: "Unit test for LogBuffer in tests/tui/logging_tests.rs"
Task: "Unit test for FileChangeTracker in tests/tui/logging_tests.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (logging module foundation)
2. Complete Phase 2: User Story 1 (tab navigation)
3. **STOP and VALIDATE**: Test tab navigation independently
4. Deploy/demo if ready
5. **Deliverable**: Visual tab navigation for spec/plan/tasks (MVP!)

### Incremental Delivery (Recommended)

1. **PR #1**: Phase 1 (Setup) → Foundation ready
2. **PR #2**: Phase 2 (US1) → Tab navigation works → Test independently → Deploy (MVP!)
3. **PR #3**: Phase 3 (US2) → Add comprehensive logging → Test independently → Deploy
4. **PR #4**: Phase 4 (US3) → Add file watching → Test independently → Deploy
5. **PR #5**: Phase 5 (US4) + Phase 6 (Polish) → Validate and polish → Deploy

Each PR adds value without breaking previous features.

### Parallel Team Strategy

With multiple developers:

1. Team completes Phase 1 (Setup) together
2. Once Phase 1 done:
   - Developer A: User Story 1 (Tab Navigation)
   - Developer B: User Story 2 (Comprehensive Logging)
   - Developer C: User Story 3 (File Change Detection)
3. User Story 4 is validation only (can be done by any developer)
4. Stories complete and integrate independently

---

## Notes

- **[P] tasks**: Different files or independent concepts, no execution dependencies
- **[Story] label**: Maps task to specific user story for traceability
- **Each user story independently completable**: Can test US1 without US2/US3/US4
- **Manual tests critical**: TUI features require visual verification per quickstart.md
- **Performance validation**: Must meet targets (<500ms tabs, <2s file detection, >30 FPS)
- **Commit frequently**: After each task or logical group
- **Stop at checkpoints**: Validate story independently before continuing

### Pull Request Strategy

**CRITICAL: Follow 5-PR strategy from plan.md to keep changes reviewable.**

**PR Size Limits** (from CLAUDE.md):
- **Ideal**: ≤ 500 lines
- **Maximum**: ≤ 1,500 lines
- **Too large**: > 3,000 lines - must split

**PR Sequence** (from plan.md):
1. **PR #1**: Phase 1 (T001-T015) → Logging module foundation (~400 lines)
2. **PR #2**: Phase 2 (T016-T030) → Tab navigation (~300 lines)
3. **PR #3**: Phase 3 (T031-T055) → Comprehensive logging (~500 lines)
4. **PR #4**: Phase 4 (T056-T075) → File change detection (~200 lines)
5. **PR #5**: Phase 5 + Phase 6 (T076-T106) → Buffer validation + polish (~100 lines)

**Before Creating PR**:
- Check line count: `git diff --stat main`
- Verify all tasks in phase complete
- Run cargo test, cargo clippy, cargo fmt
- Test manually per quickstart.md

**Benefits**:
- Each PR independently reviewable
- Can merge incrementally (deliver value sooner)
- Easier rollback if issues found
- Faster code review cycles

---

## Summary

**Total Tasks**: 106 tasks
**User Story Breakdown**:
- Phase 1 (Setup): 15 tasks
- Phase 2 (US1 - P1): 15 tasks
- Phase 3 (US2 - P1): 25 tasks
- Phase 4 (US3 - P2): 20 tasks
- Phase 5 (US4 - P3): 14 tasks
- Phase 6 (Polish): 17 tasks

**Parallel Opportunities**: 30+ tasks marked [P] can run in parallel within their phase

**Independent Test Criteria**:
- US1: Tab switching works with visual highlighting
- US2: All 4 log categories appear with correct icons/colors
- US3: File changes detected within 2 seconds
- US4: Buffer caps at 1000 entries, TUI remains responsive

**Suggested MVP Scope**: Phase 1 + Phase 2 (Setup + US1 Tab Navigation)

**Estimated Timeline**:
- Phase 1: 1-2 days (logging module)
- Phase 2: 0.5-1 day (tab navigation)
- Phase 3: 1-2 days (comprehensive logging)
- Phase 4: 0.5-1 day (file watching)
- Phase 5+6: 0.5-1 day (validation + polish)
- **Total**: 4-7 days for full feature

**Format Validation**: ✅ All 106 tasks follow checklist format with checkbox, ID, optional [P]/[Story] labels, and file paths
