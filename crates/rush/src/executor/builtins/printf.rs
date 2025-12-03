//! Implementation of the `printf` builtin command
//!
//! The `printf` builtin prints formatted output.
//!
//! Usage:
//! - `printf format [args...]` - Print formatted output
//!
//! Format specifiers:
//! - `%s` - String
//! - `%d` - Integer (decimal)
//! - `%c` - Character (first char of string)
//! - `%%` - Literal percent sign
//!
//! Escape sequences:
//! - `\n` - Newline
//! - `\t` - Tab
//! - `\\` - Backslash

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `printf` builtin command
///
/// # Arguments
/// * `_executor` - Command executor (not used)
/// * `args` - Format string and arguments
///
/// # Returns
/// * `Ok(0)` - Success
/// * `Ok(1)` - Error (missing format string)
pub fn execute(_executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("printf: usage: printf format [arguments]");
        return Ok(1);
    }

    let format = &args[0];
    let format_args = &args[1..];

    let output = format_string(format, format_args);
    print!("{}", output);

    Ok(0)
}

/// Format a string according to printf format specifiers
fn format_string(format: &str, args: &[String]) -> String {
    let mut result = String::new();
    let mut chars = format.chars().peekable();
    let mut arg_index = 0;

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                // Handle escape sequences
                if let Some(&next) = chars.peek() {
                    let escaped = match next {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '0' => '\0',
                        _ => {
                            result.push('\\');
                            continue;
                        }
                    };
                    chars.next(); // consume the escaped character
                    result.push(escaped);
                } else {
                    result.push('\\');
                }
            }
            '%' => {
                // Handle format specifiers
                if let Some(&next) = chars.peek() {
                    match next {
                        '%' => {
                            chars.next();
                            result.push('%');
                        }
                        's' => {
                            chars.next();
                            if arg_index < args.len() {
                                result.push_str(&args[arg_index]);
                                arg_index += 1;
                            }
                        }
                        'd' | 'i' => {
                            chars.next();
                            if arg_index < args.len() {
                                // Try to parse as integer, default to 0
                                let num: i64 = args[arg_index].parse().unwrap_or(0);
                                result.push_str(&num.to_string());
                                arg_index += 1;
                            } else {
                                result.push('0');
                            }
                        }
                        'c' => {
                            chars.next();
                            if arg_index < args.len() && !args[arg_index].is_empty() {
                                result.push(args[arg_index].chars().next().unwrap());
                                arg_index += 1;
                            }
                        }
                        'x' => {
                            chars.next();
                            if arg_index < args.len() {
                                let num: i64 = args[arg_index].parse().unwrap_or(0);
                                result.push_str(&format!("{:x}", num));
                                arg_index += 1;
                            } else {
                                result.push('0');
                            }
                        }
                        'X' => {
                            chars.next();
                            if arg_index < args.len() {
                                let num: i64 = args[arg_index].parse().unwrap_or(0);
                                result.push_str(&format!("{:X}", num));
                                arg_index += 1;
                            } else {
                                result.push('0');
                            }
                        }
                        'o' => {
                            chars.next();
                            if arg_index < args.len() {
                                let num: i64 = args[arg_index].parse().unwrap_or(0);
                                result.push_str(&format!("{:o}", num));
                                arg_index += 1;
                            } else {
                                result.push('0');
                            }
                        }
                        _ => {
                            // Unknown specifier, output as-is
                            result.push('%');
                        }
                    }
                } else {
                    result.push('%');
                }
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_printf_no_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_printf_simple_string() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &["hello".to_string()]);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_format_string_literal() {
        let result = format_string("hello", &[]);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_format_string_s() {
        let result = format_string("%s", &["world".to_string()]);
        assert_eq!(result, "world");
    }

    #[test]
    fn test_format_string_multiple_s() {
        let result = format_string("%s %s", &["hello".to_string(), "world".to_string()]);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_format_string_d() {
        let result = format_string("%d", &["42".to_string()]);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_format_string_d_invalid() {
        let result = format_string("%d", &["abc".to_string()]);
        assert_eq!(result, "0"); // Invalid integer defaults to 0
    }

    #[test]
    fn test_format_string_c() {
        let result = format_string("%c", &["hello".to_string()]);
        assert_eq!(result, "h"); // First character
    }

    #[test]
    fn test_format_string_percent() {
        let result = format_string("100%%", &[]);
        assert_eq!(result, "100%");
    }

    #[test]
    fn test_format_string_newline() {
        let result = format_string("hello\\nworld", &[]);
        assert_eq!(result, "hello\nworld");
    }

    #[test]
    fn test_format_string_tab() {
        let result = format_string("hello\\tworld", &[]);
        assert_eq!(result, "hello\tworld");
    }

    #[test]
    fn test_format_string_backslash() {
        let result = format_string("hello\\\\world", &[]);
        assert_eq!(result, "hello\\world");
    }

    #[test]
    fn test_format_string_mixed() {
        let result = format_string("%s: %d\\n", &["count".to_string(), "42".to_string()]);
        assert_eq!(result, "count: 42\n");
    }

    #[test]
    fn test_format_string_missing_args() {
        let result = format_string("%s %s", &["only_one".to_string()]);
        assert_eq!(result, "only_one "); // Missing arg is empty
    }

    #[test]
    fn test_format_string_hex() {
        let result = format_string("%x", &["255".to_string()]);
        assert_eq!(result, "ff");
    }

    #[test]
    fn test_format_string_hex_upper() {
        let result = format_string("%X", &["255".to_string()]);
        assert_eq!(result, "FF");
    }

    #[test]
    fn test_format_string_octal() {
        let result = format_string("%o", &["8".to_string()]);
        assert_eq!(result, "10");
    }
}
