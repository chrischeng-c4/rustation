//! Job control implementation

/// A running or suspended process managed by the shell
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    /// Shell-internal job ID
    pub id: usize,

    /// Process ID from OS
    pub pid: i32,

    /// Process group ID
    pub pgid: i32,

    /// Command text
    pub command: String,

    /// Current job state
    pub state: JobState,

    /// Started with & operator
    pub background: bool,
}

impl Job {
    /// Create a new job
    pub fn new(id: usize, pid: i32, pgid: i32, command: String, background: bool) -> Self {
        Self { id, pid, pgid, command, state: JobState::Running, background }
    }
}

/// Job execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    /// Currently executing
    Running,

    /// Stopped (Ctrl+Z)
    Suspended,

    /// Finished with exit code
    Completed(i32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_new() {
        let job = Job::new(1, 12345, 12345, "sleep 30".to_string(), false);
        assert_eq!(job.id, 1);
        assert_eq!(job.pid, 12345);
        assert_eq!(job.command, "sleep 30");
        assert_eq!(job.state, JobState::Running);
        assert!(!job.background);
    }

    #[test]
    fn test_job_state_transitions() {
        let mut job = Job::new(2, 100, 100, "test".to_string(), true);

        assert_eq!(job.state, JobState::Running);

        job.state = JobState::Suspended;
        assert_eq!(job.state, JobState::Suspended);

        job.state = JobState::Completed(0);
        assert_eq!(job.state, JobState::Completed(0));
    }

    #[test]
    fn test_job_clone() {
        let job1 = Job::new(3, 200, 200, "echo hi".to_string(), false);
        let job2 = job1.clone();
        assert_eq!(job1, job2);
    }

    #[test]
    fn test_job_background() {
        let bg_job = Job::new(1, 100, 100, "sleep 30 &".to_string(), true);
        assert!(bg_job.background);

        let fg_job = Job::new(2, 200, 200, "ls".to_string(), false);
        assert!(!fg_job.background);
    }

    #[test]
    fn test_job_state_completed_with_code() {
        let mut job = Job::new(1, 100, 100, "test".to_string(), false);

        job.state = JobState::Completed(0);
        if let JobState::Completed(code) = job.state {
            assert_eq!(code, 0);
        } else {
            panic!("Expected Completed state");
        }

        job.state = JobState::Completed(127);
        if let JobState::Completed(code) = job.state {
            assert_eq!(code, 127);
        } else {
            panic!("Expected Completed state");
        }
    }

    #[test]
    fn test_job_equality() {
        let job1 = Job::new(1, 100, 100, "test".to_string(), false);
        let job2 = Job::new(1, 100, 100, "test".to_string(), false);

        assert_eq!(job1, job2);
    }

    #[test]
    fn test_job_inequality() {
        let job1 = Job::new(1, 100, 100, "test".to_string(), false);
        let job2 = Job::new(2, 100, 100, "test".to_string(), false);

        assert_ne!(job1, job2);
    }

    #[test]
    fn test_job_different_commands() {
        let job1 = Job::new(1, 100, 100, "ls".to_string(), false);
        let job2 = Job::new(1, 100, 100, "pwd".to_string(), false);

        assert_ne!(job1.command, job2.command);
    }

    #[test]
    fn test_job_state_copy() {
        let state1 = JobState::Running;
        let state2 = state1;

        assert_eq!(state1, state2);
    }

    #[test]
    fn test_job_pgid() {
        let job = Job::new(1, 12345, 12345, "test".to_string(), false);
        assert_eq!(job.pid, job.pgid);

        let job2 = Job::new(2, 12346, 12345, "test".to_string(), false);
        assert_ne!(job2.pid, job2.pgid);
    }

    #[test]
    fn test_job_long_command() {
        let long_cmd = "a".repeat(1000);
        let job = Job::new(1, 100, 100, long_cmd.clone(), false);

        assert_eq!(job.command, long_cmd);
    }
}
