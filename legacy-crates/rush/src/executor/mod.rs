//! Command execution and job control module
//!
//! Provides:
//! - Command execution (single commands and pipelines)
//! - Job control (fg, bg, jobs)
//! - Script execution
//! - Signal handling
//! - Pipeline support (`cmd1 | cmd2`)
//!
//! # Pipeline Execution (User Story 1: Basic Two-Command Pipeline)
//!
//! The executor supports basic Unix-style pipelines using the `|` operator to chain
//! two commands. Data flows from the first command's stdout to the second command's stdin.
//!
//! ## Example
//!
//! ```ignore
//! use rush::executor::execute::CommandExecutor;
//!
//! let executor = CommandExecutor::new();
//!
//! // Single command
//! executor.execute("ls")?;
//!
//! // Two-command pipeline
//! executor.execute("echo 'hello world' | grep hello")?;
//! ```

pub mod aliases;
pub mod arithmetic;
pub mod arrays;
pub mod brace;
pub mod builtins;
pub mod case_statement;
pub mod command_group;
pub mod conditional;
pub mod execute;
pub mod expansion;
pub mod for_loop;
pub mod function;
pub mod glob;
pub mod job;
pub mod loop_control;
pub mod parser;
pub mod pipeline;
pub mod script;
pub mod subshell;
pub mod substitution;
pub mod test_expr;
pub mod tilde;
pub mod variables;
pub mod while_loop;

use crate::error::Result;
use std::path::PathBuf;

/// A parsed command ready for execution
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// Original user input
    pub raw_input: String,

    /// Command name (first word)
    pub program: String,

    /// Command arguments
    pub args: Vec<String>,

    /// Run in background (ends with &)
    pub background: bool,

    /// Chaining operators (&&, ||, ;, |)
    pub operators: Vec<Operator>,

    /// Output redirections (>, >>) - DEPRECATED: Use redirections field instead
    ///
    /// This field is kept for backward compatibility but should not be used.
    /// Use the `redirections` field for all I/O redirection operations.
    #[deprecated(
        since = "0.2.0",
        note = "Use redirections field for all I/O redirections"
    )]
    pub redirects: Vec<Redirect>,

    /// I/O redirections (>, >>, <) - current implementation
    ///
    /// Supports stdin (<), stdout (>), and stdout append (>>) redirections.
    /// Future: Will support stderr (2>), fd redirection (2>&1), etc.
    pub redirections: Vec<Redirection>,
}

impl Command {
    /// Create a new command
    pub fn new(program: String, args: Vec<String>) -> Self {
        let raw_input = format!("{} {}", program, args.join(" "));
        Self {
            raw_input,
            program,
            args,
            background: false,
            operators: Vec::new(),
            redirects: Vec::new(),
            redirections: Vec::new(),
        }
    }

    /// Validates command including redirections
    pub fn validate(&self) -> crate::error::Result<()> {
        // Validate each redirection
        for redir in &self.redirections {
            redir.validate()?;
        }
        Ok(())
    }
}

/// Command chaining operators
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    /// && - run next if this succeeds
    And,
    /// || - run next if this fails
    Or,
    /// ; - run next regardless
    Sequence,
    /// | - pipe output to next
    Pipe,
}

/// Output redirection
#[derive(Debug, Clone, PartialEq)]
pub struct Redirect {
    /// File descriptor (1=stdout, 2=stderr)
    pub fd: i32,
    /// Redirection mode
    pub mode: RedirectMode,
    /// Target file path
    pub target: PathBuf,
}

/// Redirection mode
#[derive(Debug, Clone, PartialEq)]
pub enum RedirectMode {
    /// > - overwrite
    Overwrite,
    /// >> - append
    Append,
}

