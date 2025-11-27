//! Implementation of the `unalias` builtin command
//!
//! The `unalias` builtin removes command aliases.
//!
//! Usage:
//! - `unalias name` - Remove alias
//! - `unalias name1 name2` - Remove multiple aliases

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `unalias` builtin command
///
/// # Arguments
/// * `executor` - Command executor (for accessing alias manager)
/// * `args` - Command arguments (alias names to remove)
///
/// # Returns
/// * `Ok(0)` - All aliases removed successfully
/// * `Ok(1)` - One or more aliases not found
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_unalias_existing() {
        let mut executor = CommandExecutor::new();

        // Define alias first
        executor
            .alias_manager_mut()
            .set("ll".to_string(), "ls -la".to_string())
            .unwrap();

        // Remove it
        let args = vec!["ll".to_string()];
        let result = unalias(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.alias_manager().get("ll"), None);
    }

    #[test]
    fn test_unalias_nonexistent() {
        let mut executor = CommandExecutor::new();
        let args = vec!["nonexistent".to_string()];
        let result = unalias(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_unalias_multiple() {
        let mut executor = CommandExecutor::new();

        // Define two aliases
        executor
            .alias_manager_mut()
            .set("ll".to_string(), "ls -la".to_string())
            .unwrap();
        executor
            .alias_manager_mut()
            .set("lsg".to_string(), "ls | grep".to_string())
            .unwrap();

        // Remove both
        let args = vec!["ll".to_string(), "lsg".to_string()];
        let result = unalias(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.alias_manager().get("ll"), None);
        assert_eq!(executor.alias_manager().get("lsg"), None);
    }

    #[test]
    fn test_unalias_mixed_success_failure() {
        let mut executor = CommandExecutor::new();

        // Define one alias
        executor
            .alias_manager_mut()
            .set("ll".to_string(), "ls -la".to_string())
            .unwrap();

        // Try to remove one existing and one nonexistent
        let args = vec!["ll".to_string(), "nonexistent".to_string()];
        let result = unalias(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error because one failed
        assert_eq!(executor.alias_manager().get("ll"), None); // But ll was removed
    }

    #[test]
    fn test_unalias_no_args() {
        let mut executor = CommandExecutor::new();
        let args = vec![];
        let result = unalias(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error - no args
    }
}
