# Contract: cd Builtin

**Feature**: 008-cd-builtin
**Date**: 2025-11-27

## Function Signature

```rust
/// Execute the 'cd' command to change the current working directory
///
/// # Arguments
/// * `executor` - Mutable reference to CommandExecutor for environment access
/// * `args` - Command arguments (0 or 1 argument expected)
///
/// # Returns
/// * `Ok(0)` - Directory change successful
/// * `Ok(1)` - Error occurred (message printed to stderr)
///
/// # Environment Variables Used
/// * `HOME` - Read for `cd` with no args and tilde expansion
/// * `PWD` - Written after successful directory change
/// * `OLDPWD` - Read for `cd -`, written before directory change
/// * `CDPATH` - Read for relative path search
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32>
```

## Input/Output Contract

### Inputs

| Argument | Behavior |
|----------|----------|
| (none) | Change to HOME directory |
| `-` | Change to OLDPWD (previous directory) |
| `~` | Change to HOME directory |
| `~/path` | Change to HOME/path |
| `/absolute/path` | Change to absolute path |
| `relative/path` | Change to path relative to cwd, or search CDPATH |

### Outputs

| Condition | Exit Code | Stdout | Stderr |
|-----------|-----------|--------|--------|
| Success | 0 | (none) | (none) |
| Success with `cd -` | 0 | New directory path | (none) |
| Path not found | 1 | (none) | `cd: [path]: No such file or directory` |
| Not a directory | 1 | (none) | `cd: [path]: Not a directory` |
| Permission denied | 1 | (none) | `cd: [path]: Permission denied` |
| HOME not set | 1 | (none) | `cd: HOME not set` |
| OLDPWD not set | 1 | (none) | `cd: OLDPWD not set` |

### Side Effects

1. **Process Working Directory**: Changed via `std::env::set_current_dir()`
2. **PWD**: Updated to canonicalized new directory path
3. **OLDPWD**: Updated to previous PWD value (before change)

## Helper Functions

```rust
/// Expand tilde at start of path to HOME directory
///
/// # Examples
/// * `~` → `/Users/chris`
/// * `~/Documents` → `/Users/chris/Documents`
/// * `/tmp` → `/tmp` (unchanged)
fn expand_tilde(path: &str, home: &str) -> String

/// Search CDPATH for a relative directory name
///
/// # Arguments
/// * `name` - Relative directory name to search for
/// * `cdpath` - Colon-separated list of directories
///
/// # Returns
/// * `Some(path)` - Full path to found directory
/// * `None` - Directory not found in any CDPATH entry
fn search_cdpath(name: &str, cdpath: &str) -> Option<PathBuf>

/// Validate that path exists, is a directory, and is accessible
///
/// # Returns
/// * `Ok(())` - Path is valid
/// * `Err(message)` - Validation failed with error message
fn validate_path(path: &Path) -> Result<(), String>
```

## Error Handling

All errors are handled gracefully:
1. Print error message to stderr
2. Return exit code 1
3. Do NOT modify PWD, OLDPWD, or process directory on error

## Thread Safety

The cd builtin modifies global process state (`set_current_dir`). This is acceptable because:
1. Shell commands execute sequentially (no concurrent cd calls)
2. Environment variables are managed by single-threaded EnvironmentManager
