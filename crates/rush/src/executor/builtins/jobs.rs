//! 'jobs' built-in command
//!
//! Lists active jobs with their ID, status, and command string.

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the 'jobs' command
pub fn execute(executor: &mut CommandExecutor, _args: &[String]) -> Result<i32> {
    // Update status first to ensure we show current state
    executor.check_background_jobs();

    let manager = executor.job_manager_mut();
    let mut jobs: Vec<_> = manager.jobs().collect();

    // Sort by ID for consistent output
    jobs.sort_by_key(|j| j.id);

    for job in jobs {
        println!("[{}] {} {}", job.id, job.status, job.command);
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;
    use nix::unistd::Pid;

    #[test]
    fn test_jobs_command_empty() {
        let mut executor = CommandExecutor::new();
        // With no jobs, should return 0
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_jobs_returns_zero_exit_code() {
        let mut executor = CommandExecutor::new();
        // jobs command always returns 0
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_jobs_with_mock_job() {
        let mut executor = CommandExecutor::new();
        // Add a mock job to the job manager
        let mock_pid = Pid::from_raw(9999);
        executor
            .job_manager_mut()
            .add_job(mock_pid, "sleep 100".to_string(), vec![mock_pid]);

        // jobs command should still return 0
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_jobs_with_multiple_mock_jobs() {
        let mut executor = CommandExecutor::new();
        let manager = executor.job_manager_mut();

        // Add multiple mock jobs
        let pid1 = Pid::from_raw(8001);
        let pid2 = Pid::from_raw(8002);
        let pid3 = Pid::from_raw(8003);

        manager.add_job(pid1, "sleep 100".to_string(), vec![pid1]);
        manager.add_job(pid2, "sleep 200".to_string(), vec![pid2]);
        manager.add_job(pid3, "sleep 300".to_string(), vec![pid3]);

        // jobs command should return 0 and list all jobs
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_jobs_ignores_arguments() {
        let mut executor = CommandExecutor::new();
        // jobs command should work even with arguments (which are typically ignored)
        let args = vec!["1".to_string(), "2".to_string()];
        let result = execute(&mut executor, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
