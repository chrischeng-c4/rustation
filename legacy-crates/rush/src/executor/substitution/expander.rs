//! Command substitution expander
//!
//! Expands $(...) expressions in input strings by executing the inner commands
//! and replacing the substitution with the captured output.
//!
//! Features:
//! - Recursive expansion for nested substitutions: $(echo $(date))
//! - Word splitting for command arguments (POSIX semantics)
//! - Error propagation: failed commands abort expansion
//! - Integrates lexer and executor modules

use super::executor::SubstitutionExecutor;
use super::lexer::SubstitutionLexer;
use super::SubstitutionToken;
use crate::error::{Result, RushError};

/// Expand all command substitutions in the input string
///
/// # Arguments
/// * `input` - Input string potentially containing $(...) expressions
///
/// # Returns
/// * `Ok(expanded)` - Fully expanded string with all substitutions resolved
/// * `Err(e)` - Error if any substitution fails (command not found, non-zero exit, etc.)
///
/// # Examples
/// ```ignore
/// // Basic substitution
/// let result = expand_substitutions("echo $(date)")?;
/// // result: "echo Mon Dec  3 10:30:00 PST 2025"
///
/// // Nested substitution
/// let result = expand_substitutions("echo $(echo $(pwd))")?;
/// // Expands innermost $(pwd) first, then outer $(echo ...)
///
/// // Variable assignment
/// let result = expand_substitutions("var=$(date)")?;
/// // result: "var=Mon Dec  3 10:30:00 PST 2025"
/// ```
pub fn expand_substitutions(input: &str) -> Result<String> {
    let tokens = SubstitutionLexer::tokenize(input)?;
    let mut result = String::new();

    for token in tokens {
        match token {
            SubstitutionToken::Literal(s) => {
                result.push_str(&s);
            }
            SubstitutionToken::Substitution(cmd) => {
                // Recursively expand any nested $() in the command string
                let expanded_cmd = expand_substitutions(&cmd)?;

                // Execute the expanded command and capture output
                let output = SubstitutionExecutor::execute(&expanded_cmd)?;

                // Append the captured output (already trimmed of trailing newlines)
                result.push_str(&output);
            }
        }
    }

    Ok(result)
}

/// Expand substitutions and split the result into words (for command arguments)
///
/// This performs POSIX word splitting on the expanded output:
/// - Splits on whitespace (spaces, tabs, newlines)
/// - Preserves quoted content as single words
///
/// # Arguments
/// * `input` - Input string potentially containing $(...) expressions
///
/// # Returns
/// * `Ok(words)` - Vector of words after expansion and splitting
/// * `Err(e)` - Error if any substitution fails
///
/// # Examples
/// ```ignore
/// // Multiple words from substitution
/// let words = expand_and_split("$(echo 'one two three')")?;
/// assert_eq!(words, vec!["one", "two", "three"]);
/// ```
pub fn expand_and_split(input: &str) -> Result<Vec<String>> {
    let expanded = expand_substitutions(input)?;
    Ok(split_words(&expanded))
}

/// Split a string into words using POSIX whitespace rules
///
/// Splits on spaces, tabs, and newlines. Does not handle quotes
/// (quote handling should be done before calling this).
fn split_words(input: &str) -> Vec<String> {
    input
        .split(|c: char| c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Check if an input string contains command substitutions
pub fn contains_substitution(input: &str) -> bool {
    // Quick check without full tokenization
    input.contains("$(")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Basic Expansion Tests =====

    #[test]
    fn test_no_substitution() {
        let result = expand_substitutions("hello world").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_literal_only() {
        let result = expand_substitutions("echo hello").unwrap();
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn test_basic_echo_substitution() {
        let result = expand_substitutions("$(echo hello)").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_substitution_with_prefix() {
        let result = expand_substitutions("prefix $(echo value) suffix").unwrap();
        assert_eq!(result, "prefix value suffix");
    }

    #[test]
    fn test_multiple_substitutions() {
        let result = expand_substitutions("$(echo one) $(echo two)").unwrap();
        assert_eq!(result, "one two");
    }

    #[test]
    fn test_adjacent_substitutions() {
        let result = expand_substitutions("$(echo a)$(echo b)").unwrap();
        assert_eq!(result, "ab");
    }

    // ===== Nested Substitution Tests =====

    #[test]
    fn test_nested_substitution() {
        let result = expand_substitutions("$(echo $(echo nested))").unwrap();
        assert_eq!(result, "nested");
    }

    #[test]
    fn test_deeply_nested_substitution() {
        let result = expand_substitutions("$(echo $(echo $(echo deep)))").unwrap();
        assert_eq!(result, "deep");
    }

    // ===== Variable Assignment Context Tests =====

    #[test]
    fn test_variable_assignment_substitution() {
        let result = expand_substitutions("var=$(echo value)").unwrap();
        assert_eq!(result, "var=value");
    }

    // ===== Word Splitting Tests =====

    #[test]
    fn test_expand_and_split_multiple_words() {
        let words = expand_and_split("$(echo 'one two three')").unwrap();
        assert_eq!(words, vec!["one", "two", "three"]);
    }

    #[test]
    fn test_expand_and_split_single_word() {
        let words = expand_and_split("$(echo single)").unwrap();
        assert_eq!(words, vec!["single"]);
    }

    #[test]
    fn test_expand_and_split_empty() {
        let words = expand_and_split("$(echo)").unwrap();
        assert!(words.is_empty());
    }

    #[test]
    fn test_split_words_basic() {
        let words = split_words("one two three");
        assert_eq!(words, vec!["one", "two", "three"]);
    }

    #[test]
    fn test_split_words_extra_spaces() {
        let words = split_words("  one   two   three  ");
        assert_eq!(words, vec!["one", "two", "three"]);
    }

    #[test]
    fn test_split_words_tabs_and_newlines() {
        let words = split_words("one\ttwo\nthree");
        assert_eq!(words, vec!["one", "two", "three"]);
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_nonexistent_command_error() {
        let result = expand_substitutions("$(nonexistent_command_xyz)");
        assert!(result.is_err());
    }

    #[test]
    fn test_failed_command_error() {
        let result = expand_substitutions("$(false)");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_aborts_expansion() {
        // If inner command fails, entire expansion should fail
        let result = expand_substitutions("prefix $(false) suffix");
        assert!(result.is_err());
    }

    // ===== Contains Substitution Tests =====

    #[test]
    fn test_contains_substitution_true() {
        assert!(contains_substitution("echo $(date)"));
        assert!(contains_substitution("$(pwd)"));
    }

    #[test]
    fn test_contains_substitution_false() {
        assert!(!contains_substitution("echo hello"));
        assert!(!contains_substitution("$VAR"));
        assert!(!contains_substitution("test"));
    }

    // ===== Real Command Tests =====

    #[test]
    fn test_pwd_substitution() {
        let result = expand_substitutions("$(pwd)").unwrap();
        // pwd should return an absolute path
        assert!(result.starts_with('/'));
    }

    #[test]
    fn test_true_substitution() {
        let result = expand_substitutions("$(true)").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_echo_with_args_substitution() {
        let result = expand_substitutions("$(echo arg1 arg2 arg3)").unwrap();
        assert_eq!(result, "arg1 arg2 arg3");
    }
}
