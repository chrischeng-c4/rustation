//! Built-in commands module
//!
//! Handles execution of shell built-ins like `jobs`, `fg`, `bg`, `cd`, etc.

pub mod bg;
pub mod cd;
pub mod fg;
pub mod jobs;

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute a built-in command if it exists
///
/// Returns `Some(Ok(exit_code))` if the command was a built-in and executed.
/// Returns `Some(Err(e))` if the command was a built-in but failed.
/// Returns `None` if the command is not a built-in.
pub fn execute_builtin(
    executor: &mut CommandExecutor,
    command: &str,
    args: &[String],
) -> Option<Result<i32>> {
    match command {
        "cd" => Some(cd::execute(executor, args)),
        "jobs" => Some(jobs::execute(executor, args)),
        "fg" => Some(fg::execute(executor, args)),
        "bg" => Some(bg::execute(executor, args)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_execute_builtin_dispatch() {
        let mut executor = CommandExecutor::new();

        // Test known builtins return Some
        assert!(execute_builtin(&mut executor, "jobs", &[]).is_some());

        // Test unknown command returns None
        assert!(execute_builtin(&mut executor, "not_a_builtin", &[]).is_none());
    }
}
