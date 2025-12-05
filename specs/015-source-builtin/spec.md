# Feature Specification: Source Builtin Command

**Feature Branch**: `015-source-builtin`
**Created**: 2024-12-04
**Status**: Draft
**Input**: User description: "Implement source builtin command to load and execute script files"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Execute Script File with `source` Command (Priority: P1)

As a shell user, I want to execute commands from a script file in the current shell context so that I can load configurations, define aliases, and set environment variables that persist in my session.

**Why this priority**: This is the core functionality of the source builtin. Without the ability to execute script files, the command has no purpose. This enables configuration loading which is fundamental for shell customization.

**Independent Test**: Can be fully tested by creating a script file with variable assignments and sourcing it, then verifying the variables are set in the current shell.

**Acceptance Scenarios**:

1. **Given** a script file `config.sh` containing `export MY_VAR=hello`, **When** user runs `source config.sh`, **Then** the variable `MY_VAR` is set to "hello" in the current shell and `echo $MY_VAR` outputs "hello"
2. **Given** a script file with multiple commands, **When** user runs `source script.sh`, **Then** all commands execute sequentially in the current shell context
3. **Given** a script file that sets an alias `alias ll='ls -la'`, **When** user runs `source script.sh`, **Then** the alias `ll` is available in the current shell

---

### User Story 2 - Execute Script File with `.` (Dot) Command (Priority: P1)

As a shell user familiar with POSIX shells, I want to use the `.` (dot) command as an alias for `source` so that I can use the traditional POSIX syntax.

**Why this priority**: The dot command is the POSIX-standard way to source files and many existing scripts and tutorials use this syntax. Equal priority with `source` as both syntaxes should work identically.

**Independent Test**: Can be fully tested by sourcing a file with `. script.sh` and verifying the same behavior as `source script.sh`.

**Acceptance Scenarios**:

1. **Given** a script file `config.sh`, **When** user runs `. config.sh`, **Then** the script executes identically to `source config.sh`
2. **Given** a script that exports variables, **When** user runs `. script.sh`, **Then** exported variables are available in the current shell

---

### User Story 3 - Handle Script Arguments (Priority: P2)

As a shell user, I want to pass arguments to sourced scripts so that I can parameterize my configuration scripts.

**Why this priority**: Argument passing enables reusable scripts and is commonly used in advanced shell workflows. Lower priority than basic sourcing because it's an enhancement to core functionality.

**Independent Test**: Can be tested by creating a script that uses `$1`, `$2`, etc., sourcing it with arguments, and verifying the arguments are accessible within the script.

**Acceptance Scenarios**:

1. **Given** a script `greet.sh` containing `echo "Hello, $1"`, **When** user runs `source greet.sh World`, **Then** the output is "Hello, World"
2. **Given** a script using `$@` to process all arguments, **When** user runs `source script.sh a b c`, **Then** the script receives all three arguments

---

### User Story 4 - Search PATH for Scripts (Priority: P3)

As a shell user, I want `source` to search the PATH when a script is not found in the current directory so that I can source scripts installed in system locations.

**Why this priority**: PATH searching is a convenience feature that matches bash behavior. Lower priority because users can always specify absolute or relative paths.

**Independent Test**: Can be tested by placing a script in a PATH directory, then sourcing it by name only without path.

**Acceptance Scenarios**:

1. **Given** a script `my_script.sh` in `/usr/local/bin` (which is in PATH), **When** user runs `source my_script.sh`, **Then** the script is found and executed
2. **Given** a script exists both in current directory and in PATH, **When** user runs `source script.sh`, **Then** the script in the current directory takes precedence

---

### Edge Cases

- What happens when the script file does not exist? → Shell displays an error message and returns a non-zero exit status
- What happens when the script file is not readable (permission denied)? → Shell displays a permission error and returns non-zero exit status
- What happens when source is called with no arguments? → Shell displays usage error and returns non-zero exit status
- What happens when the script contains a syntax error? → Shell reports the syntax error with file name and line number, returns non-zero exit status
- What happens when a sourced script sources another script (nested sourcing)? → Nested sourcing works correctly, executing the inner script in the same shell context
- What happens when a sourced script calls `exit`? → The current shell exits (since the script runs in the current shell context)
- What happens when source is used in a pipeline? → Source executes normally; its stdout can be piped to other commands

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Shell MUST provide a `source` builtin command that executes commands from a file in the current shell context
- **FR-002**: Shell MUST provide a `.` (dot) builtin command that behaves identically to `source`
- **FR-003**: Shell MUST execute all commands in the sourced file sequentially
- **FR-004**: Shell MUST preserve variable assignments, alias definitions, and function definitions made in the sourced file
- **FR-005**: Shell MUST propagate the exit status of the last command in the sourced file as the exit status of the source command
- **FR-006**: Shell MUST display an error if the specified file does not exist or is not readable
- **FR-007**: Shell MUST support relative paths, absolute paths, and paths with tilde expansion for the script file
- **FR-008**: Shell MUST pass additional arguments to the sourced script as positional parameters ($1, $2, etc.)
- **FR-009**: Shell MUST restore the original positional parameters after the sourced script completes
- **FR-010**: Shell MUST search PATH for the script file if not found with the given path (matching bash behavior)
- **FR-011**: Shell MUST support nested sourcing (a sourced script can source other scripts)
- **FR-012**: Shell MUST report syntax errors in sourced files with the file name and line number

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can load configuration files and have all defined variables, aliases, and settings persist in their current shell session
- **SC-002**: Both `source` and `.` commands work identically, enabling POSIX compatibility
- **SC-003**: Error messages clearly indicate the cause of failure (file not found, permission denied, syntax error) with file location information
- **SC-004**: Sourcing a script with arguments makes those arguments accessible as `$1`, `$2`, etc. within the script
- **SC-005**: Nested sourcing of up to 100 levels deep works correctly without causing shell instability

## Assumptions

- Positional parameters (`$1`, `$2`, etc.) will be implemented as part of feature 017-positional-parameters. Until then, argument passing (FR-008, FR-009, US3) will have limited functionality.
- The shell already supports variable expansion, alias definitions, and command execution in the current context.
- The shell's existing file reading infrastructure can be reused for reading script files.
