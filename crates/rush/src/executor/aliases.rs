//! Alias management for shell command shortcuts
//!
//! Provides functionality to create, manage, and expand shell aliases.
//!
//! # Example
//! ```ignore
//! let mut manager = AliasManager::new();
//! manager.add("ll", "ls -la");
//! assert_eq!(manager.expand("ll"), "ls -la");
//! assert_eq!(manager.expand("ll foo"), "ls -la foo");
//! ```

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Maximum depth for alias expansion to prevent infinite loops
const MAX_EXPANSION_DEPTH: usize = 10;

/// Manages shell aliases
#[derive(Debug, Default)]
pub struct AliasManager {
    /// Map of alias names to their expanded values
    aliases: HashMap<String, String>,
}

impl AliasManager {
    /// Create a new empty AliasManager
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }

    /// Add or update an alias
    ///
    /// # Arguments
    /// * `name` - The alias name (e.g., "ll")
    /// * `value` - The expanded command (e.g., "ls -la")
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err` if the alias name is invalid
    pub fn add(&mut self, name: &str, value: &str) -> Result<(), String> {
        // Validate alias name
        if !is_valid_alias_name(name) {
            return Err(format!("invalid alias name: {}", name));
        }

        self.aliases.insert(name.to_string(), value.to_string());
        Ok(())
    }

    /// Get an alias value by name
    pub fn get(&self, name: &str) -> Option<&str> {
        self.aliases.get(name).map(|s| s.as_str())
    }

    /// Remove an alias by name
    ///
    /// # Returns
    /// * `true` if the alias was removed
    /// * `false` if the alias didn't exist
    pub fn remove(&mut self, name: &str) -> bool {
        self.aliases.remove(name).is_some()
    }

    /// List all aliases as (name, value) pairs
    pub fn list(&self) -> Vec<(&str, &str)> {
        let mut result: Vec<_> = self
            .aliases
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        result.sort_by(|a, b| a.0.cmp(b.0));
        result
    }

    /// Check if an alias exists
    pub fn contains(&self, name: &str) -> bool {
        self.aliases.contains_key(name)
    }

    /// Get the number of aliases
    pub fn len(&self) -> usize {
        self.aliases.len()
    }

    /// Check if there are no aliases
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
    }

    /// Expand aliases in a command string
    ///
    /// Only expands the first word if it's an alias.
    /// Arguments are preserved and appended to the expanded command.
    ///
    /// # Arguments
    /// * `input` - The command line to expand
    ///
    /// # Returns
    /// The expanded command string, or the original if no alias matches
    pub fn expand(&self, input: &str) -> String {
        self.expand_with_depth(input, 0)
    }

    /// Internal expansion with depth tracking to prevent infinite recursion
    fn expand_with_depth(&self, input: &str, depth: usize) -> String {
        if depth >= MAX_EXPANSION_DEPTH {
            eprintln!("alias: maximum expansion depth exceeded (circular alias?)");
            return input.to_string();
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return input.to_string();
        }

        // Split into first word and rest
        let (first_word, rest) = split_first_word(trimmed);

        // Check if first word is an alias
        if let Some(expanded) = self.aliases.get(first_word) {
            // Expand the alias value recursively (it might contain another alias)
            let expanded_value = self.expand_with_depth(expanded, depth + 1);

            // Append any remaining arguments
            if rest.is_empty() {
                expanded_value
            } else {
                format!("{} {}", expanded_value, rest)
            }
        } else {
            input.to_string()
        }
    }

    /// Get the alias file path
    pub fn alias_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/rush/aliases")
    }

    /// Save aliases to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::alias_file_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create config directory: {}", e))?;
        }

        let mut file =
            fs::File::create(&path).map_err(|e| format!("failed to create alias file: {}", e))?;

        for (name, value) in self.list() {
            writeln!(file, "{}='{}'", name, value.replace('\'', "'\\''"))
                .map_err(|e| format!("failed to write alias: {}", e))?;
        }

        Ok(())
    }

    /// Load aliases from file
    pub fn load(&mut self) -> Result<(), String> {
        let path = Self::alias_file_path();

        if !path.exists() {
            return Ok(()); // No alias file yet, that's fine
        }

        let file =
            fs::File::open(&path).map_err(|e| format!("failed to open alias file: {}", e))?;

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|e| format!("failed to read alias file: {}", e))?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            // Parse: name='value' or name=value
            if let Some((name, value)) = parse_alias_line(line) {
                // Silently skip invalid names during load
                let _ = self.add(&name, &value);
            }
        }

        Ok(())
    }
}

/// Check if a string is a valid alias name
///
/// Valid names: alphanumeric and underscore, not starting with a digit
fn is_valid_alias_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    // First char must be letter or underscore
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Split a string into first word and rest
fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();
    match s.find(char::is_whitespace) {
        Some(pos) => (&s[..pos], s[pos..].trim_start()),
        None => (s, ""),
    }
}

/// Parse an alias line from the config file
///
/// Supports formats:
/// - `name='value'`
/// - `name="value"`
/// - `name=value`
fn parse_alias_line(line: &str) -> Option<(String, String)> {
    let equals_pos = line.find('=')?;
    let name = line[..equals_pos].trim().to_string();
    let mut value = line[equals_pos + 1..].trim();

    // Remove surrounding quotes if present
    if (value.starts_with('\'') && value.ends_with('\''))
        || (value.starts_with('"') && value.ends_with('"'))
    {
        value = &value[1..value.len() - 1];
    }

    // Handle escaped single quotes: '\'' -> '
    let value = value.replace("'\\''", "'");

    Some((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Basic Operations ===

    #[test]
    fn test_new_manager_is_empty() {
        let manager = AliasManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_add_alias() {
        let mut manager = AliasManager::new();
        assert!(manager.add("ll", "ls -la").is_ok());
        assert_eq!(manager.get("ll"), Some("ls -la"));
    }

    #[test]
    fn test_add_replaces_existing() {
        let mut manager = AliasManager::new();
        manager.add("ll", "ls -la").unwrap();
        manager.add("ll", "ls -lah").unwrap();
        assert_eq!(manager.get("ll"), Some("ls -lah"));
    }

    #[test]
    fn test_remove_alias() {
        let mut manager = AliasManager::new();
        manager.add("ll", "ls -la").unwrap();
        assert!(manager.remove("ll"));
        assert!(manager.get("ll").is_none());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut manager = AliasManager::new();
        assert!(!manager.remove("nonexistent"));
    }

    #[test]
    fn test_contains() {
        let mut manager = AliasManager::new();
        manager.add("ll", "ls -la").unwrap();
        assert!(manager.contains("ll"));
        assert!(!manager.contains("nonexistent"));
    }

    #[test]
    fn test_list_sorted() {
        let mut manager = AliasManager::new();
        manager.add("zz", "zzz").unwrap();
        manager.add("aa", "aaa").unwrap();
        manager.add("mm", "mmm").unwrap();

        let list = manager.list();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].0, "aa");
        assert_eq!(list[1].0, "mm");
        assert_eq!(list[2].0, "zz");
    }

    // === Alias Name Validation ===

    #[test]
    fn test_valid_alias_names() {
        assert!(is_valid_alias_name("ll"));
        assert!(is_valid_alias_name("gs"));
        assert!(is_valid_alias_name("my_alias"));
        assert!(is_valid_alias_name("_private"));
        assert!(is_valid_alias_name("alias123"));
    }

    #[test]
    fn test_invalid_alias_names() {
        assert!(!is_valid_alias_name("")); // Empty
        assert!(!is_valid_alias_name("123")); // Starts with digit
        assert!(!is_valid_alias_name("my-alias")); // Contains hyphen
        assert!(!is_valid_alias_name("my.alias")); // Contains dot
        assert!(!is_valid_alias_name("my alias")); // Contains space
    }

    #[test]
    fn test_add_invalid_name_fails() {
        let mut manager = AliasManager::new();
        assert!(manager.add("123", "value").is_err());
        assert!(manager.add("", "value").is_err());
    }

    // === Alias Expansion ===

    #[test]
    fn test_expand_simple() {
        let mut manager = AliasManager::new();
        manager.add("ll", "ls -la").unwrap();
        assert_eq!(manager.expand("ll"), "ls -la");
    }

    #[test]
    fn test_expand_with_args() {
        let mut manager = AliasManager::new();
        manager.add("ll", "ls -la").unwrap();
        assert_eq!(manager.expand("ll /tmp"), "ls -la /tmp");
    }

    #[test]
    fn test_expand_no_alias() {
        let manager = AliasManager::new();
        assert_eq!(manager.expand("ls -la"), "ls -la");
    }

    #[test]
    fn test_expand_nested_alias() {
        let mut manager = AliasManager::new();
        manager.add("l", "ls").unwrap();
        manager.add("ll", "l -la").unwrap();
        // ll -> l -la -> ls -la
        assert_eq!(manager.expand("ll"), "ls -la");
    }

    #[test]
    fn test_expand_empty_input() {
        let manager = AliasManager::new();
        assert_eq!(manager.expand(""), "");
        assert_eq!(manager.expand("   "), "   ");
    }

    // === Circular Alias Detection ===

    #[test]
    fn test_circular_alias_detected() {
        let mut manager = AliasManager::new();
        manager.add("a", "b").unwrap();
        manager.add("b", "a").unwrap();

        // Should not hang, should return something
        let result = manager.expand("a");
        // After max depth, returns the last expanded form
        assert!(!result.is_empty());
    }

    #[test]
    fn test_self_referential_alias() {
        let mut manager = AliasManager::new();
        manager.add("a", "a foo").unwrap();

        // Should not hang
        let result = manager.expand("a");
        assert!(!result.is_empty());
    }

    // === Helper Functions ===

    #[test]
    fn test_split_first_word() {
        assert_eq!(split_first_word("hello world"), ("hello", "world"));
        assert_eq!(split_first_word("hello"), ("hello", ""));
        // trim() removes both leading and trailing whitespace first
        assert_eq!(split_first_word("  hello  world  "), ("hello", "world"));
        // Empty string after trim returns empty
        let result = split_first_word("");
        assert_eq!(result, ("", ""));
    }

    #[test]
    fn test_parse_alias_line() {
        assert_eq!(
            parse_alias_line("ll='ls -la'"),
            Some(("ll".to_string(), "ls -la".to_string()))
        );
        assert_eq!(
            parse_alias_line("ll=\"ls -la\""),
            Some(("ll".to_string(), "ls -la".to_string()))
        );
        assert_eq!(
            parse_alias_line("ll=ls"),
            Some(("ll".to_string(), "ls".to_string()))
        );
        assert_eq!(parse_alias_line("invalid"), None);
    }

    #[test]
    fn test_parse_alias_line_with_escaped_quotes() {
        // test='echo '\''hello'\'''  should become: echo 'hello'
        assert_eq!(
            parse_alias_line("test='echo '\\''hello'\\'''"),
            Some(("test".to_string(), "echo 'hello'".to_string()))
        );
    }
}
