//! Implementation of the `pwd` builtin command
//!
//! The `pwd` builtin prints the current working directory.
//!
//! Usage:
//! - `pwd` - Print current working directory

use crate::error::Result;
use crate::executor::execute::CommandExecutor;
use std::env;

/// Execute the `pwd` builtin command
///
/// # Arguments
/// * `_executor` - Command executor (not used)
/// * `_args` - Command arguments (ignored)
///
/// # Returns
/// * `Ok(0)` - Success
/// * `Ok(1)` - Error getting current directory
pub fn execute(_executor: &mut CommandExecutor, _args: &[String]) -> Result<i32> {
    match env::current_dir() {
        Ok(path) => {
            println!("{}", path.display());
            Ok(0)
        }
        Err(e) => {
            eprintln!("pwd: error retrieving current directory: {}", e);
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_pwd_succeeds() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_pwd_ignores_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["ignored".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
