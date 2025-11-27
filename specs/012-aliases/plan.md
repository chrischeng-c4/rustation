# Implementation Plan: Command Aliases

## Overview
Implement command aliases using `alias` and `unalias` builtins, with automatic alias expansion before command execution.

## Architecture

### Components
1. **AliasManager** - Stores and manages aliases
2. **alias builtin** - Define and list aliases
3. **unalias builtin** - Remove aliases
4. **Expansion** - Replace alias names with values before execution

### Data Flow
```
User input → Alias expansion → Parse → Execute
```

## Implementation Steps

### Step 1: Create AliasManager
**File:** `crates/rush/src/executor/alias.rs` (new file)

```rust
use std::collections::HashMap;

/// Manages command aliases
pub struct AliasManager {
    aliases: HashMap<String, String>,
}

impl AliasManager {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }

    /// Add or update an alias
    pub fn set(&mut self, name: String, value: String) -> Result<()> {
        // Validate name (alphanumeric + underscore)
        if !Self::is_valid_name(&name) {
            return Err(RushError::Execution(format!(
                "alias: invalid name: {}", name
            )));
        }
        self.aliases.insert(name, value);
        Ok(())
    }

    /// Get alias value
    pub fn get(&self, name: &str) -> Option<&str> {
        self.aliases.get(name).map(|s| s.as_str())
    }

    /// Remove an alias
    pub fn remove(&mut self, name: &str) -> bool {
        self.aliases.remove(name).is_some()
    }

    /// List all aliases (sorted)
    pub fn list(&self) -> Vec<(&str, &str)> {
        let mut aliases: Vec<_> = self.aliases.iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        aliases.sort_by_key(|&(name, _)| name);
        aliases
    }

    /// Check if name is valid
    fn is_valid_name(name: &str) -> bool {
        !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }
}
```

### Step 2: Add AliasManager to CommandExecutor
**File:** `crates/rush/src/executor/execute.rs`

```rust
pub struct CommandExecutor {
    pipeline_executor: PipelineExecutor,
    job_manager: JobManager,
    alias_manager: AliasManager,  // Add this
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            pipeline_executor: PipelineExecutor::new(),
            job_manager: JobManager::new(),
            alias_manager: AliasManager::new(),  // Add this
        }
    }

    /// Get mutable reference to alias manager
    pub fn alias_manager_mut(&mut self) -> &mut AliasManager {
        &mut self.alias_manager
    }

    /// Get immutable reference to alias manager
    pub fn alias_manager(&self) -> &AliasManager {
        &self.alias_manager
    }
}
```

### Step 3: Implement alias Builtin
**File:** `crates/rush/src/executor/builtins/alias.rs` (new file)

```rust
use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Implement the `alias` builtin command
///
/// Usage:
/// - alias           : List all aliases
/// - alias name=value: Define alias
pub fn alias(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        // List all aliases
        let aliases = executor.alias_manager().list();
        for (name, value) in aliases {
            println!("alias {}='{}'", name, value);
        }
        return Ok(0);
    }

    // Define alias: alias name=value
    for arg in args {
        if let Some(pos) = arg.find('=') {
            let name = &arg[..pos];
            let value = &arg[pos + 1..];

            // Remove surrounding quotes if present
            let value = value.trim_matches(|c| c == '\'' || c == '"');

            executor.alias_manager_mut().set(
                name.to_string(),
                value.to_string(),
            )?;
        } else {
            // Just name without =, show that specific alias
            if let Some(value) = executor.alias_manager().get(arg) {
                println!("alias {}='{}'", arg, value);
            } else {
                eprintln!("rush: alias: {}: not found", arg);
                return Ok(1);
            }
        }
    }

    Ok(0)
}
```

### Step 4: Implement unalias Builtin
**File:** `crates/rush/src/executor/builtins/unalias.rs` (new file)

```rust
use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Implement the `unalias` builtin command
///
/// Usage:
/// - unalias name     : Remove alias
/// - unalias name1 name2: Remove multiple aliases
pub fn unalias(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("rush: unalias: usage: unalias name [name ...]");
        return Ok(1);
    }

    let mut exit_code = 0;

    for name in args {
        if !executor.alias_manager_mut().remove(name) {
            eprintln!("rush: unalias: {}: not found", name);
            exit_code = 1;
        }
    }

    Ok(exit_code)
}
```

### Step 5: Register Builtins
**File:** `crates/rush/src/executor/builtins/mod.rs`

```rust
mod alias;
mod unalias;

pub fn execute_builtin(
    executor: &mut CommandExecutor,
    command: &str,
    args: &[String],
) -> Option<Result<i32>> {
    match command {
        // ... existing builtins
        "alias" => Some(alias::alias(executor, args)),
        "unalias" => Some(unalias::unalias(executor, args)),
        _ => None,
    }
}
```

### Step 6: Implement Alias Expansion
**File:** `crates/rush/src/executor/execute.rs`

Add alias expansion in `execute()` method before parsing:

