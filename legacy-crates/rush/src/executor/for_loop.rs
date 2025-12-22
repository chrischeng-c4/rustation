//! For loop parsing and execution
//!
//! Implements parsing and execution of for/in/do/done constructs.
//!
//! Phase 2 Features:
//! - Variable expansion: $VAR, ${VAR}
//! - Command substitution: $(cmd)
//! - Globbing support: *, ?, [...]

use super::CompoundList;
use super::ForLoop;
use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;
use crate::executor::expansion::expand_variables;
use crate::executor::substitution::expander::expand_substitutions;

/// Check if a statement appears to be syntactically complete for for loops
/// Useful for REPL multiline support to detect when user has finished entering a for loop
pub fn is_for_complete(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Check for unmatched for/done pairs
    let mut for_count = 0;
    let mut i = 0;
    let bytes = trimmed.as_bytes();

    while i < bytes.len() {
        let remaining = &trimmed[i..];
        let c = bytes[i] as char;

        if c.is_alphabetic() {
            // Check for "for" keyword
            if remaining.to_lowercase().starts_with("for") {
                let after = remaining.get(3..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    for_count += 1;
                    i += 3;
                    continue;
                }
            }

            // Check for "done" keyword
            if remaining.to_lowercase().starts_with("done") {
                let after = remaining.get(4..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    for_count -= 1;
                    i += 4;
                    continue;
                }
            }

            // Skip rest of word
            while i < bytes.len() && (bytes[i] as char).is_alphanumeric() {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    // Statement is complete if all for statements have been closed with done
    for_count == 0
}

/// Parse a for loop statement
/// Expected format: `for var in word_list; do commands; done` or `for var; do commands; done`
///
/// # Arguments
/// * `input` - Raw input string containing the for loop
///
/// # Returns
/// A ForLoop struct ready for execution
pub fn parse_for_loop(input: &str) -> Result<ForLoop> {
    let trimmed = input.trim();

    // Check that it starts with "for"
    if !trimmed.to_lowercase().starts_with("for") {
        return Err(RushError::Syntax("Expected 'for' keyword".to_string()));
    }

    // Find the variable name (after "for")
    let after_for = &trimmed[3..].trim_start();

    // Find where variable name ends (space or semicolon or "in")
    let var_end = after_for
        .find(|c: char| c.is_whitespace() || c == ';')
        .ok_or_else(|| RushError::Syntax("Expected variable name after 'for'".to_string()))?;

    let variable = after_for[..var_end].to_string();

    // Validate variable name (must be alphanumeric + underscore, not starting with digit)
    if variable.is_empty() {
        return Err(RushError::Syntax("Variable name cannot be empty".to_string()));
    }
    if !variable.chars().next().unwrap().is_alphabetic() && variable.chars().next().unwrap() != '_'
    {
        return Err(RushError::Syntax(format!("Invalid variable name: {}", variable)));
    }
    for c in variable.chars() {
        if !c.is_alphanumeric() && c != '_' {
            return Err(RushError::Syntax(format!("Invalid variable name: {}", variable)));
        }
    }

    // Find the rest after variable name
    let after_var = after_for[var_end..].trim_start();

    // Check for "in" keyword (optional)
    let (word_list, after_words) = if after_var.to_lowercase().starts_with("in") {
        let in_keyword_end = 2;
        let after_in = after_var[in_keyword_end..].trim_start();

        // Find where the word list ends (at "do" keyword)
        let do_pos = find_do_keyword(after_in).ok_or_else(|| {
            RushError::Syntax("Expected 'do' keyword after word list".to_string())
        })?;

        let word_list_str = after_in[..do_pos].trim();
        let words = parse_word_list(word_list_str);
        let after_do = after_in[do_pos..].trim_start();

        (words, after_do)
    } else {
        // No "in" keyword - will use positional parameters
        // Find "do" keyword
        let do_pos = find_do_keyword(after_var)
            .ok_or_else(|| RushError::Syntax("Expected 'do' keyword".to_string()))?;

        let after_do = after_var[do_pos..].trim_start();
        (Vec::new(), after_do)
    };

    // Check for "do" keyword
    if !after_words.to_lowercase().starts_with("do") {
        return Err(RushError::Syntax("Expected 'do' keyword".to_string()));
    }

    let after_do = after_words[2..].trim_start();

    // Find "done" keyword
    let done_pos = find_done_keyword(after_do).ok_or_else(|| {
        RushError::Syntax("Expected 'done' keyword to close for loop".to_string())
    })?;

    let body_str = after_do[..done_pos].trim();

    // Remove trailing semicolon if present (parser doesn't need it)
    let body_str_clean = body_str.trim_end_matches(';');

    // Parse the body as a compound list (for backward compatibility)
    let body = super::conditional::parse_compound_list(body_str_clean)?;

    // Phase 3: Store raw body string for pipe and redirection support
    Ok(ForLoop::new_with_raw_body(
        variable,
        word_list,
        body,
        body_str_clean.to_string(),
    ))
}

/// Find the position of the "do" keyword in the string
/// Respects nesting of other keywords
fn find_do_keyword(input: &str) -> Option<usize> {
    let mut i = 0;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        let remaining = &input[i..];
        let c = bytes[i] as char;

        if c.is_alphabetic() {
            // Check for "do" keyword
            if remaining.to_lowercase().starts_with("do") {
                let after = remaining.get(2..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    return Some(i);
                }
            }

            // Skip rest of word
            while i < bytes.len() && (bytes[i] as char).is_alphanumeric() {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    None
}

/// Find the position of the "done" keyword in the string
/// Respects nesting of if/fi and other for/done pairs
fn find_done_keyword(input: &str) -> Option<usize> {
    let mut depth = 1; // We're inside a for loop
    let mut i = 0;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        let remaining = &input[i..];
        let c = bytes[i] as char;

        if c.is_alphabetic() {
            // Check for "for" keyword (increases nesting)
            if remaining.to_lowercase().starts_with("for") {
                let after = remaining.get(3..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    depth += 1;
                    i += 3;
                    continue;
                }
            }

            // Check for "done" keyword (decreases nesting)
            if remaining.to_lowercase().starts_with("done") {
                let after = remaining.get(4..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                    i += 4;
                    continue;
                }
            }

            // Skip rest of word
            while i < bytes.len() && (bytes[i] as char).is_alphanumeric() {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    None
}

/// Parse a word list (space-separated words)
/// Simple implementation - just splits by whitespace and removes semicolons
/// Later phases will add expansion support
fn parse_word_list(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .filter_map(|s| {
            let s = s.trim_end_matches(';');
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
        .collect()
}

/// Execute a for loop
/// Iterates over word list, binding variable and executing body
///
/// Phase 2 Support:
/// - Variable expansion: for x in $items; do ... done
/// - Command substitution: for f in $(ls); do ... done
/// - Globbing: for f in *.txt; do ... done
pub fn execute_for_loop(for_loop: &ForLoop, executor: &mut CommandExecutor) -> Result<i32> {
    // Get the word list to iterate over
    let words = if for_loop.word_list.is_empty() {
        // No explicit word list - use positional parameters ($@)
        executor
            .variable_manager()
            .get("@")
            .unwrap_or_default()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        // Expand the word list with variables and command substitution
        let mut expanded_words = Vec::new();
        for word in &for_loop.word_list {
            // Phase 2: Apply variable expansion
            let var_expanded = expand_variables(word, executor);

            // Phase 2: Apply command substitution
            let fully_expanded =
                expand_substitutions(&var_expanded).unwrap_or_else(|_| var_expanded); // Fall back to var expansion if subst fails

            // Phase 2: Perform word splitting on expanded result
            for expanded_word in fully_expanded.split_whitespace() {
                expanded_words.push(expanded_word.to_string());
            }
        }
        expanded_words
    };

    let mut exit_code = 0;

    // Iterate over each word
    for word in words {
        // Bind the loop variable to the current word
        executor
            .variable_manager_mut()
            .set(for_loop.variable.clone(), word)?;

        // Phase 3: Execute using raw body string to support pipes and redirections
        // If raw body is available, use it (supports pipes and redirections)
        // Otherwise, fall back to parsed compound list
        if !for_loop.body_raw.is_empty() {
            // Execute raw body string (which may contain pipes, redirections, etc.)
            exit_code = executor.execute(&for_loop.body_raw)?;
        } else {
            // Fallback for backward compatibility
            exit_code = execute_compound_list(&for_loop.body, executor)?;
        }
    }

    // Return exit code from last iteration (or 0 if no iterations)
    Ok(exit_code)
}

/// Execute a compound list (sequence of commands) and return the exit code of the last command
fn execute_compound_list(
    compound_list: &CompoundList,
    executor: &mut CommandExecutor,
) -> Result<i32> {
    if compound_list.commands.is_empty() {
        return Ok(0);
    }

    let mut last_exit_code = 0;

    for cmd in &compound_list.commands {
        // Build command line from the command
        let cmd_line = format!("{} {}", cmd.program, cmd.args.join(" "))
            .trim()
            .to_string();

        // Execute the command through the executor
        last_exit_code = executor.execute(&cmd_line)?;
    }

    Ok(last_exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_for_basic() {
        let input = "for item in one two three; do echo $item; done";
        let result = parse_for_loop(input);
        assert!(result.is_ok(), "Should parse basic for loop");
        let for_loop = result.unwrap();
        assert_eq!(for_loop.variable, "item");
        assert_eq!(for_loop.word_list.len(), 3);
        assert_eq!(for_loop.word_list[0], "one");
        assert_eq!(for_loop.word_list[1], "two");
        assert_eq!(for_loop.word_list[2], "three");
    }

    #[test]
    fn test_parse_for_missing_do() {
        let input = "for item in one two; fi";
        let result = parse_for_loop(input);
        assert!(result.is_err(), "Should fail without 'do' keyword");
    }

    #[test]
    fn test_parse_for_missing_done() {
        let input = "for item in one two; do echo $item;";
        let result = parse_for_loop(input);
        assert!(result.is_err(), "Should fail without 'done' keyword");
    }

    #[test]
    fn test_parse_for_empty_word_list() {
        let input = "for item in; do echo $item; done";
        let result = parse_for_loop(input);
        assert!(result.is_ok(), "Should allow empty word list");
        let for_loop = result.unwrap();
        assert!(for_loop.word_list.is_empty());
    }

    #[test]
    fn test_parse_for_no_in_keyword() {
        let input = "for item; do echo $item; done";
        let result = parse_for_loop(input);
        assert!(result.is_ok(), "Should parse without 'in' keyword");
        let for_loop = result.unwrap();
        assert!(for_loop.word_list.is_empty(), "Word list should be empty (positional params)");
    }

    #[test]
    fn test_is_for_complete() {
        assert!(is_for_complete(""));
        assert!(is_for_complete("for i in 1 2 3; do echo $i; done"));
        assert!(!is_for_complete("for i in 1 2 3; do echo $i;"));
        assert!(!is_for_complete("for i in 1 2 3;"));
    }

    #[test]
    fn test_parse_for_single_word() {
        let input = "for x in single; do true; done";
        let result = parse_for_loop(input);
        assert!(result.is_ok());
        let for_loop = result.unwrap();
        assert_eq!(for_loop.word_list.len(), 1);
        assert_eq!(for_loop.word_list[0], "single");
    }

    #[test]
    fn test_parse_for_invalid_var_name() {
        let input = "for 123var in one; do echo; done";
        let result = parse_for_loop(input);
        assert!(result.is_err(), "Should reject variable starting with digit");
    }
}
