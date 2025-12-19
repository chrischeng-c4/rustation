# Implementation Plan: Release Channels

**Branch**: `047-release-channels` | **Date**: 2025-12-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/047-release-channels/spec.md`

## Summary

Implement dual release channels for rustation: (1) local development channel with debug builds and trace-level logging installed to `~/.local/bin`, and (2) Homebrew release channel via personal tap `chrischeng-c4/homebrew-rustation` with optimized release builds and info-level logging. Both channels install `rush` and `rstn` binaries with clear build type identification in version output.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021) - existing workspace
**Primary Dependencies**: just (task runner, existing), cargo (build system), Homebrew (new external dependency for distribution)
**Storage**: N/A (build/install tooling, no data persistence)
**Testing**: cargo test (existing), manual verification of install/version commands
**Target Platform**: macOS (aligns with constitution MVP scope)
**Project Type**: single (Rust monorepo workspace with 2 binary crates: rush, rstn)
**Performance Goals**: N/A for build tooling; release builds maintain <100ms startup per constitution
**Constraints**: Debug builds larger (~22MB) with trace logging; release builds smaller (~6MB) with info logging
**Scale/Scope**: 2 binaries (rush, rstn), 2 installation channels, 1 external repository (homebrew tap)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Justification |
|-----------|--------|---------------|
| I. Performance-First | ✅ PASS | Release builds maintain optimization; debug builds explicitly for development with expected performance trade-offs |
| II. Zero-Config | ✅ PASS | Both channels work with single command (`just install-dev` or `brew install rustation`); no configuration required |
| III. Progressive Complexity | ✅ PASS | Basic install (release) is default; debug install opt-in for developers |
| IV. Modern UX | ✅ PASS | Clear build type in version output; familiar tooling (Homebrew, just) |
| V. Rust-Native | ✅ PASS | Uses cargo build, standard Rust tooling; `#[cfg(debug_assertions)]` for compile-time detection |

**All gates pass. Proceeding to Phase 0.**

## Project Structure

### Documentation (this feature)

```text
specs/047-release-channels/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── checklists/
│   └── requirements.md  # Spec validation checklist
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
# Existing workspace structure (minimal changes)
rustation/
├── Cargo.toml                          # Workspace root (no changes)
├── justfile                            # Add install-dev, which-build recipes
├── crates/
│   ├── rush/
│   │   └── src/
│   │       └── main.rs                 # (optional) Add build profile to version
│   └── rstn/
│       ├── build.rs                    # Add BUILD_PROFILE env var
│       └── src/
│           ├── main.rs                 # (no changes)
│           ├── settings.rs             # Add cfg-based log level default
│           └── version.rs              # Add build_info() function
└── target/
    ├── debug/                          # Debug builds (install-dev)
    └── release/                        # Release builds (install, homebrew)

# New external repository
homebrew-rustation/                     # github.com/chrischeng-c4/homebrew-rustation
├── README.md                           # Tap documentation
└── Formula/
    └── rustation.rb                    # Homebrew formula
```

**Structure Decision**: Minimal changes to existing workspace structure. New Homebrew tap is a separate repository. No new crates or directories needed in main repo.

## Complexity Tracking

> **No violations to justify.** All changes align with constitution principles.

## Deployment Strategy

### Pull Request Plan

**Strategy**: **Option 2: PR per User Story** - Feature has 3 user stories that can be implemented independently.

### Selected Strategy

**PR per User Story** - Each user story delivers independent value:
- US1 (P1): Local dev installation - can be used immediately
- US2 (P2): Homebrew installation - external repo, independent
- US3 (P3): Build type identification - enhances both channels

**Rationale**: 3 user stories, estimated ~300 lines each (justfile changes, settings.rs, build.rs, version.rs, formula). Each PR well under 500 lines.

### Merge Sequence

1. **PR #1: Foundation + US3 (Build Type ID)** → Merge to main
   - Add BUILD_PROFILE to build.rs
   - Update version.rs to display build type
   - Add cfg-based log level default in settings.rs
   - ~150 lines

2. **PR #2: US1 (Local Dev Channel)** → Merge to main
   - Add install-dev, install-rstn-dev, which-build to justfile
   - Test and validate debug builds
   - ~50 lines

3. **PR #3: US2 (Homebrew Channel)** → Separate repo
   - Create homebrew-rustation repository
   - Add Formula/rustation.rb
   - Create git tag v0.35.0 on rustation repo
   - Test brew install workflow

**Branch Strategy**: Work on `047-release-channels` branch, merge PRs sequentially to main.

### PR Size Validation

**Estimated sizes:**
- PR #1: ~150 lines ✅ (well under 500)
- PR #2: ~50 lines ✅ (well under 500)
- PR #3: ~100 lines ✅ (formula + README)

All PRs within ideal size limits.
