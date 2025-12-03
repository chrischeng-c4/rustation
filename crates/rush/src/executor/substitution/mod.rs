//! Command substitution ($(...)) support
//!
//! Handles lexical identification, execution, and expansion of command substitution expressions.
//! Implements a 5-component pipeline:
//! 1. Lexer: Identify $(...) regions in input
//! 2. Parser: Extract inner command strings
//! 3. Executor: Run commands and capture output
//! 4. OutputCapture: Collect and process output
//! 5. Expander: Replace $(...) with captured output

pub mod executor;
pub mod expander;
pub mod lexer;

// Re-export public API
pub use expander::{contains_substitution, expand_and_split, expand_substitutions};

/// Represents a substitution token in the input stream
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstitutionToken {
    /// Literal text (non-substitution content)
    Literal(String),
    /// Command substitution expression (contains the command string without $())
    Substitution(String),
}

/// Information about a substitution expression
#[derive(Debug, Clone)]
pub struct SubstitutionInfo {
    /// The command string inside $()
    pub command: String,
    /// Position in the original input where this substitution starts
    pub position: usize,
    /// Whether this substitution is nested inside another
    pub is_nested: bool,
}

/// Error types for substitution operations
#[derive(Debug, Clone)]
pub enum SubstitutionError {
    /// Mismatched parentheses: $(... without closing )
    MismatchedParentheses(String),
    /// Unclosed quote in substitution
    UnclosedQuote(String),
    /// Command execution failed
    ExecutionFailed(String),
    /// Command not found
    CommandNotFound(String),
    /// Non-zero exit code from command
    NonZeroExit { command: String, code: i32 },
    /// Output too large (exceeds size limit)
    OutputTooLarge { size: usize, limit: usize },
    /// UTF-8 conversion error
    InvalidUtf8,
}

impl std::fmt::Display for SubstitutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubstitutionError::MismatchedParentheses(msg) => {
                write!(f, "mismatched parentheses: {}", msg)
            }
            SubstitutionError::UnclosedQuote(msg) => write!(f, "unclosed quote: {}", msg),
            SubstitutionError::ExecutionFailed(msg) => write!(f, "execution failed: {}", msg),
            SubstitutionError::CommandNotFound(cmd) => write!(f, "command not found: {}", cmd),
            SubstitutionError::NonZeroExit { command, code } => {
                write!(f, "command '{}' failed with exit code {}", command, code)
            }
            SubstitutionError::OutputTooLarge { size, limit } => {
                write!(
                    f,
                    "output too large: {} bytes (limit: {} bytes)",
                    size, limit
                )
            }
            SubstitutionError::InvalidUtf8 => write!(f, "invalid UTF-8 in command output"),
        }
    }
}

impl std::error::Error for SubstitutionError {}
