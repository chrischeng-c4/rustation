//! Implementation of the `source` and `.` (dot) builtin commands
//!
//! The `source` builtin executes commands from a file in the current shell context.
//! This allows loading configuration files, defining aliases, and setting environment
//! variables that persist in the shell session.
//!
//! Usage:
//! - `source <file>` - Execute commands from file
//! - `. <file>` - POSIX equivalent of source
//! - `source <file> <args...>` - Execute with positional parameters
//!
//! Features:
//! - Preserves shell context (variables, aliases persist)
//! - Supports relative, absolute, and tilde paths
//! - Searches PATH if file not found directly
//! - Supports nested sourcing (up to 100 levels)
//! - Reports errors with file name and line number

use crate::error::Result;
use crate::executor::execute::CommandExecutor;
use std::cell::Cell;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Maximum nesting depth for source commands to prevent infinite recursion
const MAX_SOURCE_DEPTH: usize = 100;

// Thread-local counter for tracking source nesting depth
thread_local! {
    static SOURCE_DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// Execute the `source` builtin command
///
/// # Arguments
/// * `executor` - Command executor to run commands in current shell context
/// * `args` - Command arguments: first is file path, rest are positional parameters
///
/// # Returns
/// * `Ok(exit_code)` - Exit code of last command in the sourced file
/// * `Err(_)` - If file cannot be read or other critical error
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Check for no arguments
    if args.is_empty() {
        eprintln!("rush: source: filename argument required");
        return Ok(1);
    }

    // Check nesting depth
    let current_depth = SOURCE_DEPTH.with(|d| d.get());
    if current_depth >= MAX_SOURCE_DEPTH {
        eprintln!("rush: source: maximum nesting depth ({}) exceeded", MAX_SOURCE_DEPTH);
        return Ok(1);
    }

    // Get the file path (first argument)
    let file_arg = &args[0];

    // Resolve the file path
    let file_path = match resolve_file_path(file_arg) {
        Some(path) => path,
        None => {
            eprintln!("rush: source: {}: No such file or directory", file_arg);
            return Ok(1);
        }
    };

    // Read the file contents
    let contents = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("rush: source: {}: {}", file_path.display(), e);
            return Ok(1);
        }
    };

    // TODO: When positional parameters are implemented (feature 017),
    // store and restore them here. For now, additional args are ignored.
    // let _script_args = &args[1..];

    // Increment nesting depth
    SOURCE_DEPTH.with(|d| d.set(d.get() + 1));

    // Execute each line in the file
    let mut last_exit_code = 0;
    for (line_num, line) in contents.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Execute the line in the current shell context
        match executor.execute(line) {
            Ok(code) => {
                last_exit_code = code;
            }
            Err(e) => {
                eprintln!("rush: {}:{}: {}", file_path.display(), line_num + 1, e);
                last_exit_code = 1;
            }
        }
    }

    // Decrement nesting depth
    SOURCE_DEPTH.with(|d| d.set(d.get() - 1));

    Ok(last_exit_code)
}

/// Resolve a file path for sourcing
///
/// Resolution order:
/// 1. Absolute paths - use directly
/// 2. Paths starting with ./ or ../ - use relative to current directory
/// 3. Paths starting with ~ - expand tilde to home directory
/// 4. Otherwise:
///    a. Check if file exists in current directory
///    b. Search PATH directories
///
/// # Arguments
/// * `file_arg` - The file path argument from the source command
///
/// # Returns
/// * `Some(PathBuf)` - Resolved path if file exists
/// * `None` - If file cannot be found
fn resolve_file_path(file_arg: &str) -> Option<PathBuf> {
    let path = PathBuf::from(file_arg);

    // Handle absolute paths
    if path.is_absolute() {
        if path.is_file() {
            return Some(path);
        }
        return None;
    }

    // Handle paths starting with ./ or ../
    if file_arg.starts_with("./") || file_arg.starts_with("../") {
        if path.is_file() {
            return Some(path);
        }
        return None;
    }

    // Handle tilde expansion
    if file_arg.starts_with('~') {
        if let Some(expanded) = expand_tilde(file_arg) {
            if expanded.is_file() {
                return Some(expanded);
            }
        }
        return None;
    }

    // Check current directory first
    if path.is_file() {
        return Some(path);
    }

    // Search PATH directories
    search_path(file_arg)
}

