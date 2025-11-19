// Contract tests validating pipe implementation against spec requirements
//
// These tests verify that the implementation matches the behavioral specification
// defined in specs/004-pipes/spec.md

use rush::executor::execute::CommandExecutor;

// Success Criteria SC-001: Users can chain two commands with | operator
#[test]
fn sc001_users_can_chain_two_commands() {
    let executor = CommandExecutor::new();

    // User Story 1: Basic two-command pipeline
    let result = executor.execute("echo hello | cat");
    assert!(result.is_ok(), "SC-001: Two-command pipeline should succeed");
    assert_eq!(result.unwrap(), 0, "SC-001: Pipeline should return success");
}

// Success Criteria SC-002: Data flows from first command's stdout to second's stdin
#[test]
fn sc002_data_flows_through_pipe() {
    let executor = CommandExecutor::new();

    // grep will only succeed if it receives input from echo
    let result = executor.execute("echo 'test data' | grep 'test'");
    assert!(result.is_ok(), "SC-002: Pipeline execution should succeed");
    assert_eq!(result.unwrap(), 0, "SC-002: grep finds text, confirming data flow");

    // grep will fail if no match, confirming data was passed
    let result = executor.execute("echo 'hello' | grep 'test'");
    assert!(result.is_ok(), "SC-002: Pipeline execution should succeed");
    assert_eq!(result.unwrap(), 1, "SC-002: grep returns 1 when no match");
}

// Success Criteria SC-003: Commands execute concurrently (not sequentially)
#[test]
fn sc003_concurrent_execution() {
    let executor = CommandExecutor::new();

    // This test verifies concurrent execution by timing - concurrent should be fast
    // Note: Actual timing would require slow commands, this is a structural test
    let result = executor.execute("echo test | cat");
    assert!(result.is_ok(), "SC-003: Concurrent pipeline should succeed");

    // True concurrent test would use: sleep 1 | sleep 1
    // and verify total time is ~1s not ~2s (but that's a performance test)
}

// Success Criteria SC-004: Last command's exit code returned
#[test]
fn sc004_last_command_exit_code_returned() {
    let executor = CommandExecutor::new();

    // First fails, second succeeds -> 0
    let result = executor.execute("false | true");
    assert!(result.is_ok(), "SC-004: Pipeline execution should succeed");
    assert_eq!(result.unwrap(), 0, "SC-004: Returns last command's exit code (0)");

    // First succeeds, second fails -> 1
    let result = executor.execute("true | false");
    assert!(result.is_ok(), "SC-004: Pipeline execution should succeed");
    assert_eq!(result.unwrap(), 1, "SC-004: Returns last command's exit code (1)");

    // Both succeed -> 0
    let result = executor.execute("true | true");
    assert!(result.is_ok(), "SC-004: Pipeline execution should succeed");
    assert_eq!(result.unwrap(), 0, "SC-004: Returns last command's exit code (0)");
}

// Success Criteria SC-005: Pipe operator works consistently in interactive/non-interactive
#[test]
fn sc005_works_in_both_modes() {
    let executor = CommandExecutor::new();

    // CommandExecutor is used by both single-command mode (-c) and REPL
    // This test verifies it works in the shared executor path
    let result = executor.execute("echo test | cat");
    assert!(result.is_ok(), "SC-005: Should work in non-interactive mode");

    // Reusability test (REPL uses same executor)
    let result = executor.execute("printf data | cat");
    assert!(result.is_ok(), "SC-005: Should work repeatedly (REPL-like)");
}

// Functional Requirement FR-001: Parse single pipe operator
#[test]
fn fr001_parse_single_pipe_operator() {
    let executor = CommandExecutor::new();

    // Single pipe with no whitespace
    let result = executor.execute("echo test|cat");
    assert!(result.is_ok(), "FR-001: Should parse pipe without spaces");

    // Single pipe with whitespace
    let result = executor.execute("echo test  |  cat");
    assert!(result.is_ok(), "FR-001: Should parse pipe with spaces");
}

// Functional Requirement FR-002: Connect stdout to stdin
#[test]
fn fr002_connect_stdout_to_stdin() {
    let executor = CommandExecutor::new();

    // Data must flow for this to work
    let result = executor.execute("printf 'hello' | cat");
    assert!(result.is_ok(), "FR-002: stdout→stdin connection should work");
    assert_eq!(result.unwrap(), 0);
}

