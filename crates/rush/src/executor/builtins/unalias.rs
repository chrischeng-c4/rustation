//! Implementation of the `unalias` builtin command
//!
//! The `unalias` builtin removes shell aliases.
//!
//! Usage:
//! - `unalias name` - Remove specific alias
//! - `unalias -a` - Remove all aliases

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `unalias` builtin command
///
/// # Arguments
/// * `executor` - Command executor with alias manager
/// * `args` - Alias names to remove
///
/// # Returns
/// * `Ok(0)` - Success
/// * `Ok(1)` - Error (no args or alias not found)
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("unalias: usage: unalias name [name ...]");
        return Ok(1);
    }

    let mut exit_code = 0;

    for arg in args {
        // Handle -a flag to remove all aliases
        if arg == "-a" {
            // Remove all aliases by getting names first, then removing
            let names: Vec<String> = executor
                .alias_manager()
                .list()
                .iter()
                .map(|(n, _)| n.to_string())
                .collect();
            for name in names {
                executor.alias_manager_mut().remove(&name);
            }
            continue;
        }

        if !executor.alias_manager_mut().remove(arg) {
            eprintln!("unalias: {}: not found", arg);
            exit_code = 1;
        }
    }

    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_unalias_no_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_unalias_existing() {
        let mut executor = CommandExecutor::new();
        executor.alias_manager_mut().add("ll", "ls -la").unwrap();
        let result = execute(&mut executor, &["ll".to_string()]);
        assert_eq!(result.unwrap(), 0);
        assert!(executor.alias_manager().get("ll").is_none());
    }

    #[test]
    fn test_unalias_nonexistent() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["nonexistent".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_unalias_multiple() {
        let mut executor = CommandExecutor::new();
        executor.alias_manager_mut().add("ll", "ls -la").unwrap();
        executor.alias_manager_mut().add("gs", "git status").unwrap();
        let result = execute(&mut executor, &["ll".to_string(), "gs".to_string()]);
        assert_eq!(result.unwrap(), 0);
        assert!(executor.alias_manager().get("ll").is_none());
        assert!(executor.alias_manager().get("gs").is_none());
    }

    #[test]
    fn test_unalias_all() {
        let mut executor = CommandExecutor::new();
        executor.alias_manager_mut().add("ll", "ls -la").unwrap();
        executor.alias_manager_mut().add("gs", "git status").unwrap();
        let result = execute(&mut executor, &["-a".to_string()]);
        assert_eq!(result.unwrap(), 0);
        assert!(executor.alias_manager().is_empty());
    }

    #[test]
    fn test_unalias_mixed_results() {
        let mut executor = CommandExecutor::new();
        executor.alias_manager_mut().add("ll", "ls -la").unwrap();
        let result = execute(
            &mut executor,
            &["ll".to_string(), "nonexistent".to_string()],
        );
        assert_eq!(result.unwrap(), 1); // One failed
        assert!(executor.alias_manager().get("ll").is_none()); // But ll was removed
    }
}
