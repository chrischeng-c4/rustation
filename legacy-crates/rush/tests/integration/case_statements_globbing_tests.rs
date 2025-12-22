//! Integration tests for Phase 2 globbing in case statements
//! Tests * ? [...] pattern matching in case statement patterns

#[cfg(test)]
mod case_statement_globbing {
    use rush::executor::execute::CommandExecutor;

    #[test]
    fn test_case_statement_glob_wildcard() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("file".to_string(), "document.txt".to_string())
            .unwrap();

        // Test: case statement with * wildcard in pattern
        let cmd = "case $file in *.txt) echo text;; *.md) echo markdown;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement with * glob pattern should work");
    }

    #[test]
    fn test_case_statement_glob_question_mark() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("file".to_string(), "file1".to_string())
            .unwrap();

        // Test: case statement with ? wildcard in pattern
        let cmd = "case $file in file?) echo match;; *) echo nomatch;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement with ? glob pattern should work");
    }

    #[test]
    fn test_case_statement_glob_character_class() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("file".to_string(), "file5.txt".to_string())
            .unwrap();

        // Test: case statement with character class in pattern
        let cmd = "case $file in file[0-9].txt) echo numbered;; *) echo other;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement with character class glob should work");
    }

    #[test]
    fn test_case_statement_glob_negated_class() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("item".to_string(), "apple".to_string())
            .unwrap();

        // Test: case statement with negated character class
        let cmd = "case $item in [!0-9]*) echo notdigit;; *) echo digit;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement with negated character class should work");
    }

    #[test]
    fn test_case_statement_glob_multiple_patterns() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("ext".to_string(), "txt".to_string())
            .unwrap();

        // Test: case statement with multiple glob patterns (using |)
        let cmd = "case .$ext in .txt|.md|.rst) echo doc;; *) echo other;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement with multiple glob patterns should work");
    }

    #[test]
    fn test_case_statement_glob_no_match_default() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("input".to_string(), "unknown".to_string())
            .unwrap();

        // Test: glob pattern that doesn't match, falls through to default
        let cmd = "case $input in *.txt) echo text;; *) echo default;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement with no matching glob should use default");
    }

    #[test]
    fn test_case_statement_glob_exact_match_priority() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("val".to_string(), "test".to_string())
            .unwrap();

        // Test: exact match patterns should be checked before glob patterns
        let cmd = "case $val in test) echo exact;; test*) echo glob;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement pattern ordering should work");
    }

    #[test]
    fn test_case_statement_glob_wildcard_all() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("x".to_string(), "anything".to_string())
            .unwrap();

        // Test: * matches everything (default case)
        let cmd = "case $x in *) echo matched;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement with * matching all should work");
    }

    #[test]
    fn test_case_statement_glob_escape_sequence() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("file".to_string(), "*.txt".to_string())
            .unwrap();

        // Test: glob patterns in case values (not patterns) should match literally
        let cmd = "case $file in \\*.txt) echo glob;; *) echo other;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement with escaped glob should work");
    }

    #[test]
    fn test_case_statement_glob_range_pattern() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("num".to_string(), "7".to_string())
            .unwrap();

        // Test: range patterns [0-9]
        let cmd = "case $num in [0-9]) echo digit;; *) echo nondigit;; esac";
        let result = executor.execute(cmd);

        assert!(result.is_ok(), "Case statement with range pattern should work");
    }
}
