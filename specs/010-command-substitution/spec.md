# Feature Specification: Command Substitution ($())

**Feature Branch**: `010-command-substitution`
**Created**: 2025-11-30
**Status**: Draft

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Command Substitution (Priority: P1)

Users want to use the output of one command as arguments to another. For example, `echo $(date)` executes `date`, captures output, and passes it to `echo`.

**Why this priority**: Core functionality - without this, command substitution cannot work.

**Independent Test**: Execute `echo $(date)` and verify output contains today's date.

**Acceptance Scenarios**:

1. **Given** a command with `$()` syntax, **When** executed, **Then** inner command executes and output replaces the `$()` expression
2. **Given** `$(echo hello)`, **When** used in a command, **Then** it expands to `hello` 
3. **Given** command output with multiple words, **When** used as arguments, **Then** words split into separate arguments

---

### User Story 2 - Nested Command Substitution (Priority: P2)

Users want to nest command substitutions multiple levels deep: `echo $(echo $(date))` should work correctly.

**Why this priority**: Advanced feature for complex scripts. Less critical than basic substitution.

**Independent Test**: Execute nested `$()` expressions and verify correct execution order and results.

**Acceptance Scenarios**:

1. **Given** nested `$()` like `echo $(echo $(pwd))`, **When** executed, **Then** inner commands execute first, cascading outward
2. **Given** multiple nesting levels, **When** executed, **Then** output correctly substituted

---

### User Story 3 - Command Substitution in Variables (Priority: P2)

Users want to assign command output to variables: `current_date=$(date)` should assign date output to the variable.

**Why this priority**: Very useful for practical scripts. Important but depends on variables feature.

**Independent Test**: Assign command output to variable and verify value when expanded.

**Acceptance Scenarios**:

1. **Given** `var=$(command)`, **When** executed, **Then** variable holds command's output
2. **Given** variable used later, **When** expanded, **Then** it contains captured output

---

### User Story 4 - Multiple Arguments from Substitution (Priority: P2)

Users want command substitution to provide multiple arguments: `ls $(find . -name "*.txt")` finds files and passes them to `ls`.

**Why this priority**: Enables powerful argument generation. Important but less critical than basic substitution.

**Independent Test**: Use substitution to provide multiple arguments and verify they're correctly parsed.

**Acceptance Scenarios**:

1. **Given** `command $(other_command)` with multi-line output, **When** executed, **Then** each line becomes separate argument
2. **Given** word splitting, **When** applied, **Then** whitespace respected as argument separator

---

### User Story 5 - Error Handling (Priority: P1)

Users want proper error handling when commands in substitution fail. Failed commands should report errors and not silently pass invalid data.

**Why this priority**: Critical for reliability and debugging. Prevents silent failures.

**Independent Test**: Execute failing command in substitution and verify error reporting.

**Acceptance Scenarios**:

1. **Given** failing command in `$()`, **When** executed, **Then** error reported
2. **Given** command failure, **When** error occurs, **Then** outer command not executed with invalid data
3. **Given** substituted commands, **When** complete, **Then** exit code reflects result

---

### Edge Cases

- Empty output from substituted command?
- Special characters in command output handling?
- Substitution in quoted strings?
- Word splitting with output?
- Long-running/timeout commands in substitution?
- Substitution in redirection targets?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support `$()` syntax for command substitution
- **FR-002**: System MUST execute command inside `$()` and capture stdout
- **FR-003**: System MUST substitute captured output in place of `$()` expression
- **FR-004**: System MUST support nested command substitutions
- **FR-005**: System MUST support substitution in command arguments
- **FR-006**: System MUST support substitution in variable assignments
- **FR-007**: System MUST handle multi-line output correctly
- **FR-008**: System MUST perform proper word splitting on output
- **FR-009**: System MUST report errors when substituted commands fail
- **FR-010**: System MUST preserve exit codes through substitution

### Key Entities

- **Substitution Expression**: The `$()` syntax containing a command
- **Captured Output**: The stdout from command inside `$()`
- **Substituted Result**: The expanded text replacing `$()`

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can use `$()` syntax with 100% bash compatibility
- **SC-002**: Nested substitutions work with proper execution order
- **SC-003**: Substitution works in all contexts: arguments, variables, redirections
- **SC-004**: Output word-splitting follows POSIX semantics
- **SC-005**: Error handling prevents invalid outer command execution
- **SC-006**: All acceptance scenarios pass

## Assumptions

- `$()` syntax used instead of backticks for clarity and nesting
- Captures stdout only (stderr goes to terminal unless redirected)
- Nested execution is innermost-first
- Word splitting applies following POSIX rules
- Empty output handled gracefully
- No custom timeout per substitution

## Dependencies

- Builds on command execution infrastructure
- Depends on shell parsing for `$()` recognition
- Requires error handling for command failures
- Integrates with variable system
- Works with output redirection (005)

## Notes

- `$()` preferred over backticks in modern shells
- Fundamental feature enabling powerful scripting
- Must handle large output efficiently
- Respect quoting and escape sequences
