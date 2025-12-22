//! Implementation of the `exit` builtin command
//!
//! The `exit` builtin terminates the shell with an optional exit code.
//! When called with no arguments, exits with the status of the last executed command.
//! When called with a numeric argument, exits with that status code.
//!
//! Usage:
//! - `exit` - Exit with last command's exit code
//! - `exit N` - Exit with exit code N (masked to 0-255)
//!
//! Features:
//! - POSIX compliant exit code masking (value & 0xFF)
//! - Clear error messages for invalid arguments
//! - Does not exit on invalid arguments (allows correction)

use crate::error::Result;
use crate::executor::execute::CommandExecutor;
use crate::RushError;

/// Execute the `exit` builtin command
///
/// # Arguments
/// * `executor` - Command executor to get last exit code
/// * `args` - Command arguments: optional exit code
///
/// # Returns
/// * `Err(RushError::ExitRequest(code))` - Signal to exit shell with code
/// * `Ok(1)` - If there's an error (invalid args), don't exit
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Check for too many arguments
    if args.len() > 1 {
        eprintln!("rush: exit: too many arguments");
        return Ok(1);
    }

    // Determine exit code
    let exit_code = if args.is_empty() {
        // No arguments: use last exit code
        executor.last_exit_code()
    } else {
        // One argument: parse as exit code
        let arg = &args[0];
        match arg.parse::<i64>() {
            Ok(code) => {
                // Mask to 0-255 range (POSIX compliant)
                (code & 0xFF) as i32
            }
            Err(_) => {
                // Non-numeric argument
                eprintln!("rush: exit: {}: numeric argument required", arg);
                return Ok(1);
            }
        }
    };

    // Signal exit request
    Err(RushError::ExitRequest(exit_code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_no_args() {
        let mut executor = CommandExecutor::new();
        executor.set_last_exit_code(42);
        let result = execute(&mut executor, &[]);

        // Should return ExitRequest error with last exit code
        match result {
            Err(RushError::ExitRequest(code)) => {
                assert_eq!(code, 42);
            }
            _ => panic!("Expected ExitRequest error"),
        }
    }

    #[test]
    fn test_exit_with_code() {
        let mut executor = CommandExecutor::new();
        let args = vec!["0".to_string()];
        let result = execute(&mut executor, &args);

        match result {
            Err(RushError::ExitRequest(code)) => {
                assert_eq!(code, 0);
            }
            _ => panic!("Expected ExitRequest error"),
        }
    }

    #[test]
    fn test_exit_with_code_42() {
        let mut executor = CommandExecutor::new();
        let args = vec!["42".to_string()];
        let result = execute(&mut executor, &args);

        match result {
            Err(RushError::ExitRequest(code)) => {
                assert_eq!(code, 42);
            }
            _ => panic!("Expected ExitRequest error"),
        }
    }

    #[test]
    fn test_exit_with_code_255() {
        let mut executor = CommandExecutor::new();
        let args = vec!["255".to_string()];
        let result = execute(&mut executor, &args);

        match result {
            Err(RushError::ExitRequest(code)) => {
                assert_eq!(code, 255);
            }
            _ => panic!("Expected ExitRequest error"),
        }
    }

    #[test]
    fn test_exit_code_wrapping_256() {
        let mut executor = CommandExecutor::new();
        let args = vec!["256".to_string()];
        let result = execute(&mut executor, &args);

        match result {
            Err(RushError::ExitRequest(code)) => {
                // 256 & 0xFF = 0
                assert_eq!(code, 0);
            }
            _ => panic!("Expected ExitRequest error"),
        }
    }

    #[test]
    fn test_exit_code_wrapping_257() {
        let mut executor = CommandExecutor::new();
        let args = vec!["257".to_string()];
        let result = execute(&mut executor, &args);

        match result {
            Err(RushError::ExitRequest(code)) => {
                // 257 & 0xFF = 1
                assert_eq!(code, 1);
            }
            _ => panic!("Expected ExitRequest error"),
        }
    }

    #[test]
    fn test_exit_code_negative() {
        let mut executor = CommandExecutor::new();
        let args = vec!["-1".to_string()];
        let result = execute(&mut executor, &args);

        match result {
            Err(RushError::ExitRequest(code)) => {
                // -1 & 0xFF = 255
                assert_eq!(code, 255);
            }
            _ => panic!("Expected ExitRequest error"),
        }
    }

    #[test]
    fn test_exit_non_numeric_arg() {
        let mut executor = CommandExecutor::new();
        let args = vec!["abc".to_string()];
        let result = execute(&mut executor, &args);

        // Should return Ok(1) for error, not exit
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_exit_too_many_args() {
        let mut executor = CommandExecutor::new();
        let args = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let result = execute(&mut executor, &args);

        // Should return Ok(1) for error, not exit
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_exit_floating_point() {
        let mut executor = CommandExecutor::new();
        let args = vec!["1.5".to_string()];
        let result = execute(&mut executor, &args);

        // Should return Ok(1) for error (not numeric)
        assert_eq!(result.unwrap(), 1);
    }
}
