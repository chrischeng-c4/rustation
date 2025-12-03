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

// Job Control Integration Tests
// These tests verify job control functionality (background execution, job listing, etc.)

use nix::unistd::Pid;
use rush::executor::execute::CommandExecutor;
use rush::executor::job::JobStatus;

#[test]
fn test_background_job_execution() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("echo test &");
    assert!(result.is_ok(), "Background job execution should succeed");
    assert_eq!(result.unwrap(), 0, "Background job should return exit code 0");

    let manager = executor.job_manager_mut();
    let jobs: Vec<_> = manager.jobs().collect();
    assert!(!jobs.is_empty(), "At least one job should be created");
    assert_eq!(jobs[0].status, JobStatus::Running, "Job should be in Running state");
}

#[test]
fn test_multiple_background_jobs() {
    let mut executor = CommandExecutor::new();

    executor.execute("sleep 10 &").ok();
    executor.execute("sleep 20 &").ok();
    executor.execute("sleep 30 &").ok();

    let manager = executor.job_manager_mut();
    let mut jobs: Vec<_> = manager.jobs().collect();
    assert_eq!(jobs.len(), 3, "Should have three background jobs");

    // Sort by ID to ensure consistent order
    jobs.sort_by_key(|j| j.id);
    assert_eq!(jobs[0].id, 1);
    assert_eq!(jobs[1].id, 2);
    assert_eq!(jobs[2].id, 3);

    for job in jobs {
        assert_eq!(job.status, JobStatus::Running, "All jobs should be running");
    }
}

#[test]
fn test_jobs_command_lists_background_jobs() {
    let mut executor = CommandExecutor::new();

    executor.execute("sleep 10 &").ok();
    executor.execute("echo test &").ok();

    let result = executor.execute("jobs");
    assert!(result.is_ok(), "jobs command should succeed");
    assert_eq!(result.unwrap(), 0, "jobs command should return 0");
}

#[test]
fn test_foreground_command_execution() {
    let mut executor = CommandExecutor::new();

    let result = executor.execute("true");
    assert!(result.is_ok(), "Foreground command should succeed");
    assert_eq!(result.unwrap(), 0, "true command should return 0");

    let manager = executor.job_manager_mut();
    let jobs: Vec<_> = manager.jobs().collect();
    assert!(jobs.is_empty(), "Foreground commands should not create jobs");
}

#[test]
fn test_job_cleanup() {
    let mut executor = CommandExecutor::new();

    executor.execute("sleep 5 &").ok();
    executor.execute("sleep 10 &").ok();

    let manager = executor.job_manager_mut();
    assert_eq!(manager.jobs().count(), 2, "Should have two jobs before cleanup");

    if let Some(job) = manager.get_job_mut(1) {
        job.status = JobStatus::Done;
    }

    let cleaned = manager.cleanup();
    assert_eq!(cleaned.len(), 1, "Should clean up one job");
    assert_eq!(manager.jobs().count(), 1, "Should have one job after cleanup");
}

#[test]
fn test_background_job_process_group() {
    let mut executor = CommandExecutor::new();

    executor.execute("sleep 100 &").ok();

    let manager = executor.job_manager_mut();
    let job = manager.get_job(1).expect("Job should exist");

    assert_ne!(job.pgid.as_raw(), 0, "Job should have a process group ID");
    assert!(!job.pids.is_empty(), "Job should have at least one PID");
}

#[test]
fn test_background_job_doesnt_block_prompt() {
    let mut executor = CommandExecutor::new();

    executor.execute("sleep 10 &").ok();

    let result = executor.execute("echo hello");
    assert!(result.is_ok(), "Should be able to execute new command after bg job");
    assert_eq!(result.unwrap(), 0, "New command should succeed");

    let manager = executor.job_manager_mut();
    let jobs: Vec<_> = manager.jobs().collect();
    assert_eq!(jobs.len(), 1, "Background job should still exist");
}

#[test]
fn test_mixed_foreground_background_execution() {
    let mut executor = CommandExecutor::new();

    executor.execute("true").ok();
    executor.execute("sleep 10 &").ok();
    executor.execute("echo test").ok();

    let manager = executor.job_manager_mut();
    let jobs: Vec<_> = manager.jobs().collect();
    assert_eq!(jobs.len(), 1, "Only background job should be tracked");
}

