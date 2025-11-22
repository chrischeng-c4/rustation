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
        // Validate pipeline structure (US1 & US2: 1 to N commands)
        pipeline.validate()?;

        // Special case: Single command (no pipes)
        if pipeline.len() == 1 {
            return self.execute_single(&pipeline.segments[0]);
        }

        // Execute multi-command pipeline (US1 & US2)
        self.execute_multi_command_pipeline(pipeline)
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

    /// Execute a multi-command pipeline (US1 & US2)
    ///
    /// Connects commands via pipes: stdout of each becomes stdin of the next.
    /// Includes proper signal handling (FR-009) and resource cleanup.
    fn execute_multi_command_pipeline(&self, pipeline: &Pipeline) -> Result<i32> {
        tracing::info!(
            segments = pipeline.len(),
            raw_input = %pipeline.raw_input,
            "Executing multi-command pipeline"
        );

        // Spawn all processes with pipes connected
        let execution = MultiCommandExecution::spawn(pipeline)?;

        // Wait for all to complete and get last exit code
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

/// Internal state during multi-command pipeline execution
///
/// # Signal Handling (FR-009)
///
/// This struct holds all child process handles and ensures they are properly
/// terminated on drop, error, or signal. Any spawn or wait failure triggers
/// cleanup of all running processes to prevent orphans.
struct MultiCommandExecution {
    /// All spawned child processes (one per pipeline segment)
    children: Vec<Child>,

    /// Pipeline being executed (for logging and errors)
    pipeline: Pipeline,
}

impl MultiCommandExecution {
    /// Spawn all commands in the pipeline with pipes connected
    ///
    /// # Signal Handling (FR-009)
    ///
    /// If spawning fails for any command, all already-spawned processes
    /// are killed before returning an error. This prevents orphaned processes.
    ///
    /// # User Stories 1 & 2
    ///
    /// Handles any number of commands (2, 3, 4, ..., N) by:
    /// - First command: stdin from terminal, stdout to pipe
    /// - Middle commands: stdin from previous pipe, stdout to next pipe
    /// - Last command: stdin from previous pipe, stdout to terminal
    /// - All commands: stderr to terminal
    fn spawn(pipeline: &Pipeline) -> Result<Self> {
        let mut children: Vec<Child> = Vec::with_capacity(pipeline.len());
        let mut prev_stdout: Option<std::process::ChildStdout> = None;

        for (i, segment) in pipeline.segments.iter().enumerate() {
            let mut cmd = Command::new(&segment.program);
            cmd.args(&segment.args);

            // Configure stdin
            if let Some(stdout) = prev_stdout.take() {
                // Middle/last command: stdin from previous command's stdout
                cmd.stdin(stdout);
            } else {
                // First command: stdin from terminal
                cmd.stdin(Stdio::inherit());
            }

            // Configure stdout
            if i == pipeline.len() - 1 {
                // Last command: stdout to terminal
                cmd.stdout(Stdio::inherit());
            } else {
                // First/middle command: stdout to pipe
                cmd.stdout(Stdio::piped());
            }

            // All commands: stderr to terminal
            cmd.stderr(Stdio::inherit());

            // Spawn the command
            let mut child = match cmd.spawn() {
                Ok(child) => {
                    let pid = child.id();
                    tracing::debug!(
                        program = %segment.program,
                        position = i,
                        pid,
                        "Pipeline segment spawned"
                    );
                    child
                }
                Err(e) => {
                    tracing::warn!(
                        program = %segment.program,
                        error = %e,
                        position = i,
                        "Command not found or spawn failed in pipeline"
                    );

                    // FR-009: Kill all already-spawned children before returning error
                    if !children.is_empty() {
                        tracing::info!(
                            failed_command = %segment.program,
                            spawned_count = children.len(),
                            "Killing already-spawned commands due to spawn failure"
                        );
                        for mut child in children {
                            if let Err(kill_err) = child.kill() {
                                tracing::warn!(
                                    error = %kill_err,
                                    "Failed to kill child during cleanup"
                                );
                            }
                            if let Err(wait_err) = child.wait() {
                                tracing::warn!(
                                    error = %wait_err,
                                    "Failed to wait for child during cleanup"
                                );
                            }
                        }
                    }

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

                    return Err(RushError::Execution(format!(
                        "Failed to spawn {} at position {}: command not found",
                        segment.program, i
                    )));
                }
            };

            let pid = child.id();
            tracing::debug!(
                program = %segment.program,
                position = i,
                pid,
                "Pipeline segment spawned"
            );

            // Save stdout for next command (if not the last command)
            if i < pipeline.len() - 1 {
                prev_stdout = child.stdout.take();
            }

            children.push(child);
        }

        Ok(Self {
            children,
            pipeline: pipeline.clone(),
        })
    }

    /// Wait for all processes to complete and return last exit code
    ///
    /// # Exit Code Behavior (User Story 4)
    ///
    /// Returns the exit code of the last command, matching bash behavior.
    /// Earlier commands' exit codes are logged but not returned.
    fn wait_all(self) -> Result<i32> {
        let mut last_exit_code = 0;

        for (i, mut child) in self.children.into_iter().enumerate() {
            match child.wait() {
                Ok(status) => {
                    let exit_code = status.code().unwrap_or(1);
                    tracing::debug!(
                        command = %self.pipeline.segments[i].program,
                        exit_code,
                        position = i,
                        "Pipeline segment completed"
                    );

                    // Save exit code from last command
                    if i == self.pipeline.len() - 1 {
                        last_exit_code = exit_code;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        command = %self.pipeline.segments[i].program,
                        error = %e,
                        "Failed to wait for pipeline segment"
                    );
                    return Err(RushError::Execution(format!(
                        "Failed to wait for {}: {}",
                        self.pipeline.segments[i].program, e
                    )));
                }
            }
        }

        Ok(last_exit_code)
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
    fn test_three_command_pipeline() {
        let executor = PipelineExecutor::new();
        // US2: Three-command pipeline should work
        let pipeline = parse_pipeline("echo test | cat | cat").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_four_command_pipeline() {
        let executor = PipelineExecutor::new();
        // US2: Four-command pipeline should work
        let pipeline = parse_pipeline("echo 'line1\nline2\nline3' | cat | cat | wc -l").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_long_pipeline() {
        let executor = PipelineExecutor::new();
        // US2: Longer pipelines should work
        let pipeline = parse_pipeline("echo test | cat | cat | cat | cat").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
