//! Variable expansion for environment variables
//!
//! Handles expansion of:
//! - $VAR and ${VAR} - Variable references
//! - ${arr[0]} - Array element access
//! - ${arr[@]} - All array elements as separate words
//! - ${arr[*]} - All array elements as one word
//! - $$ - Shell process ID
//! - $? - Last exit code
//! - $0 - Shell name ("rush")
//! - $# - Number of positional arguments
//! - Escape sequences (\$ -> literal $)

use crate::executor::arrays::{parse_array_ref, ArrayRefType};
use crate::executor::execute::CommandExecutor;

/// Expand variables in a command line string
///
/// Performs variable substitution before command parsing:
/// - $VARNAME or ${VARNAME} -> variable value
/// - $$ -> process ID
/// - $? -> last exit code
/// - $0 -> "rush"
/// - $# -> 0 (no positional args in interactive shell)
/// - Non-existent variables -> empty string
/// - Escape \$ -> literal $
///
/// # Arguments
/// * `input` - Input string with potential variable references
/// * `executor` - CommandExecutor with variable manager and exit code
///
/// # Returns
/// Expanded string with all variables substituted
pub fn expand_variables(input: &str, executor: &CommandExecutor) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' if chars.peek() == Some(&'$') => {
                // Escaped $ - output literal $
                chars.next(); // consume the $
                result.push('$');
            }
            '$' => {
                // Variable or special reference
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        '$' => {
                            // $$ -> process ID
                            chars.next();
                            result.push_str(&std::process::id().to_string());
                        }
                        '?' => {
                            // $? -> last exit code
                            chars.next();
                            result.push_str(&executor.last_exit_code().to_string());
                        }
                        '0' => {
                            // $0 -> shell name
                            chars.next();
                            result.push_str("rush");
                        }
                        '#' => {
                            // $# -> number of positional args (0 for interactive)
                            chars.next();
                            result.push('0');
                        }
                        '1'..='9' => {
                            // $N -> positional argument (not used in interactive shell)
                            chars.next();
                            // Positional args not supported in interactive shell
                            // Just skip the digit
                        }
                        '{' => {
                            // ${VARNAME} or ${arr[...]} - extract until }
                            chars.next(); // consume {
                            let var_name = extract_until(&mut chars, '}');
                            if !var_name.is_empty() {
                                // Check if this is an array reference
                                if var_name.contains('[') {
                                    // Try to parse as array reference: ${arr[0]}, ${arr[@]}, ${arr[*]}
                                    let array_expr = format!("${{{}}}", var_name);
                                    if let Ok(arr_ref) = parse_array_ref(&array_expr) {
                                        match arr_ref.ref_type {
                                            ArrayRefType::Index(idx) => {
                                                // ${arr[0]} - single element
                                                if let Some(value) = executor.variable_manager().array_get(&arr_ref.name, idx) {
                                                    result.push_str(value);
                                                }
                                                // Out of bounds or non-existent array -> empty string
                                            }
                                            ArrayRefType::AllWords => {
                                                // ${arr[@]} - all elements as space-separated words
                                                if let Some(arr) = executor.variable_manager().get_array(&arr_ref.name) {
                                                    result.push_str(&arr.join(" "));
                                                }
                                            }
                                            ArrayRefType::AllAsOne => {
                                                // ${arr[*]} - all elements as one word
                                                if let Some(arr) = executor.variable_manager().get_array(&arr_ref.name) {
                                                    result.push_str(&arr.join(" "));
                                                }
                                            }
                                        }
                                    }
                                    // Invalid array syntax -> empty string (silently)
                                } else if let Some(value) = executor.variable_manager().get(&var_name) {
                                    result.push_str(value);
                                }
                                // Non-existent variables expand to empty string
                            }
                        }
                        'a'..='z' | 'A'..='Z' | '_' => {
                            // $VARNAME - extract identifier
                            let var_name = extract_identifier(&mut chars);
                            if let Some(value) = executor.variable_manager().get(&var_name) {
                                result.push_str(value);
                            }
                            // Non-existent variables expand to empty string
                        }
                        _ => {
                            // Not a valid variable reference, output $ literally
                            result.push('$');
                        }
                    }
                } else {
                    // $ at end of string
                    result.push('$');
                }
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

/// Extract characters until a specific delimiter is found
fn extract_until(chars: &mut std::iter::Peekable<std::str::Chars>, delimiter: char) -> String {
    let mut result = String::new();
    while let Some(&ch) = chars.peek() {
        if ch == delimiter {
            chars.next(); // consume delimiter
            break;
        }
        result.push(ch);
        chars.next();
    }
    result
}

