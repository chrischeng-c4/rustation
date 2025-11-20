//! Command line parser
//!
//! Parses command lines into program and arguments, handling:
//! - Quoted strings (single and double quotes)
//! - Escaped characters
//! - Whitespace splitting
//! - Redirection operators (>, >>, <)

use crate::error::{Result, RushError};

/// Token types for parsing command lines with redirections
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// Regular word (command, argument, file path)
    Word(String),
    /// Output redirection (>)
    RedirectOut,
    /// Append redirection (>>)
    RedirectAppend,
    /// Input redirection (<)
    RedirectIn,
}

/// Parse a command line into program, arguments, and redirections
///
/// Returns (program, args, redirections) tuple
pub fn parse_command_with_redirections(line: &str) -> Result<(String, Vec<String>, Vec<super::Redirection>)> {
    use super::{Redirection, RedirectionType};

    let tokens = tokenize_with_redirections(line)?;

    if tokens.is_empty() {
        return Err(RushError::Execution("Empty command".to_string()));
    }

    let mut words = Vec::new();
    let mut redirections = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Word(w) => {
                words.push(w.clone());
                i += 1;
            }
            Token::RedirectOut | Token::RedirectAppend | Token::RedirectIn => {
                // Redirection operator must be followed by a file path
                if i + 1 >= tokens.len() {
                    return Err(RushError::Execution(
                        "Redirection operator missing file path".to_string()
                    ));
                }

                // Next token must be a Word (file path)
                if let Token::Word(path) = &tokens[i + 1] {
                    let redir_type = match &tokens[i] {
                        Token::RedirectOut => RedirectionType::Output,
                        Token::RedirectAppend => RedirectionType::Append,
                        Token::RedirectIn => RedirectionType::Input,
                        _ => unreachable!(),
                    };
                    redirections.push(Redirection::new(redir_type, path.clone()));
                    i += 2; // Skip operator and path
                } else {
                    return Err(RushError::Execution(
                        "Redirection operator must be followed by file path".to_string()
                    ));
                }
            }
        }
    }

    if words.is_empty() {
        return Err(RushError::Execution("Empty command".to_string()));
    }

    let program = words[0].clone();
    let args = words[1..].to_vec();

    tracing::trace!(
        program = %program,
        args = ?args,
        redirections = ?redirections,
        "Parsed command with redirections"
    );

    Ok((program, args, redirections))
}

/// Parse a command line into program and arguments
///
/// Handles quoted strings and basic escaping.
/// Legacy function - use parse_command_with_redirections for redirection support
///
/// # Examples
///
/// ```ignore
/// let (program, args) = parse_command_line("echo hello")?;
/// assert_eq!(program, "echo");
/// assert_eq!(args, vec!["hello"]);
///
/// let (program, args) = parse_command_line("echo \"hello world\"")?;
/// assert_eq!(program, "echo");
/// assert_eq!(args, vec!["hello world"]);
/// ```
pub fn parse_command_line(line: &str) -> Result<(String, Vec<String>)> {
    let tokens = tokenize(line)?;

    if tokens.is_empty() {
        return Err(RushError::Execution("Empty command".to_string()));
    }

    let program = tokens[0].clone();
    let args = tokens[1..].to_vec();

    tracing::trace!(
        program = %program,
        args = ?args,
        "Parsed command line"
    );

    Ok((program, args))
}

/// Tokenize a command line with redirection operator support
/// Returns tokens including redirection operators as separate tokens
fn tokenize_with_redirections(line: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;
    let mut had_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if escape_next {
            // Escaped character - add literally
            current_token.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                // Backslash escapes next character
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                // Single quote - toggle single quote mode
                if in_single_quote {
                    had_quotes = true;
                }
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                // Double quote - toggle double quote mode
                if in_double_quote {
                    had_quotes = true;
                }
                in_double_quote = !in_double_quote;
            }
            '>' if !in_single_quote && !in_double_quote => {
                // Check for >> (append) operator
                if !current_token.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_token.clone()));
                    current_token.clear();
                    had_quotes = false;
                }

                if chars.peek() == Some(&'>') {
                    chars.next(); // Consume second >
                    tokens.push(Token::RedirectAppend);
                } else {
                    tokens.push(Token::RedirectOut);
                }
            }
            '<' if !in_single_quote && !in_double_quote => {
                // Input redirection operator
                if !current_token.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_token.clone()));
                    current_token.clear();
                    had_quotes = false;
                }
                tokens.push(Token::RedirectIn);
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                // Whitespace outside quotes - end current token
                if !current_token.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_token.clone()));
                    current_token.clear();
                    had_quotes = false;
                }
            }
            _ => {
                // Regular character - add to current token
                current_token.push(ch);
            }
        }
    }

    // Check for unclosed quotes
    if in_single_quote {
        return Err(RushError::Execution("Unclosed single quote in command".to_string()));
    }
    if in_double_quote {
        return Err(RushError::Execution("Unclosed double quote in command".to_string()));
    }
    if escape_next {
        return Err(RushError::Execution("Trailing backslash in command".to_string()));
    }

    // Add final token if non-empty or was quoted empty string
    if !current_token.is_empty() || had_quotes {
        tokens.push(Token::Word(current_token));
    }

    Ok(tokens)
}

