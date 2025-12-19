# Feature Specification: Trap Builtin

**Feature Branch**: `037-trap-builtin`
**Created**: 2025-12-09
**Status**: Draft
**Input**: User description: "trap builtin for signal handling"

## Clarifications

### Session 2025-12-10

- Q: Which signal handling scope should rush support for the MVP? → A: Full signal support including real-time signals (RTMIN-RTMAX)
- Q: When multiple traps are set for the same signal, what should be the behavior? → A: Error if trap already exists for that signal
- Q: How should trap handlers access the signal that triggered them? → A: No signal info passed (handler runs as-is)
- Q: What should `trap` without arguments display? → A: List all currently set traps (signal + command)
- Q: How should users clear/remove an existing trap handler? → A: Use empty command: `trap "" SIGNAL`

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Register Cleanup Handlers (Priority: P1)

As a shell script author, I need to register signal handlers to perform cleanup operations (removing temporary files, closing connections, logging) when my script receives interruption signals like SIGINT (Ctrl+C) or SIGTERM.

**Why this priority**: This is the core use case for trap - ensuring scripts clean up properly when interrupted. Without this, scripts leave orphaned resources and inconsistent state.

**Independent Test**: Can be fully tested by registering a trap handler that creates a cleanup marker file, sending SIGINT to the script, and verifying the marker file was created before script termination.

**Acceptance Scenarios**:

1. **Given** a script with `trap 'rm /tmp/lockfile' INT`, **When** user presses Ctrl+C (SIGINT), **Then** the cleanup command executes and removes the lockfile before script exits
2. **Given** a script with `trap 'echo "Exiting..." >> log.txt' TERM`, **When** script receives SIGTERM, **Then** the message is logged before termination
3. **Given** a script with `trap 'cleanup_function' EXIT`, **When** script finishes normally or abnormally, **Then** the cleanup function executes
4. **Given** no trap set for SIGINT, **When** user presses Ctrl+C, **Then** script terminates immediately without executing any handler

---

### User Story 2 - Inspect Active Traps (Priority: P2)

As a shell user debugging complex scripts, I need to view all currently registered trap handlers to understand what cleanup or signal handling is configured.

**Why this priority**: Essential for debugging and understanding script behavior, especially in interactive sessions or when sourcing multiple scripts with potential trap conflicts.

**Independent Test**: Can be fully tested by registering several traps (e.g., INT, TERM, EXIT), running `trap` with no arguments, and verifying all registered handlers are listed with their associated signals.

**Acceptance Scenarios**:

1. **Given** traps set for INT and TERM signals, **When** user runs `trap` with no arguments, **Then** output lists both handlers with format "trap -- 'command' SIGNAL"
2. **Given** no traps are set, **When** user runs `trap`, **Then** output is empty or shows no active traps
3. **Given** a trap for EXIT pseudo-signal, **When** user runs `trap`, **Then** EXIT trap is included in the listing

---

### User Story 3 - Clear Trap Handlers (Priority: P3)

As a shell script author, I need to remove previously registered trap handlers to restore default signal behavior or disable cleanup that's no longer needed.

**Why this priority**: Allows dynamic trap management in long-running scripts or interactive sessions. Lower priority because most scripts set traps once and leave them active.

**Independent Test**: Can be fully tested by setting a trap for SIGINT, clearing it with `trap "" INT`, sending SIGINT, and verifying the default behavior (script termination) occurs without executing the handler.

**Acceptance Scenarios**:

1. **Given** a trap registered for INT signal, **When** user runs `trap "" INT`, **Then** the trap is removed and `trap` listing no longer shows INT
2. **Given** a trap cleared with `trap "" INT`, **When** script receives SIGINT, **Then** default signal behavior occurs (immediate termination without handler)
3. **Given** attempting to clear a non-existent trap with `trap "" USR1`, **When** command executes, **Then** operation succeeds silently (no error)

### Edge Cases

#### Uncatchable Signals (SIGKILL, SIGSTOP)
- **Scenario**: User attempts `trap 'cleanup' KILL` or `trap 'handler' STOP`
- **Expected**: System returns error message indicating SIGKILL and SIGSTOP cannot be trapped (OS restriction)
- **Rationale**: These signals cannot be caught, blocked, or ignored by design for system stability

#### Invalid Signal Names/Numbers
- **Scenario**: User runs `trap 'cmd' INVALID` or `trap 'cmd' 9999`
- **Expected**: System returns error "invalid signal specification: INVALID/9999" and does not register any handler
- **Examples**: Invalid names (SIGFOO, XYZ), out-of-range numbers (negative, > max signal number)

#### Duplicate Trap Registration
- **Scenario**: User sets `trap 'first' INT` then attempts `trap 'second' INT` without clearing
- **Expected**: System returns error "trap already exists for signal INT" (per FR-006)
- **Note**: User must first clear with `trap "" INT` before setting new handler