/// Extract an identifier (variable name) starting from current position
///
/// An identifier is alphanumeric or underscore characters
fn extract_identifier(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut result = String::new();

    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            result.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_no_expansion() {
        let executor = CommandExecutor::new();
        let input = "echo hello world";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo hello world");
    }

    #[test]
    fn test_simple_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("greeting".to_string(), "hello".to_string())
            .unwrap();

        let input = "echo $greeting";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn test_variable_with_braces() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("name".to_string(), "world".to_string())
            .unwrap();

        let input = "echo ${name}!";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo world!");
    }

    #[test]
    fn test_multiple_variables() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("first".to_string(), "hello".to_string())
            .unwrap();
        executor
            .variable_manager_mut()
            .set("second".to_string(), "world".to_string())
            .unwrap();

        let input = "echo $first $second";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo hello world");
    }

    #[test]
    fn test_nonexistent_variable() {
        let executor = CommandExecutor::new();
        let input = "echo $nonexistent";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo ");
    }

    #[test]
    fn test_escaped_dollar() {
        let executor = CommandExecutor::new();
        let input = "echo \\$var";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo $var");
    }

    #[test]
    fn test_special_variable_pid() {
        let executor = CommandExecutor::new();
        let input = "echo $$";
        let result = expand_variables(input, &executor);
        // Result should contain a number (PID)
        assert!(result.starts_with("echo "));
        let pid_str = &result[5..];
        assert!(!pid_str.is_empty());
        assert!(pid_str.chars().all(|c| c.is_numeric()));
    }

    #[test]
    fn test_special_variable_exit_code() {
        let mut executor = CommandExecutor::new();
        executor.set_last_exit_code(42);

        let input = "echo $?";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo 42");
    }

    #[test]
    fn test_special_variable_shell_name() {
        let executor = CommandExecutor::new();
        let input = "echo $0";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo rush");
    }

    #[test]
    fn test_special_variable_arg_count() {
        let executor = CommandExecutor::new();
        let input = "echo $#";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo 0");
    }

    #[test]
    fn test_dollar_at_end() {
        let executor = CommandExecutor::new();
        let input = "echo $";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo $");
    }

    #[test]
    fn test_variable_adjacent_text() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("VAR".to_string(), "value".to_string())
            .unwrap();

        // $VARmore tries to expand variable "VARmore" (doesn't exist)
        let input = "test$VARmore";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "test");

        // ${VAR}more expands VAR and appends "more"
        let input2 = "test${VAR}more";
        let result2 = expand_variables(input2, &executor);
        assert_eq!(result2, "testvaluemore");
    }

    #[test]
    fn test_variable_with_underscore() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("MY_VAR".to_string(), "test".to_string())
            .unwrap();

        let input = "echo $MY_VAR";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo test");
    }

    #[test]
    fn test_unclosed_braces() {
        let executor = CommandExecutor::new();
        let input = "echo ${incomplete";
        let result = expand_variables(input, &executor);
        // Should output the incomplete reference as-is (since no closing })
        assert_eq!(result, "echo ");
    }

    #[test]
    fn test_empty_variable_name() {
        let executor = CommandExecutor::new();
        let input = "echo ${}";
        let result = expand_variables(input, &executor);
        // Empty variable name should expand to nothing
        assert_eq!(result, "echo ");
    }

    // ===== Array Expansion Tests =====

    #[test]
    fn test_array_index_zero() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec!["first".to_string(), "second".to_string(), "third".to_string()])
            .unwrap();

        let input = "echo ${arr[0]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo first");
    }

    #[test]
    fn test_array_index_nonzero() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string()])
            .unwrap();

        let input = "echo ${arr[2]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo c");
    }

    #[test]
    fn test_array_index_out_of_bounds() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec!["only".to_string()])
            .unwrap();

        let input = "echo ${arr[99]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo ");
    }

    #[test]
    fn test_array_all_words() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec!["one".to_string(), "two".to_string(), "three".to_string()])
            .unwrap();

        let input = "echo ${arr[@]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo one two three");
    }

    #[test]
    fn test_array_all_as_one() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec!["one".to_string(), "two".to_string(), "three".to_string()])
            .unwrap();

        let input = "echo ${arr[*]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo one two three");
    }

    #[test]
    fn test_array_nonexistent() {
        let executor = CommandExecutor::new();

        let input = "echo ${nonexistent[@]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo ");
    }

    #[test]
    fn test_array_empty() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec![])
            .unwrap();

        let input = "echo ${arr[@]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo ");
    }

    #[test]
    fn test_array_single_element() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec!["single".to_string()])
            .unwrap();

        let input = "echo ${arr[@]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo single");
    }

    #[test]
    fn test_array_mixed_with_regular_vars() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("prefix".to_string(), "PREFIX".to_string())
            .unwrap();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec!["a".to_string(), "b".to_string()])
            .unwrap();
        executor
            .variable_manager_mut()
            .set("suffix".to_string(), "SUFFIX".to_string())
            .unwrap();

        let input = "${prefix} ${arr[@]} ${suffix}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "PREFIX a b SUFFIX");
    }

    #[test]
    fn test_array_with_spaces_in_elements() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec!["hello world".to_string(), "foo bar".to_string()])
            .unwrap();

        let input = "echo ${arr[@]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "echo hello world foo bar");
    }

    #[test]
    fn test_multiple_array_refs() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set_array("arr".to_string(), vec!["x".to_string(), "y".to_string(), "z".to_string()])
            .unwrap();

        let input = "${arr[0]}+${arr[1]}+${arr[2]}";
        let result = expand_variables(input, &executor);
        assert_eq!(result, "x+y+z");
    }
}
