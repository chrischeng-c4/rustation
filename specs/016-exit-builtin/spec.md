# Feature Specification: Exit Builtin Command

**Feature Branch**: `016-exit-builtin`
**Created**: 2024-12-04
**Status**: Draft
**Input**: User description: "Implement exit builtin command that terminates the shell with an optional exit code"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Exit Shell with Default Status (Priority: P1)

As a shell user, I want to type `exit` to terminate the shell session, returning the exit status of the last executed command to the parent process.

**Why this priority**: Core exit functionality - essential for any shell to terminate properly. Without this, users cannot gracefully close the shell.

**Independent Test**: Type `exit` in an interactive shell session and verify the shell terminates and returns to the parent process.

**Acceptance Scenarios**:

1. **Given** an interactive shell session with last command exit code 0, **When** user types `exit`, **Then** shell terminates and returns exit code 0 to parent process
2. **Given** an interactive shell session where last command was `false` (exit code 1), **When** user types `exit`, **Then** shell terminates and returns exit code 1 to parent process

---

### User Story 2 - Exit with Explicit Exit Code (Priority: P1)

As a script author, I want to specify an explicit exit code when calling `exit N` so that my scripts can communicate success or failure status to calling programs.

**Why this priority**: Essential for scripting - scripts need to return specific exit codes to indicate different error conditions or success.

**Independent Test**: Run `exit 42` and verify parent process receives exit code 42.

**Acceptance Scenarios**:

1. **Given** a shell session, **When** user types `exit 0`, **Then** shell terminates with exit code 0
2. **Given** a shell session, **When** user types `exit 1`, **Then** shell terminates with exit code 1
3. **Given** a shell session, **When** user types `exit 42`, **Then** shell terminates with exit code 42
4. **Given** a shell session, **When** user types `exit 255`, **Then** shell terminates with exit code 255

---

### User Story 3 - Exit Code Validation (Priority: P2)

As a shell user, I want the shell to validate exit codes and handle edge cases gracefully, masking values to the 0-255 range as POSIX specifies.

**Why this priority**: Important for script compatibility and error prevention, but not core functionality.

**Independent Test**: Run `exit 256` and verify it wraps to 0; run `exit -1` and verify it wraps to 255.

**Acceptance Scenarios**:

1. **Given** a shell session, **When** user types `exit 256`, **Then** shell terminates with exit code 0 (256 mod 256)
2. **Given** a shell session, **When** user types `exit 257`, **Then** shell terminates with exit code 1 (257 mod 256)
3. **Given** a shell session, **When** user types `exit -1`, **Then** shell terminates with exit code 255 (two's complement wrap)

---

### User Story 4 - Error Handling for Invalid Arguments (Priority: P2)

As a shell user, I expect clear error messages when I provide invalid arguments to the exit command.

**Why this priority**: Usability improvement - helps users understand what went wrong.

**Independent Test**: Run `exit abc` and verify an error message is displayed and the shell does not exit.

**Acceptance Scenarios**:

1. **Given** a shell session, **When** user types `exit abc`, **Then** shell displays error "rush: exit: abc: numeric argument required" and does not exit
2. **Given** a shell session, **When** user types `exit 1 2 3`, **Then** shell displays error "rush: exit: too many arguments" and does not exit

---

### Edge Cases

- What happens when exit is called in a sourced script? The entire shell exits, not just the script.
- What happens with very large numbers (beyond i64 range)? Treated as parse error with numeric argument required message.
- What happens with floating point numbers like `exit 1.5`? Treated as non-numeric, shows error message.
- What happens when exit code is negative? Wrapped using POSIX rules (value AND 0xFF).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Shell MUST support `exit` command with no arguments to terminate with last command's exit code
- **FR-002**: Shell MUST support `exit N` to terminate with explicit exit code N
- **FR-003**: Shell MUST mask exit codes to 0-255 range using POSIX rules (value AND 0xFF)
- **FR-004**: Shell MUST display error message for non-numeric arguments
- **FR-005**: Shell MUST display error message when more than one argument is provided
- **FR-006**: Shell MUST NOT exit when given invalid arguments (error + stay running)
- **FR-007**: Shell MUST propagate exit code to parent process correctly
- **FR-008**: Exit command in sourced files MUST exit the entire shell, not just the sourced script

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can exit the shell with any exit code between 0-255
- **SC-002**: Scripts using exit codes behave identically to bash/zsh for the same inputs
- **SC-003**: Error messages clearly indicate the nature of argument errors
- **SC-004**: Exit command completes instantaneously (no perceptible delay)

## Assumptions

- The shell maintains a "last exit code" variable that is updated after each command execution
- The shell's main loop can be signaled to terminate with a specific exit code
- POSIX compatibility is the primary goal for exit code behavior
