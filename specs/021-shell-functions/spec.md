# Feature 021: Shell Functions

**Feature ID**: 021
**Category**: Advanced Features
**Priority**: High
**Dependencies**: Features 017-020 (Control Flow)

## Overview

Implement POSIX-compliant shell functions allowing code reuse and modular shell scripting.

## User Stories

### US1: Function Definition (function/fname())

**Title**: Define reusable functions with commands
**Priority**: Critical (P1)

**Description**:
As a shell developer, I want to define functions so I can reuse code and organize shell scripts modularly.

**Acceptance Criteria**:
- `function name { commands; }` syntax works
- `name() { commands; }` syntax works (both are equivalent)
- Function names must be valid identifiers
- Function body can contain any commands
- Functions persist in shell session (for interactive REPL)
- Nested function definitions allowed
- Exit code is from last command in function

**Example**:
```bash
$ function greet() { echo "Hello $1"; }
$ greet Alice
Hello Alice
```

---

### US2: Function Parameters & Local Variables

**Title**: Pass parameters and use local variables in functions
**Priority**: High (P2)

**Description**:
As a shell developer, I want to pass parameters to functions and use local variables.

**Acceptance Criteria**:
- Function parameters passed as positional arguments ($1, $2, ..., $@)
- `local` keyword creates function-scoped variables
- Local variables don't leak to parent scope
- Function can access global variables (unless shadowed by local)
- `$0` still refers to shell script name (not function name)
- Return value set by `return` statement (Feature 024)

**Example**:
```bash
$ function add() {
>   local sum=$(($1 + $2))
>   echo $sum
> }
$ add 3 4
7
```

---

### US3: Return Values & Exit Codes

**Title**: Functions return exit codes and values
**Priority**: High (P2)

**Description**:
As a shell developer, I want functions to return values and exit codes properly.

**Acceptance Criteria**:
- Function exit code is from last command
- `return` statement sets explicit exit code (Feature 024)
- Function can return values via:
  - Exit code ($?)
  - Output to stdout (command substitution)
  - Setting variables (globals or through parameters)
- Command substitution captures function output
- Exit code from function available in $?

**Example**:
```bash
$ function get_status() { return 42; }
$ get_status
$ echo $?
42

$ function double() { echo $(($1 * 2)); }
$ result=$(double 5)
$ echo $result
10
```

---

## Technical Requirements

### Parser Requirements
- Recognize `function` keyword or function name with `()`
- Parse function name (identifier)
- Parse function body (command list)
- Proper error handling for malformed syntax

### Execution Requirements
- Store function definition in shell environment
- When function called, create new scope for local variables
- Execute function body
- Handle parameter passing
- Return exit code from function
- Clean up local variables after function

### Integration Points
- Feature 001 (Command execution) - execute function commands
- Feature 022 (break) - in functions inside loops
- Feature 024 (return) - explicit return from function

## Success Metrics

- ✅ All 3 user stories implemented
- ✅ 25+ test cases
- ✅ POSIX compliance verified
- ✅ >95% code coverage

---

**Created**: 2025-12-06
**Status**: Specification Complete
