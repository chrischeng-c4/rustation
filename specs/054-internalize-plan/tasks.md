# Tasks: Internalize Plan Workflow

**Input**: Design documents from `/specs/054-internalize-plan/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: Unit tests included per spec requirement (US4: Testable Implementation).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `crates/rstn-core/src/plan/` for source, tests inline with `#[cfg(test)]`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create plan module structure and add to rstn-core

- [x] T001 Create plan module directory at crates/rstn-core/src/plan/
- [x] T002 [P] Create mod.rs with module declarations at crates/rstn-core/src/plan/mod.rs
- [x] T003 [P] Add `pub mod plan;` to crates/rstn-core/src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define core types that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Define PlanError enum with all error variants in crates/rstn-core/src/plan/mod.rs
- [x] T005 [P] Define PlanConfig struct with Default impl in crates/rstn-core/src/plan/mod.rs
- [x] T006 [P] Define PlanResult and PlanArtifact structs in crates/rstn-core/src/plan/mod.rs
- [x] T007 [P] Define ArtifactKind enum in crates/rstn-core/src/plan/mod.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Maintainer Simplification (Priority: P1) 🎯 MVP

**Goal**: Replace bash shell script with native Rust plan generation

**Independent Test**: Run `generate_plan()` on existing spec and verify plan.md is created without shell script

### Implementation for User Story 1

- [x] T008 [P] [US1] Create context.rs file at crates/rstn-core/src/plan/context.rs
- [x] T009 [US1] Define PlanContext struct in crates/rstn-core/src/plan/context.rs
- [x] T010 [US1] Implement PlanContext::load() to read spec.md, constitution.md, plan-template.md in crates/rstn-core/src/plan/context.rs
- [x] T011 [P] [US1] Create generator.rs file at crates/rstn-core/src/plan/generator.rs
- [x] T012 [US1] Implement check_claude_cli_available() in crates/rstn-core/src/plan/generator.rs
- [x] T013 [US1] Implement build_plan_prompt() to create Claude prompt in crates/rstn-core/src/plan/generator.rs
- [x] T014 [US1] Implement generate_plan_content() with tokio Command in crates/rstn-core/src/plan/generator.rs
- [x] T015 [US1] Implement generate_plan() main entry point in crates/rstn-core/src/plan/mod.rs
- [x] T016 [US1] Add re-exports for public types in crates/rstn-core/src/plan/mod.rs

**Checkpoint**: User Story 1 complete - plan.md generation works via Rust

---

## Phase 4: User Story 2 - Better Error Handling (Priority: P2)

**Goal**: Provide clear, actionable error messages for all failure modes

**Independent Test**: Trigger each error condition and verify message contains path/context

### Implementation for User Story 2

- [x] T017 [US2] Add SpecNotFound error with path display in crates/rstn-core/src/plan/mod.rs
- [x] T018 [P] [US2] Add TemplateNotFound error with path display in crates/rstn-core/src/plan/mod.rs
- [x] T019 [P] [US2] Add ClaudeNotFound error with install instructions in crates/rstn-core/src/plan/mod.rs
- [x] T020 [P] [US2] Add ClaudeTimeout error with timeout value in crates/rstn-core/src/plan/mod.rs
- [x] T021 [US2] Add RollbackFailed error with cleanup details in crates/rstn-core/src/plan/mod.rs
- [x] T022 [US2] Ensure all errors implement std::error::Error via thiserror in crates/rstn-core/src/plan/mod.rs

**Checkpoint**: User Story 2 complete - all errors are clear and actionable

---

## Phase 5: User Story 3 - Artifact Generation (Priority: P2)

**Goal**: Generate all planning artifacts (research.md, data-model.md, quickstart.md)

**Independent Test**: Run generate_plan() and verify all 4 artifacts exist with valid content

### Implementation for User Story 3

- [x] T023 [P] [US3] Create writer.rs file at crates/rstn-core/src/plan/writer.rs
- [x] T024 [US3] Define ArtifactWriter struct with feature_dir and created_artifacts in crates/rstn-core/src/plan/writer.rs
- [x] T025 [US3] Implement ArtifactWriter::new() in crates/rstn-core/src/plan/writer.rs
- [x] T026 [US3] Implement ArtifactWriter::write() with temp file + atomic rename in crates/rstn-core/src/plan/writer.rs
- [x] T027 [US3] Implement ArtifactWriter::rollback() to remove created artifacts in crates/rstn-core/src/plan/writer.rs
- [x] T028 [US3] Add generate_research_prompt() in crates/rstn-core/src/plan/generator.rs
- [x] T029 [P] [US3] Add generate_data_model_prompt() in crates/rstn-core/src/plan/generator.rs
- [x] T030 [P] [US3] Add generate_quickstart_prompt() in crates/rstn-core/src/plan/generator.rs
- [x] T031 [US3] Update generate_plan() to call artifact generation sequentially in crates/rstn-core/src/plan/mod.rs
- [x] T032 [US3] Add rollback on failure in generate_plan() using ArtifactWriter in crates/rstn-core/src/plan/mod.rs

**Checkpoint**: User Story 3 complete - all artifacts generated with rollback support

---

## Phase 6: User Story 4 - Testable Implementation (Priority: P3)

