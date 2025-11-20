//! Command execution implementation

use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::process::{Command as StdCommand, Stdio};

use super::parser::parse_command_with_redirections;
use super::{Redirection, RedirectionType};
use crate::error::{Result, RushError};

/// Simple command executor
///
/// Executes commands by spawning processes and waiting for completion.
/// For MVP, this handles basic command execution without:
/// - Pipes
/// - Redirections
/// - Job control
/// - Background execution
///
/// These features will be added in later phases.
pub struct CommandExecutor;

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Self {
        Self
    }

    /// Apply redirections to a command before spawning
    fn apply_redirections(&self, cmd: &mut StdCommand, redirections: &[Redirection]) -> Result<()> {
        for redir in redirections {
            match redir.redir_type {
                RedirectionType::Output => {
                    // Create/truncate file for output redirection
                    let file = File::create(&redir.file_path).map_err(|e| {
                        let msg = match e.kind() {
                            ErrorKind::PermissionDenied => {
                                format!("{}: permission denied", redir.file_path)
                            }
                            ErrorKind::IsADirectory => {
                                format!("{}: is a directory", redir.file_path)
                            }
                            _ => format!("{}: {}", redir.file_path, e),
                        };
                        RushError::Redirection(msg)
                    })?;
                    cmd.stdout(Stdio::from(file));
                }
                RedirectionType::Append => {
                    // Open file in append mode
                    let file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&redir.file_path)
                        .map_err(|e| {
                            let msg = match e.kind() {
                                ErrorKind::PermissionDenied => {
                                    format!("{}: permission denied", redir.file_path)
                                }
                                ErrorKind::IsADirectory => {
                                    format!("{}: is a directory", redir.file_path)
                                }
                                _ => format!("{}: {}", redir.file_path, e),
                            };
                            RushError::Redirection(msg)
                        })?;
                    cmd.stdout(Stdio::from(file));
                }
                RedirectionType::Input => {
                    // Open file for input redirection
                    let file = File::open(&redir.file_path).map_err(|e| {
                        let msg = match e.kind() {
                            ErrorKind::NotFound => {
                                format!("{}: file not found", redir.file_path)
                            }
                            ErrorKind::PermissionDenied => {
                                format!("{}: permission denied", redir.file_path)
                            }
                            _ => format!("{}: {}", redir.file_path, e),
                        };
                        RushError::Redirection(msg)
                    })?;
                    cmd.stdin(Stdio::from(file));
                }
            }
        }
        Ok(())
    }

    /// Execute a command line and return the exit code
    ///
    /// # Arguments
    /// * `line` - The command line to execute (e.g., "ls -la" or "ls > files.txt")
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

        // Parse command line into program, arguments, and redirections
        let (program, args, redirections) = match parse_command_with_redirections(line) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!(error = %e, "Command parsing failed");
                eprintln!("rush: parse error: {}", e);
                return Ok(1); // Parsing error, non-zero exit
            }
        };

        tracing::debug!(
            program = %program,
            args = ?args,
            redirections = ?redirections,
            "Executing command"
        );

        // Build command
        let mut cmd = StdCommand::new(&program);
        cmd.args(&args);

        // Apply redirections if any
        if !redirections.is_empty() {
            if let Err(e) = self.apply_redirections(&mut cmd, &redirections) {
                tracing::error!(error = %e, "Redirection failed");
                eprintln!("rush: {}", e);
                return Ok(1);
            }
        } else {
            // No redirections - use inherited stdio
            cmd.stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
        }

        // Note: stderr always inherits unless explicitly redirected
        cmd.stderr(Stdio::inherit());

        // Execute the command
        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                tracing::trace!(pid, "Process spawned");

                // Wait for command to complete
                match child.wait() {
                    Ok(status) => {
                        let exit_code = status.code().unwrap_or(1);
                        tracing::info!(
                            program = %program,
                            exit_code,
                            pid,
                            "Process completed"
                        );
                        Ok(exit_code)
                    }
                    Err(e) => {
                        tracing::error!(
                            program = %program,
                            error = %e,
                            pid,
                            "Failed to wait for process"
                        );
                        Err(RushError::Execution(format!("Failed to wait for command: {}", e)))
                    }
                }
            }
            Err(e) => {
                // Command not found or execution failed
                tracing::warn!(
                    program = %program,
                    error = %e,
                    "Command not found or spawn failed"
                );

                // Provide helpful error message based on error kind
                match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        eprintln!("rush: command not found: {}", program);
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!("rush: permission denied: {}", program);
                    }
                    _ => {
                        eprintln!("rush: failed to execute {}: {}", program, e);
                    }
                }
                Ok(127) // Command not found exit code
            }
        }
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
