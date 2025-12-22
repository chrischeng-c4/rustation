//! rush - A modern, fast, fish-like shell written in Rust
//!
//! This library provides the core functionality for the rush shell, including:
//! - REPL (Read-Eval-Print Loop) with line editing
//! - Command execution and job control
//! - Command history management
//! - Tab completion engine
//! - Syntax highlighting
//! - Configuration management
//!
//! # Example
//!
//! ```no_run
//! use rush::repl::Repl;
//!
//! fn main() -> rush::Result<()> {
//!     let mut repl = Repl::new()?;
//!     let exit_code = repl.run()?;
//!     std::process::exit(exit_code);
//! }
//! ```

// Public modules
pub mod cli;
pub mod completion;
pub mod config;
pub mod executor;
pub mod history;
pub mod repl;

// Re-export commonly used types
pub use config::Config;
pub use executor::Command;
pub use history::HistoryEntry;
pub use repl::Repl;

/// Error types for rush
pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum RushError {
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        #[error("Configuration error: {0}")]
        Config(String),

        #[error("History error: {0}")]
        History(String),

        #[error("Execution error: {0}")]
        Execution(String),

        #[error("REPL error: {0}")]
        Repl(String),

        #[error("Redirection error: {0}")]
        Redirection(String),

        #[error("Syntax error: {0}")]
        Syntax(String),

        #[error("Exit requested with code: {0}")]
        ExitRequest(i32),

        // Trap builtin errors
        #[error("trap: invalid signal specification: {0}")]
        InvalidSignal(String),

        #[error("trap: cannot trap {0}: signal cannot be caught")]
        UncatchableSignal(String),

        #[error("trap: trap already exists for signal {0} (use 'trap \"\" {0}' to clear first)")]
        DuplicateTrap(String),

        #[error("trap: empty command (use 'trap \"\" SIGNAL' to clear)")]
        EmptyCommand,

        // Test command errors
        #[error("test: invalid operator: {0}")]
        InvalidOperator(String),

        #[error("test: type mismatch: {0}")]
        TypeMismatch(String),

        #[error("test: invalid pattern: {0}")]
        InvalidPattern(String),

        #[error("test: pattern too long (max 10KB)")]
        PatternTooLong,

        #[error("test: file test failed: {0}")]
        FileTestFailed(String),
    }

    pub type Result<T> = std::result::Result<T, RushError>;
}

// Re-export error types
pub use error::{Result, RushError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_formatting() {
        let err = RushError::Config("Missing file".to_string());
        assert_eq!(format!("{}", err), "Configuration error: Missing file");

        let err = RushError::Execution("Command failed".to_string());
        assert_eq!(format!("{}", err), "Execution error: Command failed");
    }
}
