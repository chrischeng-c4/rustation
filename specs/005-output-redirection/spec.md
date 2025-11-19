# Feature Specification: Output Redirection Operators

**Feature Branch**: `005-output-redirection`
**Created**: 2025-11-20
**Status**: Draft
**Input**: User description: "Output redirection operators for rush shell - support for > (overwrite), >> (append), and < (input) to redirect command stdout/stdin to/from files"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Output Redirection (Priority: P1)

A developer runs a command and wants to save its output to a file instead of displaying it in the terminal. They type `ls -la > files.txt` and the directory listing is written to the file. If they run the command again with the same filename, the file is overwritten with new content. This is the most fundamental redirection operation that every shell user expects.

**Why this priority**: This is the core redirection feature that users need most frequently. Saving command output to files is essential for scripting, logging, data processing, and automation workflows. Without this, rush cannot be used for basic automation tasks.

**Independent Test**: Can be fully tested by running `echo "test" > output.txt`, verifying the file contains "test", running `echo "new" > output.txt` again, and verifying the file now contains "new" (overwritten). Delivers immediate value for capturing command output.

**Acceptance Scenarios**:

1. **Given** user runs `echo "hello" > file.txt`, **When** command completes, **Then** file.txt is created containing "hello" with a newline
2. **Given** file.txt already exists with content, **When** user runs `echo "world" > file.txt`, **Then** file.txt is overwritten and now contains only "world" with a newline
3. **Given** user runs `ls -la > listing.txt`, **When** command completes, **Then** listing.txt contains the directory listing output
4. **Given** user runs `nonexistent_command > output.txt`, **When** command fails, **Then** output.txt is created but empty (or contains stderr if redirected)
5. **Given** user runs `echo "test" > /readonly/file.txt` where directory is read-only, **When** command attempts to create file, **Then** user sees permission denied error

---

### User Story 2 - Append Output Redirection (Priority: P1)

A developer wants to add command output to an existing file without losing the previous content. They type `echo "line 1" > log.txt` to create the file, then `echo "line 2" >> log.txt` to append. The file now contains both lines. This is essential for building log files, accumulating data, and incremental file updates.

**Why this priority**: Appending is equally fundamental as overwriting. Users need both to avoid accidental data loss when building up files over multiple commands or script runs. This is critical for logging and data accumulation workflows.

**Independent Test**: Can be fully tested by running `echo "first" > test.txt`, then `echo "second" >> test.txt`, then verifying the file contains both "first" and "second" on separate lines. Delivers immediate value for log file creation and data accumulation.

**Acceptance Scenarios**:

1. **Given** file.txt exists with content "line1", **When** user runs `echo "line2" >> file.txt`, **Then** file.txt contains "line1\nline2\n"
2. **Given** file.txt does not exist, **When** user runs `echo "data" >> file.txt`, **Then** file.txt is created with "data\n" (same as >)
3. **Given** user runs multiple append commands in sequence, **When** all complete, **Then** file contains all outputs in execution order
4. **Given** log.txt exists, **When** user runs `ls >> log.txt` then `date >> log.txt`, **Then** log.txt contains directory listing followed by date output
5. **Given** user appends to a very large existing file, **When** append completes, **Then** only new data is written (no full file rewrite)

---

### User Story 3 - Input Redirection (Priority: P2)

A developer has a file containing data and wants to feed it as input to a command. They type `wc -l < file.txt` and the word count command reads from the file instead of waiting for terminal input. They can also combine it with output redirection: `sort < unsorted.txt > sorted.txt`. This enables batch processing of file data.

**Why this priority**: Input redirection is less frequently used than output redirection but essential for data processing workflows. Many commands expect stdin input, and feeding files to them is a common pattern. P2 because output redirection alone provides significant value.

**Independent Test**: Can be fully tested by creating a file with `echo -e "line1\nline2" > input.txt`, running `wc -l < input.txt`, and verifying it outputs "2". Delivers value for batch data processing and feeding file content to commands.

**Acceptance Scenarios**:

