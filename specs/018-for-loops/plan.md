# Implementation Plan: Feature 018 - For Loops

**Feature**: 018-for-loops (for/in/do/done)
**Planned Phases**: 4
**Estimated Test Coverage**: 45+ tests
**Architecture Pattern**: Follows Feature 017 (conditionals)

## Phase Architecture

Based on Feature 017 success, for-loop implementation will follow a 4-phase approach:

### Phase 1: Core Parser & Executor (Days 1-2)
**Goal**: Parse and execute basic for loops with word list

**Components**:

#### 1.1 AST Structure (`executor/mod.rs`)
```rust
#[derive(Debug, Clone)]
pub struct ForLoop {
    pub variable: String,           // Loop variable name
    pub word_list: Vec<String>,     // Expanded word list (or expr for dynamic)
    pub body: Vec<Command>,         // Commands to execute in each iteration
}
```

#### 1.2 Parser (`executor/for_loop.rs` - NEW)
- Create new module: `executor/for_loop.rs`
- Implement `parse_for_loop(input: &str) -> Result<ForLoop, RushError>`
- Parsing steps:
  1. Expect `for` keyword
  2. Parse variable name (identifier)
  3. Parse optional `in` keyword
  4. Parse word list (or empty for `$@` positional params)
  5. Expect `do` keyword
  6. Parse command list recursively (body)
  7. Expect `done` keyword
- Error handling for:
  - Missing keywords (for, do, done)
  - Invalid variable names
  - Empty/malformed word list
  - Unclosed loop

#### 1.3 Word List Expansion (`expansion/mod.rs` - REUSE)
- Reuse existing expansion functions:
  - `expand_variables()` for `$VAR`, `${VAR}`
  - `expand_command_substitution()` for `$(cmd)` or `` `cmd` ``
  - IFS-based splitting for word boundaries
- Key difference from conditionals: NO globbing in word list (treat patterns literally)
- Implement word splitting respecting quotes and escapes

#### 1.4 Executor (`executor/execute.rs` - MODIFY)
- Detect `for` keyword in command parsing
- Delegate to `ForLoop::execute()`
- Basic execution:
  ```rust
  impl ForLoop {
      pub fn execute(&self) -> Result<ExitCode, RushError> {
          let expanded_words = expand_word_list(&self.word_list);
          let mut exit_code = 0;

          for word in expanded_words {
              env::set_var(&self.variable, &word);
              exit_code = execute_commands(&self.body)?;
          }
          Ok(exit_code)
      }
  }
  ```

#### 1.5 Tests (Phase 1)
- Unit tests in `tests/unit/parser_for.rs`:
  - test_parse_for_basic
  - test_parse_for_empty_list
  - test_parse_for_missing_do
  - test_parse_for_missing_done
  - test_parse_for_invalid_var_name

- Integration tests in `tests/integration/loops_tests.rs`:
  - test_for_basic_iteration
  - test_for_empty_word_list (no execution)
  - test_for_single_word
  - test_for_exit_code (from last command)

**Success Criteria**:
- Parser accepts all valid for-loop syntax
- Basic loops execute correctly
- Empty lists handled properly (no execution, exit 0)
- Exit codes match POSIX semantics
- All Phase 1 tests pass (8 tests minimum)

---

### Phase 2: Word List Expansion & Variable Binding (Days 3-4)
**Goal**: Full support for dynamic word lists and variable scoping

**Components**:

#### 2.1 Enhanced Word List Expansion
- Variable expansion: `for file in $list *.txt`
  - Expand `$list` to all its words
  - Add literal `*.txt` (NO globbing)
  - Handle empty variables (contribute 0 words)

- Command substitution: `for file in $(find . -name "*.txt")`
  - Execute command
  - Split output on newlines and IFS
  - Handle whitespace properly

- Brace expansion (if implemented): `for i in {1..3}`
  - Expand braces before iteration
  - Each expanded word is separate iteration value

- Quote handling:
  - `for i in "a b" c d` → iterate over 3 values: "a b", "c", "d"
  - No field splitting inside quotes
  - Escapes processed correctly

#### 2.2 Variable Binding & Scoping
- Modify environment handling:
  - Loop variable is bound in current shell scope (not subshell)
  - If variable exists, it's overwritten
  - After loop, variable keeps last assigned value
  - If loop doesn't execute, variable is unchanged

