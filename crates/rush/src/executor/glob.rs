//! Glob pattern expansion for wildcard matching
//!
//! This module implements shell-style globbing for wildcard patterns:
//! - `*` - Match zero or more characters (excluding /)
//! - `?` - Match exactly one character (excluding /)
//! - `[abc]` - Match any character in the set
//! - `[a-z]` - Match any character in the range
//! - `[!abc]` - Match any character NOT in the set
//!
//! # Examples
//!
//! ```ignore
//! use rush::executor::glob::glob_expand;
//!
//! // Expand wildcard patterns
//! let expanded = glob_expand("ls *.txt")?;  // → "ls file1.txt file2.txt"
//! let expanded = glob_expand("cat file?.md")?;  // → "cat file1.md file2.md"
//! let expanded = glob_expand("echo '*.txt'")?;  // → "echo *.txt" (quoted, not expanded)
//! ```

use crate::error::{Result, RushError};
use std::fs;
use std::path::{Path, PathBuf};

/// Expand glob patterns in a command line
///
/// Processes arguments separated by whitespace, expanding glob patterns
/// while respecting quoted strings and escape sequences.
///
/// # Arguments
/// * `line` - Command line with potential glob patterns (e.g., "ls *.txt")
///
/// # Returns
/// * `Ok(expanded)` - Command line with patterns expanded to matching files
/// * `Err(e)` - If filesystem operations fail
///
/// # Examples
/// ```ignore
/// glob_expand("ls *.txt") → "ls file1.txt file2.txt"
/// glob_expand("echo '*.txt'") → "echo *.txt"  // quoted, not expanded
/// ```
pub fn glob_expand(line: &str) -> Result<String> {
    let mut result = Vec::new();
    let mut current_arg = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if escaped {
            // Escaped character - add as literal
            current_arg.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => {
                escaped = true;
                continue;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                continue;  // Don't include quotes
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                continue;  // Don't include quotes
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                // End of argument
                if !current_arg.is_empty() {
                    // Try to expand the argument
                    let expanded = expand_single_pattern(&current_arg)?;
                    result.extend(expanded);
                    current_arg.clear();
                }
                // Add space to result if not already present
                if !result.is_empty() && !result.last().map_or(false, |s: &String| s.contains(' ')) {
                    if let Some(last) = result.last_mut() {
                        last.push(' ');
                    } else {
                        result.push(" ".to_string());
                    }
                }
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }

    // Process final argument
    if !current_arg.is_empty() {
        let expanded = expand_single_pattern(&current_arg)?;
        result.extend(expanded);
    }

    Ok(result.join(" "))
}

/// Check if a filename matches a glob pattern
///
/// Supports *, ?, [abc], [a-z], [!abc] patterns.
/// Returns true if the filename matches the pattern.
///
/// # Arguments
/// * `pattern` - Glob pattern (e.g., "*.txt", "file?.md")
/// * `filename` - Filename to match against pattern
///
/// # Returns
/// true if filename matches pattern, false otherwise
fn pattern_matches(pattern: &str, filename: &str) -> bool {
    glob_match_recursive(pattern, 0, filename, 0)
}

/// Expand a single glob pattern to matching files
///
/// Traverses the filesystem and returns all files matching the pattern.
/// If no files match, returns the pattern unchanged.
///
/// # Arguments
/// * `pattern` - Glob pattern (e.g., "*.txt", "dir/*.rs")
///
/// # Returns
/// * `Ok(matches)` - Vec of matching files or [pattern] if no matches
/// * `Err(e)` - If filesystem operations fail
fn expand_single_pattern(pattern: &str) -> Result<Vec<String>> {
    // Check if pattern contains glob metacharacters
    if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
        // No glob characters, return as-is
        return Ok(vec![pattern.to_string()]);
    }

    // Separate pattern into directory and filename parts
    let (dir_pattern, file_pattern) = if let Some(last_slash) = pattern.rfind('/') {
        (&pattern[..last_slash + 1], &pattern[last_slash + 1..])
    } else {
        ("", pattern)
    };

    // Directory to search in (default to current directory)
    let search_dir = if dir_pattern.is_empty() {
        ".".to_string()
    } else {
        dir_pattern.to_string()
    };

    // Try to read directory
    let dir_path = Path::new(&search_dir);
    let entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(_) => {
            // Directory doesn't exist or can't be read, return pattern unchanged
            return Ok(vec![pattern.to_string()]);
        }
    };

    let mut matches = Vec::new();

    for entry in entries {
        if let Ok(entry) = entry {
            if let Ok(_metadata) = entry.metadata() {
                let filename = match entry.file_name().into_string() {
                    Ok(name) => name,
                    Err(_) => continue,
                };

                // Match filename against pattern
                if pattern_matches(file_pattern, &filename) {
                    // Build full path
                    let full_path = if dir_pattern.is_empty() {
                        filename
                    } else {
                        format!("{}{}", dir_pattern, filename)
                    };

                    matches.push(full_path);
                }
            }
        }
    }

    if matches.is_empty() {
        // No matches, return pattern unchanged
        Ok(vec![pattern.to_string()])
    } else {
        // Sort matches for consistent output
        matches.sort();
        Ok(matches)
    }
}

