//! Command line parser
//!
//! Parses command lines into program and arguments, handling:
//! - Quoted strings (single and double quotes)
//! - Escaped characters
//! - Whitespace splitting
//! - Pipe operators (`|`) for command composition
//! - Redirection operators (>, >>, <)
//!
//! # User Story 1: Basic Two-Command Pipeline
//!
//! The parser uses a two-stage approach for pipelines:
//! 1. Tokenization: Split input into Word, Pipe, and Redirection tokens, respecting quotes
//! 2. Segmentation: Group tokens into pipeline segments at Pipe boundaries
//!
//! Pipes and redirections inside quotes are treated as literal text, not operators.

use crate::error::{Result, RushError};
use crate::executor::{Pipeline, PipelineSegment};

/// Token types for parsing command lines with pipes and redirections
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// Regular word (command, argument, file path)
    Word(String),
    /// Pipe operator (|)
    Pipe,
    /// Output redirection (>)
    RedirectOut,
    /// Append redirection (>>)
    RedirectAppend,
    /// Input redirection (<)
    RedirectIn,
    /// Stderr redirection (2>)
    StderrOut,
    /// Stderr append redirection (2>>)
    StderrAppend,
    /// Stderr to stdout (2>&1)
    StderrToStdout,
    /// Stdout to stderr (1>&2)
    StdoutToStderr,
    /// Heredoc (<<)
    Heredoc,
    /// Heredoc with tab stripping (<<-)
    HeredocStrip,
    /// Here-string (<<<)
    HereString,
    /// Background execution (&)
    Background,
    /// Control flow keyword (if, then, elif, else, fi)
    Keyword(super::Keyword),
    /// Newline separator
    Newline,
    /// End of input
    Eof,
}

/// Check if a word is a control flow keyword and return the token
/// Keywords are only recognized at the start of a command (after pipes, operators, etc.)
fn tokenize_keyword(word: &str) -> Token {
    if let Some(kw) = super::Keyword::from_str(word) {
        Token::Keyword(kw)
    } else {
        Token::Word(word.to_string())
    }
}

/// Expect a specific keyword token and return an error if it doesn't match
/// Used in parsing conditionals to validate expected keywords
pub fn expect_keyword(tokens: &[Token], index: usize, expected: super::Keyword) -> Result<usize> {
    if index >= tokens.len() {
        return Err(RushError::Syntax(format!(
            "Expected keyword '{}', but reached end of input",
            expected.as_str()
        )));
    }

    match &tokens[index] {
        Token::Keyword(kw) if *kw == expected => Ok(index + 1),
        Token::Keyword(kw) => Err(RushError::Syntax(format!(
            "Expected keyword '{}', found '{}'",
            expected.as_str(),
            kw.as_str()
        ))),
        _ => Err(RushError::Syntax(format!(
            "Expected keyword '{}', found something else",
            expected.as_str()
        ))),
    }
}