```rust
pub fn execute(&mut self, line: &str) -> Result<i32> {
    // Handle empty input
    if line.trim().is_empty() {
        return Ok(0);
    }

    // Expand aliases FIRST (before parsing)
    let line = self.expand_aliases(line);

    // Parse command line
    let mut pipeline = parse_pipeline(&line)?;

    // ... rest of execution
}

/// Expand aliases in command line
/// Only expands the first word
fn expand_aliases(&self, line: &str) -> String {
    let trimmed = line.trim_start();

    // Find first word (up to whitespace or special char)
    let first_word_end = trimmed
        .find(|c: char| c.is_whitespace() || "| >&<;".contains(c))
        .unwrap_or(trimmed.len());

    let first_word = &trimmed[..first_word_end];
    let rest = &trimmed[first_word_end..];

    // Check if first word is an alias
    if let Some(alias_value) = self.alias_manager.get(first_word) {
        // Replace first word with alias value
        format!("{}{}", alias_value, rest)
    } else {
        line.to_string()
    }
}
```

## Testing Plan

### Unit Tests
**File:** `crates/rush/src/executor/alias.rs`

1. `test_alias_manager_new()`
2. `test_set_alias()`
3. `test_get_alias()`
4. `test_remove_alias()`
5. `test_list_aliases_sorted()`
6. `test_invalid_alias_name()`
7. `test_update_existing_alias()`

**File:** `crates/rush/src/executor/builtins/alias.rs`

1. `test_alias_list_empty()`
2. `test_alias_define()`
3. `test_alias_define_with_quotes()`
4. `test_alias_show_specific()`

**File:** `crates/rush/src/executor/builtins/unalias.rs`

1. `test_unalias_existing()`
2. `test_unalias_nonexistent()`
3. `test_unalias_multiple()`
4. `test_unalias_no_args()`

### Integration Tests
**File:** `crates/rush/tests/feature_test.rs`

1. `test_alias_simple()`
   ```rust
   executor.execute("alias ll='ls -la'").unwrap();
   let result = executor.execute("ll");
   // Should execute ls -la
   ```

2. `test_alias_with_arguments()`
   ```rust
   executor.execute("alias ll='ls -la'").unwrap();
   let result = executor.execute("ll /tmp");
   // Should execute ls -la /tmp
   ```

3. `test_alias_list()`
   ```rust
   executor.execute("alias a='echo a'").unwrap();
   executor.execute("alias b='echo b'").unwrap();
   let result = executor.execute("alias");
   // Should list both aliases
   ```

4. `test_unalias()`
   ```rust
   executor.execute("alias ll='ls -la'").unwrap();
   executor.execute("unalias ll").unwrap();
   let result = executor.execute("ll");
   // Should fail - command not found
   ```

5. `test_alias_with_pipe()`
   ```rust
   executor.execute("alias lsg='ls | grep'").unwrap();
   let result = executor.execute("lsg txt");
   // Should execute ls | grep txt
   ```

6. `test_alias_no_recursive_expansion()`
   ```rust
   executor.execute("alias echo='echo HELLO'").unwrap();
   let result = executor.execute("echo world");
   // Should execute: echo HELLO world (not infinitely recursive)
   ```

## Edge Cases

1. **Alias to itself:**
   ```bash
   $ alias ls='ls -la'
   $ ls
   # Should execute: ls -la (not recurse)
   ```

2. **Empty alias value:**
   ```bash
   $ alias empty=''
   $ empty
   # Should do nothing (empty command)
   ```

3. **Alias with quotes:**
   ```bash
   $ alias greet='echo "Hello World"'
   $ greet
   Hello World
   ```

4. **Alias name conflicts with builtin:**
   ```bash
   $ alias cd='echo no cd for you'
   $ cd /tmp
   no cd for you /tmp
   # Aliases have priority over builtins
   ```

## Files to Create/Modify

### New Files
- `crates/rush/src/executor/alias.rs` - AliasManager implementation
- `crates/rush/src/executor/builtins/alias.rs` - alias builtin
- `crates/rush/src/executor/builtins/unalias.rs` - unalias builtin

### Modified Files
- `crates/rush/src/executor/mod.rs` - Export alias module
- `crates/rush/src/executor/execute.rs` - Add AliasManager, implement expansion
- `crates/rush/src/executor/builtins/mod.rs` - Register new builtins
- `crates/rush/tests/feature_test.rs` - Integration tests

## Implementation Order

1. Create `alias.rs` with AliasManager
2. Add unit tests for AliasManager
3. Add AliasManager to CommandExecutor
4. Implement `alias.rs` builtin
5. Implement `unalias.rs` builtin
6. Register builtins in mod.rs
7. Implement alias expansion in execute.rs
8. Add integration tests
9. Test edge cases

## Notes

- Aliases are session-only (not persisted)
- Only first word is checked for aliases
- No recursive expansion (prevents infinite loops)
- Aliases have priority over builtins
- Case-sensitive alias names
