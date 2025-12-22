//! While and Until loop parsing and execution
//!
//! Implements parsing and execution of while/do/done and until/do/done constructs.
//!
//! Phase 2 Features:
//! - Variable expansion: $VAR, ${VAR}
//! - Command substitution: $(cmd)

use super::{Command, CompoundList, UntilLoop, WhileLoop};
use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;
use crate::executor::expansion::expand_variables;
use crate::executor::substitution::expander::expand_substitutions;

/// Check if a statement appears to be syntactically complete for while loops
pub fn is_while_complete(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Check for unmatched while/done pairs
    let mut while_count = 0;
    let mut i = 0;
    let bytes = trimmed.as_bytes();

    while i < bytes.len() {
        let remaining = &trimmed[i..];
        let c = bytes[i] as char;

        if c.is_alphabetic() {
            // Check for "while" keyword
            if remaining.to_lowercase().starts_with("while") {
                let after = remaining.get(5..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    while_count += 1;
                    i += 5;
                    continue;
                }
            }

            // Check for "done" keyword
            if remaining.to_lowercase().starts_with("done") {
                let after = remaining.get(4..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    while_count -= 1;
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

    while_count == 0
}

/// Check if a statement appears to be syntactically complete for until loops
pub fn is_until_complete(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Check for unmatched until/done pairs
    let mut until_count = 0;
    let mut i = 0;
    let bytes = trimmed.as_bytes();

    while i < bytes.len() {
        let remaining = &trimmed[i..];
        let c = bytes[i] as char;

        if c.is_alphabetic() {
            // Check for "until" keyword
            if remaining.to_lowercase().starts_with("until") {
                let after = remaining.get(5..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    until_count += 1;
                    i += 5;
                    continue;
                }
            }

            // Check for "done" keyword
            if remaining.to_lowercase().starts_with("done") {
                let after = remaining.get(4..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    until_count -= 1;
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

    until_count == 0
}

/// Parse a while loop from input string
/// Expects: while condition; do commands; done
pub fn parse_while_loop(input: &str) -> Result<WhileLoop> {
    let trimmed = input.trim();

    // Expect "while" keyword
    if !trimmed.to_lowercase().starts_with("while") {
        return Err(RushError::Syntax("Expected 'while' keyword".to_string()));
    }

    let rest = &trimmed[5..].trim_start();

    // Find the "do" keyword
    let do_pos = find_keyword_position(rest, "do")
        .ok_or_else(|| RushError::Syntax("Expected 'do' keyword in while loop".to_string()))?;

    let condition_str = rest[..do_pos].trim();
    if condition_str.is_empty() {
        return Err(RushError::Syntax("Empty condition in while loop".to_string()));
    }

    // Parse the condition as a command
    let condition = parse_condition(condition_str)?;

    // Find "done" keyword
    let rest_after_do = &rest[do_pos + 2..].trim_start();
    let done_pos = find_keyword_position(rest_after_do, "done")
        .ok_or_else(|| RushError::Syntax("Expected 'done' keyword in while loop".to_string()))?;

    let body_str = rest_after_do[..done_pos].trim();
    // Remove trailing semicolon if present (parser doesn't need it)
    let body_str_clean = body_str.trim_end_matches(';');
    let body = parse_command_list(body_str_clean)?;

    // Phase 3: Store raw body string for pipe and redirection support
    Ok(WhileLoop::new_with_raw_body(condition, body, body_str_clean.to_string()))
}

/// Parse an until loop from input string
/// Expects: until condition; do commands; done
pub fn parse_until_loop(input: &str) -> Result<UntilLoop> {
    let trimmed = input.trim();

    // Expect "until" keyword
    if !trimmed.to_lowercase().starts_with("until") {
        return Err(RushError::Syntax("Expected 'until' keyword".to_string()));
    }

    let rest = &trimmed[5..].trim_start();

    // Find the "do" keyword
    let do_pos = find_keyword_position(rest, "do")
        .ok_or_else(|| RushError::Syntax("Expected 'do' keyword in until loop".to_string()))?;

    let condition_str = rest[..do_pos].trim();
    if condition_str.is_empty() {
        return Err(RushError::Syntax("Empty condition in until loop".to_string()));
    }

    // Parse the condition as a command
    let condition = parse_condition(condition_str)?;

    // Find "done" keyword
    let rest_after_do = &rest[do_pos + 2..].trim_start();
    let done_pos = find_keyword_position(rest_after_do, "done")
        .ok_or_else(|| RushError::Syntax("Expected 'done' keyword in until loop".to_string()))?;

    let body_str = rest_after_do[..done_pos].trim();
    // Remove trailing semicolon if present (parser doesn't need it)
    let body_str_clean = body_str.trim_end_matches(';');
    let body = parse_command_list(body_str_clean)?;

    // Phase 3: Store raw body string for pipe and redirection support
    Ok(UntilLoop::new_with_raw_body(condition, body, body_str_clean.to_string()))
}

/// Parse a condition (command or pipeline)
fn parse_condition(input: &str) -> Result<CompoundList> {
    // For now, treat condition as a simple command
    // In a full implementation, this would handle pipes and complex commands
    let commands = vec![parse_simple_command(input)?];
    Ok(CompoundList::new(commands))
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
            !lower
                .chars()
                .nth(actual_pos + keyword_lower.len())
                .unwrap()
                .is_alphanumeric()
        };

        if before && after {
            return Some(actual_pos);
        }

        pos = actual_pos + 1;
    }

    None
}

/// Execute a while loop
pub fn execute_while_loop(while_loop: &WhileLoop, executor: &mut CommandExecutor) -> Result<i32> {
    let mut exit_code = 0;

    // Loop while condition is true (exit code 0)
    loop {
        // Evaluate condition with variable expansion
        let condition_exit_code =
            execute_compound_list_with_expansion(&while_loop.condition, executor)?;

        // Check if condition is true (exit code 0)
        if condition_exit_code != 0 {
            break; // Condition false, exit loop
        }

        // Phase 3: Execute using raw body string to support pipes and redirections
        // If raw body is available, use it (supports pipes and redirections)
        // Otherwise, fall back to parsed compound list
        if !while_loop.body_raw.is_empty() {
            // Execute raw body string (which may contain pipes, redirections, etc.)
            exit_code = executor.execute(&while_loop.body_raw)?;
        } else {
            // Fallback for backward compatibility
            exit_code = execute_compound_list(&while_loop.body, executor)?;
        }
    }

    // Return exit code from last iteration (or 0 if loop never executed)
    Ok(exit_code)
}

/// Execute an until loop
pub fn execute_until_loop(until_loop: &UntilLoop, executor: &mut CommandExecutor) -> Result<i32> {
    let mut exit_code = 0;

    // Loop until condition is true (exit code 0)
    loop {
        // Evaluate condition with variable expansion
        let condition_exit_code =
            execute_compound_list_with_expansion(&until_loop.condition, executor)?;

        // Check if condition is true (exit code 0)
        if condition_exit_code == 0 {
            break; // Condition true, exit loop
        }

        // Phase 3: Execute using raw body string to support pipes and redirections
        // If raw body is available, use it (supports pipes and redirections)
        // Otherwise, fall back to parsed compound list
        if !until_loop.body_raw.is_empty() {
            // Execute raw body string (which may contain pipes, redirections, etc.)
            exit_code = executor.execute(&until_loop.body_raw)?;
        } else {
            // Fallback for backward compatibility
            exit_code = execute_compound_list(&until_loop.body, executor)?;
        }
    }

    // Return exit code from last iteration (or 0 if loop never executed)
    Ok(exit_code)
}

/// Execute a compound list with variable expansion in commands
/// Phase 2: Applies variable expansion to all parts of the command
fn execute_compound_list_with_expansion(
    compound_list: &CompoundList,
    executor: &mut CommandExecutor,
) -> Result<i32> {
    if compound_list.commands.is_empty() {
        return Ok(0);
    }

    let mut last_exit_code = 0;

    for cmd in &compound_list.commands {
        // Phase 2: Expand variables in program name
        let expanded_program = expand_variables(&cmd.program, executor);
        let fully_expanded_program =
            expand_substitutions(&expanded_program).unwrap_or_else(|_| expanded_program);

        // Phase 2: Expand variables in arguments
        let expanded_args: Vec<String> = cmd
            .args
            .iter()
            .map(|arg| {
                let var_expanded = expand_variables(arg, executor);
                expand_substitutions(&var_expanded).unwrap_or_else(|_| var_expanded)
            })
            .collect();

        // Build command line from expanded parts
        let cmd_line = format!("{} {}", fully_expanded_program, expanded_args.join(" "))
            .trim()
            .to_string();

        // Execute the command through the executor
        last_exit_code = executor.execute(&cmd_line)?;
    }

    Ok(last_exit_code)
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
    fn test_parse_while_basic() {
        let input = "while [ $i -lt 3 ]; do echo $i; done";
        let result = parse_while_loop(input);
        assert!(result.is_ok(), "Should parse basic while loop");
    }

    #[test]
    fn test_parse_while_missing_do() {
        let input = "while [ $i -lt 3 ]; fi";
        let result = parse_while_loop(input);
        assert!(result.is_err(), "Should fail without 'do' keyword");
    }

    #[test]
    fn test_parse_while_missing_done() {
        let input = "while [ $i -lt 3 ]; do echo $i;";
        let result = parse_while_loop(input);
        assert!(result.is_err(), "Should fail without 'done' keyword");
    }

    #[test]
    fn test_parse_until_basic() {
        let input = "until [ $i -eq 3 ]; do echo $i; done";
        let result = parse_until_loop(input);
        assert!(result.is_ok(), "Should parse basic until loop");
    }

    #[test]
    fn test_parse_until_missing_do() {
        let input = "until [ $i -eq 3 ]; fi";
        let result = parse_until_loop(input);
        assert!(result.is_err(), "Should fail without 'do' keyword");
    }

    #[test]
    fn test_parse_until_missing_done() {
        let input = "until [ $i -eq 3 ]; do echo $i;";
        let result = parse_until_loop(input);
        assert!(result.is_err(), "Should fail without 'done' keyword");
    }

    #[test]
    fn test_is_while_complete() {
        assert!(is_while_complete(""));
        assert!(is_while_complete("while [ $i -lt 3 ]; do echo $i; done"));
        assert!(!is_while_complete("while [ $i -lt 3 ]; do echo $i;"));
        assert!(!is_while_complete("while [ $i -lt 3 ];"));
    }

    #[test]
    fn test_is_until_complete() {
        assert!(is_until_complete(""));
        assert!(is_until_complete("until [ $i -eq 3 ]; do echo $i; done"));
        assert!(!is_until_complete("until [ $i -eq 3 ]; do echo $i;"));
        assert!(!is_until_complete("until [ $i -eq 3 ];"));
    }
}