/// Parse a command line into program, arguments, and redirections
///
/// Returns (program, args, redirections) tuple
pub fn parse_command_with_redirections(
    line: &str,
) -> Result<(String, Vec<String>, Vec<super::Redirection>)> {
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
            Token::Pipe => {
                // Pipes are not supported in this legacy function
                // Use parse_pipeline() instead
                return Err(RushError::Execution(
                    "Pipe operator not supported - use parse_pipeline() instead".to_string(),
                ));
            }
            Token::RedirectOut
            | Token::RedirectAppend
            | Token::RedirectIn
            | Token::StderrOut
            | Token::StderrAppend => {
                // Redirection operator must be followed by a file path
                if i + 1 >= tokens.len() {
                    return Err(RushError::Execution(
                        "Redirection operator missing file path".to_string(),
                    ));
                }

                // Next token must be a Word (file path)
                if let Token::Word(path) = &tokens[i + 1] {
                    let redir_type = match &tokens[i] {
                        Token::RedirectOut => RedirectionType::Output,
                        Token::RedirectAppend => RedirectionType::Append,
                        Token::RedirectIn => RedirectionType::Input,
                        Token::StderrOut => RedirectionType::Stderr(false),
                        Token::StderrAppend => RedirectionType::Stderr(true),
                        _ => unreachable!(),
                    };
                    redirections.push(Redirection::new(redir_type, path.clone()));
                    i += 2; // Skip operator and path
                } else {
                    return Err(RushError::Execution(
                        "Redirection operator must be followed by file path".to_string(),
                    ));
                }
            }
            Token::Background => {
                return Err(RushError::Execution(
                    "Background operator not supported in this context".to_string(),
                ));
            }
            Token::StderrToStdout => {
                redirections.push(Redirection::new(RedirectionType::StderrToStdout, String::new()));
                i += 1;
            }
            Token::StdoutToStderr => {
                redirections.push(Redirection::new(RedirectionType::StdoutToStderr, String::new()));
                i += 1;
            }
            Token::Heredoc | Token::HeredocStrip => {
                // Heredoc operator must be followed by a delimiter
                if i + 1 >= tokens.len() {
                    return Err(RushError::Execution(
                        "Heredoc operator requires a delimiter".to_string(),
                    ));
                }

                if let Token::Word(delimiter) = &tokens[i + 1] {
                    let strip_tabs = matches!(&tokens[i], Token::HeredocStrip);
                    // Note: heredoc content is not collected here - this legacy function
                    // doesn't support heredocs. Use parse_with_heredocs() instead.
                    redirections.push(Redirection::new_heredoc(
                        delimiter.clone(),
                        String::new(),
                        strip_tabs,
                    ));
                    i += 2; // Skip operator and delimiter
                } else {
                    return Err(RushError::Execution(
                        "Heredoc operator must be followed by delimiter".to_string(),
                    ));
                }
            }
            Token::HereString => {
                // Here-string operator must be followed by a word (the string content)
                if i + 1 >= tokens.len() {
                    return Err(RushError::Execution(
                        "Here-string operator requires content".to_string(),
                    ));
                }

                if let Token::Word(content) = &tokens[i + 1] {
                    redirections.push(Redirection::new_herestring(content.clone()));
                    i += 2; // Skip operator and content
                } else {
                    return Err(RushError::Execution(
                        "Here-string operator must be followed by content".to_string(),
                    ));
                }
            }
            Token::Keyword(kw) => {
                return Err(RushError::Syntax(format!(
                    "Unexpected keyword '{}' in command",
                    kw.as_str()
                )));
            }
            Token::Newline | Token::Eof => {
                // End of input, return what we have so far
                break;
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

/// Extract redirections from a list of arguments (used for pipeline segments)
///
/// Takes a list of arguments that may contain redirection operators as strings
/// (>, >>, <) and extracts them, returning clean args and redirections.
///
/// # Arguments
/// * `args` - Arguments that may contain redirection operators as strings
///
/// # Returns
/// * `(clean_args, redirections)` - Args without redirection operators, and list of redirections
pub fn extract_redirections_from_args(
    args: &[String],
) -> Result<(Vec<String>, Vec<super::Redirection>)> {
    use super::{Redirection, RedirectionType};

    let mut clean_args = Vec::new();
    let mut redirections = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            ">" => {
                // Output redirection - next arg is the file path
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Redirection operator missing file path".to_string(),
                    ));
                }
                redirections.push(Redirection::new(RedirectionType::Output, args[i + 1].clone()));
                i += 2; // Skip operator and path
            }
            ">>" => {
                // Append redirection - next arg is the file path
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Redirection operator missing file path".to_string(),
                    ));
                }
                redirections.push(Redirection::new(RedirectionType::Append, args[i + 1].clone()));
                i += 2; // Skip operator and path
            }
            "<" => {
                // Input redirection - next arg is the file path
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Redirection operator missing file path".to_string(),
                    ));
                }
                redirections.push(Redirection::new(RedirectionType::Input, args[i + 1].clone()));
                i += 2; // Skip operator and path
            }
            "2>" => {
                // Stderr redirection (truncate) - next arg is the file path
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Invalid redirection: 2> requires filename".to_string(),
                    ));
                }
                redirections
                    .push(Redirection::new(RedirectionType::Stderr(false), args[i + 1].clone()));
                i += 2; // Skip operator and path
            }
            "2>>" => {
                // Stderr redirection (append) - next arg is the file path
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Invalid redirection: 2>> requires filename".to_string(),
                    ));
                }
                redirections
                    .push(Redirection::new(RedirectionType::Stderr(true), args[i + 1].clone()));
                i += 2; // Skip operator and path
            }
            "2>&1" => {
                // Stderr to stdout redirection
                redirections.push(Redirection::new(RedirectionType::StderrToStdout, String::new()));
                i += 1;
            }
            "1>&2" => {
                // Stdout to stderr redirection
                redirections.push(Redirection::new(RedirectionType::StdoutToStderr, String::new()));
                i += 1;
            }
            "<<" => {
                // Heredoc redirection - next arg is the delimiter
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Heredoc operator requires a delimiter".to_string(),
                    ));
                }
                let delimiter = args[i + 1].clone();
                // Create heredoc with empty content - content will be filled from segment.heredoc_contents
                redirections.push(Redirection::new_heredoc(delimiter, String::new(), false));
                i += 2; // Skip operator and delimiter
            }
            "<<-" => {
                // Heredoc with tab stripping - next arg is the delimiter
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Heredoc operator requires a delimiter".to_string(),
                    ));
                }
                let delimiter = args[i + 1].clone();
                // Create heredoc with empty content - content will be filled from segment.heredoc_contents
                redirections.push(Redirection::new_heredoc(delimiter, String::new(), true));
                i += 2; // Skip operator and delimiter
            }
            "<<<" => {
                // Here-string - next arg is the string content
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Here-string operator requires content".to_string(),
                    ));
                }
                let content = args[i + 1].clone();
                redirections.push(Redirection::new_herestring(content));
                i += 2; // Skip operator and content
            }
            _ => {
                // Regular argument
                clean_args.push(args[i].clone());
                i += 1;
            }
        }
    }

    Ok((clean_args, redirections))
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
            '2' if !in_single_quote
                && !in_double_quote
                && (current_token.is_empty() || current_token == "2") =>
            {
                // Check for 2> or 2>> (stderr redirection operators)
                // Only recognize 2 as redirection prefix if token is empty (we're at start of word)
                if current_token.is_empty() && chars.peek() == Some(&'>') {
                    // Finalize any previous token
                    if had_quotes {
                        tokens.push(Token::Word(current_token.clone()));
                        current_token.clear();
                        had_quotes = false;
                    }

                    chars.next(); // Consume the >
                    if chars.peek() == Some(&'>') {
                        chars.next(); // Consume second >
                        tokens.push(Token::StderrAppend);
                    } else {
                        tokens.push(Token::StderrOut);
                    }
                } else {
                    // Regular character - add to current token
                    current_token.push(ch);
                }
            }
            '>' if !in_single_quote && !in_double_quote => {
                // Check for >> (append) operator or > (output operator)
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
                // Check for <<< (here-string), << (heredoc), or < (input redirection)
                if !current_token.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_token.clone()));
                    current_token.clear();
                    had_quotes = false;
                }
                if chars.peek() == Some(&'<') {
                    chars.next(); // Consume second <
                    if chars.peek() == Some(&'<') {
                        chars.next(); // Consume third < for here-string
                        tokens.push(Token::HereString);
                    } else if chars.peek() == Some(&'-') {
                        chars.next(); // Consume -
                        tokens.push(Token::HeredocStrip);
                    } else {
                        tokens.push(Token::Heredoc);
                    }
                } else {
                    tokens.push(Token::RedirectIn);
                }
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
        // Check if this word is a keyword (only if not quoted)
        if !had_quotes {
            tokens.push(tokenize_keyword(&current_token));
        } else {
            tokens.push(Token::Word(current_token));
        }
    }

    Ok(tokens)
}

