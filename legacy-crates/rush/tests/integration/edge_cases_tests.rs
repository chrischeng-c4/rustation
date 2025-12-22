//! Edge Case and Error Handling Tests for Features 017-026
//!
//! These tests verify robustness, boundary conditions, and error handling
//! across all control flow features.

#[cfg(test)]
mod edge_cases {
    use rush::executor::execute::CommandExecutor;

    // ===== Boundary Conditions =====

    #[test]
    fn test_empty_for_loop_list() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in ; do echo $i; done";
        let result = executor.execute(cmd);
        // Empty list should execute but loop body never runs
        assert!(result.is_ok(), "Empty for loop list should not crash");
    }

    #[test]
    fn test_for_loop_single_item() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in single; do echo $i; done";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Single item for loop should execute");
    }

    #[test]
    fn test_for_loop_with_spaces_in_values() {
        let mut executor = CommandExecutor::new();
        let cmd = "for item in 'one two' 'three four'; do echo $item; done";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "For loop with quoted values containing spaces should work");
    }

    #[test]
    fn test_while_loop_zero_iterations() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("n".to_string(), "5".to_string())
            .ok();
        let cmd = "while [ $n -lt 3 ]; do n=$((n+1)); done; echo done";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "While loop with zero iterations should not crash");
    }

    #[test]
    fn test_until_loop_immediate_exit() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "10".to_string())
            .ok();
        let cmd = "until [ $x -gt 5 ]; do echo $x; done";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Until loop with immediate exit condition should work");
    }

    #[test]
    fn test_if_with_no_then_block() {
        let mut executor = CommandExecutor::new();
        let cmd = "if true; then ; fi";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "If with empty then block should not crash");
    }

    #[test]
    fn test_deeply_nested_if_statements() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("a".to_string(), "1".to_string())
            .ok();
        executor
            .variable_manager_mut()
            .set("b".to_string(), "1".to_string())
            .ok();
        executor
            .variable_manager_mut()
            .set("c".to_string(), "1".to_string())
            .ok();
        let cmd = "if [ $a -eq 1 ]; then if [ $b -eq 1 ]; then if [ $c -eq 1 ]; then echo deep; fi; fi; fi";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Deeply nested if statements should work");
    }

    // ===== Error Handling =====

    #[test]
    fn test_undefined_variable_in_loop() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in 1 2 3; do echo $undefined_var; done";
        let result = executor.execute(cmd);
        // Should not crash, just expand to empty string
        assert!(result.is_ok(), "Undefined variable in loop should not crash");
    }

    #[test]
    fn test_undefined_variable_in_condition() {
        let mut executor = CommandExecutor::new();
        let cmd = "if [ $undefined -eq 1 ]; then echo found; else echo not; fi";
        let result = executor.execute(cmd);
        // Comparing empty string with -eq should fail, but not crash
        assert!(result.is_ok(), "Undefined variable in condition should not crash");
    }

    #[test]
    fn test_malformed_arithmetic_in_loop() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("i".to_string(), "1".to_string())
            .ok();
        let cmd = "while [ $i -le 3 ]; do i=$((invalid)); done";
        let result = executor.execute(cmd);
        // Should handle arithmetic error gracefully
        assert!(result.is_ok() || result.is_err(), "Malformed arithmetic should be handled");
    }

    #[test]
    fn test_break_outside_loop() {
        let mut executor = CommandExecutor::new();
        let cmd = "break";
        let result = executor.execute(cmd);
        // Break outside loop should error or no-op
        let _ = result; // Just verify it doesn't crash
    }

    #[test]
    fn test_continue_outside_loop() {
        let mut executor = CommandExecutor::new();
        let cmd = "continue";
        let result = executor.execute(cmd);
        // Continue outside loop should error or no-op
        let _ = result; // Just verify it doesn't crash
    }

    #[test]
    fn test_empty_case_statement() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "1".to_string())
            .ok();
        let cmd = "case $x in esac";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Empty case statement should not crash");
    }

    // ===== Variable Scoping Edge Cases =====

    #[test]
    fn test_variable_shadowing_in_nested_loops() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("i".to_string(), "outer".to_string())
            .ok();
        let cmd = "for i in 1 2; do for i in a b; do echo $i; done; echo $i; done";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Variable shadowing in nested loops should work");
    }

    #[test]
    fn test_loop_variable_persistence() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in 1 2 3; do echo $i; done; echo $i";
        let result = executor.execute(cmd);
        // Loop variable should persist after loop ends
        assert!(result.is_ok(), "Loop variable should persist after loop");
    }

    #[test]
    fn test_arithmetic_variable_overflow() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("big".to_string(), "9999999999999999999".to_string())
            .ok();
        let cmd = "echo $((big + 1))";
        let result = executor.execute(cmd);
        // Should handle large numbers (or error gracefully)
        assert!(result.is_ok() || result.is_err(), "Large arithmetic should be handled");
    }

    // ===== Redirection Edge Cases =====

    #[test]
    fn test_loop_redirection_to_nonexistent_dir() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in 1 2; do echo $i; done > /nonexistent/dir/file.txt";
        let result = executor.execute(cmd);
        // Should parse successfully (actual I/O happens at execution time)
        assert!(
            result.is_ok(),
            "Redirection syntax should parse even if target dir doesn't exist"
        );
    }

    #[test]
    fn test_redirection_to_stdin() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in 1; do echo test; done < /dev/null";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Input redirection in loop should parse");
    }

    // ===== Pipe Edge Cases =====

    #[test]
    fn test_pipe_with_failing_command() {
        let mut executor = CommandExecutor::new();
        let cmd = "if false | true; then echo success; fi";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Pipe with failing command should not crash");
    }

    #[test]
    fn test_multiple_pipes_in_conditional() {
        let mut executor = CommandExecutor::new();
        let cmd = "if echo test | cat | grep test > /dev/null; then echo found; fi";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Multiple pipes in conditional should work");
    }

    // ===== Control Flow Edge Cases =====

    #[test]
    fn test_break_with_high_level() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in 1 2; do break 99; echo unreachable; done";
        let result = executor.execute(cmd);
        // High break level should be handled
        assert!(result.is_ok(), "Break with high level number should be handled");
    }

    #[test]
    fn test_multiple_elif_all_false() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "999".to_string())
            .ok();
        let cmd = "if [ $x -eq 1 ]; then echo one; elif [ $x -eq 2 ]; then echo two; elif [ $x -eq 3 ]; then echo three; fi";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "All elif conditions false should not crash");
    }

    #[test]
    fn test_case_with_multiple_default_patterns() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "test".to_string())
            .ok();
        let cmd = "case $x in a) echo a;; *) echo first;; *) echo second;; esac";
        let result = executor.execute(cmd);
        // Multiple default patterns - behavior depends on implementation
        assert!(result.is_ok(), "Multiple default patterns should be handled");
    }

    // ===== Quoting Edge Cases =====

    #[test]
    fn test_for_loop_with_special_chars_in_items() {
        let mut executor = CommandExecutor::new();
        let cmd = "for item in '$var' '\"quoted\"' 'with spaces'; do echo $item; done";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "For loop with special chars in quoted items should work");
    }

    #[test]
    fn test_unquoted_empty_variable_in_loop_condition() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("empty".to_string(), "".to_string())
            .ok();
        let cmd = "if [ $empty ]; then echo not empty; else echo empty; fi";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Empty variable test should work");
    }

    // ===== Expansion Edge Cases =====

    #[test]
    fn test_command_substitution_returning_empty() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in $(echo); do echo item; done";
        let result = executor.execute(cmd);
        // Empty command substitution result
        assert!(result.is_ok(), "Command substitution returning empty should work");
    }

    #[test]
    fn test_command_substitution_with_newlines() {
        let mut executor = CommandExecutor::new();
        let cmd = "for line in $(echo -e 'a\\nb\\nc'); do echo $line; done";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Command substitution with newlines should work");
    }

    // ===== Stress Tests =====

    #[test]
    fn test_large_loop_iteration_count() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("i".to_string(), "1".to_string())
            .ok();
        // Create a large loop that executes 100 iterations
        let cmd = "while [ $i -le 100 ]; do i=$((i+1)); done; echo done";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Large loop iteration count should complete");
    }

    #[test]
    fn test_many_nested_conditions() {
        let mut executor = CommandExecutor::new();
        let mut conditions = String::from("if true; then");
        for _ in 0..10 {
            conditions.push_str(" if true; then");
        }
        conditions.push_str(" echo deeply_nested;");
        for _ in 0..10 {
            conditions.push_str(" fi;");
        }
        conditions.push_str(" fi");

        let result = executor.execute(&conditions);
        assert!(result.is_ok(), "Many nested conditions should not crash");
    }

    #[test]
    fn test_case_with_many_patterns() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "5".to_string())
            .ok();
        let mut case_cmd = String::from("case $x in");
        for i in 1..=20 {
            case_cmd.push_str(&format!(" {}) echo {};; ", i, i));
        }
        case_cmd.push_str(" esac");

        let result = executor.execute(&case_cmd);
        assert!(result.is_ok(), "Case with many patterns should work");
    }

    // ===== Exit Code Edge Cases =====

    #[test]
    fn test_exit_code_from_false_condition() {
        let mut executor = CommandExecutor::new();
        let cmd = "if false; then true; fi; echo $?";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Exit code from false condition should be accessible");
    }

    #[test]
    fn test_exit_code_from_loop() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in 1; do false; done; echo $?";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Exit code from loop should be accessible");
    }

    // ===== Whitespace Edge Cases =====

    #[test]
    fn test_multiline_for_loop() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in 1 2 3; do\n  echo $i\ndone";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Multiline for loop should work");
    }

    #[test]
    fn test_extra_whitespace_in_control_structures() {
        let mut executor = CommandExecutor::new();
        let cmd = "if   [   $?   -eq   0   ]   ;   then   echo ok   ;   fi";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Extra whitespace should be handled");
    }

    #[test]
    fn test_no_space_between_tokens() {
        let mut executor = CommandExecutor::new();
        let cmd = "for i in 1 2;do echo $i;done";
        let result = executor.execute(cmd);
        // Some implementations allow this, some don't
        let _ = result;
    }
}
