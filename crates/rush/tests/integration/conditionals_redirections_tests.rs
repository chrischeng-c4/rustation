//! Integration tests for Phase 3: Redirection support in conditional statements
//! Tests that redirection syntax is preserved and parsed in if/then/else/elif blocks
//!
//! NOTE: Full redirection execution depends on CommandExecutor's I/O redirection implementation.
//! These tests verify that redirection syntax is supported in conditional bodies.

#[cfg(test)]
mod conditionals_redirections {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_if_then_with_output_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        let cmd = "if true; then echo hello world; fi > /tmp/if_output.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If statement with output redirection should parse successfully");
    }

    #[test]
    fn test_if_then_else_with_redirections_syntax() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "1".to_string())
            .unwrap();

        let cmd =
            "if [ $x -eq 1 ]; then echo first; else echo second; fi > /tmp/if_else_output.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If/else with output redirection should parse successfully");
    }

    #[test]
    fn test_if_elif_else_with_redirections_syntax() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("level".to_string(), "2".to_string())
            .unwrap();

        let cmd = "if [ $level -eq 1 ]; then echo one; elif [ $level -eq 2 ]; then echo two; else echo other; fi > /tmp/if_elif_else_output.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If/elif/else with output redirection should parse successfully");
    }

    #[test]
    fn test_if_then_with_append_redirection_syntax() {
        let mut executor = CommandExecutor::new();

        let cmd = "if true; then echo line1; fi >> /tmp/if_append.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If statement with append redirection should parse successfully");
    }

    #[test]
    fn test_if_then_pipe_with_redirection_syntax() {
        let mut executor = CommandExecutor::new();

        let cmd = "if true; then echo test | cat; fi > /tmp/if_pipe_output.txt";
        let result = executor.execute(cmd);

        assert!(
            result.is_ok(),
            "If statement with pipe and redirection should parse successfully"
        );
    }

    #[test]
    fn test_if_with_stderr_redirection_syntax() {
        let mut executor = CommandExecutor::new();

        let cmd = "if true; then ls /nonexistent; fi 2> /tmp/if_errors.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If statement with stderr redirection should parse successfully");
    }

    #[test]
    fn test_nested_if_with_redirections_syntax() {
        let mut executor = CommandExecutor::new();

        let cmd = "if true; then if true; then echo nested; fi; fi > /tmp/nested_if_output.txt";
        let result = executor.execute(cmd);

        assert!(
            result.is_ok(),
            "Nested if statements with redirection should parse successfully"
        );
    }

    #[test]
    fn test_if_then_multiple_redirections_syntax() {
        let mut executor = CommandExecutor::new();

        let cmd = "if true; then echo data 2>&1; fi > /tmp/combined.txt";
        let result = executor.execute(cmd);

        assert!(
            result.is_ok(),
            "If statement with multiple redirections should parse successfully"
        );
    }
}
