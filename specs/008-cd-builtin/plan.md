# Implementation Plan: cd Builtin Command

**Branch**: `008-cd-builtin` | **Date**: 2025-11-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-cd-builtin/spec.md`

## Summary

Implement the `cd` builtin command for the rush shell, enabling users to navigate the filesystem with support for absolute/relative paths, tilde expansion (~), previous directory navigation (cd -), and CDPATH search functionality. This is a core shell feature that builds on the existing EnvironmentManager infrastructure.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: std::env, std::path, existing EnvironmentManager
**Storage**: N/A (uses environment variables PWD, OLDPWD, HOME, CDPATH)
**Testing**: cargo test with unit tests and integration tests
**Target Platform**: macOS (MVP), Linux (post-MVP)
**Project Type**: Single project - Rust monorepo crate
**Performance Goals**: <100ms for any cd operation including CDPATH search
**Constraints**: Must work without configuration (zero-config philosophy)
**Scale/Scope**: Single builtin command with 4 user stories

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Performance-First | ✅ PASS | cd is synchronous filesystem operation, <100ms target easily achievable |
| II. Zero-Config | ✅ PASS | Works immediately with sensible defaults, CDPATH optional |
| III. Progressive Complexity | ✅ PASS | Basic cd (P1) works alone; cd - and CDPATH are opt-in |
| IV. Modern UX | ✅ PASS | Clear error messages, standard Unix conventions |
| V. Rust-Native | ✅ PASS | Pure Rust using std::env and std::path, no external deps |

**Gate Result**: PASS - All principles satisfied

## Project Structure

### Documentation (this feature)

```text
specs/008-cd-builtin/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── cd.md            # cd builtin contract
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/rush/src/
├── executor/
│   ├── builtins/
│   │   ├── mod.rs       # Register cd builtin
│   │   └── cd.rs        # NEW: cd builtin implementation
│   ├── environment.rs   # Existing: get/set PWD, OLDPWD, HOME, CDPATH
│   └── execute.rs       # Existing: builtin dispatch

crates/rush/tests/
└── feature_test.rs      # Add cd integration tests
```

**Structure Decision**: Add single file `cd.rs` to existing builtins module. Leverage existing EnvironmentManager for all environment variable operations.

## Complexity Tracking

No violations - implementation is straightforward and aligns with all constitution principles.

## Deployment Strategy

### Selected Strategy

**Option 2: PR per User Story** - Feature has 4 user stories with clear boundaries.

**Rationale**:
- US1+US2 (Basic + Tilde) are P1 and tightly coupled → combine in one PR
- US3 (cd -) is P2, independent → separate PR
- US4 (CDPATH) is P3, independent → separate PR

### Merge Sequence

1. **PR #1: Basic cd + Tilde Expansion (US1+US2)** → ~400 lines
   - Create cd.rs builtin
   - Implement absolute/relative path navigation
   - Implement tilde expansion
   - Update PWD after successful cd
   - Add unit and integration tests

2. **PR #2: Previous Directory (US3)** → ~150 lines
   - Track OLDPWD before each cd
   - Implement `cd -` functionality
   - Print directory when using cd -
   - Add tests for cd - scenarios

3. **PR #3: CDPATH Search (US4)** → ~200 lines
   - Implement CDPATH parsing
   - Search CDPATH when relative path not in cwd
   - Prioritize local directory over CDPATH
   - Add tests for CDPATH scenarios

**Branch Strategy**: Work on `008-cd-builtin` branch, merge each PR to main sequentially.

### PR Size Validation

```bash
git diff --stat main  # Check line count before each PR
```

All PRs estimated under 500 lines - well within limits.
