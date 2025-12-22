//! Integration tests for Phase 3: Redirection support in while/until loop bodies
//! Tests that redirection syntax is preserved and parsed in while/until loop bodies
//!
//! NOTE: Full redirection execution depends on CommandExecutor's I/O redirection implementation.
//! These tests verify that redirection syntax is supported in loop bodies.

#[cfg(test)]
mod while_loop_redirections {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_while_loop_with_output_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("i".to_string(), "1".to_string())
            .unwrap();

        let cmd = "while [ $i -le 3 ]; do echo $i; i=$((i+1)); done > /tmp/while_output.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "While loop with output redirection should parse successfully");
    }

    #[test]
    fn test_until_loop_with_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("count".to_string(), "0".to_string())
            .unwrap();

        let cmd = "until [ $count -ge 2 ]; do echo $count; count=$((count+1)); done > /tmp/until_output.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Until loop with output redirection should parse successfully");
    }

    #[test]
    fn test_while_loop_with_append_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("i".to_string(), "1".to_string())
            .unwrap();

        let cmd = "while [ $i -le 2 ]; do echo $i; i=$((i+1)); done >> /tmp/while_append.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "While loop with append redirection should parse successfully");
    }

    #[test]
    fn test_while_loop_pipe_with_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "1".to_string())
            .unwrap();

        let cmd = "while [ $x -le 2 ]; do echo data$x | cat; x=$((x+1)); done > /tmp/while_pipe_output.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "While loop with pipe and redirection should parse successfully");
    }

    #[test]
    fn test_while_loop_with_stderr_redirection_syntax() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("n".to_string(), "1".to_string())
            .unwrap();

        let cmd =
            "while [ $n -le 1 ]; do ls /nonexistent; n=$((n+1)); done 2> /tmp/while_errors.txt";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "While loop with stderr redirection should parse successfully");
    }
}