/// Type of I/O redirection operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectionType {
    /// Output redirection (>) - truncate and write to file
    Output,
    /// Append redirection (>>) - append to file
    Append,
    /// Input redirection (<) - read from file
    Input,
    /// Stderr redirection (2> or 2>>) - redirect stderr to file
    /// bool flag: false = 2> (truncate), true = 2>> (append)
    Stderr(bool),
    /// Stderr to stdout (2>&1) - redirect stderr to same destination as stdout
    StderrToStdout,
    /// Stdout to stderr (1>&2) - redirect stdout to same destination as stderr
    StdoutToStderr,
    /// Heredoc (<<) - inline document as stdin
    Heredoc,
    /// Heredoc with tab stripping (<<-) - inline document with leading tabs stripped
    HeredocStrip,
    /// Here-string (<<<) - single string as stdin
    HereString,
}

/// A single redirection operation with type and target file path
#[derive(Debug, Clone, PartialEq)]
pub struct Redirection {
    /// Type of redirection (>, >>, or <)
    pub redir_type: RedirectionType,
    /// File path for redirection target/source (or delimiter for heredocs)
    pub file_path: String,
    /// Content for heredoc redirections
    pub heredoc_content: Option<String>,
}

impl Redirection {
    /// Creates a new redirection
    pub fn new(redir_type: RedirectionType, file_path: String) -> Self {
        Self { redir_type, file_path, heredoc_content: None }
    }

    /// Creates a new heredoc redirection with content
    pub fn new_heredoc(delimiter: String, content: String, _strip_tabs: bool) -> Self {
        Self {
            redir_type: if _strip_tabs {
                RedirectionType::HeredocStrip
            } else {
                RedirectionType::Heredoc
            },
            file_path: delimiter,
            heredoc_content: Some(content),
        }
    }

    /// Creates a new here-string redirection with content
    /// The content is the string to pass as stdin (trailing newline added during execution)
    pub fn new_herestring(content: String) -> Self {
        Self {
            redir_type: RedirectionType::HereString,
            file_path: String::new(), // Not used for here-strings
            heredoc_content: Some(content),
        }
    }

    /// Validates that the redirection is well-formed
    pub fn validate(&self) -> crate::error::Result<()> {
        use crate::error::RushError;
        // Heredocs use file_path as delimiter, which can be empty for some edge cases
        // StderrToStdout and StdoutToStderr don't need file paths
        // HereString stores content in heredoc_content, not file_path
        if self.file_path.is_empty()
            && !matches!(
                self.redir_type,
                RedirectionType::StderrToStdout
                    | RedirectionType::StdoutToStderr
                    | RedirectionType::HereString
            )
        {
            return Err(RushError::Execution("Empty file path for redirection".to_string()));
        }
        Ok(())
    }
}

/// A complete pipeline parsed from user input
///
/// Supports single commands and multi-command pipelines (User Stories 1 & 2).
///
/// Example: "ls -la | grep txt | wc -l" becomes:
/// ```ignore
/// Pipeline {
///     segments: [
///         PipelineSegment { program: "ls", args: ["-la"], index: 0 },
///         PipelineSegment { program: "grep", args: ["txt"], index: 1 },
///         PipelineSegment { program: "wc", args: ["-l"], index: 2 },
///     ],
///     raw_input: "ls -la | grep txt | wc -l",
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    /// Individual commands in the pipeline (1 to N commands)
    pub segments: Vec<PipelineSegment>,

    /// Original user input for error messages and logging
    pub raw_input: String,

    /// Run in background (ends with &)
    pub background: bool,
}

impl Pipeline {
    /// Create a new pipeline from segments
    pub fn new(segments: Vec<PipelineSegment>, raw_input: String, background: bool) -> Self {
        Self { segments, raw_input, background }
    }

    /// Number of commands in the pipeline
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Check if pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Validate pipeline structure
    ///
    /// Ensures pipeline has at least one command and all segments are valid.
    /// Supports any number of commands (User Stories 1 & 2).
    ///
    /// Returns Ok(()) if valid, Err with reason if invalid.
    pub fn validate(&self) -> Result<()> {
        if self.is_empty() {
            return Err(crate::error::RushError::Execution("Empty pipeline".to_string()));
        }

        for segment in &self.segments {
            segment.validate()?;
        }

        Ok(())
    }
}

