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

pub mod execute;
pub mod job;
pub mod parser;
pub mod pipeline;
pub mod script;

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

    /// Output redirections (>, >>)
    pub redirects: Vec<Redirect>,
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
        }
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
}

impl Pipeline {
    /// Create a new pipeline from segments
    pub fn new(segments: Vec<PipelineSegment>, raw_input: String) -> Self {
        Self { segments, raw_input }
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
}

impl PipelineSegment {
    /// Create a new pipeline segment
    pub fn new(program: String, args: Vec<String>, index: usize) -> Self {
        Self { program, args, index }
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
}
