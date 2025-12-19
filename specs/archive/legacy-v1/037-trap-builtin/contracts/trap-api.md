# Trap Builtin API Contract

**Version**: 1.0.0
**Date**: 2025-12-10
**Feature**: Trap Builtin (037)
**Status**: Design

## Overview

This document specifies the public API contract for the `trap` builtin command in rush shell. The contract defines syntax, behavior, error handling, and output format to ensure consistent behavior and enable testing.

## Command Syntax

### Register Trap Handler

**Syntax**:
```bash
trap COMMAND SIGNAL [SIGNAL...]
```

**Parameters**:
- `COMMAND`: Shell command string to execute when signal received (required, non-empty)
- `SIGNAL`: Signal specification(s) - one or more (required)

**Signal Specifications** (case-insensitive):
- Signal names: `INT`, `SIGINT`, `TERM`, `SIGTERM`, `HUP`, `USR1`, etc.
- Signal numbers: `2`, `15`, `1`, `10` (POSIX signal numbers)
- Pseudo-signals: `EXIT` (executed on shell termination)
- Real-time signals: `RTMIN`, `RTMAX`, `RTMIN+N`, `RTMAX-N`

**Examples**:
```bash
trap 'rm /tmp/lockfile' INT           # Register SIGINT handler
trap 'cleanup_function' TERM QUIT     # Register same handler for multiple signals
trap 'echo Exiting' EXIT              # Register EXIT pseudo-signal handler
trap 'logger "Got USR1"' SIGUSR1      # Register SIGUSR1 handler
trap 'handler' 2                      # Register using signal number
```

**Return Values**:
- `0`: Success - handler registered
- `1`: Error - invalid signal, uncatchable signal, or duplicate trap

**Error Conditions**:
| Condition | Error Message | Exit Code |
|-----------|---------------|-----------|
| Invalid signal name | `trap: invalid signal specification: <name>` | 1 |
| Invalid signal number | `trap: invalid signal specification: <number>` | 1 |
| Uncatchable signal (KILL/STOP) | `trap: cannot trap <signal>: signal cannot be caught` | 1 |
| Duplicate trap (FR-006) | `trap: trap already exists for signal <signal> (use 'trap "" <signal>' to clear first)` | 1 |
| Empty command | `trap: empty command (use 'trap "" SIGNAL' to clear)` | 1 |
| Missing arguments | `trap: usage: trap COMMAND SIGNAL [SIGNAL...]` | 1 |

### List Trap Handlers

**Syntax**:
```bash
trap
```

**Parameters**: None

**Output Format** (one line per trap):
```
trap -- 'COMMAND' SIGNAL
```

**Examples**:
```bash
$ trap 'rm /tmp/lock' INT
$ trap 'echo Exiting' EXIT
$ trap
trap -- 'rm /tmp/lock' INT
trap -- 'echo Exiting' EXIT
```

**Special Cases**:
- No traps registered: Empty output (no lines)
- Multiple traps: Listed in signal number order (ascending)
- EXIT trap: Listed last

**Return Values**:
- `0`: Always succeeds

### Clear Trap Handler

**Syntax**:
```bash
trap "" SIGNAL [SIGNAL...]
trap '' SIGNAL [SIGNAL...]
```

**Parameters**:
- `""` or `''`: Empty command string (required, signals clearing)
- `SIGNAL`: Signal specification(s) to clear (required)

**Examples**:
```bash
trap "" INT                 # Clear SIGINT handler
trap '' TERM QUIT           # Clear SIGTERM and SIGQUIT handlers
trap "" EXIT                # Clear EXIT pseudo-signal handler
```

**Behavior**:
- Removes trap handler for specified signal(s)
- Restores default signal behavior
- Succeeds silently if no handler exists (idempotent)

**Return Values**:
- `0`: Always succeeds (even if no handler existed)

## Behavior Specifications

### Signal Handler Execution

**When**:
- Real signals: Executed when process receives the signal
- EXIT pseudo-signal: Executed when shell terminates (normal exit, error, or signal)

