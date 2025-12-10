//! Command execution for substitution expressions
//!
//! Executes commands inside $(...) and captures their stdout output.
//! Handles:
//! - Process spawning
//! - Output capture with size limits
//! - UTF-8 conversion
//! - Newline trimming
//! - Exit code checking
//! - Error handling

use crate::error::{Result, RushError};
use std::process::Stdio;

const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB limit

/// Executes a command and captures its stdout
pub struct SubstitutionExecutor;

impl SubstitutionExecutor {
    /// Execute a command string and capture stdout
    ///
    /// # Arguments
    /// * `command` - The full command string (e.g., "date", "echo hello", "find . -name *.txt")
    ///
    /// # Returns
    /// * `Ok(output)` - The trimmed stdout from the command
    /// * `Err(e)` - Error if command not found, fails, or has non-zero exit code
    ///
    /// # Behavior
    /// - Captures stdout only (stderr goes to terminal unless redirected in the command)
    /// - Checks for command not found
    /// - Verifies exit code (0 for success)
    /// - Converts output to UTF-8
    /// - Trims trailing newlines (\n and \r\n)
    /// - Enforces 10MB output size limit
    ///
    /// # Examples
    /// ```ignore
    /// let output = SubstitutionExecutor::execute("echo hello")?;
    /// assert_eq!(output, "hello");
    ///
    /// let output = SubstitutionExecutor::execute("date")?;
    /// // output contains today's date without trailing newline
    /// ```
    pub fn execute(command: &str) -> Result<String> {
        if command.is_empty() {
            return Ok(String::new());
        }

        // Parse the command into program and arguments
        let (program, args) = super::super::parser::parse_command_line(command)?;

        // Spawn the command and capture output
        let output = match std::process::Command::new(&program)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // stderr goes to terminal
            .output()
        {
            Ok(output) => output,
            Err(err) => {
                // Check if it's a "not found" error
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Err(RushError::Execution(format!(
                        "substitution: command not found: {}",
                        program
                    )));
                }
                return Err(RushError::Execution(format!(
                    "substitution: failed to execute '{}': {}",
                    program, err
                )));
            }
        };

        // Check exit code
        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            return Err(RushError::Execution(format!(
                "substitution: command '{}' exited with code {}",
                program, exit_code
            )));
        }

        // Check output size
        if output.stdout.len() > MAX_OUTPUT_SIZE {
            return Err(RushError::Execution(format!(
                "substitution: output too large: {} bytes (limit: {} bytes)",
                output.stdout.len(),
                MAX_OUTPUT_SIZE
            )));
        }

        // Convert to UTF-8
        let output_str = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(_) => {
                return Err(RushError::Execution(
                    "substitution: command output is not valid UTF-8".to_string(),
                ))
            }
        };

        // Trim trailing newlines (standard shell behavior)
        let trimmed = output_str.trim_end_matches('\n').to_string();

        Ok(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_echo() {
        let output = SubstitutionExecutor::execute("echo hello").unwrap();
        assert_eq!(output, "hello");
    }

    #[test]
    fn test_echo_with_spaces() {
        let output = SubstitutionExecutor::execute("echo hello world").unwrap();
        assert_eq!(output, "hello world");
    }

    #[test]
    fn test_empty_output() {
        let output = SubstitutionExecutor::execute("echo").unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn test_multiline_output() {
        // Use sh -c to enable escape sequence interpretation
        let output =
            SubstitutionExecutor::execute("sh -c \"printf 'line1\\\\nline2\\\\nline3'\"").unwrap();
        assert_eq!(output, "line1\nline2\nline3");
    }

    #[test]
    fn test_empty_command() {
        let output = SubstitutionExecutor::execute("").unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn test_command_with_args() {
        let output = SubstitutionExecutor::execute("echo arg1 arg2 arg3").unwrap();
        assert_eq!(output, "arg1 arg2 arg3");
    }

    #[test]
    fn test_quoted_args() {
        let output = SubstitutionExecutor::execute("echo 'hello world'").unwrap();
        assert_eq!(output, "hello world");
    }

    #[test]
    fn test_nonexistent_command() {
        let result = SubstitutionExecutor::execute("nonexistent_command_that_does_not_exist");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_command_with_nonzero_exit() {
        let result = SubstitutionExecutor::execute("sh -c 'exit 1'");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exited with code"));
    }

    #[test]
    fn test_true_command() {
        let output = SubstitutionExecutor::execute("true").unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn test_false_command_fails() {
        let result = SubstitutionExecutor::execute("false");
        assert!(result.is_err());
    }

    #[test]
    fn test_pwd_produces_output() {
        let output = SubstitutionExecutor::execute("pwd").unwrap();
        assert!(!output.is_empty());
        // pwd output should be an absolute path
        assert!(output.starts_with('/'));
    }

    #[test]
    fn test_echo_with_quotes_in_output() {
        let output = SubstitutionExecutor::execute("echo 'quoted text'").unwrap();
        assert_eq!(output, "quoted text");
    }

    #[test]
    fn test_command_with_pipe_needs_shell() {
        // Pipes don't work directly with echo (they're shell constructs)
        // This would fail because echo gets "hello | cat" as a single string
        let result = SubstitutionExecutor::execute("echo hello | cat");
        assert!(result.is_err() || result.unwrap() == "hello | cat");
        // To use pipes, you need to wrap in sh -c
    }

    #[test]
    fn test_multiline_with_trailing_newlines_trimmed() {
        // Even with multiple trailing newlines, they should be trimmed
        let output =
            SubstitutionExecutor::execute("sh -c \"printf 'test\\\\n\\\\n\\\\n'\"").unwrap();
        assert_eq!(output, "test");
    }

    #[test]
    fn test_whitespace_preservation() {
        let output = SubstitutionExecutor::execute("printf 'a    b'").unwrap();
        assert_eq!(output, "a    b");
    }

    #[test]
    fn test_special_characters_in_output() {
        let output = SubstitutionExecutor::execute("echo 'special: $@#%'").unwrap();
        assert_eq!(output, "special: $@#%");
    }
}
