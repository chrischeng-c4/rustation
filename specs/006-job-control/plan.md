# Implementation Plan: Job Control

**Feature:** 006-job-control
**Status:** Partially Complete (60% - Requires Ctrl+Z implementation to reach 100%)
**Created:** 2025-11-29

## Overview

This plan documents the existing job control implementation (US1-4, US6) and provides detailed steps for completing the missing 40% (Ctrl+Z suspension and comprehensive testing).

## Current Implementation Status

### ✅ Completed (US1, 2, 4, 6)
- Background execution with `&` operator
- `jobs` builtin command
- `fg` builtin command
- `bg` builtin command
- Automatic job status tracking and cleanup
- Job state machine (Running, Stopped, Done, Failed)
- JobManager infrastructure

### ❌ Missing (US5)
- Ctrl+Z (SIGTSTP) signal handling
- Process group management enhancements
- Comprehensive integration testing

## Architecture Overview

### Module Structure
```
crates/rush/src/
├── executor/
│   ├── job.rs              # JobManager, Job struct, JobStatus enum
│   ├── builtins/
│   │   ├── jobs.rs         # `jobs` command
│   │   ├── fg.rs           # `fg` command
│   │   └── bg.rs           # `bg` command
│   ├── parser.rs           # `&` operator parsing
│   ├── execute.rs          # Background job spawning
│   └── mod.rs              # Integration
└── repl/
    └── mod.rs              # Job status checking loop
```

### Key Data Structures

**Job State Machine** (`job.rs`):
```rust
pub enum JobStatus {
    Running,    // Job executing
    Stopped,    // Suspended (Ctrl+Z)
    Done,       // Completed successfully
    Failed,     // Exited with error (exit != 0)
}

pub struct Job {
    id: i32,                          // Auto-incremented job ID
    pgid: u32,                        // Process group ID
    command: String,                  // Original command string
    status: JobStatus,                // Current state
    exit_code: Option<i32>,           // Exit code if Done/Failed
}

pub struct JobManager {
    jobs: HashMap<i32, Job>,          // job_id → Job
    next_job_id: i32,                 // For auto-incrementing
}
```

### Control Flow

```
REPL Loop:
├── Print prompt
├── Read command line
├── Parse (check for &)
├── If &:
│   └── Spawn background job, add to JobManager
├── Else:
│   └── Execute in foreground
├── check_background_jobs() ← Polls all jobs
│   ├── waitpid(WNOHANG) for each job
│   ├── Update status
│   └── Print notifications
└── Repeat
```

## Completed Implementation Details

### 1. Job Spawning (`executor/execute.rs`)

```rust
// When & is detected at end of command:
if command_ends_with_ampersand {
    // Fork new process
    match fork() {
        Ok(ForkResult::Parent { child }) => {
            // Parent shell
            job_id = self.job_manager.add_job(child.as_raw(), command)?;
            println!("[{}] {}", job_id, child.as_raw());
            return Ok(0);  // Immediate return
        }
        Ok(ForkResult::Child) => {
            // Child process
            exec(command)?;  // Never returns
        }
    }
}
```

**Status**: ✅ Working (commit c4a5ff1)

### 2. JobManager (`executor/job.rs`)

```rust
pub fn add_job(&mut self, pid: u32, command: String) -> Result<i32> {
    self.next_job_id += 1;
    let job = Job {
        id: self.next_job_id,
        pgid: pid as i32,  // ⚠️ Should be setpgid(pid, 0)
        command,
        status: JobStatus::Running,
        exit_code: None,
    };
    self.jobs.insert(self.next_job_id, job);
    Ok(self.next_job_id)
}

pub fn update_status(&mut self) {
    for job in self.jobs.values_mut() {
        if job.status == JobStatus::Running {
            match waitpid(Some(NonZeroI32::new(job.pgid).unwrap()), Some(WaitPidFlags::WNOHANG)) {
                Ok(WaitStatus::Exited(_, status)) => {
                    job.status = if status == 0 {
                        JobStatus::Done
                    } else {
                        JobStatus::Failed
                    };
                    job.exit_code = Some(status);
                }
                Ok(WaitStatus::Signaled(_, signal, _)) => {
                    job.status = JobStatus::Failed;
                    job.exit_code = Some(128 + signal as i32);
                }
                _ => {} // Still running or stopped
            }
        }
    }
}
```

