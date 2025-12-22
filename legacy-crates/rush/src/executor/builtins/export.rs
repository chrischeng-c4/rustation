//! Implementation of the `export` builtin command
//!
//! The `export` builtin exports variables to subshells/child processes.
//!
//! Usage:
//! - `export` - List all exported variables
//! - `export NAME` - Mark existing variable as exported
//! - `export NAME=value` - Set and export variable
//! - `export NAME="value with spaces"` - Export with quoted value

use crate::error::Result;
use crate::executor::execute::CommandExecutor;

/// Execute the `export` builtin command
pub fn export(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        // List all exported variables
        let vars = executor.variable_manager().list_exported();
        for (name, value) in vars {
            println!("export {}={}", name, value);
        }
        return Ok(0);
    }

    // Process each argument
    let mut exit_code = 0;
    for arg in args {
        if let Some(pos) = arg.find('=') {
            // export NAME=value
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

            // Set the variable
            match executor
                .variable_manager_mut()
                .set(name.to_string(), value.to_string())
            {
                Ok(_) => {
                    // Then export it
                    if let Err(e) = executor.variable_manager_mut().export(name) {
                        eprintln!("rush: export: {}", e);
                        exit_code = 1;
                    }
                }
                Err(_) => {
                    eprintln!("rush: export: {}: invalid identifier", name);
                    exit_code = 1;
                }
            }
        } else {
            // export NAME - mark existing variable as exported
            match executor.variable_manager_mut().export(arg) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("rush: export: {}", e);
                    exit_code = 1;
                }
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
    fn test_export_new_variable() {
        let mut executor = CommandExecutor::new();
        let result = export(&mut executor, &vec!["myvar=value".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("myvar"), Some("value"));
        assert!(executor.variable_manager().is_exported("myvar"));
    }

    #[test]
    fn test_export_existing_variable() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("myvar".to_string(), "value".to_string())
            .unwrap();
        assert!(!executor.variable_manager().is_exported("myvar"));

        let result = export(&mut executor, &vec!["myvar".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert!(executor.variable_manager().is_exported("myvar"));
    }

    #[test]
    fn test_export_nonexistent_variable() {
        let mut executor = CommandExecutor::new();
        let result = export(&mut executor, &vec!["nonexistent".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_export_list() {
        let mut executor = CommandExecutor::new();
        executor
            .variable_manager_mut()
            .set("local".to_string(), "value1".to_string())
            .unwrap();
        executor
            .variable_manager_mut()
            .set("exported".to_string(), "value2".to_string())
            .unwrap();
        executor.variable_manager_mut().export("exported").unwrap();

        let result = export(&mut executor, &vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        // Only exported variable should be listed
        assert_eq!(executor.variable_manager().list_exported().len(), 1);
    }

    #[test]
    fn test_export_with_quoted_value() {
        let mut executor = CommandExecutor::new();
        let result = export(&mut executor, &vec!["msg=\"hello world\"".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(executor.variable_manager().get("msg"), Some("hello world"));
        assert!(executor.variable_manager().is_exported("msg"));
    }

    #[test]
    fn test_export_invalid_name() {
        let mut executor = CommandExecutor::new();
        let result = export(&mut executor, &vec!["1invalid=value".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_export_multiple_variables() {
        let mut executor = CommandExecutor::new();
        let result =
            export(&mut executor, &vec!["var1=value1".to_string(), "var2=value2".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert!(executor.variable_manager().is_exported("var1"));
        assert!(executor.variable_manager().is_exported("var2"));
    }
}
