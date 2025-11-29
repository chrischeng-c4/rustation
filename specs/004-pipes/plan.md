# Implementation Plan: Pipes and Multi-Command Pipelines

**Feature:** 004-pipes
**Status:** Complete (Implementation documented)
**Created:** 2025-11-21

## Overview

This plan documents the implementation approach for pipes. The feature is already complete and merged into main (PRs #8 and #10). This document serves as a reference for the existing implementation.

## Implementation Approach (Completed)

Pipes were implemented using Unix file descriptors to connect command I/O:

1. **Parser Phase** - Tokenize `|` operators respecting quotes
2. **Pipeline Setup** - Create intermediate pipes between commands
3. **Process Spawning** - Fork child processes with I/O redirected
4. **Signal Management** - Ensure all processes cleaned up on signals

## Architecture

### Module Structure
```
crates/rush/src/executor/
├── parser.rs        # Pipe token parsing
├── pipeline.rs      # Pipeline execution (531 lines)
├── execute.rs       # Integration point
└── mod.rs
```

### Core Components

**1. Parser (`parser.rs`)**
- **Pipe Token Recognition**
  ```rust
  pub enum Token {
      Word(String),
      Pipe,                 // New token type for |
      RedirectOut,          // Existing >
      RedirectAppend,       // Existing >>
      RedirectIn,           // Existing <
  }
  ```
- Recognizes `|` outside quotes as pipe operator
- Treats `|` inside quotes as literal text
- Handles escaped pipes (`\|`)

**2. Pipeline Executor (`pipeline.rs`)**

```rust
pub struct PipelineExecutor {
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
}

pub fn execute_pipeline(
    &mut self,
    commands: Vec<Vec<String>>,
) -> Result<i32> {
    // Implementation for N-command pipelines
}
```

Key Algorithm:
```
For N commands:
  Create N-1 pipes
  For each command i in 0..N:
    Spawn child process:
      - stdin: (i==0) ? terminal : pipe[i-1].read
      - stdout: (i==N-1) ? terminal : pipe[i].write
      - stderr: always terminal (unless redirected)
    Close pipe ends in parent (only parent has copies)
  Wait for all child processes
  Return exit code from last command
```

**3. Multi-Command Generalization (`pipeline.rs`)**

The implementation was generalized from 2-command to N-command:
- Original: Hard-coded for 2 commands (US1)
- Refactored: Loop over command list (US2)
- Result: Single implementation handles all cases

### Signal Handling (FR-009)

**Critical**: Ensure no zombie processes

```rust
// When SIGINT received:
signal_handler() {
    // Kill entire process group
    kill(-pgid, SIGINT);

    // Wait for all children
    loop {
        waitpid(-1, WNOHANG);
    }
}
```

All child processes inherit process group, so signal reaches all.

## Implementation Files

### Created Files
- **None** - Used existing parser and execution framework

### Modified Files
1. **`crates/rush/src/executor/parser.rs`**
   - Added `Token::Pipe` variant
   - Tokenization logic for `|` operator
   - Quote respecting (pipes in quotes are literals)

2. **`crates/rush/src/executor/pipeline.rs`**
   - New `PipelineExecutor` struct
   - `execute_pipeline()` function
   - Multi-command pipe creation logic
   - Child process spawning and I/O redirection
   - Signal handling and process cleanup

3. **`crates/rush/src/executor/execute.rs`**
   - Integration with parser
   - Decision point: single command vs pipeline
   - Call `pipeline_executor.execute_pipeline()`

4. **`crates/rush/src/executor/mod.rs`**
   - Added `pipeline` module
   - Exported `PipelineExecutor`

## Testing Strategy

### Unit Tests (in `pipeline.rs`)
- 10 tests covering all scenarios

**Test Coverage:**
1. `test_execute_single_command` - No pipes
2. `test_execute_true` - Basic exit code 0
3. `test_execute_false` - Basic exit code 1
4. `test_execute_two_command_pipeline` - US1 basic case
5. `test_execute_pipeline_with_grep` - Real world example
6. `test_three_command_pipeline` - US2 with 3 commands
7. `test_four_command_pipeline` - US2 with 4 commands
8. `test_long_pipeline` - US2 with 5 commands
9. `test_execute_pipeline_command_not_found` - Error handling
10. Additional error handling tests

### Integration Points
- Called from `execute()` main function
- Returns exit code like any other command
- Works with existing signal handling

## Key Design Decisions

### 1. Last Command Exit Code
**Decision**: Return exit code from last command (not first, not PIPESTATUS)
**Rationale**: Matches bash default behavior; simpler for users

### 2. Concurrent Execution
**Decision**: All commands run concurrently, not sequentially
**Rationale**: Unix pipes are designed for concurrent execution; matches shell behavior

### 3. Stderr Handling
**Decision**: Stderr not piped (always goes to terminal)
**Rationale**: Matches bash default; allows debugging; simplifies implementation

### 4. Process Group
**Decision**: All processes in same process group
**Rationale**: Signals reach all processes; prevents zombie children; standard practice

## Future Enhancements

### US3: SIGPIPE Handling
**Goal**: Gracefully handle `head -n 1 < large_file | cat`

**When**: When reader closes pipe before writer sends all data

**Implementation**:
```rust
// Ignore SIGPIPE - let write() return error instead
signal(SIGPIPE, SIG_IGN);

// In command, check for write errors
match write(pipe, data) {
    Err(BrokenPipe) => {
        // Reader closed pipe, exit gracefully
        exit(0);
    }
}
```

**Complexity**: Low - just ignore signal and handle write errors

### Stderr Piping (|&)
**Goal**: `cmd1 |& cmd2` pipes both stdout and stderr

**Implementation**:
```rust
// Redirect stderr to pipe along with stdout
stderr: pipe[i].write
```

**Complexity**: Low - just add to token parsing and I/O setup

### PIPESTATUS Array
**Goal**: `pipestatus=(0 1 2 3)` shows each command's exit code

**Implementation**: Store all exit codes, expose via variable expansion

**Complexity**: Medium - need to track and expose multiple exit codes

## Code Quality

### Testing
- ✅ 10+ existing tests
- ✅ Error scenarios covered
- ✅ Multi-command cases tested

### Code Comments
- ✅ Well-documented functions
- ✅ Algorithm explanations
- ✅ FR-009 signal handling documented

### Clippy/Formatting
- ✅ No clippy warnings
- ✅ Code formatted with cargo fmt

## Git History

**PR #8** - "004-pipes-us1-basic-two-command"
- Commit: `31a146a` (Nov 21, 2025)
- Lines: 852 changed
- Features: Two-command pipelines, signal handling (FR-009)

**PR #10** - "004-pipes-us2-multi-command"
- Commit: `cf0a1a8` (Nov 22, 2025)
- Lines: -31 net (refactoring/generalization)
- Features: N-command pipelines, maintained FR-009

## Completion Status

**Status**: ✅ COMPLETE

**What's Done**:
- ✅ US1: Two-command pipelines
- ✅ US2: Multi-command pipelines
- ✅ Exit code behavior
- ✅ Signal handling (FR-009)
- ✅ Test coverage (10+ tests)
- ✅ Quote awareness

**What's Missing** (Future):
- ⏳ US3: SIGPIPE handling
- ⏳ Stderr piping (|&)
- ⏳ PIPESTATUS array
- ⏳ Builtin commands in pipelines

## Summary

Pipes are **production-ready** in rush. Both basic and multi-command pipelines work correctly with proper signal handling. The implementation is clean, well-tested, and follows Unix conventions.

This specification documents a completed feature for:
1. **Maintenance** - Understanding the existing code
2. **Reference** - Design patterns used
3. **Future Enhancement** - Where to add SIGPIPE, PIPESTATUS, etc.
4. **Spec Compliance** - Fulfills MVP requirements for piping
