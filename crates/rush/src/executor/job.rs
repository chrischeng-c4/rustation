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