/// Tokenize a command line, respecting quotes and escapes
/// Legacy function for backward compatibility
fn tokenize(line: &str) -> Result<Vec<String>> {
    let tokens = tokenize_with_redirections(line)?;
    // Convert Token::Word to String, filter out redirection operators
    Ok(tokens
        .into_iter()
        .filter_map(|t| {
            if let Token::Word(s) = t {
                Some(s)
            } else {
                None
            }
        })
        .collect())
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

/// Parse command line into pipeline
///
/// Converts a command line string into a `Pipeline` containing one or more
/// `PipelineSegment`s. Pipes (`|`) outside quotes split the line into segments.
///
/// # User Story 1: Basic Two-Command Pipeline
///
/// Supports single commands and two-command pipelines. Three or more commands
/// will be rejected during validation.
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
/// let pipeline = parse_pipeline("echo hello | grep hello")?;
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
    let mut tokens = tokenize_with_pipes(line)?;

    // Check for background execution
    let mut background = false;
    if let Some(Token::Background) = tokens.last() {
        background = true;
        tokens.pop();
    }

    // Split tokens at pipe boundaries
    let segments = split_into_segments(tokens)?;

    // Build pipeline
    let pipeline = Pipeline::new(segments, line.to_string(), background);

    // Validate (US1: will reject 3+ commands)
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
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
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
            '>' if !in_single_quote && !in_double_quote => {
                // Redirection operator outside quotes
                if !current_word.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    had_quotes = false;
                }
                // Check for >> (append) operator
                if chars.peek() == Some(&'>') {
                    chars.next(); // Consume second >
                    tokens.push(Token::RedirectAppend);
                } else {
                    tokens.push(Token::RedirectOut);
                }
            }
            '<' if !in_single_quote && !in_double_quote => {
                // Check for << (heredoc) or < (input redirection)
                if !current_word.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    had_quotes = false;
                }
                if chars.peek() == Some(&'<') {
                    chars.next(); // Consume second <
                    if chars.peek() == Some(&'<') {
                        chars.next(); // Consume third < for here-string
                        tokens.push(Token::HereString);
                    } else if chars.peek() == Some(&'-') {
                        chars.next(); // Consume -
                        tokens.push(Token::HeredocStrip);
                    } else {
                        tokens.push(Token::Heredoc);
                    }
                } else {
                    tokens.push(Token::RedirectIn);
                }
            }
            '&' if !in_single_quote && !in_double_quote => {
                // Background operator outside quotes
                if !current_word.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    had_quotes = false;
                }
                tokens.push(Token::Background);
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
/// Vector of `PipelineSegment`s with assigned indices (0, 1, ...)
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
            // Redirection operators are treated as part of the command arguments
            // They will be parsed later when the segment is executed
            Token::RedirectOut => {
                current_segment.push(">".to_string());
            }
            Token::RedirectAppend => {
                current_segment.push(">>".to_string());
            }
            Token::RedirectIn => {
                current_segment.push("<".to_string());
            }
            Token::StderrOut => {
                current_segment.push("2>".to_string());
            }
            Token::StderrAppend => {
                current_segment.push("2>>".to_string());
            }
            Token::StderrToStdout => {
                current_segment.push("2>&1".to_string());
            }
            Token::StdoutToStderr => {
                current_segment.push("1>&2".to_string());
            }
            Token::Heredoc => {
                current_segment.push("<<".to_string());
            }
            Token::HeredocStrip => {
                current_segment.push("<<-".to_string());
            }
            Token::HereString => {
                current_segment.push("<<<".to_string());
            }
            Token::Background => {
                return Err(RushError::Execution(
                    "Background operator '&' must be at the end of the command".to_string(),
                ));
            }
            Token::Keyword(kw) => {
                return Err(RushError::Syntax(format!(
                    "Unexpected keyword '{}' in pipeline",
                    kw.as_str()
                )));
            }
            Token::Newline | Token::Eof => {
                // End of input, finalize current segment if any
                break;
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

/// Information about a pending heredoc that needs content
#[derive(Debug, Clone)]
pub struct PendingHeredoc {
    /// The delimiter that marks the end of heredoc content
    pub delimiter: String,
    /// Whether to strip leading tabs (<<- vs <<)
    pub strip_tabs: bool,
}

/// Check if a command line contains heredocs that need content collection
pub fn get_pending_heredocs(line: &str) -> Result<Vec<PendingHeredoc>> {
    let tokens = tokenize_with_redirections(line)?;
    let mut heredocs = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Heredoc | Token::HeredocStrip => {
                let strip_tabs = matches!(&tokens[i], Token::HeredocStrip);
                if i + 1 < tokens.len() {
                    if let Token::Word(delimiter) = &tokens[i + 1] {
                        heredocs.push(PendingHeredoc { delimiter: delimiter.clone(), strip_tabs });
                        i += 2;
                        continue;
                    }
                }
                return Err(RushError::Execution(
                    "Heredoc operator requires a delimiter".to_string(),
                ));
            }
            _ => i += 1,
        }
    }

    Ok(heredocs)
}

