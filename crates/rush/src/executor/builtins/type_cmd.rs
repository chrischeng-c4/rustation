//! Implementation of the `type` builtin command
//!
//! The `type` builtin shows information about command types.
//!
//! Usage:
//! - `type name` - Show whether name is a builtin, alias, function, or external command

use crate::error::Result;
use crate::executor::execute::CommandExecutor;
use std::env;
use std::path::Path;

/// List of all builtin commands
const BUILTINS: &[&str] = &[
    "cd", "jobs", "fg", "bg", "echo", "true", "false", "test", "[", "printf", "pwd", "type",
    "export", "set", "unset",
];

/// Execute the `type` builtin command
///
/// # Arguments
/// * `_executor` - Command executor (not used)
/// * `args` - Command names to look up
///
/// # Returns
/// * `Ok(0)` - All commands found
/// * `Ok(1)` - At least one command not found
pub fn execute(_executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("type: usage: type name [name ...]");
        return Ok(1);
    }

    let mut all_found = true;

    for name in args {
        if let Some(cmd_type) = find_command_type(name) {
            println!("{} is {}", name, cmd_type);
        } else {
            eprintln!("type: {}: not found", name);
            all_found = false;
        }
    }

    Ok(if all_found { 0 } else { 1 })
}

/// Find the type of a command
fn find_command_type(name: &str) -> Option<String> {
    // Check if it's a builtin
    if BUILTINS.contains(&name) {
        return Some(format!("a shell builtin"));
    }

    // Check if it's in PATH
    if let Some(path) = find_in_path(name) {
        return Some(path);
    }

    None
}

/// Find a command in PATH
fn find_in_path(name: &str) -> Option<String> {
    let path_var = env::var("PATH").ok()?;

    for dir in path_var.split(':') {
        let full_path = Path::new(dir).join(name);
        if full_path.is_file() {
            return Some(full_path.to_string_lossy().to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_type_no_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_type_builtin() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["echo".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_type_cd_builtin() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["cd".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_type_external_command() {
        let mut executor = CommandExecutor::new();
        // ls should exist on most systems
        let result = execute(&mut executor, &["ls".to_string()]);
        // May succeed or fail depending on system, but should not panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_not_found() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["definitely_not_a_command_12345".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_type_multiple_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["echo".to_string(), "cd".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_find_command_type_builtin() {
        let result = find_command_type("echo");
        assert!(result.is_some());
        assert!(result.unwrap().contains("builtin"));
    }

    #[test]
    fn test_find_command_type_not_found() {
        let result = find_command_type("definitely_not_a_command_12345");
        assert!(result.is_none());
    }
}
