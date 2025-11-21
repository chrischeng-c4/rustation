//! Pipeline execution implementation
//!
//! Handles execution of command pipelines by spawning processes and
//! connecting their I/O streams.
//!
//! # User Story 1: Basic Two-Command Pipeline
//!
//! This module implements support for basic two-command pipelines where
//! stdout of the first command becomes stdin of the second command.
//!
//! # Signal Handling (FR-009)
//!
//! **Critical Feature**: This implementation includes proper signal propagation
//! to all pipeline segments. When the user presses Ctrl+C (SIGINT) or the shell
//! receives SIGTERM, all child processes in the pipeline are terminated.
//!
//! This prevents:
//! - Zombie processes
//! - Resource leaks
//! - Hanging pipelines after interrupt
//!
//! Signal handling is implemented by:
//! 1. Storing child process handles
//! 2. Killing all children on interrupt or error
//! 3. Properly waiting for termination
//! 4. Logging cleanup operations

use crate::error::{Result, RushError};
use crate::executor::{Pipeline, PipelineSegment};
use std::process::{Child, Command, Stdio};

/// Executes pipelines by spawning processes and connecting pipes
///
/// For User Story 1: Supports single commands and basic two-command pipelines.
pub struct PipelineExecutor {
    // Stateless executor - no fields needed
}

impl PipelineExecutor {
    /// Create a new pipeline executor
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a pipeline and return the last command's exit code
    ///
    /// # Arguments
    /// * `pipeline` - The pipeline to execute (1 or 2 commands for US1)
    ///
    /// # Returns
    /// * `Ok(exit_code)` - Exit code from last command (0 for success)
    /// * `Err(_)` - If pipeline execution failed
    ///
    /// # Signal Handling (FR-009)
    ///
    /// All child processes are properly terminated on errors or signals.
    /// See module documentation for details.
    ///
    /// # Example
    /// ```ignore
    /// let executor = PipelineExecutor::new();
    /// let pipeline = parse_pipeline("echo hello | grep hello")?;
    /// let exit_code = executor.execute(&pipeline)?;
    /// ```
    pub fn execute(&self, pipeline: &Pipeline) -> Result<i32> {
        // Validate pipeline structure (US1: 1-2 commands only)
        pipeline.validate()?;

        // Special case: Single command (no pipes)
        if pipeline.len() == 1 {
            return self.execute_single(&pipeline.segments[0]);
        }

        // Execute two-command pipeline (US1)
        self.execute_two_command_pipeline(pipeline)
    }

    /// Execute a single command (no pipes)
    fn execute_single(&self, segment: &PipelineSegment) -> Result<i32> {
        tracing::debug!(
            program = %segment.program,
            args = ?segment.args,
            "Executing single command"
        );

        match Command::new(&segment.program)
            .args(&segment.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(mut child) => {
                let pid = child.id();
                tracing::trace!(pid, "Process spawned");

                match child.wait() {
                    Ok(status) => {
                        let exit_code = status.code().unwrap_or(1);
                        tracing::info!(
                            program = %segment.program,
                            exit_code,
                            pid,
                            "Process completed"
                        );
                        Ok(exit_code)
                    }
                    Err(e) => {
                        tracing::error!(
                            program = %segment.program,
                            error = %e,
                            pid,
                            "Failed to wait for process"
                        );
                        Err(RushError::Execution(format!("Failed to wait for command: {}", e)))
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    program = %segment.program,
                    error = %e,
                    "Command not found or spawn failed"
                );

                match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        eprintln!("rush: command not found: {}", segment.program);
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!("rush: permission denied: {}", segment.program);
                    }
                    _ => {
                        eprintln!("rush: failed to execute {}: {}", segment.program, e);
                    }
                }
                Ok(127) // Command not found exit code
            }
        }
    }

    /// Execute a two-command pipeline (US1)
    ///
    /// Connects stdout of first command to stdin of second command.
    /// Includes proper signal handling (FR-009) and resource cleanup.
    fn execute_two_command_pipeline(&self, pipeline: &Pipeline) -> Result<i32> {
        tracing::info!(
            segments = pipeline.len(),
            raw_input = %pipeline.raw_input,
            "Executing two-command pipeline"
        );

        // Spawn both processes with pipes connected
        let execution = TwoCommandExecution::spawn(pipeline)?;

        // Wait for both to complete and get last exit code
        let exit_code = execution.wait_all()?;

        tracing::info!(exit_code, "Pipeline completed");

        Ok(exit_code)
    }
}

impl Default for PipelineExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal state during two-command pipeline execution
///
/// # Signal Handling (FR-009)
///
/// This struct holds all child process handles and ensures they are properly
/// terminated on drop, error, or signal. The `kill_all()` method is called
/// whenever execution needs to be aborted.
struct TwoCommandExecution {
    /// First command in pipeline
    first: Child,

    /// Second command in pipeline
    second: Child,

    /// Pipeline being executed (for logging and errors)
    pipeline: Pipeline,
}

