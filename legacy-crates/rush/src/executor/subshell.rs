//! Subshell execution
//!
//! Implements subshell execution for command groups in parentheses.

use super::{Command, CompoundList, Subshell};
use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;

/// Check if a statement is a subshell (starts with parenthesis)
pub fn is_subshell(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed.starts_with('(') && trimmed.ends_with(')')
}

/// Parse a subshell command
pub fn parse_subshell(input: &str) -> Result<Subshell> {
    let trimmed = input.trim();

    // Verify subshell format
    if !trimmed.starts_with('(') {
        return Err(RushError::Syntax("Subshell must start with '('".to_string()));
    }

    if !trimmed.ends_with(')') {
        return Err(RushError::Syntax("Subshell must end with ')'".to_string()));
    }

    // Extract commands between parentheses
    let content = &trimmed[1..trimmed.len() - 1].trim();

    if content.is_empty() {
        return Err(RushError::Syntax("Empty subshell".to_string()));
    }

    // Parse commands
    let body = parse_command_list(content)?;

    Ok(Subshell { body })
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

/// Execute a subshell
pub fn execute_subshell(subshell: &Subshell, executor: &mut CommandExecutor) -> Result<i32> {
    // In a simplified implementation, execute in current shell
    // In a full implementation, this would fork a new process
    execute_compound_list(&subshell.body, executor)
}

/// Execute a compound list (sequence of commands)
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
    fn test_is_subshell() {
        assert!(is_subshell("( echo hello )"));
        assert!(is_subshell("(echo hello; echo world)"));
        assert!(!is_subshell("echo hello"));
        assert!(!is_subshell("(echo hello"));
    }

    #[test]
    fn test_parse_subshell_basic() {
        let input = "( echo hello )";
        let result = parse_subshell(input);
        assert!(result.is_ok());
        let subshell = result.unwrap();
        assert_eq!(subshell.body.commands.len(), 1);
    }

    #[test]
    fn test_parse_subshell_multiple_commands() {
        let input = "( echo hello; echo world )";
        let result = parse_subshell(input);
        assert!(result.is_ok());
        let subshell = result.unwrap();
        assert_eq!(subshell.body.commands.len(), 2);
    }

    #[test]
    fn test_parse_subshell_empty() {
        let input = "( )";
        let result = parse_subshell(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_subshell_missing_closing() {
        let input = "( echo hello";
        let result = parse_subshell(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_subshell_missing_opening() {
        let input = "echo hello )";
        let result = parse_subshell(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_subshell_nested() {
        let input = "( ( echo hello ) )";
        let result = parse_subshell(input);
        // This should parse the outer parens, inner ones are part of content
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_subshell_with_newlines() {
        let input = "(\n  echo hello\n  echo world\n)";
        let result = parse_subshell(input);
        assert!(result.is_ok());
        let subshell = result.unwrap();
        assert_eq!(subshell.body.commands.len(), 2);
    }
}
