# Tasks: Internalize Clarify Workflow

**Input**: Design documents from `/specs/053-internalize-clarify/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Tests are included as US4 explicitly requires testability.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Monorepo**: `crates/rstn-core/src/clarify/` for new module
- Module structure per plan.md: mod.rs, analyzer.rs, question.rs, session.rs, integrator.rs

---

## Phase 1: Setup (Module Structure)

**Purpose**: Create clarify module structure in rstn-core

- [X] T001 Create clarify module directory at crates/rstn-core/src/clarify/
- [X] T002 Create mod.rs with public API exports at crates/rstn-core/src/clarify/mod.rs
- [X] T003 Add `pub mod clarify;` to crates/rstn-core/src/lib.rs
- [X] T004 [P] Create placeholder analyzer.rs at crates/rstn-core/src/clarify/analyzer.rs
- [X] T005 [P] Create placeholder question.rs at crates/rstn-core/src/clarify/question.rs
- [X] T006 [P] Create placeholder session.rs at crates/rstn-core/src/clarify/session.rs
- [X] T007 [P] Create placeholder integrator.rs at crates/rstn-core/src/clarify/integrator.rs
- [X] T008 Verify `cargo build -p rstn-core` passes

---

## Phase 2: Foundational (Core Types)

**Purpose**: Define all shared types that user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T009 Define Category enum with 10 variants in crates/rstn-core/src/clarify/mod.rs
- [X] T010 Implement Category::keywords() method returning keyword patterns
- [X] T011 Implement Category::all() method returning all variants
- [X] T012 Define CoverageStatus enum (Clear, Partial, Missing, Resolved, Deferred, Outstanding)
- [X] T013 Define CoverageMap type alias as HashMap<Category, CoverageStatus>
- [X] T014 Define AnalysisResult struct (coverage, needs_clarification, match_counts)
- [X] T015 [P] Define QuestionFormat enum (MultipleChoice, ShortAnswer) in crates/rstn-core/src/clarify/mod.rs
- [X] T016 [P] Define QuestionOption struct (letter, description)
- [X] T017 [P] Define RecommendedAnswer struct (value, reasoning)
- [X] T018 Define Question struct (id, category, text, format, recommended, impact)
- [X] T019 Define Answer struct (question_id, value, accepted_recommendation, question_text)
- [X] T020 Define ClarifyConfig struct with defaults (max_questions: 5, max_answer_words: 5, etc.)
- [X] T021 Define ClarifyError enum with all error variants using thiserror
- [X] T022 Verify `cargo build -p rstn-core` passes with all types

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Spec Ambiguity Detection (Priority: P1) 🎯 MVP

**Goal**: Detect ambiguities in spec files using 10-category taxonomy

**Independent Test**: `analyze_spec("path/to/spec.md")` returns CoverageMap with correct status per category

### Implementation for User Story 1

- [X] T023 [US1] Implement load_spec_content() to read spec file in crates/rstn-core/src/clarify/analyzer.rs
- [X] T024 [US1] Implement parse_spec_sections() to split by ## headers
- [X] T025 [US1] Implement scan_category() to count keyword matches for one category
- [X] T026 [US1] Implement classify_coverage() to determine Clear/Partial/Missing from match counts
- [X] T027 [US1] Implement detect_placeholders() to find TODO/TBD/placeholder markers
- [X] T028 [US1] Implement analyze_spec() main entry point combining all analysis steps
- [X] T029 [US1] Add unit tests for analyzer in crates/rstn-core/src/clarify/analyzer.rs

**Checkpoint**: `analyze_spec()` works independently - can analyze any spec file

---

## Phase 4: User Story 2 - Interactive Q&A Session (Priority: P2)

**Goal**: Present questions and collect answers interactively

**Independent Test**: Session tracks questions asked, validates answers, enforces max 5 questions

### Implementation for User Story 2

- [X] T030 [US2] Define ClarifySession struct in crates/rstn-core/src/clarify/session.rs
- [X] T031 [US2] Implement ClarifySession::new() to initialize session from analysis
- [X] T032 [US2] Implement ClarifySession::is_complete() checking question count and queue
- [X] T033 [US2] Implement ClarifySession::next_question() to pop from queue
- [X] T034 [US2] Implement validate_multiple_choice() for option letter validation
- [X] T035 [US2] Implement validate_short_answer() for word count validation
- [X] T036 [US2] Implement submit_answer() to validate and record answers
- [X] T037 [US2] Implement start_session() public API in crates/rstn-core/src/clarify/session.rs
- [X] T038 [US2] Add unit tests for session management in crates/rstn-core/src/clarify/session.rs

**Checkpoint**: Session lifecycle works independently - can run Q&A without spec integration

---

## Phase 5: User Story 2 (continued) - Question Generation

**Goal**: Generate prioritized clarification questions from coverage analysis

**Independent Test**: Given coverage map, generates up to 5 prioritized questions with format and recommendations

### Implementation for Question Generation

- [X] T039 [US2] Implement calculate_impact() for category importance scoring in crates/rstn-core/src/clarify/question.rs
- [X] T040 [US2] Implement prioritize_categories() sorting by (Impact × Uncertainty)
- [X] T041 [US2] Implement build_question_prompt() for Claude CLI request
- [X] T042 [US2] Implement call_claude_cli() with timeout handling (reuse pattern from specify module)
- [X] T043 [US2] Implement parse_claude_response() to extract question from CLI output
- [X] T044 [US2] Implement generate_fallback_question() for template-based questions when Claude unavailable
- [X] T045 [US2] Implement generate_questions() main async entry point
- [X] T046 [US2] Add unit tests for question generation in crates/rstn-core/src/clarify/question.rs

**Checkpoint**: Question generation works independently - can generate questions from any coverage map

---

## Phase 6: User Story 3 - Spec Integration (Priority: P3)

**Goal**: Integrate clarification answers back into spec file

**Independent Test**: Given answers, spec file is updated with Clarifications section atomically

### Implementation for User Story 3

- [X] T047 [US3] Implement find_clarifications_section() to locate existing section in crates/rstn-core/src/clarify/integrator.rs
- [X] T048 [US3] Implement create_clarifications_section() to add section after Overview
- [X] T049 [US3] Implement format_session_header() for `### Session YYYY-MM-DD` format
- [X] T050 [US3] Implement format_qa_bullet() for `- Q: ... → A: ...` format
- [X] T051 [US3] Implement append_to_clarifications() to add Q&A bullets
- [X] T052 [US3] Implement check_duplicate_qa() to prevent duplicate entries
- [X] T053 [US3] Implement atomic_write_spec() using temp file + rename pattern
- [X] T054 [US3] Implement integrate_answer() to update spec after each answer
- [X] T055 [US3] Implement rollback_spec() for error recovery
- [X] T056 [US3] Add unit tests for integrator in crates/rstn-core/src/clarify/integrator.rs

