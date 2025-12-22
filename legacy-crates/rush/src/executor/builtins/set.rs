//! Implementation of the `set` builtin command
//!
//! The `set` builtin manages shell variables (local to the shell session).
//!
//! Usage:
//! - `set` - List all variables
//! - `set NAME` - Show specific variable
//! - `set NAME=value` - Set variable
//! - `set NAME="value with spaces"` - Set variable with quoted value

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `set` builtin command
pub fn set(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        // List all variables
        let vars = executor.variable_manager().list();
        for (name, value) in vars {
            println!("{}={}", name, value);
        }
        return Ok(0);
    }

    // Process each argument
    let mut exit_code = 0;
    for arg in args {
        if let Some(pos) = arg.find('=') {
            // set NAME=value
            let name = &arg[..pos];
            let value = &arg[pos + 1..];

            // Strip quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                &value[1..value.len() - 1]
            } else {
                value
            };

            match executor
                .variable_manager_mut()
                .set(name.to_string(), value.to_string())
            {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("rush: set: {}: invalid identifier", name);
                    exit_code = 1;
                }
            }
        } else {
            // set NAME - show specific variable
            if let Some(value) = executor.variable_manager().get(arg) {
                println!("{}={}", arg, value);
            } else {
                eprintln!("rush: set: {}: not set", arg);
                exit_code = 1;
            }
        }
    }

    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    #[test]
    fn test_set_variable() {
        let mut executor = CommandExecutor::new();
        let result = set(&mut executor, &vec!["myvar=value".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("myvar"), Some("value"));
    }

    #[test]
    fn test_set_with_quoted_value() {
        let mut executor = CommandExecutor::new();
        let result = set(&mut executor, &vec!["msg=\"hello world\"".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("msg"), Some("hello world"));
    }

    #[test]
    fn test_set_list_all() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("var1".to_string(), "value1".to_string())
            .unwrap();
        executor
            .variable_manager_mut()
            .set("var2".to_string(), "value2".to_string())
            .unwrap();

        let result = set(&mut executor, &vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_set_show_specific() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("myvar".to_string(), "myvalue".to_string())
            .unwrap();

        let result = set(&mut executor, &vec!["myvar".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_set_show_nonexistent() {
        let mut executor = CommandExecutor::new();
        let result = set(&mut executor, &vec!["nonexistent".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_set_invalid_name() {
        let mut executor = CommandExecutor::new();
        let result = set(&mut executor, &vec!["1invalid=value".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_set_update_existing() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("var".to_string(), "value1".to_string())
            .unwrap();

        set(&mut executor, &vec!["var=value2".to_string()]).unwrap();

        assert_eq!(executor.variable_manager().get("var"), Some("value2"));
    }

    #[test]
    fn test_set_multiple_variables() {
        let mut executor = CommandExecutor::new();
        let result =
            set(&mut executor, &vec!["var1=value1".to_string(), "var2=value2".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("var1"), Some("value1"));
        assert_eq!(executor.variable_manager().get("var2"), Some("value2"));
    }
}
