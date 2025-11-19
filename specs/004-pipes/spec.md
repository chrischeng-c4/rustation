# Feature Specification: Pipe Operator Support

**Feature Branch**: `004-pipes`
**Created**: 2025-11-19
**Status**: Draft
**Input**: User description: "Implement pipe (|) support for command composition"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Two-Command Pipeline (Priority: P1) 🎯 MVP

Users can connect two commands using the pipe operator, where the output of the first command becomes the input to the second command.

**Why this priority**: This is the fundamental building block of command composition and represents the minimal viable feature. Without this, pipes don't work at all. This alone enables the most common use cases like `ls | grep pattern` and `cat file | wc -l`.

**Independent Test**: Can be fully tested by running `echo "hello world" | grep hello` and verifying that "hello world" appears in output. Delivers immediate value for basic filtering and text processing workflows.

**Acceptance Scenarios**:

1. **Given** user types `ls | grep txt`, **When** command executes, **Then** only lines containing "txt" from ls output are displayed
2. **Given** user types `echo "test" | wc -l`, **When** command executes, **Then** output shows "1" (one line counted)
3. **Given** user types `cat README.md | head -5`, **When** command executes, **Then** first 5 lines of README.md are displayed
4. **Given** first command produces no output, **When** pipeline executes, **Then** second command receives empty input and completes successfully
5. **Given** user types `date | cat`, **When** command executes, **Then** current date/time is displayed (verifying stdout piping works)

---

### User Story 2 - Multi-Command Pipeline Chain (Priority: P2)

Users can chain three or more commands together, where each command's output feeds into the next command's input sequentially.

**Why this priority**: Extends US1 to handle real-world complexity. Many common workflows require 3+ command chains (e.g., `cat file | grep pattern | sort | uniq`). This is a natural extension of US1 and shares most of the same infrastructure.

**Independent Test**: Can be fully tested by running `echo -e "apple\nbanana\napple" | sort | uniq` and verifying deduplicated sorted output. Demonstrates value for data transformation pipelines.

**Acceptance Scenarios**:

1. **Given** user types `cat file.txt | grep error | wc -l`, **When** command executes, **Then** count of lines containing "error" is displayed
2. **Given** user types `ls -la | grep "\.md" | head -3`, **When** command executes, **Then** first 3 markdown files are listed
3. **Given** user types `echo -e "b\na\nc" | sort | tail -1`, **When** command executes, **Then** output shows "c" (last sorted item)
4. **Given** pipeline has 5 commands, **When** executed, **Then** all five commands execute in sequence with proper I/O chaining

---

### User Story 3 - Pipeline Error Handling (Priority: P3)

Users receive clear feedback when commands in a pipeline fail, and the pipeline behavior matches standard shell semantics for error propagation.

**Why this priority**: Important for usability and debugging, but not blocking for basic functionality. Users can work with pipes before this is implemented, though error diagnosis will be less helpful.

**Independent Test**: Can be fully tested by running `ls /nonexistent | grep foo` and verifying appropriate error message appears. Improves troubleshooting experience for failed pipelines.

**Acceptance Scenarios**:

1. **Given** user types `ls /nonexistent | grep foo`, **When** first command fails, **Then** error message is displayed and pipeline terminates
2. **Given** user types `echo "test" | nonexistentcmd`, **When** second command fails, **Then** error indicates which command failed
3. **Given** middle command in 3-command pipeline fails, **When** executed, **Then** appropriate error is shown and remaining commands don't execute
4. **Given** user types `cat file.txt | grep pattern`, **When** grep finds no matches, **Then** exit code reflects grep's standard behavior (exit code 1, no error message)

---

### User Story 4 - Pipeline Exit Code Handling (Priority: P4)