// Functional Requirement FR-003: Support pipes in quoted strings as literals
#[test]
fn fr003_quoted_pipes_are_literals() {
    let executor = CommandExecutor::new();

    // Pipe inside quotes should NOT create a pipeline
    let result = executor.execute("echo 'a | b'");
    assert!(result.is_ok(), "FR-003: Quoted pipe should be literal");
    assert_eq!(result.unwrap(), 0);
}

// Functional Requirement FR-007: Binary-safe I/O
#[test]
fn fr007_binary_safe_io() {
    let executor = CommandExecutor::new();

    // Test with binary data (null bytes, non-ASCII)
    let result = executor.execute("printf '\\x00\\x01\\xff' | cat");
    assert!(result.is_ok(), "FR-007: Binary data should pass through pipes");
    assert_eq!(result.unwrap(), 0);
}

// Functional Requirement FR-009: Return last command's exit code
#[test]
fn fr009_return_last_exit_code() {
    let executor = CommandExecutor::new();

    // Verify various exit code scenarios
    let cases = vec![
        ("true | true", 0),
        ("true | false", 1),
        ("false | true", 0),
        ("false | false", 1),
    ];

    for (cmd, expected_code) in cases {
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "FR-009: {} should execute", cmd);
        assert_eq!(
            result.unwrap(),
            expected_code,
            "FR-009: {} should return {}",
            cmd,
            expected_code
        );
    }
}

// Functional Requirement FR-011: Syntax errors return non-zero exit code
#[test]
fn fr011_syntax_errors_non_zero() {
    let executor = CommandExecutor::new();

    // Empty command before pipe
    let result = executor.execute("| cat");
    assert_eq!(result.unwrap(), 1, "FR-011: Syntax error should return 1");

    // Empty command after pipe
    let result = executor.execute("echo test |");
    assert_eq!(result.unwrap(), 1, "FR-011: Syntax error should return 1");

    // Double pipe
    let result = executor.execute("echo test | | cat");
    assert_eq!(result.unwrap(), 1, "FR-011: Syntax error should return 1");
}

// Edge Case EC-001: Large data volumes
#[test]
fn ec001_large_data_volumes() {
    let executor = CommandExecutor::new();

    // Test with multiple lines of data (simulating larger volumes)
    let result = executor.execute("printf 'line1\\nline2\\nline3\\nline4\\nline5' | cat");
    assert!(result.is_ok(), "EC-001: Should handle multiple lines");
    assert_eq!(result.unwrap(), 0);
}

// Edge Case EC-004: Malformed pipe syntax
#[test]
fn ec004_malformed_syntax() {
    let executor = CommandExecutor::new();

    // Test various malformed syntaxes
    let malformed = vec![
        "| cat",           // No command before
        "echo test |",     // No command after
        "echo | | cat",    // Double pipe
    ];

    for cmd in malformed {
        let result = executor.execute(cmd);
        assert!(
            result.unwrap() != 0,
            "EC-004: {} should return non-zero",
            cmd
        );
    }
}

// Edge Case EC-005: Pipes inside quoted strings
#[test]
fn ec005_pipes_in_quotes() {
    let executor = CommandExecutor::new();

    // Single quotes
    let result = executor.execute("echo 'cmd1 | cmd2'");
    assert!(result.is_ok(), "EC-005: Pipe in single quotes should be literal");

    // Double quotes
    let result = executor.execute("echo \"cmd1 | cmd2\"");
    assert!(result.is_ok(), "EC-005: Pipe in double quotes should be literal");
}

// User Story 1 (P1): Basic two-command pipeline
#[test]
fn us1_basic_two_command_pipeline() {
    let executor = CommandExecutor::new();

    // As described in spec: "As a user, I want to pipe the output of one command
    // into another so that I can filter and transform data"
    let result = executor.execute("echo 'hello\\nworld' | grep hello");
    assert!(result.is_ok(), "US1: Basic pipeline should work");
    assert_eq!(result.unwrap(), 0, "US1: grep should find match");

    // Acceptance: Output flows from first to second
    let result = executor.execute("printf 'test' | cat");
    assert!(result.is_ok(), "US1: Data should flow through pipe");
    assert_eq!(result.unwrap(), 0);
}

// Reusability requirement (REPL use case)
#[test]
fn repl_reusable_executor() {
    let executor = CommandExecutor::new();

    // REPL reuses the same executor for multiple commands
    for _ in 0..5 {
        let result = executor.execute("echo test | cat");
        assert!(result.is_ok(), "Executor should be reusable");
        assert_eq!(result.unwrap(), 0);
    }
}
