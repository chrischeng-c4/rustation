# Implementation Plan: Feature 019 - While/Until Loops

**Feature**: 019-while-until-loops
**Planned Phases**: 3
**Estimated Test Coverage**: 35+ tests
**Architecture Pattern**: Follows Features 017-018 (conditionals and for loops)

## Phase Architecture

Based on successful patterns from Features 017-018, while/until implementation will use a 3-phase approach:

### Phase 1: Core Parser & Executor (Days 1-2)
**Goal**: Parse and execute basic while and until loops

**Components**:

#### 1.1 AST Structure (`executor/mod.rs`)
```rust
#[derive(Debug, Clone)]
pub enum LoopType {
    While,  // Continue while condition is true (exit code 0)
    Until,  // Continue until condition is true (exit code 0 = exit)
}

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub loop_type: LoopType,
    pub condition: Box<Command>,  // Condition evaluated before each iteration
    pub body: Vec<Command>,        // Loop body commands
}
```

#### 1.2 Parser (`executor/while_loop.rs` - NEW)
- Create new module: `executor/while_loop.rs`
- Implement `parse_while_loop(input: &str, loop_type: LoopType) -> Result<WhileLoop, RushError>`
- Parsing steps for while:
  1. Expect `while` keyword
  2. Parse condition (any command sequence until `do`)
  3. Expect `do` keyword
  4. Parse command list recursively (body)
  5. Expect `done` keyword

- Parsing steps for until:
  1. Expect `until` keyword
  2. Parse condition (any command sequence until `do`)
  3. Expect `do` keyword
  4. Parse command list recursively (body)
  5. Expect `done` keyword

- Error handling for:
  - Missing keywords (while/until, do, done)
  - Empty/malformed condition
  - Unclosed loop

#### 1.3 Condition Evaluation
- Reuse existing command execution infrastructure
- Execute condition as a command
- Capture exit code:
  - while: exit code 0 = true (continue), non-zero = false (exit)
  - until: exit code 0 = true (exit), non-zero = false (continue)

#### 1.4 Executor (`executor/execute.rs` - MODIFY)
- Detect `while` and `until` keywords in command parsing
- Delegate to `WhileLoop::execute()`
- Basic execution:
  ```rust
  impl WhileLoop {
      pub fn execute(&self) -> Result<ExitCode, RushError> {
          let mut exit_code = 0;
          let mut iteration = 0;

          loop {
              let cond_exit = execute_command(&self.condition)?;
              let should_continue = match self.loop_type {
                  LoopType::While => cond_exit == 0,
                  LoopType::Until => cond_exit != 0,
              };

              if !should_continue {
                  break;
              }

              exit_code = execute_commands(&self.body)?;
              iteration += 1;
          }

          Ok(exit_code)
      }
  }
  ```

#### 1.5 Tests (Phase 1)
- Unit tests in `tests/unit/parser_while.rs`:
  - test_parse_while_basic
  - test_parse_until_basic
  - test_parse_while_missing_do
  - test_parse_while_missing_done
  - test_parse_until_false_condition

- Integration tests in `tests/integration/loops_tests.rs`:
  - test_while_basic_iteration
  - test_while_false_condition
  - test_until_true_condition
  - test_while_exit_code
  - test_until_exit_code

**Success Criteria**:
- Parser accepts while and until syntax
- Basic loops execute correctly
- Exit codes match POSIX semantics
- Condition evaluation works properly
- All Phase 1 tests pass (8+ tests)

---

### Phase 2: Complex Conditions & Loop Bodies (Days 3-4)
**Goal**: Support realistic use cases with complex conditions and command sequences

**Components**:

#### 2.1 Complex Condition Parsing
- Support various condition formats:
  - Test commands: `while [ -f file ]; do`
  - Simple commands: `while grep pattern file; do`
  - Pipelines: `while command1 | command2; do`
  - Command groups: `while { test -f "$file"; }; do`
  - Negated commands: `while ! command; do`
  - Compound conditions with && and ||: `while cmd1 && cmd2; do`

