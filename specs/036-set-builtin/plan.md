# Implementation Plan: Set Builtin for Shell Options

**Branch**: `036-set-builtin` | **Date**: 2025-12-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/036-set-builtin/spec.md`

## Summary

Extend the existing `set` builtin command to support POSIX shell options (`-e`, `-x`, `-o pipefail`) for error handling, command tracing, and pipeline failure detection. The implementation adds ShellOptions struct to CommandExecutor, modifies set builtin for option parsing, and integrates option checks into the execution pipeline. Primary value: defensive scripting, debugging support, and reliable error handling for production scripts.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: None (pure Rust std library)
**Storage**: In-memory (ShellOptions struct in CommandExecutor)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: macOS (MVP), Linux (post-MVP)
**Project Type**: Single Rust monorepo (crates/rush)
**Performance Goals**: <1ms option check overhead per command
**Constraints**: Option state persists across commands, errexit respects conditionals
**Scale/Scope**: 3 options (errexit, xtrace, pipefail), ~350 lines total

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### ✅ Principle I: Performance-First
**Check**: <5ms overhead per command?
- **Result**: PASS - O(1) boolean flag access, <0.1ms overhead

### ✅ Principle II: Zero-Config Philosophy
**Check**: Works without configuration?
- **Result**: PASS - All options default to "off" (bash-compatible)

### ✅ Principle III: Progressive Complexity
**Check**: Simple by default, powerful when needed?
- **Result**: PASS - Options are opt-in advanced features

### ✅ Principle IV: Modern UX
**Check**: Polished and intuitive?
- **Result**: PASS - Follows POSIX/bash conventions

### ✅ Principle V: Rust-Native
**Check**: Pure Rust without FFI?
- **Result**: PASS - No external dependencies, safe Rust only

**GATE RESULT**: ✅ ALL CHECKS PASSED

## Project Structure

### Documentation (this feature)

```text
specs/036-set-builtin/
├── spec.md              # Completed
├── plan.md              # This file
├── research.md          # Phase 0 (will generate)
├── data-model.md        # Phase 1 (will generate)
├── quickstart.md        # Phase 1 (will generate)
└── tasks.md             # Phase 2 (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/rush/src/executor/
├── execute.rs                 # Modified: Add ShellOptions, integrate checks
├── builtins/
│   ├── mod.rs                 # Modified: Add "set" to dispatch (missing!)
│   └── set.rs                 # Modified: Extend for shell options
└── pipeline.rs                # Modified: Add pipefail support
```

**Structure Decision**: Single project - extends existing executor infrastructure.

## Deployment Strategy

**Strategy**: PR per User Story (Option 2)

**Rationale**: 4 independent user stories (~75-100 lines each), total ~350 lines. Breaking into PRs allows incremental delivery, easier review, and independent testing.

### Selected Strategy

```
PR #1: Foundation (~100 lines)
  - Add ShellOptions struct
  - Add set dispatch to builtins/mod.rs
  - Basic option parsing framework
  Target: ≤ 500 lines ✅

PR #2: Exit on Error -e (~100 lines)
  - Errexit implementation
  - Conditional context exceptions
  Target: ≤ 500 lines ✅

PR #3: Command Tracing -x (~80 lines)
  - Xtrace to stderr with + prefix
  Target: ≤ 500 lines ✅

PR #4: Pipeline Failure -o pipefail (~70 lines)
  - Pipeline exit code tracking
  Target: ≤ 500 lines ✅

PR #5: Query Options (~50 lines)
  - set -o/+o output
  Target: ≤ 500 lines ✅
```

### Merge Sequence

1. **PR #1: Foundation** → main (no behavior change yet)
2. **PR #2: Errexit** → main (P1 feature delivered)
3. **PR #3: Xtrace** → main (P2 debugging)
4. **PR #4: Pipefail** → main (P2 reliability)
5. **PR #5: Query** → main (P3 introspection)

**Branch Strategy**: Create `036-set-builtin` base, then sub-branches per PR.

### PR Size Validation

```bash
git diff --stat main  # Check before each PR
```

Expected sizes: 100+100+80+70+50 = 400 lines total ✅

## Architecture Overview

### High-Level Design

```text
User: "set -ex"
  ↓
CommandExecutor.shell_options
  ├── errexit: bool
  ├── xtrace: bool
  └── pipefail: bool
  ↓
execute() integration:
  1. [Before] if xtrace { print "+ cmd" }
  2. [Execute] run command
  3. [After] if errexit && exit!=0 && !conditional { exit() }
  4. [Pipeline] if pipefail { return first_nonzero }
```

### Data Flow

**Setting option**:
```
set -e → parse_args(["-e"]) → set_option("errexit", true)
```

**Executing with errexit**:
```
false → exit_code=1 → if errexit && !conditional { exit(1) }
```

**Pipeline with pipefail**:
```
false | true → [1,0] → if pipefail { return 1 } else { return 0 }
```

### Integration Points

**1. CommandExecutor** (execute.rs):
```rust
pub struct CommandExecutor {
    // ... existing fields
    shell_options: ShellOptions,  // NEW
}