**How**:
- Command executed in current shell context
- Has access to all shell variables and functions
- Runs synchronously (shell waits for handler to complete)
- Handler exit code ignored (does not affect shell exit code)
- If handler fails, error logged but shell continues

**Timing** (Performance Targets):
- Handler invocation: Within 100ms of signal delivery (SC-002)
- Listing operation: Complete in <5 seconds (SC-005)
- Clearing operation: Instantaneous <1ms (SC-008)

**Execution Context**:
- Current working directory: Preserved
- Environment variables: Available to handler
- Redirections: Handler can use I/O redirection
- Exit status: Handler runs in subshell, exit code discarded

**Example Flow**:
```bash
$ trap 'echo "Caught INT" >> /tmp/log; cleanup' INT
$ # User presses Ctrl+C
# 1. SIGINT delivered to shell
# 2. Handler executed: echo "Caught INT" >> /tmp/log; cleanup
# 3. Shell continues (does not terminate)
```

### EXIT Handler Timing

**Execution Order**:
1. Last command in script completes (or signal terminates shell)
2. Real signal traps execute (if signal was cause of exit)
3. EXIT trap executes
4. Shell cleanup (close files, restore terminal)
5. Shell process terminates

**Example**:
```bash
trap 'echo "Cleanup" >> log' EXIT
trap 'echo "Got TERM" >> log' TERM
# User sends SIGTERM
# Output in log:
#   Got TERM
#   Cleanup
```

**Constraints**:
- EXIT handler executes exactly once per shell termination
- If EXIT handler exits or errors, shell terminates immediately
- No re-triggering if EXIT handler is interrupted

### Multiple Signal Registration

**Syntax**:
```bash
trap 'command' SIG1 SIG2 SIG3
```

**Behavior**:
- Registers same command for all specified signals
- All-or-nothing: If any signal already has handler, entire command fails (FR-006)
- Returns error code 1 if any registration fails

**Example**:
```bash
$ trap 'handler1' INT
$ trap 'handler2' INT TERM    # Fails - INT already has handler
trap: trap already exists for signal INT (use 'trap "" INT' to clear first)
```

## Signal Coverage

### Supported Signals

**POSIX Standard Signals** (SC-001):
```
SIGHUP    (1)   - Terminal hangup
SIGINT    (2)   - Interrupt (Ctrl+C)
SIGQUIT   (3)   - Quit (Ctrl+\)
SIGTERM   (15)  - Termination
SIGUSR1   (10)  - User-defined 1
SIGUSR2   (12)  - User-defined 2
SIGPIPE   (13)  - Broken pipe
SIGALRM   (14)  - Alarm clock
SIGCHLD   (20)  - Child status changed
SIGCONT   (19)  - Continue if stopped
SIGTSTP   (18)  - Stop (Ctrl+Z)
SIGTTIN   (21)  - Background read
SIGTTOU   (22)  - Background write
```

**Real-Time Signals** (SC-001, FR-002):
```
SIGRTMIN        - Minimum RT signal (typically 34)
SIGRTMAX        - Maximum RT signal (typically 64)
SIGRTMIN+N      - RT signal N above minimum
SIGRTMAX-N      - RT signal N below maximum
```

**Pseudo-Signals**:
```
EXIT            - Shell termination (any cause)
```

### Unsupported Signals (Uncatchable)

**SIGKILL** (9):
- Cannot be caught, blocked, or ignored per OS restrictions
- `trap 'cmd' KILL` returns error (SC-003)

**SIGSTOP** (19):
- Cannot be caught, blocked, or ignored per OS restrictions
- `trap 'cmd' STOP` returns error (SC-003)

## Edge Case Behaviors

### Handler Execution Failures

**Scenario**: Handler command doesn't exist or has syntax error

**Behavior**:
```bash
$ trap 'nonexistent_command' INT
$ # User presses Ctrl+C
trap: nonexistent_command: command not found
# Shell continues (signal still considered handled)
```

**Spec**: Error logged but doesn't crash shell

### Signal Name Variations

