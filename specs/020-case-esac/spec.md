# Feature 020: Case/Esac Pattern Matching

**Feature ID**: 020
**Category**: Control Flow
**Priority**: High (follows loops)
**Dependencies**: Feature 017 (Conditional Control Flow)

## Overview

Implement POSIX-compliant `case` statement allowing pattern-based branching. This provides an alternative to multiple if/elif chains for complex conditional logic.

## User Stories

### US1: Basic Case Statement (case/esac)

**Title**: Match value against patterns and execute corresponding commands
**Priority**: Critical (P1)

**Description**:
As a shell user, I want to match a value against multiple patterns using a case statement so I can implement cleaner branching logic than if/elif/else chains.

**Acceptance Criteria**:
- `case word in pattern) commands;; esac` syntax works
- Multiple patterns can be provided for single value: `pattern1|pattern2) commands;;`
- Pattern matching includes:
  - Literal strings: `"apple")`
  - Wildcards: `*.txt)`
  - Character sets: `[a-z]*)`
  - `*` matches any value (default case)
- Execute commands for first matching pattern
- `;;` terminates pattern block
- `;&` continues to next pattern (conditional)
- `;;&` tests next pattern without executing (conditional)
- Exit code is from last executed command
- Value can contain variable expansion and command substitution

**Example**:
```bash
$ fruit="apple"
$ case $fruit in
>   apple) echo "Red fruit" ;;
>   banana) echo "Yellow fruit" ;;
>   *) echo "Unknown fruit" ;;
> esac
Red fruit
```

**Edge Cases**:
- No matching patterns (exit code from previous command)
- Empty value
- Multiple matching patterns (first wins with `;;`)
- Patterns with spaces

---

### US2: Pattern Types & Matching

**Title**: Support various pattern types and matching semantics
**Priority**: High (P2)

**Description**:
As a shell user, I want different pattern types to work correctly so I can write flexible case statements.

**Acceptance Criteria**:
- Literal patterns: exact string match
- Wildcard patterns: `*`, `?`, `[abc]`, `[a-z]`, `[!abc]`
- Multiple patterns per case: `pattern1|pattern2|pattern3)`
- Default pattern: `*)` matches anything
- Pattern matching uses glob-like semantics (without expanding files)
- Quoting prevents pattern interpretation
- Exit code of matched block used
- All patterns tested before execution stops

**Example**:
```bash
$ file="test.txt"
$ case $file in
>   *.txt) echo "Text file" ;;
>   *.md) echo "Markdown file" ;;
>   *.{c,h}) echo "C file" ;;
>   *) echo "Other file" ;;
> esac
Text file
```

---

### US3: Complex Commands in Case Blocks

**Title**: Support complex commands in case blocks
**Priority**: High (P2)

**Description**:
As a shell user, I want to use complex commands in case blocks including pipes, redirections, and nested structures.

**Acceptance Criteria**:
- Single command works
- Multiple commands with semicolons
- Pipes and redirections work
- Nested conditionals work (if/then/else/fi)
- Nested loops work (for/while/until)
- Command groups and subshells work
- Exit code is from last command in block

**Example**:
```bash
$ status="running"
$ case $status in
>   running)
>     echo "Process is running"
>     ps aux | grep process
>     ;;
>   stopped)
>     echo "Process is stopped"
>     ;;
> esac
```

---

## Technical Requirements

### Parser Requirements
- Recognize `case` keyword at statement level
- Parse word (value to match)
- Parse `in` keyword
- Parse pattern blocks:
  - Patterns separated by `|`
  - Closing parenthesis `)`
  - Commands (may span multiple lines)
  - Terminator (`;;`, `;&`, or `;;&`)
- Recognize `esac` keyword
- Proper error reporting

### Execution Requirements
- Evaluate word with variable expansion and command substitution
- For each pattern block:
  - Test pattern against word using glob-like matching
  - If matches, execute commands and stop (with `;;`)
  - If matches, continue to next pattern (with `;&`)
  - If matches, test next without executing (with `;;&`)
- Return exit code from executed block or 0 if no match
- Support break statement (Feature 022) to exit case

## Success Metrics

- ✅ All 3 user stories implemented
- ✅ 30+ test cases
- ✅ POSIX compliance verified
- ✅ Pattern matching works correctly
- ✅ >95% code coverage

---

**Created**: 2025-12-06
**Status**: Specification Complete
**Next Phase**: Planning
