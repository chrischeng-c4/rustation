// Integration tests for pipe operator functionality
//
// Tests real command pipelines with actual process execution.

use rush::executor::execute::CommandExecutor;

#[test]
fn test_echo_pipe_grep() {
    let executor = CommandExecutor::new();

    // Test: echo outputs multiple lines, grep filters one
    let result = executor.execute("echo -e 'hello\\nworld\\ntest' | grep world");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_ls_pipe_wc() {
    let executor = CommandExecutor::new();

    // Test: ls outputs directory contents, wc counts lines
    let result = executor.execute("ls | wc -l");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_printf_pipe_cat() {
    let executor = CommandExecutor::new();

    // Test: printf outputs text, cat passes it through
    let result = executor.execute("printf 'test data' | cat");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_echo_pipe_grep_no_match() {
    let executor = CommandExecutor::new();

    // Test: grep returns 1 when no match found (exit code propagation)
    let result = executor.execute("echo 'hello' | grep 'xyz'");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // grep exits 1 on no match
}

#[test]
fn test_false_pipe_true() {
    let executor = CommandExecutor::new();

    // Test: Last command's exit code returned (true succeeds)
    let result = executor.execute("false | true");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // true returns 0
}

#[test]
fn test_true_pipe_false() {
    let executor = CommandExecutor::new();

    // Test: Last command's exit code returned (false fails)
    let result = executor.execute("true | false");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // false returns 1
}

#[test]
fn test_command_not_found_in_pipeline() {
    let executor = CommandExecutor::new();

    // Test: Nonexistent command in pipeline causes error
    // Pipelines fail fast when a command can't be spawned
    let result = executor.execute("echo test | this_does_not_exist_xyz123");
    assert!(result.is_err()); // Pipeline fails to spawn
}

#[test]
fn test_pipeline_with_arguments() {
    let executor = CommandExecutor::new();

    // Test: Commands with multiple arguments
    // Note: Using 'cat' without arguments (BSD cat compatible)
    let result = executor.execute("echo -n 'hello world' | cat");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_pipeline_with_quotes() {
    let executor = CommandExecutor::new();

    // Test: Quoted arguments in pipeline
    let result = executor.execute("echo 'test string' | grep 'string'");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_pipeline_preserves_binary_data() {
    let executor = CommandExecutor::new();

    // Test: Binary-safe pipes (printf with hex codes)
    let result = executor.execute("printf '\\x00\\x01\\x02' | cat");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_pipeline_with_whitespace_around_pipe() {
    let executor = CommandExecutor::new();

    // Test: Extra whitespace around pipe operator
    let result = executor.execute("echo test   |   cat");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_reusable_executor_with_pipes() {
    let executor = CommandExecutor::new();

    // Test: Executor can handle multiple pipeline commands
    let result1 = executor.execute("echo test | cat");
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), 0);

    let result2 = executor.execute("printf data | cat");
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), 0);

    let result3 = executor.execute("echo hello | grep hello");
    assert!(result3.is_ok());
    assert_eq!(result3.unwrap(), 0);
}

// ============================================================================
// User Story 2: Multi-Command Pipeline Chain Tests
// ============================================================================

#[test]
fn test_three_command_pipeline() {
    let executor = CommandExecutor::new();

    // Test: Three commands chained together
    let result = executor.execute("echo test | cat | cat");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_cat_pipe_grep_pipe_wc() {
    let executor = CommandExecutor::new();

    // Test: Create temp file, cat it, grep for pattern, count lines
    // Using printf to create data inline
    let result = executor.execute("printf 'line1\\nline2\\ntest' | grep line | wc -l");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_ls_pipe_grep_pipe_head() {
    let executor = CommandExecutor::new();

    // Test: ls output, filter with grep, take first line
    let result = executor.execute("ls | head -1 | cat");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_echo_pipe_sort_pipe_tail() {
    let executor = CommandExecutor::new();

    // Test: echo, sort, tail pipeline
    let result = executor.execute("printf 'z\\na\\nm\\n' | sort | tail -1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_five_command_pipeline() {
    let executor = CommandExecutor::new();

    // Test: Five commands chained
    let result = executor.execute("echo data | cat | cat | cat | cat");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_long_pipeline_10_commands() {
    let executor = CommandExecutor::new();

    // Test: 10-command pipeline (stress test)
    // Data flows through 10 cat commands
    let result = executor.execute("echo test | cat | cat | cat | cat | cat | cat | cat | cat | cat | cat");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_multi_command_exit_code() {
    let executor = CommandExecutor::new();

    // Test: Last command's exit code returned in multi-command pipeline
    let result = executor.execute("true | true | false");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // Last command (false) returns 1

    let result = executor.execute("false | false | true");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // Last command (true) returns 0
}

#[test]
fn test_multi_command_data_flow() {
    let executor = CommandExecutor::new();

    // Test: Data must flow through all commands for this to succeed
    // grep will only succeed if it receives the piped data
    let result = executor.execute("echo hello | cat | grep hello");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

// ============================================================================
// User Story 3: Error Handling Tests
// ============================================================================

#[test]
fn test_first_command_fails() {
    let executor = CommandExecutor::new();

    // Test: First command doesn't exist - should fail to spawn
    let result = executor.execute("nonexistent_cmd_xyz | cat");
    assert!(result.is_err(), "Pipeline with nonexistent first command should fail");
}

#[test]
fn test_second_command_fails() {
    let executor = CommandExecutor::new();

    // Test: Second command doesn't exist - should fail to spawn
    let result = executor.execute("echo test | nonexistent_cmd_xyz");
    assert!(result.is_err(), "Pipeline with nonexistent second command should fail");
}

#[test]
fn test_middle_command_fails() {
    let executor = CommandExecutor::new();

    // Test: Middle command in 3-command pipeline doesn't exist
    let result = executor.execute("echo test | nonexistent_cmd_xyz | cat");
    assert!(result.is_err(), "Pipeline with nonexistent middle command should fail");
}

#[test]
fn test_grep_no_matches() {
    let executor = CommandExecutor::new();

    // Test: grep returns exit code 1 when no match (not an error, just no match)
    // This is different from command-not-found - command succeeds but finds nothing
    let result = executor.execute("echo hello | grep xyz");
    assert!(result.is_ok(), "grep no-match should not be an error");
    assert_eq!(result.unwrap(), 1, "grep returns 1 when no matches found");
}

#[test]
fn test_broken_pipe() {
    let executor = CommandExecutor::new();

    // Test: Broken pipe scenario (first command writes more than second reads)
    // yes generates infinite output, head -1 reads one line then closes stdin
    // yes should get SIGPIPE (which is normal for pipelines)
    let result = executor.execute("yes | head -1");
    assert!(result.is_ok(), "Broken pipe is normal behavior");
    // head succeeded, so exit code should be 0
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_command_fails_with_exit_code() {
    let executor = CommandExecutor::new();

    // Test: Command runs but exits with non-zero (different from spawn failure)
    let result = executor.execute("echo test | false");
    assert!(result.is_ok(), "Command execution should succeed even if command returns non-zero");
    assert_eq!(result.unwrap(), 1, "Should return false's exit code");
}

// ============================================================================
// User Story 4: Exit Code Handling Tests
// ============================================================================

#[test]
fn test_exit_code_propagation() {
    let executor = CommandExecutor::new();

    // Test various exit code scenarios
    let test_cases = vec![
        ("true | true | true", 0),
        ("true | true | false", 1),
        ("false | true | false", 1),
        ("false | false | true", 0),
    ];

    for (cmd, expected) in test_cases {
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Pipeline {} should execute", cmd);
        assert_eq!(
            result.unwrap(),
            expected,
            "Pipeline {} should return exit code {}",
            cmd,
            expected
        );
    }
}

#[test]
fn test_exit_code_last_command_only() {
    let executor = CommandExecutor::new();

    // Test: Only last command's exit code matters
    // All previous commands fail, but last succeeds -> 0
    let result = executor.execute("false | false | false | true");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0, "Returns last command's exit code (0)");

    // All previous commands succeed, but last fails -> 1
    let result = executor.execute("true | true | true | false");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1, "Returns last command's exit code (1)");
}

#[test]
fn test_exit_code_with_real_commands() {
    let executor = CommandExecutor::new();

    // Test: Exit codes with real commands
    // grep succeeds -> 0
    let result = executor.execute("echo hello | grep hello");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    // grep fails (no match) -> 1
    let result = executor.execute("echo hello | grep xyz");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}