1. **Given** input.txt contains "test data", **When** user runs `cat < input.txt`, **Then** "test data" is displayed
2. **Given** numbers.txt contains "3\n1\n2", **When** user runs `sort < numbers.txt`, **Then** output displays "1\n2\n3" in sorted order
3. **Given** user runs `grep "pattern" < data.txt`, **When** command executes, **Then** grep searches the file content (same as `grep "pattern" data.txt`)
4. **Given** data.txt does not exist, **When** user runs `cat < data.txt`, **Then** user sees "no such file" error
5. **Given** user runs `command < input.txt` where command doesn't read stdin, **When** command executes, **Then** command runs normally (stdin ignored if not needed)

---

### User Story 4 - Combined Redirections (Priority: P2)

A developer wants to perform complex file operations combining multiple redirection operators. They run `sort < input.txt > output.txt` to read from one file and write sorted results to another. They can also combine with pipes: `cat < data.txt | grep "pattern" > results.txt`. This enables sophisticated data transformation workflows.

**Why this priority**: Real-world usage often requires combining operations. Users expect shell operators to compose naturally. P2 because individual redirections must work first, but combinations are important for practical use.

**Independent Test**: Can be fully tested by creating `echo -e "c\nb\na" > unsorted.txt`, running `sort < unsorted.txt > sorted.txt`, and verifying sorted.txt contains "a\nb\nc". Delivers value for complex file processing workflows.

**Acceptance Scenarios**:

1. **Given** user runs `sort < input.txt > output.txt`, **When** command completes, **Then** output.txt contains sorted contents of input.txt
2. **Given** user runs `command < in.txt >> out.txt`, **When** command completes, **Then** processed input is appended to out.txt
3. **Given** user runs `cat < file1.txt | grep "x" > file2.txt`, **When** command completes, **Then** file2.txt contains grep results
4. **Given** user runs `echo "data" | tee output.txt > final.txt`, **When** command completes, **Then** both files contain "data"
5. **Given** user runs `< input.txt grep "pattern"`, **When** command executes, **Then** works same as `grep "pattern" < input.txt` (order doesn't matter)

---

### User Story 5 - Error Handling and Edge Cases (Priority: P3)

A developer encounters error conditions while using redirections. When they try to redirect to a directory instead of a file (`ls > /etc`), they see a clear error message. When a command fails but has output redirection, the behavior is predictable. When they redirect to a file they're currently reading from (`sort < file.txt > file.txt`), they understand the consequences.

**Why this priority**: Robust error handling prevents data loss and user confusion. While not blocking basic functionality, good error messages and predictable behavior are essential for production use. P3 because core features must work before edge case handling.

**Independent Test**: Can be fully tested by running error scenarios: `echo "x" > /etc` (directory error), `false > out.txt` (command failure), `cat < file.txt > file.txt` (same file warning). Delivers confidence in error scenarios and prevents data loss.

**Acceptance Scenarios**:

1. **Given** user runs `echo "test" > /existing_directory`, **When** redirection attempts to create file, **Then** error message says "is a directory"
2. **Given** user runs `false > output.txt`, **When** command fails, **Then** output.txt is created but empty (exit code is 1 from command)
3. **Given** user runs `cat < file.txt > file.txt` (same file), **When** command executes, **Then** file is truncated to empty (read/write race condition)
4. **Given** user runs `echo "test" > /root/file.txt` without permissions, **When** redirection attempts, **Then** error message says "permission denied"
5. **Given** user runs `command 2> errors.txt`, **When** command produces errors, **Then** stderr is redirected to errors.txt (stdout still to terminal)

---

### Edge Cases

- What happens when redirecting to a file in a directory that doesn't exist?
- What happens when redirecting to a file that is already open by another process?
- What happens when redirecting output but the command produces no output?
- What happens when redirecting input from an empty file?
- What happens when redirecting input from a binary file to a text-processing command?
- What happens when using redirection operators inside quoted strings?
- What happens when the filesystem is full during output redirection?
- What happens when redirecting to a symbolic link?
- What happens when redirecting both stdout and stderr to the same file?
- What happens when combining redirections with background jobs (`command > output.txt &`)?
- What happens when a command reads from stdin but user provides input redirection?
- What happens with very large files (multi-GB) in input/output redirection?

## Requirements *(mandatory)*

### Functional Requirements

**Output Redirection (`>`)**:

- **FR-001**: System MUST parse `>` operator and treat it as stdout overwrite redirection
- **FR-002**: System MUST create target file if it doesn't exist when using `>`
- **FR-003**: System MUST truncate (empty) target file before writing when using `>`
- **FR-004**: System MUST preserve `>` as literal text when inside quotes (`echo "a > b"` outputs "a > b")
- **FR-005**: System MUST redirect only stdout to file (stderr still goes to terminal unless separately redirected)
- **FR-006**: System MUST report error if target path exists but is a directory

**Append Redirection (`>>`)**:

- **FR-007**: System MUST parse `>>` operator and treat it as stdout append redirection
- **FR-008**: System MUST create target file if it doesn't exist when using `>>`
- **FR-009**: System MUST append to end of file without modifying existing content when using `>>`
- **FR-010**: System MUST preserve `>>` as literal text when inside quotes
- **FR-011**: System MUST seek to end of file before writing (even if file grows between check and write)

**Input Redirection (`<`)**:

- **FR-012**: System MUST parse `<` operator and treat it as stdin redirection
- **FR-013**: System MUST open source file and connect it to command's stdin when using `<`
- **FR-014**: System MUST report error if source file doesn't exist when using `<`
- **FR-015**: System MUST preserve `<` as literal text when inside quotes
- **FR-016**: System MUST allow `<` to appear before or after command (`< file.txt cat` or `cat < file.txt`)

**Combined Operations**:

- **FR-017**: System MUST support combining input and output redirections in single command (`< in.txt command > out.txt`)
- **FR-018**: System MUST support redirections with pipes (`cat < file.txt | grep x > results.txt`)
- **FR-019**: System MUST support multiple output redirections (last one wins: `echo x > a.txt > b.txt` writes to b.txt)
- **FR-020**: System MUST support redirection with background jobs (`command > output.txt &`)

**Error Handling**:

- **FR-021**: System MUST report clear error if output file cannot be created (permissions, full disk, path not found)
- **FR-022**: System MUST report clear error if input file cannot be opened (not found, permissions)
- **FR-023**: System MUST create output file immediately before command execution (to catch errors early)
- **FR-024**: System MUST close file descriptors properly even if command fails or is interrupted
- **FR-025**: System MUST propagate command exit code (not mask it with redirection success/failure)

**Parsing Rules**:

- **FR-026**: System MUST tokenize redirections separately from command arguments
- **FR-027**: System MUST handle whitespace around operators (`>file.txt`, `> file.txt`, `>  file.txt` all valid)
- **FR-028**: System MUST treat `>` and `>>` as distinct operators (not confuse `>>` with two `>`)
- **FR-029**: System MUST support file paths with spaces when quoted (`echo x > "my file.txt"`)
- **FR-030**: System MUST validate redirection syntax before command execution

### Key Entities

- **Redirection**: Represents a single redirection operation with type (`>`, `>>`, `<`), target/source file path, and file descriptor (stdout/stdin)
- **Command**: Extended to include optional list of redirections to apply before execution
- **File Handle**: Represents opened file for reading/writing, with path, mode (read/write/append), and file descriptor

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can redirect command output to files using `>` operator with 100% success rate for valid paths
- **SC-002**: Users can append command output to files using `>>` operator without data loss or corruption
- **SC-003**: Users can redirect command input from files using `<` operator for all commands that read stdin
- **SC-004**: Redirection operations complete with less than 1ms overhead compared to direct command execution (measured via benchmarks)
- **SC-005**: Error messages for redirection failures clearly identify the problem (file not found, permission denied, is a directory) in 100% of cases
- **SC-006**: All existing tests continue to pass (backward compatibility maintained for commands without redirections)
- **SC-007**: Users can successfully combine redirections with pipes (`|`) and background jobs (`&`) in single command
- **SC-008**: Redirection parsing correctly handles quoted strings and preserves operators as literals when quoted
- **SC-009**: File handles are properly closed and cleaned up even when commands are interrupted (Ctrl+C) or fail
- **SC-010**: Redirection feature works identically in both interactive REPL mode and script execution mode (`rush -c "command > file.txt"`)

## Dependencies *(optional)*

### Internal Dependencies

- **Parser**: Requires extending existing command parser to recognize and tokenize redirection operators (`>`, `>>`, `<`)
- **Pipeline Executor**: Must be enhanced to set up file descriptors before spawning processes
- **CommandExecutor**: Needs to handle redirections for both single commands and pipelines

### External Dependencies

- **File System**: Relies on OS file system for creating, opening, reading, and writing files
- **File Descriptors**: Uses Unix-style file descriptors (stdin=0, stdout=1, stderr=2) and requires dup2/redirect capabilities

## Assumptions *(optional)*

- Users are familiar with POSIX shell redirection semantics (same as bash/zsh behavior)
- Target platform (macOS for MVP) supports standard Unix file I/O operations
- File paths follow Unix conventions (forward slashes, case-sensitive)
- Default file permissions for created files are 0644 (rw-r--r--)
- Shell does not implement advanced redirections yet (like `2>`, `&>`, `<<<`, or file descriptor manipulation)
- Redirections are processed left-to-right in order they appear (standard shell behavior)
- Empty files created by redirection are valid and acceptable (not an error condition)

## Out of Scope *(optional)*

The following are explicitly NOT included in this feature:

- **Stderr redirection** (`2>`, `2>>`): Deferred to future feature
- **Combined stdout/stderr** (`&>`, `&>>`): Deferred to future feature
- **Here documents** (`<<`, `<<<`): Deferred to future feature
- **File descriptor manipulation** (`>&3`, `<&4`): Deferred to future feature
- **Process substitution** (`<(command)`, `>(command)`): Deferred to future feature
- **Noclobber option** (`>|` to override): Not needed for MVP
- **Named pipes/FIFOs**: Standard files only
- **Network redirections**: Files only, no sockets or URLs
- **Advanced file locking**: Basic OS-level file operations only

## Notes *(optional)*

### Design Considerations

**Performance**: Redirection setup must be fast (<1ms) to maintain constitution requirement of <5ms command execution overhead. File opening should happen after fork but before exec to minimize parent process blocking.

**Compatibility**: Behavior should match bash/zsh semantics to meet user expectations. Where POSIX behavior is ambiguous or varies, prefer bash-compatible behavior for familiarity.

**Error Clarity**: Error messages should clearly distinguish between:
- File not found for input (`<`)
- Permission denied for output (`>`, `>>`)
- Path is a directory
- Disk full or I/O error

**Quote Handling**: The existing quote parser must be extended to treat redirection operators inside quotes as literal text, not operators. This requires coordination with the parser from feature 004 (pipes).

**Integration**: Should work seamlessly with existing features:
- Feature 001: Basic command execution
- Feature 004: Pipes (combined: `cat < in.txt | grep x > out.txt`)
- Future: Job control (background with redirection: `command > log.txt &`)

### Security Considerations

- Prevent directory traversal attacks by validating file paths
- Respect file system permissions (don't attempt privilege escalation)
- Avoid race conditions between file checks and opens (use open directly, handle errors)
- Close file descriptors properly to prevent leaks
- Limit file descriptor usage to prevent exhaustion

### Testing Strategy

**Unit Tests**:
- Parser correctly tokenizes redirection operators
- Redirections are correctly identified and separated from arguments
- Quote handling preserves operators as literals
- Edge cases: empty files, non-existent paths, invalid permissions

**Integration Tests**:
- End-to-end command execution with each redirection type
- Combined redirections (input + output)
- Redirections with pipes
- Error scenarios with proper error messages
- File content verification after redirections

**Performance Benchmarks**:
- Redirection setup overhead measurement
- Large file handling (verify no buffering bottlenecks)
- Multiple redirections in rapid sequence

**Manual Testing**:
- Interactive REPL usage with redirections
- Real-world workflows (log file creation, data processing)
- Error message clarity and helpfulness
