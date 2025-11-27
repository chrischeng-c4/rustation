# Implementation Plan: Command Substitution

## Overview
Implement `$(command)` syntax for command substitution in rush shell. This involves parsing command substitution patterns, executing commands, capturing output, and substituting results back into the command line.

## Architecture

### Phase 1: Parsing
Command substitution needs to be detected and extracted during tokenization, before variable expansion and glob expansion.

**Order of Operations:**
1. Parse command line → identify `$(...)` patterns
2. Extract and recursively parse nested substitutions
3. Execute substitutions (inner-most first)
4. Replace `$(...)` with command output
5. Continue with normal parsing (variables, globs, etc.)

### Phase 2: Execution
Each `$(...)` is executed as a separate command, output captured, and substituted into the original command line.

## Implementation Steps

### Step 1: Add Command Substitution Detection
**File:** `crates/rush/src/executor/parser.rs`

Add function to detect and extract command substitutions:
```rust
/// Detect if a string contains command substitution patterns
fn contains_command_substitution(s: &str) -> bool

/// Extract all command substitutions from a string
/// Returns Vec of (start_pos, end_pos, command_string)
fn extract_command_substitutions(s: &str) -> Result<Vec<(usize, usize, String)>>
```

**Logic:**
- Scan for `$(` pattern
- Track parenthesis depth to handle nesting
- Respect quote context (don't substitute in single quotes)
- Handle escaping: `\$(` is literal
- Return error for unclosed `$(`

### Step 2: Add Substitution Execution
**File:** `crates/rush/src/executor/substitution.rs` (new file)

Create new module for command substitution:
```rust
pub struct SubstitutionExecutor;

impl SubstitutionExecutor {
    /// Execute a command and capture its stdout
    pub fn execute_substitution(command: &str, env_map: &HashMap<String, String>) -> Result<String>

    /// Process a string containing command substitutions
    /// Recursively handles nested substitutions
    pub fn expand_substitutions(input: &str, env_map: &HashMap<String, String>) -> Result<String>
}
```

**Execution Logic:**
1. Parse inner command using `parse_pipeline()`
2. Execute command with `PipelineExecutor`
3. Capture stdout using `Stdio::piped()`
4. Trim trailing newlines (POSIX behavior)
5. Return captured output

### Step 3: Integrate into Command Execution
**File:** `crates/rush/src/executor/execute.rs`

Add substitution expansion before variable expansion:
```rust
pub fn execute(&mut self, line: &str) -> Result<i32> {
    // 1. Expand command substitutions FIRST
    let line = expand_substitutions(line, &self.env_manager.as_env_map())?;

    // 2. Parse pipeline
    let mut pipeline = parse_pipeline(&line)?;

    // 3. Expand variables
    expand_variables(&mut pipeline.segments, &self.env_manager);

    // 4. Expand globs
    expand_globs(&mut pipeline.segments);

    // ... rest of execution
}
```

### Step 4: Handle Nested Substitutions
**File:** `crates/rush/src/executor/substitution.rs`

Implement recursive substitution:
```rust
pub fn expand_substitutions(input: &str, env_map: &HashMap<String, String>) -> Result<String> {
    let substitutions = extract_command_substitutions(input)?;

    if substitutions.is_empty() {
        return Ok(input.to_string());
    }

    let mut result = input.to_string();

    // Process from innermost to outermost
    for (start, end, command) in substitutions.iter().rev() {
        // Recursively expand nested substitutions in command
        let expanded_command = expand_substitutions(command, env_map)?;

        // Execute the command
        let output = execute_substitution(&expanded_command, env_map)?;

        // Replace $(command) with output
        result.replace_range(start..=end, &output);
    }

    Ok(result)
}
```

### Step 5: Quote Handling
Command substitutions inside double quotes preserve newlines:
```rust
fn process_substitution_output(output: &str, in_quotes: bool) -> String {
    if in_quotes {
        // Preserve newlines in quotes
        output.trim_end_matches('\n').to_string()
    } else {
        // Convert newlines to spaces outside quotes
        output.trim_end_matches('\n').split('\n').collect::<Vec<_>>().join(" ")
    }
}
```

## Testing Plan

### Unit Tests
**File:** `crates/rush/src/executor/substitution.rs` (tests module)

1. `test_contains_command_substitution()`
   - Detect `$(...)` patterns
   - Ignore escaped `\$(`
   - Ignore in single quotes

2. `test_extract_simple_substitution()`
   - Extract `$(echo hello)`
   - Get correct start/end positions

3. `test_extract_nested_substitution()`
   - Extract `$(echo $(whoami))`
   - Correctly identify inner and outer substitutions

4. `test_unclosed_substitution_error()`
   - Error on `$(echo hello`
   - Error on `$(echo $(whoami)`

5. `test_execute_substitution()`
   - Execute simple command
   - Capture output correctly
   - Trim trailing newlines

6. `test_expand_substitutions_simple()`
   - Replace `$(echo hello)` with `hello`

7. `test_expand_substitutions_nested()`
   - Handle `$(echo $(whoami))` correctly

### Integration Tests
**File:** `crates/rush/tests/feature_test.rs`

1. `test_command_substitution_basic()`
   ```rust
   let result = executor.execute("echo $(echo hello)");
   // Should execute echo with arg "hello"
   ```

2. `test_command_substitution_in_quotes()`
   ```rust
   let result = executor.execute("echo \"Today is $(date)\"");
   // Should work inside quotes
   ```

3. `test_command_substitution_multiple()`
   ```rust
   let result = executor.execute("echo $(whoami) $(echo rush)");
   // Multiple substitutions
   ```

4. `test_command_substitution_nested()`
   ```rust
   let result = executor.execute("echo $(echo $(whoami))");
   // Nested substitutions
   ```

5. `test_command_substitution_with_args()`
   ```rust
   let result = executor.execute("ls $(echo \"-la\")");
   // Substitution as arguments
   ```

6. `test_command_substitution_in_variable()`
   ```rust
   executor.execute("export TODAY=$(date)");
   // Check variable value contains date output
   ```

## Error Handling

1. **Unclosed substitution:**
   ```
   $ echo $(date
   rush: parse error: Unclosed command substitution
   ```

2. **Command execution failure:**
   - Propagate error from inner command
   - Don't substitute if command fails

3. **Empty substitution:**
   ```
   $ echo $()
   # Results in "echo" with no arguments
   ```

## Edge Cases

1. **Escaped dollar:**
   ```
   $ echo \$(date)
   $(date)  # Literal output
   ```

2. **Single quotes:**
   ```
   $ echo '$(date)'
   $(date)  # Not substituted
   ```

3. **Whitespace handling:**
   ```
   $ echo $(echo "hello   world")
   hello   world  # Preserves spaces in quotes

   $ echo $(printf "line1\nline2")
   line1 line2  # Newlines become spaces
   ```

4. **Parentheses in strings:**
   ```
   $ echo $(echo "()")
   ()  # Literal parens in output
   ```

## Files to Create/Modify

### New Files
- `crates/rush/src/executor/substitution.rs` - Command substitution logic

### Modified Files
- `crates/rush/src/executor/mod.rs` - Export substitution module
- `crates/rush/src/executor/execute.rs` - Integrate substitution expansion
- `crates/rush/src/executor/parser.rs` - May need quote tracking helpers
- `crates/rush/tests/feature_test.rs` - Integration tests

## Implementation Order

1. Create `substitution.rs` with parsing functions
2. Add unit tests for parsing
3. Add execution logic to `substitution.rs`
4. Add unit tests for execution
5. Integrate into `execute.rs`
6. Add integration tests
7. Test nested substitutions
8. Handle edge cases and error scenarios

## Performance Considerations

- Each `$(...)` spawns a new process
- Nested substitutions create multiple processes
- Consider caching if same substitution appears multiple times (future optimization)

## Compatibility Notes

- POSIX shells: Trailing newlines are trimmed
- Bash behavior: Internal newlines become spaces when unquoted
- We're NOT implementing:
  - Backtick syntax `` `...` ``
  - Arithmetic expansion `$((...))`
  - Process substitution `<(...)` / `>(...)`
