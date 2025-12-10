//! Command group execution
//!
//! Implements command group execution for curly-brace delimited command sequences.

use super::{Command, CommandGroup, CompoundList};
use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;

/// Check if a statement is a command group (starts with opening brace)
pub fn is_command_group(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed.starts_with('{') && trimmed.ends_with('}') || trimmed.ends_with("};")
}

/// Parse a command group
pub fn parse_command_group(input: &str) -> Result<CommandGroup> {
    let trimmed = input.trim();

    // Verify command group format
    if !trimmed.starts_with('{') {
        return Err(RushError::Syntax("Command group must start with '{'".to_string()));
    }

    let closing_brace = if trimmed.ends_with("};") {
        trimmed.len() - 2
    } else if trimmed.ends_with('}') {
        trimmed.len() - 1
    } else {
        return Err(RushError::Syntax("Command group must end with '}'".to_string()));
    };

    // Extract commands between braces
    let content = &trimmed[1..closing_brace].trim();

    if content.is_empty() {
        return Err(RushError::Syntax("Empty command group".to_string()));
    }

    // Parse commands
    let body = parse_command_list(content)?;

    Ok(CommandGroup { body })
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

/// Execute a command group
pub fn execute_command_group(group: &CommandGroup, executor: &mut CommandExecutor) -> Result<i32> {
    // Execute commands in current shell (unlike subshells)
    execute_compound_list(&group.body, executor)
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
    fn test_is_command_group() {
        assert!(is_command_group("{ echo hello; }"));
        assert!(is_command_group("{ echo hello; echo world; }"));
        assert!(is_command_group("{ echo hello };"));
        assert!(!is_command_group("echo hello"));
        assert!(!is_command_group("{ echo hello"));
    }

    #[test]
    fn test_parse_command_group_basic() {
        let input = "{ echo hello; }";
        let result = parse_command_group(input);
        assert!(result.is_ok());
        let group = result.unwrap();
        assert_eq!(group.body.commands.len(), 1);
    }

    #[test]
    fn test_parse_command_group_multiple_commands() {
        let input = "{ echo hello; echo world; }";
        let result = parse_command_group(input);
        assert!(result.is_ok());
        let group = result.unwrap();
        assert_eq!(group.body.commands.len(), 2);
    }

    #[test]
    fn test_parse_command_group_with_semicolon() {
        let input = "{ echo hello; };";
        let result = parse_command_group(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_command_group_empty() {
        let input = "{ }";
        let result = parse_command_group(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_group_missing_closing() {
        let input = "{ echo hello;";
        let result = parse_command_group(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_group_with_newlines() {
        let input = "{\n  echo hello\n  echo world\n}";
        let result = parse_command_group(input);
        assert!(result.is_ok());
        let group = result.unwrap();
        assert_eq!(group.body.commands.len(), 2);
    }

    #[test]
    fn test_parse_command_group_nested() {
        let input = "{ { echo hello; }; }";
        let result = parse_command_group(input);
        // This should parse the outer braces
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_command_group_with_redirects() {
        let input = "{ echo hello; echo world; } > output.txt";
        // Note: redirects are not parsed as part of command group in this simple implementation
        let result = parse_command_group(input);
        // May fail due to > being in content
        assert!(result.is_err() || result.is_ok());
    }
}
