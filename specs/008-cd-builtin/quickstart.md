# Quickstart: cd Builtin Command

**Feature**: 008-cd-builtin
**Date**: 2025-11-27

## Overview

This guide covers implementing the `cd` builtin command for the rush shell.

## Prerequisites

- Rust 1.75+ installed
- rush codebase cloned
- Familiarity with existing builtins (export, set, jobs, fg, bg)

## Quick Implementation

### Step 1: Create cd.rs

```bash
touch crates/rush/src/executor/builtins/cd.rs
```

### Step 2: Basic Structure

```rust
//! 'cd' built-in command
//!
//! Changes the current working directory.

use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;
use std::env;
use std::path::Path;

pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Get target directory
    let target = match args.first() {
        None | Some(s) if s.is_empty() => {
            // cd with no args → HOME
            get_home(executor)?
        }
        Some(s) if s == "-" => {
            // cd - → OLDPWD
            get_oldpwd(executor)?
        }
        Some(path) => {
            // Expand tilde and use path
            expand_tilde(path, executor)
        }
    };

    // Validate and change directory
    change_directory(&target, executor)
}
```

### Step 3: Register Builtin

In `crates/rush/src/executor/builtins/mod.rs`:

```rust
pub mod cd;

// In execute_builtin():
"cd" => Some(cd::execute(executor, args)),
```

### Step 4: Run Tests

```bash
cargo test -p rush cd
cargo run -p rush
# Then: cd /tmp && pwd
```

## Key Implementation Points

### Tilde Expansion

```rust
fn expand_tilde(path: &str, executor: &CommandExecutor) -> String {
    if path.starts_with("~/") {
        if let Some(home) = executor.env_manager().get("HOME") {
            return path.replacen("~", home, 1);
        }
    } else if path == "~" {
        if let Some(home) = executor.env_manager().get("HOME") {
            return home.to_string();
        }
    }
    path.to_string()
}
```

### Directory Change

```rust
fn change_directory(target: &str, executor: &mut CommandExecutor) -> Result<i32> {
    let path = Path::new(target);

    // Validate
    if !path.exists() {
        eprintln!("cd: {}: No such file or directory", target);
        return Ok(1);
    }
    if !path.is_dir() {
        eprintln!("cd: {}: Not a directory", target);
        return Ok(1);
    }

    // Save current directory as OLDPWD
    if let Ok(current) = env::current_dir() {
        executor.env_manager_mut()
            .set("OLDPWD".to_string(), current.to_string_lossy().to_string())?;
    }

    // Change directory
    if let Err(e) = env::set_current_dir(path) {
        eprintln!("cd: {}: {}", target, e);
        return Ok(1);
    }

    // Update PWD
    if let Ok(new_dir) = env::current_dir() {
        executor.env_manager_mut()
            .set("PWD".to_string(), new_dir.to_string_lossy().to_string())?;
    }

    Ok(0)
}
```

### CDPATH Search

```rust
fn search_cdpath(name: &str, cdpath: &str) -> Option<String> {
    for dir in cdpath.split(':') {
        if dir.is_empty() { continue; }
        let candidate = Path::new(dir).join(name);
        if candidate.is_dir() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        // Test with mock home
        assert_eq!(
            expand_tilde_with_home("~/Documents", "/home/user"),
            "/home/user/Documents"
        );
    }

    #[test]
    fn test_cd_to_tmp() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["/tmp".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
```

### Integration Tests

Add to `crates/rush/tests/feature_test.rs`:

```rust
#[test]
fn test_cd_basic() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("cd /tmp");
    assert!(result.is_ok());
    assert_eq!(std::env::current_dir().unwrap(), Path::new("/tmp"));
}

#[test]
fn test_cd_home() {
    let mut executor = CommandExecutor::new();
    let home = std::env::var("HOME").unwrap();
    let result = executor.execute("cd ~");
    assert!(result.is_ok());
    assert_eq!(std::env::current_dir().unwrap(), Path::new(&home));
}
```

## Common Issues

### Issue: PWD not updated
**Fix**: Always update PWD after successful `set_current_dir()`

### Issue: Tilde not expanding
**Fix**: Check HOME is set in EnvironmentManager, expand before path operations

### Issue: cd - doesn't work
**Fix**: Ensure OLDPWD is saved before every successful cd

## Next Steps

After basic implementation:
1. Add comprehensive error messages
2. Implement CDPATH search (US4)
3. Add edge case tests
4. Run coverage check: `cargo tarpaulin -p rush`
