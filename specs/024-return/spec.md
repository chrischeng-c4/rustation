# Feature 024: Return Statement

**Feature ID**: 024
**Category**: Functions
**Priority**: High
**Dependencies**: Feature 021 (Shell Functions)

## Overview

Implement `return` statement for explicit function exit with status code.

## User Stories

### US1: Return from Functions

**Title**: Exit function with specific exit code
**Priority**: Critical (P1)

**Description**:
As a shell developer, I want to exit functions with a specific exit code using the `return` statement.

**Acceptance Criteria**:
- `return` statement exits function
- `return n` sets exit code to n (0-255)
- `return` without argument returns 0
- Exit code available in $?
- Can be used inside nested structures (if, loops)
- Only valid inside functions

**Example**:
```bash
$ function check_file() {
>   if [ ! -f "$1" ]; then
>     return 1
>   fi
>   return 0
> }
$ check_file /etc/passwd
$ echo $?
0
```

---

## Technical Requirements

### Parser Requirements
- Recognize `return` keyword
- Parse optional exit code argument

### Execution Requirements
- Signal function to exit
- Set exit code
- Propagate exit code to caller

## Success Metrics

- ✅ User story implemented
- ✅ 10+ test cases
- ✅ >95% code coverage

---

**Created**: 2025-12-06
**Status**: Specification Complete
