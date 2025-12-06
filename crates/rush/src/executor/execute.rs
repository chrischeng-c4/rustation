//! Command execution implementation

use super::aliases::AliasManager;
use super::expansion::expand_variables;
use super::glob::glob_expand;
use super::job::JobManager;
use super::parser::parse_pipeline;
use super::pipeline::PipelineExecutor;
use super::variables::VariableManager;
use crate::error::Result;
use nix::unistd::{setpgid, Pid};

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
/// - Environment variables
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
    variable_manager: VariableManager,
    alias_manager: AliasManager,
    last_exit_code: i32,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Self {
        Self {
            pipeline_executor: PipelineExecutor::new(),
            job_manager: JobManager::new(),
            variable_manager: VariableManager::new(),
            alias_manager: AliasManager::new(),
            last_exit_code: 0,
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
    pub fn execute(&mut self, line: &str) -> Result<i32> {
        // Handle empty input
        if line.trim().is_empty() {
            tracing::trace!("Empty command line");
            return Ok(0);
        }

        // Check if this is an if statement (before alias/variable expansion)
        let trimmed = line.trim();
        if trimmed.starts_with("if") && (trimmed.len() == 2 || trimmed.chars().nth(2).map_or(false, |c| c.is_whitespace())) {
            tracing::debug!("Detected if statement");
            return self.execute_if_statement(trimmed);
        }

        // Check if this is a for loop (before alias/variable expansion)
        if trimmed.starts_with("for") && (trimmed.len() == 3 || trimmed.chars().nth(3).map_or(false, |c| c.is_whitespace())) {
            tracing::debug!("Detected for loop");
            return self.execute_for_loop(trimmed);
        }

        // Check if this is a while loop (before alias/variable expansion)
        if trimmed.starts_with("while") && (trimmed.len() == 5 || trimmed.chars().nth(5).map_or(false, |c| c.is_whitespace())) {
            tracing::debug!("Detected while loop");
            return self.execute_while_loop(trimmed);
        }

        // Check if this is an until loop (before alias/variable expansion)
        if trimmed.starts_with("until") && (trimmed.len() == 5 || trimmed.chars().nth(5).map_or(false, |c| c.is_whitespace())) {
            tracing::debug!("Detected until loop");
            return self.execute_until_loop(trimmed);
        }

        // Check if this is a case statement (before alias/variable expansion)
        if trimmed.starts_with("case") && (trimmed.len() == 4 || trimmed.chars().nth(4).map_or(false, |c| c.is_whitespace())) {
            tracing::debug!("Detected case statement");
            return self.execute_case_statement(trimmed);
        }

        // Expand aliases first (before variable expansion)
        let aliased_line = self.alias_manager.expand(line);

        // Expand variables in the command line
        let expanded_line = expand_variables(&aliased_line, self);

        // Expand glob patterns (*, ?, [abc]) in arguments
        let globbed_line = glob_expand(&expanded_line)?;

        // Parse command line into pipeline (handles quotes, pipes, and redirections)
        let pipeline = match parse_pipeline(&globbed_line) {
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
                // Store exit code for $? expansion
                let exit_code = result?;
                self.last_exit_code = exit_code;
                return Ok(exit_code);
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
            Err(_) => {
                self.last_exit_code = 127;
                return Ok(127);
            }
        };

        let exit_code = if pipeline.background {
            // Background execution
            let pids: Vec<Pid> = execution
                .pids()
                .into_iter()
                .map(|id| Pid::from_raw(id as i32))
                .collect();

            // Create a new process group for the background job (Task 3.2)
            // This ensures signals (like SIGTSTP) reach all processes in the job
            let pgid = if let Some(&first_pid) = pids.first() {
                // Create new process group with first process as group leader
                // Allow this to fail gracefully - may have already been set by OS
                let _ = setpgid(first_pid, first_pid);

                // Add other processes to the same process group
                for &pid in pids.iter().skip(1) {
                    let _ = setpgid(pid, first_pid);
                }

                first_pid
            } else {
                Pid::from_raw(0)
            };

            let job_id = self
                .job_manager
                .add_job(pgid, pipeline.raw_input.clone(), pids.clone());

            // Print job info: [1] 12345
            if let Some(last_pid) = pids.last() {
                println!("[{}] {}", job_id, last_pid);
            }

            0
        } else {
            // Foreground execution (handles Ctrl+Z)
            let (exit_code, stopped_pids) = execution.wait_all()?;

            // Check if any process was stopped (Ctrl+Z)
            if !stopped_pids.is_empty() {
                // Convert stopped foreground process to background job
                let pids: Vec<Pid> = stopped_pids
                    .iter()
                    .map(|&id| Pid::from_raw(id as i32))
                    .collect();

                // Create a process group for the stopped job (if not already)
                let pgid = if let Some(&first_pid) = pids.first() {
                    // Allow this to fail gracefully
                    let _ = setpgid(first_pid, first_pid);

                    // Add other processes to the same process group
                    for &pid in pids.iter().skip(1) {
                        let _ = setpgid(pid, first_pid);
                    }

                    first_pid
                } else {
                    Pid::from_raw(0)
                };

                let job_id =
                    self.job_manager
                        .add_job(pgid, pipeline.raw_input.clone(), pids.clone());

                // Set job status to Stopped
                if let Some(job) = self.job_manager_mut().get_job_mut(job_id) {
                    job.status = crate::executor::job::JobStatus::Stopped;
                }

                // Print notification
                println!("\n[{}] Stopped    {}", job_id, pipeline.raw_input);

                // Don't return the exit code - the process is stopped, not finished
                0
            } else {
                exit_code
            }
        };

        self.last_exit_code = exit_code;
        Ok(exit_code)
    }

    /// Execute an if statement
    /// This is called when a line starts with the "if" keyword
    fn execute_if_statement(&mut self, line: &str) -> Result<i32> {
        use super::conditional;

        // Parse the if statement
        let if_block = match conditional::parse_if_clause(line) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!(error = %e, "If statement parsing failed");
                eprintln!("rush: {}", e);
                return Ok(1);
            }
        };

        // Execute the if block
        let exit_code = conditional::execute_if_block(&if_block, self)?;
        self.last_exit_code = exit_code;
        Ok(exit_code)
    }

    /// Execute a for loop
    /// This is called when a line starts with the "for" keyword
    fn execute_for_loop(&mut self, line: &str) -> Result<i32> {
        use super::for_loop;

        // Parse the for loop
        let for_loop = match for_loop::parse_for_loop(line) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!(error = %e, "For loop parsing failed");
                eprintln!("rush: {}", e);
                return Ok(1);
            }
        };

        // Execute the for loop
        let exit_code = for_loop::execute_for_loop(&for_loop, self)?;
        self.last_exit_code = exit_code;
        Ok(exit_code)
    }

    fn execute_while_loop(&mut self, line: &str) -> Result<i32> {
        use super::while_loop;

        // Parse the while loop
        let while_loop = match while_loop::parse_while_loop(line) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!(error = %e, "While loop parsing failed");
                eprintln!("rush: {}", e);
                return Ok(1);
            }
        };

        // Execute the while loop
        let exit_code = while_loop::execute_while_loop(&while_loop, self)?;
        self.last_exit_code = exit_code;
        Ok(exit_code)
    }

    fn execute_until_loop(&mut self, line: &str) -> Result<i32> {
        use super::while_loop;

        // Parse the until loop
        let until_loop = match while_loop::parse_until_loop(line) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!(error = %e, "Until loop parsing failed");
                eprintln!("rush: {}", e);
                return Ok(1);
            }
        };

        // Execute the until loop
        let exit_code = while_loop::execute_until_loop(&until_loop, self)?;
        self.last_exit_code = exit_code;
        Ok(exit_code)
    }

    fn execute_case_statement(&mut self, line: &str) -> Result<i32> {
        use super::case_statement;

        // Parse the case statement
        let case_stmt = match case_statement::parse_case_statement(line) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!(error = %e, "Case statement parsing failed");
                eprintln!("rush: {}", e);
                return Ok(1);
            }
        };

        // Execute the case statement
        let exit_code = case_statement::execute_case_statement(&case_stmt, self)?;
        self.last_exit_code = exit_code;
        Ok(exit_code)
    }

    /// Get mutable reference to job manager (for builtins)
    pub fn job_manager_mut(&mut self) -> &mut JobManager {
        &mut self.job_manager
    }

    /// Get mutable reference to variable manager (for builtins)
    pub fn variable_manager_mut(&mut self) -> &mut VariableManager {
        &mut self.variable_manager
    }

    /// Get reference to variable manager
    pub fn variable_manager(&self) -> &VariableManager {
        &self.variable_manager
    }

    /// Set the last exit code (for $? expansion)
    pub fn set_last_exit_code(&mut self, code: i32) {
        self.last_exit_code = code;
    }

    /// Get the last exit code
    pub fn last_exit_code(&self) -> i32 {
        self.last_exit_code
    }

    /// Get mutable reference to alias manager (for builtins)
    pub fn alias_manager_mut(&mut self) -> &mut AliasManager {
        &mut self.alias_manager
    }

    /// Get reference to alias manager
    pub fn alias_manager(&self) -> &AliasManager {
        &self.alias_manager
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
