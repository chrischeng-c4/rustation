# Tasks: Internalize Spec Generation

**Feature**: 052-internalize-spec-generation
**Branch**: `052-internalize-spec-generation`
**Generated**: 2025-12-16

## Overview

Replace bash shell script with native Rust implementation in `rstn-core`. This feature has 4 user stories that map to independent, testable components.

### User Story Mapping

| Story | Description | Components | Priority |
|-------|-------------|------------|----------|
| US1 | Maintainer: Rust codebase | Module structure, types | P1 |
| US2 | User: Faster generation | Number allocator, name generator | P1 |
| US3 | Developer: Better errors | Error types, rollback | P2 |
| US4 | Contributor: Testable logic | Unit tests, integration tests | P2 |

### Dependency Graph

```
Phase 1: Setup
    │
    ▼
Phase 2: Foundational (Types, Errors)
    │
    ├──────────────────┐
    ▼                  ▼
Phase 3: US1       Phase 4: US2
(Core Logic)       (Performance)
    │                  │
    └──────┬───────────┘
           ▼
    Phase 5: US3 (Error Handling)
           │
           ▼
    Phase 6: US4 (Testing)
           │
           ▼
    Phase 7: Polish
```

---

## Phase 1: Setup ✅ COMPLETE

**Goal**: Create module structure and add to rstn-core.

- [X] T001 Create specify module directory at `crates/rstn-core/src/specify/`
- [X] T002 Create module file with submodule declarations in `crates/rstn-core/src/specify/mod.rs`
- [X] T003 Add `pub mod specify;` to `crates/rstn-core/src/lib.rs`
- [X] T004 Create placeholder files for each submodule:
  - `crates/rstn-core/src/specify/number_allocator.rs`
  - `crates/rstn-core/src/specify/name_generator.rs`
  - `crates/rstn-core/src/specify/directory_setup.rs`
  - `crates/rstn-core/src/specify/spec_generator.rs`
  - `crates/rstn-core/src/specify/catalog_updater.rs`
- [X] T005 Verify `cargo check -p rstn-core` passes

**Checkpoint**: Module compiles with empty implementations. ✅

---

## Phase 2: Foundational ✅ COMPLETE

**Goal**: Define core types and error handling infrastructure.

- [X] T006 Define `NewFeature` struct in `crates/rstn-core/src/specify/mod.rs`
- [X] T007 Define `SpecResult` struct in `crates/rstn-core/src/specify/mod.rs`
- [X] T008 Define `SpecifyConfig` struct with `Default` impl in `crates/rstn-core/src/specify/mod.rs`
- [X] T009 Define `SpecifyError` enum with thiserror in `crates/rstn-core/src/specify/mod.rs`
- [X] T010 Define `FeaturesCatalog` and `FeatureEntry` structs with serde in `crates/rstn-core/src/specify/catalog_updater.rs`
- [X] T011 Add re-exports to `crates/rstn-core/src/lib.rs` for public types
- [X] T012 Verify `cargo check -p rstn-core` passes with all types

**Checkpoint**: All types defined, project compiles. ✅

---

## Phase 3: US1 - Core Logic (Maintainer Story) ✅ COMPLETE

**Goal**: Implement core spec generation logic in Rust.
**Independent Test**: Can allocate numbers, generate names, create directories without Claude CLI.

### Number Allocator (FR-1)

- [X] T013 [US1] Implement `allocate_feature_number()` that reads `specs/features.json` in `crates/rstn-core/src/specify/number_allocator.rs`
- [X] T014 [US1] Add directory scan fallback to find highest number in `specs/` in `crates/rstn-core/src/specify/number_allocator.rs`
- [X] T015 [US1] Implement max-of-both strategy (catalog + directory) in `crates/rstn-core/src/specify/number_allocator.rs`
- [X] T016 [US1] Add zero-padding format (3 digits) in `crates/rstn-core/src/specify/number_allocator.rs`

### Name Generator (FR-2)

- [X] T017 [P] [US1] Define stop words constant list in `crates/rstn-core/src/specify/name_generator.rs`
- [X] T018 [P] [US1] Implement `generate_feature_name()` with kebab-case conversion in `crates/rstn-core/src/specify/name_generator.rs`
- [X] T019 [P] [US1] Implement `extract_title()` for human-readable title in `crates/rstn-core/src/specify/name_generator.rs`
- [X] T020 [P] [US1] Add length truncation (50 chars) at word boundary in `crates/rstn-core/src/specify/name_generator.rs`

### Directory Setup (FR-3)

- [X] T021 [US1] Implement `setup_feature_directory()` to create `specs/{NNN}-{name}/` in `crates/rstn-core/src/specify/directory_setup.rs`
- [X] T022 [US1] Add placeholder file creation (spec.md from template) in `crates/rstn-core/src/specify/directory_setup.rs`
- [X] T023 [US1] Add template loading from `.specify/templates/spec-template.md` in `crates/rstn-core/src/specify/directory_setup.rs`

### Catalog Updater (FR-5)

- [X] T024 [US1] Implement `read_features_catalog()` to parse `specs/features.json` in `crates/rstn-core/src/specify/catalog_updater.rs`
- [X] T025 [US1] Implement `update_features_catalog()` to add new entry in `crates/rstn-core/src/specify/catalog_updater.rs`
- [X] T026 [US1] Add atomic write (temp file + rename) in `crates/rstn-core/src/specify/catalog_updater.rs`

