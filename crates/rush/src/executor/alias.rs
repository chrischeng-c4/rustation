//! Command alias management
//!
//! This module provides alias storage and management for the shell.
//! Aliases are shortcuts that expand to longer commands.
//!
//! Example:
//! ```ignore
//! let mut manager = AliasManager::new();
//! manager.set("ll".to_string(), "ls -la".to_string()).unwrap();
//! assert_eq!(manager.get("ll"), Some("ls -la"));
//! ```

use crate::error::{Result, RushError};
use std::collections::HashMap;

/// Manages command aliases
///
/// Aliases are stored in-memory and are session-only (not persisted to file).
/// Alias names must be valid identifiers (alphanumeric + underscore).
#[derive(Debug, Clone)]
pub struct AliasManager {
    aliases: HashMap<String, String>,
}

impl AliasManager {
    /// Create a new alias manager
    pub fn new() -> Self {
        Self { aliases: HashMap::new() }
    }

    /// Add or update an alias
    ///
    /// # Arguments
    /// * `name` - Alias name (must be valid identifier)
    /// * `value` - Command to expand to
    ///
    /// # Errors
    /// Returns error if alias name is invalid
    pub fn set(&mut self, name: String, value: String) -> Result<()> {
        if !Self::is_valid_name(&name) {
            return Err(RushError::Execution(format!("alias: invalid name: {}", name)));
        }
        self.aliases.insert(name, value);
        Ok(())
    }

    /// Get alias value by name
    ///
    /// Returns `None` if alias doesn't exist
    pub fn get(&self, name: &str) -> Option<&str> {
        self.aliases.get(name).map(|s| s.as_str())
    }

    /// Remove an alias
    ///
    /// Returns `true` if alias was removed, `false` if it didn't exist
    pub fn remove(&mut self, name: &str) -> bool {
        self.aliases.remove(name).is_some()
    }

    /// List all aliases (sorted alphabetically)
    ///
    /// Returns vector of (name, value) tuples
    pub fn list(&self) -> Vec<(&str, &str)> {
        let mut aliases: Vec<_> = self
            .aliases
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        aliases.sort_by_key(|&(name, _)| name);
        aliases
    }

    /// Check if alias name is valid
    ///
    /// Valid names: alphanumeric characters and underscores only
    fn is_valid_name(name: &str) -> bool {
        !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Get number of aliases
    pub fn len(&self) -> usize {
        self.aliases.len()
    }

    /// Check if there are no aliases
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
    }
}

impl Default for AliasManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_manager_new() {
        let manager = AliasManager::new();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_set_alias() {
        let mut manager = AliasManager::new();
        let result = manager.set("ll".to_string(), "ls -la".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_get_alias() {
        let mut manager = AliasManager::new();
        manager.set("ll".to_string(), "ls -la".to_string()).unwrap();
        assert_eq!(manager.get("ll"), Some("ls -la"));
        assert_eq!(manager.get("nonexistent"), None);
    }

    #[test]
    fn test_remove_alias() {
        let mut manager = AliasManager::new();
        manager.set("ll".to_string(), "ls -la".to_string()).unwrap();
        assert!(manager.remove("ll"));
        assert_eq!(manager.get("ll"), None);
        assert!(!manager.remove("ll")); // Already removed
    }

    #[test]
    fn test_list_aliases_sorted() {
        let mut manager = AliasManager::new();
        manager.set("c".to_string(), "cmd3".to_string()).unwrap();
        manager.set("a".to_string(), "cmd1".to_string()).unwrap();
        manager.set("b".to_string(), "cmd2".to_string()).unwrap();

        let list = manager.list();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], ("a", "cmd1"));
        assert_eq!(list[1], ("b", "cmd2"));
        assert_eq!(list[2], ("c", "cmd3"));
    }

    #[test]
    fn test_invalid_alias_name() {
        let mut manager = AliasManager::new();

        // Empty name
        let result = manager.set("".to_string(), "value".to_string());
        assert!(result.is_err());

        // Name with dash
        let result = manager.set("my-alias".to_string(), "value".to_string());
        assert!(result.is_err());

        // Name with space
        let result = manager.set("my alias".to_string(), "value".to_string());
        assert!(result.is_err());

        // Name with special char
        let result = manager.set("my@alias".to_string(), "value".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_alias_names() {
        let mut manager = AliasManager::new();

        // Alphanumeric
        assert!(manager.set("ll".to_string(), "ls".to_string()).is_ok());

        // With numbers
        assert!(manager.set("ll2".to_string(), "ls".to_string()).is_ok());

        // With underscore
        assert!(manager
            .set("my_alias".to_string(), "ls".to_string())
            .is_ok());

        assert_eq!(manager.len(), 3);
    }

    #[test]
    fn test_update_existing_alias() {
        let mut manager = AliasManager::new();
        manager.set("ll".to_string(), "ls -la".to_string()).unwrap();
        assert_eq!(manager.get("ll"), Some("ls -la"));

        // Update it
        manager
            .set("ll".to_string(), "ls -lah".to_string())
            .unwrap();
        assert_eq!(manager.get("ll"), Some("ls -lah"));
        assert_eq!(manager.len(), 1); // Still just one alias
    }

    #[test]
    fn test_alias_value_with_quotes() {
        let mut manager = AliasManager::new();
        manager
            .set("greet".to_string(), "echo \"Hello World\"".to_string())
            .unwrap();
        assert_eq!(manager.get("greet"), Some("echo \"Hello World\""));
    }

    #[test]
    fn test_alias_value_with_pipes() {
        let mut manager = AliasManager::new();
        manager
            .set("lsg".to_string(), "ls | grep".to_string())
            .unwrap();
        assert_eq!(manager.get("lsg"), Some("ls | grep"));
    }

    #[test]
    fn test_empty_alias_value() {
        let mut manager = AliasManager::new();
        manager.set("empty".to_string(), "".to_string()).unwrap();
        assert_eq!(manager.get("empty"), Some(""));
    }
}
