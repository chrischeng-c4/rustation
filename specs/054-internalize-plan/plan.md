# Implementation Plan: Internalize Plan Workflow

**Branch**: `054-internalize-plan` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/054-internalize-plan/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Move the `/speckit.plan` workflow from bash scripts to native Rust code in rstn-core. This creates a new `plan` module alongside the existing `specify` module that:
1. Loads spec and constitution context
2. Invokes Claude Code CLI in headless mode to fill the plan template
3. Generates research.md, data-model.md, and quickstart.md artifacts
4. Provides atomic file writes with rollback on failure

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: tokio (async runtime), serde_json (JSON parsing), thiserror (error handling), which (CLI detection) - all already in workspace
**Storage**: File system - `specs/{NNN}-{name}/` directory structure
**Testing**: cargo test with tempfile crate for integration tests
**Target Platform**: macOS (MVP), Linux (post-MVP)
**Project Type**: Single library crate (rstn-core)
**Performance Goals**: <60s plan generation (Claude API bound), <100ms file operations
**Constraints**: Atomic file writes (temp file + rename), rollback on partial failure
**Scale/Scope**: Single user CLI tool, ~200 lines of new code + tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Performance-First | ✅ PASS | File ops <100ms, Claude API time is unavoidable bottleneck |
| II. Zero-Config | ✅ PASS | Uses existing template paths as defaults, no user config needed |
| III. Progressive Complexity | ✅ PASS | Basic plan generation default, artifacts generated as needed |
| IV. Modern UX | ✅ PASS | Clear error messages, actionable feedback on failures |
| V. Rust-Native | ✅ PASS | Pure Rust implementation, uses existing workspace dependencies |
| Spec-Driven Development | ✅ PASS | Spec exists (054/spec.md), following SDD workflow |

## Project Structure

### Documentation (this feature)

```text
specs/054-internalize-plan/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (from /speckit.tasks)
```

### Source Code (repository root)

```text
crates/rstn-core/src/
├── specify/             # Existing spec generation module (feature 052)
│   ├── mod.rs           # Module root with SpecifyError, etc.
│   ├── spec_generator.rs # Claude CLI integration pattern
│   └── ...
└── plan/                # NEW: Plan generation module (this feature)
    ├── mod.rs           # PlanError, PlanResult, generate_plan()
    ├── context_loader.rs # Load spec, constitution, template
    ├── plan_generator.rs # Claude CLI integration for plan filling
    ├── artifact_writer.rs # Atomic writes for research.md, data-model.md, etc.
    └── rollback.rs      # Cleanup on partial failure
```

**Structure Decision**: Follows the established `specify` module pattern. New `plan` module sits alongside it in rstn-core, reusing error types and Claude CLI patterns.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*No violations - all constitution principles pass.*

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**CRITICAL: Keep PRs small and reviewable (see CLAUDE.md for limits).**

### Selected Strategy

**Option 1: Single PR** - Entire feature ≤500 lines expected

This feature is a focused internal refactoring that:
- Adds ~200 lines of new Rust code
- Follows established patterns from feature 052
- Has clear boundaries (plan module only)

**Rationale**: Feature is small enough for single PR. Estimated ~300-400 lines including tests.

### Merge Sequence

1. **PR: feat(054): internalize plan workflow into Rust** → Merge to main
   - Add `plan` module to rstn-core
   - Context loading (spec, constitution, template)
   - Claude CLI integration for plan generation
   - Artifact generation (research.md, data-model.md, quickstart.md)
   - Rollback on failure
   - Unit tests

**Branch Strategy**: Use existing `054-internalize-plan` branch, merge directly to main.

### PR Size Validation

**Before creating PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

If PR exceeds limits, split into Foundation + Implementation PRs.
