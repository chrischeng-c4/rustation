# Feature Specification: Command Substitution

## Overview
Command substitution allows the output of a command to replace the command itself in the command line. Rush will support the modern `$(command)` syntax for command substitution.

## User Stories

### US1: Basic Command Substitution
**As a** shell user
**I want** to use `$(command)` to substitute command output
**So that** I can use the output of one command as arguments to another

**Acceptance Criteria:**
- `echo $(date)` executes `date` and prints its output
- `echo "Today is $(date)"` works inside double quotes
- Multiple substitutions in one command: `echo $(whoami) $(date)`
- Whitespace in output is preserved within quotes
- Newlines in output are converted to spaces when unquoted

**Examples:**
```bash
# Basic substitution
$ echo $(whoami)
chrischeng

# Inside quotes
$ echo "User: $(whoami)"
User: chrischeng

# As command arguments
$ ls -l $(which bash)
-rwxr-xr-x  1 root  wheel  1234567 Jan 1 12:00 /bin/bash

# Multiple substitutions
$ echo $(whoami) is using $(echo rush)
chrischeng is using rush
```

### US2: Command Substitution in Different Contexts
**As a** shell user
**I want** command substitution to work in various contexts
**So that** I have flexibility in how I use it

**Acceptance Criteria:**
- Works as command arguments: `ls $(echo "-la")`
- Works in variable assignments: `export TODAY=$(date)`
- Works in redirections: `cat < $(echo "file.txt")`
- Substitution happens before variable expansion

**Examples:**
```bash
# In arguments
$ ls $(echo "-la")
total 24
drwxr-xr-x  5 user  staff  160 Nov 27 12:00 .
...

# In variables
$ export TODAY=$(date)
$ echo $TODAY
Wed Nov 27 12:00:00 PST 2025

# In redirections
$ cat < $(echo "input.txt")
[contents of input.txt]
```

### US3: Nested Command Substitution
**As a** shell user
**I want** to nest command substitutions
**So that** I can build complex command pipelines

**Acceptance Criteria:**
- Inner substitutions are evaluated first
- `echo $(echo $(whoami))` works correctly
- Arbitrary nesting depth is supported
- Parentheses are properly matched

**Examples:**
```bash
# Simple nesting
$ echo $(echo $(whoami))
chrischeng

# Practical nesting
$ cat $(find $(pwd) -name "*.txt" | head -1)
[contents of first .txt file in current directory]
```

## Technical Requirements

### Parsing
- Detect `$(` and `)` patterns in command line
- Handle nested parentheses correctly
- Preserve quotes inside command substitution
- Handle escaping: `\$(` should be literal

### Execution
- Execute substituted command in a subshell
- Capture stdout of the command
- Trim trailing newlines from output (POSIX behavior)
- Convert internal newlines to spaces when unquoted
- Preserve internal newlines when quoted

### Error Handling
- Unclosed `$(` should error: "Unclosed command substitution"
- Command execution failure should propagate
- Empty substitution `$()` should result in empty string

## Out of Scope
- Backtick syntax `` `command` `` (legacy, not implementing)
- Arithmetic expansion `$((...))`
- Process substitution `<(command)` or `>(command)`

## Testing Strategy
- Unit tests for parsing `$(...)` patterns
- Unit tests for nested substitution parsing
- Integration tests for command execution with substitution
- Integration tests for substitution in quotes
- Integration tests for nested substitutions
- Error handling tests for unclosed substitutions
