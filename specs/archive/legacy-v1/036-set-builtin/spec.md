# Feature Specification: Set Builtin

**Feature Branch**: `036-set-builtin`
**Created**: 2025-12-09
**Status**: Draft
**Input**: User description: "set builtin for shell options (-e, -x, -o pipefail)"

## User Scenarios & Testing

### User Story 1 - Exit on Error (-e) (Priority: P1)

As a shell script author, I want the shell to exit immediately when any command fails so that I can catch errors early and prevent cascading failures in my scripts.

**Why this priority**: This is the most critical safety feature for shell scripting. The `-e` option (errexit) prevents scripts from continuing execution after errors, which is essential for production scripts and CI/CD pipelines. It's the foundation of defensive shell scripting.

**Independent Test**: Can be fully tested by running `set -e` followed by a failing command (e.g., `false`) and verifying the shell exits with the command's error code. Delivers immediate value by making scripts safer.

**Acceptance Scenarios**:

1. **Given** user runs `set -e`, **When** user executes a command that fails with non-zero exit code, **Then** the shell exits immediately with that exit code
2. **Given** user runs `set -e`, **When** user executes a successful command (exit code 0), **Then** the shell continues normal execution
3. **Given** user runs `set -e` in a script, **When** any command in the script fails, **Then** the script terminates and returns the failure exit code
4. **Given** user runs `set +e`, **When** user executes a failing command, **Then** the shell continues execution (errexit disabled)
5. **Given** errexit is enabled, **When** a failing command is part of a conditional (if/while/until condition), **Then** the shell does NOT exit (conditional context exception)
6. **Given** errexit is enabled, **When** a failing command is preceded by `!`, **Then** the shell does NOT exit (negation exception)

---

### User Story 2 - Command Tracing (-x) (Priority: P2)

As a shell script debugger, I want to see each command before it executes with all expansions resolved so that I can trace script execution and diagnose issues.

**Why this priority**: Command tracing is the primary debugging tool for shell scripts. The `-x` option (xtrace) prints each command with variables expanded before execution, making it invaluable for troubleshooting. Common in development and debugging workflows.

**Independent Test**: Can be tested by running `set -x` followed by any command (e.g., `echo $HOME`) and verifying the expanded command is printed to stderr with a `+` prefix before execution.

**Acceptance Scenarios**:

1. **Given** user runs `set -x`, **When** user executes any command, **Then** the command is printed to stderr with expanded variables before execution
2. **Given** xtrace is enabled, **When** user executes `echo $HOME`, **Then** stderr shows `+ echo /home/user` (with actual HOME value) and stdout shows the result
3. **Given** user runs `set +x`, **When** user executes commands, **Then** commands are NOT traced (xtrace disabled)
4. **Given** xtrace is enabled, **When** multi-line commands or loops execute, **Then** each command in the sequence is traced separately
5. **Given** xtrace is enabled, **When** user executes a pipeline, **Then** all commands in the pipeline are traced

---

### User Story 3 - Pipeline Failure (-o pipefail) (Priority: P2)

As a shell script author, I want pipelines to fail if any command in the pipeline fails so that I can detect errors in the middle of pipelines, not just the last command.

**Why this priority**: By default, pipelines only return the exit code of the last command, which can hide errors in earlier commands. The `pipefail` option makes pipelines fail if ANY command fails, which is critical for reliable scripts. Commonly used with `-e` for comprehensive error handling.

**Independent Test**: Can be tested by running `set -o pipefail` followed by a pipeline with a failing middle command (e.g., `false | true`) and verifying the pipeline fails with the error code from `false`.

**Acceptance Scenarios**:

1. **Given** user runs `set -o pipefail`, **When** user executes a pipeline where any command fails, **Then** the pipeline returns the exit code of the failed command (not just the last command)
2. **Given** pipefail is enabled, **When** user runs `false | true`, **Then** the pipeline exit code is 1 (from false), not 0 (from true)
3. **Given** pipefail is disabled (default), **When** user runs `false | true`, **Then** the pipeline exit code is 0 (only last command matters)
4. **Given** user runs `set +o pipefail`, **When** user executes pipelines, **Then** only the last command's exit code matters (pipefail disabled)
5. **Given** pipefail is enabled and errexit is enabled, **When** a pipeline command fails, **Then** the shell exits immediately

---

### User Story 4 - Query Option Status (Priority: P3)

As a shell user, I want to check which options are currently enabled so that I can verify my shell configuration and understand the current execution mode.

**Why this priority**: Useful for debugging and introspection, but less critical than the options themselves. Allows users to query current state and verify settings.

**Independent Test**: Can be tested by running `set -o` without arguments and verifying it prints all available options with their current on/off status.

**Acceptance Scenarios**:

1. **Given** user runs `set -o` without arguments, **When** command executes, **Then** shell prints all available options with their current status (on/off)
2. **Given** user has enabled some options, **When** user runs `set -o`, **Then** enabled options show as "on" and disabled options show as "off"
3. **Given** user runs `set +o` without arguments, **When** command executes, **Then** shell prints set commands to recreate the current option state

---

### Edge Cases

