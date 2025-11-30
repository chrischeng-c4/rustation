# Implementation Tasks: Feature 010 - Command Substitution $(…)

**Feature**: Command Substitution
**Branch**: `010-command-substitution`
**Status**: Ready for Implementation

## Overview

Implement `$()` command substitution with support for:
- Basic substitution: `echo $(date)`
- Nested substitution: `echo $(echo $(pwd))`
- Variable assignment: `var=$(cmd)`
- Multiple arguments: `ls $(find . -name "*.txt")`
- Error handling: Failed substitution prevents execution

## User Story Summary

| ID | Story | Priority | Test |
|----|-------|----------|------|
| US1 | Basic Substitution | P1 | ✓ |
| US2 | Nested Substitution | P2 | ✓ |
| US3 | Variable Assignment | P2 | ✓ |
| US4 | Multiple Arguments | P2 | ✓ |
| US5 | Error Handling | P1 | ✓ |

---

## Phase 1: Lexer & Pattern Identification (2-3 days)

**Goal**: Identify and extract $(...) regions from input

### Tasks

- [ ] T001 Create `crates/rush/src/executor/substitution/mod.rs` with module structure
- [ ] T002 Define Substitution and SubstitutionError types (Position, Command, Nested fields)
- [ ] T003 Implement lexer to scan for $(...) patterns in input string
- [ ] T004 Handle quoted strings: don't match $(...) inside single/double quotes
- [ ] T005 Identify nested substitutions recursively (extract all regions)
- [ ] T006 Extract inner command strings preserving position and nesting info
- [ ] T007 Write lexer unit tests for pattern matching, quotes, nesting

---

## Phase 2: Execution & Output Capture (2-3 days)

**Goal**: Execute inner commands and capture output

### Tasks

- [ ] T008 Create `crates/rush/src/executor/substitution/executor.rs`
- [ ] T009 Implement command execution spawning subprocess
- [ ] T010 Capture stdout to Vec<u8>, handle large outputs (10MB limit)
- [ ] T011 Convert bytes to String, handle UTF-8 errors gracefully
- [ ] T012 Trim trailing newlines from output (standard behavior)
- [ ] T013 Handle non-zero exit codes: store error info, abort execution
- [ ] T014 Handle command not found errors clearly
- [ ] T015 Write executor tests for spawning, output capture, error handling

---

## Phase 3: Expansion & Integration (3-4 days)

**Goal**: Integrate captured output back into command

### Tasks

- [ ] T016 [US1] Create `crates/rush/src/executor/substitution/expander.rs`
- [ ] T017 [US1] Implement word splitting: split output by whitespace (POSIX rules)
- [ ] T018 [US1] Replace $(...) in token stream with captured tokens
- [ ] T019 [US1] Integrate into parser: call expand_substitutions() before parsing
- [ ] T020 [US1] Modify `executor/parser.rs` to call expander for each command
- [ ] T021 [US2] Implement nested substitution: recursive expansion (innermost first)
- [ ] T022 [US2] Handle substitution within substitution correctly
- [ ] T023 [US3] Support substitution in variable assignment: `var=$(cmd)`
- [ ] T024 [US3] Assign captured output to variable storage
- [ ] T025 [US4] Verify multiple arguments passed correctly from substitution
- [ ] T026 [US5] Propagate errors: abort if inner command fails
- [ ] T027 Write integration tests: each user story end-to-end

---

## Phase 4: Advanced Features & Polish (1-2 days)

**Goal**: Error messages, edge cases, validation

### Tasks

- [ ] T028 Implement clear error messages for failed substitutions
- [ ] T029 Test empty output handling: `${empty=$()} → empty args`
- [ ] T030 Test large output handling: verify 10MB limit works
- [ ] T031 Test special characters in output: newlines, tabs, quotes
- [ ] T032 Handle very deeply nested substitutions
- [ ] T033 Run full test suite: `cargo test -p rush`
- [ ] T034 Run clippy: `cargo clippy -p rush --all-targets`
- [ ] T035 Final validation: all user stories pass

---

**Total Tasks**: 35
**Estimated Duration**: 10-14 hours
**MVP Scope**: T001-T020 (basic + nested)
