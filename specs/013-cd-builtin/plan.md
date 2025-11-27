# Implementation Plan: cd Builtin

**Feature:** 013-cd-builtin
**Created:** 2025-11-28

## Implementation Approach

The `cd` builtin will be implemented as a new module in `crates/rush/src/executor/builtins/cd.rs` following the same pattern as existing builtins (jobs, fg, bg).

## Architecture

### Module Structure
```
crates/rush/src/executor/builtins/
├── mod.rs          # Register cd builtin
├── cd.rs           # New: cd implementation
├── jobs.rs         # Existing
├── fg.rs           # Existing
└── bg.rs           # Existing
```

### Key Components

1. **cd.rs** - Main implementation
   - `execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32>`
   - Helper functions for path resolution
   - Environment variable management

2. **mod.rs** - Registration
   - Add `pub mod cd;`
   - Add `"cd" => Some(cd::execute(executor, args))` to match statement

## Implementation Steps

### Step 1: Create cd.rs Module

**File:** `crates/rush/src/executor/builtins/cd.rs`

```rust
use crate::error::Result;
use crate::executor::execute::CommandExecutor;
use std::env;
use std::path::PathBuf;

pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Validate arguments
    if args.len() > 1 {
        eprintln!("rush: cd: too many arguments");
        return Ok(1);
    }

    // Determine target directory
    let target = match args.first() {
        None => get_home_directory()?,           // cd with no args
        Some(path) if path == "-" => get_oldpwd()?,  // cd -
        Some(path) if path.starts_with('~') => expand_tilde(path)?,  // cd ~
        Some(path) => PathBuf::from(path),       // cd <path>
    };

    // Change directory
    change_directory(&target)
}
```

**Helper Functions:**

```rust
fn get_home_directory() -> Result<PathBuf> {
    env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| RushError::Execution("cd: HOME not set".to_string()))
}

fn get_oldpwd() -> Result<PathBuf> {
    env::var("OLDPWD")
        .map(PathBuf::from)
        .map_err(|_| RushError::Execution("cd: OLDPWD not set".to_string()))
}

fn expand_tilde(path: &str) -> Result<PathBuf> {
    if path == "~" {
        get_home_directory()
    } else {
        // ~/<rest> -> $HOME/<rest>
        let home = get_home_directory()?;
        Ok(home.join(&path[2..]))  // Skip "~/"
    }
}

fn change_directory(target: &PathBuf) -> Result<i32> {
    // Save current directory as OLDPWD
    if let Ok(current) = env::current_dir() {
        env::set_var("OLDPWD", current.to_string_lossy().as_ref());
    }

    // Change directory
    match env::set_current_dir(target) {
        Ok(_) => {
            // Update PWD
            if let Ok(new_pwd) = env::current_dir() {
                env::set_var("PWD", new_pwd.to_string_lossy().as_ref());
            }
            Ok(0)
        }
        Err(e) => {
            eprintln!("rush: cd: {}: {}", target.display(), e);
            Ok(1)
        }
    }
}
```

**Special Case for cd -:**

```rust
// In execute():
Some(path) if path == "-" => {
    let oldpwd = get_oldpwd()?;
    // Print new directory (bash behavior)
    println!("{}", oldpwd.display());
    change_directory(&oldpwd)
}
```

### Step 2: Register in mod.rs

**File:** `crates/rush/src/executor/builtins/mod.rs`

Add module declaration:
```rust
pub mod cd;
```

Add to match statement:
```rust
pub fn execute_builtin(executor: &mut CommandExecutor, command: &str, args: &[String]) -> Option<Result<i32>> {
    match command {
        "cd" => Some(cd::execute(executor, args)),
        "jobs" => Some(jobs::execute(executor, args)),
        // ...
    }
}
```

### Step 3: Add Unit Tests

