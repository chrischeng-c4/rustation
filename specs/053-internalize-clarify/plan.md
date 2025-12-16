# Implementation Plan: Internalize Clarify Workflow

**Branch**: `053-internalize-clarify` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/053-internalize-clarify/spec.md`

## Summary

Internalize the `/speckit.clarify` workflow into native Rust code in `rstn-core`. This creates a `clarify` module that analyzes spec files for ambiguities using an 11-category taxonomy, generates prioritized clarification questions, manages interactive Q&A sessions, and integrates answers back into the spec file atomically.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: tokio, serde, serde_json, thiserror, regex (all in workspace)
**Storage**: File system (`specs/{NNN}-{name}/spec.md`)
**Testing**: cargo test (unit tests for analysis, integration tests with temp files)
**Target Platform**: macOS (MVP), Linux (future)
**Project Type**: Monorepo - new module in `rstn-core` crate
**Performance Goals**: Spec analysis <500ms, file operations <100ms
**Constraints**: No new dependencies, atomic file writes, preserve markdown formatting
**Scale/Scope**: Single spec file per session, up to 5 questions per session

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Performance-First | ✅ PASS | Analysis <500ms, async operations, no blocking |
| II. Zero-Config | ✅ PASS | Works without config, uses hardcoded taxonomy |
| III. Progressive Complexity | ✅ PASS | Simple API, complex logic hidden internally |
| IV. Modern UX | ✅ PASS | Interactive Q&A, clear recommendations |
| V. Rust-Native | ✅ PASS | Pure Rust, reuses workspace crates |

**Gate Result**: PASS - All principles satisfied.

## Project Structure

### Documentation (this feature)

```text
specs/053-internalize-clarify/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/rstn-core/src/
├── lib.rs                    # Add: pub mod clarify;
└── clarify/                  # NEW MODULE
    ├── mod.rs                # Public API, types, ClarifyError
    ├── analyzer.rs           # Spec analysis, coverage mapping
    ├── question.rs           # Question generation, formatting
    ├── integrator.rs         # Spec file updates, atomic writes
    └── session.rs            # Q&A session state management
```

**Structure Decision**: Add new `clarify` module to existing `rstn-core` crate, following the pattern established by `specify` module (feature 052).

## Complexity Tracking

> No constitution violations - table not required.

## Deployment Strategy

### Selected Strategy

**Option 2: PR per User Story** - 4 PRs total

**Rationale**: Feature has 4 user stories (analyzer, interactive Q&A, integrator, tests). Each is independently testable. Estimated ~300-500 lines per PR.

### Merge Sequence

1. **PR #1: Foundation + Analyzer** (~400 lines)
   - Add `clarify` module structure
   - Implement `analyzer.rs` with taxonomy categories
   - Implement coverage mapping
   - Unit tests for analysis
   → Merge to main

2. **PR #2: Question Generation** (~400 lines)
   - Implement `question.rs` with Question types
   - Add question formatting (MC, short-answer)
   - Add prioritization logic
   - Claude CLI integration for generation
   → Merge to main

3. **PR #3: Session + Integrator** (~500 lines)
   - Implement `session.rs` for Q&A state
   - Implement `integrator.rs` for spec updates
   - Atomic file writes
   - Create/update Clarifications section
   → Merge to main

4. **PR #4: Polish + Tests** (~300 lines)
   - Integration tests
   - Completion report generation
   - Documentation
   - Wire up public API
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

Base branch: `053-internalize-clarify`
- `053-analyzer` for PR #1
- `053-questions` for PR #2
- `053-session-integrator` for PR #3
- `053-polish` for PR #4
