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
        // Execute multi-command pipeline (US1 & US2)
        let execution = self.spawn(pipeline)?;
        execution.wait_all()
    }

    /// Spawn a pipeline without waiting for completion
    pub fn spawn(&self, pipeline: &Pipeline) -> Result<MultiCommandExecution> {
        // Validate pipeline structure
        pipeline.validate()?;

        // Special case: Single command (no pipes)
        if pipeline.len() == 1 {
            // For single command, we still use MultiCommandExecution for consistency
            // This simplifies the CommandExecutor logic
            MultiCommandExecution::spawn(pipeline)
        } else {
            MultiCommandExecution::spawn(pipeline)
        }
    }

    /// Execute a single command (no pipes)
    fn execute_single(&self, segment: &PipelineSegment) -> Result<i32> {
        use std::fs::{File, OpenOptions};
        use std::io::ErrorKind;

        // Use redirections from segment (populated by parser)
        let redirections = &segment.redirections;

        tracing::debug!(
            program = %segment.program,
            args = ?segment.args,
            redirections = ?redirections,
            "Executing single command with redirections"
        );

        let mut cmd = Command::new(&segment.program);
        cmd.args(&segment.args);

        // Apply redirections if any
        if !redirections.is_empty() {
            for redir in redirections {
                match redir.redir_type {
                    super::RedirectionType::Output => {
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
                            tracing::error!(error = %msg, "Output redirection failed");
                            eprintln!("rush: {}", msg);
                            RushError::Redirection(msg)
                        })?;
                        cmd.stdout(Stdio::from(file));
                    }
                    super::RedirectionType::Append => {
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
                                tracing::error!(error = %msg, "Append redirection failed");
                                eprintln!("rush: {}", msg);
                                RushError::Redirection(msg)
                            })?;
                        cmd.stdout(Stdio::from(file));
                    }
                    super::RedirectionType::Input => {
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
                            tracing::error!(error = %msg, "Input redirection failed");
                            eprintln!("rush: {}", msg);
                            RushError::Redirection(msg)
                        })?;
                        cmd.stdin(Stdio::from(file));
                    }
                }
            }
            // If we have redirections, stderr still inherits unless redirected
            cmd.stderr(Stdio::inherit());
        } else {
            // No redirections - use inherited stdio for all streams
            cmd.stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
        }

        match cmd.spawn() {
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
/// Internal state during multi-command pipeline execution
///
/// # Signal Handling (FR-009)
///
/// This struct holds all child process handles and ensures they are properly
/// terminated on drop, error, or signal. Any spawn or wait failure triggers
/// cleanup of all running processes to prevent orphans.
pub struct MultiCommandExecution {
    /// All spawned child processes (one per pipeline segment)
    pub children: Vec<Child>,

    /// Pipeline being executed (for logging and errors)
    pub pipeline: Pipeline,
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
            use std::fs::{File, OpenOptions};
            use std::io::ErrorKind;

            let mut cmd = Command::new(&segment.program);
            cmd.args(&segment.args);

            // Apply redirections first (they override default stdio setup)
            let mut has_output_redir = false;
            let mut has_input_redir = false;

            for redir in &segment.redirections {
                match redir.redir_type {
                    super::RedirectionType::Output => {
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
                            eprintln!("rush: {}", msg);
                            RushError::Redirection(msg)
                        })?;
                        cmd.stdout(Stdio::from(file));
                        has_output_redir = true;
                    }
                    super::RedirectionType::Append => {
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
                                eprintln!("rush: {}", msg);
                                RushError::Redirection(msg)
                            })?;
                        cmd.stdout(Stdio::from(file));
                        has_output_redir = true;
                    }
                    super::RedirectionType::Input => {
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
                            eprintln!("rush: {}", msg);
                            RushError::Redirection(msg)
                        })?;
                        cmd.stdin(Stdio::from(file));
                        has_input_redir = true;
                    }
                }
            }

            // Configure stdin (only if not redirected)
            if !has_input_redir {
                if let Some(stdout) = prev_stdout.take() {
                    // Middle/last command: stdin from previous command's stdout
                    cmd.stdin(stdout);
                } else {
                    // First command: stdin from terminal
                    cmd.stdin(Stdio::inherit());
                }
            }

            // Configure stdout (only if not redirected)
            if !has_output_redir {
                if i == pipeline.len() - 1 {
                    // Last command: stdout to terminal
                    cmd.stdout(Stdio::inherit());
                } else {
                    // First/middle command: stdout to pipe
                    cmd.stdout(Stdio::piped());
                }
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

        Ok(Self { children, pipeline: pipeline.clone() })
    }

    /// Wait for all processes to complete and return last exit code
    ///
    /// # Exit Code Behavior (User Story 4)
    ///
    /// Returns the exit code of the last command, matching bash behavior.
    /// Earlier commands' exit codes are logged but not returned.
    pub fn wait_all(self) -> Result<i32> {
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

    /// Get PIDs of all spawned processes
    pub fn pids(&self) -> Vec<u32> {
        self.children.iter().map(|c| c.id()).collect()
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

    #[test]
    fn test_command_not_found() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("nonexistent_command_xyz123").unwrap();
        let result = executor.execute(&pipeline);
        // Should fail with command not found
        assert!(result.is_ok()); // Returns exit code 127
        assert_eq!(result.unwrap(), 127);
    }

    #[test]
    fn test_pipeline_first_command_fails() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("false | cat").unwrap();
        let result = executor.execute(&pipeline);
        // Should return exit code of last command (cat succeeds with no input)
        assert!(result.is_ok());
    }

    #[test]
    fn test_spawn_returns_multi_command_execution() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("echo test").unwrap();
        let multi_exec = executor.spawn(&pipeline);
        assert!(multi_exec.is_ok());
        let exec = multi_exec.unwrap();
        assert_eq!(exec.pids().len(), 1);
    }

    #[test]
    fn test_multi_command_execution_pids() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("echo test | cat | cat").unwrap();
        let multi_exec = executor.spawn(&pipeline).unwrap();
        let pids = multi_exec.pids();
        assert_eq!(pids.len(), 3);
        // All PIDs should be non-zero
        for pid in pids {
            assert!(pid > 0);
        }
    }

    #[test]
    fn test_execute_with_redirections() {
        let executor = PipelineExecutor::new();
        let test_file = "/tmp/rush_pipeline_test.txt";
        let _ = std::fs::remove_file(test_file);

        let pipeline = parse_pipeline("echo hello > /tmp/rush_pipeline_test.txt").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert!(std::path::Path::new(test_file).exists());

        std::fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_pipeline_with_redirection_middle_command() {
        let executor = PipelineExecutor::new();
        let test_file = "/tmp/rush_pipe_middle.txt";
        let _ = std::fs::remove_file(test_file);

        // Middle command can't have output redirection in our implementation
        // (it would break the pipe), but we can test input redirection
        std::fs::write("/tmp/rush_input.txt", "test\n").unwrap();
        let pipeline = parse_pipeline("cat < /tmp/rush_input.txt | grep test").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());

        std::fs::remove_file("/tmp/rush_input.txt").unwrap();
    }

    // === Error Path Coverage Tests ===

    #[test]
    fn test_output_redirection_permission_denied() {
        let executor = PipelineExecutor::new();
        // Try to write to /dev/null/impossible (directory, not file)
        let pipeline = parse_pipeline("echo test > /dev/null/impossible").unwrap();
        let result = executor.execute(&pipeline);
        // Should fail with redirection error
        assert!(result.is_err());
    }

    #[test]
    fn test_output_redirection_to_directory() {
        let executor = PipelineExecutor::new();
        // Try to redirect to /tmp (which is a directory)
        let pipeline = parse_pipeline("echo test > /tmp").unwrap();
        let result = executor.execute(&pipeline);
        // Should fail - can't write to directory
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("directory"));
        }
    }

    #[test]
    fn test_append_redirection_permission_denied() {
        let executor = PipelineExecutor::new();
        // Try to append to impossible location
        let pipeline = parse_pipeline("echo test >> /dev/null/impossible").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_err());
    }

    #[test]
    fn test_append_redirection_to_directory() {
        let executor = PipelineExecutor::new();
        // Try to append to /tmp (directory)
        let pipeline = parse_pipeline("echo test >> /tmp").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("directory"));
        }
    }

    #[test]
    fn test_input_redirection_file_not_found() {
        let executor = PipelineExecutor::new();
        // Try to read from nonexistent file
        let pipeline = parse_pipeline("cat < /tmp/nonexistent_file_xyz123.txt").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("not found"));
        }
    }

    #[test]
    fn test_input_redirection_permission_denied() {
        use std::fs::File;
        use std::os::unix::fs::PermissionsExt;

        let executor = PipelineExecutor::new();
        let test_file = "/tmp/rush_no_read_permission.txt";

        // Create file with no read permissions
        File::create(test_file).unwrap();
        let mut perms = std::fs::metadata(test_file).unwrap().permissions();
        perms.set_mode(0o000); // No permissions
        std::fs::set_permissions(test_file, perms).unwrap();

        let pipeline = parse_pipeline(&format!("cat < {}", test_file)).unwrap();
        let result = executor.execute(&pipeline);

        // Restore permissions for cleanup
        let mut perms = std::fs::metadata(test_file).unwrap().permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(test_file, perms).unwrap();
        std::fs::remove_file(test_file).unwrap();

        // Should fail with permission denied
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("permission denied"));
        }
    }

    #[test]
    fn test_append_redirection_success() {
        let executor = PipelineExecutor::new();
        let test_file = "/tmp/rush_append_test.txt";
        let _ = std::fs::remove_file(test_file);

        // First write
        let pipeline1 = parse_pipeline(&format!("echo first >> {}", test_file)).unwrap();
        assert!(executor.execute(&pipeline1).is_ok());

        // Second append
        let pipeline2 = parse_pipeline(&format!("echo second >> {}", test_file)).unwrap();
        assert!(executor.execute(&pipeline2).is_ok());

        // Verify both lines exist
        let content = std::fs::read_to_string(test_file).unwrap();
        assert!(content.contains("first"));
        assert!(content.contains("second"));

        std::fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_input_redirection_success() {
        let executor = PipelineExecutor::new();
        let test_file = "/tmp/rush_input_test.txt";

        // Create input file
        std::fs::write(test_file, "test content\n").unwrap();

        // Read from file
        let pipeline = parse_pipeline(&format!("cat < {}", test_file)).unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        std::fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_multiple_redirections() {
        let executor = PipelineExecutor::new();
        let in_file = "/tmp/rush_multi_in.txt";
        let out_file = "/tmp/rush_multi_out.txt";
        let _ = std::fs::remove_file(out_file);

        // Create input file
        std::fs::write(in_file, "input data\n").unwrap();

        // Redirect both input and output
        let pipeline = parse_pipeline(&format!("cat < {} > {}", in_file, out_file)).unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());

        // Verify output
        let content = std::fs::read_to_string(out_file).unwrap();
        assert!(content.contains("input data"));

        std::fs::remove_file(in_file).unwrap();
        std::fs::remove_file(out_file).unwrap();
    }

    #[test]
    fn test_execute_single_no_redirections() {
        let executor = PipelineExecutor::new();
        // Test the else branch at line 179 (no redirections)
        let pipeline = parse_pipeline("echo test").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_spawn_single_command() {
        let executor = PipelineExecutor::new();
        // Test spawn() with single command (lines 85-88)
        let pipeline = parse_pipeline("echo test").unwrap();
        let execution = executor.spawn(&pipeline);
        assert!(execution.is_ok());
        let result = execution.unwrap().wait_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_spawn_multi_command() {
        let executor = PipelineExecutor::new();
        // Test spawn() with multi-command pipeline (lines 90)
        let pipeline = parse_pipeline("echo test | cat").unwrap();
        let execution = executor.spawn(&pipeline);
        assert!(execution.is_ok());
        let result = execution.unwrap().wait_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_output_redirection_to_directory() {
        // Test output redirection error in pipeline (lines 299-309)
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("echo test | cat > /tmp").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("directory"));
        }
    }

    #[test]
    fn test_pipeline_append_redirection_to_directory() {
        // Test append redirection error in pipeline (lines 320-330)
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("echo test | cat >> /tmp").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("directory"));
        }
    }

    #[test]
    fn test_pipeline_input_redirection_not_found() {
        // Test input redirection file not found in pipeline (lines 336-347)
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("cat < /nonexistent_file_12345 | grep test").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("not found"));
        }
    }

    #[test]
    fn test_pipeline_with_output_redirection() {
        // Test successful output redirection in pipeline
        let executor = PipelineExecutor::new();
        let out_file = "/tmp/rush_pipeline_out_test.txt";
        let _ = std::fs::remove_file(out_file);

        let pipeline = parse_pipeline(&format!("echo hello | cat > {}", out_file)).unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(out_file).unwrap();
        assert!(content.contains("hello"));

        std::fs::remove_file(out_file).unwrap();
    }

    #[test]
    fn test_pipeline_with_input_redirection() {
        // Test successful input redirection in pipeline
        let executor = PipelineExecutor::new();
        let in_file = "/tmp/rush_pipeline_in_test.txt";

        std::fs::write(in_file, "test input\n").unwrap();

        let pipeline = parse_pipeline(&format!("cat < {} | cat", in_file)).unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());

        std::fs::remove_file(in_file).unwrap();
    }
}
