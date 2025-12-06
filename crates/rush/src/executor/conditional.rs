//! Conditional control flow parsing and execution
//!
//! Implements parsing and execution of if/then/elif/else/fi constructs.

use crate::error::{Result, RushError};
use super::{CompoundList, IfBlock, ElifClause, Keyword};

/// Parse a compound list (sequence of commands separated by `;` or newlines)
///
/// # Arguments
/// * `input` - Raw input string containing the commands
///
/// # Returns
/// A CompoundList containing parsed commands
pub fn parse_compound_list(input: &str) -> Result<CompoundList> {
    // This is a placeholder for now - will be fully implemented in Phase 2
    // For now, we'll parse simple commands separated by semicolons

    let mut commands = Vec::new();

    // Split by semicolon to get individual commands
    for cmd_str in input.split(';') {
        let trimmed = cmd_str.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Create a basic command - actual parsing will be done by the main parser
        // This is a simplified version just to get the structure in place
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let program = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();

        commands.push(super::Command::new(program, args));
    }

    Ok(CompoundList::new(commands))
}

/// Parse an if clause (if/then/elif/else/fi)
///
/// Parses the POSIX if/then/fi construct from a raw input string.
/// For User Story 1 (Phase 3), this handles basic if/then/fi without elif/else.
///
/// # Arguments
/// * `input` - Raw input string containing the if statement (must start with "if")
///
/// # Returns
/// An IfBlock representing the parsed if statement
pub fn parse_if_clause(input: &str) -> Result<IfBlock> {
    let trimmed = input.trim();

    // Remove "if" from the beginning
    if !trimmed.starts_with("if") {
        return Err(RushError::Syntax("Expected 'if' keyword at start of conditional".to_string()));
    }

    let rest = &trimmed[2..].trim_start();

    // Find "then" keyword - everything between "if" and "then" is the condition
    let then_pos = find_keyword_position(rest, "then")
        .ok_or_else(|| RushError::Syntax("Expected 'then' keyword after if condition".to_string()))?;

    let condition_str = rest[..then_pos].trim();
    if condition_str.is_empty() {
        return Err(RushError::Syntax("Empty condition in if statement".to_string()));
    }

    let after_then = &rest[then_pos + 4..].trim_start(); // Skip "then"

    // Find "fi" keyword - everything between "then" and "fi" is the then block
    let fi_pos = find_keyword_position(after_then, "fi")
        .ok_or_else(|| RushError::Syntax("Expected 'fi' keyword to close if statement".to_string()))?;

    let then_str = after_then[..fi_pos].trim();

    // Parse condition and then block
    let condition = parse_compound_list(condition_str)?;
    let then_block = parse_compound_list(then_str)?;

    Ok(IfBlock::new(condition, then_block))
}

/// Find the position of a keyword in a string
/// Keywords are recognized only when they are complete words (surrounded by whitespace or special chars)
fn find_keyword_position(s: &str, keyword: &str) -> Option<usize> {
    let keyword_lower = keyword.to_lowercase();
    let s_lower = s.to_lowercase();

    for (i, _) in s_lower.match_indices(&keyword_lower) {
        // Check if this is a complete word (not part of a larger word)
        let is_start_boundary = i == 0 || s[..i].chars().last().map_or(false, |c| c.is_whitespace() || c == ';' || c == '\n');
        let is_end_boundary = i + keyword.len() >= s.len() || {
            let next_char = s[i + keyword.len()..].chars().next();
            next_char.map_or(false, |c| c.is_whitespace() || c == ';' || c == '\n' || c == '>')
        };

        if is_start_boundary && is_end_boundary {
            return Some(i);
        }
    }

    None
}

/// Execute an if block - evaluates the condition and executes the appropriate branch
///
/// For User Story 1 (Phase 3), this handles basic if/then/fi without elif/else.
/// The condition is executed, and if its exit code is 0, the then_block is executed.
///
/// # Arguments
/// * `if_block` - The IfBlock to execute
/// * `executor` - The CommandExecutor for executing commands (requires mutable access)
///
/// # Returns
/// The exit code of the executed branch (exit code of last command in the branch)
pub fn execute_if_block(if_block: &IfBlock, executor: &mut super::execute::CommandExecutor) -> Result<i32> {
    // Execute condition commands and get exit code
    let condition_exit_code = execute_compound_list(&if_block.condition, executor)?;

    // Check condition result
    if condition_exit_code == 0 {
        // Condition succeeded - execute then block
        execute_compound_list(&if_block.then_block, executor)
    } else {
        // Condition failed - check for elif/else
        // For now (Phase 3), we just return the condition's exit code
        // else/elif handling will be added in Phase 4+

        if let Some(else_block) = &if_block.else_block {
            execute_compound_list(else_block, executor)
        } else {
            Ok(condition_exit_code)
        }
    }
}

/// Execute a compound list (sequence of commands) and return the exit code of the last command
///
/// # Arguments
/// * `compound_list` - The list of commands to execute
/// * `executor` - The CommandExecutor for executing commands (requires mutable access)
///
/// # Returns
/// The exit code of the last command in the list, or 0 if the list is empty
pub fn execute_compound_list(compound_list: &CompoundList, executor: &mut super::execute::CommandExecutor) -> Result<i32> {
    if compound_list.is_empty() {
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

/// Parse the optional else/elif part of an if statement
///
/// # Arguments
/// * `input` - Raw input string after the initial then block
///
/// # Returns
/// A tuple of (elif_clauses, else_block)
pub fn parse_else_part(input: &str) -> Result<(Vec<ElifClause>, Option<CompoundList>)> {
    // Placeholder for full implementation in Phase 4
    // For now, return empty elif clauses and no else block
    Ok((Vec::new(), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compound_list_empty() {
        let result = parse_compound_list("");
        assert!(result.is_ok());
        let compound_list = result.unwrap();
        assert!(compound_list.is_empty());
    }

    #[test]
    fn test_parse_compound_list_single_command() {
        let result = parse_compound_list("echo hello");
        assert!(result.is_ok());
        let compound_list = result.unwrap();
        assert_eq!(compound_list.len(), 1);
    }

    #[test]
    fn test_parse_compound_list_multiple_commands() {
        let result = parse_compound_list("echo hello; echo world");
        assert!(result.is_ok());
        let compound_list = result.unwrap();
        assert_eq!(compound_list.len(), 2);
    }
}
