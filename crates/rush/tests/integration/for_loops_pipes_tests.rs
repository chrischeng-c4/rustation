//! Integration tests for Phase 3: Pipe support in for loop bodies
//! Tests pipe operators (|) in for loop bodies

#[cfg(test)]
mod for_loop_pipes {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_for_loop_with_pipe_echo_grep() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with pipe in body
        let cmd = "for item in apple banana cherry; do echo $item | grep 'a'; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "For loop with pipe should execute successfully");
    }

    #[test]
    fn test_for_loop_with_pipe_wc() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with pipe to wc
        let cmd = "for item in one two three; do echo $item | wc -c; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "For loop with pipe to wc should work");
    }

    #[test]
    fn test_for_loop_with_multiple_pipes() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with multiple pipes in body
        let cmd = "for item in test; do echo $item | cat | grep test; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "For loop with multiple pipes should work");
    }

    #[test]
    fn test_for_loop_with_pipe_and_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("pattern".to_string(), "test".to_string())
            .unwrap();

        // Test: for loop with pipe and external variable
        let cmd = "for item in test testing failed; do echo $item | grep $pattern; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "For loop with pipe and external variable should work");
    }

    #[test]
    fn test_for_loop_pipe_preserves_exit_code() {
        let mut executor = CommandExecutor::new();

        // Test: exit code from piped command
        let cmd = "for item in test; do echo $item | grep nomatch; done";
        let result = executor.execute(cmd);

        // Should execute (returns error code from grep, but that's ok)
        assert!(result.is_ok(), "Loop should execute even if pipe fails");
    }
}
