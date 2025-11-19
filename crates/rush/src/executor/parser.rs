//! Command line parser
//!
//! Parses command lines into program and arguments, handling:
//! - Quoted strings (single and double quotes)
//! - Escaped characters
//! - Whitespace splitting
//! - Pipe operators (`|`) for command composition
//!
//! The parser uses a two-stage approach:
//! 1. Tokenization: Split input into Word and Pipe tokens, respecting quotes
//! 2. Segmentation: Group tokens into pipeline segments at Pipe boundaries
//!
//! Pipes inside quotes are treated as literal text, not operators.

use crate::error::{Result, RushError};
use crate::executor::{Pipeline, PipelineSegment};

/// Token types after parsing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// Regular word (command, argument)
    Word(String),
    /// Pipe operator
    Pipe,
}

/// Parse a command line into program and arguments
///
/// Handles quoted strings and basic escaping.
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

/// Tokenize a command line, respecting quotes and escapes
fn tokenize(line: &str) -> Result<Vec<String>> {
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

/// Parse command line into pipeline
///
/// Converts a command line string into a `Pipeline` containing one or more
/// `PipelineSegment`s. Pipes (`|`) outside quotes split the line into segments.
///
/// # Examples
///
/// ```ignore
/// use rush::executor::parser::parse_pipeline;
///
/// // Single command
/// let pipeline = parse_pipeline("ls")?;
/// assert_eq!(pipeline.len(), 1);
///
/// // Two commands with pipe
/// let pipeline = parse_pipeline("ls | grep txt")?;
/// assert_eq!(pipeline.len(), 2);
///
/// // Pipe inside quotes (literal, not operator)
/// let pipeline = parse_pipeline("echo \"a | b\"")?;
/// assert_eq!(pipeline.len(), 1);
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - Empty command before or after pipe (`| cmd` or `cmd |`)
/// - Unclosed quotes
/// - Empty input after trimming
pub fn parse_pipeline(line: &str) -> Result<Pipeline> {
    // Tokenize with pipe detection
    let tokens = tokenize_with_pipes(line)?;

    // Split tokens at pipe boundaries
    let segments = split_into_segments(tokens)?;

    // Build pipeline
    let pipeline = Pipeline::new(segments, line.to_string());

    // Validate
    pipeline.validate()?;

    Ok(pipeline)
}

/// Tokenize command line, recognizing pipes as special tokens
///
/// Splits input into `Word` and `Pipe` tokens. Pipes inside quotes are
/// treated as literal text and become part of Word tokens.
///
/// # Algorithm
///
/// 1. Scan character by character, tracking quote state
/// 2. When `|` found outside quotes → emit Pipe token
/// 3. When whitespace found outside quotes → end current word
/// 4. Otherwise → accumulate into current word
fn tokenize_with_pipes(line: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;
    let mut had_quotes = false;

    for ch in line.chars() {
        if escape_next {
            current_word.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => escape_next = true,
            '\'' if !in_double_quote => {
                if in_single_quote {
                    had_quotes = true;
                }
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                if in_double_quote {
                    had_quotes = true;
                }
                in_double_quote = !in_double_quote;
            }
            '|' if !in_single_quote && !in_double_quote => {
                // Pipe outside quotes - emit current word and Pipe token
                if !current_word.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    had_quotes = false;
                }
                tokens.push(Token::Pipe);
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                // Whitespace outside quotes - end word
                if !current_word.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    had_quotes = false;
                }
            }
            _ => {
                current_word.push(ch);
            }
        }
    }

    // Validation
    if in_single_quote {
        return Err(RushError::Execution("Unclosed single quote".to_string()));
    }
    if in_double_quote {
        return Err(RushError::Execution("Unclosed double quote".to_string()));
    }
    if escape_next {
        return Err(RushError::Execution("Trailing backslash".to_string()));
    }

    // Emit final word
    if !current_word.is_empty() || had_quotes {
        tokens.push(Token::Word(current_word));
    }

    Ok(tokens)
}

/// Split tokens into pipeline segments at Pipe boundaries
///
/// Takes a flat list of tokens and groups them into segments, splitting at
/// each Pipe token. Each segment becomes one command in the pipeline.
///
/// # Validation
///
/// - Empty segment before pipe → Error: "Empty command before pipe"
/// - Empty segment after pipe → Error: "Empty command after pipe"
/// - No program in segment → Error: "Empty command"
///
/// # Returns
///
/// Vector of `PipelineSegment`s with assigned indices (0, 1, 2, ...)
fn split_into_segments(tokens: Vec<Token>) -> Result<Vec<PipelineSegment>> {
    let mut segments = Vec::new();
    let mut current_segment: Vec<String> = Vec::new();

    for token in tokens {
        match token {
            Token::Word(word) => {
                current_segment.push(word);
            }
            Token::Pipe => {
                // Validate segment before pipe
                if current_segment.is_empty() {
                    return Err(RushError::Execution("Empty command before pipe".to_string()));
                }

                // Create segment
                let program = current_segment[0].clone();
                let args = current_segment[1..].to_vec();
                segments.push(PipelineSegment::new(program, args, segments.len()));

                current_segment.clear();
            }
        }
    }

    // Validate final segment
    if current_segment.is_empty() {
        if !segments.is_empty() {
            // Had pipes but no command after last pipe
            return Err(RushError::Execution("Empty command after pipe".to_string()));
        }
        // No commands at all
        return Err(RushError::Execution("Empty command".to_string()));
    }

    // Add final segment
    let program = current_segment[0].clone();
    let args = current_segment[1..].to_vec();
    segments.push(PipelineSegment::new(program, args, segments.len()));

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
