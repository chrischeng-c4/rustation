//! Job control implementation
//!
//! Handles management of background jobs, process groups, and job status.

use nix::unistd::Pid;
use std::collections::HashMap;
use std::fmt;

/// Status of a job
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    /// Running in background
    Running,
    /// Stopped (suspended)
    Stopped,
    /// Completed successfully
    Done,
    /// Terminated with error
    Failed,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Stopped => write!(f, "Stopped"),
            JobStatus::Done => write!(f, "Done"),
            JobStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// A job represents a running pipeline
#[derive(Debug, Clone)]
pub struct Job {
    /// Job ID (1, 2, 3...)
    pub id: usize,
    /// Process Group ID
    pub pgid: Pid,
    /// Command string
    pub command: String,
    /// PIDs of processes in this job
    pub pids: Vec<Pid>,
    /// Current status
    pub status: JobStatus,
}

impl Job {
    /// Create a new job
    pub fn new(id: usize, pgid: Pid, command: String, pids: Vec<Pid>) -> Self {
        Self { id, pgid, command, pids, status: JobStatus::Running }
    }
}

/// Manages active jobs
pub struct JobManager {
    /// Active jobs mapped by Job ID
    jobs: HashMap<usize, Job>,
    /// Next available Job ID
    next_id: usize,
}

impl JobManager {
    /// Create a new job manager
    pub fn new() -> Self {
        Self { jobs: HashMap::new(), next_id: 1 }
    }

