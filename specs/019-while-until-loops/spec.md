# Feature 019: While/Until Loops

**Feature ID**: 019
**Category**: Control Flow
**Priority**: High (follows immediately after for loops)
**Dependencies**: Feature 017 (Conditional Control Flow), Feature 018 (For Loops)

## Overview

Implement POSIX-compliant `while` and `until` loops allowing iteration based on condition evaluation. These complement for loops by enabling conditional iteration patterns.

## User Stories

### US1: While Loop (while/do/done)

**Title**: Repeat commands while condition is true
**Priority**: Critical (P1)

**Description**:
As a shell user, I want to repeatedly execute commands while a condition is true using a while loop so I can iterate based on dynamic conditions.

**Acceptance Criteria**:
- `while condition; do commands; done` syntax works correctly
- Condition is evaluated before each iteration
- If condition is true (exit code 0), loop body executes
- If condition is false (non-zero exit code), loop exits
- Exit code is from the last executed command (or 0 if loop never executes)
- Loop continues until condition becomes false
- Loop can be infinite if condition always evaluates to true

**Example**:
```bash
$ count=0
$ while [ $count -lt 3 ]; do
>   echo "Count: $count"
>   count=$((count + 1))
> done
Count: 0
Count: 1
Count: 2
```

**Edge Cases**:
- Condition always true (infinite loop, requires Ctrl+C)
- Condition always false (loop doesn't execute)
- Condition with side effects (variable modifications)
- Complex conditions with operators
- Condition uses variables modified in loop body

---

### US2: Until Loop (until/do/done)

**Title**: Repeat commands until condition becomes true
**Priority**: Critical (P1)

**Description**:
As a shell user, I want to repeatedly execute commands until a condition becomes true using an until loop for negation-based iteration patterns.

**Acceptance Criteria**:
- `until condition; do commands; done` syntax works correctly
- Condition is evaluated before each iteration
- If condition is false (non-zero exit code), loop body executes
- If condition is true (exit code 0), loop exits
- Exit code is from the last executed command (or 0 if loop never executes)
- Loop continues until condition becomes true
- `until cond; do X; done` is equivalent to `while ! cond; do X; done`

**Example**:
```bash
$ count=0
$ until [ $count -eq 3 ]; do
>   echo "Count: $count"
>   count=$((count + 1))
> done
Count: 0
Count: 1
Count: 2
```

**Edge Cases**:
- Condition always false (infinite loop)
- Condition always true (loop doesn't execute)
- Complex conditions
- Equivalence with negated while loop

---

### US3: Condition Evaluation & Exit Codes

**Title**: Proper condition evaluation and exit code handling
**Priority**: High (P2)

**Description**:
As a shell user, I want conditions to be evaluated correctly with proper exit code semantics so control flow matches expected behavior.

**Acceptance Criteria**:
- Condition is any command or pipeline
- Exit code 0 means "true" for while (condition met, continue loop)
- Non-zero exit code means "false" for while (condition failed, exit loop)
- For until, semantics are inverted (0 = true = exit, non-zero = false = continue)
- Condition can be:
  - Simple command: `while true; do ...`
  - Command with arguments: `while [ $x -lt 10 ]; do ...`
  - Pipeline: `while grep pattern file | wc -l > 0; do ...`
  - Command group: `while { test -f "$file"; }; do ...`
  - Negated command: `while ! command; do ...`
- Complex conditions with && and || work
- Exit code from loop reflects last command in body

**Example**:
```bash
$ while true; do echo "Loop"; break; done  # (break not yet implemented)
Loop

$ until false; do echo "Loop"; break; done  # (break not yet implemented)
Loop

$ while [ -f /tmp/flag ]; do sleep 1; done
# Loop continues while /tmp/flag exists
```

**Edge Cases**:
- Condition is a negation (`! command`)
- Condition modifies variables (side effects)
- Condition modifies files/state
- Condition with complex operators

---

### US4: Loop Body Execution

**Title**: Support complex loop bodies with proper exit code handling
**Priority**: High (P2)

**Description**:
As a shell user, I want complex command sequences, nested structures, and proper exit code handling in loop bodies.

**Acceptance Criteria**:
- Simple commands work in loop body
- Multiple commands separated by semicolons work
- Pipes, redirections, and command groups work
- Nested conditionals (if/then/else/fi) work in loop body
- Nested loops (for/while/until) work in loop body
- Exit code of loop is from last command in final iteration
- If loop doesn't execute, exit code is 0
- Break statement (Feature 022) will work in loop body
- Continue statement (Feature 023) will work in loop body

**Example**:
```bash
$ count=0
$ while [ $count -lt 3 ]; do
>   if [ $count -eq 1 ]; then
>     echo "One"
>   else
>     echo "Other: $count"
>   fi
>   count=$((count + 1))
> done
Other: 0
One
Other: 2
```

**Edge Cases**:
- Loop body is a pipeline
- Loop body modifies loop condition variable
- Nested loops with same condition variable
- Subshells in loop body

---

### US5: Loop Control (Preparation for Features 022/023)

**Title**: Prepare infrastructure for break and continue statements
**Priority**: Medium (P3)

**Description**:
As a developer, I want the while/until loop implementation to support break and continue statements when implemented in Features 022/023.

**Acceptance Criteria**:
- Loop execution tracks loop context (for break/continue signal propagation)
- Exit signal handling allows breaking out of loops
- Continue signal handling allows skipping to next iteration
- Nested loops properly propagate signals to correct loop
- Integration points for Features 022/023 are clear and documented

**Notes**:
- This US establishes the architecture for loop control
- Actual break/continue implementation is in Features 022/023
- This feature documents the integration points

**Example** (when Features 022/023 are implemented):
```bash
$ for i in 1 2 3 4 5; do
>   if [ $i -eq 3 ]; then
>     continue  # Skip to next iteration
>   fi
>   if [ $i -eq 4 ]; then
>     break     # Exit loop
>   fi
>   echo $i
> done
1
2
5  # Never printed, loop exited at 4
```

---

## Technical Requirements

### Parser Requirements
- Recognize `while` or `until` keywords at statement level
- Parse condition (any command, command group, or pipeline)
- Parse `do` keyword
- Parse command list (loop body) recursively
- Parse `done` keyword
- Handle newlines and semicolons as statement separators
- Proper error reporting for malformed syntax
- Support condition negation with `!` operator

### Execution Requirements
- Evaluate condition before each iteration
- For while: continue loop if condition exit code is 0
- For until: continue loop if condition exit code is non-zero
- Execute loop body if condition matches
- Continue loop, re-evaluate condition
- Return exit code of last executed command (or 0 if no execution)
- Support break/continue signals (when Features 022/023 implemented)

### Integration Points
- Feature 001 (Command execution) - execute conditions and loop body
- Feature 017 (Conditionals) - support nested if statements
- Feature 018 (For loops) - parallel control flow structure
- Feature 022 (break) - implement break statement support
- Feature 023 (continue) - implement continue statement support

## Success Metrics

- ✅ All 5 user stories fully implemented
- ✅ 35+ test cases (unit and integration combined)
  - US1: 8 tests (basic while, infinite while, false condition, variable updates)
  - US2: 8 tests (basic until, infinite until, true condition, equivalence to while)
  - US3: 10 tests (exit codes, complex conditions, negation, pipelines)
  - US4: 6 tests (pipes, redirects, nested structures)
  - US5: 3 tests (loop control infrastructure)
- ✅ POSIX compatibility verified against reference shell behavior
- ✅ Performance: loops with 100+ iterations complete in <100ms
- ✅ All tests pass with 100% code coverage for while/until implementation
- ✅ Documentation includes usage examples and edge case handling

## POSIX Specification Reference

From POSIX.1-2017 (Shell and Utilities):

```
While Loop:
  while list ; do list ; done

Until Loop:
  until list ; do list ; done
```

Where `list` is a compound list (one or more pipelines separated by `;`, `&`, `&&`, or `||`).

## Architecture Notes

The while/until loop implementation will follow the pattern established by Features 017-018:

1. **Parser** (`executor/while_loop.rs` - NEW)
   - Recursive descent parsing
   - Condition parsing (any command list)
   - Body parsing (compound list)

2. **AST** (`executor/mod.rs` - MODIFY)
   - `WhileLoop` and `UntilLoop` structs

3. **Executor** (`executor/execute.rs` - MODIFY)
   - Detect `while` and `until` keywords
   - Delegate to loop handlers
   - Condition evaluation with proper exit code interpretation

4. **Integration** (`repl/mod.rs` - MODIFY)
   - Support multiline while/until loops in REPL
   - Completion tracking for nested structures

## Constraints

- Must maintain single-threaded execution model
- Must preserve variable scope rules
- Exit code semantics must follow POSIX (0 = success/true, non-zero = failure/false)
- Must handle signals properly during loop execution
- Condition evaluation must not have unexpected side effects

## Notes

- Feature 019 builds on Features 017-018 (control flow foundation)
- while/until provide condition-based iteration vs. for's list-based iteration
- Together with for loops, while/until provide complete control flow coverage
- break and continue (features 022/023) will integrate naturally with this architecture

---

**Created**: 2025-12-06
**Status**: Specification Complete
**Next Phase**: Planning (create plan.md)
