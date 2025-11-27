//! 'fg' built-in command
//!
//! Brings a background job to the foreground.

use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;
use crate::executor::job::JobStatus;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{getpgrp, tcsetpgrp};
use std::io::stdin;

/// Execute the 'fg' command
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    let manager = executor.job_manager_mut();

    // Parse job ID or use default (last job)
    let job_id = if let Some(arg) = args.first() {
        arg.parse::<usize>()
            .map_err(|_| RushError::Execution("Invalid job ID".to_string()))?
    } else {
        // Find last job
        manager
            .jobs()
            .max_by_key(|j| j.id)
            .map(|j| j.id)
            .ok_or_else(|| RushError::Execution("No current job".to_string()))?
    };

    // Get job
    let job = manager
        .get_job_mut(job_id)
        .ok_or_else(|| RushError::Execution(format!("Job {} not found", job_id)))?;

    let pgid = job.pgid;
    let pids = job.pids.clone();
    let cmd = job.command.clone();

    println!("{}", cmd);

    // Give terminal control to job
    // Ignore errors if not running in a TTY
    let _ = tcsetpgrp(stdin(), pgid);

    // Send SIGCONT if stopped
    if job.status == JobStatus::Stopped {
        let _ = kill(pgid, Signal::SIGCONT);
    }

    job.status = JobStatus::Running;

    // Wait for job to finish or stop
    // We need to wait for ALL processes in the job
    // For MVP, we just wait for the last one or any that stops
    // A proper shell implementation is more complex here

    let mut exit_code = 0;
    let mut stopped = false;

    for pid in pids {
        match waitpid(pid, None) {
            Ok(WaitStatus::Exited(_, code)) => {
                exit_code = code;
            }
            Ok(WaitStatus::Signaled(_, _, _)) => {
                exit_code = 128 + 9; // SIGKILL approximation
            }
            Ok(WaitStatus::Stopped(_, _)) => {
                stopped = true;
            }
            _ => {}
        }
    }

    // Take back terminal control
    let shell_pgid = getpgrp();
    let _ = tcsetpgrp(stdin(), shell_pgid);

    if stopped {
        if let Some(job) = manager.get_job_mut(job_id) {
            job.status = JobStatus::Stopped;
            println!("\n[{}] Stopped {}", job_id, cmd);
        }
    } else {
        // Job finished, remove it
        manager.remove_job(job_id);
    }

    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_fg_no_jobs() {
        let mut executor = CommandExecutor::new();
        // Should fail if no jobs
        let result = execute(&mut executor, &[]);
        assert!(result.is_err());
    }
}