- Proper parsing to find `do` keyword boundary
  - Handle nested structures in condition
  - Respect quotes and escapes
  - Distinguish condition from loop body

#### 2.2 Complex Loop Bodies
- Support command sequences:
  - Semicolon-separated commands
  - Piped commands
  - Redirections
  - Command groups
  - Subshells

- Exit code semantics:
  - If body is pipeline, exit code from last command
  - If body is multiple commands, exit code from last command
  - Proper handling of redirections

#### 2.3 Variable Interaction
- Loop body can modify variables used in condition:
  ```bash
  count=0
  while [ $count -lt 3 ]; do
    echo $count
    count=$((count + 1))
  done
  ```

- Proper re-evaluation of condition with updated variables
- Variables set in loop body persist after loop

#### 2.4 Equivalence Testing
- Verify `until cond; do X; done` behaves same as `while ! cond; do X; done`
- Test with various condition types

#### 2.5 Tests (Phase 2)
- Condition parsing and evaluation:
  - test_while_with_test_command
  - test_while_with_pipeline
  - test_while_with_negation
  - test_until_with_test_command
  - test_while_condition_with_variables
  - test_until_equivalence_to_while_negation

- Complex bodies:
  - test_while_with_pipes
  - test_while_with_variable_updates
  - test_while_with_command_groups

**Success Criteria**:
- Complex conditions parse and execute correctly
- Complex loop bodies work
- Variable updates in conditions work
- Until/while equivalence verified
- 12+ new tests pass (cumulative: 20+ total)

---

### Phase 3: Nesting & Integration (Days 5-6)
**Goal**: Support realistic patterns with nested structures and multiline REPL input

**Components**:

#### 3.1 Multiline Loop Support in REPL
- Extend REPL multiline detection (`repl/mod.rs`):
  - Detect incomplete while/until loops (missing `done`)
  - Show continuation prompt on subsequent lines
  - Accumulate lines until `done` keyword
  - Handle nested structures (while inside if, etc.)

#### 3.2 Nested Structures Support
- while/until loops inside conditionals:
  ```bash
  if [ $count -gt 0 ]; then
    while [ $count -gt 0 ]; do
      count=$((count - 1))
    done
  fi
  ```

- Conditionals inside while/until:
  ```bash
  while [ $count -lt 10 ]; do
    if [ $((count % 2)) -eq 0 ]; then
      echo "Even: $count"
    fi
    count=$((count + 1))
  done
  ```

- Nested while/until:
  ```bash
  i=0
  while [ $i -lt 3 ]; do
    j=0
    while [ $j -lt 2 ]; do
      echo "$i-$j"
      j=$((j + 1))
    done
    i=$((i + 1))
  done
  ```

- for loops with while/until:
  ```bash
  for item in a b c; do
    while [ condition ]; do
      process "$item"
    done
  done
  ```

- Proper keyword detection with depth tracking
  - Similar to Feature 017's nested conditional handling
  - Only match `done` that closes current loop

#### 3.3 Break/Continue Infrastructure (Feature 022/023 Preparation)
- Define signal mechanism for loop control:
  - `LoopSignal::Break` - exit loop immediately
  - `LoopSignal::Continue` - skip to next iteration
  - Propagate signals through nested loops

- Document integration points for Features 022/023:
  - Where break/continue commands will be parsed
  - How signals will be caught and handled in loop execution

#### 3.4 Tests (Phase 3)
- Multiline and nesting:
  - test_while_multiline_in_repl
  - test_while_in_if_statement
  - test_if_in_while_statement
  - test_nested_while_loops
  - test_for_with_while_body
  - test_while_with_for_body
  - test_while_with_variable_in_condition

- Integration:
  - test_while_and_until_in_same_script
  - test_complex_nested_control_flow

**Success Criteria**:
- Multiline while/until loops work in REPL
- Nested structures work correctly
- All 35+ tests pass
- Break/continue integration plan documented
- Ready for Features 022/023 integration