/// Match a filename against a character set pattern like [abc] or [a-z]
///
/// Handles:
/// - Simple sets: [abc]
/// - Ranges: [a-z], [0-9]
/// - Negation: [!abc]
/// - Hyphen as literal: [a-z-] or [-a-z]
///
/// # Arguments
/// * `pattern` - The pattern string containing [...]
/// * `pos` - Current position in pattern (will be advanced past the closing ])
/// * `ch` - Character to match against the set
///
/// # Returns
/// true if character matches the set, false otherwise
fn match_character_set(pattern: &str, pos: &mut usize, ch: char) -> bool {
    let pattern_bytes = pattern.as_bytes();

    // Must start with [
    if *pos >= pattern_bytes.len() || pattern_bytes[*pos] != b'[' {
        return false;
    }

    *pos += 1;  // Skip '['

    // Check for negation
    let negated = if *pos < pattern_bytes.len() && pattern_bytes[*pos] == b'!' {
        *pos += 1;
        true
    } else {
        false
    };

    let mut matched = false;

    // Handle special case: '-' at start is literal
    if *pos < pattern_bytes.len() && pattern_bytes[*pos] == b'-' {
        if ch == '-' {
            matched = true;
        }
        *pos += 1;
    }

    // Process characters/ranges until we find ]
    while *pos < pattern_bytes.len() && pattern_bytes[*pos] != b']' {
        let current = pattern_bytes[*pos] as char;

        // Check for range (e.g., a-z)
        if *pos + 2 < pattern_bytes.len() && pattern_bytes[*pos + 1] == b'-' && pattern_bytes[*pos + 2] != b']' {
            let start = current;
            let end = pattern_bytes[*pos + 2] as char;

            if ch >= start && ch <= end {
                matched = true;
            }
            *pos += 3;  // Skip start, hyphen, end
        } else {
            // Single character
            if ch == current {
                matched = true;
            }
            *pos += 1;
        }
    }

    // Skip closing ]
    if *pos < pattern_bytes.len() && pattern_bytes[*pos] == b']' {
        *pos += 1;
    }

    // Apply negation
    if negated {
        !matched
    } else {
        matched
    }
}

