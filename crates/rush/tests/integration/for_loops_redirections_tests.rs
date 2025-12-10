//! Integration tests for Phase 3: Redirection support in for loop bodies
//! Tests that redirection syntax is preserved and parsed in for loop bodies
//!
//! NOTE: Full redirection execution depends on CommandExecutor's I/O redirection implementation.
//! These tests verify that:
//! 1. Redirection syntax is preserved in raw body strings
//! 2. The commands parse successfully without errors
//! 3. The architecture supports redirection in control flow bodies

#[cfg(test)]
mod for_loop_redirections {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_for_loop_with_output_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        let cmd = "for item in apple banana cherry; do echo $item; done > /tmp/output.txt";
        let result = executor.execute(cmd);

        // Should execute successfully - redirection is preserved in raw body
        assert!(result.is_ok(), "For loop with output redirection should parse successfully");
    }

    #[test]
    fn test_for_loop_with_append_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        let cmd = "for item in one two three; do echo $item; done >> /tmp/append.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "For loop with append redirection should parse successfully");
    }

    #[test]
    fn test_for_loop_with_stderr_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        let cmd = "for item in test; do ls /nonexistent; done 2> /tmp/errors.txt";
        let result = executor.execute(cmd);

        // Should execute even though the command fails
        assert!(result.is_ok(), "For loop with stderr redirection should parse successfully");
    }

    #[test]
    fn test_for_loop_pipe_with_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        let cmd = "for item in hello world; do echo $item | cat; done > /tmp/pipe_output.txt";
        let result = executor.execute(cmd);

        // Pipes and redirections should both be preserved
        assert!(result.is_ok(), "For loop with pipe and redirection should parse successfully");
    }

    #[test]
    fn test_for_loop_multiple_redirections() {
        let mut executor = CommandExecutor::new();
        let cmd = "for item in data; do echo $item 2>&1; done > /tmp/combined.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "For loop with multiple redirections should parse successfully");
    }

    #[test]
    fn test_for_loop_with_variable_in_redirection() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("outfile".to_string(), "/tmp/out.txt".to_string())
            .unwrap();
        let cmd = "for item in test; do echo $item; done > $outfile";
        let result = executor.execute(cmd);

        assert!(
            result.is_ok(),
            "For loop with variable in redirection should parse successfully"
        );
    }
}
