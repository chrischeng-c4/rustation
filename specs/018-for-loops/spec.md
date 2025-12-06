# Feature 018: For/In/Do/Done Loops

**Feature ID**: 018
**Category**: Control Flow
**Priority**: High (follows immediately after conditionals)
**Dependencies**: Feature 017 (Conditional Control Flow)

## Overview

Implement POSIX-compliant `for` loops allowing iteration over word lists with command execution for each iteration. This is a fundamental control flow construct for shell scripting.

## User Stories

### US1: Basic For Loop (for/in/do/done)

**Title**: Execute commands for each word in a list
**Priority**: Critical (P1)

**Description**:
As a shell user, I want to iterate over a list of words using a for loop so I can execute commands repeatedly with different values.

**Acceptance Criteria**:
- `for var in word1 word2 ... wordN; do commands; done` syntax works correctly
- Loop variable is bound to each word sequentially
- Loop body executes for each word
- Final exit code is the exit code of the last executed command
- Loop variable persists after loop completion with the last value

**Example**:
```bash
$ for name in Alice Bob Charlie; do echo "Hello $name"; done
Hello Alice
Hello Bob
Hello Charlie
```

**Edge Cases**:
- Empty word list (loop doesn't execute, exit code 0)
- Single word in list
- Words with spaces (when quoted)
- Special characters in words (globbing should not occur in explicit word list)
- Variable expansion in word list
- Command substitution in word list

---

### US2: Default Word List (Positional Parameters)

**Title**: Iterate over positional parameters when no word list provided
**Priority**: High (P2)

**Description**:
As a shell user, I want to iterate over positional parameters using `for var; do` syntax (without explicit word list) when no arguments are provided.

**Acceptance Criteria**:
- `for var; do commands; done` iterates over `$@` (all positional parameters)
- `for var; do commands; done` is equivalent to `for var in "$@"; do commands; done`
- Works correctly with 0, 1, or multiple positional parameters
- Proper handling of parameters with spaces and special characters
- Exit code semantics match US1

**Example**:
```bash
$ function iterate() { for item; do echo $item; done; }
$ iterate apple banana cherry
apple
banana
cherry
```

**Edge Cases**:
- No positional parameters (loop doesn't execute)
- Single positional parameter
- Parameters containing spaces
- Parameters starting with special characters ($, *, etc.)

---

### US3: Word List Expansion

**Title**: Support variable expansion and command substitution in word list
**Priority**: High (P2)

**Description**:
As a shell user, I want variable expansion and command substitution to work in the word list so I can dynamically generate iteration values.

**Acceptance Criteria**:
- Variable expansion (`$VAR`, `${VAR}`) works in word list
- Command substitution (`$(cmd)` or `` `cmd` ``) works in word list
- Results are split on word boundaries (IFS)
- Brace expansion works in word list
- Globbing does NOT occur in word list (words are used as-is after expansion)
- Proper quoting prevents unwanted expansion

**Example**:
```bash
$ names="John Jane"
$ for person in $names extra; do echo $person; done
John
Jane
extra

$ for file in $(ls *.txt); do echo "Processing $file"; done
Processing file1.txt
Processing file2.txt
```

**Edge Cases**:
- Empty variable expansion (no words added)
- Variable containing multiple words (should split)
- Command substitution returning multiple lines (should split)
- Nested expansions

---

### US4: Loop Variable Scoping

**Title**: Loop variable binding and scoping rules
**Priority**: Medium (P3)

**Description**:
As a shell user, I want proper variable scoping behavior so that loop variables don't unexpectedly shadow or corrupt existing variables.

**Acceptance Criteria**:
- Loop variable is created if it doesn't exist
- Loop variable overwrites existing variable with same name
- After loop completion, loop variable retains its last assigned value
- If loop doesn't execute (empty word list), loop variable is not modified
- Loop variable is in the current shell scope (not subshell)
- Loop variable is visible to all commands in the loop body

**Example**:
```bash
$ x="original"
$ for x in one two three; do echo $x; done
one
two
three
$ echo $x
three

$ for y in; do echo "never"; done
$ echo "y is: $y"
y is:
```

**Edge Cases**:
- Loop variable shadows function parameter
- Multiple nested loops with same variable name
- Loop variable used in nested conditional or loop

---

### US5: Complex Loop Bodies

**Title**: Support complex command sequences in loop body
**Priority**: High (P2)

**Description**:
As a shell user, I want to use complex command sequences including pipes, redirections, and nested control structures in the loop body.

**Acceptance Criteria**:
- Simple command (single command) works in loop body
- Command pipeline works (`cmd1 | cmd2 | cmd3`)
- Output redirection works (`>`, `>>`, `<`)
- Multiple commands separated by semicolons work
- Nested conditionals work (if/then/else/fi inside for loop)
- Nested loops work (for/while/until inside for loop)
- Command groups work `{ cmd1; cmd2; }`
- Subshells work `( cmd1; cmd2 )`
- Exit code reflects the exit code of the last command in the loop body

**Example**:
```bash
$ for num in 1 2 3; do
>   if [ $num -eq 2 ]; then
>     echo "Found two"
>   else
>     echo "Number: $num"
>   fi
> done
Number: 1
Found two
Number: 3

$ for file in *.txt; do
>   lines=$(wc -l < "$file")
>   echo "$file has $lines lines"
> done
```

**Edge Cases**:
- Loop body is a pipeline (exit code is from last command)
- Loop body has subshell (variables modified in subshell don't affect outer scope)
- Loop body uses break/continue (feature 022/023)

---

## Technical Requirements

### Parser Requirements
- Recognize `for` keyword at statement level
- Parse `var` (identifier)
- Parse optional `in` keyword followed by word list
- Parse `do` keyword
- Parse command list (loop body) recursively
- Parse `done` keyword
- Handle newlines and semicolons as statement separators
- Proper error reporting for malformed syntax

### Execution Requirements
- Expand word list using current shell expansion rules (parameters, command substitution, brace expansion)
- For each word in the expanded list:
  - Bind loop variable to the word
  - Execute loop body
  - Capture exit code
- Return exit code of last iteration (or 0 if no iterations)
- Handle break/continue signals (when implemented in features 022/023)

### Integration Points
- Feature 001 (Command execution) - execute commands in loop body
- Feature 017 (Conditionals) - support nested if statements
- Feature 019 (while/until loops) - parallel implementation for while/until
- Feature 022 (break) - implement break statement support
- Feature 023 (continue) - implement continue statement support

## Success Metrics

- ✅ All 5 user stories fully implemented
- ✅ 45+ test cases (unit and integration combined)
  - US1: 8 tests (basic, empty list, single word, special chars, expansions)
  - US2: 8 tests (positional parameters with 0, 1, N args)
  - US3: 12 tests (variable expansion, command substitution, globbing prevention)
  - US4: 8 tests (variable scoping, shadowing, persistence)
  - US5: 9 tests (pipes, redirects, nested structures)
- ✅ POSIX compatibility verified against reference shell behavior
- ✅ Performance: loops with 100+ iterations complete in <100ms
- ✅ All tests pass with 100% code coverage for for-loop implementation
- ✅ Documentation includes usage examples and edge case handling

## POSIX Specification Reference

From POSIX.1-2017 (Shell and Utilities):

```
For Loop:
  for name in [word ...] ; do list ; done
  for name ; do list ; done          (equivalent to: for name in "$@" ; do list ; done)
```

This feature implements the core for-loop construct as specified in POSIX, excluding the `in` keyword variants that reference shell functions (feature 021).

## Architecture Notes

The for loop implementation will follow the pattern established by Feature 017 (conditionals):

1. **Parser** (`executor/loop.rs` or `executor/for_loop.rs`)
   - Recursive descent parsing with keyword detection
   - Word list expansion using existing expansion utilities
   - Proper error handling and reporting

2. **AST** (`executor/mod.rs`)
   - `ForLoop` struct containing:
     - variable name (String)
     - word list (Vec<String> or expression)
     - body (Vec<Command>)

3. **Executor** (`executor/execute.rs`)
   - Detect `for` keyword
   - Delegate to for-loop handler
   - Iterate and execute body

4. **Integration** (`repl/mod.rs`)
   - Support multiline for loops in REPL
   - Completion tracking for nested structures

## Constraints

- Must maintain single-threaded execution model
- Must preserve variable scope rules (no subshells unless explicitly requested)
- Word list expansion order matters (left to right, with proper IFS handling)
- Must handle signals properly (SIGINT, SIGTERM)
- Exit codes must follow POSIX semantics

## Notes

- Feature 018 builds directly on Feature 017 (conditional control flow)
- Break and continue (features 022/023) will be implemented as separate features that integrate with the for loop body execution
- While and until loops (Feature 019) will use similar infrastructure
- This feature is essential for any real shell usage (scripting, batch operations)

---

**Created**: 2025-12-06
**Status**: Specification Complete
**Next Phase**: Planning (create plan.md)
