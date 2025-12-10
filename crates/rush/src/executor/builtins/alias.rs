//! Implementation of the `alias` builtin command
//!
//! The `alias` builtin creates, displays, and manages shell aliases.
//!
//! Usage:
//! - `alias` - List all aliases
//! - `alias name` - Show specific alias
//! - `alias name='value'` - Create or update alias
//! - `alias name=value` - Create or update alias (unquoted)

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `alias` builtin command
///
/// # Arguments
/// * `executor` - Command executor with alias manager
/// * `args` - Command arguments
///
/// # Returns
/// * `Ok(0)` - Success
/// * `Ok(1)` - Error (invalid syntax or alias not found)
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // No arguments: list all aliases
    if args.is_empty() {
        let aliases = executor.alias_manager().list();
        if aliases.is_empty() {
            // No aliases defined - just return success (bash behavior)
            return Ok(0);
        }
        for (name, value) in aliases {
            println!("alias {}='{}'", name, value.replace('\'', "'\\''"));
        }
        return Ok(0);
    }

    let mut exit_code = 0;

    for arg in args {
        // Check if this is an assignment (contains =)
        if let Some(equals_pos) = arg.find('=') {
            // alias name='value' or alias name=value
            let name = &arg[..equals_pos];
            let mut value = &arg[equals_pos + 1..];

            // Remove surrounding quotes if present
            if (value.starts_with('\'') && value.ends_with('\''))
                || (value.starts_with('"') && value.ends_with('"'))
            {
                if value.len() >= 2 {
                    value = &value[1..value.len() - 1];
                }
            }

            // Add the alias
            if let Err(e) = executor.alias_manager_mut().add(name, value) {
                eprintln!("alias: {}", e);
                exit_code = 1;
            }
        } else {
            // alias name - show specific alias
            if let Some(value) = executor.alias_manager().get(arg) {
                println!("alias {}='{}'", arg, value.replace('\'', "'\\''"));
            } else {
                eprintln!("alias: {}: not found", arg);
                exit_code = 1;
            }
        }
    }

    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_alias_list_empty() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_alias_create() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["ll='ls -la'".to_string()]);
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -la"));
    }

    #[test]
    fn test_alias_create_no_quotes() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["ll=ls".to_string()]);
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.alias_manager().get("ll"), Some("ls"));
    }

    #[test]
    fn test_alias_create_double_quotes() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["ll=\"ls -la\"".to_string()]);
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -la"));
    }

    #[test]
    fn test_alias_show_specific() {
        let mut executor = CommandExecutor::new();
        executor.alias_manager_mut().add("ll", "ls -la").unwrap();
        let result = execute(&mut executor, &["ll".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_alias_show_not_found() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["nonexistent".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_alias_invalid_name() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["123='invalid'".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_alias_multiple() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["ll='ls -la'".to_string(), "gs='git status'".to_string()]);
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -la"));
        assert_eq!(executor.alias_manager().get("gs"), Some("git status"));
    }
}
