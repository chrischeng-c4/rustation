//! Case/esac statement parsing and execution
//!
//! Implements parsing and execution of case/esac pattern matching statements.

use crate::error::{Result, RushError};
use super::{CaseStatement, CasePattern, CompoundList, Command};
use crate::executor::execute::CommandExecutor;

/// Check if a statement appears to be syntactically complete for case statements
pub fn is_case_complete(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Check for unmatched case/esac pairs
    let mut case_count = 0;
    let mut i = 0;
    let bytes = trimmed.as_bytes();

    while i < bytes.len() {
        let remaining = &trimmed[i..];
        let c = bytes[i] as char;

        if c.is_alphabetic() {
            // Check for "case" keyword
            if remaining.to_lowercase().starts_with("case") {
                let after = remaining.get(4..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    case_count += 1;
                    i += 4;
                    continue;
                }
            }

            // Check for "esac" keyword
            if remaining.to_lowercase().starts_with("esac") {
                let after = remaining.get(4..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    case_count -= 1;
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

    case_count == 0
}

/// Parse a case statement from input string
/// Expects: case word in pattern) commands;; pattern2) commands2;; esac
pub fn parse_case_statement(input: &str) -> Result<CaseStatement> {
    let trimmed = input.trim();

    // Expect "case" keyword
    if !trimmed.to_lowercase().starts_with("case") {
        return Err(RushError::Syntax("Expected 'case' keyword".to_string()));
    }

    let rest = &trimmed[4..].trim_start();

    // Find the "in" keyword
    let in_pos = find_keyword_position(rest, "in")
        .ok_or_else(|| RushError::Syntax("Expected 'in' keyword in case statement".to_string()))?;

    let value_str = rest[..in_pos].trim();
    if value_str.is_empty() {
        return Err(RushError::Syntax("Empty value in case statement".to_string()));
    }

    // The value can be a simple word or variable reference
    let value = value_str.to_string();

    // Parse patterns
    let rest_after_in = &rest[in_pos + 2..].trim_start();

    // Find "esac" keyword
    let esac_pos = find_keyword_position(rest_after_in, "esac")
        .ok_or_else(|| RushError::Syntax("Expected 'esac' keyword in case statement".to_string()))?;

    let patterns_str = rest_after_in[..esac_pos].trim();

    // Parse all patterns
    let patterns = parse_case_patterns(patterns_str)?;

    Ok(CaseStatement { value, patterns })
}

/// Parse case patterns from the patterns section
fn parse_case_patterns(input: &str) -> Result<Vec<CasePattern>> {
    let mut patterns = Vec::new();
    let mut current_pos = 0;

    loop {
        let remaining = &input[current_pos..].trim_start();
        if remaining.is_empty() {
            break;
        }

        // Find the closing paren for this pattern
        if let Some(paren_pos) = remaining.find(')') {
            let pattern_str = remaining[..paren_pos].trim();

            // Parse multiple patterns separated by |
            let pattern_list: Vec<String> = pattern_str
                .split('|')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();

            if pattern_list.is_empty() {
                return Err(RushError::Syntax("Empty pattern in case statement".to_string()));
            }

            // Find the commands after the closing paren
            let rest_after_paren = &remaining[paren_pos + 1..].trim_start();

            // Look for ;; or ;& or ;;& to end this pattern block
            let (commands_str, separator, commands_end) = extract_pattern_commands(rest_after_paren)?;

            let body = if commands_str.trim().is_empty() {
                CompoundList::new(Vec::new())
            } else {
                parse_command_list(commands_str)?
            };

            patterns.push(CasePattern {
                patterns: pattern_list,
                body,
                fall_through: separator == "&",  // ;& means continue to next pattern
                test_next: separator == ";&",    // ;;& means test next pattern
            });

            // Move position past this pattern block
            current_pos += paren_pos + 1 + commands_end;
        } else {
            break;
        }
    }

    if patterns.is_empty() {
        return Err(RushError::Syntax("No patterns found in case statement".to_string()));
    }

    Ok(patterns)
}

/// Extract commands from a pattern block (between ) and ;; / ;& / ;;&)
fn extract_pattern_commands(input: &str) -> Result<(&str, &str, usize)> {
    // Find ;; or ;& or ;;& separator
    let mut pos = 0;
    let bytes = input.as_bytes();

    while pos + 1 < bytes.len() {
        if bytes[pos] == b';' {
            // Check what follows the semicolon
            if bytes[pos + 1] == b';' {
                // Found ;;
                return Ok((&input[..pos], ";;", pos + 2));
            } else if bytes[pos + 1] == b'&' {
                // Found ;&
                if pos + 2 < bytes.len() && bytes[pos + 2] == b';' {
                    // Found ;;&
                    return Ok((&input[..pos], ";;", pos + 3));
                } else {
                    // Found ;&
                    return Ok((&input[..pos], "&", pos + 2));
                }
            }
        }
        pos += 1;
    }

    Err(RushError::Syntax("Expected ';;' or ';&' to terminate pattern block in case statement".to_string()))
}

/// Parse a command list from input
fn parse_command_list(input: &str) -> Result<CompoundList> {
    if input.trim().is_empty() {
        return Ok(CompoundList::new(Vec::new()));
    }

    // Split by semicolons and newlines, then parse each command
    let commands: Result<Vec<Command>> = input
        .split(|c| c == ';' || c == '\n')
        .map(|cmd| cmd.trim())
        .filter(|cmd| !cmd.is_empty())
        .map(parse_simple_command)
        .collect();

    Ok(CompoundList::new(commands?))
}

/// Parse a simple command from a string
fn parse_simple_command(input: &str) -> Result<Command> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(RushError::Syntax("Empty command".to_string()));
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return Err(RushError::Syntax("Empty command".to_string()));
    }

    let program = parts[0].to_string();
    let args = parts[1..].iter().map(|s| s.to_string()).collect();

    Ok(Command {
        raw_input: trimmed.to_string(),
        program,
        args,
        background: false,
        operators: vec![],
        redirects: vec![],
        redirections: vec![],
    })
}

/// Find the position of a keyword in the input
fn find_keyword_position(input: &str, keyword: &str) -> Option<usize> {
    let lower = input.to_lowercase();
    let keyword_lower = keyword.to_lowercase();

    let mut pos = 0;
    while let Some(found) = lower[pos..].find(&keyword_lower) {
        let actual_pos = pos + found;
        let before = if actual_pos == 0 {
            true
        } else {
            !lower.chars().nth(actual_pos - 1).unwrap().is_alphanumeric()
        };

        let after = if actual_pos + keyword_lower.len() >= lower.len() {
            true
        } else {
            !lower.chars().nth(actual_pos + keyword_lower.len()).unwrap().is_alphanumeric()
        };

        if before && after {
            return Some(actual_pos);
        }

        pos = actual_pos + 1;
    }

    None
}

/// Check if a word matches a pattern
fn pattern_matches(value: &str, pattern: &str) -> bool {
    // Simple pattern matching - for now just glob-like wildcards
    // * matches any sequence of characters
    // ? matches any single character
    // [abc] matches any of a, b, or c

    if pattern == "*" {
        return true; // Default case
    }

    // Basic glob matching
    glob_match(value, pattern)
}

/// Simple glob pattern matching
fn glob_match(value: &str, pattern: &str) -> bool {
    // Convert to byte arrays for easier manipulation
    let v_bytes = value.as_bytes();
    let p_bytes = pattern.as_bytes();
    match_recursive(v_bytes, p_bytes, 0, 0)
}

/// Recursive glob pattern matching
fn match_recursive(value: &[u8], pattern: &[u8], v_idx: usize, p_idx: usize) -> bool {
    // If both are exhausted, it's a match
    if p_idx >= pattern.len() {
        return v_idx >= value.len();
    }

    // If pattern is *, it matches everything
    if pattern[p_idx] == b'*' {
        // Skip consecutive *
        let mut next_p = p_idx + 1;
        while next_p < pattern.len() && pattern[next_p] == b'*' {
            next_p += 1;
        }

        // If * is at the end, it matches the rest
        if next_p >= pattern.len() {
            return true;
        }

        // Try matching at each position
        for v in v_idx..=value.len() {
            if match_recursive(value, pattern, v, next_p) {
                return true;
            }
        }
        return false;
    }

    // If value is exhausted but pattern isn't (and it's not just *), no match
    if v_idx >= value.len() {
        return false;
    }

    // Match single character
    if pattern[p_idx] == b'?' || pattern[p_idx] == value[v_idx] {
        return match_recursive(value, pattern, v_idx + 1, p_idx + 1);
    }

    false
}

/// Execute a case statement
pub fn execute_case_statement(
    case_stmt: &CaseStatement,
    executor: &mut CommandExecutor,
) -> Result<i32> {
    // Expand the value (variable substitution, etc.)
    let expanded_value = executor.variable_manager().get(&case_stmt.value)
        .unwrap_or(&case_stmt.value)
        .to_string();

    let mut last_exit_code = 0;
    let mut matched = false;

    // Check each pattern
    for pattern in &case_stmt.patterns {
        // Check if value matches any of the patterns
        let matches = pattern.patterns.iter().any(|p| pattern_matches(&expanded_value, p));

        if matches && !matched {
            // Execute this pattern's commands
            last_exit_code = execute_compound_list(&pattern.body, executor)?;
            matched = true;

            // If fall_through is set, continue to next pattern
            if !pattern.fall_through {
                break;
            }
        }

        if matched && pattern.fall_through {
            // Continue to next pattern
            continue;
        }
    }

    Ok(last_exit_code)
}

/// Execute a compound list (sequence of commands) and return the exit code of the last command
fn execute_compound_list(compound_list: &CompoundList, executor: &mut CommandExecutor) -> Result<i32> {
    if compound_list.commands.is_empty() {
        return Ok(0);
    }

    let mut last_exit_code = 0;

    for cmd in &compound_list.commands {
        // Build command line from the command
        let cmd_line = format!("{} {}", cmd.program, cmd.args.join(" ")).trim().to_string();

        // Execute the command through the executor
        last_exit_code = executor.execute(&cmd_line)?;
    }

    Ok(last_exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_case_basic() {
        let input = "case $fruit in apple) echo red;; banana) echo yellow;; esac";
        let result = parse_case_statement(input);
        assert!(result.is_ok(), "Should parse basic case statement");
        let case_stmt = result.unwrap();
        assert_eq!(case_stmt.value, "$fruit");
        assert_eq!(case_stmt.patterns.len(), 2);
    }

    #[test]
    fn test_parse_case_with_wildcards() {
        let input = "case $file in *.txt) echo text;; *.md) echo markdown;; *) echo unknown;; esac";
        let result = parse_case_statement(input);
        assert!(result.is_ok(), "Should parse case with wildcards");
        let case_stmt = result.unwrap();
        assert_eq!(case_stmt.patterns.len(), 3);
    }

    #[test]
    fn test_parse_case_missing_in() {
        let input = "case $var fi apple) echo;;esac";
        let result = parse_case_statement(input);
        assert!(result.is_err(), "Should fail without 'in' keyword");
    }

    #[test]
    fn test_parse_case_missing_esac() {
        let input = "case $var in apple) echo;;";
        let result = parse_case_statement(input);
        assert!(result.is_err(), "Should fail without 'esac' keyword");
    }

    #[test]
    fn test_parse_case_multiple_patterns() {
        let input = "case $x in apple|orange) echo fruit;; esac";
        let result = parse_case_statement(input);
        assert!(result.is_ok());
        let case_stmt = result.unwrap();
        assert_eq!(case_stmt.patterns[0].patterns.len(), 2);
    }

    #[test]
    fn test_is_case_complete() {
        assert!(is_case_complete(""));
        assert!(is_case_complete("case $x in a) b;; esac"));
        assert!(!is_case_complete("case $x in a) b;;"));
        assert!(!is_case_complete("case $x in"));
    }

    #[test]
    fn test_pattern_match_wildcard() {
        assert!(pattern_matches("test.txt", "*.txt"));
        assert!(pattern_matches("anything", "*"));
        assert!(!pattern_matches("test.md", "*.txt"));
    }

    #[test]
    fn test_pattern_match_exact() {
        assert!(pattern_matches("apple", "apple"));
        assert!(!pattern_matches("apple", "banana"));
    }
}
