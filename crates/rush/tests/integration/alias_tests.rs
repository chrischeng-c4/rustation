//! Integration tests for alias functionality
//!
//! Tests the complete alias workflow including:
//! - Defining aliases
//! - Listing aliases
//! - Using aliases (expansion)
//! - Removing aliases

use rush::executor::execute::CommandExecutor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_define_and_expand() {
        let mut executor = CommandExecutor::new();

        // Define an alias
        let result = executor.execute("alias ll='ls -la'");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify alias was stored
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -la"));

        // Use the alias - it should expand
        // Note: This will actually try to run ls -la, so we just verify it doesn't error
        let result = executor.execute("ll");
        assert!(result.is_ok());
        // ls -la should succeed (exit code 0)
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_alias_list() {
        let mut executor = CommandExecutor::new();

        // Define multiple aliases
        executor.execute("alias ll='ls -la'").unwrap();
        executor.execute("alias la='ls -A'").unwrap();
        executor.execute("alias grep='grep --color=auto'").unwrap();

        // List all aliases
        let result = executor.execute("alias");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify all aliases exist
        assert_eq!(executor.alias_manager().len(), 3);
    }

    #[test]
    fn test_alias_show_specific() {
        let mut executor = CommandExecutor::new();

        // Define an alias
        executor.execute("alias ll='ls -la'").unwrap();

        // Show specific alias
        let result = executor.execute("alias ll");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_alias_nonexistent() {
        let mut executor = CommandExecutor::new();

        // Try to show nonexistent alias
        let result = executor.execute("alias nonexistent");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_unalias() {
        let mut executor = CommandExecutor::new();

        // Define an alias
        executor.execute("alias ll='ls -la'").unwrap();
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -la"));

        // Remove it
        let result = executor.execute("unalias ll");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify it's gone
        assert_eq!(executor.alias_manager().get("ll"), None);
    }

    #[test]
    fn test_unalias_nonexistent() {
        let mut executor = CommandExecutor::new();

        // Try to remove nonexistent alias
        let result = executor.execute("unalias nonexistent");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_unalias_multiple() {
        let mut executor = CommandExecutor::new();

        // Define multiple aliases
        executor.execute("alias ll='ls -la'").unwrap();
        executor.execute("alias la='ls -A'").unwrap();

        // Remove both
        let result = executor.execute("unalias ll la");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify both are gone
        assert_eq!(executor.alias_manager().get("ll"), None);
        assert_eq!(executor.alias_manager().get("la"), None);
    }

    #[test]
    fn test_alias_with_arguments() {
        let mut executor = CommandExecutor::new();

        // Define an alias for echo
        executor.execute("alias greet='echo Hello'").unwrap();

        // Use alias with additional arguments
        // The alias should expand to "echo Hello world"
        let result = executor.execute("greet world");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_alias_update() {
        let mut executor = CommandExecutor::new();

        // Define an alias
        executor.execute("alias ll='ls -la'").unwrap();
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -la"));

        // Update it
        executor.execute("alias ll='ls -lah'").unwrap();
        assert_eq!(executor.alias_manager().get("ll"), Some("ls -lah"));
    }

    #[test]
    fn test_alias_with_pipes() {
        let mut executor = CommandExecutor::new();

        // Define an alias with a pipe
        executor.execute("alias lsg='ls | grep'").unwrap();
        assert_eq!(executor.alias_manager().get("lsg"), Some("ls | grep"));

        // Note: Actually using this would require providing grep pattern
        // For now, just verify it was stored correctly
    }

    #[test]
    fn test_alias_invalid_name() {
        let mut executor = CommandExecutor::new();

        // Try to define alias with invalid name (contains dash)
        let result = executor.execute("alias my-alias='ls'");
        assert!(result.is_err());
    }
}
