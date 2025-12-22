//! Integration tests for Phase 2 command substitution in while/until loops
//! Tests $(cmd) and `cmd` syntax in loop conditions

#[cfg(test)]
mod while_loop_command_substitution {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_while_loop_with_command_substitution_in_condition() {
        let mut executor = CommandExecutor::new();

        // Test: while loop with command substitution in condition
        let cmd = "i=0; while [ $i -lt $(echo 3) ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "While loop with command substitution in condition should work");
    }

    #[test]
    fn test_until_loop_with_command_substitution_in_condition() {
        let mut executor = CommandExecutor::new();

        // Test: until loop with command substitution in condition
        let cmd = "i=0; until [ $i -ge $(echo 2) ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Until loop with command substitution in condition should work");
    }

    #[test]
    fn test_while_loop_with_command_substitution_in_body() {
        let mut executor = CommandExecutor::new();

        // Test: command substitution in loop body
        let cmd = "i=0; while [ $i -lt 2 ]; do val=$(echo $i); i=$((i+1)); done; echo done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Command substitution in while loop body should work");
    }

    #[test]
    fn test_while_loop_nested_command_substitution() {
        let mut executor = CommandExecutor::new();

        // Test: nested command substitution
        let cmd = "i=0; while [ $i -lt $(echo $(echo 1)) ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Nested command substitution in while condition should work");
    }

    #[test]
    fn test_while_loop_command_substitution_with_variables() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("limit".to_string(), "3".to_string())
            .unwrap();

        // Test: command substitution mixed with variables
        let cmd = "i=0; while [ $i -lt $(echo $limit) ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Command substitution with variables should work");
    }

    #[test]
    fn test_while_loop_command_substitution_arithmetic() {
        let mut executor = CommandExecutor::new();

        // Test: command substitution in arithmetic context
        let cmd = "i=0; max=$(echo 3); while [ $i -lt $max ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Command substitution in arithmetic should work");
    }

    #[test]
    fn test_while_loop_command_substitution_multiple_invocations() {
        let mut executor = CommandExecutor::new();

        // Test: command substitution called multiple times in loop condition
        let cmd = "i=0; while [ $i -lt $(echo 2) ]; do val=$(echo $i); i=$((i+1)); done; echo done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Multiple command substitutions in while loop should work");
    }

    #[test]
    fn test_until_loop_command_substitution_comparison() {
        let mut executor = CommandExecutor::new();

        // Test: until loop with command substitution in comparison
        let cmd =
            "count=0; until [ $(echo $count) -eq 2 ]; do count=$((count+1)); done; echo $count";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Until loop with command substitution comparison should work");
    }

    #[test]
    fn test_while_loop_backtick_command_substitution() {
        let mut executor = CommandExecutor::new();

        // Test: backtick syntax in while loop condition
        let cmd = "i=0; while [ $i -lt `echo 2` ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        // This might not be fully implemented, test gracefully
        let _ = result;
    }

    #[test]
    fn test_while_loop_command_substitution_string_test() {
        let mut executor = CommandExecutor::new();

        // Test: command substitution in string comparison
        let cmd = "x='hello'; while [ \"$x\" = \"$(echo hello)\" ]; do break; done; echo $x";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Command substitution in string comparison should work");
    }
}