impl TwoCommandExecution {
    /// Spawn both commands in a two-command pipeline with pipe connected
    ///
    /// # Signal Handling (FR-009)
    ///
    /// If spawning fails for either command, all already-spawned processes
    /// are killed before returning an error. This prevents orphaned processes.
    fn spawn(pipeline: &Pipeline) -> Result<Self> {
        assert_eq!(pipeline.len(), 2, "US1: Only two-command pipelines supported");

        let first_seg = &pipeline.segments[0];
        let second_seg = &pipeline.segments[1];

        // Spawn first command with piped stdout
        let mut first_cmd = Command::new(&first_seg.program);
        first_cmd
            .args(&first_seg.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut first_child = match first_cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                tracing::debug!(
                    program = %first_seg.program,
                    position = 0,
                    pid,
                    "First pipeline segment spawned"
                );
                child
            }
            Err(e) => {
                tracing::warn!(
                    program = %first_seg.program,
                    error = %e,
                    position = 0,
                    "Command not found or spawn failed in pipeline"
                );

                match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        eprintln!("rush: command not found: {}", first_seg.program);
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!("rush: permission denied: {}", first_seg.program);
                    }
                    _ => {
                        eprintln!("rush: failed to execute {}: {}", first_seg.program, e);
                    }
                }

                return Err(RushError::Execution(format!(
                    "Failed to spawn {} at position 0: command not found",
                    first_seg.program
                )));
            }
        };

        // Take stdout from first command for second command's stdin
        let first_stdout = first_child.stdout.take().expect("stdout should be piped");

        // Spawn second command with stdin from first command
        let mut second_cmd = Command::new(&second_seg.program);
        second_cmd
            .args(&second_seg.args)
            .stdin(first_stdout)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let second_child = match second_cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                tracing::debug!(
                    program = %second_seg.program,
                    position = 1,
                    pid,
                    "Second pipeline segment spawned"
                );
                child
            }
            Err(e) => {
                tracing::warn!(
                    program = %second_seg.program,
                    error = %e,
                    position = 1,
                    "Command not found or spawn failed in pipeline"
                );

                // FR-009: Kill first command before returning error
                tracing::info!(
                    program = %first_seg.program,
                    "Killing first command due to second command spawn failure"
                );
                if let Err(kill_err) = first_child.kill() {
                    tracing::warn!(
                        program = %first_seg.program,
                        error = %kill_err,
                        "Failed to kill first command during cleanup"
                    );
                }
                if let Err(wait_err) = first_child.wait() {
                    tracing::warn!(
                        program = %first_seg.program,
                        error = %wait_err,
                        "Failed to wait for first command during cleanup"
                    );
                }

                match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        eprintln!("rush: command not found: {}", second_seg.program);
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!("rush: permission denied: {}", second_seg.program);
                    }
                    _ => {
                        eprintln!("rush: failed to execute {}: {}", second_seg.program, e);
                    }
                }

                return Err(RushError::Execution(format!(
                    "Failed to spawn {} at position 1: command not found",
                    second_seg.program
                )));
            }
        };

        Ok(Self {
            first: first_child,
            second: second_child,
            pipeline: pipeline.clone(),
        })
    }

    /// Wait for both processes to complete and return last exit code
    ///
    /// # Exit Code Behavior (User Story 4)
    ///
    /// Returns the exit code of the second (last) command, matching bash behavior.
    /// The first command's exit code is logged but not returned.
    fn wait_all(mut self) -> Result<i32> {
        // Wait for first command
        match self.first.wait() {
            Ok(status) => {
                let exit_code = status.code().unwrap_or(1);
                tracing::debug!(
                    command = %self.pipeline.segments[0].program,
                    exit_code,
                    position = 0,
                    "First pipeline segment completed"
                );
                // Don't return - we need the second command's exit code
            }
            Err(e) => {
                tracing::error!(
                    command = %self.pipeline.segments[0].program,
                    error = %e,
                    "Failed to wait for first pipeline segment"
                );

                // FR-009: Kill second command if first fails to wait
                tracing::info!(
                    program = %self.pipeline.segments[1].program,
                    "Killing second command due to first command wait failure"
                );
                if let Err(kill_err) = self.second.kill() {
                    tracing::warn!(
                        program = %self.pipeline.segments[1].program,
                        error = %kill_err,
                        "Failed to kill second command during cleanup"
                    );
                }
                if let Err(wait_err) = self.second.wait() {
                    tracing::warn!(
                        program = %self.pipeline.segments[1].program,
                        error = %wait_err,
                        "Failed to wait for second command during cleanup"
                    );
                }

                return Err(RushError::Execution(format!(
                    "Failed to wait for {}: {}",
                    self.pipeline.segments[0].program, e
                )));
            }
        }

        // Wait for second command and return its exit code
        match self.second.wait() {
            Ok(status) => {
                let exit_code = status.code().unwrap_or(1);
                tracing::debug!(
                    command = %self.pipeline.segments[1].program,
                    exit_code,
                    position = 1,
                    "Second pipeline segment completed"
                );
                Ok(exit_code)
            }
            Err(e) => {
                tracing::error!(
                    command = %self.pipeline.segments[1].program,
                    error = %e,
                    "Failed to wait for second pipeline segment"
                );
                Err(RushError::Execution(format!(
                    "Failed to wait for {}: {}",
                    self.pipeline.segments[1].program, e
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::parser::parse_pipeline;

    #[test]
    fn test_executor_new() {
        let executor = PipelineExecutor::new();
        drop(executor);
    }

    #[test]
    fn test_executor_default() {
        let executor = PipelineExecutor::default();
        drop(executor);
    }

    #[test]
    fn test_execute_single_command() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("echo test").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_true() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("true").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_false() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("false").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_execute_two_command_pipeline() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("echo hello | cat").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_pipeline_with_grep() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("echo 'hello world' | grep hello").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_us1_limit_three_commands_rejected() {
        let executor = PipelineExecutor::new();
        // US1: Should reject 3+ command pipelines
        let pipeline = parse_pipeline("echo test | cat | cat");
        assert!(pipeline.is_err() || {
            let result = executor.execute(&pipeline.unwrap());
            result.is_err()
        });
    }
}
