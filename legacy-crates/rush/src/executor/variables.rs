//! Environment variable management
//!
//! Manages variables for the shell, tracking which ones are exported
//! (visible to subshells) and which are local to the shell session.
//!
//! Supports both string and array variables.

use crate::error::{Result, RushError};
use std::collections::{HashMap, HashSet};

/// Variable value types
#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    /// String variable
    String(String),
    /// Array variable
    Array(Vec<String>),
}

impl Variable {
    /// Get string representation for display/export
    pub fn as_string(&self) -> String {
        match self {
            Variable::String(s) => s.clone(),
            Variable::Array(arr) => {
                // Arrays are displayed as space-separated values
                arr.join(" ")
            }
        }
    }

    /// Check if this is a string variable
    pub fn is_string(&self) -> bool {
        matches!(self, Variable::String(_))
    }

    /// Check if this is an array variable
    pub fn is_array(&self) -> bool {
        matches!(self, Variable::Array(_))
    }
}

/// Manages shell environment variables
#[derive(Debug, Clone)]
pub struct VariableManager {
    /// All variables (name -> value)
    variables: HashMap<String, Variable>,
    /// Which variables are exported (for subshells)
    exported: HashSet<String>,
    /// Scope stack for local variables (saved values for each function scope)
    /// Each entry maps variable name -> previous value (None if didn't exist)
    scope_stack: Vec<HashMap<String, Option<Variable>>>,
}

impl VariableManager {
    /// Create a new variable manager
    pub fn new() -> Self {
        Self { variables: HashMap::new(), exported: HashSet::new(), scope_stack: Vec::new() }
    }

    /// Push a new scope (called when entering a function)
    pub fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Pop a scope and restore saved variables (called when exiting a function)
    pub fn pop_scope(&mut self) {
        if let Some(saved) = self.scope_stack.pop() {
            for (name, old_value) in saved {
                match old_value {
                    Some(v) => {
                        self.variables.insert(name, v);
                    }
                    None => {
                        self.variables.remove(&name);
                    }
                }
            }
        }
    }

    /// Get current scope depth (0 = global, >0 = inside function)
    pub fn scope_depth(&self) -> usize {
        self.scope_stack.len()
    }

    /// Set a local variable (only valid inside a function scope)
    ///
    /// Saves the current value (if any) so it can be restored when the scope exits.
    ///
    /// # Returns
    /// * `Ok(())` - Variable set locally
    /// * `Err(_)` - Not in a function scope or invalid name
    pub fn set_local(&mut self, name: String, value: Option<String>) -> Result<()> {
        if self.scope_stack.is_empty() {
            return Err(RushError::Execution("local: can only be used in a function".to_string()));
        }

        if !Self::is_valid_name(&name) {
            return Err(RushError::Execution(format!("local: {}: invalid identifier", name)));
        }

        // Save current value in scope stack (if not already saved in this scope)
        let current_scope = self.scope_stack.last_mut().unwrap();
        if !current_scope.contains_key(&name) {
            let old_value = self.variables.get(&name).cloned();
            current_scope.insert(name.clone(), old_value);
        }

        // Set the new value
        match value {
            Some(v) => {
                self.variables.insert(name, Variable::String(v));
            }
            None => {
                // local var without value - set to empty string
                self.variables.insert(name, Variable::String(String::new()));
            }
        }

        Ok(())
    }

    /// Set a string variable
    ///
    /// # Arguments
    /// * `name` - Variable name (must be valid identifier)
    /// * `value` - Variable value
    ///
    /// # Returns
    /// * `Ok(())` - Variable set successfully
    /// * `Err(_)` - Invalid variable name
    pub fn set(&mut self, name: String, value: String) -> Result<()> {
        if !Self::is_valid_name(&name) {
            return Err(RushError::Execution(format!("set: {}: invalid identifier", name)));
        }
        self.variables.insert(name, Variable::String(value));
        Ok(())
    }

    /// Get a string variable value (returns None if variable is an array)
    pub fn get(&self, name: &str) -> Option<&str> {
        self.variables.get(name).and_then(|v| match v {
            Variable::String(s) => Some(s.as_str()),
            Variable::Array(_) => None,
        })
    }

