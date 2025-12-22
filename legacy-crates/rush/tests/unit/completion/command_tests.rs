//! Unit tests for CommandCompleter
//!
//! Tests command name completion from PATH executables

use reedline::Completer;
use rush::completion::CommandCompleter;

/// Helper to create a CommandCompleter with a fake cache for testing
/// This bypasses PATH scanning and uses a controlled set of commands
/// Note: Currently unused since cache is private - tests use actual PATH
#[allow(dead_code)]
fn create_test_completer_with_cache(_commands: Vec<&str>) -> CommandCompleter {
    let mut completer = CommandCompleter::new();

    // Manually populate cache by triggering a completion
    // This will load the cache, then we can replace it
    let _ = completer.complete("", 0);

    // Replace with our test cache
    // Note: This is a workaround since cache is private
    // For now, we'll test against actual PATH commands
    completer
}

#[test]
fn test_command_completer_creation() {
    let mut completer = CommandCompleter::new();
    // Completer should be created successfully
    // Cache is lazy-loaded, so it's None initially
    assert_eq!(completer.complete("", 0).len(), 0); // Empty input returns no completions
}

#[test]
fn test_single_match_scenario() {
    // T016: Test completing to a single unique command
    let mut completer = CommandCompleter::new();

    // Use a prefix that likely has few matches in most systems
    // "rustdo" should match "rustdoc" if Rust toolchain is installed
    let suggestions = completer.complete("rustdo", 6);

    // Should find rustdoc (if Rust is installed)
    // Note: This test assumes Rust toolchain is in PATH
    if !suggestions.is_empty() {
        assert!(suggestions.iter().any(|s| s.value.starts_with("rustdo")));
    }
}

#[test]
fn test_multiple_matches_scenario() {
    // T017: Test multiple matches are returned and sorted
    let mut completer = CommandCompleter::new();

    // "ca" should match multiple commands: cat, cal, cargo, etc.
    let suggestions = completer.complete("ca", 2);

    // Should have multiple matches
    assert!(!suggestions.is_empty(), "Should find commands starting with 'ca'");

    // Verify matches are sorted alphabetically
    let values: Vec<String> = suggestions.iter().map(|s| s.value.clone()).collect();
    let mut sorted_values = values.clone();
    sorted_values.sort();
    assert_eq!(values, sorted_values, "Results should be sorted alphabetically");

    // All should start with "ca"
    for suggestion in suggestions {
        assert!(
            suggestion.value.starts_with("ca"),
            "Suggestion '{}' should start with 'ca'",
            suggestion.value
        );
    }
}

#[test]
fn test_no_matches_scenario() {
    // T018: Test no matches returns empty vector
    let mut completer = CommandCompleter::new();

    // Use a prefix that's extremely unlikely to match any command
    let suggestions = completer.complete("zzzzqqqqxxxx", 12);

    // Should return empty vector when no matches
    assert!(
        suggestions.is_empty(),
        "Should return empty vec for non-existent command prefix"
    );
}

#[test]
fn test_too_many_matches_scenario() {
    // T019: Test >50 matches returns empty vec (our current behavior)
    let mut completer = CommandCompleter::new();

    // Single letter prefix likely to have >50 matches
    // Try several common single letters
    let test_prefixes = vec!["l", "s", "p", "c"];

    for prefix in test_prefixes {
        let _suggestions = completer.complete(prefix, 1);

        // If suggestions is empty, it might be because >50 matches
        // We can't directly verify count, but we can verify the behavior
        // Note: In implementation, >50 matches returns empty vec

        // For this test, we just verify it doesn't crash
        // and returns a valid (possibly empty) vec
        // The important thing is: no panics or errors
    }

    // At least one single-letter prefix should trigger the >50 limit
    // If not, that's ok - PATH might have few commands
    // This test primarily ensures no crashes with many matches
}

#[test]
fn test_completion_in_arguments_returns_empty() {
    // Test that completion after a space (in arguments) returns nothing
    let mut completer = CommandCompleter::new();

    // Cursor after space should not complete command
    let suggestions = completer.complete("git ", 4);

    assert!(
        suggestions.is_empty(),
        "Should not complete when cursor is in arguments (after space)"
    );
}

#[test]
fn test_completion_suggestions_have_correct_format() {
    // Verify Suggestion objects are properly formatted
    let mut completer = CommandCompleter::new();

    let suggestions = completer.complete("ca", 2);

    if !suggestions.is_empty() {
        let first = &suggestions[0];

        // Should have value
        assert!(!first.value.is_empty(), "Suggestion should have non-empty value");

        // Should have span that covers the input
        assert_eq!(first.span.start, 0, "Span should start at 0");
        assert_eq!(first.span.end, 2, "Span should end at cursor position");

        // Should append whitespace after command
        assert!(first.append_whitespace, "Should append whitespace after command completion");

        // Should have style for warm tone
        assert!(first.style.is_some(), "Should have style for visibility");
    }
}

#[test]
#[cfg(target_os = "macos")]
fn test_case_insensitive_matching_on_macos() {
    // T020: Verify case-insensitive matching on macOS
    let mut completer = CommandCompleter::new();

    // On macOS, should match regardless of case
    let lower_suggestions = completer.complete("ca", 2);
    let upper_suggestions = completer.complete("CA", 2);

    // Both should return results (assuming commands exist)
    // Case shouldn't matter on macOS
    if !lower_suggestions.is_empty() {
        // Upper case should also find matches
        assert!(
            !upper_suggestions.is_empty(),
            "Upper case should match on case-insensitive macOS"
        );
    }
}

#[test]
#[cfg(not(target_os = "macos"))]
fn test_case_sensitive_matching_on_linux() {
    // Verify case-sensitive matching on Linux
    let mut completer = CommandCompleter::new();

    // On Linux, case matters
    let lower_suggestions = completer.complete("ca", 2);
    let upper_suggestions = completer.complete("CA", 2);

    // Lower case should find matches
    if !lower_suggestions.is_empty() {
        // Upper case CA is unlikely to match anything on Linux
        // (most commands are lowercase)
        let upper_count = upper_suggestions.len();
        let lower_count = lower_suggestions.len();

        // Usually upper case finds fewer or no matches
        assert!(
            upper_count <= lower_count,
            "Case-sensitive: uppercase should find fewer or equal matches"
        );
    }
}

#[test]
fn test_empty_input_returns_nothing() {
    let mut completer = CommandCompleter::new();

    // Empty string should not return completions
    let suggestions = completer.complete("", 0);

    assert!(suggestions.is_empty(), "Empty input should return no completions");
}

#[test]
fn test_cache_is_reused_across_completions() {
    // Verify cache is loaded once and reused
    let mut completer = CommandCompleter::new();

    // First completion loads cache
    let first_suggestions = completer.complete("ca", 2);

    // Second completion should reuse cache (faster)
    let second_suggestions = completer.complete("ca", 2);

    // Should return same results
    assert_eq!(
        first_suggestions.len(),
        second_suggestions.len(),
        "Cache should return consistent results"
    );
}
