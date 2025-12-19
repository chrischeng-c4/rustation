# Implementation Plan: Trap Builtin

**Branch**: `037-trap-builtin` | **Date**: 2025-12-10 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/037-trap-builtin/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement the `trap` builtin command for signal handling in rush shell. This feature allows users to register cleanup handlers for POSIX signals (including real-time signals SIGRTMIN-SIGRTMAX), list active traps, and clear trap handlers. The implementation will leverage Rust's `nix` crate (already in dependencies) for signal handling infrastructure, integrate with the existing builtin command pattern in `executor/builtins/`, and maintain rush's performance-first philosophy with <100ms handler execution latency.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: nix 0.29 (signal handling), existing in Cargo.toml
**Storage**: In-memory HashMap in CommandExecutor (trap registry persists for shell session lifetime)
**Testing**: cargo test (unit tests for signal parsing, registration, listing), integration tests for signal delivery and handler execution
**Target Platform**: macOS (MVP per constitution), Linux post-MVP
**Project Type**: Single project (monorepo crate structure)
**Performance Goals**: <100ms trap handler execution latency (FR SC-002), <5 seconds for trap listing (SC-005), instantaneous clearing (SC-008)
**Constraints**: Must reject uncatchable signals (SIGKILL/SIGSTOP) per OS restrictions, must integrate with existing job control signal infrastructure
**Scale/Scope**: Support all ~31 POSIX signals + 32 real-time signals (~63 total signals), single trap handler per signal

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Performance-First ✅

- **Fast startup**: No impact on shell initialization (<100ms maintained) - trap registry initialized lazily on first use
- **Instant responsiveness**: <100ms handler execution meets <16ms command prompt target
- **Minimal overhead**: In-memory HashMap adds negligible overhead (~1KB for typical 5-10 traps)
- **Memory efficiency**: <10MB baseline maintained - trap data structure ~8 bytes per signal + command string
- **No blocking operations**: Signal handler registration is synchronous but <1ms; handler execution is async-safe

**Compliance**: PASS - Feature maintains all performance targets

### Principle II: Zero-Config Philosophy ✅

- **Sensible defaults**: trap command works immediately with no setup required
- **No mandatory setup**: Users can register handlers without any configuration files
- **Progressive disclosure**: Advanced features (real-time signals, EXIT pseudo-signal) discoverable through help/documentation

**Compliance**: PASS - Feature requires zero configuration

### Principle III: Progressive Complexity ✅

- **Layered functionality**:
  - Basic: `trap 'cleanup' INT` (P1 - most common use case)
  - Intermediate: `trap` to list handlers (P2 - debugging)
  - Advanced: Real-time signals RTMIN-RTMAX (P3 - power users)
- **No forced complexity**: Users not using trap pay zero cost
- **Discoverability**: `trap --help` reveals all options

**Compliance**: PASS - Three-tier priority structure aligns with progressive complexity

### Principle IV: Modern UX ✅

- **Clear feedback**: Informative error messages for invalid signals, duplicate traps
- **Visual indicators**: `trap` listing shows clear format "trap -- 'command' SIGNAL"
- **Helpful errors**: "invalid signal specification: INVALID" better than cryptic errno codes

**Compliance**: PASS - Error messages and output format prioritize clarity

### Principle V: Rust-Native ✅

- **Pure Rust**: Uses existing `nix` crate (Rust bindings to POSIX signals)
- **Ecosystem integration**: Follows existing builtin command pattern in codebase
- **Zero-cost abstractions**: HashMap<Signal, String> for trap registry
- **Idiomatic code**: Standard Result<i32> return type, module structure matches existing builtins

**Compliance**: PASS - Implementation leverages Rust ecosystem and idioms

**Final Constitution Check**: ✅ **PASS** - No violations, all principles satisfied

## Project Structure

### Documentation (this feature)