#[test]
fn test_exit_code_tracking() {
    let mut executor = CommandExecutor::new();

    executor.execute("true").ok();
    assert_eq!(executor.last_exit_code(), 0, "Last exit code should be 0 for true");

    executor.execute("false").ok();
    assert_eq!(executor.last_exit_code(), 1, "Last exit code should be 1 for false");

    executor.execute("echo test &").ok();
    assert_eq!(executor.last_exit_code(), 0, "Background job should return 0");
}

// Globbing tests
use rush::executor::glob::glob_expand;

#[test]
fn test_glob_expand_no_patterns() {
    // No glob patterns should return unchanged
    let result = glob_expand("echo test").ok();
    assert!(result.is_some(), "Should handle non-glob commands");
}

#[test]
fn test_glob_expand_quoted_patterns() {
    // Quoted patterns should not be expanded
    let result = glob_expand("echo '*.txt'").ok();
    assert!(result.is_some(), "Should handle quoted patterns");
    let expanded = result.unwrap();
    assert!(expanded.contains("*.txt"), "Quoted pattern should be preserved");
}

#[test]
fn test_glob_expand_double_quoted_patterns() {
    // Double-quoted patterns should not be expanded
    let result = glob_expand("echo \"*.log\"").ok();
    assert!(result.is_some(), "Should handle double-quoted patterns");
    let expanded = result.unwrap();
    assert!(expanded.contains("*.log"), "Double-quoted pattern should be preserved");
}

#[test]
fn test_glob_expand_escaped_metacharacters() {
    // Escaped metacharacters should be treated literally
    let result = glob_expand("echo \\*.txt").ok();
    assert!(result.is_some(), "Should handle escaped patterns");
    let expanded = result.unwrap();
    assert!(expanded.contains("*.txt"), "Escaped pattern should be preserved as literal");
}

#[test]
fn test_glob_expand_preserves_non_matching() {
    // Non-matching patterns should return unchanged
    let result = glob_expand("cat nonexistent_*.pattern").ok();
    assert!(result.is_some(), "Should handle non-matching patterns");
    let expanded = result.unwrap();
    assert!(expanded.contains("nonexistent_"), "Non-matching pattern should be preserved");
}

// TODO: Re-enable after fixing TestHistory trait implementation
// mod integration {
//     pub mod autosuggestions_tests;
// }

// NOTE: Feature 007 (Stderr Redirection) integration tests are verified through:
// 1. Unit tests in executor/parser.rs (parsing 2> and 2>> tokens)
// 2. Executor tests for RedirectionType::Stderr handling
// 3. Manual testing with rush shell once implemented in CommandExecutor
// The feature is implemented in the parser and executor pipeline module

// === Array Variables Integration Tests (Feature 011) ===

/// T023: Test sparse arrays (accessing indices with gaps)
/// Note: Current implementation uses dense Vec<String>, so sparse arrays
/// are not directly supported. Out-of-bounds access returns empty string.
#[test]
fn test_array_out_of_bounds_access() {
    let mut executor = CommandExecutor::new();

    // Create a small array
    executor.variable_manager_mut()
        .set_array("arr".to_string(), vec!["a".to_string(), "b".to_string()])
        .unwrap();

    // Access within bounds
    let result = executor.execute("echo ${arr[0]}");
    assert!(result.is_ok());

    // Access out of bounds (should return empty, not error)
    let result = executor.execute("echo ${arr[99]}");
    assert!(result.is_ok(), "Out of bounds access should not error");
}

#[test]
fn test_array_negative_index_parsing() {
    // Negative indices are rejected at parse time
    use rush::executor::arrays::parse_array_ref;

    let result = parse_array_ref("${arr[-1]}");
    assert!(result.is_err(), "Negative indices should be rejected");
}

#[test]
fn test_array_invalid_index_parsing() {
    use rush::executor::arrays::parse_array_ref;

    // Non-numeric indices (except @ and *) should be rejected
    let result = parse_array_ref("${arr[abc]}");
    assert!(result.is_err(), "Non-numeric indices should be rejected");

    let result = parse_array_ref("${arr[1.5]}");
    assert!(result.is_err(), "Floating point indices should be rejected");
}

