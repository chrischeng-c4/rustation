//! The `local` builtin command for declaring function-scoped variables.
//!
//! Creates variables that are local to the current function scope.
//! When the function exits, local variables are removed and any shadowed
//! global variables are restored.
//!
//! # Usage
//! ```text
//! local name[=value] [name[=value] ...]
//! ```
//!
//! # Examples
//! ```text
//! local x=10         # Declare local x with value 10
//! local y            # Declare local y with empty value
//! local a=1 b=2 c    # Multiple declarations
//! ```
//!
//! # Exit Status
//! - 0: Success
//! - 1: Used outside a function

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `local` builtin command.
///
/// Declares one or more variables as local to the current function scope.
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Check if we're in a function context
    if executor.variable_manager().scope_depth() == 0 {
        eprintln!("local: can only be used in a function");
        return Ok(1);
    }

    if args.is_empty() {
        // No arguments - just return success (bash behavior)
        return Ok(0);
    }

    for arg in args {
        // Parse name=value or just name
        if let Some(eq_pos) = arg.find('=') {
            let name = arg[..eq_pos].to_string();
            let value = arg[eq_pos + 1..].to_string();

            if let Err(e) = executor.variable_manager_mut().set_local(name, Some(value)) {
                eprintln!("local: {}", e);
                return Ok(1);
            }
        } else {
            // Just a name without value
            if let Err(e) = executor.variable_manager_mut().set_local(arg.clone(), None) {
                eprintln!("local: {}", e);
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
    fn test_local_outside_function() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["x=5".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error: not in function
    }

    #[test]
    fn test_local_in_function_scope() {
        let mut executor = CommandExecutor::new();

        // Simulate entering a function
        executor.variable_manager_mut().push_scope();

        let result = execute(&mut executor, &["x=5".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("x"), Some("5"));

        // Exit function scope
        executor.variable_manager_mut().pop_scope();
        assert_eq!(executor.variable_manager().get("x"), None);
    }

    #[test]
    fn test_local_shadows_global() {
        let mut executor = CommandExecutor::new();

        // Set global variable
        executor
            .variable_manager_mut()
            .set("x".to_string(), "global".to_string())
            .unwrap();

        // Enter function scope
        executor.variable_manager_mut().push_scope();

        // Declare local with same name
        let result = execute(&mut executor, &["x=local".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("x"), Some("local"));

        // Exit function scope - global should be restored
        executor.variable_manager_mut().pop_scope();
        assert_eq!(executor.variable_manager().get("x"), Some("global"));
    }

    #[test]
    fn test_local_without_value() {
        let mut executor = CommandExecutor::new();
        executor.variable_manager_mut().push_scope();

        let result = execute(&mut executor, &["y".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("y"), Some(""));

        executor.variable_manager_mut().pop_scope();
    }

    #[test]
    fn test_local_multiple_variables() {
        let mut executor = CommandExecutor::new();
        executor.variable_manager_mut().push_scope();

        let result =
            execute(&mut executor, &["a=1".to_string(), "b=2".to_string(), "c".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("a"), Some("1"));
        assert_eq!(executor.variable_manager().get("b"), Some("2"));
        assert_eq!(executor.variable_manager().get("c"), Some(""));

        executor.variable_manager_mut().pop_scope();
        assert_eq!(executor.variable_manager().get("a"), None);
        assert_eq!(executor.variable_manager().get("b"), None);
        assert_eq!(executor.variable_manager().get("c"), None);
    }

    #[test]
    fn test_local_nested_scopes() {
        let mut executor = CommandExecutor::new();

        // Outer function
        executor.variable_manager_mut().push_scope();
        execute(&mut executor, &["x=outer".to_string()]).unwrap();
        assert_eq!(executor.variable_manager().get("x"), Some("outer"));

        // Inner function
        executor.variable_manager_mut().push_scope();
        execute(&mut executor, &["x=inner".to_string()]).unwrap();
        assert_eq!(executor.variable_manager().get("x"), Some("inner"));

        // Exit inner
        executor.variable_manager_mut().pop_scope();
        assert_eq!(executor.variable_manager().get("x"), Some("outer"));

        // Exit outer
        executor.variable_manager_mut().pop_scope();
        assert_eq!(executor.variable_manager().get("x"), None);
    }

    #[test]
    fn test_local_no_args() {
        let mut executor = CommandExecutor::new();
        executor.variable_manager_mut().push_scope();

        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        executor.variable_manager_mut().pop_scope();
    }
}
