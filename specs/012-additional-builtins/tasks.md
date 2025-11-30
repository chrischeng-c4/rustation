# Implementation Tasks: Feature 012 - Additional Built-in Commands

**Feature**: Additional Built-in Commands (echo, printf, test, true, false, pwd, type, [)
**Branch**: `012-additional-builtins`
**Status**: Ready for Implementation

## Overview

Implement standard POSIX shell builtins:
- `echo`: Print text with newline (supports `-n` flag)
- `true`: Always succeeds (exit code 0)
- `false`: Always fails (exit code 1)
- `test`/`[`: Conditional testing (file, string, numeric operations)
- `printf`: Formatted output (basic format strings)
- `pwd`: Print working directory
- `type`: Show command type

## User Story Summary

| ID | Story | Priority |
|----|-------|----------|
| US1 | Echo with newline handling | P1 |
| US2 | Printf formatted output | P2 |
| US3 | Test conditional expressions | P1 |
| US4 | True/False exit codes | P2 |
| US5 | Pwd/Type utilities | P2 |

---

## Phase 1: Echo & True/False Builtins (1-2 days)

- [ ] T001 [P1] [US1] Create `executor/builtins/echo.rs` module skeleton
- [ ] T002 [P1] [US1] Define BuiltinError enum for echo error handling
- [ ] T003 [P1] [US1] Implement parse_echo_args to handle `-n` flag and text arguments
- [ ] T004 [P1] [US1] Implement echo function: print args with spaces, newline by default
- [ ] T005 [P1] [US1] Handle `-n` flag to suppress trailing newline
- [ ] T006 [P1] [US4] Create `executor/builtins/true.rs` module
- [ ] T007 [P1] [US4] Implement true builtin: always return exit code 0
- [ ] T008 [P1] [US4] Create `executor/builtins/false.rs` module
- [ ] T009 [P1] [US4] Implement false builtin: always return exit code 1
- [ ] T010 [P1] [US1/US4] Write unit tests for echo (with/without -n), true, false
- [ ] T011 [P1] [US1/US4] Register echo, true, false in `executor/builtins/mod.rs` dispatcher

---

## Phase 2: Test Builtin (2-3 days)

- [ ] T012 [P1] [US3] Create `executor/builtins/test.rs` module with TestError enum
- [ ] T013 [P1] [US3] Define TestOperator enum for all supported test operations
  - File tests: `-f`, `-d`, `-e`, `-n` (not empty), `-z` (empty string)
  - String tests: `=`, `!=`, `-n`, `-z`
  - Numeric tests: `-eq`, `-ne`, `-lt`, `-le`, `-gt`, `-ge`
- [ ] T014 [P1] [US3] Implement parse_test_args to extract operator and operands from args
- [ ] T015 [P1] [US3] Implement file existence test: `-f` (regular file exists)
- [ ] T016 [P1] [US3] Implement directory test: `-d` (directory exists)
- [ ] T017 [P1] [US3] Implement path existence test: `-e` (any path exists)
- [ ] T018 [P1] [US3] Implement string length tests: `-n` (non-empty), `-z` (empty)
- [ ] T019 [P1] [US3] Implement string equality test: `=` and `!=` operators
- [ ] T020 [P1] [US3] Implement numeric comparison tests: `-eq`, `-ne`, `-lt`, `-le`, `-gt`, `-ge`
- [ ] T021 [P1] [US3] Handle unary operations (single operand: `-n file`, `-z var`)
- [ ] T022 [P1] [US3] Handle binary operations (two operands: `str1 = str2`, `int1 -eq int2`)
- [ ] T023 [P1] [US3] Implement proper exit codes: 0 for true, 1 for false
- [ ] T024 [P1] [US3] Create `executor/builtins/bracket.rs` for `[` builtin (delegates to test)
- [ ] T025 [P1] [US3] Write comprehensive unit tests for all test operators and edge cases

---

## Phase 3: Printf Builtin (1-2 days)

- [ ] T026 [P2] [US2] Create `executor/builtins/printf.rs` module with PrintfError enum
- [ ] T027 [P2] [US2] Implement parse_printf_format to extract format string and arguments
- [ ] T028 [P2] [US2] Implement format string parsing: identify %s, %d, %c, %% placeholders
- [ ] T029 [P2] [US2] Implement %s (string) formatter: interpolate string arguments
- [ ] T030 [P2] [US2] Implement %d (integer) formatter: parse and format integers
- [ ] T031 [P2] [US2] Implement %c (character) formatter: output single character from string
- [ ] T032 [P2] [US2] Implement %% escape: literal % in output
- [ ] T033 [P2] [US2] Handle escape sequences: `\n` (newline), `\t` (tab), `\\` (backslash)
- [ ] T034 [P2] [US2] Handle missing arguments: use empty strings or 0 for missing values
- [ ] T035 [P2] [US2] Write unit tests for printf with various format strings and arguments

---

## Phase 4: Pwd & Type Utilities (1 day)

- [ ] T036 [P2] [US5] Create `executor/builtins/pwd.rs` module
- [ ] T037 [P2] [US5] Implement pwd builtin: use `std::env::current_dir()` to get working directory
- [ ] T038 [P2] [US5] Handle pwd errors: report errors when current directory is unavailable
- [ ] T039 [P2] [US5] Create `executor/builtins/type.rs` module with TypeError enum
- [ ] T040 [P2] [US5] Implement type command: determine if argument is builtin/function/command
- [ ] T041 [P2] [US5] Check builtins list first in type command
- [ ] T042 [P2] [US5] Check PATH for external commands in type command
- [ ] T043 [P2] [US5] Write unit tests for pwd and type builtins

---

## Phase 5: Polish (1 day)

- [ ] T044 [US1-US5] Error handling: all builtins return non-zero on error
- [ ] T045 [US1-US5] Test escape sequence handling in echo and printf (edge cases)
- [ ] T046 [US1-US5] Test special characters in arguments (quotes, spaces, special chars)
- [ ] T047 [US1-US5] Test all builtins with no arguments (error handling)
- [ ] T048 [US1-US5] Test all builtins with too many arguments (error handling)
- [ ] T049 [US1-US5] Run full test suite: `cargo test -p rush`
- [ ] T050 [US1-US5] Run clippy: `cargo clippy -p rush --all-targets`

---

**Total Tasks**: 50
**Estimated Duration**: 5-8 hours

## MVP Scope

**Minimum Viable Product (T001-T025)**:
- Echo with -n flag
- True/False builtins
- Test with file/string operations
- Bracket [  alternative
- Estimated: 3-4 hours

**Full Feature (T001-T050)**:
- All MVP items plus:
- Printf with basic format strings
- Pwd and type utilities
- Comprehensive error handling and polish
- Estimated: 5-8 hours