/// One command in a pipeline
///
/// Example: In "ls -la | grep txt", the first segment is:
/// ```ignore
/// PipelineSegment {
///     program: "ls",
///     args: ["-la"],
///     index: 0,
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineSegment {
    /// Command name (e.g., "ls", "grep")
    pub program: String,

    /// Command arguments (e.g., ["-la"], ["txt"])
    pub args: Vec<String>,

    /// Position in pipeline (0-indexed)
    /// First command is 0, second is 1
    pub index: usize,

    /// Heredoc contents mapped by delimiter
    pub heredoc_contents: std::collections::HashMap<String, String>,
}

impl PipelineSegment {
    /// Create a new pipeline segment
    pub fn new(program: String, args: Vec<String>, index: usize) -> Self {
        Self { program, args, index, heredoc_contents: std::collections::HashMap::new() }
    }

    /// Add heredoc content for a delimiter
    pub fn add_heredoc_content(&mut self, delimiter: String, content: String) {
        self.heredoc_contents.insert(delimiter, content);
    }

    /// Validate segment
    pub fn validate(&self) -> Result<()> {
        if self.program.is_empty() {
            return Err(crate::error::RushError::Execution(format!(
                "Empty program at position {}",
                self.index
            )));
        }
        Ok(())
    }

    /// Check if this is the first segment in a pipeline
    pub fn is_first(&self) -> bool {
        self.index == 0
    }

    /// Check if this is the last segment in a pipeline
    pub fn is_last(&self, pipeline_len: usize) -> bool {
        self.index == pipeline_len - 1
    }
}

// Conditional control flow structures (Feature 017)

/// Control flow keywords for parsing conditionals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    /// `if` keyword
    If,
    /// `then` keyword
    Then,
    /// `elif` keyword
    Elif,
    /// `else` keyword
    Else,
    /// `fi` keyword
    Fi,
}

impl Keyword {
    /// Convert a string to a keyword if it matches
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "if" => Some(Keyword::If),
            "then" => Some(Keyword::Then),
            "elif" => Some(Keyword::Elif),
            "else" => Some(Keyword::Else),
            "fi" => Some(Keyword::Fi),
            _ => None,
        }
    }

    /// Get the string representation of a keyword
    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::If => "if",
            Keyword::Then => "then",
            Keyword::Elif => "elif",
            Keyword::Else => "else",
            Keyword::Fi => "fi",
        }
    }
}

/// A sequence of commands separated by `;` or newlines
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundList {
    /// Commands in the list
    pub commands: Vec<Command>,
}

impl CompoundList {
    /// Create a new compound list with commands
    pub fn new(commands: Vec<Command>) -> Self {
        Self { commands }
    }

    /// Create an empty compound list (no-op)
    pub fn empty() -> Self {
        Self { commands: Vec::new() }
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get the number of commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }
}

/// An `elif` clause in an if statement
#[derive(Debug, Clone, PartialEq)]
pub struct ElifClause {
    /// Condition command(s) for the elif
    pub condition: Box<CompoundList>,
    /// Commands to execute if condition succeeds (exit code 0)
    pub then_block: Box<CompoundList>,
    /// Raw then block string (for proper pipe and redirection support)
    /// Phase 3: Used to execute block directly, supporting pipes and redirections
    pub then_block_raw: String,
}

impl ElifClause {
    /// Create a new elif clause
    pub fn new(condition: CompoundList, then_block: CompoundList) -> Self {
        Self {
            condition: Box::new(condition),
            then_block: Box::new(then_block),
            then_block_raw: String::new(),
        }
    }

    /// Create a new elif clause with raw then block string (Phase 3+: for pipe support)
    pub fn new_with_raw_body(
        condition: CompoundList,
        then_block: CompoundList,
        then_block_raw: String,
    ) -> Self {
        Self { condition: Box::new(condition), then_block: Box::new(then_block), then_block_raw }
    }
}