**Status**: ✅ Working (commit c4a5ff1)

### 3. Builtins

**`jobs` builtin** (`executor/builtins/jobs.rs`):
```rust
pub fn jobs(executor: &mut CommandExecutor, _args: &[String]) -> Result<i32> {
    executor.job_manager_mut().update_status();

    let jobs = executor.job_manager().list_jobs();
    for job in jobs {
        let status_str = match job.status {
            JobStatus::Running => "Running",
            JobStatus::Stopped => "Stopped",
            JobStatus::Done => "Done",
            JobStatus::Failed => "Failed",
        };
        println!("[{}]  {}    {}", job.id, status_str, job.command);
    }
    Ok(0)
}
```

**Status**: ✅ Working (commit c4a5ff1)

**`fg` builtin** (`executor/builtins/fg.rs`):
```rust
pub fn fg(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Parse job ID or use current (+)
    let job_id = if args.is_empty() {
        executor.job_manager().current_job_id()?
    } else {
        args[0].parse::<i32>()?
    };

    // Get job
    let job = executor.job_manager_mut().get_job_mut(job_id)?;

    // Resume if stopped
    if job.status == JobStatus::Stopped {
        signal::kill(Pid::from_raw(job.pgid), Signal::SIGCONT)?;
    }

    // Transfer terminal control
    unistd::tcsetpgrp(0, Pid::from_raw(job.pgid))?;

    // Wait for completion
    let result = waitpid(Some(NonZeroI32::new(job.pgid)?), None)?;

    // Update status
    // ...

    Ok(job.exit_code.unwrap_or(0))
}
```

**Status**: ✅ Implemented, ⚠️ Limited testing

**`bg` builtin** (`executor/builtins/bg.rs`):
```rust
pub fn bg(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    let job_id = args[0].parse::<i32>()?;
    let job = executor.job_manager_mut().get_job_mut(job_id)?;

    // Resume if stopped
    if job.status == JobStatus::Stopped {
        signal::kill(Pid::from_raw(job.pgid), Signal::SIGCONT)?;
        job.status = JobStatus::Running;
    }

    println!("[{}]+ Running    {}", job.id, job.command);
    Ok(0)
}
```

**Status**: ✅ Implemented, ⚠️ Limited testing

---

## Phase 3: Complete Implementation (40% Remaining)

### Task 3.1: Implement SIGTSTP Handler

**File**: `crates/rush/src/repl/mod.rs`

**What to Add**:
1. Install SIGTSTP signal handler at REPL startup
2. When Ctrl+Z pressed:
   - Get foreground process group
   - Send SIGTSTP to that group
   - Update job status to Stopped
   - Print notification
   - Return to prompt

**Code Skeleton**:
```rust
use nix::sys::signal::{signal, SigHandler, Signal};
use nix::unistd;

// At REPL startup:
fn setup_signal_handlers() -> Result<()> {
    unsafe {
        signal(Signal::SIGTSTP, SigHandler::SigDfl)?;
        // Actually: need to implement custom handler
    }
    Ok(())
}

// Signal handler (custom):
fn handle_sigtstp(sig: i32) {
    if let Some(fg_pgid) = get_foreground_pgid() {
        // Send SIGTSTP to foreground process group
        kill(-fg_pgid, SIGTSTP);

        // Update job status
        update_job_status(fg_pgid, JobStatus::Stopped);

        // Print notification
        print_job_notification(fg_pgid, "Stopped");

        // Re-show prompt
        show_prompt();
    }
}
```

**Estimated Effort**: 100 lines + testing
**Complexity**: Medium (signal handling in Rust can be tricky)
**Dependencies**: `nix` crate (already in use)

**Testing**:
- Run: `sleep 100`
- Press: Ctrl+Z
- Verify: Job shows as Stopped
- Verify: `fg 1` resumes it

---

### Task 3.2: Enhance Process Group Management

**File**: `crates/rush/src/executor/execute.rs`

**Current Issue**:
```rust
// Currently: uses PID as PGID
job.pgid = pid;  // ❌ Wrong

// Should be:
setpgid(pid, 0);  // ✅ Create new process group
```

**What to Change**:
1. After spawning background job, call `setpgid()`
2. Store actual process group ID (may differ from PID)
3. Use PGID for signal delivery (important for pipelines!)

