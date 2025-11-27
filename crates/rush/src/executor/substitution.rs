// Command substitution implementation for $(command) syntax
//
// This module handles parsing and executing command substitutions in the shell.
// Command substitution allows the output of a command to replace the command itself.
//
// Examples:
// - echo $(date) - replaces $(date) with the output of the date command
// - echo "User: $(whoami)" - works inside double quotes
// - echo $(echo $(whoami)) - supports nesting

use crate::error::{Result, RushError};
use std::collections::HashMap;
use std::io::Read;
use std::process::{Command, Stdio};

/// Check if a string contains command substitution patterns
pub fn contains_command_substitution(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    let mut in_single_quote = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '\'' => in_single_quote = !in_single_quote,
            '$' if !in_single_quote => {
                if chars.peek() == Some(&'(') {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

/// Extract command substitutions from a string
/// Returns Vec of (start_pos, end_pos, command_string)
/// Handles nested substitutions by tracking parenthesis depth
pub fn extract_command_substitutions(s: &str) -> Result<Vec<(usize, usize, String)>> {
    let mut substitutions = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while i < chars.len() {
        // Handle quotes
        if chars[i] == '\'' && (i == 0 || chars[i - 1] != '\\') {
            in_single_quote = !in_single_quote;
            i += 1;
            continue;
        }

        if chars[i] == '"' && (i == 0 || chars[i - 1] != '\\') {
            in_double_quote = !in_double_quote;
            i += 1;
            continue;
        }

        // Skip escaped characters
        if i > 0 && chars[i - 1] == '\\' {
            i += 1;
            continue;
        }

        // Look for $( pattern (not in single quotes)
        if !in_single_quote && chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '(' {
            let start = i;
            i += 2; // Skip $(

            // Find matching )
            let mut depth = 1;
            let command_start = i;
            let mut sub_in_single = false;
            let mut sub_in_double = false;

            while i < chars.len() && depth > 0 {
                // Track quotes inside substitution
                if chars[i] == '\'' && (i == 0 || chars[i - 1] != '\\') {
                    sub_in_single = !sub_in_single;
                } else if chars[i] == '"' && (i == 0 || chars[i - 1] != '\\') {
                    sub_in_double = !sub_in_double;
                } else if !sub_in_single {
                    // Only count parens outside single quotes
                    if chars[i] == '(' && (i == 0 || chars[i - 1] != '\\') {
                        depth += 1;
                    } else if chars[i] == ')' && (i == 0 || chars[i - 1] != '\\') {
                        depth -= 1;
                    }
                }
                i += 1;
            }

            if depth != 0 {
                return Err(RushError::Execution(
                    "Unclosed command substitution".to_string(),
                ));
            }

            // Extract command (everything between $( and ))
            let command: String = chars[command_start..i - 1].iter().collect();
            substitutions.push((start, i - 1, command));
        } else {
            i += 1;
        }
    }

    Ok(substitutions)
}

/// Execute a command and capture its stdout
/// Trims trailing newlines (POSIX behavior)
pub fn execute_substitution(
    command: &str,
    env_map: &HashMap<String, String>,
) -> Result<String> {
    // Parse the command
    use super::parser::parse_pipeline;
    let _pipeline = parse_pipeline(command)?;

    // For simplicity, we'll execute using sh -c for now
    // This ensures proper command parsing and execution
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);

    // Set environment
    cmd.env_clear().envs(env_map);

    // Capture stdout
    cmd.stdout(Stdio::piped()).stderr(Stdio::inherit());

    // Execute
    let mut child = cmd.spawn().map_err(|e| {
        RushError::Execution(format!("Failed to execute command substitution: {}", e))
    })?;

    // Capture output
    let mut output = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_string(&mut output).map_err(|e| {
            RushError::Execution(format!("Failed to read command output: {}", e))
        })?;
    }

    // Wait for completion
    let _status = child.wait().map_err(|e| {
        RushError::Execution(format!("Failed to wait for command: {}", e))
    })?;

    // Check exit status (non-zero is not an error for substitution)
    // The command output is used even if it failed

    // Trim trailing newlines (POSIX behavior)
    let trimmed = output.trim_end_matches('\n').to_string();

    Ok(trimmed)
}

/// Expand all command substitutions in a string
/// Handles nested substitutions recursively (innermost first)
pub fn expand_substitutions(
    input: &str,
    env_map: &HashMap<String, String>,
) -> Result<String> {
    if !contains_command_substitution(input) {
        return Ok(input.to_string());
    }

    let substitutions = extract_command_substitutions(input)?;

    if substitutions.is_empty() {
        return Ok(input.to_string());
    }

    let mut result = input.to_string();
    let chars: Vec<char> = input.chars().collect();

    // Process substitutions from last to first (to maintain correct indices)
    for (start, end, command) in substitutions.into_iter().rev() {
        // Recursively expand nested substitutions in the command
        let expanded_command = expand_substitutions(&command, env_map)?;

        // Execute the command
        let output = execute_substitution(&expanded_command, env_map)?;

        // Replace $(command) with output
        // Calculate byte positions from char positions
        let start_byte: usize = chars.iter().take(start).map(|c| c.len_utf8()).sum();
        let end_byte: usize = chars.iter().take(end + 1).map(|c| c.len_utf8()).sum();

        result.replace_range(start_byte..end_byte, &output);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_command_substitution_true() {
        assert!(contains_command_substitution("echo $(date)"));
        assert!(contains_command_substitution("$(whoami)"));
        assert!(contains_command_substitution("prefix $(cmd) suffix"));
    }

    #[test]
    fn test_contains_command_substitution_false() {
        assert!(!contains_command_substitution("echo hello"));
        assert!(!contains_command_substitution("$PATH"));
        assert!(!contains_command_substitution("echo '$(date)'"));
        assert!(!contains_command_substitution("echo \\$(date)"));
    }

    #[test]
    fn test_extract_simple_substitution() {
        let result = extract_command_substitutions("echo $(date)").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].2, "date");
    }

    #[test]
    fn test_extract_multiple_substitutions() {
        let result = extract_command_substitutions("echo $(whoami) $(date)").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].2, "whoami");
        assert_eq!(result[1].2, "date");
    }

    #[test]
    fn test_extract_nested_substitution() {
        let result = extract_command_substitutions("echo $(echo $(whoami))").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].2, "echo $(whoami)");
    }

    #[test]
    fn test_extract_unclosed_substitution() {
        let result = extract_command_substitutions("echo $(date");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unclosed command substitution"));
    }

    #[test]
    fn test_extract_unclosed_nested() {
        let result = extract_command_substitutions("echo $(echo $(date)");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_substitution_simple() {
        let env = HashMap::new();
        let result = execute_substitution("echo hello", &env).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_execute_substitution_trim_newlines() {
        let env = HashMap::new();
        let result = execute_substitution("printf 'hello\\n\\n'", &env).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_expand_substitutions_simple() {
        let env = HashMap::new();
        let result = expand_substitutions("echo $(echo hello)", &env).unwrap();
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn test_expand_substitutions_multiple() {
        let env = HashMap::new();
        let result = expand_substitutions("$(echo a) $(echo b)", &env).unwrap();
        assert_eq!(result, "a b");
    }

    #[test]
    fn test_expand_substitutions_nested() {
        let env = HashMap::new();
        let result = expand_substitutions("echo $(echo $(echo hello))", &env).unwrap();
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn test_expand_no_substitution() {
        let env = HashMap::new();
        let result = expand_substitutions("echo hello", &env).unwrap();
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn test_expand_substitution_in_quotes() {
        let env = HashMap::new();
        let result = expand_substitutions("echo \"$(echo hello)\"", &env).unwrap();
        assert_eq!(result, "echo \"hello\"");
    }

    #[test]
    fn test_expand_escaped_substitution() {
        let env = HashMap::new();
        // Escaped substitutions should not be expanded
        // This test documents current behavior - escaping may need special handling
        let result = expand_substitutions("echo \\$(date)", &env).unwrap();
        // For now, we don't handle escaping in expand_substitutions
        // This is a known limitation
        assert_eq!(result, "echo \\$(date)");
    }
}