/// Tokenize a command line, respecting quotes and escapes
/// Legacy function for backward compatibility
fn tokenize(line: &str) -> Result<Vec<String>> {
    let tokens = tokenize_with_redirections(line)?;
    // Convert Token::Word to String, filter out redirection operators
    Ok(tokens.into_iter().filter_map(|t| {
        if let Token::Word(s) = t {
            Some(s)
        } else {
            None
        }
    }).collect())
}

/// Original tokenize implementation kept for reference
fn _tokenize_original(line: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;
    let mut had_quotes = false; // Track if current token was quoted
    let chars = line.chars();

    for ch in chars {
        if escape_next {
            // Escaped character - add literally
            current_token.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                // Backslash escapes next character
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                // Single quote - toggle single quote mode
                if in_single_quote {
                    // Closing quote - mark that we had quotes
                    had_quotes = true;
                }
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                // Double quote - toggle double quote mode
                if in_double_quote {
                    // Closing quote - mark that we had quotes
                    had_quotes = true;
                }
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                // Whitespace outside quotes - end current token
                if !current_token.is_empty() || had_quotes {
                    // Push token if non-empty OR if it was a quoted empty string
                    tokens.push(current_token.clone());
                    current_token.clear();
                    had_quotes = false;
                }
            }
            _ => {
                // Regular character - add to current token
                current_token.push(ch);
            }
        }
    }

    // Check for unclosed quotes
    if in_single_quote {
        return Err(RushError::Execution("Unclosed single quote in command".to_string()));
    }
    if in_double_quote {
        return Err(RushError::Execution("Unclosed double quote in command".to_string()));
    }
    if escape_next {
        return Err(RushError::Execution("Trailing backslash in command".to_string()));
    }

    // Add final token if non-empty or was quoted empty string
    if !current_token.is_empty() || had_quotes {
        tokens.push(current_token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::RedirectionType;

    #[test]
    fn test_parse_simple_command() {
        let (program, args) = parse_command_line("echo hello").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello"]);
    }

    #[test]
    fn test_parse_command_with_multiple_args() {
        let (program, args) = parse_command_line("ls -la /tmp").unwrap();
        assert_eq!(program, "ls");
        assert_eq!(args, vec!["-la", "/tmp"]);
    }

    #[test]
    fn test_parse_command_with_double_quotes() {
        let (program, args) = parse_command_line("echo \"hello world\"").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_command_with_single_quotes() {
        let (program, args) = parse_command_line("echo 'hello world'").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_command_with_mixed_quotes() {
        let (program, args) = parse_command_line("echo \"hello\" 'world'").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello", "world"]);
    }

    #[test]
    fn test_parse_command_with_escaped_chars() {
        let (program, args) = parse_command_line("echo hello\\ world").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_command_with_escaped_quote() {
        let (program, args) = parse_command_line("echo \\\"hello\\\"").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["\"hello\""]);
    }

    #[test]
    fn test_parse_empty_quotes() {
        let (program, args) = parse_command_line("echo \"\"").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec![""]);
    }

    #[test]
    fn test_parse_multiple_spaces() {
        let (program, args) = parse_command_line("echo    hello    world").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello", "world"]);
    }

    #[test]
    fn test_parse_tabs() {
        let (program, args) = parse_command_line("echo\thello\tworld").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello", "world"]);
    }

    #[test]
    fn test_parse_unclosed_double_quote() {
        let result = parse_command_line("echo \"hello");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unclosed double quote"));
    }

    #[test]
    fn test_parse_unclosed_single_quote() {
        let result = parse_command_line("echo 'hello");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unclosed single quote"));
    }

    #[test]
    fn test_parse_trailing_backslash() {
        let result = parse_command_line("echo hello\\");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Trailing backslash"));
    }

    #[test]
    fn test_parse_quotes_in_quotes() {
        let (program, args) = parse_command_line("echo \"it's fine\"").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["it's fine"]);
    }

    #[test]
    fn test_parse_complex_command() {
        let (program, args) =
            parse_command_line("git commit -m \"fix: improve parser\" --verbose").unwrap();
        assert_eq!(program, "git");
        assert_eq!(args, vec!["commit", "-m", "fix: improve parser", "--verbose"]);
    }

    #[test]
    fn test_tokenize_empty_string() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens, Vec::<String>::new());
    }

    #[test]
    fn test_tokenize_whitespace_only() {
        let tokens = tokenize("   ").unwrap();
        assert_eq!(tokens, Vec::<String>::new());
    }

    // Redirection tokenization tests
    #[test]
    fn test_tokenize_with_output_redirect() {
        let tokens = tokenize_with_redirections("echo hello > file.txt").unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word("hello".to_string()));
        assert_eq!(tokens[2], Token::RedirectOut);
        assert_eq!(tokens[3], Token::Word("file.txt".to_string()));
    }

    #[test]
    fn test_tokenize_with_append_redirect() {
        let tokens = tokenize_with_redirections("echo test >> log.txt").unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word("test".to_string()));
        assert_eq!(tokens[2], Token::RedirectAppend);
        assert_eq!(tokens[3], Token::Word("log.txt".to_string()));
    }

    #[test]
    fn test_tokenize_append_not_two_outputs() {
        let tokens = tokenize_with_redirections("cmd >> file").unwrap();
        // Should be RedirectAppend, not two RedirectOut
        assert!(matches!(tokens[1], Token::RedirectAppend));
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_tokenize_with_input_redirect() {
        let tokens = tokenize_with_redirections("sort < input.txt").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Word("sort".to_string()));
        assert_eq!(tokens[1], Token::RedirectIn);
        assert_eq!(tokens[2], Token::Word("input.txt".to_string()));
    }

    #[test]
    fn test_tokenize_operators_in_quotes_are_words() {
        let tokens = tokenize_with_redirections("echo \"a > b\"").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word("a > b".to_string()));
    }

    #[test]
    fn test_tokenize_whitespace_around_operators() {
        let tokens1 = tokenize_with_redirections("echo test >file.txt").unwrap();
        let tokens2 = tokenize_with_redirections("echo test > file.txt").unwrap();
        let tokens3 = tokenize_with_redirections("echo test >  file.txt").unwrap();

        // All should parse the same way
        assert_eq!(tokens1[2], Token::RedirectOut);
        assert_eq!(tokens2[2], Token::RedirectOut);
        assert_eq!(tokens3[2], Token::RedirectOut);
    }

    // parse_command_with_redirections tests
    #[test]
    fn test_parse_with_output_redirection() {
        let (program, args, redirs) = parse_command_with_redirections("ls -la > files.txt").unwrap();
        assert_eq!(program, "ls");
        assert_eq!(args, vec!["-la"]);
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::Output);
        assert_eq!(redirs[0].file_path, "files.txt");
    }

    #[test]
    fn test_parse_with_append_redirection() {
        let (program, args, redirs) = parse_command_with_redirections("echo line >> log.txt").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["line"]);
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::Append);
        assert_eq!(redirs[0].file_path, "log.txt");
    }

    #[test]
    fn test_parse_with_input_redirection() {
        let (program, args, redirs) = parse_command_with_redirections("wc -l < input.txt").unwrap();
        assert_eq!(program, "wc");
        assert_eq!(args, vec!["-l"]);
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::Input);
        assert_eq!(redirs[0].file_path, "input.txt");
    }

    #[test]
    fn test_parse_no_redirections() {
        let (program, args, redirs) = parse_command_with_redirections("echo hello").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello"]);
        assert_eq!(redirs.len(), 0);
    }

    #[test]
    fn test_parse_redirect_missing_path() {
        let result = parse_command_with_redirections("echo test >");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing file path"));
    }

    #[test]
    fn test_parse_quoted_operators_literal() {
        let (program, args, redirs) = parse_command_with_redirections("echo \"test > file\"").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["test > file"]);
        assert_eq!(redirs.len(), 0); // No redirections, it's in quotes
    }
}
