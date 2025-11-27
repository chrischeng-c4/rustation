//! Implementation of the `alias` builtin command
//!
//! The `alias` builtin allows users to create command shortcuts.
//!
//! Usage:
//! - `alias` - List all aliases
//! - `alias name` - Show specific alias
//! - `alias name=value` - Define/update alias

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `alias` builtin command
///
/// # Arguments
/// * `executor` - Command executor (for accessing alias manager)
/// * `args` - Command arguments
///
/// # Returns
/// * `Ok(0)` - Success
/// * `Ok(1)` - Alias not found (when querying specific alias)
pub fn alias(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        // List all aliases
        let aliases = executor.alias_manager().list();
        for (name, value) in aliases {
            println!("alias {}='{}'", name, value);
        }
        return Ok(0);
    }

    // Process each argument
    for arg in args {
        if let Some(pos) = arg.find('=') {
            // Define alias: alias name=value
            let name = &arg[..pos];
            let value = &arg[pos + 1..];

            // Remove surrounding quotes if present
            let value = value
                .trim_start_matches(|c| c == '\'' || c == '"')
                .trim_end_matches(|c| c == '\'' || c == '"');

            executor
                .alias_manager_mut()
                .set(name.to_string(), value.to_string())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_alias_list_empty() {
        let mut executor = CommandExecutor::new();
        let result = alias(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_alias_define() {
        let mut executor = CommandExecutor::new();
        let args = vec!["ll=ls -la".to_string()];
        let result = alias(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -la"));
    }

    #[test]
    fn test_alias_define_with_quotes() {
        let mut executor = CommandExecutor::new();

        // Single quotes
        let args = vec!["ll='ls -la'".to_string()];
        alias(&mut executor, &args).unwrap();
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -la"));

        // Double quotes
        let args = vec!["lsg=\"ls | grep\"".to_string()];
        alias(&mut executor, &args).unwrap();
        assert_eq!(executor.alias_manager().get("lsg"), Some("ls | grep"));
    }

    #[test]
    fn test_alias_show_specific() {
        let mut executor = CommandExecutor::new();

        // Define alias first
        executor
            .alias_manager_mut()
            .set("ll".to_string(), "ls -la".to_string())
            .unwrap();

        // Query it
        let args = vec!["ll".to_string()];
        let result = alias(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_alias_show_nonexistent() {
        let mut executor = CommandExecutor::new();
        let args = vec!["nonexistent".to_string()];
        let result = alias(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_alias_invalid_name() {
        let mut executor = CommandExecutor::new();
        let args = vec!["my-alias=value".to_string()];
        let result = alias(&mut executor, &args);
        assert!(result.is_err()); // Invalid name causes error
    }

    #[test]
    fn test_alias_update_existing() {
        let mut executor = CommandExecutor::new();

        // Define alias
        let args = vec!["ll=ls -la".to_string()];
        alias(&mut executor, &args).unwrap();
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -la"));

        // Update it
        let args = vec!["ll=ls -lah".to_string()];
        alias(&mut executor, &args).unwrap();
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -lah"));
    }
}