---

## Implementation Order

1. **Phase 1** (Days 1-2):
   - Create `executor/while_loop.rs` module
   - Implement `WhileLoop` struct and parser
   - Add while/until detection to `executor/execute.rs`
   - Write 8 initial tests
   - Verify basic functionality

2. **Phase 2** (Days 3-4):
   - Enhance condition parsing for complex cases
   - Support complex loop bodies
   - Implement variable interaction
   - Write 12 new tests
   - Verify conditions and bodies work

3. **Phase 3** (Days 5-6):
   - Add REPL multiline support
   - Implement nested structures
   - Document break/continue integration
   - Write 15 new tests
   - Verify nesting and REPL work

---

## File Modifications Summary

### New Files
- `crates/rush/src/executor/while_loop.rs` - while/until parser and executor
- `crates/rush/tests/unit/parser_while.rs` - Unit tests

### Modified Files
- `crates/rush/src/executor/mod.rs` - Add `WhileLoop` and `LoopType` structs
- `crates/rush/src/executor/execute.rs` - Add while/until detection and dispatch
- `crates/rush/src/repl/mod.rs` - Add multiline support for while/until
- `crates/rush/tests/unit_tests.rs` - Add while loop test module

### Unchanged (Reuse)
- Command execution infrastructure from Features 001, 017, 018

---

## Testing Strategy

### Unit Tests (10 tests)
- Parser validation
- Condition parsing
- Basic execution
- Error conditions

### Integration Tests (25+ tests)
- Basic while/until loops
- Complex conditions
- Complex bodies
- Nested structures
- Variable interactions
- POSIX compliance

### Coverage Target
- >95% code coverage for while/until implementation
- All user stories covered by acceptance tests
- All edge cases from spec covered

---

## Integration with Break/Continue (Features 022/023)

### Design for Future Integration

When Features 022/023 implement break and continue, the following changes will be minimal:

1. **Parser**: Recognize `break` and `continue` keywords
2. **Executor**: Handle loop signals:
   ```rust
   pub enum LoopSignal {
       None,
       Break,
       Continue,
   }

   impl WhileLoop {
       pub fn execute(&self) -> Result<ExitCode, RushError> {
           loop {
               let should_continue = /* ... */;
               if !should_continue { break; }

               let signal = execute_commands(&self.body)?;
               match signal {
                   LoopSignal::Break => break,
                   LoopSignal::Continue => continue,
                   LoopSignal::None => {},
               }
           }
           Ok(exit_code)
       }
   }
   ```

3. **For loops**: Apply same signal handling pattern
4. **Nesting**: Propagate signals to correct loop level

---

## Dependencies & Blockers

- ✅ Feature 001 (Command execution) - Fully implemented
- ✅ Feature 017 (Conditional control flow) - Implemented
- ✅ Feature 018 (For loops) - Ready for implementation
- ❌ Feature 022 (break statement) - Blocks break support in loops
- ❌ Feature 023 (continue statement) - Blocks continue support in loops

**Proceed**: Feature 019 can be fully implemented independently

---

## Performance Considerations

- Condition evaluation on every iteration
  - Don't cache condition result (variables may change)
  - Profile condition evaluation overhead
  - Optimize hot paths if needed

- Loop body execution
  - Reuse existing command execution (already optimized)
  - No additional overhead vs. sequential commands

- Target: 100+ iterations complete in <100ms

---

## Rollout Plan

1. **Initial Implementation** (Phase 1-2)
   - Merge when basic while/until work
   - Focus: Core parsing, execution, conditions
   - PR: Feature 019 Part 1

2. **Enhancement** (Phase 3)
   - Merge when nesting works
   - Focus: Multiline, nesting, complex scenarios
   - PR: Feature 019 Part 2

---

**Created**: 2025-12-06
**Last Updated**: 2025-12-06
**Phase**: Planning Complete
**Next**: Task Generation (create tasks.md)