- What happens when user runs `set -e` and a command fails in a subshell? (Subshell should exit, parent shell should continue unless subshell exit code is checked)
- What happens when user combines multiple options like `set -ex`? (Both options should be enabled simultaneously)
- What happens when user runs `set -o` with an invalid option name? (Should print error message and return non-zero exit code)
- What happens when errexit is enabled and a command in a `&&` or `||` chain fails? (Should NOT exit - logical operators are conditional contexts)
- What happens when xtrace is enabled and commands contain special characters or quotes? (Should print escaped/quoted version that's valid shell syntax)
- What happens when user runs `set` with no arguments? (Traditionally prints all shell variables - out of scope for this feature)
- What happens with option precedence when both short (-e) and long (-o errexit) forms are used? (They're equivalent - enabling one enables the other)
- What happens when xtrace prints to stderr and stderr is redirected? (Trace output follows stderr redirection)
- What happens when pipefail is enabled but all commands in pipeline succeed? (Pipeline succeeds normally with exit code 0)
- What happens when user runs `set -` with no option characters? (Should be no-op or error - bash treats as no-op)

## Requirements

### Functional Requirements

- **FR-001**: System MUST implement `set -e` to enable errexit mode (exit immediately when any command fails with non-zero exit code)
- **FR-002**: System MUST implement `set +e` to disable errexit mode (default behavior - continue on errors)
- **FR-003**: System MUST implement `set -x` to enable xtrace mode (print each command with expanded arguments to stderr before execution)
- **FR-004**: System MUST implement `set +x` to disable xtrace mode (default behavior - no command tracing)
- **FR-005**: System MUST implement `set -o pipefail` to enable pipefail mode (pipeline fails if any command fails, not just the last)
- **FR-006**: System MUST implement `set +o pipefail` to disable pipefail mode (default behavior - pipeline exit code is from last command only)
- **FR-007**: System MUST implement `set -o errexit` as equivalent to `set -e` (long-form option name)
- **FR-008**: System MUST implement `set -o xtrace` as equivalent to `set -x` (long-form option name)
- **FR-009**: System MUST implement `set -o` without arguments to print all available options with their current on/off status
- **FR-010**: System MUST implement `set +o` without arguments to print set commands to recreate current option state
- **FR-011**: System MUST support combining multiple short-form options in a single command (e.g., `set -ex` enables both errexit and xtrace)
- **FR-012**: System MUST NOT exit in errexit mode when failing command is part of a conditional expression (if/while/until condition, `&&`, `||`)
- **FR-013**: System MUST NOT exit in errexit mode when failing command is negated with `!`
- **FR-014**: System MUST print xtrace output to stderr with a `+` prefix before each traced command
- **FR-015**: System MUST show fully expanded commands in xtrace output (variables, command substitution, etc. all resolved)
- **FR-016**: System MUST persist option settings across commands within the same shell session until explicitly changed
- **FR-017**: System MUST return non-zero exit code when `set` is called with invalid option names
- **FR-018**: System MUST support both enabling and disabling options in a single command (e.g., `set -e +x` enables errexit and disables xtrace)
- **FR-019**: System MUST support common option aliases (e.g., `-o errexit` = `-e`, `-o xtrace` = `-x`, `-o pipefail` = pipefail only)
- **FR-020**: System MUST respect errexit setting in scripts and interactive mode

### Key Entities

- **Shell Option**: A boolean flag that changes shell behavior. Has a name (e.g., "errexit"), short form (e.g., "-e"), long form (e.g., "-o errexit"), and current state (enabled/disabled)
- **Option Set**: The current collection of all shell options and their states for the active shell session

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can enable errexit mode and verify scripts terminate on first error within 5 seconds of testing
- **SC-002**: Users can enable xtrace mode and see command expansions immediately for debugging purposes
- **SC-003**: Users can enable pipefail mode and detect failures in pipeline middle commands that previously went unnoticed
- **SC-004**: 100% of errexit tests with conditional contexts (if/while/&&/||/!) correctly avoid exiting (conditional exception handling works)
- **SC-005**: 100% of xtrace output correctly shows expanded variables and command substitutions
- **SC-006**: Users can query current option state with `set -o` and get accurate status for all implemented options
- **SC-007**: Feature maintains compatibility with bash/zsh behavior for errexit, xtrace, and pipefail (90%+ behavioral parity)

## Assumptions

1. The rush shell already supports exit codes and can detect command failures (dependency on basic command execution)
2. The rush shell already has stderr output stream available for xtrace messages
3. The rush shell already supports pipelines and can access individual command exit codes in pipelines (dependency on feature 004)
4. Option state is maintained in the shell's execution context and persists across commands
5. The shell runs on POSIX-compatible systems (Linux, macOS, BSD)
6. Default state for all options is disabled (errexit=off, xtrace=off, pipefail=off) matching bash defaults

## Dependencies

- Feature 004 (pipes): Required for pipefail option to work with pipeline exit codes
- Feature 006 (job-control): May interact with background jobs and option inheritance
- Feature 017 (conditional-control-flow): Required for conditional context exception handling in errexit mode

## Out of Scope

- Other shell options beyond errexit, xtrace, and pipefail (e.g., nounset, noglob, noclobber) - can be added incrementally later
- The `set` command without options (printing all shell variables) - different feature, use `export` or `env`
- Option inheritance by subshells (subshells start with parent's options) - may be added later
- Shell option configuration files or startup scripts (e.g., .rushrc) - separate feature
- Option-specific detailed trace formatting (e.g., PS4 prompt for xtrace) - can be added later
- The `shopt` command (bash-specific alternative to `set -o`) - not POSIX, lower priority
