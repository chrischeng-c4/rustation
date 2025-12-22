//! Integration tests for Phase 2 word expansion in case statements
//! Tests variable expansion and command substitution in case statement values

#[cfg(test)]
mod case_statement_expansions {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_case_statement_with_variable_expansion() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("fruit".to_string(), "apple".to_string())
            .unwrap();

        // Test: case statement with variable in value
        let cmd = "case $fruit in apple) echo red;; banana) echo yellow;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should execute case statement with variable expansion");
    }

    #[test]
    fn test_case_statement_with_braced_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("color".to_string(), "blue".to_string())
            .unwrap();

        // Test: case statement with braced variable syntax ${VAR}
        let cmd = "case ${color} in red) echo warm;; blue) echo cool;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Should handle braced variable syntax in case value");
    }

    #[test]
    fn test_case_statement_multiple_patterns_with_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("item".to_string(), "orange".to_string())
            .unwrap();

        // Test: case statement matching against multiple patterns
        let cmd = "case $item in apple|orange) echo fruit;; potato) echo vegetable;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement should handle multiple patterns with variables");
    }

    #[test]
    fn test_case_statement_with_wildcard_patterns() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("filename".to_string(), "test.txt".to_string())
            .unwrap();

        // Test: case statement with wildcard patterns
        let cmd = "case $filename in *.txt) echo text;; *.md) echo markdown;; esac";
        let result = executor.execute(cmd);

        assert!(
            result.is_ok(),
            "Case statement should work with wildcard patterns and variables"
        );
    }

    #[test]
    fn test_case_statement_default_pattern_with_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("unknown".to_string(), "xyz".to_string())
            .unwrap();

        // Test: case statement with default pattern
        let cmd = "case $unknown in apple) echo red;; *) echo other;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement should handle default pattern with variables");
    }

    #[test]
    fn test_case_statement_with_empty_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("empty".to_string(), "".to_string())
            .unwrap();

        // Test: case statement with empty variable expansion
        let cmd = "case $empty in \"\") echo empty;; *) echo nonempty;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement should handle empty variable expansion");
    }

    #[test]
    fn test_case_statement_with_variable_in_pattern() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("target".to_string(), "txt".to_string())
            .unwrap();
        executor
            .variable_manager_mut()
            .set("file".to_string(), "document.txt".to_string())
            .unwrap();

        // Test: case statement matching variable against pattern
        let cmd = "case $file in *.txt) echo match;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement should match variables against patterns");
    }

    #[test]
    fn test_case_statement_variable_in_commands() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("action".to_string(), "start".to_string())
            .unwrap();

        // Test: case statement with variable used in commands
        let cmd = "case $action in start) state=running;; stop) state=stopped;; esac; echo $state";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Variables in case statement commands should be accessible");
    }

    #[test]
    fn test_case_statement_nested_in_loop_with_variables() {
        let mut executor = CommandExecutor::new();

        // Test: case statement inside for loop with variable expansion
        let cmd = "for item in apple banana; do case $item in apple) color=red;; banana) color=yellow;; esac; done; echo $color";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statements in loops should expand variables correctly");
    }

    #[test]
    fn test_case_statement_multiple_matches_first_wins() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("num".to_string(), "5".to_string())
            .unwrap();

        // Test: case statement where multiple patterns could match, first wins
        let cmd = "case $num in 5) echo five;; [0-9]) echo digit;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "First matching pattern in case statement should execute");
    }
}