    /// Add a new job
    pub fn add_job(&mut self, pgid: Pid, command: String, pids: Vec<Pid>) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let job = Job::new(id, pgid, command, pids);
        self.jobs.insert(id, job);
        id
    }

    /// Get a job by ID
    pub fn get_job(&self, id: usize) -> Option<&Job> {
        self.jobs.get(&id)
    }

    /// Get a mutable job by ID
    pub fn get_job_mut(&mut self, id: usize) -> Option<&mut Job> {
        self.jobs.get_mut(&id)
    }

    /// Remove a job by ID
    pub fn remove_job(&mut self, id: usize) -> Option<Job> {
        self.jobs.remove(&id)
    }

    /// Get all jobs
    pub fn jobs(&self) -> impl Iterator<Item = &Job> {
        self.jobs.values()
    }

    /// Update status of all jobs by checking for process changes
    pub fn update_status(&mut self) {
        use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

        for job in self.jobs.values_mut() {
            if job.status != JobStatus::Running && job.status != JobStatus::Stopped {
                continue;
            }

            let mut all_done = true;
            // Check each process in the job
            for pid in &job.pids {
                // WNOHANG returns immediately if no status change
                match waitpid(*pid, Some(WaitPidFlag::WNOHANG)) {
                    Ok(WaitStatus::Exited(_, _)) | Ok(WaitStatus::Signaled(_, _, _)) => {
                        // Process finished
                    }
                    Ok(WaitStatus::Stopped(_, _)) => {
                        // Process stopped
                        job.status = JobStatus::Stopped;
                        all_done = false;
                    }
                    Ok(WaitStatus::StillAlive) => {
                        all_done = false;
                    }
                    Err(_) => {
                        // Process likely gone or not a child
                    }
                    _ => {
                        all_done = false;
                    }
                }
            }

            if all_done {
                job.status = JobStatus::Done;
            }
        }
    }

    /// Clean up finished jobs (Done or Failed)
    /// Returns list of cleaned up jobs for notification
    pub fn cleanup(&mut self) -> Vec<Job> {
        let mut finished_ids = Vec::new();

        for (id, job) in &self.jobs {
            if matches!(job.status, JobStatus::Done | JobStatus::Failed) {
                finished_ids.push(*id);
            }
        }

        let mut removed_jobs = Vec::new();
        for id in finished_ids {
            if let Some(job) = self.jobs.remove(&id) {
                removed_jobs.push(job);
            }
        }

        removed_jobs
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::unistd::Pid;

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Running.to_string(), "Running");
        assert_eq!(JobStatus::Stopped.to_string(), "Stopped");
        assert_eq!(JobStatus::Done.to_string(), "Done");
        assert_eq!(JobStatus::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_job_status_eq() {
        assert_eq!(JobStatus::Running, JobStatus::Running);
        assert_ne!(JobStatus::Running, JobStatus::Stopped);
        assert_eq!(JobStatus::Done, JobStatus::Done);
        assert_ne!(JobStatus::Done, JobStatus::Failed);
    }

    #[test]
    fn test_job_creation() {
        let pids = vec![Pid::from_raw(1234), Pid::from_raw(5678)];
        let job = Job::new(1, Pid::from_raw(1234), "echo test".to_string(), pids.clone());

        assert_eq!(job.id, 1);
        assert_eq!(job.pgid, Pid::from_raw(1234));
        assert_eq!(job.command, "echo test");
        assert_eq!(job.pids, pids);
        assert_eq!(job.status, JobStatus::Running);
    }

    #[test]
    fn test_job_clone() {
        let pids = vec![Pid::from_raw(1234)];
        let job1 = Job::new(1, Pid::from_raw(1234), "test".to_string(), pids.clone());
        let job2 = job1.clone();

        assert_eq!(job1.id, job2.id);
        assert_eq!(job1.pgid, job2.pgid);
        assert_eq!(job1.command, job2.command);
        assert_eq!(job1.status, job2.status);
    }

    #[test]
    fn test_job_manager_new() {
        let manager = JobManager::new();
        assert_eq!(manager.jobs().count(), 0);
    }

    #[test]
    fn test_job_manager_default() {
        let manager = JobManager::default();
        assert_eq!(manager.jobs().count(), 0);
    }

    #[test]
    fn test_add_job() {
        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(1234)];

        let id1 = manager.add_job(Pid::from_raw(1234), "echo test".to_string(), pids.clone());
        assert_eq!(id1, 1);
        assert_eq!(manager.jobs().count(), 1);

        let id2 = manager.add_job(Pid::from_raw(5678), "cat file".to_string(), pids.clone());
        assert_eq!(id2, 2);
        assert_eq!(manager.jobs().count(), 2);
    }

    #[test]
    fn test_get_job() {
        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(1234)];

        let id = manager.add_job(Pid::from_raw(1234), "echo test".to_string(), pids.clone());

        let job = manager.get_job(id);
        assert!(job.is_some());
        assert_eq!(job.unwrap().command, "echo test");

        let missing = manager.get_job(999);
        assert!(missing.is_none());
    }

    #[test]
    fn test_get_job_mut() {
        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(1234)];

        let id = manager.add_job(Pid::from_raw(1234), "echo test".to_string(), pids.clone());

        // Modify job status
        let job = manager.get_job_mut(id).unwrap();
        job.status = JobStatus::Stopped;

        // Verify change persisted
        let job = manager.get_job(id).unwrap();
        assert_eq!(job.status, JobStatus::Stopped);
    }

    #[test]
    fn test_remove_job() {
        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(1234)];

        let id = manager.add_job(Pid::from_raw(1234), "echo test".to_string(), pids.clone());
        assert_eq!(manager.jobs().count(), 1);

        let removed = manager.remove_job(id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().command, "echo test");
        assert_eq!(manager.jobs().count(), 0);

        // Removing again returns None
        let missing = manager.remove_job(id);
        assert!(missing.is_none());
    }

    #[test]
    fn test_jobs_iterator() {
        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(1234)];

        manager.add_job(Pid::from_raw(1234), "cmd1".to_string(), pids.clone());
        manager.add_job(Pid::from_raw(5678), "cmd2".to_string(), pids.clone());
        manager.add_job(Pid::from_raw(9012), "cmd3".to_string(), pids.clone());

        let count = manager.jobs().count();
        assert_eq!(count, 3);

        let commands: Vec<String> = manager.jobs().map(|j| j.command.clone()).collect();
        assert!(commands.contains(&"cmd1".to_string()));
        assert!(commands.contains(&"cmd2".to_string()));
        assert!(commands.contains(&"cmd3".to_string()));
    }

    #[test]
    fn test_cleanup_no_finished_jobs() {
        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(1234)];

        manager.add_job(Pid::from_raw(1234), "cmd1".to_string(), pids.clone());
        manager.add_job(Pid::from_raw(5678), "cmd2".to_string(), pids.clone());

        let cleaned = manager.cleanup();
        assert_eq!(cleaned.len(), 0);
        assert_eq!(manager.jobs().count(), 2);
    }

    #[test]
    fn test_cleanup_done_jobs() {
        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(1234)];

        let id1 = manager.add_job(Pid::from_raw(1234), "cmd1".to_string(), pids.clone());
        let id2 = manager.add_job(Pid::from_raw(5678), "cmd2".to_string(), pids.clone());
        let id3 = manager.add_job(Pid::from_raw(9012), "cmd3".to_string(), pids.clone());

        // Mark some jobs as done
        manager.get_job_mut(id1).unwrap().status = JobStatus::Done;
        manager.get_job_mut(id3).unwrap().status = JobStatus::Failed;

        let cleaned = manager.cleanup();
        assert_eq!(cleaned.len(), 2);
        assert_eq!(manager.jobs().count(), 1);

        // Verify only running job remains
        let remaining = manager.get_job(id2).unwrap();
        assert_eq!(remaining.command, "cmd2");
        assert_eq!(remaining.status, JobStatus::Running);
    }

    #[test]
    fn test_cleanup_all_jobs() {
        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(1234)];

        let id1 = manager.add_job(Pid::from_raw(1234), "cmd1".to_string(), pids.clone());
        let id2 = manager.add_job(Pid::from_raw(5678), "cmd2".to_string(), pids.clone());

        manager.get_job_mut(id1).unwrap().status = JobStatus::Done;
        manager.get_job_mut(id2).unwrap().status = JobStatus::Failed;

        let cleaned = manager.cleanup();
        assert_eq!(cleaned.len(), 2);
        assert_eq!(manager.jobs().count(), 0);
    }

    #[test]
    fn test_update_status_skips_non_running_jobs() {
        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(99999)]; // Non-existent PID

        let id = manager.add_job(Pid::from_raw(99999), "cmd".to_string(), pids);

        // Mark as done
        manager.get_job_mut(id).unwrap().status = JobStatus::Done;

        // update_status should skip it
        manager.update_status();

        // Status should remain Done
        assert_eq!(manager.get_job(id).unwrap().status, JobStatus::Done);
    }

    #[test]
    fn test_update_status_handles_exited_process() {
        use std::process::Command;
        use std::thread;
        use std::time::Duration;

        // Spawn a process that exits quickly
        let child = Command::new("true")
            .spawn()
            .expect("Failed to spawn true process");

        let child_pid = child.id() as i32;

        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(child_pid)];

        let id = manager.add_job(Pid::from_raw(child_pid), "true".to_string(), pids);

        // Give the process time to exit
        thread::sleep(Duration::from_millis(100));

        // update_status should mark it as Done when process exits (line 113-114)
        manager.update_status();

        // Status should now be Done because the process exited
        let job = manager.get_job(id).expect("Job should still exist");
        assert_eq!(job.status, JobStatus::Done);
    }

    #[test]
    fn test_update_status_detects_still_alive_process() {
        use std::process::Command;
        use std::thread;
        use std::time::Duration;

        // Spawn a long-running process
        let child = Command::new("sleep")
            .arg("5")
            .spawn()
            .expect("Failed to spawn sleep process");

        let child_pid = child.id() as i32;

        let mut manager = JobManager::new();
        let pids = vec![Pid::from_raw(child_pid)];

        let id = manager.add_job(Pid::from_raw(child_pid), "sleep 5".to_string(), pids);

        // Give the process a moment
        thread::sleep(Duration::from_millis(50));

        // update_status should keep it as Running if still alive (line 121-122)
        manager.update_status();

        // Status should still be Running because the process is still alive
        let job = manager.get_job(id).expect("Job should still exist");
        assert_eq!(job.status, JobStatus::Running);
    }
}
