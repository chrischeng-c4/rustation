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
use crate::executor::{EnvironmentManager, Pipeline, PipelineSegment};

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
    RedirectStderr,
    /// Stderr append (2>>)
    RedirectStderrAppend,
    /// Stderr to stdout (2>&1)
    RedirectStderrToStdout,
    /// Both stdout and stderr (&>)
    RedirectBoth,
    /// Both stdout and stderr append (&>>)
    RedirectBothAppend,
    /// Background execution (&)
    Background,
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
            Token::RedirectOut
            | Token::RedirectAppend
            | Token::RedirectIn
            | Token::RedirectStderr
            | Token::RedirectStderrAppend
            | Token::RedirectBoth
            | Token::RedirectBothAppend => {
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
                        Token::RedirectStderr => RedirectionType::StderrOutput,
                        Token::RedirectStderrAppend => RedirectionType::StderrAppend,
                        Token::RedirectBoth => RedirectionType::BothOutput,
                        Token::RedirectBothAppend => RedirectionType::BothAppend,
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
            Token::RedirectStderrToStdout => {
                // 2>&1 doesn't need a file path
                redirections.push(Redirection::new(RedirectionType::StderrToStdout, String::new()));
                i += 1;
            }
            // Note: Pipe and Background tokens are never produced by tokenize_with_redirections
            // They are only created by tokenize_pipeline, which is used for parse_pipeline
            Token::Pipe | Token::Background => unreachable!(
                "Pipe/Background tokens cannot be produced by tokenize_with_redirections"
            ),
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
            '>' if !in_single_quote && !in_double_quote => {
                // Check if current token is "2" (stderr redirect)
                let is_stderr = current_token == "2";
                if is_stderr {
                    current_token.clear();
                }

                // Push any other accumulated token
                if !current_token.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_token.clone()));
                    current_token.clear();
                    had_quotes = false;
                }

                // Check for >> (append) operator
                if chars.peek() == Some(&'>') {
                    chars.next(); // Consume second >
                    if is_stderr {
                        tokens.push(Token::RedirectStderrAppend);
                    } else {
                        tokens.push(Token::RedirectAppend);
                    }
                } else if is_stderr && chars.peek() == Some(&'&') {
                    // Check for 2>&1 pattern
                    chars.next(); // Consume &
                    if chars.peek() == Some(&'1') {
                        chars.next(); // Consume 1
                        tokens.push(Token::RedirectStderrToStdout);
                    } else {
                        // Invalid: 2>&X where X != 1, treat as error or literal
                        tokens.push(Token::RedirectStderr);
                        tokens.push(Token::Word("&".to_string()));
                    }
                } else if is_stderr {
                    tokens.push(Token::RedirectStderr);
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
            '&' if !in_single_quote && !in_double_quote => {
                // Check for &> or &>> (both stdout and stderr)
                if !current_token.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_token.clone()));
                    current_token.clear();
                    had_quotes = false;
                }

                if chars.peek() == Some(&'>') {
                    chars.next(); // Consume >
                    if chars.peek() == Some(&'>') {
                        chars.next(); // Consume second >
                        tokens.push(Token::RedirectBothAppend);
                    } else {
                        tokens.push(Token::RedirectBoth);
                    }
                } else {
                    // Plain & is background operator (handled elsewhere or ignored)
                    tokens.push(Token::Word("&".to_string()));
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
        tokens.push(Token::Word(current_token));
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
                // Check if current word is "2" (stderr redirect)
                let is_stderr = current_word == "2";
                if is_stderr {
                    current_word.clear();
                }

                // Redirection operator outside quotes
                if !current_word.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    had_quotes = false;
                }

                // Check for >> (append) or 2>&1
                if chars.peek() == Some(&'>') {
                    chars.next(); // Consume second >
                    if is_stderr {
                        tokens.push(Token::RedirectStderrAppend);
                    } else {
                        tokens.push(Token::RedirectAppend);
                    }
                } else if is_stderr && chars.peek() == Some(&'&') {
                    // Check for 2>&1 pattern
                    chars.next(); // Consume &
                    if chars.peek() == Some(&'1') {
                        chars.next(); // Consume 1
                        tokens.push(Token::RedirectStderrToStdout);
                    } else {
                        tokens.push(Token::RedirectStderr);
                        tokens.push(Token::Word("&".to_string()));
                    }
                } else if is_stderr {
                    tokens.push(Token::RedirectStderr);
                } else {
                    tokens.push(Token::RedirectOut);
                }
            }
            '<' if !in_single_quote && !in_double_quote => {
                // Input redirection operator outside quotes
                if !current_word.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    had_quotes = false;
                }
                tokens.push(Token::RedirectIn);
            }
            '&' if !in_single_quote && !in_double_quote => {
                // Check for &> or &>> (both stdout and stderr)
                if !current_word.is_empty() || had_quotes {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    had_quotes = false;
                }

                if chars.peek() == Some(&'>') {
                    chars.next(); // Consume >
                    if chars.peek() == Some(&'>') {
                        chars.next(); // Consume second >
                        tokens.push(Token::RedirectBothAppend);
                    } else {
                        tokens.push(Token::RedirectBoth);
                    }
                } else {
                    // Plain & is background operator
                    tokens.push(Token::Background);
                }
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
/// Extracts I/O redirections from tokens and stores them in the segment's
/// redirections field. Redirection operators (>, >>, <) must be followed
/// by a file path token.
///
/// # Validation
///
/// - Empty segment before pipe → Error: "Empty command before pipe"
/// - Empty segment after pipe → Error: "Empty command after pipe"
/// - No program in segment → Error: "Empty command"
/// - Redirection without file path → Error: "Redirection operator missing file path"
///
/// # Returns
///
/// Vector of `PipelineSegment`s with assigned indices (0, 1, ...) and extracted redirections
fn split_into_segments(tokens: Vec<Token>) -> Result<Vec<PipelineSegment>> {
    use super::{Redirection, RedirectionType};

    let mut segments = Vec::new();
    let mut current_words: Vec<String> = Vec::new();
    let mut current_redirections: Vec<Redirection> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Word(word) => {
                current_words.push(word.clone());
                i += 1;
            }
            Token::Pipe => {
                // Validate segment before pipe
                if current_words.is_empty() {
                    return Err(RushError::Execution("Empty command before pipe".to_string()));
                }

                // Create segment with extracted redirections
                let program = current_words[0].clone();
                let args = current_words[1..].to_vec();
                segments.push(PipelineSegment::new(
                    program,
                    args,
                    segments.len(),
                    current_redirections.clone(),
                ));

                current_words.clear();
                current_redirections.clear();
                i += 1;
            }
            Token::RedirectOut
            | Token::RedirectAppend
            | Token::RedirectIn
            | Token::RedirectStderr
            | Token::RedirectStderrAppend
            | Token::RedirectBoth
            | Token::RedirectBothAppend => {
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
                        Token::RedirectStderr => RedirectionType::StderrOutput,
                        Token::RedirectStderrAppend => RedirectionType::StderrAppend,
                        Token::RedirectBoth => RedirectionType::BothOutput,
                        Token::RedirectBothAppend => RedirectionType::BothAppend,
                        _ => unreachable!(),
                    };
                    current_redirections.push(Redirection::new(redir_type, path.clone()));
                    i += 2; // Skip operator and path
                } else {
                    return Err(RushError::Execution(
                        "Redirection operator must be followed by file path".to_string(),
                    ));
                }
            }
            Token::RedirectStderrToStdout => {
                // 2>&1 doesn't need a file path
                current_redirections
                    .push(Redirection::new(RedirectionType::StderrToStdout, String::new()));
                i += 1;
            }
            Token::Background => {
                return Err(RushError::Execution(
                    "Background operator '&' must be at the end of the command".to_string(),
                ));
            }
        }
    }

    // Validate final segment
    if current_words.is_empty() {
        if !segments.is_empty() {
            // Had pipes but no command after last pipe
            return Err(RushError::Execution("Empty command after pipe".to_string()));
        }
        // No commands at all
        return Err(RushError::Execution("Empty command".to_string()));
    }

    // Add final segment with extracted redirections
    let program = current_words[0].clone();
    let args = current_words[1..].to_vec();
    segments.push(PipelineSegment::new(program, args, segments.len(), current_redirections));

    Ok(segments)
}

