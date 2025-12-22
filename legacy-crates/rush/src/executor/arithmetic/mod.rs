//! Arithmetic expansion module for rush shell.
//!
//! This module provides support for:
//! - Arithmetic expansion `$((expression))`
//! - The `let` builtin command
//! - Arithmetic command `(( expression ))`
//!
//! # Example
//! ```text
//! echo $((5 + 3))      # Output: 8
//! x=10; echo $((x * 2)) # Output: 20
//! let x=5+3            # Sets x=8
//! (( x > 5 )) && echo yes
//! ```

pub mod evaluator;
pub mod expander;
pub mod lexer;
pub mod parser;

pub use evaluator::evaluate;
pub use expander::{contains_arithmetic, expand_arithmetic};
pub use lexer::{Lexer, Token};
pub use parser::{parse, Expr};

use thiserror::Error;

/// Errors that can occur during arithmetic operations.
#[derive(Debug, Error, PartialEq)]
pub enum ArithmeticError {
    #[error("division by zero")]
    DivisionByZero,

    #[error("syntax error: {0}")]
    SyntaxError(String),

    #[error("invalid number: {0}")]
    InvalidNumber(String),

    #[error("unexpected token: {0}")]
    UnexpectedToken(String),

    #[error("unexpected end of expression")]
    UnexpectedEnd,

    #[error("invalid assignment target")]
    InvalidAssignmentTarget,
}

pub type Result<T> = std::result::Result<T, ArithmeticError>;
