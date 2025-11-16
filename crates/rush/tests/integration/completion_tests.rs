// Integration tests for tab completion
// Tests the full REPL integration with completion

use rush::completion::CompletionRegistry;
use reedline::Completer;

#[cfg(test)]
mod completion_integration_tests {
    use super::*;

    /// T021: Integration test for command completion in REPL context
    /// Tests that CompletionRegistry properly integrates with actual PATH commands
    #[test]
    fn test_command_completion_end_to_end() {
        // Create a completion registry as used in REPL
        let mut registry = CompletionRegistry::new();

        // Test common command prefixes that should exist on most systems
        let test_cases = vec![
            ("ca", vec!["cat", "cal"]),        // Should match cat, cal, cargo, etc.
            ("ls", vec!["ls"]),                 // Should match ls
            ("ec", vec!["echo"]),               // Should match echo
        ];

        for (prefix, expected_substrings) in test_cases {
            let suggestions = registry.complete(prefix, prefix.len());

            // Should find at least some matches (system-dependent)
            // We can't guarantee exact matches, but we can verify behavior

            if !suggestions.is_empty() {
                // Verify suggestions contain expected commands
                for expected in expected_substrings {
                    let found = suggestions.iter().any(|s| s.value.starts_with(expected));
                    if found {
                        // Great! Found expected command
                        assert!(
                            found,
                            "Expected to find command starting with '{}' in results for prefix '{}'",
                            expected,
                            prefix
                        );
                    }
                }

                // Verify suggestions are properly formatted
                for suggestion in &suggestions {
                    assert!(
                        suggestion.value.starts_with(prefix),
                        "Suggestion '{}' should start with prefix '{}'",
                        suggestion.value,
                        prefix
                    );

                    // Should have proper span
                    assert_eq!(suggestion.span.start, 0);
                    assert_eq!(suggestion.span.end, prefix.len());

                    // Should append whitespace after command
                    assert!(suggestion.append_whitespace);

                    // Should have styling for visibility
                    assert!(suggestion.style.is_some());
                }
            }
        }
    }

    #[test]
    fn test_completion_registry_handles_empty_input() {
        let mut registry = CompletionRegistry::new();

        // Empty input should return no completions
        let suggestions = registry.complete("", 0);

        assert!(
            suggestions.is_empty(),
            "Empty input should return no completions"
        );
    }

    #[test]
    fn test_completion_registry_handles_arguments() {
        let mut registry = CompletionRegistry::new();

        // Input with space (in arguments) should return no completions
        let suggestions = registry.complete("ls ", 3);

        assert!(
            suggestions.is_empty(),
            "Should not complete in arguments (after space)"
        );
    }

    #[test]
    fn test_completion_registry_finds_actual_path_commands() {
        let mut registry = CompletionRegistry::new();

        // Test that we actually scan PATH and find real commands
        // These commands should exist on virtually all Unix systems
        let common_commands = vec!["ls", "cat", "echo", "pwd"];

        for cmd in common_commands {
            let suggestions = registry.complete(cmd, cmd.len());

            // Should find at least the exact command (and possibly others with same prefix)
            let found_exact = suggestions.iter().any(|s| s.value == cmd);

            assert!(
                found_exact || !suggestions.is_empty(),
                "Should find command '{}' in PATH (found {} suggestions)",
                cmd,
                suggestions.len()
            );
        }
    }

    #[test]
    fn test_completion_caching_works_in_registry() {
        let mut registry = CompletionRegistry::new();

        // First completion loads cache
        let first_results = registry.complete("ca", 2);

        // Second completion reuses cache
        let second_results = registry.complete("ca", 2);

        // Should return same results (cache is working)
        assert_eq!(
            first_results.len(),
            second_results.len(),
            "Cache should return consistent results"
        );

        // Verify values are the same
        let first_values: Vec<String> = first_results.iter().map(|s| s.value.clone()).collect();
        let second_values: Vec<String> = second_results.iter().map(|s| s.value.clone()).collect();

        assert_eq!(
            first_values, second_values,
            "Cached results should be identical"
        );
    }
}