**In cd.rs:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_cd_home() {
        // Test cd with no args goes to HOME
    }

    #[test]
    fn test_cd_absolute_path() {
        // Test cd /tmp
    }

    #[test]
    fn test_cd_relative_path() {
        // Test cd .. and cd ./subdir
    }

    #[test]
    fn test_cd_tilde() {
        // Test cd ~ and cd ~/Documents
    }

    #[test]
    fn test_cd_dash() {
        // Test cd - toggles between directories
    }

    #[test]
    fn test_cd_nonexistent() {
        // Test error handling for invalid paths
    }

    #[test]
    fn test_cd_too_many_args() {
        // Test cd foo bar returns error
    }
}
```

### Step 4: Add Integration Tests

**File:** `crates/rush/tests/integration/builtin_tests.rs` (new or existing)

```rust
#[test]
fn test_cd_integration() {
    let mut executor = CommandExecutor::new();

    // Test cd to /tmp
    let result = executor.execute("cd /tmp");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    // Verify we're in /tmp
    assert_eq!(env::current_dir().unwrap(), PathBuf::from("/tmp"));
}
```

## Edge Cases to Handle

1. **Invalid Arguments:**
   - Too many arguments: `cd foo bar` → error
   - Empty string argument: `cd ""` → error

2. **Permission Errors:**
   - Directory exists but no execute permission
   - Return exit code 1 with clear error message

3. **Special Cases:**
   - `cd .` → no-op (stay in current directory)
   - `cd ..` from root → stay at root
   - `cd ~` when HOME not set → error
   - `cd -` when OLDPWD not set → error

4. **Path Resolution:**
   - `cd ~/./Documents/../Downloads` → resolve to ~/Downloads
   - Symlinks → follow by default

## Testing Strategy

### Unit Tests (in cd.rs)
- Test each user story independently
- Mock environment variables where needed
- Test error paths

### Integration Tests
- Test cd actually changes shell's working directory
- Test environment variables are updated correctly
- Test interaction with other commands after cd

### Manual Testing
```bash
$ cargo run -p rush
> pwd
> cd /tmp
> pwd        # Should show /tmp
> cd ~
> pwd        # Should show $HOME
> cd /tmp
> cd -
> pwd        # Should show $HOME again
```

## Environment Variable Management

The cd builtin must maintain these environment variables:

| Variable | Purpose | Update Timing |
|----------|---------|---------------|
| PWD | Current directory | After successful cd |
| OLDPWD | Previous directory | Before cd |
| HOME | Home directory | Read-only (used for ~) |

**Update Logic:**
```rust
1. Save current PWD to OLDPWD
2. Call env::set_current_dir(target)
3. If successful, update PWD to new directory
4. If failed, leave PWD and OLDPWD unchanged
```

## Error Messages

Follow bash-style error messages:

| Error | Message Format |
|-------|----------------|
| No such directory | `rush: cd: /path: No such file or directory` |
| Permission denied | `rush: cd: /path: Permission denied` |
| Not a directory | `rush: cd: /path: Not a directory` |
| Too many args | `rush: cd: too many arguments` |
| HOME not set | `rush: cd: HOME not set` |
| OLDPWD not set | `rush: cd: OLDPWD not set` |

## Success Criteria

- [ ] All 4 user stories implemented
- [ ] Unit tests pass (7+ tests)
- [ ] Integration tests pass
- [ ] Error messages match bash format
- [ ] PWD and OLDPWD correctly maintained
- [ ] cd - prints directory like bash
- [ ] No clippy warnings
- [ ] Formatted with cargo fmt

## Non-Goals

These features are explicitly out of scope:
- CDPATH support
- cd ~username (other users' home directories)
- -P and -L flags (physical vs logical paths)
- Directory stack (pushd/popd/dirs)

## Dependencies

- `std::env` - Environment variable access and CWD manipulation
- `std::path::PathBuf` - Path handling
- Existing `RushError` types

No new external dependencies required.
