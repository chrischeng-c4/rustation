//! Pipeline execution implementation
//!
//! Handles execution of command pipelines by spawning processes and
//! connecting their I/O streams.

use crate::error::{Result, RushError};
use crate::executor::{Pipeline, PipelineSegment};
use std::process::{Child, Command, Stdio};

/// Executes pipelines by spawning processes and connecting pipes
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
    /// * `pipeline` - The pipeline to execute
    ///
    /// # Returns
    /// * `Ok(exit_code)` - Exit code from last command (0 for success)
    /// * `Err(_)` - If pipeline execution failed
    ///
    /// # Example
    /// ```ignore
    /// let executor = PipelineExecutor::new();
    /// let pipeline = parse_pipeline("ls | grep txt")?;
    /// let exit_code = executor.execute(&pipeline)?;
    /// ```
    pub fn execute(&self, pipeline: &Pipeline) -> Result<i32> {
        // Validate pipeline structure
        pipeline.validate()?;

        // Special case: Single command (no pipes)
        if pipeline.len() == 1 {
            return self.execute_single(&pipeline.segments[0]);
        }

        // Execute multi-command pipeline
        self.execute_pipeline(pipeline)
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

    /// Execute a multi-command pipeline
    fn execute_pipeline(&self, pipeline: &Pipeline) -> Result<i32> {
        tracing::info!(
            segments = pipeline.len(),
            raw_input = %pipeline.raw_input,
            "Executing pipeline"
        );

        // Spawn all processes with pipes connected
        let execution = PipelineExecution::spawn(pipeline)?;

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

/// Internal state during pipeline execution
struct PipelineExecution {
    /// Spawned child processes (one per segment)
    children: Vec<Child>,

    /// Pipeline being executed (for logging and errors)
    pipeline: Pipeline,
}

impl PipelineExecution {
    /// Spawn all commands in the pipeline with pipes connected
    fn spawn(pipeline: &Pipeline) -> Result<Self> {
        let mut children: Vec<Child> = Vec::with_capacity(pipeline.len());
        let mut prev_stdout: Option<std::process::ChildStdout> = None;

        for (i, segment) in pipeline.segments.iter().enumerate() {
            let mut command = Command::new(&segment.program);
            command.args(&segment.args);

            // First command: stdin from terminal
            // Middle/last commands: stdin from previous command's stdout
            if let Some(stdout) = prev_stdout.take() {
                command.stdin(stdout);
            } else {
                command.stdin(Stdio::inherit());
            }

            // Last command: stdout to terminal
            // Other commands: stdout to pipe
            if i == pipeline.len() - 1 {
                command.stdout(Stdio::inherit());
            } else {
                command.stdout(Stdio::piped());
            }

            // All commands: stderr to terminal
            command.stderr(Stdio::inherit());

            // Spawn process
            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(e) => {
                    tracing::warn!(
                        program = %segment.program,
                        error = %e,
                        position = i,
                        "Command not found or spawn failed in pipeline"
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

                    // For pipelines, if any command fails to spawn, we need to kill
                    // already-spawned children and return error code
                    // Kill any already-spawned children
                    for mut child in children {
                        let _ = child.kill();
                        let _ = child.wait();
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

            // Save stdout for next command
            if i < pipeline.len() - 1 {
                prev_stdout = child.stdout.take();
            }

            children.push(child);
        }

        Ok(Self { children, pipeline: pipeline.clone() })
    }

    /// Wait for all processes to complete and return last exit code
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
}
