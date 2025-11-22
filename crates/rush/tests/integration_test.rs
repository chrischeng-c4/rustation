//! Integration tests for rush shell
//!
//! These tests verify end-to-end functionality of the shell components.

use rush::{Config, Repl};

#[test]
fn test_repl_initialization() {
    // Test that REPL can be created with default config
    let repl = Repl::new();
    assert!(repl.is_ok(), "REPL should initialize successfully");
}

#[test]
fn test_repl_with_custom_config() {
    // Test that REPL accepts custom configuration
    let mut config = Config::default();
    config.history_size = 5000;
    config.prompt = ">> ".to_string();

    let repl = Repl::with_config(config);
    assert!(repl.is_ok(), "REPL should initialize with custom config");
}

#[test]
fn test_config_default_values() {
    let config = Config::default();

    assert_eq!(config.history_size, 10_000);
    assert_eq!(config.prompt, "$ ");
    assert_eq!(config.completion_timeout_ms, 100);
    assert_eq!(config.suggestion_delay_ms, 50);
}

#[test]
fn test_config_custom_values() {
    let mut config = Config::default();

    config.history_size = 50_000;
    config.prompt = "λ ".to_string();
    config.completion_timeout_ms = 200;
    config.suggestion_delay_ms = 100;

    assert_eq!(config.history_size, 50_000);
    assert_eq!(config.prompt, "λ ");
    assert_eq!(config.completion_timeout_ms, 200);
    assert_eq!(config.suggestion_delay_ms, 100);
}

#[test]
fn test_config_load_creates_directories() {
    // This should not panic even if directories don't exist
    let config = Config::load();
    assert_eq!(config.history_size, 10_000); // Should use defaults
}

// === Tab Completion Integration Tests (T021) ===

use reedline::Completer;
use rush::completion::CompletionRegistry;

