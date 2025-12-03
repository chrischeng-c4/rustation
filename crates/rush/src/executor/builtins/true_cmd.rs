//! Implementation of the `true` builtin command
//!
//! The `true` builtin always succeeds (returns exit code 0).
//! Used in conditionals and scripting.
//!
//! Usage:
//! - `true` - Always exits with code 0

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `true` builtin command
///
/// # Arguments
/// * `_executor` - Command executor (not used)
/// * `_args` - Command arguments (ignored)
///
/// # Returns
/// * `Ok(0)` - Always succeeds
pub fn execute(_executor: &mut CommandExecutor, _args: &[String]) -> Result<i32> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_true_returns_zero() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_true_ignores_args() {
        let mut executor = CommandExecutor::new();
        let args = vec!["ignored".to_string(), "args".to_string()];
        let result = execute(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
