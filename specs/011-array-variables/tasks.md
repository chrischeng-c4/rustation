# Implementation Tasks: Feature 011 - Array Variables

**Feature**: Array Variables
**Branch**: `011-array-variables`
**Status**: Ready for Implementation

## Overview

Implement bash-style indexed arrays supporting:
- Creation: `arr=(a b c)`
- Access: `${arr[0]}` → "a"
- Length: `${#arr[@]}` → 3
- Iteration: `for x in "${arr[@]}"`
- Modification: `arr[1]=new`, `arr+=(x)`

## User Story Summary

| ID | Story | Priority |
|----|-------|----------|
| US1 | Create Arrays | P1 |
| US2 | Access Elements | P1 |
| US3 | Array Length | P1 |
| US4 | Iterate Arrays | P2 |
| US5 | Modify Arrays | P2 |

---

## Phase 1: Data Structures (1-2 days)

- [ ] T001 Create `executor/variables.rs` (if new) or modify to add ArrayValue variant
- [ ] T002 Define Variable enum with Array(Vec<String>) variant
- [ ] T003 Implement array indexing: get element by index with bounds checking
- [ ] T004 Implement array assignment: set element or append
- [ ] T005 Implement array length calculation
- [ ] T006 Write unit tests for array operations

---

## Phase 2: Parsing Array Syntax (2-3 days)

- [ ] T007 [US1] Modify parser to recognize `arr=(a b c)` syntax
- [ ] T008 [US1] Parse array elements separated by whitespace
- [ ] T009 [US1] Create ArrayValue and store in variables
- [ ] T010 [US2] Parse `${arr[i]}` syntax for array access
- [ ] T011 [US2] Implement array element retrieval in expansion
- [ ] T012 [US3] Parse `${#arr[@]}` for array length
- [ ] T013 [US3] Implement length calculation in expansion
- [ ] T014 Write parser tests for all array syntax

---

## Phase 3: Expansion & Integration (2-3 days)

- [ ] T015 [US4] Implement array expansion: `${arr[@]}` → separate arguments
- [ ] T016 [US4] Implement `${arr[*]}` → single string
- [ ] T017 [US4] Support array iteration in for loops
- [ ] T018 [US5] Implement array modification: `arr[i]=value`
- [ ] T019 [US5] Implement array append: `arr+=(val)`
- [ ] T020 [US5] Support multiple element append
- [ ] T021 Write integration tests for all operations

---

## Phase 4: Polish (1-2 days)

- [ ] T022 Error handling: invalid indices, out of bounds
- [ ] T023 Test sparse arrays: missing indices
- [ ] T024 Test nested arrays in substitutions
- [ ] T025 Run full test suite
- [ ] T026 Code review and validation

---

**Total Tasks**: 26
**Estimated Duration**: 6-10 hours