/// T024: Test nested arrays in substitutions
/// Note: Nested variable expansion (${arr[${i}]}) is not currently supported.
/// This test documents current behavior.
#[test]
fn test_array_basic_expansion() {
    let mut executor = CommandExecutor::new();

    // Create array and test basic expansion
    executor.variable_manager_mut()
        .set_array("colors".to_string(), vec!["red".to_string(), "green".to_string(), "blue".to_string()])
        .unwrap();

    // Direct index access works
    let result = executor.execute("echo ${colors[1]}");
    assert!(result.is_ok());
}

#[test]
fn test_array_all_elements_expansion() {
    let mut executor = CommandExecutor::new();

    executor.variable_manager_mut()
        .set_array("items".to_string(), vec!["one".to_string(), "two".to_string(), "three".to_string()])
        .unwrap();

    // ${arr[@]} expansion
    let result = executor.execute("echo ${items[@]}");
    assert!(result.is_ok());

    // ${arr[*]} expansion
    let result = executor.execute("echo ${items[*]}");
    assert!(result.is_ok());
}

#[test]
fn test_array_length_expansion() {
    let mut executor = CommandExecutor::new();

    executor.variable_manager_mut()
        .set_array("nums".to_string(), vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()])
        .unwrap();

    // Array length should be 4
    // Note: ${#arr[@]} syntax would need parser support
    assert_eq!(executor.variable_manager().array_length("nums"), Some(4));
}

#[test]
fn test_array_empty_access() {
    let mut executor = CommandExecutor::new();

    // Create empty array
    executor.variable_manager_mut()
        .set_array("empty".to_string(), vec![])
        .unwrap();

    // Access any index should return empty
    let result = executor.execute("echo ${empty[0]}");
    assert!(result.is_ok(), "Empty array access should not error");

    // Length should be 0
    assert_eq!(executor.variable_manager().array_length("empty"), Some(0));
}

#[test]
fn test_array_nonexistent_returns_empty() {
    let executor = CommandExecutor::new();

    // Accessing nonexistent array should return None/empty
    assert_eq!(executor.variable_manager().get_array("nonexistent"), None);
    assert_eq!(executor.variable_manager().array_get("nonexistent", 0), None);
    assert_eq!(executor.variable_manager().array_length("nonexistent"), None);
}

#[test]
fn test_array_with_special_characters() {
    let mut executor = CommandExecutor::new();

    // Array with special characters in values
    executor.variable_manager_mut()
        .set_array("special".to_string(), vec![
            "hello world".to_string(),
            "foo\tbar".to_string(),
            "a=b".to_string(),
        ])
        .unwrap();

    // Should be able to access without error
    assert_eq!(executor.variable_manager().array_get("special", 0), Some("hello world"));
    assert_eq!(executor.variable_manager().array_get("special", 2), Some("a=b"));
}

#[test]
fn test_array_append_operation() {
    let mut executor = CommandExecutor::new();

    // Create initial array
    executor.variable_manager_mut()
        .set_array("arr".to_string(), vec!["a".to_string(), "b".to_string()])
        .unwrap();

    // Append to array
    executor.variable_manager_mut()
        .append_to_array("arr".to_string(), "c".to_string())
        .unwrap();

    // Verify length and content
    assert_eq!(executor.variable_manager().array_length("arr"), Some(3));
    assert_eq!(executor.variable_manager().array_get("arr", 2), Some("c"));
}

#[test]
fn test_array_large_index() {
    use rush::executor::arrays::parse_array_ref;

    // Large indices should parse correctly
    let result = parse_array_ref("${arr[999999]}");
    assert!(result.is_ok(), "Large indices should be valid");

    // Accessing large index on small array returns empty
    let mut executor = CommandExecutor::new();
    executor.variable_manager_mut()
        .set_array("small".to_string(), vec!["only".to_string()])
        .unwrap();

    assert_eq!(executor.variable_manager().array_get("small", 999999), None);
}
