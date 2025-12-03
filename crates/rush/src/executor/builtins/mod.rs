//! Built-in commands module
//!
//! Handles execution of shell built-ins like `jobs`, `fg`, `bg`, `cd`, `echo`, `test`, `alias`, etc.

pub mod alias;
pub mod bg;
pub mod bracket;
pub mod cd;
pub mod echo;
pub mod false_cmd;
pub mod fg;
pub mod jobs;
pub mod printf;
pub mod pwd;
pub mod test;
pub mod true_cmd;
pub mod type_cmd;
pub mod unalias;

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
        "echo" => Some(echo::execute(executor, args)),
        "true" => Some(true_cmd::execute(executor, args)),
        "false" => Some(false_cmd::execute(executor, args)),
        "test" => Some(test::execute(executor, args)),
        "[" => Some(bracket::execute(executor, args)),
        "printf" => Some(printf::execute(executor, args)),
        "pwd" => Some(pwd::execute(executor, args)),
        "type" => Some(type_cmd::execute(executor, args)),
        "alias" => Some(alias::execute(executor, args)),
        "unalias" => Some(unalias::execute(executor, args)),
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
        assert!(execute_builtin(&mut executor, "echo", &[]).is_some());
        assert!(execute_builtin(&mut executor, "true", &[]).is_some());
        assert!(execute_builtin(&mut executor, "false", &[]).is_some());

        // Test unknown command returns None
        assert!(execute_builtin(&mut executor, "not_a_builtin", &[]).is_none());
    }

    #[test]
    fn test_echo_builtin() {
        let mut executor = CommandExecutor::new();
        let result = execute_builtin(&mut executor, "echo", &["hello".to_string()]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().unwrap(), 0);
    }

    #[test]
    fn test_true_builtin() {
        let mut executor = CommandExecutor::new();
        let result = execute_builtin(&mut executor, "true", &[]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().unwrap(), 0);
    }

    #[test]
    fn test_false_builtin() {
        let mut executor = CommandExecutor::new();
        let result = execute_builtin(&mut executor, "false", &[]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().unwrap(), 1);
    }
}