#### Signal Handler Execution Failures
- **Scenario**: Trap command `trap 'nonexistent_command' INT` triggers when handler doesn't exist
- **Expected**: Error message logged/displayed, handler execution fails, but signal is still considered handled (script doesn't terminate)
- **Scenario**: Handler command has syntax error `trap 'echo "missing quote' INT`
- **Expected**: Syntax error reported when signal fires, original signal handling interrupted

#### EXIT Pseudo-Signal Timing
- **Scenario**: Script with `trap 'cleanup' EXIT` terminates normally (reaches end)
- **Expected**: EXIT handler executes exactly once after last command, before shell process termination
- **Scenario**: Script with EXIT trap receives SIGTERM while also having TERM trap
- **Expected**: TERM handler executes first, then EXIT handler executes during shutdown
- **Scenario**: EXIT handler itself exits or receives signal
- **Expected**: EXIT handler runs once; if it exits prematurely, shell terminates (no re-triggering)

#### Multiple Signals in Single Command
- **Scenario**: User runs `trap 'handler' INT TERM QUIT`
- **Expected**: System registers the same handler for all three signals (or returns error per FR-006 if any already have handlers)

#### Real-Time Signals Boundary
- **Scenario**: User references `trap 'handler' RTMIN` and `trap 'handler' RTMAX`
- **Expected**: System correctly resolves RTMIN and RTMAX to platform-specific signal numbers and registers handlers
- **Platform variation**: Number of real-time signals varies by OS (typically SIGRTMIN through SIGRTMIN+31)

#### Empty Signal List
- **Scenario**: User runs `trap 'command'` with no signal specified
- **Expected**: System returns error "usage: trap command signal..." indicating signal argument is required

#### Signal Name Case Sensitivity
- **Scenario**: User tries `trap 'cmd' int` (lowercase) vs `trap 'cmd' INT` (uppercase)
- **Expected**: System accepts both (case-insensitive matching) or normalizes to uppercase internally

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST support all POSIX standard signals (HUP, INT, QUIT, TERM, KILL, etc.)
- **FR-002**: System MUST support real-time signals (SIGRTMIN through SIGRTMAX)
- **FR-003**: System MUST support the EXIT pseudo-signal for cleanup handlers
- **FR-004**: System MUST support signal names (e.g., "INT") and numbers (e.g., "2")
- **FR-005**: System MUST allow registering, modifying, and clearing trap handlers
- **FR-006**: System MUST return an error if attempting to set a trap for a signal that already has a handler registered
- **FR-007**: System MUST execute trap handlers as-is without passing signal information as parameters or special variables
- **FR-008**: System MUST list all currently set traps (signal and command) when `trap` is invoked without arguments
- **FR-009**: System MUST allow clearing trap handlers using empty command syntax: `trap "" SIGNAL`
- **FR-010**: System MUST reject attempts to trap SIGKILL and SIGSTOP with clear error messages
- **FR-011**: System MUST validate signal names and numbers, returning errors for invalid specifications
- **FR-012**: System MUST accept signal specifications in multiple formats (INT, SIGINT, int, 2) with case-insensitive matching
- **FR-013**: System MUST support registering the same handler for multiple signals in a single command
- **FR-014**: System MUST execute EXIT handlers exactly once during shell termination, after all other trap handlers

### Key Entities

- **Trap Handler**: Represents a registered signal handler
  - Signal identifier (name or number)
  - Command string to execute when signal received
  - Registration status (active/cleared)
  - Cannot exist for SIGKILL or SIGSTOP

- **Signal Registry**: Collection of all active trap handlers
  - Maps signal identifiers to their handler commands
  - Enforces uniqueness (one handler per signal per FR-006)
  - Persists for lifetime of shell session
  - Cleared on shell exit

- **Signal Specification**: User input identifying a signal
  - Can be signal name (INT, TERM, HUP) with or without SIG prefix
  - Can be signal number (2, 15, 1)
  - Can be pseudo-signal (EXIT)
  - Can be symbolic constant (RTMIN, RTMAX)
  - Must resolve to valid, catchable signal

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can register signal handlers for all POSIX standard signals (SIGHUP through SIGUSR2) and all available real-time signals on the platform
- **SC-002**: Trap handlers execute within 100ms of signal receipt, ensuring cleanup operations complete before script termination
- **SC-003**: System correctly rejects 100% of attempts to trap uncatchable signals (SIGKILL, SIGSTOP) with clear error messages
- **SC-004**: EXIT pseudo-signal handlers execute exactly once per script termination with 100% reliability across normal exits, error exits, and signal-induced terminations
- **SC-005**: Users can inspect all active traps in under 5 seconds using parameterless `trap` command, receiving clearly formatted output listing all registered handlers
- **SC-006**: 100% of duplicate trap registration attempts are detected and rejected with informative error messages before modifying existing handlers
- **SC-007**: System correctly handles all signal name formats (INT, SIGINT, int, sigint, 2) with case-insensitive matching and accepts both name and number specifications
- **SC-008**: Trap clearing operations complete instantaneously and restore default signal behavior for 100% of previously trapped signals