The shell's exit code reflects the pipeline's success or failure according to standard Unix pipeline semantics (last command's exit code).

**Why this priority**: Critical for scripting and automation, but less important for interactive use. Can be implemented after basic pipeline functionality is working. Affects advanced users more than casual users.

**Independent Test**: Can be fully tested by running `echo test | false; echo $?` and verifying exit code is 1 (from false command). Important for script reliability but not blocking for interactive use.

**Acceptance Scenarios**:

1. **Given** user types `true | false`, **When** pipeline completes, **Then** exit code is 1 (last command failed)
2. **Given** user types `false | true`, **When** pipeline completes, **Then** exit code is 0 (last command succeeded)
3. **Given** user types `echo test | grep test`, **When** grep succeeds, **Then** exit code is 0
4. **Given** user types `echo test | grep nomatch`, **When** grep finds no match, **Then** exit code is 1

---

### Edge Cases

- What happens when a pipeline command produces extremely large output (>1GB)?
  - System should handle gracefully without memory exhaustion
  - Backpressure mechanisms prevent overwhelming pipes

- How does system handle binary data in pipes?
  - Pipes should pass binary data unmodified (important for `tar | gzip` workflows)

- What happens when user presses Ctrl+C during pipeline execution?
  - All commands in pipeline should receive SIGINT and terminate cleanly

- How are empty commands handled in pipeline (e.g., `ls | | grep`)?
  - Parser should reject syntax error before execution

- What happens with pipes combined with quotes (e.g., `echo "hello | world"`)?
  - Pipes inside quotes should be treated as literal text, not operators

- How does system handle very long pipelines (20+ commands)?
  - Should work correctly up to reasonable limits (suggest 100 commands max)
  - Performance degrades gracefully beyond optimal range

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse the pipe operator (`|`) as a special token that separates commands in a command line
- **FR-002**: System MUST execute piped commands sequentially from left to right
- **FR-003**: System MUST connect each command's standard output (stdout) to the next command's standard input (stdin)
- **FR-004**: System MUST allow pipelines of 2 or more commands (no arbitrary limit below 100 commands)
- **FR-005**: System MUST preserve binary data when piping between commands (no text-only assumption)
- **FR-006**: System MUST execute all commands in a pipeline concurrently (not sequentially waiting for completion)
- **FR-007**: System MUST handle commands that produce large output without blocking or memory exhaustion
- **FR-008**: System MUST distinguish between pipe operators and pipe characters within quoted strings (e.g., `echo "a | b"` should NOT create a pipeline)
- **FR-009**: System MUST propagate signals (SIGINT, SIGTERM) to all commands in a pipeline
- **FR-010**: System MUST wait for all pipeline commands to complete before returning control to user
- **FR-011**: System MUST set the shell's exit code to the exit code of the last command in the pipeline
- **FR-012**: System MUST display error messages from any failing command in the pipeline
- **FR-013**: System MUST handle empty output from commands (zero bytes) without error
- **FR-014**: System MUST reject malformed pipelines (e.g., `| grep foo`, `ls |`, `ls | | grep`) with clear syntax error messages
- **FR-015**: System MUST preserve the existing behavior for commands without pipes (backward compatibility)

### Key Entities

- **Pipeline**: A sequence of two or more commands connected by pipe operators
  - Contains: List of command segments
  - Represents: The complete command line entered by user
  - Relationships: Composed of multiple Command Segments

- **Command Segment**: An individual command within a pipeline
  - Contains: Command name, arguments, position in pipeline
  - Represents: One executable unit in the chain
  - Relationships: Part of a Pipeline, connects to adjacent segments via stdin/stdout

- **Pipe Connection**: The I/O relationship between two adjacent commands
  - Contains: Source command (stdout), destination command (stdin), buffer state
  - Represents: The data flow channel between commands
  - Relationships: Links two Command Segments

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully execute `ls | grep pattern` and receive filtered results
- **SC-002**: System handles pipelines of up to 10 commands without errors or performance degradation
- **SC-003**: Pipeline execution completes within 5% overhead compared to running commands individually (performance requirement from constitution)
- **SC-004**: Users can pipe binary data (e.g., `tar czf - dir | ssh user@host "tar xzf -"`) without corruption
- **SC-005**: 100% of standard Unix pipe behaviors work correctly (exit codes, signal propagation, I/O chaining)
- **SC-006**: Pipeline parsing completes in <1ms for typical command lines (<500 characters)
- **SC-007**: Users can successfully compose commands to solve filtering, sorting, and counting tasks without leaving the shell
