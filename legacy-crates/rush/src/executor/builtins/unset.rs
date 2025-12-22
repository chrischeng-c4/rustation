//! Implementation of the `unset` builtin command
//!
//! The `unset` builtin removes variables from the shell.
//!
//! Usage:
//! - `unset NAME` - Remove variable
//! - `unset NAME1 NAME2 ...` - Remove multiple variables

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `unset` builtin command
pub fn unset(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("rush: unset: usage: unset name [name ...]");
        return Ok(1);
    }

    let mut exit_code = 0;
    for name in args {
        if !executor.variable_manager_mut().remove(name) {
            eprintln!("rush: unset: {}: not found", name);
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
    fn test_unset_existing_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("myvar".to_string(), "value".to_string())
            .unwrap();
        assert_eq!(executor.variable_manager().get("myvar"), Some("value"));

        let result = unset(&mut executor, &vec!["myvar".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("myvar"), None);
    }

    #[test]
    fn test_unset_nonexistent_variable() {
        let mut executor = CommandExecutor::new();
        let result = unset(&mut executor, &vec!["nonexistent".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_unset_no_arguments() {
        let mut executor = CommandExecutor::new();
        let result = unset(&mut executor, &vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_unset_multiple_variables() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("var1".to_string(), "value1".to_string())
            .unwrap();
        executor
            .variable_manager_mut()
            .set("var2".to_string(), "value2".to_string())
            .unwrap();

        let result = unset(&mut executor, &vec!["var1".to_string(), "var2".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("var1"), None);
        assert_eq!(executor.variable_manager().get("var2"), None);
    }

    #[test]
    fn test_unset_mixed_success_and_failure() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("existing".to_string(), "value".to_string())
            .unwrap();

        let result = unset(&mut executor, &vec!["existing".to_string(), "nonexistent".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code (at least one failed)
        assert_eq!(executor.variable_manager().get("existing"), None);
    }
}
