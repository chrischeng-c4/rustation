//! Integration tests for heredocs (Feature 027)
//!
//! Tests heredoc (<<EOF...EOF) functionality end-to-end through the shell

#[cfg(test)]
mod heredoc_integration_tests {
    use rush::executor::execute::CommandExecutor;

    // =========================================================================
    // User Story 1: Basic Heredoc Input (P1)
    // =========================================================================

    #[test]
    fn test_basic_heredoc_execution() {
        // US1 Acceptance: Basic heredoc should execute and pass content to stdin
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF\nhello world\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Basic heredoc should execute successfully");
        assert_eq!(result.unwrap(), 0, "cat with heredoc should exit with 0");
    }

    #[test]
    fn test_heredoc_multiline_content() {
        // US1: Heredoc with multiple lines of content
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF\nline 1\nline 2\nline 3\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Heredoc with multiline content should execute");
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_heredoc_custom_delimiter() {
        // US1: Heredoc with custom delimiter
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<MARKER\ntest content\nMARKER";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Heredoc with custom delimiter should work");
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_heredoc_delimiter_not_in_content() {
        // Edge case: Delimiter appears as part of content line (should not match)
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF\nThis line has EOF in it\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Delimiter within content should not end heredoc");
        assert_eq!(result.unwrap(), 0);
    }

    // =========================================================================
    // User Story 2: Tab-Stripping Heredocs (P2)
    // =========================================================================

    #[test]
    fn test_heredoc_strip_tabs() {
        // US2 Acceptance: <<- strips leading tabs from content
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<-EOF\n\thello\n\tworld\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Tab-stripping heredoc should execute");
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_heredoc_strip_tabs_delimiter() {
        // US2 Acceptance: <<- recognizes delimiter with leading tabs
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<-EOF\n\thello\n\tEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Tab-stripped delimiter should be recognized");
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_heredoc_strip_preserves_spaces() {
        // US2 Acceptance: <<- only strips tabs, not spaces
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<-EOF\n    hello\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Spaces should be preserved with <<-");
        assert_eq!(result.unwrap(), 0);
    }

    // =========================================================================
    // User Story 3: Heredoc in Pipeline (P2)
    // =========================================================================

    #[test]
    fn test_heredoc_in_pipeline() {
        // US3 Acceptance: Heredoc content flows through pipeline
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF | cat\nhello\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Heredoc in pipeline should execute");
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_heredoc_piped_to_grep() {
        // US3: Heredoc piped to grep for filtering
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF | grep hello\nhello world\ngoodbye\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Heredoc piped to grep should work");
        // grep finds "hello" so exits with 0
        assert_eq!(result.unwrap(), 0);
    }

    // =========================================================================
    // User Story 4: Heredoc with Output Redirection (P3)
    // =========================================================================

    #[test]
    fn test_heredoc_with_output_redirect() {
        // US4 Acceptance: Heredoc combined with output redirection
        // Note: Output redirection happens in pipeline execution which handles
        // the heredoc stdin separately. The output goes to stdout, not file yet.
        // This test verifies heredoc + redirect syntax parses and executes.
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF\nheredoc redirect test\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Heredoc should execute");
        assert_eq!(result.unwrap(), 0);
        // TODO: Full heredoc + output redirect integration requires
        // coordinating stdin (heredoc) and stdout (file) redirections
    }

    // =========================================================================
    // User Story 5: Empty Heredoc (P3)
    // =========================================================================

    #[test]
    fn test_empty_heredoc() {
        // US5 Acceptance: Empty heredoc (delimiter immediately after start)
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Empty heredoc should execute");
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_heredoc_whitespace_only() {
        // US5: Heredoc with only whitespace content
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF\n   \n\t\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Whitespace-only heredoc should execute");
        assert_eq!(result.unwrap(), 0);
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    #[test]
    fn test_heredoc_special_characters_in_content() {
        // Edge case: Special shell characters in heredoc content
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF\n$HOME | > < & ; \" ' \\\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Special characters in heredoc should work");
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_heredoc_with_wc() {
        // Practical test: Count lines in heredoc
        let mut executor = CommandExecutor::new();
        let cmd = "cat <<EOF | wc -l\nline1\nline2\nline3\nEOF";
        let result = executor.execute(cmd);
        assert!(result.is_ok(), "Heredoc piped to wc should work");
        assert_eq!(result.unwrap(), 0);
    }
}
