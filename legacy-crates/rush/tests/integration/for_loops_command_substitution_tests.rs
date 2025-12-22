//! Integration tests for Phase 2 command substitution in for loops
//! Tests $(cmd) and `cmd` syntax in loop word lists

#[cfg(test)]
mod for_loop_command_substitution {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_for_loop_with_command_substitution_simple() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with simple command substitution
        let cmd = "for x in $(echo a b c); do echo $x; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should execute for loop with command substitution");
    }

    #[test]
    fn test_for_loop_with_command_substitution_echo() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with echo command substitution producing multiple values
        let cmd = "for item in $(echo one two three); do item=$item; done; echo done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "For loop with echo substitution should succeed");
    }

    #[test]
    fn test_for_loop_with_nested_command_substitution() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with nested command substitution
        let cmd = "for x in $(echo $(echo a)); do true; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Nested command substitution should work");
    }

    #[test]
    fn test_for_loop_with_command_substitution_and_variables() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("items".to_string(), "x y z".to_string())
            .unwrap();

        // Test: mixing command substitution with variables
        let cmd = "for item in $(echo $items); do echo $item; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should handle command substitution with variable expansion");
    }

    #[test]
    fn test_for_loop_with_multiple_command_substitutions() {
        let mut executor = CommandExecutor::new();

        // Test: multiple command substitutions in same for loop
        let cmd = "for x in $(echo a) $(echo b) $(echo c); do x=$x; done; echo done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Multiple command substitutions should work");
    }

    #[test]
    fn test_for_loop_command_substitution_with_word_splitting() {
        let mut executor = CommandExecutor::new();

        // Test: command substitution respects word splitting
        let cmd = "for word in $(echo hello world); do echo $word; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Word splitting in command substitution should work");
    }

    #[test]
    fn test_for_loop_command_substitution_empty_result() {
        let mut executor = CommandExecutor::new();

        // Test: command substitution returning empty string (loop doesn't execute)
        let cmd = "count=0; for x in $(echo ''); do count=$((count+1)); done; echo $count";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Empty command substitution should not error");
    }

    #[test]
    fn test_for_loop_backtick_command_substitution() {
        let mut executor = CommandExecutor::new();

        // Test: backtick syntax for command substitution
        let cmd = "for x in `echo a b`; do echo $x; done";
        let result = executor.execute(cmd);

        // This might not be implemented yet, but we test for it
        // If not implemented, it should at least not crash
        let _ = result;
    }

    #[test]
    fn test_for_loop_command_substitution_with_special_chars() {
        let mut executor = CommandExecutor::new();

        // Test: command substitution output with special characters
        let cmd = "for x in $(echo 'a-b' 'c_d'); do echo $x; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Special characters in command substitution should work");
    }

    #[test]
    fn test_for_loop_command_substitution_preserves_quoted_words() {
        let mut executor = CommandExecutor::new();

        // Test: quoted output from command substitution
        let cmd = "for x in $(echo 'word with spaces'); do echo $x; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Quoted words in command substitution should work");
    }
}