**Checkpoint**: Spec integration works independently - can update any spec file with clarifications

---

## Phase 7: User Story 3 (continued) - Finalization & Report

**Goal**: Generate completion report after clarification session

### Implementation for Finalization

- [X] T057 [US3] Define ClarifyReport struct in crates/rstn-core/src/clarify/mod.rs
- [X] T058 [US3] Implement count_sections_touched() to track modified sections
- [X] T059 [US3] Implement generate_coverage_summary() for final status
- [X] T060 [US3] Implement identify_outstanding() for unresolved categories
- [X] T061 [US3] Implement identify_deferred() for planning-phase categories
- [X] T062 [US3] Implement suggest_next_command() based on report state
- [X] T063 [US3] Implement finalize_session() main entry point
- [X] T064 [US3] Add unit tests for report generation

**Checkpoint**: Full clarify workflow functional end-to-end

---

## Phase 8: User Story 4 - Testability (Priority: P4)

**Goal**: Comprehensive test coverage for all clarify logic

### Integration Tests

- [ ] T065 [US4] Create test fixture spec file with known ambiguities in crates/rstn-core/src/clarify/test_fixtures/
- [ ] T066 [US4] Integration test: empty spec → all Missing in crates/rstn-core/src/clarify/tests.rs
- [ ] T067 [US4] Integration test: complete spec → all Clear
- [ ] T068 [US4] Integration test: partial spec → mixed statuses
- [ ] T069 [US4] Integration test: full session with mock Claude
- [ ] T070 [US4] Integration test: spec file before/after comparison
- [ ] T071 [US4] Integration test: atomic write verified (crash recovery)
- [ ] T072 [US4] Integration test: rollback on error

