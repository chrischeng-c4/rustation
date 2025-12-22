//! Implementation of the `false` builtin command
//!
//! The `false` builtin always fails (returns exit code 1).
//! Used in conditionals and scripting.
//!
//! Usage:
//! - `false` - Always exits with code 1

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `false` builtin command
///
/// # Arguments
/// * `_executor` - Command executor (not used)
/// * `_args` - Command arguments (ignored)
///
/// # Returns
/// * `Ok(1)` - Always fails
pub fn execute(_executor: &mut CommandExecutor, _args: &[String]) -> Result<i32> {
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_false_returns_one() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_false_ignores_args() {
        let mut executor = CommandExecutor::new();
        let args = vec!["ignored".to_string(), "args".to_string()];
        let result = execute(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
