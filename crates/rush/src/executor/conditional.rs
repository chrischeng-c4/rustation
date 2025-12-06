//! Conditional control flow parsing and execution
//!
//! Implements parsing and execution of if/then/elif/else/fi constructs.

use crate::error::{Result, RushError};
use super::{CompoundList, IfBlock, ElifClause, Keyword};

/// Check if a statement appears to be syntactically complete
/// Useful for REPL multiline support to detect when user has finished entering a statement
pub fn is_statement_complete(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Check for unmatched if/fi pairs
    // Count "if" keywords and "fi" keywords
    let mut if_count = 0;
    let mut i = 0;
    let bytes = trimmed.as_bytes();

    while i < bytes.len() {
        let remaining = &trimmed[i..];
        let c = bytes[i] as char;

        if c.is_alphabetic() {
            // Check for "if" keyword
            if remaining.to_lowercase().starts_with("if") {
                let after = remaining.get(2..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    if_count += 1;
                    i += 2;
                    continue;
                }
            }

            // Check for "fi" keyword
            if remaining.to_lowercase().starts_with("fi") {
                let after = remaining.get(2..).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    if_count -= 1;
                    i += 2;
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

    // Statement is complete if all if statements have been closed with fi
    if_count == 0
}

/// Parse a compound list (sequence of commands separated by `;` or newlines)
///
/// # Arguments
/// * `input` - Raw input string containing the commands
///
/// # Returns
/// A CompoundList containing parsed commands
pub fn parse_compound_list(input: &str) -> Result<CompoundList> {
    let mut commands = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        let trimmed = remaining.trim_start();
        if trimmed.is_empty() {
            break;
        }

        // Check if we start with an if statement
        if trimmed.starts_with("if") && (trimmed.len() == 2 || trimmed.chars().nth(2).map_or(false, |c| c.is_whitespace())) {
            // Parse the entire if statement
            match parse_if_clause(trimmed) {
                Ok(_if_block) => {
                    // Found a complete if statement - create a marker command that includes the raw if statement
                    // We'll use the "__if__" program name with the raw statement as the first argument
                    let fi_pos = find_matching_fi(trimmed)
                        .ok_or_else(|| RushError::Syntax("Unmatched 'if' in compound list".to_string()))?;

                    let if_stmt = trimmed[..fi_pos].to_string();
                    commands.push(super::Command::new("__if__".to_string(), vec![if_stmt]));

                    // Skip past this if statement
                    remaining = &trimmed[fi_pos..];
                },
                Err(e) => {
                    // If parsing fails, just return the error
                    return Err(e);
                }
            }
        } else {
            // Find the next semicolon or end of string
            let end_pos = trimmed.find(';').unwrap_or(trimmed.len());
            let cmd_str = trimmed[..end_pos].trim();

            if !cmd_str.is_empty() {
                let parts: Vec<&str> = cmd_str.split_whitespace().collect();
                if !parts.is_empty() {
                    let program = parts[0].to_string();
                    let args = parts[1..].iter().map(|s| s.to_string()).collect();
                    commands.push(super::Command::new(program, args));
                }
            }

            // Move past the command and the semicolon
            if end_pos < trimmed.len() {
                remaining = &trimmed[end_pos + 1..];
            } else {
                break;
            }
        }
    }

    Ok(CompoundList::new(commands))
}

/// Find the position of the matching 'fi' for an 'if' statement
/// Counts nested if/fi pairs to find the correct matching fi
fn find_matching_fi(input: &str) -> Option<usize> {
    let mut if_depth = 0;
    let mut in_word = false;
    let mut word_start = 0;
    let mut i = 0;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        let c = bytes[i] as char;

        if c.is_whitespace() || c == ';' {
            if in_word {
                let word = &input[word_start..i];
                if word.eq_ignore_ascii_case("if") {
                    if_depth += 1;
                } else if word.eq_ignore_ascii_case("fi") {
                    if_depth -= 1;
                    if if_depth == 0 {
                        // Found the matching fi, return position after it
                        return Some(i);
                    }
                }
                in_word = false;
            }
            i += 1;
        } else {
            if !in_word {
                word_start = i;
                in_word = true;
            }
            i += 1;
        }
    }

    // Handle the last word if any
    if in_word {
        let word = &input[word_start..];
        if word.eq_ignore_ascii_case("if") {
            if_depth += 1;
        } else if word.eq_ignore_ascii_case("fi") {
            if_depth -= 1;
            if if_depth == 0 {
                return Some(input.len());
            }
        }
    }

    None
}

/// Parse an if clause (if/then/elif/else/fi)
///
/// Parses the POSIX if/then/fi construct from a raw input string.
/// For User Story 1 (Phase 3), this handles basic if/then/fi without elif/else.
/// For User Story 2 (Phase 4), this also handles if/then/else/fi.
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

    // Look for "elif", "else", or "fi" keyword
    // For "fi", we need to find the MATCHING fi, not just the first one (for nested conditionals)
    let elif_pos = find_keyword_position(after_then, "elif");
    let else_pos = find_keyword_position(after_then, "else");
    let fi_pos = find_matching_fi_position(after_then);

    // Determine which comes first and parse accordingly
    let (then_str, elif_clauses, else_block) = match (elif_pos, else_pos, fi_pos) {
        (Some(elif_idx), Some(else_idx), Some(fi_idx)) => {
            // All three found - determine which comes first
            if elif_idx < else_idx {
                // elif comes first
                let then_part = after_then[..elif_idx].trim();
                let after_elif = &after_then[elif_idx..];
                let (elif_clauses, else_block) = parse_else_part(after_elif)?;
                (then_part, elif_clauses, else_block)
            } else {
                // else comes first
                let then_part = after_then[..else_idx].trim();
                let after_else = &after_then[else_idx..];
                let (elif_clauses, else_block) = parse_else_part(after_else)?;
                (then_part, elif_clauses, else_block)
            }
        }
        (Some(elif_idx), None, Some(fi_idx)) => {
            // elif and fi found
            let then_part = after_then[..elif_idx].trim();
            let after_elif = &after_then[elif_idx..];
            let (elif_clauses, else_block) = parse_else_part(after_elif)?;
            (then_part, elif_clauses, else_block)
        }
        (None, Some(else_idx), Some(fi_idx)) => {
            // else and fi found
            let then_part = after_then[..else_idx].trim();
            let after_else = &after_then[else_idx..];
            let (elif_clauses, else_block) = parse_else_part(after_else)?;
            (then_part, elif_clauses, else_block)
        }
        (None, None, Some(fi_idx)) => {
            // Only fi found, no else/elif
            let then_part = after_then[..fi_idx].trim();
            (then_part, Vec::new(), None)
        }
        _ => {
            // No valid fi found
            return Err(RushError::Syntax("Expected 'fi' keyword to close if statement".to_string()));
        }
    };

    // Parse condition and then block
    let condition = parse_compound_list(condition_str)?;
    let then_block = parse_compound_list(then_str)?;

    // Create if block
    let mut if_block = IfBlock::new(condition, then_block);

    // Add elif clauses
    for elif_clause in elif_clauses {
        if_block.add_elif(elif_clause);
    }

    // Set else block if present
    if let Some(else_block) = else_block {
        if_block.set_else(else_block);
    }

    Ok(if_block)
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

/// Find the position of the matching 'fi' keyword for an 'if' statement
/// Handles nested if/fi pairs correctly
fn find_matching_fi_position(s: &str) -> Option<usize> {
    let mut if_depth = 1; // We're already inside an if
    let mut i = 0;
    let bytes = s.as_bytes();

    while i < bytes.len() {
        let c = bytes[i] as char;

        // Try to match keywords
        if c.is_alphabetic() || c == '_' {
            let remaining = &s[i..];

            // Check for "if" keyword
            if remaining.to_lowercase().starts_with("if") {
                let after_if = remaining.get(2..).unwrap_or("");
                if after_if.is_empty() || !after_if.chars().next().unwrap().is_alphanumeric() {
                    if_depth += 1;
                    i += 2;
                    continue;
                }
            }

            // Check for "fi" keyword
            if remaining.to_lowercase().starts_with("fi") {
                let after_fi = remaining.get(2..).unwrap_or("");
                if after_fi.is_empty() || !after_fi.chars().next().unwrap().is_alphanumeric() {
                    if_depth -= 1;
                    if if_depth == 0 {
                        return Some(i);
                    }
                    i += 2;
                    continue;
                }
            }

            // Skip the rest of the word
            while i < bytes.len() && (bytes[i] as char).is_alphanumeric() {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    None
}

/// Execute an if block - evaluates the condition and executes the appropriate branch
///
/// For User Story 1 (Phase 3), this handles basic if/then/fi without elif/else.
/// For User Story 2 (Phase 4), this also handles else clause.
/// For User Story 3 (Phase 5), this also handles elif clauses with short-circuit evaluation.
///
/// The condition is executed, and if its exit code is 0, the then_block is executed.
/// If the condition fails, elif conditions are checked in order until one succeeds.
/// If no conditions succeed, the else block is executed if present.
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
        // Condition failed - check elif clauses (short-circuit evaluation)
        for elif_clause in &if_block.elif_clauses {
            let elif_condition_code = execute_compound_list(&elif_clause.condition, executor)?;
            if elif_condition_code == 0 {
                // This elif condition succeeded - execute its then block
                return execute_compound_list(&elif_clause.then_block, executor);
            }
        }

        // No elif succeeded - check for else block
        if let Some(else_block) = &if_block.else_block {
            execute_compound_list(else_block, executor)
        } else {
            // No else block - return the original condition's exit code
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
        // Check if this is a marker for a nested if statement
        if cmd.program == "__if__" && !cmd.args.is_empty() {
            // This is a nested if statement - parse and execute it
            let if_stmt = &cmd.args[0];
            let if_block = parse_if_clause(if_stmt)?;
            last_exit_code = execute_if_block(&if_block, executor)?;
        } else {
            // Build command line from the command
            let cmd_line = format!("{} {}", cmd.program, cmd.args.join(" ")).trim().to_string();

            // Execute the command through the executor
            last_exit_code = executor.execute(&cmd_line)?;
        }
    }

    Ok(last_exit_code)
}

/// Parse the optional else/elif part of an if statement
///
/// For User Story 2 (Phase 4), this handles basic else clause.
/// For User Story 3 (Phase 5), this also handles elif clauses.
///
/// # Arguments
/// * `input` - Raw input string after the last then/fi block
///
/// # Returns
/// A tuple of (elif_clauses, else_block)
pub fn parse_else_part(input: &str) -> Result<(Vec<ElifClause>, Option<CompoundList>)> {
    let trimmed = input.trim();

    // Check if there's an else/elif clause
    if trimmed.is_empty() {
        // No else clause
        return Ok((Vec::new(), None));
    }

    // Look for elif keyword first
    if trimmed.starts_with("elif") && (trimmed.len() == 4 || trimmed.chars().nth(4).map_or(false, |c| c.is_whitespace() || c == ';')) {
        // We have an elif clause - parse it recursively
        // elif condition; then block; elif|else|fi
        let rest = &trimmed[4..].trim_start();

        // Find "then" keyword
        let then_pos = find_keyword_position(rest, "then")
            .ok_or_else(|| RushError::Syntax("Expected 'then' keyword after elif condition".to_string()))?;

        let elif_condition_str = rest[..then_pos].trim();
        if elif_condition_str.is_empty() {
            return Err(RushError::Syntax("Empty condition in elif statement".to_string()));
        }

        let after_then = &rest[then_pos + 4..].trim_start(); // Skip "then"

        // Find either "elif", "else", or "fi"
        let elif_pos = find_keyword_position(after_then, "elif");
        let else_pos = find_keyword_position(after_then, "else");
        let fi_pos = find_keyword_position(after_then, "fi");

        // Determine which comes first
        let then_block_end = [elif_pos, else_pos, fi_pos]
            .iter()
            .filter_map(|&pos| pos)
            .min()
            .ok_or_else(|| RushError::Syntax("Expected elif/else/fi after elif then block".to_string()))?;

        let elif_then_str = after_then[..then_block_end].trim();

        // Parse the elif condition and then block
        let elif_condition = parse_compound_list(elif_condition_str)?;
        let elif_then_block = parse_compound_list(elif_then_str)?;

        // Recursively parse the rest (might be more elif or else)
        let after_block = &after_then[then_block_end..].trim_start();
        let (mut elif_clauses, else_block) = parse_else_part(after_block)?;

        // Prepend this elif clause to the list
        let new_elif = ElifClause::new(elif_condition, elif_then_block);
        let mut result_clauses = vec![new_elif];
        result_clauses.extend(elif_clauses);

        Ok((result_clauses, else_block))
    } else if trimmed.starts_with("else") && (trimmed.len() == 4 || trimmed.chars().nth(4).map_or(false, |c| c.is_whitespace() || c == ';')) {
        // We have an else clause - parse until we find "fi"
        let after_else = &trimmed[4..].trim_start();

        // Find the "fi" keyword to know where the else block ends
        let fi_pos = find_keyword_position(after_else, "fi")
            .ok_or_else(|| RushError::Syntax("Expected 'fi' keyword to close else block".to_string()))?;

        let else_block_str = after_else[..fi_pos].trim();

        // Parse the else block as a compound list (excluding "fi")
        let else_block = parse_compound_list(else_block_str)?;

        Ok((Vec::new(), Some(else_block)))
    } else if trimmed.starts_with("fi") {
        // Just fi, no else clause
        Ok((Vec::new(), None))
    } else {
        // Unrecognized keyword or end of input
        Ok((Vec::new(), None))
    }
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
