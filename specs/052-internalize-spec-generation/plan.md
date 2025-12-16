# Implementation Plan: Internalize Spec Generation

**Branch**: `052-internalize-spec-generation` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/052-internalize-spec-generation/spec.md`

## Summary

Replace the external bash script (`create-new-feature.sh`) with native Rust implementation in `rstn-core`. This provides better error handling, testability, and eliminates platform-specific shell dependencies while maintaining the same functionality: feature number allocation, name generation, directory setup, spec generation via Claude CLI, and features.json catalog updates.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: tokio, serde_json, thiserror (all already in workspace)
**Storage**: File system (`specs/` directory, `features.json`)
**Testing**: cargo test (unit tests for pure functions, integration tests with tempdir)
**Target Platform**: macOS (MVP), Linux (future)
**Project Type**: Monorepo - new module in `rstn-core` crate
**Performance Goals**: File operations <100ms, spec generation bound by Claude API (<30s)
**Constraints**: No new external dependencies, atomic file writes for safety
**Scale/Scope**: Single feature directory per invocation, ~100 features max in catalog

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Performance-First | ✅ PASS | File ops <100ms, async Claude CLI call, no blocking |
| II. Zero-Config | ✅ PASS | Works without user config, uses hardcoded defaults |
| III. Progressive Complexity | ✅ PASS | Simple API, complexity hidden in implementation |
| IV. Modern UX | ✅ PASS | Better error messages than shell script |
| V. Rust-Native | ✅ PASS | Pure Rust, no new dependencies, reuses workspace crates |

**Gate Result**: PASS - All principles satisfied. No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/052-internalize-spec-generation/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/rstn-core/src/
├── lib.rs                    # Add: pub mod specify;
├── errors.rs                 # Add: SpecifyError variants
└── specify/                  # NEW MODULE
    ├── mod.rs                # Public API: generate_spec()
    ├── number_allocator.rs   # Feature number allocation
    ├── name_generator.rs     # Kebab-case name generation
    ├── directory_setup.rs    # Create directory structure
    ├── spec_generator.rs     # Claude CLI integration
    └── catalog_updater.rs    # Update features.json

crates/rstn-core/src/specify/tests/
├── mod.rs                    # Test module
├── number_allocator_tests.rs # Unit tests
├── name_generator_tests.rs   # Unit tests
└── integration_tests.rs      # Full workflow tests
```

**Structure Decision**: Add new `specify` module to existing `rstn-core` crate. This follows the established pattern (git, build, test, service modules) and reuses existing error handling infrastructure.

## Complexity Tracking

> No constitution violations - table not required.

## Deployment Strategy

### Selected Strategy

**Option 2: PR per User Story** - 4 PRs total

**Rationale**: Feature has 4 logical user stories (maintainer, user, developer, contributor). Each can be implemented and tested independently. Estimated ~300-500 lines per PR.

### Merge Sequence

1. **PR #1: Foundation** (~300 lines)
   - Add `specify` module structure to rstn-core
   - Implement `NumberAllocator` and `NameGenerator`
   - Unit tests for pure functions
   → Merge to main

2. **PR #2: Directory & Catalog** (~400 lines)
   - Implement `DirectorySetup` and `CatalogUpdater`
   - Atomic file writes with temp file + rename
   - Unit tests and integration tests with tempdir
   → Merge to main

3. **PR #3: Claude Integration** (~500 lines)
   - Implement `SpecGenerator` with Claude CLI
   - Async process spawning with tokio
   - Timeout and error handling
   - Rollback on partial failure
   → Merge to main

4. **PR #4: Integration & Polish** (~300 lines)
   - Wire up to TUI (Feature 051 integration point)
   - Add `generate_spec()` public API
   - Update rstn-core re-exports
   - Integration tests for full workflow
   → Merge to main

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

### Branch Strategy

Base branch: `052-internalize-spec-generation`
- `052-foundation` for PR #1
- `052-directory-catalog` for PR #2
- `052-claude-integration` for PR #3
- `052-integration` for PR #4