/// An `if` statement with optional `elif` and `else` clauses
#[derive(Debug, Clone, PartialEq)]
pub struct IfBlock {
    /// Condition command(s) for the if
    pub condition: Box<CompoundList>,
    /// Commands to execute if condition succeeds (exit code 0)
    pub then_block: Box<CompoundList>,
    /// Raw then block string (for proper pipe and redirection support)
    /// Phase 3: Used to execute block directly, supporting pipes and redirections
    pub then_block_raw: String,
    /// Optional elif clauses (each with its own condition and then block)
    pub elif_clauses: Vec<ElifClause>,
    /// Optional else block (executed if no conditions succeed)
    pub else_block: Option<Box<CompoundList>>,
    /// Raw else block string (for proper pipe and redirection support)
    /// Phase 3: Used to execute block directly, supporting pipes and redirections
    pub else_block_raw: String,
}

impl IfBlock {
    /// Create a new if block with just condition and then block
    pub fn new(condition: CompoundList, then_block: CompoundList) -> Self {
        Self {
            condition: Box::new(condition),
            then_block: Box::new(then_block),
            then_block_raw: String::new(),
            elif_clauses: Vec::new(),
            else_block: None,
            else_block_raw: String::new(),
        }
    }

    /// Create a new if block with raw body strings (Phase 3+: for pipe support)
    pub fn new_with_raw_body(
        condition: CompoundList,
        then_block: CompoundList,
        then_block_raw: String,
    ) -> Self {
        Self {
            condition: Box::new(condition),
            then_block: Box::new(then_block),
            then_block_raw,
            elif_clauses: Vec::new(),
            else_block: None,
            else_block_raw: String::new(),
        }
    }

    /// Add an elif clause
    pub fn add_elif(&mut self, elif_clause: ElifClause) {
        self.elif_clauses.push(elif_clause);
    }

    /// Set the else block
    pub fn set_else(&mut self, else_block: CompoundList) {
        self.else_block = Some(Box::new(else_block));
    }

    /// Set the else block with raw body string (Phase 3+: for pipe support)
    pub fn set_else_with_raw(&mut self, else_block: CompoundList, else_block_raw: String) {
        self.else_block = Some(Box::new(else_block));
        self.else_block_raw = else_block_raw;
    }
}

/// A `for` loop with list-based iteration
#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop {
    /// Loop variable name (identifier)
    pub variable: String,
    /// Word list to iterate over
    pub word_list: Vec<String>,
    /// Commands to execute in each iteration
    pub body: Box<CompoundList>,
    /// Raw body string (for proper pipe and redirection support)
    /// Phase 3: Used to execute body directly, supporting pipes and redirections
    pub body_raw: String,
}

impl ForLoop {
    /// Create a new for loop
    pub fn new(variable: String, word_list: Vec<String>, body: CompoundList) -> Self {
        Self { variable, word_list, body: Box::new(body), body_raw: String::new() }
    }

    /// Create a for loop with raw body string (Phase 3+: for pipe support)
    pub fn new_with_raw_body(
        variable: String,
        word_list: Vec<String>,
        body: CompoundList,
        body_raw: String,
    ) -> Self {
        Self { variable, word_list, body: Box::new(body), body_raw }
    }

    /// Create a for loop with empty word list (uses positional parameters)
    pub fn new_with_positional(variable: String, body: CompoundList) -> Self {
        Self {
            variable,
            word_list: Vec::new(), // Empty means use $@
            body: Box::new(body),
            body_raw: String::new(),
        }
    }
}

/// While loop: repeat commands while condition is true
#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    /// Condition to evaluate (command that returns exit code)
    pub condition: Box<CompoundList>,
    /// Commands to execute while condition is true
    pub body: Box<CompoundList>,
    /// Raw body string (for proper pipe and redirection support)
    /// Phase 3: Used to execute body directly, supporting pipes and redirections
    pub body_raw: String,
}

