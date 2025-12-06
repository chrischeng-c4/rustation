# Feature 023: Continue Statement

**Feature ID**: 023
**Category**: Loop Control
**Priority**: High
**Dependencies**: Features 018-019 (Loops)

## Overview

Implement `continue` statement for skipping to the next loop iteration.

## User Stories

### US1: Continue Loop Iteration

**Title**: Skip remaining commands and proceed to next iteration
**Priority**: Critical (P1)

**Description**:
As a shell developer, I want to skip remaining loop body commands using the `continue` statement.

**Acceptance Criteria**:
- `continue` statement skips to next iteration of innermost loop
- `continue n` continues from n nested loops
- Works in for, while, and until loops
- Exit code is 0
- Condition is re-evaluated (for while/until)
- Outer structures not affected

**Example**:
```bash
$ for i in 1 2 3 4 5; do
>   if [ $i -eq 3 ]; then continue; fi
>   echo $i
> done
1
2
4
5
```

---

## Technical Requirements

### Parser Requirements
- Recognize `continue` keyword
- Parse optional count argument

### Execution Requirements
- Signal innermost loop to continue
- Support n-level continue for nested loops
- Proper scope handling

## Success Metrics

- ✅ User story implemented
- ✅ 15+ test cases
- ✅ >95% code coverage

---

**Created**: 2025-12-06
**Status**: Specification Complete