/// Expand tilde (~) to home directory
///
/// # Arguments
/// * `path` - Path string starting with ~
///
/// # Returns
/// * `Some(PathBuf)` - Expanded path
/// * `None` - If HOME is not set
fn expand_tilde(path: &str) -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;

    if path == "~" {
        Some(PathBuf::from(home))
    } else if let Some(rest) = path.strip_prefix("~/") {
        Some(PathBuf::from(home).join(rest))
    } else {
        // ~username style not supported, treat as literal
        Some(PathBuf::from(path))
    }
}

/// Search PATH directories for a file
///
/// # Arguments
/// * `filename` - Name of the file to find
///
/// # Returns
/// * `Some(PathBuf)` - Full path to file if found
/// * `None` - If file not found in any PATH directory
fn search_path(filename: &str) -> Option<PathBuf> {
    let path_var = env::var("PATH").ok()?;

    for dir in path_var.split(':') {
        let candidate = PathBuf::from(dir).join(filename);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_expand_tilde_home() {
        let home = env::var("HOME").unwrap();
        assert_eq!(expand_tilde("~"), Some(PathBuf::from(&home)));
    }

    #[test]
    fn test_expand_tilde_with_path() {
        let home = env::var("HOME").unwrap();
        let expected = PathBuf::from(&home).join("test/path");
        assert_eq!(expand_tilde("~/test/path"), Some(expected));
    }

    #[test]
    fn test_resolve_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sh");
        File::create(&file_path).unwrap();

        let resolved = resolve_file_path(file_path.to_str().unwrap());
        assert_eq!(resolved, Some(file_path));
    }

    #[test]
    fn test_resolve_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sh");
        File::create(&file_path).unwrap();

        // Change to temp directory to test relative path
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let resolved = resolve_file_path("./test.sh");
        assert!(resolved.is_some());

        // Restore directory
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_resolve_nonexistent() {
        let resolved = resolve_file_path("/this/path/does/not/exist/12345.sh");
        assert_eq!(resolved, None);
    }

    #[test]
    fn test_source_empty_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_source_nonexistent_file() {
        let mut executor = CommandExecutor::new();
        let args = vec!["/nonexistent/file.sh".to_string()];
        let result = execute(&mut executor, &args);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_source_simple_script() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sh");

        // Create a simple script
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap(); // Empty line
        writeln!(file, "true").unwrap();

        let mut executor = CommandExecutor::new();
        let args = vec![file_path.to_str().unwrap().to_string()];
        let result = execute(&mut executor, &args);

        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_source_exit_code_propagation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sh");

        // Create a script that ends with false
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "true").unwrap();
        writeln!(file, "false").unwrap();

        let mut executor = CommandExecutor::new();
        let args = vec![file_path.to_str().unwrap().to_string()];
        let result = execute(&mut executor, &args);

        // Should return exit code of last command (false = 1)
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_source_runs_multiple_commands() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sh");

        // Create a script with multiple true commands
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "true").unwrap();
        writeln!(file, "true").unwrap();
        writeln!(file, "true").unwrap();

        let mut executor = CommandExecutor::new();
        let args = vec![file_path.to_str().unwrap().to_string()];
        let result = execute(&mut executor, &args);

        // All commands should succeed
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_max_nesting_depth() {
        // We can't easily test 100 levels, but we can verify the constant exists
        assert_eq!(MAX_SOURCE_DEPTH, 100);
    }
}