impl WhileLoop {
    /// Create a new while loop
    pub fn new(condition: CompoundList, body: CompoundList) -> Self {
        Self { condition: Box::new(condition), body: Box::new(body), body_raw: String::new() }
    }

    /// Create a new while loop with raw body string (Phase 3+: for pipe support)
    pub fn new_with_raw_body(
        condition: CompoundList,
        body: CompoundList,
        body_raw: String,
    ) -> Self {
        Self { condition: Box::new(condition), body: Box::new(body), body_raw }
    }
}

/// Until loop: repeat commands until condition becomes true
#[derive(Debug, Clone, PartialEq)]
pub struct UntilLoop {
    /// Condition to evaluate (command that returns exit code)
    pub condition: Box<CompoundList>,
    /// Commands to execute until condition becomes true
    pub body: Box<CompoundList>,
    /// Raw body string (for proper pipe and redirection support)
    /// Phase 3: Used to execute body directly, supporting pipes and redirections
    pub body_raw: String,
}

impl UntilLoop {
    /// Create a new until loop
    pub fn new(condition: CompoundList, body: CompoundList) -> Self {
        Self { condition: Box::new(condition), body: Box::new(body), body_raw: String::new() }
    }

    /// Create a new until loop with raw body string (Phase 3+: for pipe support)
    pub fn new_with_raw_body(
        condition: CompoundList,
        body: CompoundList,
        body_raw: String,
    ) -> Self {
        Self { condition: Box::new(condition), body: Box::new(body), body_raw }
    }
}

/// Case statement: pattern matching with multiple branches
#[derive(Debug, Clone, PartialEq)]
pub struct CaseStatement {
    /// Value to match against patterns
    pub value: String,
    /// List of patterns with corresponding commands
    pub patterns: Vec<CasePattern>,
}

/// Shell function definition
#[derive(Debug, Clone, PartialEq)]
pub struct ShellFunction {
    /// Function name
    pub name: String,
    /// Commands in function body
    pub body: CompoundList,
    /// Function parameters (for future use)
    pub parameters: Vec<String>,
}

/// Subshell: command execution in a separate process
#[derive(Debug, Clone, PartialEq)]
pub struct Subshell {
    /// Commands to execute in subshell
    pub body: CompoundList,
}

/// Command group: curly-brace delimited commands in current shell
#[derive(Debug, Clone, PartialEq)]
pub struct CommandGroup {
    /// Commands to execute in the group
    pub body: CompoundList,
}

/// A single pattern case with commands
#[derive(Debug, Clone, PartialEq)]
pub struct CasePattern {
    /// Patterns to match (can be multiple with |)
    pub patterns: Vec<String>,
    /// Commands to execute if pattern matches
    pub body: CompoundList,
    /// Whether to fall through to next pattern (;& operator)
    pub fall_through: bool,
    /// Whether to test next pattern without executing (;;& operator)
    pub test_next: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_new() {
        let cmd = Command::new("ls".to_string(), vec!["-la".to_string()]);
        assert_eq!(cmd.program, "ls");
        assert_eq!(cmd.args, vec!["-la"]);
        assert!(!cmd.background);
        assert!(cmd.operators.is_empty());
        assert!(cmd.redirects.is_empty());
    }

    #[test]
    fn test_command_clone() {
        let cmd1 = Command::new("echo".to_string(), vec!["hello".to_string()]);
        let cmd2 = cmd1.clone();
        assert_eq!(cmd1, cmd2);
    }

    #[test]
    fn test_operator_variants() {
        let ops = vec![
            Operator::And,
            Operator::Or,
            Operator::Sequence,
            Operator::Pipe,
        ];
        assert_eq!(ops.len(), 4);
    }

    #[test]
    fn test_command_with_background() {
        let mut cmd = Command::new("sleep".to_string(), vec!["30".to_string()]);
        cmd.background = true;

        assert!(cmd.background);
        assert_eq!(cmd.program, "sleep");
    }

