//! Integration tests for Phase 3: Pipe support in while/until loop bodies
//! Tests pipe operators (|) in while and until loop bodies

#[cfg(test)]
mod while_loop_pipes {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_while_loop_with_pipe() {
        let mut executor = CommandExecutor::new();

        // Test: while loop with pipe in body
        let cmd = "i=0; while [ $i -lt 2 ]; do echo $i | wc -c; i=$((i+1)); done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "While loop with pipe should execute successfully");
    }

    #[test]
    fn test_until_loop_with_pipe() {
        let mut executor = CommandExecutor::new();

        // Test: until loop with pipe in body
        let cmd = "i=0; until [ $i -ge 2 ]; do echo item$i | cat; i=$((i+1)); done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Until loop with pipe should execute successfully");
    }

    #[test]
    fn test_while_loop_with_grep_pipe() {
        let mut executor = CommandExecutor::new();

        // Test: while loop with grep through pipe
        let cmd = "i=0; while [ $i -lt 1 ]; do echo test | grep test; i=$((i+1)); done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "While loop with grep pipe should work");
    }

    #[test]
    fn test_while_loop_multiple_pipes() {
        let mut executor = CommandExecutor::new();

        // Test: while loop with multiple pipes in body
        let cmd = "i=0; while [ $i -lt 1 ]; do echo data | cat | cat; i=$((i+1)); done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "While loop with multiple pipes should work");
    }

    #[test]
    fn test_until_loop_with_variable_and_pipe() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("msg".to_string(), "hello".to_string())
            .unwrap();

        // Test: until loop using variable in piped command
        let cmd = "i=0; until [ $i -ge 1 ]; do echo $msg | wc -c; i=$((i+1)); done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Until loop with variable and pipe should work");
    }
}