**Case Insensitivity**:
```bash
trap 'cmd' int     # ✅ Accepted (lowercase)
trap 'cmd' INT     # ✅ Accepted (uppercase)
trap 'cmd' Int     # ✅ Accepted (mixed case)
trap 'cmd' SIGINT  # ✅ Accepted (SIG prefix)
trap 'cmd' sigint  # ✅ Accepted (lowercase with prefix)
```

All forms resolve to same signal.

### Empty Signal List

**Scenario**: User runs `trap 'command'` with no signals

**Behavior**:
```bash
$ trap 'cleanup'
trap: usage: trap COMMAND SIGNAL [SIGNAL...]
```

**Exit Code**: 1

### Real-Time Signal Boundaries

**Valid**:
```bash
trap 'cmd' RTMIN       # ✅ First RT signal
trap 'cmd' RTMAX       # ✅ Last RT signal
trap 'cmd' RTMIN+5     # ✅ RTMIN + 5
trap 'cmd' RTMAX-2     # ✅ RTMAX - 2
```

**Invalid**:
```bash
trap 'cmd' RTMIN-1     # ❌ Below minimum
trap 'cmd' RTMAX+1     # ❌ Above maximum
trap 'cmd' RTMIN+99    # ❌ Out of range
```

## Testing Contract

### Unit Test Requirements

**Signal Parsing**:
- Parse valid signal names (INT, SIGINT, int)
- Parse valid signal numbers (2, 15)
- Reject invalid names (SIGFOO, XYZ)
- Reject invalid numbers (-1, 999)
- Reject uncatchable signals (KILL, STOP, 9, 19)

**Registry Operations**:
- Register handler succeeds
- Duplicate registration fails with error
- Clear handler succeeds
- Clear non-existent handler succeeds (idempotent)
- List empty registry returns empty
- List populated registry returns correct format

### Integration Test Requirements

**Signal Delivery**:
- Register trap → send signal → verify handler executed
- Register multiple traps → send signals → verify each executes
- Clear trap → send signal → verify default behavior

**EXIT Handler**:
- Normal exit → EXIT handler executes
- Error exit → EXIT handler executes
- Signal termination → EXIT handler executes after signal handler

**Error Scenarios**:
- Invalid signal → error message matches contract
- Duplicate trap → error message includes clear suggestion
- Uncatchable signal → specific error for KILL/STOP

## Compatibility

### POSIX Compatibility

**Differences from POSIX**:
- Duplicate trap registration: POSIX allows overwrite, rush requires explicit clear (FR-006)
- This is an intentional design choice documented in spec clarifications

**Alignment with POSIX**:
- Signal name parsing (with/without SIG prefix)
- EXIT pseudo-signal behavior
- Handler execution context
- Output format for listing

### Bash/Zsh Compatibility

**Similarities**:
- Syntax: `trap 'command' SIGNAL`
- Signal names and numbers accepted
- Case-insensitive parsing
- Multiple signals in one command

**Differences**:
- Duplicate behavior (bash allows overwrite, rush errors)
- This is intentional and documented

## Versioning

**Version**: 1.0.0 (initial implementation)

**Future Compatibility**:
- Breaking changes will increment major version
- New pseudo-signals (DEBUG, ERR, RETURN) will increment minor version
- Bug fixes will increment patch version

## Performance Guarantees

Per Success Criteria:

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Handler execution | <100ms | Signal delivery to handler completion (SC-002) |
| Trap listing | <5 seconds | `trap` command completion (SC-005) |
| Trap clearing | Instantaneous | <1ms (SC-008) |
| Signal validation | <1ms | Parse and validate signal spec |

## Security Considerations

**Command Injection**: Trap commands are shell commands, subject to normal shell escaping rules

**Signal Safety**: Handlers execute in safe context (main loop), not in async signal handler

**Resource Limits**: Maximum 63 simultaneous traps (all POSIX + RT signals)

## API Stability

**Stable APIs** (will not change in 1.x):
- Command syntax: `trap COMMAND SIGNAL [SIGNAL...]`
- Output format: `trap -- 'COMMAND' SIGNAL`
- Error message format
- Return values (0 for success, 1 for error)

**Internal APIs** (may change):
- TrapRegistry implementation details
- Signal handler registration mechanism
- Internal data structures
