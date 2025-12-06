# Feature 019 Task Breakdown: While/Until Loops

**Feature**: 019-while-until-loops
**Total Tasks**: 48 tasks across 3 phases
**Estimated Effort**: 30-40 hours

---

## Phase 1: Core Parser & Executor (Days 1-2)

### Parser Foundation
- [ ] **T1.1** Create `crates/rush/src/executor/while_loop.rs` module
- [ ] **T1.2** Define `LoopType` enum (While, Until) in `executor/mod.rs`
- [ ] **T1.3** Define `WhileLoop` struct with fields: loop_type, condition, body
- [ ] **T1.4** Implement `parse_while_loop(input: &str, loop_type: LoopType) -> Result<WhileLoop, RushError>`
- [ ] **T1.5** Add `while` keyword detection in command parser
- [ ] **T1.6** Add `until` keyword detection in command parser
- [ ] **T1.7** Implement error handling for missing `do` keyword
- [ ] **T1.8** Implement error handling for missing `done` keyword

### Condition Parsing
- [ ] **T1.9** Parse condition as command sequence (until `do` keyword)
- [ ] **T1.10** Handle simple command conditions: `while true; do`
- [ ] **T1.11** Handle test command conditions: `while [ -f file ]; do`
- [ ] **T1.12** Handle complex conditions: `while grep pattern file; do`
- [ ] **T1.13** Properly distinguish condition from loop body
- [ ] **T1.14** Handle whitespace and newlines in condition

### Body Parsing
- [ ] **T1.15** Implement recursive command list parsing for loop body
- [ ] **T1.16** Parse commands separated by semicolons and newlines
- [ ] **T1.17** Properly detect `done` keyword (boundary detection)
- [ ] **T1.18** Handle nested structures in body (if statements)

### Basic Execution
- [ ] **T1.19** Implement `WhileLoop::execute()` method
- [ ] **T1.20** Evaluate condition before each iteration
- [ ] **T1.21** For while: continue if condition exit code = 0
- [ ] **T1.22** For until: continue if condition exit code ≠ 0
- [ ] **T1.23** Execute loop body for each iteration
- [ ] **T1.24** Capture exit code from last command in body
- [ ] **T1.25** Return proper exit code (or 0 if no iterations)
- [ ] **T1.26** Return exit code from last iteration

### Phase 1 Tests
- [ ] **T1.27** Unit test: `test_parse_while_basic`
- [ ] **T1.28** Unit test: `test_parse_until_basic`
- [ ] **T1.29** Unit test: `test_parse_while_missing_do`
- [ ] **T1.30** Unit test: `test_parse_while_missing_done`
- [ ] **T1.31** Integration test: `test_while_basic_iteration`
- [ ] **T1.32** Integration test: `test_while_false_condition`
- [ ] **T1.33** Integration test: `test_until_true_condition`
- [ ] **T1.34** Integration test: `test_while_exit_code`

---

## Phase 2: Complex Conditions & Loop Bodies (Days 3-4)

### Complex Condition Support
- [ ] **T2.1** Support test commands: `while [ $x -lt 10 ]; do`
- [ ] **T2.2** Support pipelines in conditions
- [ ] **T2.3** Support command groups in conditions: `while { test -f "$file"; }; do`
- [ ] **T2.4** Support negated commands: `while ! command; do`
- [ ] **T2.5** Support compound conditions with && and ||
- [ ] **T2.6** Verify until/while equivalence with negation
- [ ] **T2.7** Handle condition side effects (variable modifications)

### Complex Loop Bodies
- [ ] **T2.8** Support command sequences (multiple commands)
- [ ] **T2.9** Support pipes in loop body
- [ ] **T2.10** Support redirections in loop body
- [ ] **T2.11** Support command groups in loop body
- [ ] **T2.12** Support subshells in loop body
- [ ] **T2.13** Proper exit code from pipeline in body
- [ ] **T2.14** Proper exit code from multiple commands in body

### Variable Interaction
- [ ] **T2.15** Loop body can modify variables in condition
- [ ] **T2.16** Condition re-evaluated with updated variables
- [ ] **T2.17** Variables set in body persist after loop
- [ ] **T2.18** Handle variable expansion in condition
- [ ] **T2.19** Test with counters and state changes

### Phase 2 Tests
- [ ] **T2.20** Integration test: `test_while_with_test_command`
- [ ] **T2.21** Integration test: `test_while_with_pipeline`
- [ ] **T2.22** Integration test: `test_while_with_negation`
- [ ] **T2.23** Integration test: `test_until_with_test_command`
- [ ] **T2.24** Integration test: `test_while_condition_with_variables`
- [ ] **T2.25** Integration test: `test_until_equivalence_to_while_negation`
- [ ] **T2.26** Integration test: `test_while_with_pipes_in_body`
- [ ] **T2.27** Integration test: `test_while_with_variable_updates`
- [ ] **T2.28** Integration test: `test_while_with_command_groups`
- [ ] **T2.29** Integration test: `test_until_with_variable_persistence`