    #[test]
    fn test_command_raw_input() {
        let cmd = Command::new(
            "git".to_string(),
            vec!["commit".to_string(), "-m".to_string(), "test".to_string()],
        );

        assert!(cmd.raw_input.contains("git"));
        assert!(cmd.raw_input.contains("commit"));
    }

    #[test]
    fn test_redirect_mode() {
        let mode1 = RedirectMode::Overwrite;
        let mode2 = RedirectMode::Append;

        assert_ne!(mode1, mode2);
    }

    #[test]
    fn test_redirect_struct() {
        let redirect =
            Redirect { fd: 1, mode: RedirectMode::Overwrite, target: PathBuf::from("output.txt") };

        assert_eq!(redirect.fd, 1);
        assert_eq!(redirect.mode, RedirectMode::Overwrite);
        assert_eq!(redirect.target, PathBuf::from("output.txt"));
    }

    #[test]
    fn test_command_with_operators() {
        let mut cmd = Command::new("ls".to_string(), vec![]);
        cmd.operators.push(Operator::Pipe);

        assert_eq!(cmd.operators.len(), 1);
        assert_eq!(cmd.operators[0], Operator::Pipe);
    }

    #[test]
    fn test_command_with_redirects() {
        let mut cmd = Command::new("echo".to_string(), vec!["test".to_string()]);
        cmd.redirects.push(Redirect {
            fd: 1,
            mode: RedirectMode::Overwrite,
            target: PathBuf::from("out.txt"),
        });

        assert_eq!(cmd.redirects.len(), 1);
        assert_eq!(cmd.redirects[0].fd, 1);
    }

    #[test]
    fn test_command_no_args() {
        let cmd = Command::new("pwd".to_string(), vec![]);

        assert_eq!(cmd.program, "pwd");
        assert!(cmd.args.is_empty());
        assert!(!cmd.background);
    }

    #[test]
    fn test_command_many_args() {
        let args: Vec<String> = (0..100).map(|i| format!("arg{}", i)).collect();
        let cmd = Command::new("test".to_string(), args.clone());

        assert_eq!(cmd.args.len(), 100);
        assert_eq!(cmd.args, args);
    }

    #[test]
    fn test_operator_equality() {
        assert_eq!(Operator::And, Operator::And);
        assert_ne!(Operator::And, Operator::Or);
        assert_ne!(Operator::Pipe, Operator::Sequence);
    }

    #[test]
    fn test_redirect_clone() {
        let redirect1 =
            Redirect { fd: 2, mode: RedirectMode::Append, target: PathBuf::from("err.log") };
        let redirect2 = redirect1.clone();

        assert_eq!(redirect1, redirect2);
    }

    #[test]
    fn test_redirection_type_variants() {
        let output = RedirectionType::Output;
        let append = RedirectionType::Append;
        let input = RedirectionType::Input;

        assert_ne!(output, append);
        assert_ne!(output, input);
        assert_ne!(append, input);
    }

    #[test]
    fn test_redirection_new() {
        let redir = Redirection::new(RedirectionType::Output, "output.txt".to_string());
        assert_eq!(redir.redir_type, RedirectionType::Output);
        assert_eq!(redir.file_path, "output.txt");
    }

    #[test]
    fn test_redirection_validate_valid() {
        let redir = Redirection::new(RedirectionType::Output, "file.txt".to_string());
        assert!(redir.validate().is_ok());
    }

    #[test]
    fn test_redirection_validate_empty_path() {
        let redir = Redirection::new(RedirectionType::Output, "".to_string());
        assert!(redir.validate().is_err());
    }

    #[test]
    fn test_redirection_clone() {
        let redir1 = Redirection::new(RedirectionType::Append, "log.txt".to_string());
        let redir2 = redir1.clone();
        assert_eq!(redir1, redir2);
    }
}