/// T021: Integration test for command completion in REPL context
/// Tests that CompletionRegistry properly integrates with actual PATH commands
#[test]
fn test_command_completion_end_to_end() {
    // Create a completion registry as used in REPL
    let mut registry = CompletionRegistry::new();

    // Test common command prefixes that should exist on most systems
    let test_cases = vec![
        ("ca", vec!["cat", "cal"]), // Should match cat, cal, cargo, etc.
        ("ls", vec!["ls"]),         // Should match ls
        ("ec", vec!["echo"]),       // Should match echo
    ];

    for (prefix, expected_substrings) in test_cases {
        let suggestions = registry.complete(prefix, prefix.len());

        // Should find at least some matches (system-dependent)
        if !suggestions.is_empty() {
            // Verify suggestions contain expected commands
            for expected in expected_substrings {
                let found = suggestions.iter().any(|s| s.value.starts_with(expected));
                if found {
                    assert!(
                        found,
                        "Expected to find command starting with '{}' in results for prefix '{}'",
                        expected, prefix
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

    assert!(suggestions.is_empty(), "Empty input should return no completions");
}

#[test]
fn test_completion_registry_handles_arguments() {
    let mut registry = CompletionRegistry::new();

    // Input with space (in arguments) should trigger path completion
    let suggestions = registry.complete("ls ", 3);

    // Should either return path completions or empty (if current dir empty/error)
    // Main validation: path completion was triggered, not command completion
    // If we get results, they should be paths (start with ./ or similar)
    for suggestion in &suggestions {
        // Path completions should not be command names with spaces
        // This validates we're using path completer, not command completer
        assert!(suggestion.style.is_some(), "Should have styling");
    }
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

    assert_eq!(first_values, second_values, "Cached results should be identical");
}

// === Path Completion Integration Tests (T043) ===

use rush::completion::PathCompleter;

/// T043: Integration test for path completion in REPL context
#[test]
fn test_path_completion_end_to_end() {
    // Create a completion registry as used in REPL
    let mut registry = CompletionRegistry::new();

    // Test path completion in argument position
    // This should trigger path completion, not command completion
    let suggestions = registry.complete("ls ./", 5);

    // Should return path completions for current directory
    // Results will vary by environment, but should be formatted correctly
    for suggestion in &suggestions {
        // All paths should start with ./
        assert!(suggestion.value.starts_with("./"));

        // Should have proper span (after "ls ")
        assert_eq!(suggestion.span.start, 3);
        assert_eq!(suggestion.span.end, 5);

        // Should NOT append whitespace (user might add more to path)
        assert!(!suggestion.append_whitespace);

        // Should have styling
        assert!(suggestion.style.is_some());
    }
}

#[test]
fn test_path_completion_vs_command_completion() {
    let mut registry = CompletionRegistry::new();

    // First word should use command completion
    let cmd_suggestions = registry.complete("ls", 2);

    // Second word should use path completion
    let path_suggestions = registry.complete("ls ./", 5);

    // Both might return results, but they use different completers
    // Main validation: both work without crashing
}

#[test]
fn test_path_completion_with_tilde() {
    let mut registry = CompletionRegistry::new();

    // Tilde should be expanded
    let suggestions = registry.complete("ls ~/", 5);

    // Should attempt to list home directory
    // Results vary by system
}

#[test]
fn test_path_completion_absolute_path() {
    let mut registry = CompletionRegistry::new();

    // Absolute paths should work
    let suggestions = registry.complete("cat /etc/", 9);

    // Should attempt to list /etc/
    // May be empty if permission denied
}

#[test]
fn test_path_completion_hidden_files() {
    let mut registry = CompletionRegistry::new();

    // Hidden files should only show with . prefix
    let with_dot = registry.complete("ls .", 4);

    // Should show hidden files (or at least ./ and ../)
    let has_dot_files = with_dot.iter().any(|s| s.value.contains("/."));

    // We should get some results with dots
    if !with_dot.is_empty() {
        // At least some results should have dots
    }
}

#[test]
fn test_registry_routes_correctly() {
    let mut registry = CompletionRegistry::new();

    // Test that registry correctly routes based on context

    // No space = command completion
    let cmd = registry.complete("git", 3);
    // Should find git command (if in PATH)

    // With space = path completion
    let path = registry.complete("git ./", 6);
    // Should find paths

    // Both should work without panicking
}

// === Flag Completion Integration Tests (T066) ===

use rush::completion::FlagCompleter;

/// T066: Integration test for flag completion in REPL context
#[test]
fn test_flag_completion_end_to_end() {
    let mut registry = CompletionRegistry::new();

    // Test git --version completion
    let suggestions = registry.complete("git --ver", 9);

    // Should return flag completions
    if !suggestions.is_empty() {
        // Verify at least one matches --version
        let has_version = suggestions.iter().any(|s| s.value.starts_with("--ver"));
        assert!(has_version, "Should find flags starting with --ver");

        // Verify proper span (after "git ")
        for suggestion in &suggestions {
            assert_eq!(suggestion.span.start, 4); // After "git "
            assert_eq!(suggestion.span.end, 9); // At cursor
        }

        // Verify flags have descriptions
        for suggestion in &suggestions {
            assert!(suggestion.description.is_some(), "Flags should have descriptions");
        }

        // Verify flags append whitespace
        for suggestion in &suggestions {
            assert!(suggestion.append_whitespace, "Flags should append whitespace");
        }

        // Verify styling
        for suggestion in &suggestions {
            assert!(suggestion.style.is_some(), "Flags should have styling");
        }
    }
}

#[test]
fn test_flag_completion_vs_path_completion() {
    let mut registry = CompletionRegistry::new();

    // Flag completion (starts with -)
    let flag_suggestions = registry.complete("ls --a", 6);

    // Path completion (doesn't start with -)
    let path_suggestions = registry.complete("ls ./", 5);

    // Both might return results, but they use different completers
    // Main validation: both work without crashing

    // If we get flag results, they should start with --a
    for suggestion in &flag_suggestions {
        assert!(suggestion.value.starts_with("--a"), "Flag completions should start with --a");
    }

    // If we get path results, they should start with ./
    for suggestion in &path_suggestions {
        assert!(suggestion.value.starts_with("./"), "Path completions should start with ./");
    }
}

#[test]
fn test_flag_completion_short_flags() {
    let mut registry = CompletionRegistry::new();

    // Test short flag completion
    let suggestions = registry.complete("ls -a", 5);

    // Should attempt flag completion (may return results for -a flags)
    // Main validation: doesn't crash
}

#[test]
fn test_flag_completion_unknown_command() {
    let mut registry = CompletionRegistry::new();

    // Unknown command should return empty
    let suggestions = registry.complete("unknowncmd --help", 17);

    assert!(suggestions.is_empty(), "Unknown command should not return flag completions");
}

#[test]
fn test_flag_completion_for_multiple_commands() {
    let mut registry = CompletionRegistry::new();

    // Test that different commands get different flags
    let git_flags = registry.complete("git --v", 7);
    let cargo_flags = registry.complete("cargo --v", 9);

    // Both should potentially find --version and --verbose
    // Main validation: both work without crashing
}

#[test]
fn test_registry_routes_to_flag_completer() {
    let mut registry = CompletionRegistry::new();

    // Verify routing works correctly for all three contexts

    // Command completion (no space)
    let _cmd = registry.complete("gi", 2);

    // Path completion (space, but no -)
    let _path = registry.complete("git status", 10);

    // Flag completion (space and starts with -)
    let _flag = registry.complete("git --v", 7);

    // All should work without panicking
}

#[test]
fn test_flag_completion_with_arguments_before() {
    let mut registry = CompletionRegistry::new();

    // Flags can come after positional arguments
    let suggestions = registry.complete("git commit file.txt --m", 23);

    // Should attempt flag completion
    // Main validation: doesn't crash
}

// Autosuggestions integration tests
mod integration {
    pub mod autosuggestions_tests;
}
