//! Shell function definition and execution
//!
//! Implements parsing and execution of shell function definitions and calls.

use crate::error::{Result, RushError};
use super::{ShellFunction, CompoundList, Command};
use crate::executor::execute::CommandExecutor;
use std::collections::HashMap;

/// Global function registry
static mut FUNCTION_REGISTRY: Option<HashMap<String, ShellFunction>> = None;

/// Initialize function registry
pub fn init_registry() {
    unsafe {
        FUNCTION_REGISTRY = Some(HashMap::new());
    }
}

/// Register a function
pub fn register_function(name: String, func: ShellFunction) -> Result<()> {
    unsafe {
        if let Some(ref mut registry) = FUNCTION_REGISTRY {
            registry.insert(name, func);
            Ok(())
        } else {
            Err(RushError::Execution("Function registry not initialized".to_string()))
        }
    }
}

/// Get a registered function
pub fn get_function(name: &str) -> Option<ShellFunction> {
    unsafe {
        if let Some(ref registry) = FUNCTION_REGISTRY {
            registry.get(name).cloned()
        } else {
            None
        }
    }
}

/// Check if a statement is a function definition
pub fn is_function_definition(input: &str) -> bool {
    let trimmed = input.trim();
    // Function definition: name() { commands; }
    if let Some(paren_pos) = trimmed.find("()") {
        let name_part = &trimmed[..paren_pos].trim();
        // Check if name is valid identifier
        is_valid_function_name(name_part) && trimmed[paren_pos + 2..].trim_start().starts_with('{')
    } else {
        false
    }
}

/// Check if a name is a valid function name (valid shell identifier)
fn is_valid_function_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let first = name.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Parse a function definition
pub fn parse_function_definition(input: &str) -> Result<ShellFunction> {
    let trimmed = input.trim();

    // Find ()
    let paren_pos = trimmed.find("()")
        .ok_or_else(|| RushError::Syntax("Expected '()' in function definition".to_string()))?;

    let name = trimmed[..paren_pos].trim().to_string();

    if !is_valid_function_name(&name) {
        return Err(RushError::Syntax(format!("Invalid function name: {}", name)));
    }

    // Find opening brace
    let rest = &trimmed[paren_pos + 2..].trim_start();
    if !rest.starts_with('{') {
        return Err(RushError::Syntax("Expected '{' in function definition".to_string()));
    }

    // Find closing brace
    let closing_brace = rest.rfind('}')
        .ok_or_else(|| RushError::Syntax("Expected '}' in function definition".to_string()))?;

    let body_str = &rest[1..closing_brace].trim();

    // Parse body as command list
    let body = parse_command_list(body_str)?;

    Ok(ShellFunction {
        name,
        body,
        parameters: Vec::new(), // Could parse parameter list from () in future
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

/// Execute a function call
pub fn call_function(
    name: &str,
    args: Vec<String>,
    executor: &mut CommandExecutor,
) -> Result<i32> {
    if let Some(func) = get_function(name) {
        // Execute function body
        execute_compound_list(&func.body, executor)
    } else {
        Err(RushError::Execution(format!("Unknown function: {}", name)))
    }
}

/// Execute a compound list (sequence of commands)
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
    fn test_is_function_definition() {
        assert!(is_function_definition("myfunction() { echo hello; }"));
        assert!(is_function_definition("func() { echo; }"));
        assert!(!is_function_definition("echo hello"));
        assert!(!is_function_definition("123invalid() { echo; }"));
    }

    #[test]
    fn test_parse_function_basic() {
        let input = "myfunction() { echo hello; }";
        let result = parse_function_definition(input);
        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name, "myfunction");
    }

    #[test]
    fn test_parse_function_missing_brace() {
        let input = "myfunction() echo hello;";
        let result = parse_function_definition(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_function_missing_closing_brace() {
        let input = "myfunction() { echo hello;";
        let result = parse_function_definition(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_function_name() {
        assert!(is_valid_function_name("myFunc"));
        assert!(is_valid_function_name("_private"));
        assert!(is_valid_function_name("func123"));
        assert!(!is_valid_function_name("123func"));
        assert!(!is_valid_function_name(""));
    }

    #[test]
    fn test_parse_function_multiline() {
        let input = "myfunction() {\n  echo hello\n  echo world\n}";
        let result = parse_function_definition(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_registry() {
        init_registry();
        let func = ShellFunction {
            name: "test".to_string(),
            body: CompoundList::new(vec![]),
            parameters: vec![],
        };
        assert!(register_function("test".to_string(), func).is_ok());
        assert!(get_function("test").is_some());
        assert!(get_function("unknown").is_none());
    }

    #[test]
    fn test_parse_function_invalid_name() {
        let input = "123invalid() { echo; }";
        let result = parse_function_definition(input);
        assert!(result.is_err());
    }
}