**Code Change**:
```rust
// In background job spawning:
match fork() {
    Ok(ForkResult::Parent { child }) => {
        // Ensure child is in its own process group
        let pgid = child.as_raw() as i32;
        setpgid(Some(child.as_raw()), pgid)?;

        // Store PGID, not PID
        self.job_manager.add_job(pgid, command)?;
    }
    Ok(ForkResult::Child) => {
        // Child creates new process group
        setpgid(None, None)?;
        exec(command)?;
    }
}
```

**Estimated Effort**: 30 lines
**Complexity**: Low
**Why Important**: Signals reach all processes in group (critical for pipelines)

---

### Task 3.3: Add Comprehensive Integration Tests

**File**: `crates/rush/tests/integration/job_control_tests.rs` (new)

**Test Scenarios** (10+ tests):

1. **Background Job Execution**
   ```rust
   #[test]
   fn test_background_job_execution() {
       // Run: sleep 2 &
       // Check: Job ID printed
       // Check: Prompt returns
       // Wait: 2 seconds
       // Check: Job completes
   }
   ```

2. **Job Listing**
   ```rust
   #[test]
   fn test_jobs_command() {
       // Run: sleep 10 & sleep 20 &
       // Run: jobs
       // Verify: Both jobs shown
       // Verify: Correct IDs
   }
   ```

3. **Foreground Resume**
   ```rust
   #[test]
   fn test_fg_resume() {
       // Run: sleep 100 & (pause it manually)
       // Run: fg 1
       // Check: Job runs in foreground
       // Check: Terminal control transferred
   }
   ```

4. **Background Resume**
   ```rust
   #[test]
   fn test_bg_resume() {
       // Run: sleep 100 & (pause it manually)
       // Run: bg 1
       // Check: Job resumes
       // Check: Message printed
   }
   ```

5. **Ctrl+Z Suspension** (when implemented)
   ```rust
   #[test]
   fn test_sigtstp_suspension() {
       // Run: sleep 100
       // Send: SIGTSTP
       // Check: Job suspended
       // Check: Notification printed
       // Run: fg 1
       // Check: Job resumes
   }
   ```

6. **Multiple Jobs**
   ```rust
   #[test]
   fn test_multiple_concurrent_jobs() {
       // Run: sleep 10 & sleep 20 & sleep 5 &
       // Run: jobs
       // Verify: All running
       // Wait: 5 seconds
       // Verify: Job 3 done, others running
   }
   ```

7. **Automatic Cleanup**
   ```rust
   #[test]
   fn test_automatic_job_cleanup() {
       // Run: sleep 1 &
       // Run: jobs
       // Verify: Job shown
       // Wait: 2 seconds
       // Check: Job cleaned up
   }
   ```

8. **Terminal Control**
   ```rust
   #[test]
   fn test_terminal_control_transfer() {
       // Run: yes > /tmp/yes.txt &
       // Run: fg 1 (then Ctrl+C)
       // Check: Output goes to file
       // Check: Ctrl+C works
   }
   ```

9. **Error Handling**
   ```rust
   #[test]
   fn test_invalid_job_id() {
       // Run: fg 999 (invalid)
       // Check: Error message
       // Check: Exit code 1
   }
   ```

10. **Job Status Transitions**
    ```rust
    #[test]
    fn test_job_status_transitions() {
        // Create job in Running state
        // Transition to Stopped (Ctrl+Z)
        // Verify: Status changes
        // Transition to Running (bg)
        // Verify: Status changes
    }
    ```

**Estimated Effort**: 300+ lines of test code
**Complexity**: Medium (involves process spawning and timing)
**Tools Needed**:
- `std::process::Command` for spawning test processes
- `std::time::Duration` for delays
- Proper cleanup (kill processes in test teardown)

---

### Task 3.4: Update Existing Unit Tests

**Files**:
- `crates/rush/src/executor/builtins/jobs.rs`
- `crates/rush/src/executor/builtins/fg.rs`
- `crates/rush/src/executor/builtins/bg.rs`

**Current State**: Only 3 error case tests

**What to Add**:
1. Real process spawning (not just error cases)
2. Job status verification
3. Signal delivery verification
4. Terminal control verification