### Edge Case Tests

- [ ] T073 [P] [US4] Test: markdown parsing edge cases (nested headers, code blocks)
- [ ] T074 [P] [US4] Test: answer validation edge cases (empty, too many words, invalid option)
- [ ] T075 [P] [US4] Test: duplicate Q&A prevention
- [ ] T076 [P] [US4] Test: session quota enforcement (max 5 questions)

**Checkpoint**: All tests pass, full coverage of clarify module

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and integration

- [X] T077 Wire up public API exports in crates/rstn-core/src/clarify/mod.rs
- [X] T078 Add module-level documentation in crates/rstn-core/src/clarify/mod.rs
- [X] T079 [P] Run `cargo clippy -p rstn-core` and fix warnings
- [X] T080 [P] Run `cargo fmt -p rstn-core` for formatting
- [X] T081 Verify all 052 specify module tests still pass
- [ ] T082 Run quickstart.md validation scenarios
- [ ] T083 Update CLAUDE.md if needed for clarify workflow

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1 (Analyzer) can start after Foundational
  - US2 (Session/Questions) can start after Foundational
  - US3 (Integrator) can start after Foundational
  - US4 (Tests) depends on US1, US2, US3 completion
- **Polish (Phase 9)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Analyzer - no dependencies on other stories
- **User Story 2 (P2)**: Session/Questions - uses analyzer output, but independently testable
- **User Story 3 (P3)**: Integrator - uses session/answers, but independently testable
- **User Story 4 (P4)**: Tests - depends on all stories for integration tests

### Within Each User Story

- Core functions before composite functions
- Validation before main entry points
- Unit tests at end of each story phase

### Parallel Opportunities

- T004-T007: All placeholder files can be created in parallel
- T015-T017: Question-related types can be defined in parallel
- T073-T076: Edge case tests can run in parallel
- US1, US2, US3 can be developed in parallel after Phase 2

---

## Parallel Example: Phase 2 Types

```bash
# Launch all independent type definitions together:
Task: "Define QuestionFormat enum in crates/rstn-core/src/clarify/mod.rs"
Task: "Define QuestionOption struct in crates/rstn-core/src/clarify/mod.rs"
Task: "Define RecommendedAnswer struct in crates/rstn-core/src/clarify/mod.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (module structure)
2. Complete Phase 2: Foundational (core types)
3. Complete Phase 3: User Story 1 (analyzer)
4. **STOP and VALIDATE**: Test analyzer independently
5. Can analyze specs even without Q&A or integration

### Incremental Delivery

1. Setup + Foundational → Module compiles
2. Add US1 (Analyzer) → Can analyze specs (MVP!)
3. Add US2 (Session/Questions) → Can generate Q&A
4. Add US3 (Integrator) → Full clarify workflow
5. Add US4 (Tests) → Regression-proof
6. Each story adds value without breaking previous

### PR Strategy (from plan.md)

1. **PR #1**: Phase 1-3 (Setup + Foundation + Analyzer) ~400 lines
2. **PR #2**: Phase 4-5 (Session + Questions) ~400 lines
3. **PR #3**: Phase 6-7 (Integrator + Report) ~500 lines
4. **PR #4**: Phase 8-9 (Tests + Polish) ~300 lines

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Reuse patterns from 052-specify module where applicable
- Claude CLI integration pattern from spec_generator.rs

### Pull Request Strategy

**PR Size Limits** (from plan.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

**Branch Strategy**:
- Base: `053-internalize-clarify`
- PR #1: `053-analyzer`
- PR #2: `053-questions`
- PR #3: `053-session-integrator`
- PR #4: `053-polish`