**Goal**: Enable reliable testing with unit tests and mock Claude CLI

**Independent Test**: Run `cargo test -p rstn-core plan` and verify all tests pass

### Implementation for User Story 4

- [x] T033 [P] [US4] Add unit test for PlanConfig::default() in crates/rstn-core/src/plan/mod.rs
- [x] T034 [P] [US4] Add unit test for PlanContext::load() with tempfile in crates/rstn-core/src/plan/context.rs
- [x] T035 [P] [US4] Add unit test for build_plan_prompt() in crates/rstn-core/src/plan/generator.rs
- [x] T036 [P] [US4] Add unit test for ArtifactWriter::write() with tempfile in crates/rstn-core/src/plan/writer.rs
- [x] T037 [P] [US4] Add unit test for ArtifactWriter::rollback() in crates/rstn-core/src/plan/writer.rs
- [x] T038 [US4] Add integration test for full generate_plan() flow with mock setup in crates/rstn-core/src/plan/mod.rs

**Checkpoint**: User Story 4 complete - all tests pass, >80% coverage on pure functions

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and validation

- [x] T039 Run cargo clippy -p rstn-core and fix any warnings
- [x] T040 Run cargo fmt -p rstn-core to ensure formatting
- [x] T041 Verify all tests pass with cargo test -p rstn-core plan
- [x] T042 Update crates/rstn-core/src/lib.rs docs to mention plan module

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 (Phase 3): Can proceed immediately after Foundational
  - US2 (Phase 4): Can proceed in parallel with US1 (different focus)
  - US3 (Phase 5): Depends on US1 completion (needs generate_plan() base)
  - US4 (Phase 6): Depends on US1-US3 completion (tests the implementations)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

```
Phase 1: Setup
    │
    ▼
Phase 2: Foundational (BLOCKS all)
    │
    ├──► Phase 3: US1 (P1) - Core plan generation
    │         │
    │         ├──► Phase 4: US2 (P2) - Error handling (parallel with US1)
    │         │
    │         └──► Phase 5: US3 (P2) - Artifact generation (after US1)
    │                   │
    │                   ▼
    │              Phase 6: US4 (P3) - Tests (after US1-US3)
    │
    └──► Phase 7: Polish (after all stories)
```

### Within Each User Story

- Types before implementations
- Context loading before generation
- Generation before writing
- Core implementation before integration

### Parallel Opportunities

**Phase 1 (Setup)**:
- T002, T003 can run in parallel

**Phase 2 (Foundational)**:
- T005, T006, T007 can run in parallel (after T004)

**Phase 3 (US1)**:
- T008, T011 can run in parallel (file creation)

**Phase 4 (US2)**:
- T018, T019, T020 can run in parallel (error variants)

**Phase 5 (US3)**:
- T023 alone, then T028, T029, T030 can run in parallel (prompts)

**Phase 6 (US4)**:
- T033, T034, T035, T036, T037 can ALL run in parallel (unit tests)

---

## Parallel Example: User Story 4 Tests

```bash
# Launch all unit tests in parallel:
Task: "Add unit test for PlanConfig::default() in crates/rstn-core/src/plan/mod.rs"
Task: "Add unit test for PlanContext::load() with tempfile in crates/rstn-core/src/plan/context.rs"
Task: "Add unit test for build_plan_prompt() in crates/rstn-core/src/plan/generator.rs"
Task: "Add unit test for ArtifactWriter::write() with tempfile in crates/rstn-core/src/plan/writer.rs"
Task: "Add unit test for ArtifactWriter::rollback() in crates/rstn-core/src/plan/writer.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (3 tasks)
2. Complete Phase 2: Foundational (4 tasks)
3. Complete Phase 3: User Story 1 (9 tasks)
4. **STOP and VALIDATE**: Test plan.md generation works
5. Can ship as MVP without artifacts or tests

### Incremental Delivery

1. Setup + Foundational → Foundation ready (7 tasks)
2. Add US1 → Test plan.md generation → MVP ready (16 tasks total)
3. Add US2 → Better error messages → Enhanced UX (22 tasks total)
4. Add US3 → All artifacts generated → Complete workflow (32 tasks total)
5. Add US4 → Full test coverage → Production ready (38 tasks total)
6. Polish → Ship (42 tasks total)

### Single PR Strategy (per plan.md)

Since estimated ~300-400 lines:
1. Complete all phases
2. Verify with `git diff --stat main`
3. If ≤500 lines: Single PR
4. If >500 lines: Split into Foundation PR + Implementation PR

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Use tempfile crate for tests (already in workspace)

### Task Summary

| Phase | Tasks | Cumulative |
|-------|-------|------------|
| Setup | 3 | 3 |
| Foundational | 4 | 7 |
| US1 (P1) | 9 | 16 |
| US2 (P2) | 6 | 22 |
| US3 (P2) | 10 | 32 |
| US4 (P3) | 6 | 38 |
| Polish | 4 | 42 |
| **Total** | **42** | |

### MVP Scope

**Recommended MVP**: Complete through Phase 3 (User Story 1)
- 16 tasks total
- Delivers: Native Rust plan generation
- Validates: Core workflow works without shell scripts
- Estimated: ~150 lines of code
