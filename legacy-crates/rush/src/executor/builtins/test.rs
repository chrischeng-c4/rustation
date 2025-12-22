//! Implementation of the `test` builtin command
//!
//! The `test` builtin evaluates conditional expressions.
//!
//! Usage:
//! - `test expression` - Evaluate expression
//! - `test -f file` - True if file exists and is regular file
//! - `test -d dir` - True if directory exists
//! - `test -e path` - True if path exists
//! - `test -n string` - True if string is non-empty
//! - `test -z string` - True if string is empty
//! - `test str1 = str2` - True if strings are equal
//! - `test str1 != str2` - True if strings are not equal
//! - `test n1 -eq n2` - True if integers are equal
//! - `test n1 -ne n2` - True if integers are not equal
//! - `test n1 -lt n2` - True if n1 < n2
//! - `test n1 -le n2` - True if n1 <= n2
//! - `test n1 -gt n2` - True if n1 > n2
//! - `test n1 -ge n2` - True if n1 >= n2

use crate::error::Result;
use crate::executor::execute::CommandExecutor;
use std::path::Path;

/// Execute the `test` builtin command
///
/// # Arguments
/// * `_executor` - Command executor (not used)
/// * `args` - Test expression
///
/// # Returns
/// * `Ok(0)` - Expression is true
/// * `Ok(1)` - Expression is false
/// * `Ok(2)` - Invalid expression or error
pub fn execute(_executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // No arguments: false
    if args.is_empty() {
        return Ok(1);
    }

    // Single argument: true if non-empty string
    if args.len() == 1 {
        return Ok(if args[0].is_empty() { 1 } else { 0 });
    }

    // Two arguments: unary operator
    if args.len() == 2 {
        return evaluate_unary(&args[0], &args[1]);
    }

    // Three arguments: binary operator
    if args.len() == 3 {
        return evaluate_binary(&args[0], &args[1], &args[2]);
    }

    // Too many arguments
    eprintln!("test: too many arguments");
    Ok(2)
}

/// Evaluate unary expressions like `-f file`, `-d dir`, `-n string`
fn evaluate_unary(operator: &str, operand: &str) -> Result<i32> {
    let result = match operator {
        "-f" => {
            // File exists and is regular file
            let path = Path::new(operand);
            path.is_file()
        }
        "-d" => {
            // Directory exists
            let path = Path::new(operand);
            path.is_dir()
        }
        "-e" => {
            // Path exists (file or directory)
            let path = Path::new(operand);
            path.exists()
        }
        "-n" => {
            // String is non-empty
            !operand.is_empty()
        }
        "-z" => {
            // String is empty
            operand.is_empty()
        }
        "-r" => {
            // File is readable
            let path = Path::new(operand);
            path.exists() // Simplified: just check exists
        }
        "-w" => {
            // File is writable
            let path = Path::new(operand);
            path.exists() // Simplified: just check exists
        }
        "-x" => {
            // File is executable
            let path = Path::new(operand);
            path.exists() // Simplified: just check exists
        }
        "-s" => {
            // File exists and has size > 0
            let path = Path::new(operand);
            path.is_file() && path.metadata().map(|m| m.len() > 0).unwrap_or(false)
        }
        _ => {
            eprintln!("test: unknown unary operator: {}", operator);
            return Ok(2);
        }
    };

    Ok(if result { 0 } else { 1 })
}

/// Evaluate binary expressions like `str1 = str2`, `n1 -eq n2`
fn evaluate_binary(left: &str, operator: &str, right: &str) -> Result<i32> {
    let result = match operator {
        // String comparisons
        "=" | "==" => left == right,
        "!=" => left != right,

        // Integer comparisons
        "-eq" => compare_integers(left, right, |a, b| a == b),
        "-ne" => compare_integers(left, right, |a, b| a != b),
        "-lt" => compare_integers(left, right, |a, b| a < b),
        "-le" => compare_integers(left, right, |a, b| a <= b),
        "-gt" => compare_integers(left, right, |a, b| a > b),
        "-ge" => compare_integers(left, right, |a, b| a >= b),

        _ => {
            eprintln!("test: unknown binary operator: {}", operator);
            return Ok(2);
        }
    };

    Ok(if result { 0 } else { 1 })
}

/// Compare two strings as integers using a comparison function
fn compare_integers<F>(left: &str, right: &str, cmp: F) -> bool
where
    F: Fn(i64, i64) -> bool,
{
    match (left.parse::<i64>(), right.parse::<i64>()) {
        (Ok(a), Ok(b)) => cmp(a, b),
        _ => false, // Invalid integers compare as false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;
    use std::fs::File;
    use tempfile::TempDir;

    // === No Arguments ===

    #[test]
    fn test_no_args_returns_false() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert_eq!(result.unwrap(), 1);
    }

    // === Single Argument (String Truth) ===

    #[test]
    fn test_single_arg_nonempty_true() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["hello".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_single_arg_empty_false() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    // === File Tests ===

    #[test]
    fn test_file_exists() {
        let mut executor = CommandExecutor::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let result =
            execute(&mut executor, &["-f".to_string(), file_path.to_string_lossy().to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_file_not_exists() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["-f".to_string(), "/nonexistent/file.txt".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_directory_exists() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["-d".to_string(), "/tmp".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_directory_not_exists() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["-d".to_string(), "/nonexistent/dir".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_path_exists() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["-e".to_string(), "/tmp".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    // === String Tests ===

    #[test]
    fn test_string_nonempty() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["-n".to_string(), "hello".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_string_empty_n() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["-n".to_string(), "".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_string_empty_z() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["-z".to_string(), "".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_string_nonempty_z() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["-z".to_string(), "hello".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    // === String Equality ===

    #[test]
    fn test_string_equal() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["hello".to_string(), "=".to_string(), "hello".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_string_not_equal() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["hello".to_string(), "=".to_string(), "world".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_string_not_equal_operator() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["hello".to_string(), "!=".to_string(), "world".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    // === Integer Comparisons ===

    #[test]
    fn test_integer_equal() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["42".to_string(), "-eq".to_string(), "42".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_integer_not_equal() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["42".to_string(), "-ne".to_string(), "43".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_integer_less_than() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["5".to_string(), "-lt".to_string(), "10".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_integer_less_equal() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["10".to_string(), "-le".to_string(), "10".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_integer_greater_than() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["10".to_string(), "-gt".to_string(), "5".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_integer_greater_equal() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["10".to_string(), "-ge".to_string(), "10".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_integer_invalid_returns_false() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["abc".to_string(), "-eq".to_string(), "42".to_string()]);
        assert_eq!(result.unwrap(), 1);
    }

    // === Error Cases ===

    #[test]
    fn test_too_many_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(
            &mut executor,
            &[
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
        );
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_unknown_unary_operator() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["-unknown".to_string(), "value".to_string()]);
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_unknown_binary_operator() {
        let mut executor = CommandExecutor::new();
        let result =
            execute(&mut executor, &["a".to_string(), "-unknown".to_string(), "b".to_string()]);
        assert_eq!(result.unwrap(), 2);
    }
}
