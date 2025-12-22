//! Implementation of the `echo` builtin command
//!
//! The `echo` builtin prints its arguments to stdout.
//!
//! Usage:
//! - `echo [args...]` - Print arguments separated by spaces, followed by newline
//! - `echo -n [args...]` - Print arguments without trailing newline

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `echo` builtin command
///
/// # Arguments
/// * `_executor` - Command executor (not used for echo)
/// * `args` - Command arguments to print
///
/// # Returns
/// * `Ok(0)` - Always succeeds
pub fn execute(_executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    let mut suppress_newline = false;
    let mut print_args: &[String] = args;

    // Check for -n flag (suppress trailing newline)
    if !args.is_empty() && args[0] == "-n" {
        suppress_newline = true;
        print_args = &args[1..];
    }

    // Print arguments separated by spaces
    let output = print_args.join(" ");

    if suppress_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_echo_simple() {
        let mut executor = CommandExecutor::new();
        let args = vec!["hello".to_string()];
        let result = execute(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_echo_multiple_args() {
        let mut executor = CommandExecutor::new();
        let args = vec!["hello".to_string(), "world".to_string()];
        let result = execute(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_echo_no_args() {
        let mut executor = CommandExecutor::new();
        let args: Vec<String> = vec![];
        let result = execute(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_echo_with_n_flag() {
        let mut executor = CommandExecutor::new();
        let args = vec!["-n".to_string(), "hello".to_string()];
        let result = execute(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_echo_n_flag_alone() {
        let mut executor = CommandExecutor::new();
        let args = vec!["-n".to_string()];
        let result = execute(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_echo_always_succeeds() {
        let mut executor = CommandExecutor::new();

        // Echo always returns 0
        assert_eq!(execute(&mut executor, &[]).unwrap(), 0);
        assert_eq!(execute(&mut executor, &["test".to_string()]).unwrap(), 0);
        assert_eq!(execute(&mut executor, &["-n".to_string()]).unwrap(), 0);
    }
}
