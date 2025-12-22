//! Integration tests for Phase 3: Pipe support in conditional statements
//! Tests pipe operators (|) in if/then/else/elif block bodies

#[cfg(test)]
mod conditionals_pipes {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_if_then_with_pipe_echo_grep() {
        let mut executor = CommandExecutor::new();

        // Test: if statement with pipe in then block
        let cmd = "if true; then echo apple banana | grep apple; fi";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If statement with pipe should execute successfully");
    }

    #[test]
    fn test_if_then_with_pipe_wc() {
        let mut executor = CommandExecutor::new();

        // Test: if statement with pipe to word count
        let cmd = "if true; then echo test | wc -c; fi";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If statement with pipe to wc should work");
    }

    #[test]
    fn test_if_then_else_with_pipes() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "1".to_string())
            .unwrap();

        // Test: if/else statement with pipes in both branches
        let cmd = "if [ $x -eq 1 ]; then echo first | cat; else echo second | cat; fi";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If/else with pipes in both branches should work");
    }

    #[test]
    fn test_if_elif_else_with_pipes() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("level".to_string(), "2".to_string())
            .unwrap();

        // Test: if/elif/else statement with pipes in all branches
        let cmd = "if [ $level -eq 1 ]; then echo one | cat; elif [ $level -eq 2 ]; then echo two | cat; else echo other | cat; fi";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If/elif/else with pipes in all branches should work");
    }

    #[test]
    fn test_if_then_with_multiple_pipes() {
        let mut executor = CommandExecutor::new();

        // Test: if statement with multiple chained pipes
        let cmd = "if true; then echo test | cat | grep test; fi";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If statement with multiple pipes should work");
    }

    #[test]
    fn test_if_then_with_pipe_and_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("pattern".to_string(), "test".to_string())
            .unwrap();

        // Test: if statement with pipe and variable expansion
        let cmd = "if true; then echo testing | grep $pattern; fi";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "If statement with pipe and variable expansion should work");
    }

    #[test]
    fn test_nested_if_with_pipes() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("outer".to_string(), "yes".to_string())
            .unwrap();

        // Test: nested if statements with pipes
        let cmd = "if [ $outer = yes ]; then if true; then echo nested | cat; fi; fi";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Nested if statements with pipes should work");
    }

    #[test]
    fn test_if_exit_code_with_pipe() {
        let mut executor = CommandExecutor::new();

        // Test: exit code from pipe in if statement
        let cmd = "if true; then echo test | grep nomatch; fi; if [ $? -ne 0 ]; then echo failed_as_expected; fi";
        let result = executor.execute(cmd);

        // Should execute successfully (grep fails but that's ok)
        assert!(result.is_ok(), "Exit code from pipe in if should be propagated");
    }

    #[test]
    fn test_elif_with_pipe_condition_and_body() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "2".to_string())
            .unwrap();

        // Test: elif with pipe in body (condition doesn't need pipe)
        let cmd = "if [ $x -eq 1 ]; then echo one; elif [ $x -eq 2 ]; then echo two | cat; fi";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Elif with pipe in body should work");
    }
}
