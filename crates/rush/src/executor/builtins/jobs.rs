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

    #[test]
    fn test_jobs_command() {
        let mut executor = CommandExecutor::new();
        // Just verify it runs without error (hard to mock active jobs without running processes)
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
