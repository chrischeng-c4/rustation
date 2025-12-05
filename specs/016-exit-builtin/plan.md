# Implementation Plan: Exit Builtin Command

**Branch**: `016-exit-builtin` | **Date**: 2024-12-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/016-exit-builtin/spec.md`

## Summary

Implement `exit` builtin command that terminates the shell with an optional exit code. When called with no arguments, exits with status of last command. When called with a numeric argument, exits with that status code. Supports standard POSIX shell exit behavior for scripting and interactive use.

## Technical Context

**Language/Version**: Rust 1.75+ (Rust 2021 edition)
**Primary Dependencies**: std only (no new dependencies needed)
**Storage**: N/A (no persistence needed)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: macOS (MVP)
**Project Type**: Single project (Rust monorepo workspace)
**Performance Goals**: Instantaneous execution (<1ms)
**Constraints**: Must signal main loop to terminate, not just return
**Scale/Scope**: Small feature - single builtin command

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Performance-First | ✅ PASS | Exit is instantaneous with no I/O operations |
| II. Zero-Config | ✅ PASS | Works without configuration; standard behavior |
| III. Progressive Complexity | ✅ PASS | Basic usage simple (`exit`); advanced features (explicit code) opt-in |
| IV. Modern UX | ✅ PASS | Clear error messages for invalid arguments |
| V. Rust-Native | ✅ PASS | Pure Rust implementation using std library only |

**Constitution Gate**: PASSED - No violations. Feature aligns with all core principles.

## Project Structure

### Documentation (this feature)

```text
specs/016-exit-builtin/
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
│   │   ├── mod.rs           # Add exit to dispatch
│   │   └── exit.rs          # NEW: exit builtin implementation
│   └── execute.rs           # CommandExecutor (needs to handle exit signal)
├── error.rs                 # May need ExitSignal error type
└── main.rs                  # Handle exit signal from executor

crates/rush/src/tests/
└── integration_test.rs      # Add exit builtin tests
```

**Structure Decision**: Single project structure. The exit builtin follows the existing pattern in `executor/builtins/` with a dedicated module file.

## Design Decisions

### D1: Exit Signal Mechanism

**Decision**: Use a special error type (`RushError::ExitRequest(i32)`) to signal exit from builtins.

**Rationale**:
- Builtins return `Result<i32>` but exit needs to terminate the shell, not continue
- Using an error type allows the signal to propagate up to main loop
- Main loop catches this specific error and exits with the code

**Alternative Rejected**: A boolean "should_exit" flag would require threading through all execution layers.

### D2: Exit Code Range Handling

**Decision**: Use `(value as i32) & 0xFF` to mask exit codes to 0-255 range.

**Rationale**:
- Matches POSIX specification for exit codes
- Handles negative numbers correctly (e.g., -1 becomes 255)
- Matches bash behavior for edge cases

### D3: Too Many Arguments Behavior

**Decision**: Print error and do NOT exit when given invalid arguments.

**Rationale**:
- Matches bash behavior - invalid exit command is an error, not a request to exit
- Allows user to correct their mistake and try again
- More forgiving for interactive use

## Deployment Strategy

### Selected Strategy: Single PR

**Rationale**: Feature is small (~150-200 lines), self-contained, and implements a single cohesive capability. All user stories share the same execution path.

### Merge Sequence

1. **PR #1: Exit Builtin** (~200 lines) → Merge to main
   - `exit.rs` - Core implementation
   - Update `builtins/mod.rs` - Add dispatch entry
   - Update `execute.rs` - Handle ExitRequest error
   - Integration tests
   - All 4 user stories in one PR

### PR Size Validation

**Estimated Size**:
- `exit.rs`: ~80 lines
- `mod.rs` changes: ~5 lines
- `execute.rs` changes: ~10 lines
- Error type changes: ~10 lines
- Tests: ~100 lines
- **Total**: ~205 lines (well under 500 line ideal)

## Implementation Outline

### Core Components

1. **Exit Builtin Module** (`exit.rs`)
   - `execute()` function for `exit` command
   - Argument parsing and validation
   - Exit code masking to 0-255 range
   - Error messages for invalid arguments

2. **Exit Signal** (`error.rs`)
   - Add `ExitRequest(i32)` variant to RushError
   - Used to signal exit to main loop

3. **Signal Handling** (`execute.rs` and `main.rs`)
   - Catch ExitRequest error in executor
   - Propagate to main loop
   - Main loop exits with requested code

4. **Tests**
   - Unit tests for argument parsing and exit code masking
   - Integration tests for shell exit behavior

### Key Implementation Details

```rust
// Signature matches existing builtins pattern
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32>

// Argument handling:
// 1. No args → exit with executor.last_exit_code()
// 2. One numeric arg → exit with (arg & 0xFF)
// 3. One non-numeric arg → error "numeric argument required"
// 4. Multiple args → error "too many arguments"

// Exit signal (in error.rs):
pub enum RushError {
    // ... existing variants
    ExitRequest(i32),  // Signal to exit shell with code
}
```

## Dependencies

### Upstream Dependencies
- None - uses only existing CommandExecutor infrastructure

### Downstream Dependencies
- Feature 015 (source) already implemented - exit in sourced files will work correctly
- Script execution uses same executor, will handle exit correctly

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Exit signal not caught in all code paths | Medium | Test exit from interactive, script, and sourced contexts |
| Improper cleanup on exit | Low | Ensure destructors run; Rust's Drop handles this |
| Incompatibility with bash behavior | Low | Test against bash for key scenarios |

## Checklist

- [ ] `exit.rs` implements core functionality
- [ ] Exit command registered in dispatch
- [ ] ExitRequest error type added
- [ ] Main loop handles ExitRequest
- [ ] No-argument exit uses last exit code
- [ ] Explicit exit code masked to 0-255
- [ ] Error messages for invalid arguments
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Documentation updated