    /// Set an array variable
    ///
    /// # Arguments
    /// * `name` - Variable name (must be valid identifier)
    /// * `values` - Array values
    ///
    /// # Returns
    /// * `Ok(())` - Array set successfully
    /// * `Err(_)` - Invalid variable name
    pub fn set_array(&mut self, name: String, values: Vec<String>) -> Result<()> {
        if !Self::is_valid_name(&name) {
            return Err(RushError::Execution(format!("set_array: {}: invalid identifier", name)));
        }
        self.variables.insert(name, Variable::Array(values));
        Ok(())
    }

    /// Get an array variable (returns None if variable is a string or doesn't exist)
    pub fn get_array(&self, name: &str) -> Option<&Vec<String>> {
        self.variables.get(name).and_then(|v| match v {
            Variable::Array(arr) => Some(arr),
            Variable::String(_) => None,
        })
    }

    /// Get an element from an array by index (zero-based)
    ///
    /// # Returns
    /// * `Some(value)` - Element exists at that index
    /// * `None` - Array doesn't exist, index out of bounds, or variable is string
    pub fn array_get(&self, name: &str, index: usize) -> Option<&str> {
        self.variables.get(name).and_then(|v| match v {
            Variable::Array(arr) => arr.get(index).map(|s| s.as_str()),
            Variable::String(_) => None,
        })
    }

    /// Get the length of an array
    ///
    /// # Returns
    /// * `Some(length)` - Array exists
    /// * `None` - Variable doesn't exist or is a string
    pub fn array_length(&self, name: &str) -> Option<usize> {
        self.variables.get(name).and_then(|v| match v {
            Variable::Array(arr) => Some(arr.len()),
            Variable::String(_) => None,
        })
    }

    /// Append a value to an array
    ///
    /// Creates a new array if it doesn't exist.
    /// If the variable exists but is a string, converts it to an array with [string, value]
    ///
    /// # Returns
    /// * `Ok(())` - Value appended successfully
    /// * `Err(_)` - Invalid variable name
    pub fn append_to_array(&mut self, name: String, value: String) -> Result<()> {
        if !Self::is_valid_name(&name) {
            return Err(RushError::Execution(format!(
                "append_to_array: {}: invalid identifier",
                name
            )));
        }

        match self.variables.get_mut(&name) {
            Some(Variable::Array(arr)) => {
                // Append to existing array
                arr.push(value);
            }
            Some(Variable::String(s)) => {
                // Convert string to array: [old_string, new_value]
                let old_value = s.clone();
                self.variables
                    .insert(name, Variable::Array(vec![old_value, value]));
            }
            None => {
                // Create new array with single element
                self.variables.insert(name, Variable::Array(vec![value]));
            }
        }

        Ok(())
    }

    /// Get the type of a variable
    pub fn get_type(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|v| match v {
            Variable::String(_) => "string",
            Variable::Array(_) => "array",
        })
    }

    /// Remove a variable
    ///
    /// # Returns
    /// * `true` - Variable was removed
    /// * `false` - Variable didn't exist
    pub fn remove(&mut self, name: &str) -> bool {
        self.exported.remove(name);
        self.variables.remove(name).is_some()
    }

    /// Mark a variable as exported (visible to subshells)
    ///
    /// # Returns
    /// * `Ok(())` - Variable marked as exported
    /// * `Err(_)` - Variable doesn't exist
    pub fn export(&mut self, name: &str) -> Result<()> {
        if !self.variables.contains_key(name) {
            return Err(RushError::Execution(format!("export: {}: not set", name)));
        }
        self.exported.insert(name.to_string());
        Ok(())
    }

    /// Check if a variable is exported
    pub fn is_exported(&self, name: &str) -> bool {
        self.exported.contains(name)
    }

    /// List all variables (sorted by name)
    pub fn list(&self) -> Vec<(&str, &str)> {
        let mut vars: Vec<_> = self
            .variables
            .iter()
            .filter_map(|(k, v)| match v {
                Variable::String(s) => Some((k.as_str(), s.as_str())),
                Variable::Array(_) => None, // Arrays not listed in string format
            })
            .collect();
        vars.sort_by(|a, b| a.0.cmp(b.0));
        vars
    }

    /// List only exported variables (sorted by name)
    pub fn list_exported(&self) -> Vec<(&str, &str)> {
        let mut vars: Vec<_> = self
            .variables
            .iter()
            .filter_map(|(k, v)| {
                if self.exported.contains(k.as_str()) {
                    match v {
                        Variable::String(s) => Some((k.as_str(), s.as_str())),
                        Variable::Array(_) => None,
                    }
                } else {
                    None
                }
            })
            .collect();
        vars.sort_by(|a, b| a.0.cmp(b.0));
        vars
    }

    /// Get number of variables
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Check if a variable exists
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Check if a variable name is valid
    ///
    /// Valid names are: alphanumeric + underscore, must start with letter or underscore
    fn is_valid_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let first = name.chars().next().unwrap();
        if !first.is_alphabetic() && first != '_' {
            return false;
        }

        name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }
}