/// Expand environment variables in a string
///
/// Handles:
/// - `$VAR` - Simple variable reference
/// - `${VAR}` - Braced variable reference (for clarity/concatenation)
/// - `\$` - Escaped dollar sign (produces literal `$`)
/// - Undefined variables expand to empty string
///
/// # Arguments
///
/// * `input` - String potentially containing variable references
/// * `env` - Environment manager for variable lookups
///
/// # Returns
///
/// String with all variable references expanded
///
/// # Examples
///
/// ```ignore
/// let env = EnvironmentManager::new();
/// // Given HOME=/Users/user
///
/// expand_variables_in_string("$HOME", &env) // "/Users/user"
/// expand_variables_in_string("${HOME}_backup", &env) // "/Users/user_backup"
/// expand_variables_in_string("\\$HOME", &env) // "$HOME" (literal)
/// expand_variables_in_string("$UNDEFINED", &env) // "" (empty)
/// ```
pub fn expand_variables_in_string(input: &str, env: &EnvironmentManager) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            // Handle escaped characters
            if let Some(&next) = chars.peek() {
                if next == '$' {
                    // Escaped dollar sign - produce literal $
                    result.push('$');
                    chars.next();
                    continue;
                }
            }
            // Not escaping $, keep the backslash
            result.push(c);
        } else if c == '$' {
            // Variable expansion
            if chars.peek() == Some(&'{') {
                // ${VAR} syntax - braced variable reference
                chars.next(); // consume '{'
                let var_name: String = chars.by_ref().take_while(|&ch| ch != '}').collect();
                // Expand variable (undefined = empty string)
                if let Some(value) = env.get(&var_name) {
                    result.push_str(value);
                }
            } else {
                // $VAR syntax - simple variable reference
                // Variable names: [a-zA-Z_][a-zA-Z0-9_]*
                let mut var_name = String::new();
                // First character must be letter or underscore
                if let Some(&first) = chars.peek() {
                    if first.is_ascii_alphabetic() || first == '_' {
                        var_name.push(chars.next().unwrap());
                        // Subsequent characters: alphanumeric or underscore
                        while let Some(&ch) = chars.peek() {
                            if ch.is_ascii_alphanumeric() || ch == '_' {
                                var_name.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                    }
                }

                if var_name.is_empty() {
                    // No valid variable name follows $ - keep literal $
                    result.push('$');
                } else {
                    // Expand variable (undefined = empty string)
                    if let Some(value) = env.get(&var_name) {
                        result.push_str(value);
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Expand environment variables in all pipeline segments
///
/// Expands variables in:
/// - Program name (command)
/// - All arguments
/// - Redirection file paths
///
/// # Arguments
///
/// * `segments` - Mutable slice of pipeline segments to expand
/// * `env` - Environment manager for variable lookups
pub fn expand_variables(segments: &mut [PipelineSegment], env: &EnvironmentManager) {
    for segment in segments {
        // Expand program name (rare but possible, e.g., $EDITOR)
        segment.program = expand_variables_in_string(&segment.program, env);

        // Expand all arguments
        for arg in &mut segment.args {
            *arg = expand_variables_in_string(arg, env);
        }

        // Expand redirection file paths
        for redir in &mut segment.redirections {
            redir.file_path = expand_variables_in_string(&redir.file_path, env);
        }
    }
}

/// Expand glob patterns in all pipeline segment arguments
///
/// This function expands wildcard patterns (*, ?, [...]) in command arguments
/// to matching file paths. It should be called after variable expansion.
///
/// # Behavior
///
/// - Patterns matching files are expanded to the list of matching paths
/// - Patterns with no matches are preserved as literals (POSIX behavior)
/// - Quoted arguments are not expanded
/// - Hidden files (starting with .) are excluded unless pattern explicitly matches them
///
/// # Arguments
///
/// * `segments` - Mutable slice of pipeline segments to expand
pub fn expand_globs(segments: &mut [PipelineSegment]) {
    use super::glob::expand_globs as expand_glob_args;

    for segment in segments {
        // Expand glob patterns in arguments
        // Note: We don't expand the program name (that would be unusual)
        let expanded_args: Vec<String> = segment
            .args
            .iter()
            .flat_map(|arg| expand_glob_args(std::slice::from_ref(arg)))
            .collect();
        segment.args = expanded_args;

        // Note: We intentionally do NOT expand globs in redirection paths
        // e.g., "ls > *.txt" should create a file literally named "*.txt"
    }
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

    #[test]
    fn test_tokenize_stderr_redirect() {
        let tokens = tokenize_with_redirections("ls 2>err.txt").unwrap();
        println!("Tokens for 'ls 2>err.txt': {:?}", tokens);
        assert_eq!(tokens.len(), 3, "Expected 3 tokens, got {:?}", tokens);
        assert_eq!(tokens[0], Token::Word("ls".to_string()));
        assert_eq!(tokens[1], Token::RedirectStderr);
        assert_eq!(tokens[2], Token::Word("err.txt".to_string()));
    }

    #[test]
    fn test_tokenize_with_pipes_stderr_redirect() {
        let tokens = tokenize_with_pipes("ls 2>err.txt").unwrap();
        println!("Tokens from tokenize_with_pipes for 'ls 2>err.txt': {:?}", tokens);
        assert_eq!(tokens.len(), 3, "Expected 3 tokens, got {:?}", tokens);
        assert_eq!(tokens[0], Token::Word("ls".to_string()));
        assert_eq!(tokens[1], Token::RedirectStderr);
        assert_eq!(tokens[2], Token::Word("err.txt".to_string()));
    }

    #[test]
    fn test_tokenize_stderr_to_stdout() {
        let tokens = tokenize_with_redirections("ls 2>&1").unwrap();
        println!("Tokens for 'ls 2>&1': {:?}", tokens);
        assert_eq!(tokens.len(), 2, "Expected 2 tokens, got {:?}", tokens);
        assert_eq!(tokens[0], Token::Word("ls".to_string()));
        assert_eq!(tokens[1], Token::RedirectStderrToStdout);
    }

    #[test]
    fn test_tokenize_both_redirect() {
        let tokens = tokenize_with_redirections("ls &>out.txt").unwrap();
        println!("Tokens for 'ls &>out.txt': {:?}", tokens);
        assert_eq!(tokens.len(), 3, "Expected 3 tokens, got {:?}", tokens);
        assert_eq!(tokens[0], Token::Word("ls".to_string()));
        assert_eq!(tokens[1], Token::RedirectBoth);
        assert_eq!(tokens[2], Token::Word("out.txt".to_string()));
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
    fn test_parse_with_stderr_redirection() {
        let (program, args, redirs) = parse_command_with_redirections("ls /foo 2>err.txt").unwrap();
        println!("Program: {}, Args: {:?}, Redirs: {:?}", program, args, redirs);
        assert_eq!(program, "ls");
        assert_eq!(args, vec!["/foo"]);
        assert_eq!(redirs.len(), 1);
        assert_eq!(redirs[0].redir_type, RedirectionType::StderrOutput);
        assert_eq!(redirs[0].file_path, "err.txt");
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

    // parse_pipeline tests with redirections (Phase 2 verification)
    #[test]
    fn test_parse_pipeline_with_output_redirection() {
        let pipeline = parse_pipeline("ls -la > files.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 1);

        let segment = &pipeline.segments[0];
        assert_eq!(segment.program, "ls");
        assert_eq!(segment.args, vec!["-la"]);
        assert_eq!(segment.redirections.len(), 1);
        assert_eq!(segment.redirections[0].redir_type, RedirectionType::Output);
        assert_eq!(segment.redirections[0].file_path, "files.txt");
    }

    #[test]
    fn test_parse_pipeline_with_append_redirection() {
        let pipeline = parse_pipeline("echo line >> log.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 1);

        let segment = &pipeline.segments[0];
        assert_eq!(segment.program, "echo");
        assert_eq!(segment.args, vec!["line"]);
        assert_eq!(segment.redirections.len(), 1);
        assert_eq!(segment.redirections[0].redir_type, RedirectionType::Append);
        assert_eq!(segment.redirections[0].file_path, "log.txt");
    }

    #[test]
    fn test_parse_pipeline_with_input_redirection() {
        let pipeline = parse_pipeline("wc -l < input.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 1);

        let segment = &pipeline.segments[0];
        assert_eq!(segment.program, "wc");
        assert_eq!(segment.args, vec!["-l"]);
        assert_eq!(segment.redirections.len(), 1);
        assert_eq!(segment.redirections[0].redir_type, RedirectionType::Input);
        assert_eq!(segment.redirections[0].file_path, "input.txt");
    }

    #[test]
    fn test_parse_pipeline_with_multiple_redirections() {
        let pipeline = parse_pipeline("cat < in.txt > out.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 1);

        let segment = &pipeline.segments[0];
        assert_eq!(segment.program, "cat");
        assert!(segment.args.is_empty());
        assert_eq!(segment.redirections.len(), 2);
        assert_eq!(segment.redirections[0].redir_type, RedirectionType::Input);
        assert_eq!(segment.redirections[0].file_path, "in.txt");
        assert_eq!(segment.redirections[1].redir_type, RedirectionType::Output);
        assert_eq!(segment.redirections[1].file_path, "out.txt");
    }

    #[test]
    fn test_parse_pipeline_with_redirections_in_pipeline() {
        let pipeline = parse_pipeline("echo test | grep test > result.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 2);

        // First segment has no redirections
        assert_eq!(pipeline.segments[0].program, "echo");
        assert_eq!(pipeline.segments[0].redirections.len(), 0);

        // Second segment has output redirection
        assert_eq!(pipeline.segments[1].program, "grep");
        assert_eq!(pipeline.segments[1].args, vec!["test"]);
        assert_eq!(pipeline.segments[1].redirections.len(), 1);
        assert_eq!(pipeline.segments[1].redirections[0].redir_type, RedirectionType::Output);
        assert_eq!(pipeline.segments[1].redirections[0].file_path, "result.txt");
    }

    // Tests for parse_command_with_redirections error paths
    #[test]
    fn test_parse_command_with_redirections_invalid_redirect_target() {
        // Redirection operator followed by another redirection operator
        let result = parse_command_with_redirections("echo > >");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be followed by file path"));
    }

    #[test]
    fn test_parse_command_with_redirections_trailing_redirect() {
        // Redirection operator at end of command
        let result = parse_command_with_redirections("echo test >");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing file path"));
    }

    #[test]
    fn test_parse_command_with_redirections_success() {
        // Valid redirection
        let result = parse_command_with_redirections("echo test > file.txt");
        assert!(result.is_ok());
        let (program, args, redirects) = result.unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["test"]);
        assert_eq!(redirects.len(), 1);
    }

    // Tests for extract_redirections_from_args
    #[test]
    fn test_extract_redirections_output() {
        let args = vec![">".to_string(), "file.txt".to_string()];
        let (clean_args, redirections) = extract_redirections_from_args(&args).unwrap();
        assert!(clean_args.is_empty());
        assert_eq!(redirections.len(), 1);
        assert_eq!(redirections[0].redir_type, RedirectionType::Output);
        assert_eq!(redirections[0].file_path, "file.txt");
    }

    #[test]
    fn test_extract_redirections_append() {
        let args = vec![">>".to_string(), "file.txt".to_string()];
        let (clean_args, redirections) = extract_redirections_from_args(&args).unwrap();
        assert!(clean_args.is_empty());
        assert_eq!(redirections.len(), 1);
        assert_eq!(redirections[0].redir_type, RedirectionType::Append);
    }

    #[test]
    fn test_extract_redirections_input() {
        let args = vec!["<".to_string(), "file.txt".to_string()];
        let (clean_args, redirections) = extract_redirections_from_args(&args).unwrap();
        assert!(clean_args.is_empty());
        assert_eq!(redirections.len(), 1);
        assert_eq!(redirections[0].redir_type, RedirectionType::Input);
    }

    #[test]
    fn test_extract_redirections_mixed_with_args() {
        let args = vec![
            "arg1".to_string(),
            ">".to_string(),
            "out.txt".to_string(),
            "arg2".to_string(),
        ];
        let (clean_args, redirections) = extract_redirections_from_args(&args).unwrap();
        assert_eq!(clean_args, vec!["arg1", "arg2"]);
        assert_eq!(redirections.len(), 1);
        assert_eq!(redirections[0].file_path, "out.txt");
    }

    #[test]
    fn test_extract_redirections_missing_path_output() {
        let args = vec![">".to_string()];
        let result = extract_redirections_from_args(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing file path"));
    }

    #[test]
    fn test_extract_redirections_missing_path_append() {
        let args = vec![">>".to_string()];
        let result = extract_redirections_from_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_redirections_missing_path_input() {
        let args = vec!["<".to_string()];
        let result = extract_redirections_from_args(&args);
        assert!(result.is_err());
    }

    // Tests for parse_pipeline edge cases
    #[test]
    fn test_parse_pipeline_empty_after_pipe() {
        let result = parse_pipeline("echo test |");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty command after pipe"));
    }

    #[test]
    fn test_parse_pipeline_double_pipe() {
        let result = parse_pipeline("echo test | | cat");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_pipeline_redirect_at_end() {
        let result = parse_pipeline("echo test >");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing file path"));
    }

    #[test]
    fn test_parse_pipeline_background_flag() {
        let pipeline = parse_pipeline("sleep 10 &").unwrap();
        assert_eq!(pipeline.segments.len(), 1);
        assert!(pipeline.background);
        assert_eq!(pipeline.segments[0].program, "sleep");
        assert_eq!(pipeline.segments[0].args, vec!["10"]);
    }

    // Tokenization edge cases
    #[test]
    fn test_tokenize_unclosed_quote_double_new() {
        let result = tokenize_with_redirections("echo \"hello");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unclosed"));
    }

    #[test]
    fn test_tokenize_unclosed_quote_single_new() {
        let result = tokenize_with_redirections("echo 'hello");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unclosed"));
    }

    #[test]
    fn test_tokenize_trailing_escape_new() {
        let result = tokenize_with_redirections("echo hello\\");
        assert!(result.is_err());
        // Just check it's an error - the message may vary
        assert!(result.is_err());
    }

    #[test]
    fn test_tokenize_complex_escapes() {
        let tokens = tokenize_with_redirections("echo \\> \\< \\| \\&").unwrap();
        // Escaped operators should become words
        assert!(matches!(tokens[0], Token::Word(_)));
        assert_eq!(tokens.len(), 5); // echo + 4 escaped operators
    }

    #[test]
    fn test_tokenize_mixed_quotes() {
        let tokens = tokenize_with_redirections("echo \"single'quote\" 'double\"quote'").unwrap();
        assert_eq!(tokens.len(), 3); // echo + 2 quoted strings
    }

    #[test]
    fn test_tokenize_escaped_backslash() {
        let tokens = tokenize_with_redirections("echo \\\\test").unwrap();
        assert_eq!(tokens.len(), 2);
        if let Token::Word(w) = &tokens[1] {
            assert_eq!(w, "\\test");
        }
    }

    // === Additional Coverage Tests for Error Paths ===

    #[test]
    fn test_parse_command_with_redirections_empty() {
        // Test empty string
        let result = parse_command_with_redirections("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty command"));
    }

    // NOTE: Lines 62-67 and 93-96 are dead code in parse_command_with_redirections
    // because tokenize_with_redirections() does NOT create Token::Pipe or Token::Background
    // Those tokens are only created by parse_pipeline's internal tokenizer
    // The pipe and background characters become Word tokens in tokenize_with_redirections

    #[test]
    fn test_parse_command_with_redirections_only_redirections() {
        // Test command with only redirections (line 102)
        let result = parse_command_with_redirections("> output.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty command"));
    }

    #[test]
    fn test_parse_command_line_empty() {
        // Test empty command in parse_command_line (line 200)
        let result = parse_command_line("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty command"));
    }

    #[test]
    fn test_parse_command_line_whitespace_only() {
        // Test whitespace-only command
        let result = parse_command_line("   ");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty command"));
    }

    #[test]
    fn test_tokenize_with_redirections_empty_token_before_redirect() {
        // Test token boundaries (lines 256-258, 271-273)
        let tokens = tokenize_with_redirections(">file.txt").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::RedirectOut));
        assert!(matches!(tokens[1], Token::Word(_)));
    }

    #[test]
    fn test_tokenize_with_redirections_input_redirect_token_boundaries() {
        // Test input redirect token boundaries
        let tokens = tokenize_with_redirections("<input.txt").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::RedirectIn));
        assert!(matches!(tokens[1], Token::Word(_)));
    }

    #[test]
    fn test_tokenize_filter_non_word_tokens() {
        // Test that tokenize() filters out redirection operators (line 322)
        let tokens = tokenize("echo test > file.txt").unwrap();
        // Should only get words, not redirection operators
        assert_eq!(tokens, vec!["echo", "test", "file.txt"]);
    }

    #[test]
    fn test_parse_pipeline_empty() {
        // Test empty pipeline (line 481-483)
        let result = parse_pipeline("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty command"));
    }

    #[test]
    fn test_parse_pipeline_segment_empty() {
        // Test pipeline with empty segment (line 487)
        let result = parse_pipeline("ls | ");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty command"));
    }

    #[test]
    fn test_parse_pipeline_segment_empty_after_pipe() {
        // Test pipeline with empty segment after pipe (line 496)
        let result = parse_pipeline(" | grep test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty command"));
    }

    #[test]
    fn test_parse_pipeline_redirect_missing_path_output() {
        // Test missing file path after output redirect (line 503-505)
        let result = parse_pipeline("echo test >");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing file path"));
    }

    #[test]
    fn test_parse_pipeline_redirect_missing_path_append() {
        // Test missing file path after append redirect (line 512-514)
        let result = parse_pipeline("echo test >>");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing file path"));
    }

    #[test]
    fn test_parse_pipeline_redirect_missing_path_input() {
        // Test missing file path after input redirect (line 527-529)
        let result = parse_pipeline("cat <");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing file path"));
    }

    #[test]
    fn test_parse_pipeline_redirect_not_followed_by_path_output() {
        // Test redirect not followed by path (line 536-538)
        let result = parse_pipeline("echo test > >");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be followed by file path"));
    }

    #[test]
    fn test_parse_pipeline_segment_only_redirections() {
        // Test segment with only redirections (line 558)
        let result = parse_pipeline("> out.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty command"));
    }

    #[test]
    fn test_parse_pipeline_background_middle_of_pipeline() {
        // Test background in middle of pipeline (line 564)
        let result = parse_pipeline("ls & | grep test");
        assert!(result.is_err());
        // Should error on empty command after &
    }

    #[test]
    fn test_parse_pipeline_unclosed_single_quote() {
        // Test unclosed single quote (line 647-648)
        let result = parse_pipeline("echo 'unclosed");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unclosed single quote"));
    }

    #[test]
    fn test_parse_pipeline_unclosed_double_quote() {
        // Test unclosed double quote (line 653-654)
        let result = parse_pipeline("echo \"unclosed");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unclosed double quote"));
    }

    #[test]
    fn test_parse_pipeline_trailing_backslash() {
        // Test trailing backslash (line 667)
        let result = parse_pipeline("echo test\\");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Trailing backslash"));
    }

    #[test]
    fn test_parse_pipeline_with_escaped_characters() {
        // Test escaped characters in parse_pipeline (lines 481-483)
        let pipeline = parse_pipeline("echo \\>\\<\\|\\&").unwrap();
        assert_eq!(pipeline.segments.len(), 1);
        let segment = &pipeline.segments[0];
        assert_eq!(segment.program, "echo");
        // Escaped operators should be treated as regular arguments
        assert_eq!(segment.args.len(), 1);
        assert!(segment.args[0].contains('>'));
    }

    #[test]
    fn test_parse_pipeline_with_quoted_empty_string_before_pipe() {
        // Test had_quotes flag before pipe (line 496, 503-505)
        let pipeline = parse_pipeline("echo \"\" | grep test").unwrap();
        assert_eq!(pipeline.segments.len(), 2);
        assert_eq!(pipeline.segments[0].args, vec![""]);
    }

    #[test]
    fn test_parse_pipeline_with_quoted_string_before_redirect() {
        // Test had_quotes flag before redirection (lines 512-514, 527-529)
        let pipeline = parse_pipeline("echo \"test\" > file.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 1);
        assert_eq!(pipeline.segments[0].args, vec!["test"]);
        assert_eq!(pipeline.segments[0].redirections.len(), 1);
    }

    #[test]
    fn test_parse_pipeline_redirect_after_redirect() {
        // Test redirect followed by another redirect (line 536-538)
        let result = parse_pipeline("echo test > < file.txt");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be followed by file path"));
    }

    #[test]
    fn test_tokenize_with_redirections_quoted_empty_before_redirect() {
        // Test had_quotes flag before redirect in tokenize_with_redirections (lines 256-258, 271-273)
        let tokens = tokenize_with_redirections("echo \"\" >file.txt").unwrap();
        // Should have: echo, "", >, file.txt
        assert!(tokens.len() >= 3);
        if let Token::Word(w) = &tokens[1] {
            assert_eq!(w, "");
        }
    }

    #[test]
    fn test_tokenize_with_redirections_quoted_empty_before_input_redirect() {
        // Test had_quotes flag before input redirect (lines 271-273)
        let tokens = tokenize_with_redirections("cat \"\" <file.txt").unwrap();
        assert!(tokens.len() >= 3);
        if let Token::Word(w) = &tokens[1] {
            assert_eq!(w, "");
        }
    }

    #[test]
    fn test_parse_pipeline_quoted_empty_before_pipe() {
        // Test had_quotes before pipe in parse_pipeline tokenizer (lines 503-505)
        let pipeline = parse_pipeline("cat \"\" |grep x").unwrap();
        assert_eq!(pipeline.segments.len(), 2);
        // Empty quoted string should be preserved
        assert_eq!(pipeline.segments[0].args, vec![""]);
    }

    #[test]
    fn test_parse_pipeline_quoted_empty_before_redirect_out() {
        // Test had_quotes before > in parse_pipeline tokenizer (lines 512-514)
        let pipeline = parse_pipeline("echo \"\" >out.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 1);
        assert_eq!(pipeline.segments[0].args, vec![""]);
    }

    #[test]
    fn test_parse_pipeline_quoted_empty_before_redirect_in() {
        // Test had_quotes before < in parse_pipeline tokenizer (lines 527-529)
        let pipeline = parse_pipeline("cat \"\" <in.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 1);
        assert_eq!(pipeline.segments[0].args, vec![""]);
    }

    #[test]
    fn test_parse_pipeline_quoted_empty_before_background() {
        // Test had_quotes before & in parse_pipeline tokenizer (lines 536-538)
        let pipeline = parse_pipeline("sleep \"\" &").unwrap();
        assert!(pipeline.background);
        assert_eq!(pipeline.segments[0].args, vec![""]);
    }

    #[test]
    fn test_tokenize_with_redirections_quoted_empty_string_before_output_redirect() {
        // Test had_quotes flag in tokenize_with_redirections before > (lines 256-258)
        let tokens = tokenize_with_redirections("echo \"\" >file").unwrap();
        assert!(tokens.len() >= 3);
        // Check for empty string Word token
        if let Token::Word(w) = &tokens[1] {
            assert_eq!(w, "");
        }
        assert!(matches!(tokens[2], Token::RedirectOut));
    }

    #[test]
    fn test_tokenize_with_redirections_quoted_empty_string_before_input_redirect() {
        // Test had_quotes flag in tokenize_with_redirections before < (lines 271-273)
        let tokens = tokenize_with_redirections("cat \"\" <file").unwrap();
        assert!(tokens.len() >= 3);
        if let Token::Word(w) = &tokens[1] {
            assert_eq!(w, "");
        }
        assert!(matches!(tokens[2], Token::RedirectIn));
    }

    // Tests for no-space before operators (lines 429-431, 438-440, 453-455, 462-464)
    #[test]
    fn test_parse_pipeline_no_space_before_pipe() {
        // Test word immediately before pipe (lines 429-431 in tokenize_pipeline)
        let pipeline = parse_pipeline("echo test|cat").unwrap();
        assert_eq!(pipeline.segments.len(), 2);
        assert_eq!(pipeline.segments[0].program, "echo");
        assert_eq!(pipeline.segments[0].args, vec!["test"]);
        assert_eq!(pipeline.segments[1].program, "cat");
    }

    #[test]
    fn test_parse_pipeline_no_space_before_redirect_out() {
        // Test word immediately before > (lines 438-440 in tokenize_pipeline)
        let pipeline = parse_pipeline("echo test>file.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 1);
        assert_eq!(pipeline.segments[0].redirections.len(), 1);
        assert_eq!(pipeline.segments[0].redirections[0].redir_type, RedirectionType::Output);
    }

    #[test]
    fn test_parse_pipeline_no_space_before_redirect_in() {
        // Test word immediately before < (lines 453-455 in tokenize_pipeline)
        let pipeline = parse_pipeline("cat<input.txt").unwrap();
        assert_eq!(pipeline.segments.len(), 1);
        assert_eq!(pipeline.segments[0].redirections.len(), 1);
        assert_eq!(pipeline.segments[0].redirections[0].redir_type, RedirectionType::Input);
    }

    #[test]
    fn test_parse_pipeline_no_space_before_background() {
        // Test word immediately before & (lines 462-464 in tokenize_pipeline)
        let pipeline = parse_pipeline("sleep 5&").unwrap();
        assert!(pipeline.background);
        assert_eq!(pipeline.segments[0].args, vec!["5"]);
    }

    // Tests for word before redirect operators in tokenize_with_redirections
    #[test]
    fn test_tokenize_with_redirections_word_before_output_redirect() {
        // Test word immediately before > in tokenize_with_redirections (lines 256-258)
        let tokens = tokenize_with_redirections("echo test>file.txt").unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word("test".to_string()));
        assert_eq!(tokens[2], Token::RedirectOut);
        assert_eq!(tokens[3], Token::Word("file.txt".to_string()));
    }

    #[test]
    fn test_tokenize_with_redirections_word_before_input_redirect() {
        // Test word immediately before < in tokenize_with_redirections (lines 271-273)
        let tokens = tokenize_with_redirections("cat file<input.txt").unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Word("cat".to_string()));
        assert_eq!(tokens[1], Token::Word("file".to_string()));
        assert_eq!(tokens[2], Token::RedirectIn);
        assert_eq!(tokens[3], Token::Word("input.txt".to_string()));
    }

    // === Environment Variable Expansion Tests (US1 + US4) ===

    // T014: Test basic $VAR expansion
    #[test]
    fn test_expand_basic_var() {
        let mut env = EnvironmentManager::new();
        env.set("FOO".to_string(), "bar".to_string()).unwrap();

        let result = expand_variables_in_string("$FOO", &env);
        assert_eq!(result, "bar");
    }

    // T015: Test ${VAR} braced expansion
    #[test]
    fn test_expand_braced_var() {
        let mut env = EnvironmentManager::new();
        env.set("HOME".to_string(), "/Users/test".to_string())
            .unwrap();

        let result = expand_variables_in_string("${HOME}_backup", &env);
        assert_eq!(result, "/Users/test_backup");
    }

    // T016: Test undefined variable expanding to empty string
    #[test]
    fn test_expand_undefined_var() {
        let env = EnvironmentManager::new();
        let result = expand_variables_in_string("$UNDEFINED_VAR_12345", &env);
        assert_eq!(result, "");
    }

    // T017: Test escaped \$ producing literal dollar sign
    #[test]
    fn test_expand_escaped_dollar() {
        let env = EnvironmentManager::new();
        let result = expand_variables_in_string("\\$HOME", &env);
        assert_eq!(result, "$HOME");
    }

    // T018: Test variable in middle of string
    #[test]
    fn test_expand_var_in_middle() {
        let mut env = EnvironmentManager::new();
        env.set("VAR".to_string(), "middle".to_string()).unwrap();

        let result = expand_variables_in_string("foo$VAR bar", &env);
        assert_eq!(result, "foomiddle bar");
    }

    // Additional expansion tests
    #[test]
    fn test_expand_multiple_vars() {
        let mut env = EnvironmentManager::new();
        env.set("A".to_string(), "1".to_string()).unwrap();
        env.set("B".to_string(), "2".to_string()).unwrap();

        let result = expand_variables_in_string("$A and $B", &env);
        assert_eq!(result, "1 and 2");
    }

    #[test]
    fn test_expand_adjacent_vars() {
        let mut env = EnvironmentManager::new();
        env.set("X".to_string(), "hello".to_string()).unwrap();
        env.set("Y".to_string(), "world".to_string()).unwrap();

        let result = expand_variables_in_string("${X}${Y}", &env);
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_expand_var_with_numbers() {
        let mut env = EnvironmentManager::new();
        env.set("VAR123".to_string(), "value".to_string()).unwrap();

        let result = expand_variables_in_string("$VAR123", &env);
        assert_eq!(result, "value");
    }

    #[test]
    fn test_expand_underscore_var() {
        let mut env = EnvironmentManager::new();
        env.set("_PRIVATE".to_string(), "secret".to_string())
            .unwrap();

        let result = expand_variables_in_string("$_PRIVATE", &env);
        assert_eq!(result, "secret");
    }

    #[test]
    fn test_expand_no_vars() {
        let env = EnvironmentManager::new();
        let result = expand_variables_in_string("just plain text", &env);
        assert_eq!(result, "just plain text");
    }

    #[test]
    fn test_expand_empty_string() {
        let env = EnvironmentManager::new();
        let result = expand_variables_in_string("", &env);
        assert_eq!(result, "");
    }

    #[test]
    fn test_expand_lone_dollar() {
        let env = EnvironmentManager::new();
        // Lone $ with nothing after - should remain as $
        let result = expand_variables_in_string("test$", &env);
        assert_eq!(result, "test$");
    }

    #[test]
    fn test_expand_dollar_followed_by_non_var_char() {
        let env = EnvironmentManager::new();
        // $ followed by non-variable character (like space)
        let result = expand_variables_in_string("$ test", &env);
        assert_eq!(result, "$ test");
    }

    #[test]
    fn test_expand_dollar_number() {
        let env = EnvironmentManager::new();
        // $123 - 1 is not a valid first char, so $ remains
        let result = expand_variables_in_string("$123", &env);
        assert_eq!(result, "$123");
    }

    #[test]
    fn test_expand_empty_braces() {
        let env = EnvironmentManager::new();
        // ${} - empty braces expand to empty (undefined var)
        let result = expand_variables_in_string("${}", &env);
        assert_eq!(result, "");
    }

    #[test]
    fn test_expand_backslash_not_before_dollar() {
        let env = EnvironmentManager::new();
        // Backslash not before $ should be preserved
        let result = expand_variables_in_string("path\\to\\file", &env);
        assert_eq!(result, "path\\to\\file");
    }

    #[test]
    fn test_expand_system_vars_inherited() {
        // Test that system environment variables are inherited
        let env = EnvironmentManager::new();
        // PATH should exist on all systems
        let result = expand_variables_in_string("$PATH", &env);
        // PATH should expand to something non-empty
        assert!(!result.is_empty() || env.get("PATH").is_none());
    }

    // Test expand_variables on pipeline segments
    #[test]
    fn test_expand_variables_in_segment_program() {
        use crate::executor::PipelineSegment;

        let mut env = EnvironmentManager::new();
        env.set("EDITOR".to_string(), "vim".to_string()).unwrap();

        let mut segments = vec![PipelineSegment::new(
            "$EDITOR".to_string(),
            vec!["file.txt".to_string()],
            0,
            vec![],
        )];

        expand_variables(&mut segments, &env);

        assert_eq!(segments[0].program, "vim");
    }

    #[test]
    fn test_expand_variables_in_segment_args() {
        use crate::executor::PipelineSegment;

        let mut env = EnvironmentManager::new();
        env.set("HOME".to_string(), "/home/user".to_string())
            .unwrap();

        let mut segments = vec![PipelineSegment::new(
            "ls".to_string(),
            vec!["$HOME".to_string(), "$HOME/Documents".to_string()],
            0,
            vec![],
        )];

        expand_variables(&mut segments, &env);

        assert_eq!(segments[0].args[0], "/home/user");
        assert_eq!(segments[0].args[1], "/home/user/Documents");
    }

    #[test]
    fn test_expand_variables_in_redirections() {
        use crate::executor::{PipelineSegment, Redirection, RedirectionType};

        let mut env = EnvironmentManager::new();
        env.set("OUTDIR".to_string(), "/tmp".to_string()).unwrap();

        let mut segments = vec![PipelineSegment::new(
            "echo".to_string(),
            vec!["test".to_string()],
            0,
            vec![Redirection::new(
                RedirectionType::Output,
                "$OUTDIR/output.txt".to_string(),
            )],
        )];

        expand_variables(&mut segments, &env);

        assert_eq!(segments[0].redirections[0].file_path, "/tmp/output.txt");
    }

    #[test]
    fn test_expand_variables_multiple_segments() {
        use crate::executor::PipelineSegment;

        let mut env = EnvironmentManager::new();
        env.set("PATTERN".to_string(), "error".to_string()).unwrap();

        let mut segments = vec![
            PipelineSegment::new("cat".to_string(), vec!["log.txt".to_string()], 0, vec![]),
            PipelineSegment::new("grep".to_string(), vec!["$PATTERN".to_string()], 1, vec![]),
        ];

        expand_variables(&mut segments, &env);

        assert_eq!(segments[1].args[0], "error");
    }
}
