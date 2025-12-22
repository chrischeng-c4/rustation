//! The `let` builtin command for arithmetic evaluation.
//!
//! Evaluates arithmetic expressions and assigns results to variables.
//!
//! # Usage
//! ```text
//! let expression [expression ...]
//! let "expression with spaces"
//! ```
//!
//! # Examples
//! ```text
//! let x=5+3          # x = 8
//! let "y = 10 * 2"   # y = 20 (spaces allowed in quotes)
//! let x=5 y=10       # Multiple expressions
//! let x++            # Increment x
//! ```
//!
//! # Exit Status
//! - Returns 0 if the last expression evaluates to non-zero (true)
//! - Returns 1 if the last expression evaluates to zero (false)

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `let` builtin command.
///
/// Evaluates each argument as an arithmetic expression.
/// Returns exit status based on the last expression's value.
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("let: expression expected");
        return Ok(2);
    }

    let mut last_result: i64 = 0;

    for arg in args {
        // Each argument is an arithmetic expression
        match executor.evaluate_arithmetic(arg) {
            Ok(value) => {
                last_result = value;
            }
            Err(e) => {
                eprintln!("let: {}: {}", arg, e);
                return Ok(1);
            }
        }
    }

    // Exit status: 0 if last result is non-zero (true), 1 if zero (false)
    if last_result != 0 {
        Ok(0)
    } else {
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_let_simple_assignment() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["x=5".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // 5 is non-zero
        assert_eq!(executor.variable_manager().get("x"), Some("5"));
    }

    #[test]
    fn test_let_expression() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["x=5+3".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("x"), Some("8"));
    }

    #[test]
    fn test_let_multiple_expressions() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["x=5".to_string(), "y=10".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("x"), Some("5"));
        assert_eq!(executor.variable_manager().get("y"), Some("10"));
    }

    #[test]
    fn test_let_increment() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "5".to_string())
            .unwrap();
        let result = execute(&mut executor, &["x++".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // 5 (old value) is non-zero
        assert_eq!(executor.variable_manager().get("x"), Some("6"));
    }

    #[test]
    fn test_let_zero_result() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["x=0".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // 0 is false
    }

    #[test]
    fn test_let_no_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2); // Error
    }

    #[test]
    fn test_let_compound_assignment() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "10".to_string())
            .unwrap();
        let result = execute(&mut executor, &["x+=5".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("x"), Some("15"));
    }
}
