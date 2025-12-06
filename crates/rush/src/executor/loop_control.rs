//! Loop control statements (break, continue, return)
//!
//! Implements break, continue, and return statement execution.

use crate::error::{Result, RushError};

/// Loop control signal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopSignal {
    /// Normal execution, no loop control
    None,
    /// Break from loop
    Break,
    /// Continue to next iteration
    Continue,
    /// Return from function
    Return(i32),
}

thread_local! {
    static LOOP_SIGNAL: std::cell::RefCell<LoopSignal> = std::cell::RefCell::new(LoopSignal::None);
}

/// Check if a statement is a break statement
pub fn is_break_statement(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed == "break" || (trimmed.starts_with("break ") && trimmed[6..].trim().parse::<i32>().is_ok())
}

/// Check if a statement is a continue statement
pub fn is_continue_statement(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed == "continue" || (trimmed.starts_with("continue ") && trimmed[9..].trim().parse::<i32>().is_ok())
}

/// Check if a statement is a return statement
pub fn is_return_statement(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed == "return" || (trimmed.starts_with("return ") && trimmed[7..].trim().parse::<i32>().is_ok())
}

/// Execute a break statement
pub fn execute_break(input: &str) -> Result<i32> {
    // Parse optional level parameter (for nested loops)
    let trimmed = input.trim();
    let level: i32 = if trimmed == "break" {
        1
    } else {
        trimmed[5..].trim().parse().unwrap_or(1)
    };

    // Set the break signal (simplified: just break once)
    set_loop_signal(LoopSignal::Break);
    Ok(0)
}

/// Execute a continue statement
pub fn execute_continue(input: &str) -> Result<i32> {
    // Parse optional level parameter
    let trimmed = input.trim();
    let level: i32 = if trimmed == "continue" {
        1
    } else {
        trimmed[8..].trim().parse().unwrap_or(1)
    };

    // Set the continue signal
    set_loop_signal(LoopSignal::Continue);
    Ok(0)
}

/// Execute a return statement
pub fn execute_return(input: &str) -> Result<i32> {
    // Parse optional exit code parameter
    let trimmed = input.trim();
    let exit_code: i32 = if trimmed == "return" {
        0
    } else {
        trimmed[6..].trim().parse().unwrap_or(0)
    };

    // Set the return signal with exit code
    set_loop_signal(LoopSignal::Return(exit_code));
    Ok(exit_code)
}

/// Set the current loop signal
pub fn set_loop_signal(signal: LoopSignal) {
    LOOP_SIGNAL.with(|s| {
        *s.borrow_mut() = signal;
    });
}

/// Get the current loop signal
pub fn get_loop_signal() -> LoopSignal {
    LOOP_SIGNAL.with(|s| *s.borrow())
}

/// Clear the loop signal
pub fn clear_loop_signal() {
    LOOP_SIGNAL.with(|s| {
        *s.borrow_mut() = LoopSignal::None;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_break_statement() {
        assert!(is_break_statement("break"));
        assert!(is_break_statement("break 1"));
        assert!(is_break_statement("  break  "));
        assert!(!is_break_statement("breaking"));
        assert!(!is_break_statement("echo break"));
    }

    #[test]
    fn test_is_continue_statement() {
        assert!(is_continue_statement("continue"));
        assert!(is_continue_statement("continue 1"));
        assert!(is_continue_statement("  continue  "));
        assert!(!is_continue_statement("continuing"));
        assert!(!is_continue_statement("echo continue"));
    }

    #[test]
    fn test_execute_break() {
        clear_loop_signal();
        let result = execute_break("break");
        assert!(result.is_ok());
        assert_eq!(get_loop_signal(), LoopSignal::Break);
    }

    #[test]
    fn test_execute_continue() {
        clear_loop_signal();
        let result = execute_continue("continue");
        assert!(result.is_ok());
        assert_eq!(get_loop_signal(), LoopSignal::Continue);
    }

    #[test]
    fn test_loop_signal_management() {
        clear_loop_signal();
        assert_eq!(get_loop_signal(), LoopSignal::None);

        set_loop_signal(LoopSignal::Break);
        assert_eq!(get_loop_signal(), LoopSignal::Break);

        set_loop_signal(LoopSignal::Continue);
        assert_eq!(get_loop_signal(), LoopSignal::Continue);

        clear_loop_signal();
        assert_eq!(get_loop_signal(), LoopSignal::None);
    }

    #[test]
    fn test_break_with_level() {
        clear_loop_signal();
        let result = execute_break("break 2");
        assert!(result.is_ok());
        assert_eq!(get_loop_signal(), LoopSignal::Break);
    }

    #[test]
    fn test_continue_with_level() {
        clear_loop_signal();
        let result = execute_continue("continue 1");
        assert!(result.is_ok());
        assert_eq!(get_loop_signal(), LoopSignal::Continue);
    }

    #[test]
    fn test_is_return_statement() {
        assert!(is_return_statement("return"));
        assert!(is_return_statement("return 42"));
        assert!(is_return_statement("  return  "));
        assert!(!is_return_statement("returning"));
        assert!(!is_return_statement("echo return"));
    }

    #[test]
    fn test_execute_return() {
        clear_loop_signal();
        let result = execute_return("return 42");
        assert!(result.is_ok());
        assert_eq!(get_loop_signal(), LoopSignal::Return(42));
    }

    #[test]
    fn test_execute_return_default() {
        clear_loop_signal();
        let result = execute_return("return");
        assert!(result.is_ok());
        assert_eq!(get_loop_signal(), LoopSignal::Return(0));
    }
}