- Implement scoping check:
  - Variables set in loop body are visible after loop (global scope)
  - Proper interaction with subshells (variables in `(...)` don't escape)

#### 2.3 Positional Parameter Handling (US2)
- `for var; do` equivalent to `for var in "$@"; do`
- Access `$@` or positional parameter list when `in` keyword omitted
- Pass positional parameters through word list expansion
- Handle case of no positional parameters (loop doesn't execute)

#### 2.4 Tests (Phase 2)
- Unit tests for word expansion:
  - test_expand_variable_in_list
  - test_expand_command_substitution
  - test_no_globbing_in_word_list
  - test_quote_handling
  - test_empty_expansion

- Integration tests for binding:
  - test_loop_variable_persistence
  - test_variable_shadowing
  - test_positional_parameters_default
  - test_positional_parameters_multiple
  - test_variable_not_modified_if_loop_empty

**Success Criteria**:
- Word lists with variable expansion work correctly
- Command substitution in word lists works
- NO globbing occurs in word lists
- Loop variable persists after loop
- Positional parameter syntax works
- 10+ new tests pass (Phase 1 + Phase 2 = 18+ total)

---

### Phase 3: Complex Loop Bodies & Nesting (Days 5-7)
**Goal**: Support realistic use cases with nested structures and complex commands

**Components**:

#### 3.1 Multiline Loop Support in REPL
- Extend REPL multiline detection (`repl/mod.rs`):
  - Detect incomplete for loops (missing `done`)
  - Show continuation prompt (">\> ") on subsequent lines
  - Accumulate lines until `done` keyword
  - Properly handle nested structures (for inside if, etc.)

#### 3.2 Complex Loop Bodies
- Support command sequences:
  - Semicolon-separated commands: `for i in 1 2 3; do cmd1; cmd2; done`
  - Piped commands: `for file in *.txt; do cat "$file" | grep pattern; done`
  - Redirections: `for i in 1 2; do echo $i >> output.txt; done`
  - Command groups: `for i in 1 2; do { cmd1; cmd2; }; done`
  - Subshells: `for i in 1 2; do (cd /tmp && pwd); done`

- Exit code semantics:
  - If body is pipeline, exit code from last command
  - If body is multiple commands, exit code from last command
  - Proper handling of redirections

#### 3.3 Nested Structures Support
- For loops inside conditionals:
  ```bash
  if [ $count -gt 0 ]; then
    for item in $list; do
      process "$item"
    done
  fi
  ```

- Conditionals inside for loops:
  ```bash
  for file in *.txt; do
    if [ -s "$file" ]; then
      process "$file"
    fi
  done
  ```

- Nested for loops:
  ```bash
  for i in 1 2 3; do
    for j in a b c; do
      echo "$i-$j"
    done
  done
  ```

- Proper keyword detection with depth tracking
  - Track nesting depth of if/for/while structures
  - Only match `done` that closes current for loop
  - Similar to Feature 017's nested conditional handling

#### 3.4 Tests (Phase 3)
- Integration tests for complex bodies:
  - test_for_with_pipes
  - test_for_with_redirections
  - test_for_with_command_groups
  - test_for_with_subshells
  - test_for_in_if_statement
  - test_if_in_for_statement
  - test_nested_for_loops
  - test_for_with_variable_in_condition
  - test_exit_code_from_pipeline_in_loop

**Success Criteria**:
- Multiline for loops work in REPL
- Pipes and redirections in loop body work
- Nested conditionals in loop body work
- Nested for loops work
- Exit codes are correct for complex bodies
- 15+ new tests pass (cumulative: 33+ total)

---

### Phase 4: Edge Cases & Optimization (Days 8-9)
**Goal**: Handle edge cases, optimize performance, ensure POSIX compliance

**Components**:

#### 4.1 Edge Case Handling
- Special characters in words:
  - Words with spaces: `for w in "hello world" foo`
  - Words with special chars: `for f in "file$1.txt" /path/*/file`
  - Words with newlines from command substitution

- IFS handling:
  - Respect IFS for word splitting (default: space/tab/newline)
  - Handle custom IFS values
  - Interaction with quotes and escapes

- Large word lists:
  - 100+ iterations should complete quickly
  - No memory leaks or excessive allocations
  - Proper cleanup after loop

- Error conditions:
  - Invalid variable names (should reject at parse time)
  - Command failures in loop body (exit code from failed command)
  - Signals during loop execution (SIGINT should stop loop)

#### 4.2 POSIX Compliance Verification
- Test against reference shell (bash, dash):
  - Compare exit codes
  - Compare variable persistence
  - Compare word expansion behavior
  - Verify edge case handling matches reference

#### 4.3 Performance Optimization
- Profile loop execution with 100+ iterations
- Optimize hot paths:
  - Word list expansion (cache results if no dynamic content)
  - Variable binding (efficient environment updates)
  - Command execution (reuse existing executor)

#### 4.4 Tests (Phase 4)
- Edge case tests:
  - test_for_words_with_spaces
  - test_for_words_with_special_chars
  - test_for_with_newlines_in_words
  - test_for_large_word_list (100+ items)
  - test_for_with_signals
  - test_for_command_failure_exit_code
  - test_custom_ifs_handling

- Compliance tests:
  - test_posix_compliance_vs_bash
  - test_posix_compliance_variable_scope
  - test_posix_compliance_word_expansion

- Performance tests:
  - test_loop_1000_iterations_performance

**Success Criteria**:
- All 45+ tests pass
- POSIX compliance verified
- Performance acceptable (<100ms for 100 iterations)
- No memory leaks detected
- Code coverage >95% for for-loop implementation
- Ready for production use

---

## Implementation Order

1. **Phase 1** (Days 1-2):
   - Create `executor/for_loop.rs` module
   - Implement `ForLoop` struct and parser
   - Add to `executor/mod.rs` and `executor/execute.rs`
   - Write 8 initial tests
   - Verify basic functionality

2. **Phase 2** (Days 3-4):
   - Enhance word list expansion in parser
   - Add variable binding and scoping
   - Implement positional parameter handling
   - Write 10 new tests
   - Verify expansion and binding work

3. **Phase 3** (Days 5-7):
   - Add REPL multiline support
   - Support complex loop bodies
   - Implement nested structure handling
   - Write 15 new tests
   - Verify complex use cases work

4. **Phase 4** (Days 8-9):
   - Handle edge cases
   - Verify POSIX compliance
   - Optimize performance
   - Write 12 additional tests
   - Final validation and cleanup

---

## File Modifications Summary

### New Files
- `crates/rush/src/executor/for_loop.rs` - For loop parser and executor
- `crates/rush/tests/unit/parser_for.rs` - Unit tests
- `crates/rush/tests/integration/loops_tests.rs` - Integration tests (or add to existing)

### Modified Files
- `crates/rush/src/executor/mod.rs` - Add `ForLoop` struct
- `crates/rush/src/executor/execute.rs` - Add for loop detection and dispatch
- `crates/rush/src/repl/mod.rs` - Add multiline support for for loops
- `crates/rush/src/repl/prompt.rs` - Continuation prompt for loops
- `crates/rush/tests/unit_tests.rs` - Add for loop test module

### Unchanged (Reuse)
- `crates/rush/src/expansion/mod.rs` - Reuse variable/command substitution
- `crates/rush/src/executor/execute.rs` - Reuse command execution

---

## Testing Strategy

### Unit Tests (15 tests)
- Parser validation
- Word list expansion
- Variable binding
- Error conditions

### Integration Tests (30 tests)
- Basic for loops
- Complex bodies
- Nested structures
- Edge cases
- POSIX compliance
- Performance

### Coverage Target
- >95% code coverage for for-loop implementation
- All user stories covered by acceptance tests
- All edge cases from spec covered

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Complex nested keyword detection | Follow Feature 017 pattern with depth tracking |
| Word list expansion complexity | Reuse existing expansion functions |
| Variable scoping issues | Clear scoping rules, comprehensive tests |
| POSIX compliance divergence | Test against reference shells |
| Performance degradation | Profile hot paths, optimize as needed |

---

## Dependencies & Blockers

- ✅ Feature 001 (Command execution) - Fully implemented
- ✅ Feature 017 (Conditional control flow) - Implemented
- ❌ Feature 022 (break statement) - Blocks break support in loops
- ❌ Feature 023 (continue statement) - Blocks continue support in loops
- ✅ Expansion utilities - Already available

**Proceed**: Feature 018 can be implemented independently and later integrated with 022/023

---

## Rollout Plan

1. **Initial Implementation** (Phase 1-2)
   - Merge when basic for loops work
   - Focus: Core parsing, execution, word expansion
   - PR: Feature 018 Part 1

2. **Enhancement** (Phase 3)
   - Merge when complex bodies work
   - Focus: Multiline, nesting, complex commands
   - PR: Feature 018 Part 2

3. **Finalization** (Phase 4)
   - Merge when edge cases handled
   - Focus: POSIX compliance, performance
   - PR: Feature 018 Part 3

---

**Created**: 2025-12-06
**Last Updated**: 2025-12-06
**Phase**: Planning Complete
**Next**: Task Generation (create tasks.md)
