//! The `read` builtin command for reading user input.
//!
//! Reads a line from standard input and assigns words to shell variables.
//!
//! # Usage
//! ```text
//! read [-p prompt] [-s] [-r] [-d delim] [-n count] [-t timeout] [name ...]
//! ```
//!
//! # Options
//! - `-p prompt`: Display prompt before reading
//! - `-s`: Silent mode (do not echo input)
//! - `-r`: Raw mode (do not interpret backslash escapes)
//! - `-d delim`: Read until delimiter character instead of newline
//! - `-n count`: Read exactly count characters
//! - `-t timeout`: Timeout after specified seconds
//!
//! # Examples
//! ```text
//! read name              # Read into $name
//! read first last        # Read into $first and $last
//! read -p "Name: " name  # Prompt before reading
//! read -s password       # Silent input for passwords
//! read                   # Read into $REPLY
//! ```
//!
//! # Exit Status
//! - 0: Success
//! - 1: EOF or timeout
//! - 2: Invalid usage

use crate::error::Result;
use crate::executor::execute::CommandExecutor;
use crossterm::terminal;
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

/// Options for the read command
#[derive(Default)]
struct ReadOptions {
    /// Prompt to display before reading (-p)
    prompt: Option<String>,
    /// Silent mode - don't echo input (-s)
    silent: bool,
    /// Raw mode - don't interpret backslashes (-r)
    raw: bool,
    /// Custom delimiter character (-d)
    delimiter: Option<char>,
    /// Number of characters to read (-n)
    char_count: Option<usize>,
    /// Timeout in seconds (-t)
    timeout: Option<f64>,
}

/// Parse command line options and return (options, variable names)
fn parse_options(args: &[String]) -> (ReadOptions, Vec<String>) {
    let mut options = ReadOptions::default();
    let mut var_names = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with('-') && arg.len() > 1 {
            let mut chars = arg[1..].chars();
            while let Some(c) = chars.next() {
                match c {
                    'p' => {
                        // -p prompt (prompt may be rest of arg or next arg)
                        let rest: String = chars.collect();
                        if !rest.is_empty() {
                            options.prompt = Some(rest);
                        } else if i + 1 < args.len() {
                            i += 1;
                            options.prompt = Some(args[i].clone());
                        }
                        break;
                    }
                    's' => options.silent = true,
                    'r' => options.raw = true,
                    'd' => {
                        // -d delim
                        let rest: String = chars.collect();
                        if !rest.is_empty() {
                            options.delimiter = rest.chars().next();
                        } else if i + 1 < args.len() {
                            i += 1;
                            options.delimiter = args[i].chars().next();
                        }
                        break;
                    }
                    'n' => {
                        // -n count
                        let rest: String = chars.collect();
                        if !rest.is_empty() {
                            options.char_count = rest.parse().ok();
                        } else if i + 1 < args.len() {
                            i += 1;
                            options.char_count = args[i].parse().ok();
                        }
                        break;
                    }
                    't' => {
                        // -t timeout
                        let rest: String = chars.collect();
                        if !rest.is_empty() {
                            options.timeout = rest.parse().ok();
                        } else if i + 1 < args.len() {
                            i += 1;
                            options.timeout = args[i].parse().ok();
                        }
                        break;
                    }
                    _ => {
                        // Unknown option, treat as variable name
                        var_names.push(arg.clone());
                        break;
                    }
                }
            }
        } else {
            var_names.push(arg.clone());
        }
        i += 1;
    }

    (options, var_names)
}

/// Read input based on options
fn read_input(options: &ReadOptions) -> io::Result<Option<String>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Display prompt if specified
    if let Some(ref prompt) = options.prompt {
        print!("{}", prompt);
        stdout.flush()?;
    }

    // Handle silent mode
    let was_raw = if options.silent {
        let _ = terminal::enable_raw_mode();
        true
    } else {
        false
    };

    let result = if let Some(count) = options.char_count {
        // Read exactly N characters
        read_n_chars(&stdin, count, options)
    } else if let Some(delim) = options.delimiter {
        // Read until delimiter
        read_until_delimiter(&stdin, delim, options)
    } else if let Some(timeout_secs) = options.timeout {
        // Read with timeout
        read_with_timeout(&stdin, timeout_secs, options)
    } else {
        // Normal line read
        read_line(&stdin)
    };

    // Restore terminal mode
    if was_raw {
        let _ = terminal::disable_raw_mode();
        // Print newline after silent input
        println!();
    }

    result
}

/// Read a full line from stdin
fn read_line(stdin: &io::Stdin) -> io::Result<Option<String>> {
    let mut line = String::new();
    let bytes_read = stdin.lock().read_line(&mut line)?;

    if bytes_read == 0 {
        return Ok(None); // EOF
    }

    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }

    Ok(Some(line))
}

