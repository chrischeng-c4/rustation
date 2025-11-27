//! 'bg' built-in command
//!
//! Resumes a stopped job in the background.

use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;
use crate::executor::job::JobStatus;
use nix::sys::signal::{kill, Signal};

/// Execute the 'bg' command
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    let manager = executor.job_manager_mut();

    // Parse job ID or use default (current/last stopped job)
    let job_id = if let Some(arg) = args.first() {
        arg.parse::<usize>()
            .map_err(|_| RushError::Execution("Invalid job ID".to_string()))?
    } else {
        // Find last stopped job
        manager
            .jobs()
            .filter(|j| j.status == JobStatus::Stopped)
            .max_by_key(|j| j.id)
            .map(|j| j.id)
            .ok_or_else(|| RushError::Execution("No stopped jobs".to_string()))?
    };

    // Get job
    let job = manager
        .get_job_mut(job_id)
        .ok_or_else(|| RushError::Execution(format!("Job {} not found", job_id)))?;

    let pgid = job.pgid;
    let cmd = job.command.clone();

    // Send SIGCONT
    if let Err(e) = kill(pgid, Signal::SIGCONT) {
        return Err(RushError::Execution(format!("Failed to resume job {}: {}", job_id, e)));
    }

    job.status = JobStatus::Running;
    println!("[{}] {} &", job_id, cmd);

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_bg_no_jobs() {
        let mut executor = CommandExecutor::new();
        // Should fail if no jobs
        let result = execute(&mut executor, &[]);
        assert!(result.is_err());
    }
}
