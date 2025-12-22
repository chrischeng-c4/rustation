//! Implementation of the `[` builtin command
//!
//! The `[` builtin is an alias for `test` with a required `]` at the end.
//!
//! Usage:
//! - `[ expression ]` - Evaluate expression (requires closing `]`)

use crate::error::Result;
use crate::executor::builtins::test;
use crate::executor::execute::CommandExecutor;

/// Execute the `[` builtin command
///
/// # Arguments
/// * `executor` - Command executor
/// * `args` - Test expression (must end with `]`)
///
/// # Returns
/// * `Ok(0)` - Expression is true
/// * `Ok(1)` - Expression is false
/// * `Ok(2)` - Invalid expression or missing `]`
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Must have at least `]` at the end
    if args.is_empty() {
        eprintln!("[: missing `]`");
        return Ok(2);
    }

    // Check for closing bracket
    if args.last() != Some(&"]".to_string()) {
        eprintln!("[: missing `]`");
        return Ok(2);
    }

    // Remove the closing bracket and delegate to test
    let test_args = &args[..args.len() - 1];
    test::execute(executor, test_args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_bracket_simple() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["hello".to_string(), "]".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bracket_empty_expression() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["]".to_string()]);
        assert_eq!(result.unwrap(), 1); // Empty test returns false
    }

    #[test]
    fn test_bracket_missing_closing() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["hello".to_string()]);
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_bracket_no_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_bracket_string_equal() {
        let mut executor = CommandExecutor::new();
        let result = execute(
            &mut executor,
            &[
                "hello".to_string(),
                "=".to_string(),
                "hello".to_string(),
                "]".to_string(),
            ],
        );
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bracket_file_test() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["-d".to_string(), "/tmp".to_string(), "]".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }
}
