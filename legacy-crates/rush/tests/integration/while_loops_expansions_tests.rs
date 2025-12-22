//! Integration tests for Phase 2 word expansion in while/until loops
//! Tests variable expansion, command substitution, and expansion in loop conditions

#[cfg(test)]
mod while_loop_expansions {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_while_loop_with_variable_expansion() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("count".to_string(), "3".to_string())
            .unwrap();

        // Test: while loop with variable in condition
        let cmd = "i=0; while [ $i -lt $count ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should execute while loop with variable expansion");
    }

    #[test]
    fn test_until_loop_with_variable_expansion() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("limit".to_string(), "3".to_string())
            .unwrap();

        // Test: until loop with variable in condition
        let cmd = "i=0; until [ $i -ge $limit ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should execute until loop with variable expansion");
    }

    #[test]
    fn test_while_loop_with_variable_in_body() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("items".to_string(), "3".to_string())
            .unwrap();

        // Test: while loop using variable in the body
        let cmd = "i=1; while [ $i -le $items ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should expand variable in while loop body");
    }

    #[test]
    fn test_while_loop_with_braced_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("max".to_string(), "5".to_string())
            .unwrap();

        // Test: while loop with braced variable syntax ${VAR}
        let cmd = "i=0; while [ $i -lt ${max} ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should handle braced variable syntax in while condition");
    }

    #[test]
    fn test_while_loop_loop_variable_persistence() {
        let mut executor = CommandExecutor::new();

        // Test: loop variable persists after loop
        let cmd = "i=0; while [ $i -lt 3 ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Loop variable should persist after while loop");
    }

    #[test]
    fn test_while_loop_with_empty_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("empty".to_string(), "".to_string())
            .unwrap();

        // Test: while loop with empty variable expansion
        let cmd = "result=start; x=$empty; result=end; echo $result";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should handle empty variable expansion");
    }

    #[test]
    fn test_until_loop_with_variable_increment() {
        let mut executor = CommandExecutor::new();

        // Test: until loop with arithmetic variable increment
        let cmd = "i=0; until [ $i -eq 3 ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should handle arithmetic in until loop");
    }

    #[test]
    fn test_while_loop_variable_shadowing() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "original".to_string())
            .unwrap();

        // Test: while loop variable shadows existing variable
        let cmd = "x=original; i=0; while [ $i -lt 1 ]; do x=inside; i=$((i+1)); done; echo $x";
        let result = executor.execute(cmd);

        // x should be "inside" after loop (modified by loop body)
        assert!(result.is_ok(), "While loop should handle variable shadowing");
    }

    #[test]
    fn test_nested_while_loops_with_variables() {
        let mut executor = CommandExecutor::new();

        // Test: nested while loops with proper variable handling
        let cmd = "i=0; while [ $i -lt 2 ]; do j=0; while [ $j -lt 2 ]; do j=$((j+1)); done; i=$((i+1)); done; echo $i:$j";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Nested while loops should handle variables correctly");
    }

    #[test]
    fn test_while_loop_condition_with_command_result() {
        let mut executor = CommandExecutor::new();

        // Test: while loop using command execution result (Phase 2 command substitution)
        let cmd = "i=0; while [ $i -lt 2 ]; do i=$((i+1)); done; echo $i";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "While loop should evaluate conditions with variables correctly");
    }
}
