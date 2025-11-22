//! Command execution implementation

use super::parser::parse_pipeline;
use super::pipeline::PipelineExecutor;
use crate::error::Result;

/// Simple command executor
///
/// Executes commands by spawning processes and waiting for completion.
///
/// # Current Features
///
/// - I/O redirections (>, >>, <)
/// - Pipelines via PipelineExecutor (single and multi-command)
/// - Command execution with proper exit codes
/// - Signal handling (FR-009) for pipeline processes
///
/// # Future Enhancements
///
/// Not yet implemented:
/// - Job control (bg, fg, jobs)
/// - Background execution (&)
/// - Combining redirections with pipelines
pub struct CommandExecutor {
    pipeline_executor: PipelineExecutor,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Self {
        Self {
            pipeline_executor: PipelineExecutor::new(),
        }
    }


    /// Execute a command line and return the exit code
    ///
    /// # Arguments
    /// * `line` - The command line to execute
    ///   - Single command: "ls -la"
    ///   - Pipeline: "ls | grep txt"
    ///   - Redirection: "ls > files.txt" (parsed but not yet executed with redirections)
    ///
    /// # Returns
    /// * `Ok(exit_code)` - The command's exit code (0 for success)
    /// * `Err(_)` - If the command could not be executed
    pub fn execute(&self, line: &str) -> Result<i32> {
        // Handle empty input
        if line.trim().is_empty() {
            tracing::trace!("Empty command line");
            return Ok(0);
        }

        // Parse command line into pipeline (handles quotes, pipes, and redirections)
        let pipeline = match parse_pipeline(line) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!(error = %e, "Command parsing failed");
                eprintln!("rush: parse error: {}", e);
                return Ok(1); // Parsing error, non-zero exit
            }
        };

        tracing::debug!(
            segments = pipeline.len(),
            raw_input = %pipeline.raw_input,
            "Executing command line"
        );

        // Execute the pipeline
        self.pipeline_executor.execute(&pipeline)
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_new() {
        let _executor = CommandExecutor::new();
    }

    #[test]
    fn test_executor_default() {
        let _executor = CommandExecutor::default();
    }

    #[test]
    fn test_execute_empty_command() {
        let executor = CommandExecutor::new();
        let result = executor.execute("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_echo() {
        let executor = CommandExecutor::new();
        let result = executor.execute("echo test");
        assert!(result.is_ok());
        // echo should succeed (exit code 0)
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_true() {
        let executor = CommandExecutor::new();
        let result = executor.execute("true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_false() {
        let executor = CommandExecutor::new();
        let result = executor.execute("false");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // false returns 1
    }

    #[test]
    fn test_execute_nonexistent_command() {
        let executor = CommandExecutor::new();
        let result = executor.execute("this_command_definitely_does_not_exist_12345");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 127); // Command not found
    }

    #[test]
    fn test_execute_with_args() {
        let executor = CommandExecutor::new();
        // Test command with arguments
        let result = executor.execute("printf hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_pwd() {
        let executor = CommandExecutor::new();
        let result = executor.execute("pwd");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_with_multiple_args() {
        let executor = CommandExecutor::new();
        let result = executor.execute("echo hello world test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_with_flags() {
        let executor = CommandExecutor::new();
        let result = executor.execute("ls -l -a");
        assert!(result.is_ok());
        // ls should succeed
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_with_special_chars_in_args() {
        let executor = CommandExecutor::new();
        let result = executor.execute("printf test123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_date() {
        let executor = CommandExecutor::new();
        let result = executor.execute("date");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_whoami() {
        let executor = CommandExecutor::new();
        let result = executor.execute("whoami");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_whitespace_command() {
        let executor = CommandExecutor::new();
        let result = executor.execute("   ");
        assert!(result.is_ok());
        // Empty/whitespace-only should return 0
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_executor_is_reusable() {
        let executor = CommandExecutor::new();

        // Execute multiple commands with same executor
        let result1 = executor.execute("true");
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), 0);

        let result2 = executor.execute("false");
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 1);

        let result3 = executor.execute("true");
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), 0);
    }
}
