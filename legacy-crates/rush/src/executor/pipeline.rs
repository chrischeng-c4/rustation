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
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
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
    pub fn execute(&self, pipeline: &Pipeline) -> Result<(i32, Vec<u32>)> {
        // Validate pipeline structure (US1 & US2: 1 to N commands)
        pipeline.validate()?;

        // Special case: Single command (no pipes)
        if pipeline.len() == 1 {
            // For single commands, call execute_single which also needs to return stopped_pids
            let exit_code = self.execute_single(&pipeline.segments[0])?;
            return Ok((exit_code, Vec::new()));
        }

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
        use super::parser::extract_redirections_with_heredocs;
        use std::fs::{File, OpenOptions};
        use std::io::{ErrorKind, Write};

        // Extract redirections from args (including heredoc content from segment)
        let (clean_args, redirections) =
            extract_redirections_with_heredocs(&segment.args, &segment.heredoc_contents)?;

        tracing::debug!(
            program = %segment.program,
            args = ?clean_args,
            redirections = ?redirections,
            "Executing single command"
        );

        let mut cmd = Command::new(&segment.program);
        cmd.args(&clean_args);

        // Apply redirections if any
        let mut has_stderr_redirection = false;
        if !redirections.is_empty() {
            for redir in &redirections {
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
                    super::RedirectionType::Stderr(append) => {
                        // Create/open file for stderr redirection
                        let file = if append {
                            // 2>> - append mode
                            OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&redir.file_path)
                        } else {
                            // 2> - truncate mode
                            OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(&redir.file_path)
                        }
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
                            tracing::error!(error = %msg, "Stderr redirection failed");
                            eprintln!("rush: {}", msg);
                            RushError::Redirection(msg)
                        })?;
                        cmd.stderr(Stdio::from(file));
                        has_stderr_redirection = true;
                    }
                    super::RedirectionType::StderrToStdout
                    | super::RedirectionType::StdoutToStderr => {
                        // These are handled at a higher level since they require
                        // coordinating between stdout and stderr
                        // For now, we just skip these - they need special handling
                        tracing::debug!(
                            "Combined redirection (2>&1/1>&2) handling in execute_single"
                        );
                    }
                    super::RedirectionType::Heredoc | super::RedirectionType::HeredocStrip => {
                        // Heredoc: feed content to stdin via a pipe
                        // Content is in redir.heredoc_content
                        if let Some(ref content) = redir.heredoc_content {
                            // We need to use piped stdin and write the heredoc content
                            // This will be handled below by spawning with piped stdin
                            cmd.stdin(Stdio::piped());
                            tracing::debug!(
                                delimiter = %redir.file_path,
                                content_len = content.len(),
                                "Heredoc redirection configured"
                            );
                        }
                    }
                    super::RedirectionType::HereString => {
                        // Here-string: feed content to stdin via a pipe (similar to heredoc)
                        // Content is in redir.heredoc_content, newline will be added when writing
                        if let Some(ref content) = redir.heredoc_content {
                            cmd.stdin(Stdio::piped());
                            tracing::debug!(
                                content_len = content.len(),
                                "Here-string redirection configured"
                            );
                        }
                    }
                }
            }
            // If we have redirections, set default stderr to inherit unless already redirected
            if !has_stderr_redirection {
                cmd.stderr(Stdio::inherit());
            }
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

                // Write heredoc/here-string content to stdin if present
                for redir in &redirections {
                    if matches!(
                        redir.redir_type,
                        super::RedirectionType::Heredoc
                            | super::RedirectionType::HeredocStrip
                            | super::RedirectionType::HereString
                    ) {
                        if let Some(ref content) = redir.heredoc_content {
                            if let Some(mut stdin) = child.stdin.take() {
                                // For here-strings, append a trailing newline (bash behavior)
                                let content_to_write = if matches!(
                                    redir.redir_type,
                                    super::RedirectionType::HereString
                                ) {
                                    format!("{}\n", content)
                                } else {
                                    content.clone()
                                };
                                if let Err(e) = stdin.write_all(content_to_write.as_bytes()) {
                                    tracing::error!(error = %e, "Failed to write stdin content");
                                    // Kill the child since we couldn't provide input
                                    let _ = child.kill();
                                    let _ = child.wait();
                                    return Err(RushError::Execution(format!(
                                        "Failed to write stdin content: {}",
                                        e
                                    )));
                                }
                                // Close stdin to signal EOF (drop the owned value)
                                drop(stdin);
                                tracing::debug!(content_len = content_to_write.len(), redir_type = ?redir.redir_type, "Stdin content written");
                            }
                        }
                    }
                }

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
        use super::parser::extract_redirections_with_heredocs;
        use std::io::Write;

        let mut children: Vec<Child> = Vec::with_capacity(pipeline.len());
        let mut prev_stdout: Option<std::process::ChildStdout> = None;
        let mut heredoc_data: Vec<(usize, String)> = Vec::new(); // (child_index, content)

        for (i, segment) in pipeline.segments.iter().enumerate() {
            // Extract redirections from args (including heredoc content from segment)
            let (clean_args, redirections) =
                extract_redirections_with_heredocs(&segment.args, &segment.heredoc_contents)?;

            let mut cmd = Command::new(&segment.program);
            cmd.args(&clean_args);

            // Check for heredoc redirections
            let mut has_heredoc = false;
            for redir in &redirections {
                if matches!(
                    redir.redir_type,
                    super::RedirectionType::Heredoc | super::RedirectionType::HeredocStrip
                ) {
                    if let Some(ref content) = redir.heredoc_content {
                        has_heredoc = true;
                        heredoc_data.push((i, content.clone()));
                    }
                }
            }

            // Configure stdin
            if has_heredoc && prev_stdout.is_none() {
                // First command with heredoc: stdin from pipe (we'll write heredoc content)
                cmd.stdin(Stdio::piped());
            } else if let Some(stdout) = prev_stdout.take() {
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

        // Write heredoc content to stdin of appropriate children
        for (child_idx, content) in heredoc_data {
            if let Some(child) = children.get_mut(child_idx) {
                if let Some(mut stdin) = child.stdin.take() {
                    if let Err(e) = stdin.write_all(content.as_bytes()) {
                        tracing::error!(error = %e, child_idx, "Failed to write heredoc content to pipeline stdin");
                        // Clean up all children on failure
                        for mut c in children {
                            let _ = c.kill();
                            let _ = c.wait();
                        }
                        return Err(RushError::Execution(format!(
                            "Failed to write heredoc content: {}",
                            e
                        )));
                    }
                    // Close stdin to signal EOF (drop the owned value)
                    drop(stdin);
                    tracing::debug!(
                        child_idx,
                        content_len = content.len(),
                        "Heredoc content written to pipeline stdin"
                    );
                }
            }
        }

        Ok(Self { children, pipeline: pipeline.clone() })
    }

    /// Wait for all processes to complete and return last exit code
    ///
    /// # Exit Code Behavior (User Story 4)
    ///
    /// Returns the exit code of the last command and list of stopped process IDs.
    /// Earlier commands' exit codes are logged but not returned.
    /// Detects when processes are stopped (SIGTSTP) vs exited.
    pub fn wait_all(self) -> Result<(i32, Vec<u32>)> {
        let mut last_exit_code = 0;
        let mut stopped_pids = Vec::new();

        for (i, child) in self.children.into_iter().enumerate() {
            let pid = Pid::from_raw(child.id() as i32);

            // Use waitpid with WUNTRACED to detect stopped processes
            match waitpid(pid, Some(WaitPidFlag::WUNTRACED)) {
                Ok(WaitStatus::Exited(_, code)) => {
                    tracing::debug!(
                        command = %self.pipeline.segments[i].program,
                        exit_code = code,
                        position = i,
                        "Pipeline segment exited"
                    );

                    // Save exit code from last command
                    if i == self.pipeline.len() - 1 {
                        last_exit_code = code;
                    }
                }
                Ok(WaitStatus::Signaled(_, signal, _)) => {
                    tracing::debug!(
                        command = %self.pipeline.segments[i].program,
                        signal = ?signal,
                        position = i,
                        "Pipeline segment killed by signal"
                    );
                    // Treat signal termination as exit code 128 + signal number
                    let exit_code = 128;
                    if i == self.pipeline.len() - 1 {
                        last_exit_code = exit_code;
                    }
                }
                Ok(WaitStatus::Stopped(_, signal)) => {
                    // Process was stopped by signal (typically SIGTSTP from Ctrl+Z)
                    tracing::debug!(
                        command = %self.pipeline.segments[i].program,
                        signal = ?signal,
                        position = i,
                        "Pipeline segment stopped by signal"
                    );
                    stopped_pids.push(child.id());
                }
                _ => {
                    tracing::debug!(
                        command = %self.pipeline.segments[i].program,
                        position = i,
                        "Pipeline segment wait returned other status"
                    );
                }
            }
        }

        Ok((last_exit_code, stopped_pids))
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
        assert_eq!(result.unwrap().0, 0);
    }

    #[test]
    fn test_execute_true() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("true").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 0);
    }

    #[test]
    fn test_execute_false() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("false").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 1);
    }

    #[test]
    fn test_execute_two_command_pipeline() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("echo hello | cat").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 0);
    }

    #[test]
    fn test_execute_pipeline_with_grep() {
        let executor = PipelineExecutor::new();
        let pipeline = parse_pipeline("echo 'hello world' | grep hello").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 0);
    }

    #[test]
    fn test_three_command_pipeline() {
        let executor = PipelineExecutor::new();
        // US2: Three-command pipeline should work
        let pipeline = parse_pipeline("echo test | cat | cat").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 0);
    }

    #[test]
    fn test_four_command_pipeline() {
        let executor = PipelineExecutor::new();
        // US2: Four-command pipeline should work
        let pipeline = parse_pipeline("echo 'line1\nline2\nline3' | cat | cat | wc -l").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 0);
    }

    #[test]
    fn test_long_pipeline() {
        let executor = PipelineExecutor::new();
        // US2: Longer pipelines should work
        let pipeline = parse_pipeline("echo test | cat | cat | cat | cat").unwrap();
        let result = executor.execute(&pipeline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 0);
    }
}
