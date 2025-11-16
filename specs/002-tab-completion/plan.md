# Implementation Plan: Tab Completion

**Branch**: `002-tab-completion` | **Date**: 2025-11-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-tab-completion/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement tab completion for rush shell that allows users to complete command names, file paths, and command flags by pressing Tab. The implementation leverages reedline's built-in Completer trait to provide zero-configuration, performance-first completion with <100ms latency. The feature is delivered in three independent priority tiers: P1 (command completion), P2 (path completion), P3 (flag completion).

## Technical Context

**Language/Version**: Rust 2021 edition (workspace standard)
**Primary Dependencies**: reedline 0.26+ (already in use, has Completer trait support)
**Storage**: In-memory caches for PATH executables and filesystem entries (no persistent storage)
**Testing**: cargo test with unit tests for completers + integration tests for REPL
**Target Platform**: macOS (MVP v0.1), Linux support deferred to post-MVP
**Project Type**: Single binary project (rush shell)
**Performance Goals**: <100ms completion latency for typical scenarios (per spec SC-001)
**Constraints**:
  - <10MB baseline memory footprint (constitution)
  - Zero configuration required (constitution)
  - <5ms command execution overhead (constitution)
**Scale/Scope**:
  - ~500-1000 executables in typical PATH
  - Directories with up to 10,000 entries
  - Support 20+ common flags across git, cargo, ls commands

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Performance-First ✅

- **Requirement**: Completions must appear within 100ms (spec SC-001)
- **Approach**: Lazy-loaded in-memory caches, PATH scan on first use
- **Compliance**: Aligns with constitution's "instant responsiveness" and "no blocking operations"
- **Justification**: Caching strategy ensures <100ms target without blocking REPL

### Principle II: Zero-Config Philosophy ✅

- **Requirement**: FR-011 - Must work with zero configuration
- **Approach**: Automatic PATH scanning, no config files required
- **Compliance**: Aligns with "sensible defaults" and "no mandatory setup"
- **Justification**: Feature works immediately after installation

### Principle III: Progressive Complexity ✅

- **Requirement**: Three priority tiers (P1: commands, P2: paths, P3: flags)
- **Approach**: Each tier independently deployable and testable
- **Compliance**: Aligns with "layered functionality" and "no forced complexity"
- **Justification**: Users benefit from P1 alone; P2/P3 are progressive enhancements

### Principle IV: Modern UX ✅

- **Requirement**: Smart completions with menu display for multiple matches
- **Approach**: Context-aware completers with visual menu feedback
- **Compliance**: Aligns with "smart completions" and "visual feedback"
- **Justification**: Tab completion is fundamental modern shell UX

### Principle V: Rust-Native ✅

- **Requirement**: Use Rust ecosystem libraries
- **Approach**: Leverage reedline's Completer trait (pure Rust)
- **Compliance**: Aligns with "ecosystem integration" and "pure Rust"
- **Justification**: reedline is mature, actively maintained, zero FFI

**Overall Status**: ✅ **PASSED** - No violations, all principles aligned

## Project Structure

### Documentation (this feature)

```text
specs/002-tab-completion/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── completer-trait.md  # Completer trait interface contracts
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/rush/
├── src/
│   ├── main.rs                     # Entry point (minimal changes)
│   ├── repl/
│   │   └── mod.rs                  # REPL loop (integrate completers)
│   ├── completion/                 # NEW: Completion module
│   │   ├── mod.rs                  # Module exports
│   │   ├── command.rs              # P1: CommandCompleter
│   │   ├── path.rs                 # P2: PathCompleter
│   │   ├── flag.rs                 # P3: FlagCompleter
│   │   └── registry.rs             # Multi-completer coordinator
│   └── ...
└── tests/
    ├── integration/
    │   └── completion_tests.rs     # Integration tests for REPL completion
    └── unit/
        └── completion/              # Unit tests for each completer
            ├── command_tests.rs
            ├── path_tests.rs
            └── flag_tests.rs
```

**Structure Decision**: Single project structure with new `completion/` module under `src/`. This aligns with rush's monorepo organization and keeps completion logic isolated. The module is structured by priority tier (command, path, flag) to enable independent development and testing per spec user stories.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations - this section is empty.

All complexity is justified by spec requirements and aligns with constitution principles:
- In-memory caching justified by performance requirements (<100ms)
- Multi-completer design justified by progressive complexity (P1/P2/P3 independence)
- reedline dependency already in use, no new external dependencies required

---

## Phase 0: Research (Generated Below)

Research tasks identified:
1. **reedline Completer trait API** - Understand interface, examples, best practices
2. **PATH scanning on macOS** - Efficient scanning, permission handling, symlink resolution
3. **Filesystem completion patterns** - Hidden files, spaces in paths, case sensitivity
4. **Completion menu UX** - Limiting matches, navigation, visual feedback
5. **Performance optimization** - Caching strategies, lazy loading, memory management

These will be documented in `research.md`.

## Phase 1: Design (Generated Below)

Artifacts to generate:
1. **data-model.md** - Define Completer trait, cache structures, entities
2. **contracts/completer-trait.md** - Document reedline Completer interface and custom implementations
3. **quickstart.md** - Developer guide for testing and extending completers

---

**Next Step**: Generate research.md, then data-model.md and contracts/.
