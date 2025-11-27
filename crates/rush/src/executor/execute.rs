//! Command execution implementation

use super::job::JobManager;
use super::parser::parse_pipeline;
use super::pipeline::PipelineExecutor;
use crate::error::Result;
use nix::unistd::Pid;

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
    job_manager: JobManager,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Self {
        Self { pipeline_executor: PipelineExecutor::new(), job_manager: JobManager::new() }
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
    pub fn execute(&mut self, line: &str) -> Result<i32> {
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

        // Check for built-ins (only if single command and not background)
        if pipeline.len() == 1 && !pipeline.background {
            let segment = &pipeline.segments[0];
            if let Some(result) =
                super::builtins::execute_builtin(self, &segment.program, &segment.args)
            {
                return result;
            }
        }

        tracing::debug!(
            segments = pipeline.len(),
            raw_input = %pipeline.raw_input,
            "Executing command line"
        );

        // Execute the pipeline
        let execution = match self.pipeline_executor.spawn(&pipeline) {
            Ok(execution) => execution,
            Err(_) => return Ok(127),
        };

        if pipeline.background {
            // Background execution
            let pids: Vec<Pid> = execution
                .pids()
                .into_iter()
                .map(|id| Pid::from_raw(id as i32))
                .collect();

            // Use the process group of the first process as the job's PGID
            // In a real shell, we would setpgid here, but for MVP we trust the OS/spawn
            let pgid = pids.first().copied().unwrap_or_else(|| Pid::from_raw(0));

            let job_id = self
                .job_manager
                .add_job(pgid, pipeline.raw_input.clone(), pids.clone());

            // Print job info: [1] 12345
            if let Some(last_pid) = pids.last() {
                println!("[{}] {}", job_id, last_pid);
            }

            Ok(0)
        } else {
            // Foreground execution
            execution.wait_all()
        }
    }

    /// Get mutable reference to job manager (for builtins)
    pub fn job_manager_mut(&mut self) -> &mut JobManager {
        &mut self.job_manager
    }

    /// Check for finished background jobs and print their status
    pub fn check_background_jobs(&mut self) {
        self.job_manager.update_status();
        let finished_jobs = self.job_manager.cleanup();

        for job in finished_jobs {
            println!("[{}] {} {}", job.id, job.status, job.command);
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
        let mut executor = CommandExecutor::new();
        let result = executor.execute("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_echo() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("echo test");
        assert!(result.is_ok());
        // echo should succeed (exit code 0)
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_true() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_false() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("false");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // false returns 1
    }

    #[test]
    fn test_execute_nonexistent_command() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("this_command_definitely_does_not_exist_12345");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 127); // Command not found
    }

    #[test]
    fn test_execute_with_args() {
        let mut executor = CommandExecutor::new();
        // Test command with arguments
        let result = executor.execute("printf hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_pwd() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("pwd");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_with_multiple_args() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("echo hello world test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_with_flags() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("ls -l -a");
        assert!(result.is_ok());
        // ls should succeed
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_with_special_chars_in_args() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("printf test123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_date() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("date");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_whoami() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("whoami");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_whitespace_command() {
        let mut executor = CommandExecutor::new();
        let result = executor.execute("   ");
        assert!(result.is_ok());
        // Empty/whitespace-only should return 0
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_executor_is_reusable() {
        let mut executor = CommandExecutor::new();

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
