# Feature 026: Command Groups

**Feature ID**: 026
**Category**: Advanced Features
**Priority**: Medium
**Dependencies**: Feature 001 (Basic Shell)

## Overview

Implement command group syntax for logical grouping without spawning new process.

## User Stories

### US1: Command Groups

**Title**: Group commands with shared I/O redirection
**Priority**: High (P1)

**Description**:
As a shell developer, I want to group commands together so I can apply I/O redirections to multiple commands at once.

**Acceptance Criteria**:
- `{ commands; }` syntax executes in current shell scope
- Multiple commands can be grouped
- Redirections apply to entire group: `{ cmd1; cmd2; } > file`
- Exit code is from last command in group
- Variables set in group persist in parent shell
- `cd` in group affects parent shell
- No new process spawned (unlike subshells)

**Example**:
```bash
$ { echo "Line 1"; echo "Line 2"; echo "Line 3"; } > output.txt
$ cat output.txt
Line 1
Line 2
Line 3
```

---

### US2: Complex Grouping

**Title**: Use command groups in complex scenarios
**Priority**: Medium (P2)

**Description**:
As a shell developer, I want to use command groups in pipes, conditionals, and loops.

**Acceptance Criteria**:
- Command groups work in pipelines
- Command groups work in conditionals
- Command groups work in loops
- Proper exit code handling
- Proper variable scope handling

**Example**:
```bash
$ { echo "Hello"; echo "World"; } | sort
Hello
World

$ if { true; }; then echo "yes"; fi
yes
```

---

## Technical Requirements

### Parser Requirements
- Recognize `{` for group start
- Parse command list
- Recognize `}` for group end
- Handle properly in various contexts (pipes, redirects, etc.)

### Execution Requirements
- Execute commands in current shell scope
- Handle I/O redirection for entire group
- Proper exit code propagation

## Success Metrics

- ✅ All user stories implemented
- ✅ 15+ test cases
- ✅ >95% code coverage

---

**Created**: 2025-12-06
**Status**: Specification Complete
