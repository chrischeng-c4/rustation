# Feature Specification: Additional Built-in Commands

**Feature Branch**: `012-additional-builtins`
**Created**: 2025-11-30
**Status**: Draft

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Echo Builtin (Priority: P1)

Users want to use `echo` to print text to the terminal. This is one of the most basic and frequently used shell commands, used in scripts and interactive sessions.

**Why this priority**: Fundamental command used constantly. Required for basic shell usability.

**Independent Test**: Execute `echo hello` and verify "hello" is printed.

**Acceptance Scenarios**:

1. **Given** `echo hello`, **When** executed, **Then** prints `hello` to stdout
2. **Given** `echo -n hello`, **When** executed, **Then** prints `hello` without newline
3. **Given** `echo "hello world"`, **When** executed, **Then** prints entire string

---

### User Story 2 - Printf Builtin (Priority: P2)

Users want formatted output with `printf` for scripts that need precise formatting. For example, `printf "%s: %d\n" name 42` produces formatted output.

**Why this priority**: Important for sophisticated scripts. Less critical than echo but valuable for formatting.

**Independent Test**: Execute printf with format string and verify output format.

**Acceptance Scenarios**:

1. **Given** `printf "%s\n" hello`, **When** executed, **Then** prints `hello` with newline
2. **Given** `printf "%d" 42`, **When** executed, **Then** prints `42` as integer
3. **Given** format string with multiple values, **When** executed, **Then** values formatted correctly

---

### User Story 3 - Test Builtin/Conditional Expressions (Priority: P1)

Users want to test conditions with `test` (or `[...]` syntax) for use in if statements. For example, `test -f file.txt` checks if a file exists, or `[ "$var" = "value" ]` checks variable equality.

**Why this priority**: Essential for conditionals and control flow. Fundamental to scripting.

**Independent Test**: Execute test command and verify correct exit codes (0 for true, non-zero for false).

**Acceptance Scenarios**:

1. **Given** `test -f existing_file`, **When** executed, **Then** returns exit code 0 (true)
2. **Given** `test -f non_existent_file`, **When** executed, **Then** returns non-zero (false)
3. **Given** `[ "$var" = "value" ]` with matching value, **When** executed, **Then** returns 0
4. **Given** `test -d directory`, **When** executed, **Then** returns 0 if directory exists

---

### User Story 4 - True and False Builtins (Priority: P2)

Users want simple commands that always succeed (`true`) or always fail (`false`) for use in conditionals and loops. These are used in scripting for control flow.

**Why this priority**: Less frequently used but important for complete shell functionality.

**Independent Test**: Execute `true` and `false` and verify exit codes (0 and 1 respectively).

**Acceptance Scenarios**:

1. **Given** `true`, **When** executed, **Then** exits with code 0
2. **Given** `false`, **When** executed, **Then** exits with code 1
3. **Given** used in if condition, **When** evaluated, **Then** true always succeeds, false always fails

---

### User Story 5 - Additional Common Builtins (Priority: P2)

Users want common utilities like `pwd` (print working directory), `type` (show command type), `which` (find command), and others for practical shell use.

**Why this priority**: Useful utilities but less critical than core commands. Can be added incrementally.

**Independent Test**: Execute each builtin and verify appropriate output.

**Acceptance Scenarios**:

1. **Given** `pwd`, **When** executed, **Then** prints current working directory
2. **Given** `type echo`, **When** executed, **Then** shows that echo is a builtin
3. **Given** `which ls`, **When** executed, **Then** shows path to ls command

---

### Edge Cases

- Echo with special characters and escape sequences?
- Printf with complex format strings?
- Test with complex boolean expressions?
- Multiple arguments to builtins?
- Error messages for invalid usage?
- Option parsing and flag handling?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST implement `echo` builtin with `-n` option support
- **FR-002**: System MUST implement `printf` builtin with format string support
- **FR-003**: System MUST implement `test` builtin with file/string/numeric tests
- **FR-004**: System MUST support `[` as alternative syntax to `test`
- **FR-005**: System MUST implement `true` builtin (always succeeds)
- **FR-006**: System MUST implement `false` builtin (always fails)
- **FR-007**: System MUST implement `pwd` builtin to print working directory
- **FR-008**: System MUST implement `type` builtin to show command information
- **FR-009**: System MUST implement proper exit codes for all builtins
- **FR-010**: System MUST handle errors and invalid arguments gracefully

### Key Entities

- **Builtin Command**: Shell command implemented directly in the shell (not external)
- **Exit Code**: Return value indicating success (0) or failure (non-zero)
- **Format String**: printf-style format specification with placeholders

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Echo works with POSIX-compatible options and output formatting
- **SC-002**: Printf supports common format specifiers correctly
- **SC-003**: Test supports file, string, and numeric comparisons with bash compatibility
- **SC-004**: All builtins return appropriate exit codes (0 for success, non-zero for failure)
- **SC-005**: Error handling provides clear messages for invalid usage
- **SC-006**: All acceptance scenarios pass

## Assumptions

- Builtins follow POSIX shell conventions for maximum compatibility
- Exit codes follow standard shell convention (0 = success, non-zero = failure)
- Echo and printf handle escape sequences like `\n`, `\t`, etc.
- Test supports common operations: `-f` (file exists), `-d` (directory), `-z` (zero length), etc.
- Commands are implemented as builtins (not external programs) for efficiency

## Dependencies

- Builds on core shell infrastructure
- Depends on option parsing for handling flags
- Integrates with exit code system
- Works with conditionals and control flow

## Notes

- These are fundamental builtins present in all POSIX shells
- Implementation should prioritize echo and test first (most commonly used)
- Printf can follow after basic builtins are stable
- Additional builtins can be added incrementally as separate tasks
- Consider performance of frequently-used builtins like echo