```text
specs/037-trap-builtin/
├── spec.md              # Feature specification (completed)
├── plan.md              # This file (current)
├── research.md          # Phase 0 output (signal handling patterns, nix API)
├── data-model.md        # Phase 1 output (TrapHandler, SignalRegistry structures)
├── quickstart.md        # Phase 1 output (usage examples)
├── contracts/           # Phase 1 output (trap builtin API contract)
│   └── trap-api.md      # Public API specification
├── checklists/          # Quality validation
│   └── requirements.md  # Completed specification checklist
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/rush/
├── src/
│   ├── executor/
│   │   ├── builtins/
│   │   │   ├── mod.rs          # Register trap builtin [MODIFY]
│   │   │   └── trap.rs          # New: trap command implementation [CREATE]
│   │   ├── execute.rs           # CommandExecutor with trap registry [MODIFY]
│   │   └── mod.rs               # Exports [MODIFY]
│   ├── error.rs                 # Error types [MODIFY - add TrapError variants]
│   └── lib.rs                   # Public API exports [NO CHANGE]
└── tests/
    ├── integration/
    │   └── trap_tests.rs        # New: integration tests [CREATE]
    └── unit/
        └── trap_unit_tests.rs   # New: unit tests [CREATE]
```

**Structure Decision**: Single project structure (Option 1) - rush is a monorepo crate. The trap builtin follows the established pattern in `executor/builtins/` where each builtin is a separate module (cd.rs, echo.rs, set.rs, etc.). Trap functionality integrates with existing CommandExecutor which already manages builtins, variables, and job control.

## Complexity Tracking

**No violations** - Constitution Check passed all principles without exceptions.

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**CRITICAL: Keep PRs small and reviewable (see CLAUDE.md for limits).**

**Strategy**: PR per User Story (Option 2) - RECOMMENDED for multi-story features

Rationale: 3 user stories with distinct functionality:
- US1 (P1): Register signal handlers (~800 lines - core logic)
- US2 (P2): List active traps (~300 lines - display logic)
- US3 (P3): Clear trap handlers (~200 lines - removal logic)

Total estimated ~1,300 lines, but each story is independently valuable and testable.

### Selected Strategy

**Option 2: PR per User Story** - Each user story delivers standalone value

**Rationale**:
- US1 alone provides MVP signal handling capability (independently testable)
- US2 adds debugging/inspection capability (independently testable)
- US3 enables dynamic trap management (independently testable)
- Each PR ≤ 1,000 lines, well within limits
- Allows incremental review and early user feedback

### Merge Sequence

1. **PR #1: Foundation + US1 (Register Cleanup Handlers)** → Merge to main
   - Create `trap.rs` module with signal parsing and registration
   - Add TrapRegistry to CommandExecutor
   - Implement `trap 'command' SIGNAL` syntax
   - Support POSIX signals + real-time signals
   - Error handling for SIGKILL/SIGSTOP, invalid signals
   - Unit tests for signal parsing, validation, registration
   - Integration tests for handler execution on signal delivery
   - Estimated: ~800 lines
   - **Delivers**: Core trap functionality (P1 complete)

2. **PR #2: US2 (Inspect Active Traps)** → Merge to main
   - Implement `trap` (no args) listing functionality
   - Format output as "trap -- 'command' SIGNAL"
   - Unit tests for listing format
   - Integration tests for listing accuracy
   - Estimated: ~300 lines
   - **Delivers**: Debugging/inspection capability (P2 complete)

3. **PR #3: US3 (Clear Trap Handlers)** → Merge to main
   - Implement `trap "" SIGNAL` clearing syntax
   - Update registry removal logic
   - Restore default signal behavior
   - Unit tests for clearing operations
   - Integration tests for default behavior restoration
   - Estimated: ~200 lines
   - **Delivers**: Dynamic trap management (P3 complete)

4. **PR #4: Documentation + Edge Cases** → Merge to main
   - Add comprehensive help text (`trap --help`)
   - Document all edge cases from spec
   - Add quickstart.md examples
   - Polish error messages
   - Estimated: ~150 lines
   - **Delivers**: Complete user-facing documentation

**Branch Strategy**:
- Base branch: `037-trap-builtin` (current)
- Feature branches: `037-trap-US1`, `037-trap-US2`, `037-trap-US3`, `037-trap-docs`
- Each PR merges feature branch → `037-trap-builtin`
- Final PR merges `037-trap-builtin` → `main`

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

If any PR exceeds limits, split into smaller increments.

**Estimated Total**: ~1,450 lines across 4 PRs - within acceptable limits