/// Collect heredoc content from lines until the delimiter is found
pub fn collect_heredoc_content<'a, I>(
    lines: &mut I,
    delimiter: &str,
    strip_tabs: bool,
) -> Result<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut content = String::new();

    for line in lines {
        let trimmed = if strip_tabs {
            line.trim_start_matches('\t')
        } else {
            line
        };

        if trimmed == delimiter {
            return Ok(content);
        }

        if strip_tabs {
            content.push_str(line.trim_start_matches('\t'));
        } else {
            content.push_str(line);
        }
        content.push('\n');
    }

    Err(RushError::Execution(format!("Heredoc delimiter '{}' not found", delimiter)))
}

/// Check if all heredocs in the input have their closing delimiters
/// Returns true if no heredocs present, or all heredocs have been closed
pub fn is_heredoc_complete(input: &str) -> bool {
    let mut lines = input.lines();

    // Get the first line (command line)
    let first_line = match lines.next() {
        Some(line) => line,
        None => return true, // Empty input is complete
    };

    // Check for pending heredocs in the command line
    let pending = match get_pending_heredocs(first_line) {
        Ok(p) => p,
        Err(_) => return true, // Parse error - let executor handle it
    };

    if pending.is_empty() {
        return true; // No heredocs
    }

    // Track which heredocs have been closed
    let mut remaining: Vec<&str> = pending.iter().map(|h| h.delimiter.as_str()).collect();

    // Scan through remaining lines looking for delimiters
    for line in lines {
        // For <<-, delimiter can match with leading tabs stripped
        let trimmed_for_strip = line.trim_start_matches('\t');

        // Check if this line matches any pending delimiter
        remaining.retain(|delim| line != *delim && trimmed_for_strip != *delim);

        if remaining.is_empty() {
            return true; // All heredocs closed
        }
    }

    // Some delimiters still pending
    false
}

