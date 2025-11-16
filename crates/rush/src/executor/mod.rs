//! Command execution and job control module
//!
//! Provides:
//! - Command execution
//! - Job control (fg, bg, jobs)
//! - Script execution
//! - Signal handling

pub mod execute;
pub mod job;
pub mod parser;
pub mod script;

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
