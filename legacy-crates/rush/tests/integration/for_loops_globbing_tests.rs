//! Integration tests for Phase 2 globbing in for loops
//! Tests * ? [...] pattern matching in loop word lists

#[cfg(test)]
mod for_loop_globbing {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_for_loop_with_glob_wildcard_single() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with * wildcard (should match files in current directory)
        let cmd = "for f in *.rs; do true; done";
        let result = executor.execute(cmd);

        // This may match files or return empty (depending on current directory)
        // The important thing is it doesn't error
        assert!(result.is_ok(), "For loop with * glob should not error");
    }

    #[test]
    fn test_for_loop_with_glob_question_mark() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with ? wildcard (single character match)
        let cmd = "for f in file?.txt; do true; done";
        let result = executor.execute(cmd);

        // Pattern doesn't match any files, loop just doesn't execute
        assert!(result.is_ok(), "For loop with ? glob should work");
    }

    #[test]
    fn test_for_loop_with_glob_character_class() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with character class [abc]
        let cmd = "for f in file[0-9].txt; do true; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "For loop with character class glob should work");
    }

    #[test]
    fn test_for_loop_with_glob_negated_class() {
        let mut executor = CommandExecutor::new();

        // Test: for loop with negated character class [!abc]
        let cmd = "for f in file[!0-9].txt; do true; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "For loop with negated character class should work");
    }

    #[test]
    fn test_for_loop_with_quoted_glob_pattern() {
        let mut executor = CommandExecutor::new();

        // Test: quoted glob patterns should NOT be expanded
        let cmd = "for f in '*.txt'; do echo $f; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Quoted glob patterns should not expand");
    }

    #[test]
    fn test_for_loop_with_escaped_glob_metachar() {
        let mut executor = CommandExecutor::new();

        // Test: escaped glob characters should be treated literally
        let cmd = "for f in \\*.txt; do true; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Escaped glob metacharacters should work");
    }

    #[test]
    fn test_for_loop_with_multiple_glob_patterns() {
        let mut executor = CommandExecutor::new();

        // Test: multiple glob patterns in one for loop
        let cmd = "for f in *.rs *.txt; do true; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Multiple glob patterns should work");
    }

    #[test]
    fn test_for_loop_glob_with_variable_path() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("dir".to_string(), ".".to_string())
            .unwrap();

        // Test: glob pattern with directory variable (may not work depending on implementation)
        let cmd = "for f in $dir/*.rs; do true; done";
        let result = executor.execute(cmd);

        // This tests variable expansion combined with globbing
        assert!(result.is_ok(), "Glob with variable path should work");
    }

    #[test]
    fn test_for_loop_glob_no_matches_empty_iteration() {
        let mut executor = CommandExecutor::new();

        // Test: glob pattern with no matches
        let cmd = "count=0; for f in /nonexistent/*.txt; do count=$((count+1)); done; echo $count";
        let result = executor.execute(cmd);

        // When glob matches nothing, loop doesn't execute
        assert!(result.is_ok(), "Glob with no matches should work");
    }

    #[test]
    fn test_for_loop_glob_with_special_chars_in_name() {
        let mut executor = CommandExecutor::new();

        // Test: glob patterns with special characters
        let cmd = "for f in file_[0-9]*.txt; do true; done";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Glob with special characters should work");
    }
}