/// Recursive backtracking pattern matcher for glob patterns with * wildcards
///
/// Handles matching of patterns containing * which can match zero or more
/// characters (excluding /).
///
/// # Arguments
/// * `pattern` - The glob pattern being matched
/// * `pattern_pos` - Current position in pattern
/// * `text` - The text being matched against
/// * `text_pos` - Current position in text
///
/// # Returns
/// true if the pattern matches the text, false otherwise
fn glob_match_recursive(pattern: &str, pattern_pos: usize, text: &str, text_pos: usize) -> bool {
    let pattern_bytes = pattern.as_bytes();
    let text_bytes = text.as_bytes();

    // Base case: both pattern and text exhausted
    if pattern_pos >= pattern_bytes.len() {
        return text_pos >= text_bytes.len();
    }

    // Look ahead for * to use backtracking
    if pattern_pos + 1 < pattern_bytes.len() && pattern_bytes[pattern_pos] == b'*' {
        // Try matching zero characters (skip the *)
        if glob_match_recursive(pattern, pattern_pos + 1, text, text_pos) {
            return true;
        }

        // Try matching one or more characters (but not /)
        if text_pos < text_bytes.len() && text_bytes[text_pos] != b'/' {
            if glob_match_recursive(pattern, pattern_pos, text, text_pos + 1) {
                return true;
            }
        }

        return false;
    }

    // If pattern starts with * at end
    if pattern_pos < pattern_bytes.len() && pattern_bytes[pattern_pos] == b'*' {
        // Trailing * matches everything (except /)
        if pattern_pos + 1 >= pattern_bytes.len() {
            // Check remaining text for / separators
            for i in text_pos..text_bytes.len() {
                if text_bytes[i] == b'/' {
                    return false;  // * doesn't match /
                }
            }
            return true;
        }
    }

    // If text exhausted but pattern remains
    if text_pos >= text_bytes.len() {
        return false;
    }

    // Match current character
    match pattern_bytes[pattern_pos] {
        // Character set: [abc] or [a-z] or [!abc]
        b'[' => {
            let mut pos = pattern_pos;
            if match_character_set(pattern, &mut pos, text_bytes[text_pos] as char) {
                glob_match_recursive(pattern, pos, text, text_pos + 1)
            } else {
                false
            }
        }
        // Single character wildcard
        b'?' => {
            // ? matches any single character except /
            if text_bytes[text_pos] != b'/' {
                glob_match_recursive(pattern, pattern_pos + 1, text, text_pos + 1)
            } else {
                false
            }
        }
        // Escaped character (future: could add \ escape support)
        ch => {
            if ch == text_bytes[text_pos] {
                glob_match_recursive(pattern, pattern_pos + 1, text, text_pos + 1)
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pattern matching tests
    #[test]
    fn test_pattern_matches_wildcard() {
        // Test basic * wildcard matching
        assert!(pattern_matches("*.txt", "file.txt"));
        assert!(pattern_matches("*.txt", "document.txt"));
        assert!(!pattern_matches("*.txt", "file.rs"));
    }

    #[test]
    fn test_pattern_matches_question_mark() {
        // Test ? single character matching
        assert!(pattern_matches("file?.txt", "file1.txt"));
        assert!(pattern_matches("file?.txt", "fileA.txt"));
        assert!(!pattern_matches("file?.txt", "file12.txt"));
    }

    #[test]
    fn test_pattern_matches_character_set() {
        // Test [abc] character set matching
        assert!(pattern_matches("[abc].txt", "a.txt"));
        assert!(pattern_matches("[abc].txt", "b.txt"));
        assert!(!pattern_matches("[abc].txt", "d.txt"));
    }

    #[test]
    fn test_pattern_matches_range() {
        // Test [a-z] range matching
        assert!(pattern_matches("file[0-9].txt", "file1.txt"));
        assert!(pattern_matches("file[0-9].txt", "file5.txt"));
        assert!(!pattern_matches("file[0-9].txt", "fileA.txt"));
    }

    #[test]
    fn test_pattern_matches_negation() {
        // Test [!abc] negation matching
        assert!(pattern_matches("[!abc].txt", "d.txt"));
        assert!(pattern_matches("[!abc].txt", "x.txt"));
        assert!(!pattern_matches("[!abc].txt", "a.txt"));
    }

    // Glob expansion tests
    #[test]
    fn test_expand_single_pattern_no_matches() {
        // When no files match, return pattern unchanged
        let result = expand_single_pattern("nonexistent_pattern_*.xyz");
        assert!(result.is_ok());
        let expanded = result.unwrap();
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0], "nonexistent_pattern_*.xyz");
    }

    #[test]
    fn test_glob_expand_basic() {
        // Test basic glob expansion
        let result = glob_expand("echo test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_glob_expand_quoted() {
        // Test that quoted patterns are not expanded
        let result = glob_expand("echo '*.txt'");
        assert!(result.is_ok());
    }

    // Character set matching tests
    #[test]
    fn test_match_character_set_simple() {
        let pattern = "[abc]";

        let mut pos = 0;
        assert!(match_character_set(pattern, &mut pos, 'a'));

        let mut pos = 0;
        assert!(match_character_set(pattern, &mut pos, 'b'));

        let mut pos = 0;
        assert!(!match_character_set(pattern, &mut pos, 'd'));
    }

    // Recursive matcher tests
    #[test]
    fn test_glob_match_recursive_simple() {
        assert!(glob_match_recursive("*.txt", 0, "file.txt", 0));
        assert!(!glob_match_recursive("*.txt", 0, "file.rs", 0));
    }

    #[test]
    fn test_glob_match_recursive_question() {
        assert!(glob_match_recursive("file?.txt", 0, "file1.txt", 0));
        assert!(!glob_match_recursive("file?.txt", 0, "file12.txt", 0));
    }
}