struct ShellOptions {
    errexit: bool,
    xtrace: bool,
    pipefail: bool,
}
```

**2. execute()** (execute.rs ~line 80):
```rust
// NEW: Xtrace before execution
if self.shell_options.xtrace {
    eprintln!("+ {}", cmd);
}

let exit_code = /* execute */;

// NEW: Errexit after execution
if self.shell_options.errexit && exit_code != 0 && !self.in_conditional() {
    std::process::exit(exit_code);
}
```

**3. Pipeline** (pipeline.rs):
```rust
// NEW: Pipefail support
if self.shell_options.pipefail {
    exit_codes.iter().find(|&&c| c != 0).copied().unwrap_or(0)
} else {
    *exit_codes.last().unwrap_or(&0)
}
```

**4. Builtin dispatch** (builtins/mod.rs line 38):
```rust
match command {
    "set" => Some(set::execute(executor, args)),  // ADD THIS
    // ... other builtins
}
```

## Key Design Decisions

### Decision 1: Shell Options Storage

**Chosen**: ShellOptions struct on CommandExecutor

**Rationale**:
- O(1) access, zero allocation
- Thread-safe (each executor has own)
- Easy to test
- Subshell-friendly (future)

**Alternatives Rejected**:
- Global static: Not thread-safe
- Environment variables: Too slow
- Separate manager: Over-engineered

### Decision 2: Conditional Context Detection

**Chosen**: conditional_depth counter

**Implementation**:
```rust
struct CommandExecutor {
    conditional_depth: usize,  // >0 = inside conditional
}

// Before if/while condition:
self.conditional_depth += 1;
execute(condition);
self.conditional_depth -= 1;

// After command:
if errexit && exit!=0 && conditional_depth==0 {
    exit(exit_code);
}
```

**Rationale**: Simple, accurate, O(1) performance

### Decision 3: Xtrace Format

**Chosen**: Print to stderr with `+` prefix (bash style)

**Format**:
```bash
$ set -x
$ echo hello
+ echo hello      # stderr
hello             # stdout
```

**Rationale**: POSIX compatible, separates trace from output

### Decision 4: Option Syntax

**Chosen**: Both short (-e) and long (-o errexit) forms

**Mapping**:
```
-e = -o errexit
-x = -o xtrace
-o pipefail (no short form)
```

**Rationale**: POSIX compliance, bash compatibility

## Implementation Strategy

### Phase 0: Research

**Questions**:
1. Bash errexit behavior in nested conditionals?
2. Xtrace format for pipelines/loops?
3. Pipefail with errexit interaction?

**Output**: research.md with bash behavior documentation

### Phase 1: Foundation (PR #1)

1. Create ShellOptions struct
2. Add to CommandExecutor
3. Extend set::execute() for option parsing
4. Add set to builtins dispatch
5. Unit tests

**Deliverable**: Options parsed and stored, no behavior change

### Phase 2-5: Implement Each Option (PR #2-5)

Each PR implements one user story with tests.

### Testing Strategy

**Unit Tests**:
- Option parsing
- ShellOptions methods
- Conditional depth tracking

**Integration Tests**:
- Scripts with set -e exit on errors
- set -x prints commands
- set -o pipefail detects failures

## Edge Cases

1. **Combined options**: `set -ex` → parse each character
2. **Mixed enable/disable**: `set -e +x` → process sequentially
3. **Invalid option**: Print error, return 1
4. **Errexit in subshell**: Subshell exits, parent continues
5. **Xtrace with redirection**: Follows stderr redirect
6. **Pipefail all success**: Returns 0
7. **Errexit with negation**: `! false` continues (conditional)
8. **Option persistence**: Persist across commands

## Success Metrics

- <0.5ms option check overhead
- 100% conditional context tests pass
- 90%+ bash behavioral parity
- Test coverage >85%
- All clippy warnings addressed

## Known Limitations

1. Subshell option inheritance not in MVP
2. PS4 customization not supported
3. Only 3 options (nounset/noglob/noclobber later)
4. Options not saved to config

## Dependencies

**Code**:
- CommandExecutor (execute.rs)
- PipelineExecutor (pipeline.rs)
- set builtin (builtins/set.rs)
- Builtin dispatch (builtins/mod.rs)

**Features**:
- 004 (pipes) - for pipefail
- 017 (conditionals) - for errexit exceptions

## Post-Implementation

1. Version → 0.36.0
2. features.json → mark 036 complete
3. Tag v0.36.0
4. Update CLAUDE.md with examples

## Future Enhancements

1. More options (nounset, noglob)
2. Subshell inheritance
3. PS4 customization
4. Config persistence