/// Parse a multi-line command that may contain heredocs
pub fn parse_with_heredocs(input: &str) -> Result<(String, Vec<(String, String, bool)>)> {
    let mut lines = input.lines();

    let command_line = match lines.next() {
        Some(line) => line.to_string(),
        None => return Err(RushError::Execution("Empty input".to_string())),
    };

    let pending = get_pending_heredocs(&command_line)?;

    if pending.is_empty() {
        return Ok((command_line, Vec::new()));
    }

    let mut heredoc_contents = Vec::new();
    for heredoc in pending {
        let content = collect_heredoc_content(&mut lines, &heredoc.delimiter, heredoc.strip_tabs)?;
        heredoc_contents.push((heredoc.delimiter, content, heredoc.strip_tabs));
    }

    Ok((command_line, heredoc_contents))
}

/// Extract redirections from args with heredoc content support
pub fn extract_redirections_with_heredocs(
    args: &[String],
    heredoc_contents: &std::collections::HashMap<String, String>,
) -> Result<(Vec<String>, Vec<super::Redirection>)> {
    use super::{Redirection, RedirectionType};

    let mut clean_args = Vec::new();
    let mut redirections = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            ">" => {
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Redirection operator missing file path".to_string(),
                    ));
                }
                redirections.push(Redirection::new(RedirectionType::Output, args[i + 1].clone()));
                i += 2;
            }
            ">>" => {
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Redirection operator missing file path".to_string(),
                    ));
                }
                redirections.push(Redirection::new(RedirectionType::Append, args[i + 1].clone()));
                i += 2;
            }
            "<" => {
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Redirection operator missing file path".to_string(),
                    ));
                }
                redirections.push(Redirection::new(RedirectionType::Input, args[i + 1].clone()));
                i += 2;
            }
            "2>" => {
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Invalid redirection: 2> requires filename".to_string(),
                    ));
                }
                redirections
                    .push(Redirection::new(RedirectionType::Stderr(false), args[i + 1].clone()));
                i += 2;
            }
            "2>>" => {
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Invalid redirection: 2>> requires filename".to_string(),
                    ));
                }
                redirections
                    .push(Redirection::new(RedirectionType::Stderr(true), args[i + 1].clone()));
                i += 2;
            }
            "2>&1" => {
                redirections.push(Redirection::new(RedirectionType::StderrToStdout, String::new()));
                i += 1;
            }
            "1>&2" => {
                redirections.push(Redirection::new(RedirectionType::StdoutToStderr, String::new()));
                i += 1;
            }
            "<<" => {
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Heredoc operator requires a delimiter".to_string(),
                    ));
                }
                let delimiter = args[i + 1].clone();
                let content = heredoc_contents
                    .get(&delimiter)
                    .cloned()
                    .unwrap_or_default();
                redirections.push(Redirection::new_heredoc(delimiter, content, false));
                i += 2;
            }
            "<<-" => {
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Heredoc operator requires a delimiter".to_string(),
                    ));
                }
                let delimiter = args[i + 1].clone();
                let content = heredoc_contents
                    .get(&delimiter)
                    .cloned()
                    .unwrap_or_default();
                redirections.push(Redirection::new_heredoc(delimiter, content, true));
                i += 2;
            }
            "<<<" => {
                // Here-string - next arg is the string content
                if i + 1 >= args.len() {
                    return Err(RushError::Execution(
                        "Here-string operator requires content".to_string(),
                    ));
                }
                let content = args[i + 1].clone();
                redirections.push(Redirection::new_herestring(content));
                i += 2;
            }
            _ => {
                clean_args.push(args[i].clone());
                i += 1;
            }
        }
    }

    Ok((clean_args, redirections))
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
        let (program, args, redirs) =
            parse_command_with_redirections("ls -la > files.txt").unwrap();
        assert_eq!(program, "ls");
        assert_eq!(args, vec!["-la"]);
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::Output);
        assert_eq!(redirs[0].file_path, "files.txt");
    }

    #[test]
    fn test_parse_with_append_redirection() {
        let (program, args, redirs) =
            parse_command_with_redirections("echo line >> log.txt").unwrap();
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing file path"));
    }

    #[test]
    fn test_parse_quoted_operators_literal() {
        let (program, args, redirs) =
            parse_command_with_redirections("echo \"test > file\"").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["test > file"]);
        assert_eq!(redirs.len(), 0); // No redirections, it's in quotes
    }

    // Stderr redirection tests
    #[test]
    fn test_tokenize_with_stderr_redirect() {
        let tokens = tokenize_with_redirections("echo error 2>error.txt").unwrap();
        // Should have: echo, error, 2>, error.txt
        assert!(tokens.iter().any(|t| matches!(t, Token::StderrOut)));
    }

    #[test]
    fn test_tokenize_with_stderr_append_redirect() {
        let tokens = tokenize_with_redirections("echo msg 2>>error.txt").unwrap();
        // Should recognize 2>> as StderrAppend token
        assert!(tokens.iter().any(|t| matches!(t, Token::StderrAppend)));
    }

    #[test]
    fn test_parse_with_stderr_redirection() {
        let (program, args, redirs) =
            parse_command_with_redirections("cat somefile 2>error.txt").unwrap();
        assert_eq!(program, "cat");
        assert_eq!(args, vec!["somefile"]);
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::Stderr(false));
        assert_eq!(redirs[0].file_path, "error.txt");
    }

    #[test]
    fn test_parse_with_stderr_append_redirection() {
        let (program, args, redirs) =
            parse_command_with_redirections("echo msg 2>>log.txt").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["msg"]);
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::Stderr(true));
        assert_eq!(redirs[0].file_path, "log.txt");
    }

    #[test]
    fn test_parse_stderr_redirect_missing_path() {
        let result = parse_command_with_redirections("echo test 2>");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing file path"));
    }

    #[test]
    fn test_parse_combined_stdout_stderr_redirections() {
        let (program, args, redirs) =
            parse_command_with_redirections("cmd > out.txt 2> err.txt").unwrap();
        assert_eq!(program, "cmd");
        assert_eq!(redirs.len(), 2);
        assert_eq!(redirs[0].redir_type, RedirectionType::Output);
        assert_eq!(redirs[0].file_path, "out.txt");
        assert_eq!(redirs[1].redir_type, RedirectionType::Stderr(false));
        assert_eq!(redirs[1].file_path, "err.txt");
    }

    // Heredoc tokenization tests
    #[test]
    fn test_tokenize_heredoc() {
        let tokens = tokenize_with_pipes("cat << EOF").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Word("cat".to_string()));
        assert_eq!(tokens[1], Token::Heredoc);
        assert_eq!(tokens[2], Token::Word("EOF".to_string()));
    }

    #[test]
    fn test_tokenize_heredoc_strip() {
        let tokens = tokenize_with_pipes("cat <<- MARKER").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Word("cat".to_string()));
        assert_eq!(tokens[1], Token::HeredocStrip);
        assert_eq!(tokens[2], Token::Word("MARKER".to_string()));
    }

    #[test]
    fn test_get_pending_heredocs() {
        let heredocs = get_pending_heredocs("cat << EOF").unwrap();
        assert_eq!(heredocs.len(), 1);
        assert_eq!(heredocs[0].delimiter, "EOF");
        assert!(!heredocs[0].strip_tabs);
    }

    #[test]
    fn test_get_pending_heredocs_strip_tabs() {
        let heredocs = get_pending_heredocs("cat <<- MARKER").unwrap();
        assert_eq!(heredocs.len(), 1);
        assert_eq!(heredocs[0].delimiter, "MARKER");
        assert!(heredocs[0].strip_tabs);
    }

    #[test]
    fn test_collect_heredoc_content() {
        let lines = "line1\nline2\nEOF\nextra";
        let mut iter = lines.lines();
        let content = collect_heredoc_content(&mut iter, "EOF", false).unwrap();
        assert_eq!(content, "line1\nline2\n");
    }

    #[test]
    fn test_collect_heredoc_content_strip_tabs() {
        let lines = "\tline1\n\t\tline2\nEOF";
        let mut iter = lines.lines();
        let content = collect_heredoc_content(&mut iter, "EOF", true).unwrap();
        // <<- strips ALL leading tabs from each line
        assert_eq!(content, "line1\nline2\n");
    }

    #[test]
    fn test_collect_heredoc_content_missing_delimiter() {
        let lines = "line1\nline2\nno_delimiter";
        let mut iter = lines.lines();
        let result = collect_heredoc_content(&mut iter, "EOF", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_parse_with_heredocs() {
        let input = "cat << EOF\nhello world\nEOF";
        let (cmd_line, heredocs) = parse_with_heredocs(input).unwrap();
        assert_eq!(cmd_line, "cat << EOF");
        assert_eq!(heredocs.len(), 1);
        assert_eq!(heredocs[0].0, "EOF"); // delimiter
        assert_eq!(heredocs[0].1, "hello world\n"); // content
        assert!(!heredocs[0].2); // strip_tabs = false
    }

    #[test]
    fn test_parse_with_heredocs_strip_tabs() {
        let input = "cat <<- END\n\thello\n\t\tworld\nEND";
        let (cmd_line, heredocs) = parse_with_heredocs(input).unwrap();
        assert_eq!(cmd_line, "cat <<- END");
        assert_eq!(heredocs.len(), 1);
        assert_eq!(heredocs[0].0, "END"); // delimiter
                                          // <<- strips ALL leading tabs from each line
        assert_eq!(heredocs[0].1, "hello\nworld\n"); // content (all leading tabs stripped)
        assert!(heredocs[0].2); // strip_tabs = true
    }

    #[test]
    fn test_extract_redirections_with_heredocs() {
        use std::collections::HashMap;
        let args = vec!["arg1".to_string(), "<<".to_string(), "EOF".to_string()];
        let mut heredoc_contents = HashMap::new();
        heredoc_contents.insert("EOF".to_string(), "test content\n".to_string());

        let (clean_args, redirs) =
            extract_redirections_with_heredocs(&args, &heredoc_contents).unwrap();
        assert_eq!(clean_args, vec!["arg1"]);
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::Heredoc);
        assert_eq!(redirs[0].file_path, "EOF");
        assert_eq!(redirs[0].heredoc_content, Some("test content\n".to_string()));
    }

    #[test]
    fn test_extract_redirections_from_args_heredoc() {
        let args = vec![
            "arg1".to_string(),
            "<<".to_string(),
            "DELIMITER".to_string(),
        ];
        let (clean_args, redirs) = extract_redirections_from_args(&args).unwrap();
        assert_eq!(clean_args, vec!["arg1"]);
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::Heredoc);
        assert_eq!(redirs[0].file_path, "DELIMITER");
        // Content is empty when using extract_redirections_from_args
        assert_eq!(redirs[0].heredoc_content, Some(String::new()));
    }

    #[test]
    fn test_extract_redirections_from_args_heredoc_strip() {
        let args = vec!["<<-".to_string(), "MARKER".to_string()];
        let (clean_args, redirs) = extract_redirections_from_args(&args).unwrap();
        assert!(clean_args.is_empty());
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::HeredocStrip);
        assert_eq!(redirs[0].file_path, "MARKER");
    }

    #[test]
    fn test_heredoc_missing_delimiter_error() {
        let args = vec!["<<".to_string()];
        let result = extract_redirections_from_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("delimiter"));
    }

    // is_heredoc_complete tests
    #[test]
    fn test_is_heredoc_complete_no_heredocs() {
        assert!(is_heredoc_complete("echo hello"));
        assert!(is_heredoc_complete("ls -la | grep foo"));
        assert!(is_heredoc_complete(""));
    }

    #[test]
    fn test_is_heredoc_complete_with_delimiter() {
        assert!(is_heredoc_complete("cat <<EOF\nhello world\nEOF"));
        assert!(is_heredoc_complete("cat << MARKER\nline1\nline2\nMARKER"));
    }

    #[test]
    fn test_is_heredoc_incomplete() {
        assert!(!is_heredoc_complete("cat <<EOF\nhello"));
        assert!(!is_heredoc_complete("cat <<EOF"));
        assert!(!is_heredoc_complete("cat <<EOF\nline1\nline2"));
    }

    #[test]
    fn test_is_heredoc_complete_strip_tabs() {
        // <<- allows delimiter with leading tabs
        assert!(is_heredoc_complete("cat <<-EOF\n\thello\n\tEOF"));
        assert!(is_heredoc_complete("cat <<-EOF\nhello\nEOF"));
    }

    #[test]
    fn test_is_heredoc_complete_empty_content() {
        assert!(is_heredoc_complete("cat <<EOF\nEOF"));
    }

    #[test]
    fn test_is_heredoc_complete_multiple() {
        // Multiple heredocs - both must be closed
        assert!(is_heredoc_complete("cmd <<A <<B\nfirst\nA\nsecond\nB"));
        assert!(!is_heredoc_complete("cmd <<A <<B\nfirst\nA\nsecond"));
    }
}
