// Contract Test: User Story 4 - Pipeline Exit Code Handling
//
// Tests that pipeline exit codes follow Unix semantics (last command's exit code).
//
// Specification: specs/004-pipes/spec.md (Lines 62-76)
// Priority: P4

use rush::executor::parser::parse_pipeline;
use rush::executor::pipeline::PipelineExecutor;

/// US4-AS1: Given user types `true | false`, When pipeline completes,
/// Then exit code is 1 (last command failed)
#[test]
fn test_true_pipe_false() {
    let pipeline_input = "true | false";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Exit code from last command (false)
    assert_eq!(exit_code, 1, "Pipeline should return false's exit code (1)");
}

/// US4-AS2: Given user types `false | true`, When pipeline completes,
/// Then exit code is 0 (last command succeeded)
#[test]
fn test_false_pipe_true() {
    let pipeline_input = "false | true";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Exit code from last command (true)
    assert_eq!(exit_code, 0, "Pipeline should return true's exit code (0)");
}

/// US4-AS3: Given user types `echo test | grep test`, When grep succeeds,
/// Then exit code is 0
#[test]
fn test_echo_pipe_grep_success() {
    let pipeline_input = "echo test | grep test";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Exit code 0 (grep found match)
    assert_eq!(exit_code, 0, "Pipeline should return 0 when grep finds match");
}

/// US4-AS4: Given user types `echo test | grep nomatch`, When grep finds no match,
/// Then exit code is 1
#[test]
fn test_echo_pipe_grep_no_match() {
    let pipeline_input = "echo test | grep nomatch";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Exit code 1 (grep found no match)
    assert_eq!(exit_code, 1, "Pipeline should return 1 when grep finds no match");
}

/// Additional Test: Multiple failures - last wins
#[test]
fn test_multiple_failures_last_wins() {
    // All commands fail, but only last exit code is returned
    let pipeline_input = "false | false | false";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Exit code from last false
    assert_eq!(exit_code, 1, "Pipeline should return last command's exit code");
}

/// Additional Test: Mixed successes and failures
#[test]
fn test_mixed_success_failure() {
    // First succeeds, middle fails, last succeeds
    let pipeline_input = "true | false | true";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Exit code from last command (true)
    assert_eq!(exit_code, 0, "Pipeline should return last command's exit code (0)");
}

/// Additional Test: Non-zero exit codes other than 1
#[test]
fn test_custom_exit_codes() {
    // Use exit codes other than 0/1
    // sh -c 'exit 42' returns 42
    let pipeline_input = "true | sh -c 'exit 42'";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Custom exit code propagated
    assert_eq!(exit_code, 42, "Pipeline should return custom exit code");
}

/// Additional Test: Long pipeline exit code
#[test]
fn test_long_pipeline_exit_code() {
    // 5-command pipeline with mixed exit codes
    // Only last command's exit code should be returned
    let pipeline_input = "false | true | false | true | false";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Exit code from last command (false = 1)
    assert_eq!(exit_code, 1, "Long pipeline should return last command's exit code");
}

/// Additional Test: Exit code 127 (command not found)
#[test]
fn test_exit_code_127_command_not_found() {
    let pipeline_input = "true | nonexistent_command_xyz";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let result = executor.execute(&pipeline);

    // Verify: Returns 127 or error
    match result {
        Ok(code) => {
            assert_eq!(code, 127, "Should return 127 for command not found");
        }
        Err(_) => {
            // Also acceptable - command not found can be returned as error
        }
    }
}

/// Additional Test: Exit code from signal termination
#[test]
#[cfg(unix)]
fn test_exit_code_from_signal() {
    // Kill command with specific signal
    // sh -c 'kill -TERM $$' sends SIGTERM to itself
    let pipeline_input = "true | sh -c 'kill -TERM $$'";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Exit code reflects signal termination
    // SIGTERM typically results in exit code 143 (128 + 15)
    assert!(exit_code > 128, "Signal termination should return code > 128, got {}", exit_code);
}

/// Additional Test: Verify intermediate exit codes are not returned
#[test]
fn test_intermediate_exit_codes_ignored() {
    use std::fs;

    // Create temp file for logging
    let log_file = std::env::temp_dir().join("rush_pipe_test_us4_log.txt");

    // Pipeline: first fails (exit 1), last succeeds (exit 0)
    let pipeline_input = format!("sh -c 'echo first; exit 1' | tee {} | cat", log_file.display());
    let pipeline = parse_pipeline(&pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Exit code is 0 (from cat), not 1 (from sh)
    assert_eq!(exit_code, 0, "Pipeline should ignore intermediate failures");

    // Cleanup
    fs::remove_file(&log_file).ok();
}
