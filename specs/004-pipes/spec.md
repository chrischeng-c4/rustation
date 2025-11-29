# Specification: Pipes and Multi-Command Pipelines

**Feature ID:** 004-pipes
**Status:** Complete (Implementation)
**Created:** 2025-11-21
**Updated:** 2025-11-29

## Overview

Implement pipe operators (`|`) for the rush shell, allowing commands to be chained together where the output of one command becomes the input of the next.

The pipe operator enables powerful data processing pipelines:
- `cat file.txt | grep "pattern"` - Find lines matching a pattern
- `ls -la | sort | head -10` - Sort and limit output
- `find . -name "*.txt" | wc -l` - Count matching files

## Motivation

Pipes are fundamental to shell usage and Unix philosophy. Without pipes, users cannot:
- Chain multiple commands together
- Process data through multiple filters
- Build complex command sequences
- Take full advantage of Unix tools

Pipes are among the first features users expect from any shell.

## User Stories

### US1: Basic Two-Command Pipeline ✅ IMPLEMENTED

**As a** shell user
**I want to** pipe the output of one command to another
**So that** I can chain two commands together for data processing

**Status**: ✅ Complete (PR #8, Commit 31a146a)

**Acceptance Criteria:**
- Pipe operator (`|`) is recognized outside quotes as a pipe operator
- First command's standard output connects to second command's standard input
- First command reads from terminal input, second command outputs to terminal
- Both commands execute concurrently in separate processes
- Exit code from the second (last) command is returned
- Pipes inside quotes (`"ls | grep"`) are treated as literal characters, not operators
- Signal propagation (Ctrl+C) terminates all processes in pipeline

**Examples:**
```bash
$ ls | grep txt
file1.txt
file2.txt
$ cat data.csv | grep "user"
user1,John,Doe
user2,Jane,Smith
$ ps aux | grep rush
chrischeng  1234  ... rush
```

**Implementation Notes:**
- `parser.rs` tokenizes pipe operators respecting quotes
- `pipeline.rs` creates pipe between processes using Unix pipes
- Signal propagation (FR-009) ensures no zombie processes

---

### US2: Multi-Command Pipelines ✅ IMPLEMENTED

**As a** shell user
**I want to** chain more than two commands together
**So that** I can build complex data processing pipelines

**Status**: ✅ Complete (PR #10, Commit cf0a1a8)

**Acceptance Criteria:**
- Support arbitrarily long pipelines (3, 4, 5, ... N commands)
- Each command's stdout connects to next command's stdin
- All commands execute concurrently in separate processes
- Each intermediate command reads from previous pipe, writes to next pipe
- Exit code from the last command is returned
- Signal propagation works correctly for all processes
- Works with any number of pipes

**Examples:**
```bash
$ cat file.txt | grep "pattern" | sort | uniq -c
$ find . -name "*.rs" | xargs wc -l | sort -n | tail -5
$ ps aux | grep -v grep | awk '{print $2}' | xargs kill -9
```

**Implementation Details:**
- Generalized pipeline architecture supports N commands
- Dynamic pipe creation: each command connects to previous/next
- Process group management ensures signals reach all processes
- No arbitrary limits on pipeline length

---

### US3: Pipeline Error Handling ⚠️ PARTIALLY IMPLEMENTED

**As a** shell user
**I want to** understand when pipeline commands fail
**So that** I can debug issues and know what went wrong

**Status**: ⚠️ Partial (Basic error handling exists, SIGPIPE not implemented)

**Acceptance Criteria (Partially Met):**
- ✅ Command not found in pipeline is reported with error message
- ✅ Permission denied errors are reported
- ⚠️ SIGPIPE handling for when reader closes pipe early (not implemented)
- ⚠️ Broken pipe detection and graceful handling (not implemented)
- ⚠️ Detailed error context showing which command in pipeline failed (partial)

**Examples:**
```bash
$ cat nonexistent.txt | grep foo
rush: cat: nonexistent.txt: No such file or directory

$ /no/such/cmd | grep foo
rush: /no/such/cmd: command not found

$ cat file.txt | head -1
# Works correctly - head closes pipe, cat gets SIGPIPE (should handle gracefully)
```

**Future Enhancement**: Implement SIGPIPE handling for graceful broken pipe scenarios

---

### US4: Exit Code Behavior ✅ IMPLEMENTED

**As a** shell script author
**I want to** reliable exit codes from pipelines
**So that** I can write reliable shell scripts

**Status**: ✅ Implemented (Returns last command's exit code)

**Acceptance Criteria:**
- ✅ Pipeline returns exit code from the last (rightmost) command
- ✅ Exit code is available for inspection in shell context
- ✅ Exit code 0 indicates success, non-zero indicates failure
- ✅ Matches bash behavior for pipeline exit codes
- ⚠️ Advanced: `PIPESTATUS` array not implemented (bash-specific feature)

**Examples:**
```bash
$ true | false
$ echo $?
1                    # Last command failed

$ false | true
$ echo $?
0                    # Last command succeeded

$ ls | grep nonexistent | head
$ echo $?
1                    # grep found no matches
```

**Rationale**: Bash returns the last command's exit code by default. Some shells offer `set -o pipefail` to fail on any command in pipeline, but that's an advanced feature.

---

## Technical Requirements

### Architecture

**Key Components:**
1. **Parser** (`parser.rs`)
   - Tokenize pipe operators (`|`) from command line
   - Respect quote boundaries (pipes in quotes are literal)
   - Handle escaped pipes (`\|`)

2. **Pipeline Executor** (`pipeline.rs`)
   - Create N-1 pipes for N commands
   - Spawn N child processes
   - Connect each process's stdin/stdout to appropriate pipes
   - Manage process lifecycle (fork, exec, wait)

3. **Integration** (`execute.rs`)
   - Call pipeline executor with parsed command list
   - Handle pipeline vs single command execution
   - Manage exit codes

### Data Flow

```
Command Line: "cmd1 | cmd2 | cmd3"
                 ↓
Parser: [Token::Word("cmd1"), Token::Pipe, Token::Word("cmd2"), ...]
                 ↓
Pipeline Executor:
  cmd1 stdout → [pipe1] → cmd2 stdin
  cmd2 stdout → [pipe2] → cmd3 stdin
  cmd3 stdout → terminal
                 ↓
Exit Code: cmd3's exit code
```

### Process Management

- Each command runs in separate process (fork)
- Parent process (shell) waits for all children
- Pipes are inherited across child processes
- Signal handling ensures all processes terminated on Ctrl+C

### Signal Handling (FR-009)

**Critical Requirement**: No zombie processes on pipeline failure

When Ctrl+C pressed:
1. Signal handler catches SIGINT
2. Terminates process group containing all pipeline processes
3. Parent reaps all child processes
4. Returns to prompt

## Success Metrics

1. **Functionality**
   - ✅ Two-command pipelines work correctly
   - ✅ Multi-command pipelines (3+ commands) work
   - ✅ Exit codes properly propagated
   - ✅ Signal handling prevents zombies

2. **Compatibility**
   - ✅ Behavior matches bash for common cases
   - ✅ Pipes outside quotes expand correctly
   - ✅ Pipes in quotes treated as literals
   - ✅ Works with all external commands

3. **Testing**
   - ✅ 10+ unit/integration tests
   - ✅ Tests for various pipeline lengths
   - ✅ Tests for error scenarios
   - ✅ Tests for signal handling

## Out of Scope

These features are NOT included and may be added later:
- `|&` operator (pipe stderr and stdout, bash 4.0+)
- `PIPESTATUS` array (bash-specific)
- Builtin commands in pipelines (only external commands tested)
- Process substitution (`<(cmd)`, `>(cmd)`)
- Redirection in pipelines (e.g., `cmd1 | cmd2 > file`)

## Dependencies

- `std::process` - Spawn and manage processes
- `std::os::unix` - Unix pipes and file descriptors
- `nix` crate - Signal handling (already in use)
- Existing parser and execution infrastructure

## Timeline

**Completed in**: v0.1.0
- **Specification**: Implicit (not spec-driven initially)
- **Implementation**: ~2 weeks (iterative development)
- **Testing**: Comprehensive (10+ tests)
- **Documentation**: Inline code comments

---

## Implementation Status

### ✅ Complete (100%)
- Basic two-command pipelines
- Multi-command pipelines (arbitrary length)
- Concurrent process execution
- Exit code propagation (last command)
- Signal handling (no zombie processes)
- Quote awareness (pipes in quotes are literal)
- Test coverage (10+ tests)

### ⚠️ Partial (0%)
- SIGPIPE handling
- Broken pipe detection
- Error context messages

### ❌ Not Implemented (0%)
- Stderr piping (`|&`)
- PIPESTATUS array
- Redirection in pipelines
- Builtin commands in pipelines

## Testing

**Existing Tests**: 10+ in `pipeline.rs`
- Single command execution
- Two-command pipelines
- Three-command pipelines
- Four-command pipelines
- Five-command pipeline (stress test)
- Exit code verification
- Command not found scenarios

**Test Coverage**: All basic functionality and multi-command support

---

## Summary

Pipes are **fully implemented and working** in rush v0.1.0. Both basic two-command and arbitrary N-command pipelines work correctly with proper signal handling. This specification documents the completed implementation for future reference and maintenance.