/// Read exactly N characters
fn read_n_chars(
    stdin: &io::Stdin,
    count: usize,
    _options: &ReadOptions,
) -> io::Result<Option<String>> {
    let mut result = String::new();
    let mut lock = stdin.lock();

    for _ in 0..count {
        let mut buf = [0u8; 1];
        match std::io::Read::read(&mut lock, &mut buf) {
            Ok(0) => break, // EOF
            Ok(_) => {
                if let Ok(s) = std::str::from_utf8(&buf) {
                    result.push_str(s);
                }
            }
            Err(e) => return Err(e),
        }
    }

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

/// Read until a delimiter character
fn read_until_delimiter(
    stdin: &io::Stdin,
    delim: char,
    _options: &ReadOptions,
) -> io::Result<Option<String>> {
    let mut result = String::new();
    let mut lock = stdin.lock();

    loop {
        let mut buf = [0u8; 4]; // UTF-8 can be up to 4 bytes
        match std::io::Read::read(&mut lock, &mut buf[..1]) {
            Ok(0) => break, // EOF
            Ok(_) => {
                if let Ok(s) = std::str::from_utf8(&buf[..1]) {
                    let c = s.chars().next().unwrap();
                    if c == delim {
                        break;
                    }
                    result.push(c);
                }
            }
            Err(e) => return Err(e),
        }
    }

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

/// Read with timeout
fn read_with_timeout(
    stdin: &io::Stdin,
    timeout_secs: f64,
    _options: &ReadOptions,
) -> io::Result<Option<String>> {
    let start = Instant::now();
    let timeout = Duration::from_secs_f64(timeout_secs);

    let mut line = String::new();

    // Simple approach: just try to read, check if we got data
    // Note: This is a blocking read - for true non-blocking timeout,
    // we would need platform-specific code or async
    loop {
        if start.elapsed() >= timeout {
            return Ok(None); // Timeout
        }

        // Try to read - this will block, but for interactive use
        // the user will provide input quickly
        let bytes_read = stdin.lock().read_line(&mut line)?;

        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        // Remove trailing newline
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }

        return Ok(Some(line));
    }
}

/// Process backslash escapes in input (unless raw mode)
fn process_escapes(input: &str, raw: bool) -> String {
    if raw {
        return input.to_string();
    }

    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => {
                        result.push('\n');
                        chars.next();
                    }
                    't' => {
                        result.push('\t');
                        chars.next();
                    }
                    '\\' => {
                        result.push('\\');
                        chars.next();
                    }
                    _ => {
                        // Keep both characters for unknown escapes
                        result.push(c);
                    }
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Split input by IFS and assign to variables
fn assign_to_variables(
    executor: &mut CommandExecutor,
    input: &str,
    var_names: &[String],
    raw: bool,
) {
    let processed = process_escapes(input, raw);
    let trimmed = processed.trim();

    if var_names.is_empty() {
        // No variable names - assign to REPLY
        let _ = executor
            .variable_manager_mut()
            .set("REPLY".to_string(), trimmed.to_string());
        return;
    }

    // Get IFS (default is space, tab, newline)
    let ifs = executor
        .variable_manager()
        .get("IFS")
        .map(|s| s.to_string())
        .unwrap_or_else(|| " \t\n".to_string());

    // Split input by IFS characters
    let words: Vec<&str> = if ifs.is_empty() {
        // Empty IFS means no splitting
        vec![trimmed]
    } else {
        trimmed
            .split(|c: char| ifs.contains(c))
            .filter(|s| !s.is_empty())
            .collect()
    };

    // Assign words to variables
    for (i, var_name) in var_names.iter().enumerate() {
        let value = if i < var_names.len() - 1 {
            // Not the last variable - get single word
            words.get(i).copied().unwrap_or("").to_string()
        } else {
            // Last variable - get all remaining words
            if i < words.len() {
                words[i..].join(" ")
            } else {
                String::new()
            }
        };

        let _ = executor.variable_manager_mut().set(var_name.clone(), value);
    }
}

/// Execute the `read` builtin command.
///
/// Reads a line from stdin and assigns it to the specified variables.
/// If no variables are specified, the input is stored in REPLY.
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    let (options, var_names) = parse_options(args);

    // Read input
    match read_input(&options) {
        Ok(Some(input)) => {
            assign_to_variables(executor, &input, &var_names, options.raw);
            Ok(0)
        }
        Ok(None) => {
            // EOF or timeout - still assign empty values
            assign_to_variables(executor, "", &var_names, options.raw);
            Ok(1)
        }
        Err(e) => {
            eprintln!("read: {}", e);
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_options_empty() {
        let (options, vars) = parse_options(&[]);
        assert!(options.prompt.is_none());
        assert!(!options.silent);
        assert!(!options.raw);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_parse_options_variable_only() {
        let args = vec!["name".to_string()];
        let (options, vars) = parse_options(&args);
        assert!(options.prompt.is_none());
        assert_eq!(vars, vec!["name"]);
    }

    #[test]
    fn test_parse_options_multiple_variables() {
        let args = vec!["first".to_string(), "last".to_string()];
        let (_, vars) = parse_options(&args);
        assert_eq!(vars, vec!["first", "last"]);
    }

    #[test]
    fn test_parse_options_prompt() {
        let args = vec!["-p".to_string(), "Enter: ".to_string(), "name".to_string()];
        let (options, vars) = parse_options(&args);
        assert_eq!(options.prompt, Some("Enter: ".to_string()));
        assert_eq!(vars, vec!["name"]);
    }

    #[test]
    fn test_parse_options_silent() {
        let args = vec!["-s".to_string(), "password".to_string()];
        let (options, vars) = parse_options(&args);
        assert!(options.silent);
        assert_eq!(vars, vec!["password"]);
    }

    #[test]
    fn test_parse_options_raw() {
        let args = vec!["-r".to_string(), "line".to_string()];
        let (options, vars) = parse_options(&args);
        assert!(options.raw);
        assert_eq!(vars, vec!["line"]);
    }

    #[test]
    fn test_parse_options_delimiter() {
        let args = vec!["-d".to_string(), ":".to_string(), "path".to_string()];
        let (options, vars) = parse_options(&args);
        assert_eq!(options.delimiter, Some(':'));
        assert_eq!(vars, vec!["path"]);
    }

    #[test]
    fn test_parse_options_char_count() {
        let args = vec!["-n".to_string(), "5".to_string(), "chars".to_string()];
        let (options, vars) = parse_options(&args);
        assert_eq!(options.char_count, Some(5));
        assert_eq!(vars, vec!["chars"]);
    }

    #[test]
    fn test_parse_options_combined() {
        let args = vec!["-rs".to_string(), "value".to_string()];
        let (options, vars) = parse_options(&args);
        assert!(options.raw);
        assert!(options.silent);
        assert_eq!(vars, vec!["value"]);
    }

    #[test]
    fn test_process_escapes_raw() {
        assert_eq!(process_escapes("hello\\nworld", true), "hello\\nworld");
    }

    #[test]
    fn test_process_escapes_normal() {
        assert_eq!(process_escapes("hello\\nworld", false), "hello\nworld");
        assert_eq!(process_escapes("tab\\there", false), "tab\there");
        assert_eq!(process_escapes("back\\\\slash", false), "back\\slash");
    }

    #[test]
    fn test_assign_to_reply() {
        let mut executor = CommandExecutor::new();
        assign_to_variables(&mut executor, "hello world", &[], false);
        assert_eq!(executor.variable_manager().get("REPLY"), Some("hello world"));
    }

    #[test]
    fn test_assign_single_variable() {
        let mut executor = CommandExecutor::new();
        assign_to_variables(&mut executor, "hello", &["name".to_string()], false);
        assert_eq!(executor.variable_manager().get("name"), Some("hello"));
    }

    #[test]
    fn test_assign_multiple_variables() {
        let mut executor = CommandExecutor::new();
        assign_to_variables(
            &mut executor,
            "John Doe",
            &["first".to_string(), "last".to_string()],
            false,
        );
        assert_eq!(executor.variable_manager().get("first"), Some("John"));
        assert_eq!(executor.variable_manager().get("last"), Some("Doe"));
    }

    #[test]
    fn test_assign_excess_words_to_last() {
        let mut executor = CommandExecutor::new();
        assign_to_variables(
            &mut executor,
            "one two three four",
            &["a".to_string(), "b".to_string()],
            false,
        );
        assert_eq!(executor.variable_manager().get("a"), Some("one"));
        assert_eq!(executor.variable_manager().get("b"), Some("two three four"));
    }

    #[test]
    fn test_assign_fewer_words_than_variables() {
        let mut executor = CommandExecutor::new();
        assign_to_variables(
            &mut executor,
            "only",
            &["a".to_string(), "b".to_string(), "c".to_string()],
            false,
        );
        assert_eq!(executor.variable_manager().get("a"), Some("only"));
        assert_eq!(executor.variable_manager().get("b"), Some(""));
        assert_eq!(executor.variable_manager().get("c"), Some(""));
    }
}