---

## Phase 3: Nesting & Integration (Days 5-6)

### Multiline REPL Support
- [ ] **T3.1** Extend `is_statement_complete()` to detect incomplete while/until
- [ ] **T3.2** Check for matching `while`/`done` and `until`/`done` pairs
- [ ] **T3.3** Track nesting depth for nested structures
- [ ] **T3.4** Show continuation prompt for incomplete loops
- [ ] **T3.5** Accumulate multiline input until `done`

### Nested Structures
- [ ] **T3.6** Support while/until inside conditionals
- [ ] **T3.7** Support conditionals inside while/until body
- [ ] **T3.8** Support while inside for loops
- [ ] **T3.9** Support for loops inside while/until body
- [ ] **T3.10** Support nested while/until loops
- [ ] **T3.11** Proper keyword detection with depth tracking
- [ ] **T3.12** Only match `done` that closes current loop

### Break/Continue Infrastructure (Preparation for Features 022-023)
- [ ] **T3.13** Design signal mechanism for loop control
- [ ] **T3.14** Implement placeholder for break signal handling
- [ ] **T3.15** Implement placeholder for continue signal handling
- [ ] **T3.16** Document integration points for Features 022/023
- [ ] **T3.17** Verify infrastructure can support signal propagation

### Phase 3 Tests
- [ ] **T3.18** Integration test: `test_while_multiline_in_repl`
- [ ] **T3.19** Integration test: `test_while_in_if_statement`
- [ ] **T3.20** Integration test: `test_if_in_while_statement`
- [ ] **T3.21** Integration test: `test_nested_while_loops`
- [ ] **T3.22** Integration test: `test_for_with_while_body`
- [ ] **T3.23** Integration test: `test_while_with_for_body`
- [ ] **T3.24** Integration test: `test_while_with_variable_in_condition`
- [ ] **T3.25** Integration test: `test_until_nested_in_while`
- [ ] **T3.26** Integration test: `test_while_and_until_in_same_script`
- [ ] **T3.27** Integration test: `test_complex_nested_control_flow`

### Edge Cases & Validation
- [ ] **T3.28** Infinite while loop handling (Ctrl+C works)
- [ ] **T3.29** Condition always true (until) or always false (while)
- [ ] **T3.30** Empty loop body execution
- [ ] **T3.31** POSIX compliance verification
- [ ] **T3.32** Code coverage validation (>95%)

### Final Integration
- [ ] **T3.33** Run full test suite with Features 018-019
- [ ] **T3.34** Verify no regressions in existing features
- [ ] **T3.35** Performance verification

---

## Implementation Dependencies

### Required Before Starting
- ✅ Feature 017 (Conditional Control Flow) - parser patterns
- ✅ Feature 018 (For Loops) - loop control infrastructure
- ✅ Feature 001 (Command Execution) - for body execution

### Integration Points for Future Features
- Feature 022 (break) - will hook into loop execution
- Feature 023 (continue) - will hook into loop execution

---

## Success Criteria Checklist

**Phase 1 Complete When**:
- [ ] All parser tests pass (8 tests)
- [ ] Basic while/until loops execute correctly
- [ ] Exit codes match POSIX semantics
- [ ] Code compiles without warnings

**Phase 2 Complete When**:
- [ ] Condition parsing and evaluation tests pass (10 tests)
- [ ] Complex bodies execute correctly
- [ ] Variable interaction works properly
- [ ] All Phase 1 + Phase 2 tests pass (18+ total)

**Phase 3 Complete When**:
- [ ] Multiline REPL support works
- [ ] Nested structures work correctly
- [ ] Break/continue infrastructure designed
- [ ] All 35+ tests pass
- [ ] POSIX compliance verified

---

## Testing Strategy

### Unit Tests (12 tests)
- Parser functionality
- Condition evaluation
- Loop type (while vs until)
- Error conditions

### Integration Tests (23+ tests)
- Basic while/until loops
- Complex conditions
- Complex bodies
- Nested structures
- Variable interactions
- POSIX compliance

---

## Notes

- Reuse condition evaluation from Feature 017 where possible
- Leverage Feature 018's loop control infrastructure
- Document all signal integration points for Features 022/023
- Consider optimization for condition evaluation on each iteration
- Test against reference shells (bash, dash) for POSIX compliance

---

**Created**: 2025-12-06
**Status**: Task Breakdown Complete
**Estimated Total Duration**: 6-8 development days
**Next Phase**: Begin Phase 1 implementation after Feature 018 Phase 1 complete