impl Default for VariableManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // String variable tests (backward compatibility)
    #[test]
    fn test_set_and_get() {
        let mut vm = VariableManager::new();
        vm.set("name".to_string(), "value".to_string()).unwrap();
        assert_eq!(vm.get("name"), Some("value"));
    }

    #[test]
    fn test_get_nonexistent() {
        let vm = VariableManager::new();
        assert_eq!(vm.get("nonexistent"), None);
    }

    #[test]
    fn test_remove() {
        let mut vm = VariableManager::new();
        vm.set("name".to_string(), "value".to_string()).unwrap();
        assert!(vm.remove("name"));
        assert_eq!(vm.get("name"), None);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut vm = VariableManager::new();
        assert!(!vm.remove("nonexistent"));
    }

    #[test]
    fn test_export() {
        let mut vm = VariableManager::new();
        vm.set("name".to_string(), "value".to_string()).unwrap();
        vm.export("name").unwrap();
        assert!(vm.is_exported("name"));
    }

    #[test]
    fn test_export_nonexistent() {
        let mut vm = VariableManager::new();
        assert!(vm.export("nonexistent").is_err());
    }

    #[test]
    fn test_list() {
        let mut vm = VariableManager::new();
        vm.set("alpha".to_string(), "1".to_string()).unwrap();
        vm.set("beta".to_string(), "2".to_string()).unwrap();
        vm.set("gamma".to_string(), "3".to_string()).unwrap();

        let list = vm.list();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].0, "alpha");
        assert_eq!(list[1].0, "beta");
        assert_eq!(list[2].0, "gamma");
    }

    #[test]
    fn test_list_exported() {
        let mut vm = VariableManager::new();
        vm.set("local".to_string(), "1".to_string()).unwrap();
        vm.set("exported".to_string(), "2".to_string()).unwrap();
        vm.export("exported").unwrap();

        let exported = vm.list_exported();
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].0, "exported");
    }

    #[test]
    fn test_invalid_names() {
        let mut vm = VariableManager::new();

        // Starting with number
        assert!(vm.set("1name".to_string(), "value".to_string()).is_err());

        // With hyphen
        assert!(vm.set("my-var".to_string(), "value".to_string()).is_err());

        // With space
        assert!(vm.set("my var".to_string(), "value".to_string()).is_err());
    }

    #[test]
    fn test_valid_names() {
        let mut vm = VariableManager::new();

        // Starting with letter
        assert!(vm.set("name".to_string(), "value".to_string()).is_ok());

        // Starting with underscore
        assert!(vm.set("_private".to_string(), "value".to_string()).is_ok());

        // With numbers
        assert!(vm.set("var123".to_string(), "value".to_string()).is_ok());

        // Uppercase
        assert!(vm.set("MYVAR".to_string(), "value".to_string()).is_ok());
    }

    #[test]
    fn test_update_variable() {
        let mut vm = VariableManager::new();
        vm.set("name".to_string(), "value1".to_string()).unwrap();
        assert_eq!(vm.get("name"), Some("value1"));

        vm.set("name".to_string(), "value2".to_string()).unwrap();
        assert_eq!(vm.get("name"), Some("value2"));
    }

    #[test]
    fn test_remove_exported_variable() {
        let mut vm = VariableManager::new();
        vm.set("name".to_string(), "value".to_string()).unwrap();
        vm.export("name").unwrap();
        assert!(vm.is_exported("name"));

        vm.remove("name");
        assert!(!vm.is_exported("name"));
        assert_eq!(vm.get("name"), None);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut vm = VariableManager::new();
        assert!(vm.is_empty());
        assert_eq!(vm.len(), 0);

        vm.set("name".to_string(), "value".to_string()).unwrap();
        assert!(!vm.is_empty());
        assert_eq!(vm.len(), 1);

        vm.remove("name");
        assert!(vm.is_empty());
        assert_eq!(vm.len(), 0);
    }

    // Array variable tests
    #[test]
    fn test_set_and_get_array() {
        let mut vm = VariableManager::new();
        vm.set_array("arr".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string()])
            .unwrap();
        let arr = vm.get_array("arr").unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], "a");
        assert_eq!(arr[1], "b");
        assert_eq!(arr[2], "c");
    }

    #[test]
    fn test_array_get_element() {
        let mut vm = VariableManager::new();
        vm.set_array(
            "arr".to_string(),
            vec!["zero".to_string(), "one".to_string(), "two".to_string()],
        )
        .unwrap();
        assert_eq!(vm.array_get("arr", 0), Some("zero"));
        assert_eq!(vm.array_get("arr", 1), Some("one"));
        assert_eq!(vm.array_get("arr", 2), Some("two"));
        assert_eq!(vm.array_get("arr", 3), None);
    }

    #[test]
    fn test_array_length() {
        let mut vm = VariableManager::new();
        vm.set_array("arr".to_string(), vec!["a".to_string(), "b".to_string()])
            .unwrap();
        assert_eq!(vm.array_length("arr"), Some(2));
    }

    #[test]
    fn test_array_length_nonexistent() {
        let vm = VariableManager::new();
        assert_eq!(vm.array_length("nonexistent"), None);
    }

    #[test]
    fn test_append_to_array() {
        let mut vm = VariableManager::new();
        vm.set_array("arr".to_string(), vec!["a".to_string()])
            .unwrap();
        vm.append_to_array("arr".to_string(), "b".to_string())
            .unwrap();
        vm.append_to_array("arr".to_string(), "c".to_string())
            .unwrap();

        let arr = vm.get_array("arr").unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], "a");
        assert_eq!(arr[1], "b");
        assert_eq!(arr[2], "c");
    }

    #[test]
    fn test_append_to_new_array() {
        let mut vm = VariableManager::new();
        vm.append_to_array("new_arr".to_string(), "first".to_string())
            .unwrap();
        assert_eq!(vm.array_length("new_arr"), Some(1));
        assert_eq!(vm.array_get("new_arr", 0), Some("first"));
    }

    #[test]
    fn test_append_to_string_converts_to_array() {
        let mut vm = VariableManager::new();
        vm.set("var".to_string(), "original".to_string()).unwrap();
        vm.append_to_array("var".to_string(), "appended".to_string())
            .unwrap();

        // Should be an array now
        let arr = vm.get_array("var").unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], "original");
        assert_eq!(arr[1], "appended");
        assert_eq!(vm.get("var"), None); // No longer a string
    }

    #[test]
    fn test_get_type() {
        let mut vm = VariableManager::new();
        vm.set("str_var".to_string(), "value".to_string()).unwrap();
        vm.set_array("arr_var".to_string(), vec!["a".to_string()])
            .unwrap();

        assert_eq!(vm.get_type("str_var"), Some("string"));
        assert_eq!(vm.get_type("arr_var"), Some("array"));
        assert_eq!(vm.get_type("nonexistent"), None);
    }

    #[test]
    fn test_contains() {
        let mut vm = VariableManager::new();
        assert!(!vm.contains("var"));
        vm.set("var".to_string(), "value".to_string()).unwrap();
        assert!(vm.contains("var"));
    }

    #[test]
    fn test_variable_enum_as_string() {
        let s = Variable::String("hello".to_string());
        assert_eq!(s.as_string(), "hello");

        let a = Variable::Array(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(a.as_string(), "a b c");
    }

    #[test]
    fn test_variable_enum_type_checks() {
        let s = Variable::String("value".to_string());
        assert!(s.is_string());
        assert!(!s.is_array());

        let a = Variable::Array(vec!["val".to_string()]);
        assert!(!a.is_string());
        assert!(a.is_array());
    }
}