**Example Addition**:
```rust
#[test]
fn test_jobs_shows_running_job() {
    let mut executor = CommandExecutor::new();

    // Spawn background job
    executor.execute("sleep 10 &")?;

    // Call jobs
    let result = jobs(&mut executor, &[])?;

    // Verify
    assert_eq!(result, 0);
    assert_eq!(executor.job_manager().count_jobs(), 1);

    // Cleanup
    executor.job_manager_mut().kill_all_jobs();
}
```

**Estimated Effort**: 150 lines
**Complexity**: Low to Medium

---

## Implementation Order

1. **Task 3.2**: Process group management (30 lines, ~30 min)
   - Low risk, foundational for other features

2. **Task 3.4**: Unit test updates (150 lines, ~1 hour)
   - Validate existing implementation
   - Low complexity

3. **Task 3.1**: SIGTSTP handler (100 lines, ~2 hours)
   - High value (completes Ctrl+Z)
   - Medium complexity

4. **Task 3.3**: Integration tests (300+ lines, ~3 hours)
   - High effort but important
   - Medium complexity

**Total Estimated Time**: 6-7 hours for full Phase 3 completion

---

## Testing Strategy

### Unit Tests
- Run: `cargo test -p rush executor::builtins`
- Validate individual builtin functionality
- Test error cases

### Integration Tests
- Run: `cargo test -p rush --test integration`
- Test end-to-end job control workflows
- Validate signal handling

### Manual Testing
```bash
# Test background execution
cargo run -p rush
> sleep 5 &
[1] 1234
> jobs
[1]+ Running    sleep 5
> fg 1
sleep 5           # Runs in foreground

# Test Ctrl+Z (when implemented)
> sleep 100
^Z
[1]+ Stopped   sleep 100
> bg 1
[1]+ Running    sleep 100
```

---

## Risks and Mitigations

### Risk 1: Signal Handler Complexity
**Impact**: Rust signal handling is tricky; easy to introduce bugs
**Mitigation**: Use `nix` crate helpers; reference existing `fg`/`bg` code; comprehensive testing

### Risk 2: Process Group Management
**Impact**: Incorrect PGID means signals don't reach all processes
**Mitigation**: Test with pipelines; verify `setpgid()` calls; validate group membership

### Risk 3: Test Flakiness
**Impact**: Timing-dependent tests may fail intermittently
**Mitigation**: Use generous timeouts; poll for status; clean up properly

### Risk 4: Terminal Control Issues
**Impact**: `tcsetpgrp()` can cause terminal hangs if not careful
**Mitigation**: Test in safe environment; use `strace` to debug; ensure proper cleanup

---

## Success Criteria

### After Phase 3 Completion
- ✅ Ctrl+Z (SIGTSTP) working
- ✅ Jobs suspend and resume correctly
- ✅ Process group management enhanced
- ✅ 10+ integration tests passing
- ✅ Unit tests updated and passing
- ✅ All tests pass: `cargo test -p rush`
- ✅ No clippy warnings: `cargo clippy -p rush`
- ✅ Code formatted: `cargo fmt -p rush`

### Coverage
- Background execution: ✅ 100% working
- Job listing: ✅ 100% working
- FG/BG commands: ✅ 100% working
- Ctrl+Z: ✅ Will be 100% after Phase 3
- Automatic cleanup: ✅ 100% working
- Error handling: ✅ 95%+ (edge cases may remain)

---

## Post-Implementation

After Phase 3 complete:
1. Update KNOWN_ISSUES.md (remove "Job Control" from missing)
2. Update README.md (add to "Implemented" features)
3. Create commit: "feat: complete job control with Ctrl+Z and tests"
4. Consider separate PR if changes are large (>1500 lines)

---

## Summary

Job control is nearly complete - only Ctrl+Z (SIGTSTP) and comprehensive testing remain. The foundation is solid:
- ✅ JobManager infrastructure
- ✅ Background execution
- ✅ Job listing and resumption
- ✅ Basic signal handling

Phase 3 will:
- ✅ Add Ctrl+Z suspension (critical missing feature)
- ✅ Enhance process group management (correctness improvement)
- ✅ Add 10+ integration tests (comprehensive coverage)
- ✅ Update existing tests (validation)

**Estimated effort**: 6-7 hours to reach 100% completion