**Checkpoint**: Can create feature directories and update catalog without Claude. ✅

---

## Phase 4: US2 - Claude Integration (User Story) ✅ COMPLETE

**Goal**: Integrate with Claude Code CLI for spec generation.
**Independent Test**: Can call Claude CLI and capture output.

### Spec Generator (FR-4)

- [X] T027 [US2] Implement `check_claude_cli_available()` using `which` crate in `crates/rstn-core/src/specify/spec_generator.rs`
- [X] T028 [US2] Implement `generate_spec_content()` with `tokio::process::Command` in `crates/rstn-core/src/specify/spec_generator.rs`
- [X] T029 [US2] Add timeout handling (configurable, default 120s) in `crates/rstn-core/src/specify/spec_generator.rs`
- [X] T030 [US2] Add stdout/stderr capture and parsing in `crates/rstn-core/src/specify/spec_generator.rs`
- [X] T031 [US2] Implement prompt building with template in `crates/rstn-core/src/specify/spec_generator.rs`

### Main Workflow

- [X] T032 [US2] Implement `generate_spec()` async function orchestrating full flow in `crates/rstn-core/src/specify/mod.rs`
- [X] T033 [US2] Wire up all components in correct order in `crates/rstn-core/src/specify/mod.rs`

**Checkpoint**: Full spec generation works with real Claude CLI. ✅

---

## Phase 5: US3 - Error Handling (Developer Story) ✅ COMPLETE

**Goal**: Comprehensive error handling and rollback.
**Independent Test**: Errors are descriptive, rollback cleans up.

### Error Handling (FR-6)

- [X] T034 [US3] Implement `rollback_directory()` to clean up on failure in `crates/rstn-core/src/specify/directory_setup.rs`
- [X] T035 [US3] Add rollback call in `generate_spec()` error path in `crates/rstn-core/src/specify/mod.rs`
- [X] T036 [US3] Enhance error messages with context (paths, numbers) in `crates/rstn-core/src/specify/mod.rs`
- [X] T037 [US3] Add tracing instrumentation for debugging in `crates/rstn-core/src/specify/mod.rs`

**Checkpoint**: Errors are actionable, partial state is cleaned up. ✅

---

## Phase 6: US4 - Testing (Contributor Story) ✅ COMPLETE

**Goal**: Comprehensive test coverage for reliability.
**Independent Test**: All unit tests pass, integration tests work with mock.

### Unit Tests

- [X] T038 [P] [US4] Add unit tests for number allocator (empty, existing, gaps) in `crates/rstn-core/src/specify/number_allocator.rs`
- [X] T039 [P] [US4] Add unit tests for name generator (simple, special chars, long, unicode) in `crates/rstn-core/src/specify/name_generator.rs`
- [X] T040 [P] [US4] Add unit tests for catalog updater (empty, existing, atomic write) in `crates/rstn-core/src/specify/catalog_updater.rs`
- [X] T041 [P] [US4] Add unit tests for directory setup (create, exists error) in `crates/rstn-core/src/specify/directory_setup.rs`

### Integration Tests

- [X] T042 [US4] Create test helper `setup_test_workspace()` in `crates/rstn-core/src/specify/mod.rs` (cfg test)
- [X] T043 [US4] Add integration test for full workflow (mock Claude) in `crates/rstn-core/src/specify/mod.rs` (cfg test)
- [X] T044 [US4] Add integration test for rollback on error in `crates/rstn-core/src/specify/mod.rs` (cfg test)

**Checkpoint**: `cargo test -p rstn-core` passes with >80% coverage. ✅

---

## Phase 7: Polish ✅ COMPLETE

**Goal**: Final cleanup, documentation, and integration prep.

- [X] T045 Add rustdoc comments to all public types and functions in `crates/rstn-core/src/specify/mod.rs`
- [X] T046 Run `cargo clippy -p rstn-core` and fix all warnings
- [X] T047 Run `cargo fmt -p rstn-core` to ensure formatting
- [X] T048 Update rstn-core re-exports in `crates/rstn-core/src/lib.rs`
- [X] T049 Verify `cargo build -p rstn-core` and `cargo test -p rstn-core` pass

**Checkpoint**: Ready for Feature 051 integration. ✅

---

## Summary

| Phase | Tasks | Parallel | Story | Status |
|-------|-------|----------|-------|--------|
| 1. Setup | 5 | 0 | - | ✅ |
| 2. Foundational | 7 | 0 | - | ✅ |
| 3. US1 Core Logic | 14 | 4 | US1 | ✅ |
| 4. US2 Claude | 7 | 0 | US2 | ✅ |
| 5. US3 Errors | 4 | 0 | US3 | ✅ |
| 6. US4 Testing | 7 | 4 | US4 | ✅ |
| 7. Polish | 5 | 0 | - | ✅ |
| **Total** | **49** | **8** | - | ✅ |

### Implementation Complete

All 49 tasks completed. The `specify` module is ready for integration with Feature 051.

**Test Results**: 29 tests passing
**Clippy**: Clean (0 warnings)
**Build**: Passing
