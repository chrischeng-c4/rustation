# Implementation Plan: Source Builtin Command

**Branch**: `015-source-builtin` | **Date**: 2024-12-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/015-source-builtin/spec.md`

## Summary

Implement `source` and `.` (dot) builtin commands that execute commands from a script file in the current shell context. This enables configuration loading, alias definitions, and environment variable setup that persists in the user's session.

## Technical Context

**Language/Version**: Rust 1.75+ (Rust 2021 edition)
**Primary Dependencies**: std::fs, std::path (no new dependencies needed)
**Storage**: N/A (reads script files from filesystem)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: macOS (MVP)
**Project Type**: Single project (Rust monorepo workspace)
**Performance Goals**: Script file loading in <10ms for typical config files
**Constraints**: Must execute in current shell context (no subprocess)
**Scale/Scope**: Small feature - single builtin with 2 syntax variants

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Performance-First | ✅ PASS | File reading is fast; no blocking operations required beyond file I/O |
| II. Zero-Config | ✅ PASS | Works without configuration; enables RC file support for future |
| III. Progressive Complexity | ✅ PASS | Basic usage simple (`source file.sh`); advanced features (args, PATH search) opt-in |
| IV. Modern UX | ✅ PASS | Clear error messages with file/line information |
| V. Rust-Native | ✅ PASS | Pure Rust implementation using std library only |

**Constitution Gate**: PASSED - No violations. Feature aligns with all core principles.

## Project Structure

### Documentation (this feature)

```text
specs/015-source-builtin/
├── spec.md              # Feature specification
├── plan.md              # This file
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Implementation tasks (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/rush/src/
├── executor/
│   ├── builtins/
│   │   ├── mod.rs           # Add source and dot to dispatch
│   │   └── source.rs        # NEW: source/dot builtin implementation
│   └── execute.rs           # CommandExecutor (provides execute_line method)
└── error.rs                 # Error types

crates/rush/src/tests/
└── integration_test.rs      # Add source builtin tests
```

**Structure Decision**: Single project structure. The source builtin follows the existing pattern in `executor/builtins/` with a dedicated module file.

## Design Decisions

### D1: Script Execution Architecture

**Decision**: Reuse `CommandExecutor::execute()` for each line in the script file.

**Rationale**:
- Maintains consistency with existing command execution
- Preserves variable/alias state across lines
- Handles all existing features (pipes, redirections, etc.) automatically

**Alternative Rejected**: Creating a separate script executor would duplicate logic and risk inconsistencies.

### D2: PATH Search Implementation

**Decision**: Current directory first, then PATH directories (matching bash behavior).

**Rationale**:
- Matches user expectations from bash
- Prevents accidental execution of wrong file when local file exists

### D3: Error Reporting

**Decision**: Report errors with filename and line number, but continue executing remaining lines.

**Rationale**:
- Matches bash behavior for non-fatal errors
- Allows partial configuration loading
- Syntax errors in individual commands don't abort entire script

### D4: Nested Sourcing

**Decision**: Allow recursive sourcing with depth limit of 100.

**Rationale**:
- Enables modular configuration files
- Depth limit prevents infinite recursion
- 100 levels is more than sufficient for real-world use

## Deployment Strategy

### Selected Strategy: Single PR

**Rationale**: Feature is small (~300-400 lines), self-contained, and implements a single cohesive capability. All user stories are tightly coupled (source/dot commands share 95% of code).

### Merge Sequence

1. **PR #1: Source Builtin** (~400 lines) → Merge to main
   - `source.rs` - Core implementation
   - Update `builtins/mod.rs` - Add dispatch entries
   - Integration tests
   - All 4 user stories in one PR

### PR Size Validation

**Estimated Size**:
- `source.rs`: ~200 lines
- `mod.rs` changes: ~10 lines
- Tests: ~150 lines
- **Total**: ~360 lines (well under 500 line ideal)

## Implementation Outline

### Core Components

1. **Source Builtin Module** (`source.rs`)
   - `execute()` function for `source` command
   - `execute_dot()` function for `.` command (calls execute)
   - File resolution (relative, absolute, tilde, PATH search)
   - Line-by-line execution via CommandExecutor
   - Error handling with file context

2. **Builtin Registration** (`mod.rs`)
   - Add `source` and `.` to dispatch match
   - Export source module

3. **Tests**
   - Unit tests for file resolution
   - Integration tests for script execution
   - Edge case tests (missing file, permission denied, nested sourcing)

### Key Implementation Details

```rust
// Signature matches existing builtins pattern
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32>

// File resolution order:
// 1. If path is absolute or starts with ./ or ../ -> use directly
// 2. If path starts with ~ -> expand tilde
// 3. If file exists in current directory -> use it
// 4. Search PATH directories
// 5. Return error if not found

// Execution:
// 1. Read entire file
// 2. For each line:
//    a. Skip empty lines and comments (#)
//    b. Execute via executor.execute(line)
//    c. Track exit code
// 3. Return last command's exit code
```

## Dependencies

### Upstream Dependencies
- None - uses only existing CommandExecutor infrastructure

### Downstream Dependencies
- Feature 042 (RC file execution) will use this to execute `~/.rushrc`
- Feature 043 (Profile files) will use this for `~/.rush_profile`

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Infinite recursion via nested sourcing | High | Implement depth limit (100 levels) |
| Large file performance | Low | Lazy line-by-line execution; stream file |
| Inconsistent behavior with bash | Medium | Test against bash behavior for key scenarios |

## Checklist

- [ ] `source.rs` implements core functionality
- [ ] Both `source` and `.` commands registered
- [ ] File resolution supports all path types
- [ ] PATH search implemented
- [ ] Error messages include file and line context
- [ ] Nested sourcing works with depth limit
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Documentation updated
