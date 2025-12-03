//! Array syntax parsing
//!
//! Provides functions to parse bash array syntax:
//! - Array assignment: `arr=(val1 val2 val3)`
//! - Array indexing: `${arr[0]}`
//! - Array expansion: `${arr[@]}` and `${arr[*]}`

use crate::error::{Result, RushError};

/// Result of parsing an array assignment: `arr=(val1 val2 val3)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayAssignment {
    /// The array variable name
    pub name: String,
    /// The values to assign to the array
    pub values: Vec<String>,
}

/// Type of array reference in expansion: `${arr[...]}` or `${arr*}` or `${arr@}`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayRefType {
    /// Specific index: ${arr[0]}
    Index(usize),
    /// All elements as separate words: ${arr[@]}
    AllWords,
    /// All elements as one word: ${arr[*]}
    AllAsOne,
}

/// Result of parsing an array reference: `${arr[0]}`, `${arr[@]}`, or `${arr[*]}`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayRef {
    /// The array variable name
    pub name: String,
    /// The type of array reference
    pub ref_type: ArrayRefType,
}

/// Check if a string is an array assignment: `name=(val1 val2 val3)`
///
/// # Examples
/// ```ignore
/// assert!(is_array_assignment("arr=(1 2 3)"));
/// assert!(is_array_assignment("my_array=(a b c)"));
/// assert!(!is_array_assignment("var=value"));
/// assert!(!is_array_assignment("arr=value"));
/// ```
pub fn is_array_assignment(input: &str) -> bool {
    input.contains("=(") && input.ends_with(')')
}

/// Parse an array assignment: `name=(val1 val2 val3)`
///
/// # Arguments
/// * `input` - A string like "arr=(val1 val2 val3)"
///
/// # Returns
/// * `Ok(ArrayAssignment)` - Parsed array with name and values
/// * `Err(_)` - Invalid syntax
///
/// # Examples
/// ```ignore
/// let arr = parse_array_assignment("arr=(1 2 3)")?;
/// assert_eq!(arr.name, "arr");
/// assert_eq!(arr.values, vec!["1", "2", "3"]);
/// ```
pub fn parse_array_assignment(input: &str) -> Result<ArrayAssignment> {
    // Find the = sign
    let eq_pos = input.find('=').ok_or_else(|| {
        RushError::Execution(format!("parse_array_assignment: no '=' in '{}'", input))
    })?;

    let name = input[..eq_pos].to_string();
    let value_str = &input[eq_pos + 1..];

    // Validate array name
    if !is_valid_identifier(&name) {
        return Err(RushError::Execution(format!(
            "parse_array_assignment: '{}' is not a valid identifier",
            name
        )));
    }

    // Parse the array values: (val1 val2 val3)
    if !value_str.starts_with('(') || !value_str.ends_with(')') {
        return Err(RushError::Execution(format!(
            "parse_array_assignment: array syntax must be (val1 val2 ...), got '{}'",
            value_str
        )));
    }

    let inner = &value_str[1..value_str.len() - 1]; // Remove ( and )
    let values: Vec<String> = if inner.is_empty() {
        Vec::new()
    } else {
        // Split by whitespace, respecting quotes
        split_array_values(inner)?
    };

    Ok(ArrayAssignment { name, values })
}

/// Split array values respecting quotes
fn split_array_values(input: &str) -> Result<Vec<String>> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => escape_next = true,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            ' ' | '\t' | '\n' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    values.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }

    if in_single_quote {
        return Err(RushError::Execution(
            "parse_array_values: unclosed single quote".to_string(),
        ));
    }
    if in_double_quote {
        return Err(RushError::Execution(
            "parse_array_values: unclosed double quote".to_string(),
        ));
    }
    if escape_next {
        return Err(RushError::Execution(
            "parse_array_values: trailing backslash".to_string(),
        ));
    }

    if !current.is_empty() {
        values.push(current);
    }

    Ok(values)
}

/// Check if a string is an array reference: `${arr[...]}` or `${arr[@]}` or `${arr[*]}`
pub fn is_array_ref(input: &str) -> bool {
    input.starts_with("${") && (input.contains('[') || input.ends_with('@')) && input.ends_with('}')
}

/// Parse an array reference: `${arr[0]}`, `${arr[@]}`, or `${arr[*]}`
///
/// # Arguments
/// * `input` - A string like "${arr[0]}" or "${arr[@]}" or "${arr[*]}"
///
/// # Returns
/// * `Ok(ArrayRef)` - Parsed array reference with name and type
/// * `Err(_)` - Invalid syntax
///
/// # Examples
/// ```ignore
/// let arr = parse_array_ref("${arr[0]}")?;
/// assert_eq!(arr.name, "arr");
/// assert!(matches!(arr.ref_type, ArrayRefType::Index(0)));
/// ```
pub fn parse_array_ref(input: &str) -> Result<ArrayRef> {
    if !input.starts_with("${") || !input.ends_with('}') {
        return Err(RushError::Execution(format!(
            "parse_array_ref: invalid syntax '{}', expected '${{...}}'",
            input
        )));
    }

    // Remove ${ and }
    let inner = &input[2..input.len() - 1];

    // Handle the different cases
    if let Some(bracket_pos) = inner.find('[') {
        // Handle ${arr[...]} cases
        let name = inner[..bracket_pos].to_string();

        if !is_valid_identifier(&name) {
            return Err(RushError::Execution(format!(
                "parse_array_ref: '{}' is not a valid identifier",
                name
            )));
        }

        // Find closing ]
        if !inner.ends_with(']') {
            return Err(RushError::Execution(format!(
                "parse_array_ref: missing closing ']' in '{}'",
                input
            )));
        }

        let index_str = &inner[bracket_pos + 1..inner.len() - 1];

        let ref_type = match index_str {
            "@" => ArrayRefType::AllWords,
            "*" => ArrayRefType::AllAsOne,
            _ => {
                // Try to parse as a number
                match index_str.parse::<usize>() {
                    Ok(idx) => ArrayRefType::Index(idx),
                    Err(_) => {
                        return Err(RushError::Execution(format!(
                            "parse_array_ref: invalid array index '{}' in '{}'",
                            index_str, input
                        )))
                    }
                }
            }
        };

        Ok(ArrayRef { name, ref_type })
    } else {
        Err(RushError::Execution(format!(
            "parse_array_ref: expected '[...]' in array reference '{}'",
            input
        )))
    }
}

/// Check if a string is a valid bash identifier (variable/array name)
fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let first = name.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Array Assignment Tests =====

    #[test]
    fn test_is_array_assignment() {
        assert!(is_array_assignment("arr=(1 2 3)"));
        assert!(is_array_assignment("my_array=(a b c)"));
        assert!(is_array_assignment("_arr=()"));
        assert!(!is_array_assignment("var=value"));
        assert!(!is_array_assignment("arr=value"));
        assert!(!is_array_assignment("arr=("));
    }

    #[test]
    fn test_parse_array_assignment_basic() {
        let arr = parse_array_assignment("arr=(1 2 3)").unwrap();
        assert_eq!(arr.name, "arr");
        assert_eq!(arr.values, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_parse_array_assignment_empty() {
        let arr = parse_array_assignment("arr=()").unwrap();
        assert_eq!(arr.name, "arr");
        assert!(arr.values.is_empty());
    }

    #[test]
    fn test_parse_array_assignment_single() {
        let arr = parse_array_assignment("arr=(single)").unwrap();
        assert_eq!(arr.name, "arr");
        assert_eq!(arr.values, vec!["single"]);
    }

    #[test]
    fn test_parse_array_assignment_quoted_values() {
        let arr = parse_array_assignment("arr=(\"hello world\" 'foo bar' baz)").unwrap();
        assert_eq!(arr.name, "arr");
        assert_eq!(arr.values, vec!["hello world", "foo bar", "baz"]);
    }

    #[test]
    fn test_parse_array_assignment_with_spaces() {
        let arr = parse_array_assignment("arr=(  a   b   c  )").unwrap();
        assert_eq!(arr.name, "arr");
        assert_eq!(arr.values, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_array_assignment_underscores_in_name() {
        let arr = parse_array_assignment("_my_arr_=(1 2)").unwrap();
        assert_eq!(arr.name, "_my_arr_");
    }

    #[test]
    fn test_parse_array_assignment_invalid_name() {
        assert!(parse_array_assignment("1arr=(1 2)").is_err());
        assert!(parse_array_assignment("-arr=(1 2)").is_err());
    }

    #[test]
    fn test_parse_array_assignment_invalid_syntax() {
        assert!(parse_array_assignment("arr=1 2 3").is_err());
        assert!(parse_array_assignment("arr=(1 2 3").is_err());
    }

    #[test]
    fn test_parse_array_assignment_unclosed_quote() {
        assert!(parse_array_assignment("arr=(\"unclosed)").is_err());
    }

    // ===== Array Reference Tests =====

    #[test]
    fn test_is_array_ref() {
        assert!(is_array_ref("${arr[0]}"));
        assert!(is_array_ref("${arr[@]}"));
        assert!(is_array_ref("${arr[*]}"));
        assert!(!is_array_ref("${arr}"));
        assert!(!is_array_ref("$arr[0]"));
        assert!(!is_array_ref("arr[0]"));
    }

    #[test]
    fn test_parse_array_ref_index_zero() {
        let arr = parse_array_ref("${arr[0]}").unwrap();
        assert_eq!(arr.name, "arr");
        assert_eq!(arr.ref_type, ArrayRefType::Index(0));
    }

    #[test]
    fn test_parse_array_ref_index_nonzero() {
        let arr = parse_array_ref("${arr[5]}").unwrap();
        assert_eq!(arr.name, "arr");
        assert_eq!(arr.ref_type, ArrayRefType::Index(5));
    }

    #[test]
    fn test_parse_array_ref_all_words() {
        let arr = parse_array_ref("${arr[@]}").unwrap();
        assert_eq!(arr.name, "arr");
        assert_eq!(arr.ref_type, ArrayRefType::AllWords);
    }

    #[test]
    fn test_parse_array_ref_all_as_one() {
        let arr = parse_array_ref("${arr[*]}").unwrap();
        assert_eq!(arr.name, "arr");
        assert_eq!(arr.ref_type, ArrayRefType::AllAsOne);
    }

    #[test]
    fn test_parse_array_ref_underscore_name() {
        let arr = parse_array_ref("${_arr[0]}").unwrap();
        assert_eq!(arr.name, "_arr");
    }

    #[test]
    fn test_parse_array_ref_invalid_name() {
        assert!(parse_array_ref("${1arr[0]}").is_err());
        assert!(parse_array_ref("${-arr[0]}").is_err());
    }

    #[test]
    fn test_parse_array_ref_invalid_index() {
        assert!(parse_array_ref("${arr[invalid]}").is_err());
        assert!(parse_array_ref("${arr[-1]}").is_err());
    }

    #[test]
    fn test_parse_array_ref_missing_bracket() {
        assert!(parse_array_ref("${arr}").is_err());
    }

    #[test]
    fn test_parse_array_ref_missing_closing_bracket() {
        assert!(parse_array_ref("${arr[0}").is_err());
    }

    #[test]
    fn test_parse_array_ref_missing_closing_brace() {
        assert!(parse_array_ref("${arr[0]").is_err());
    }

    #[test]
    fn test_parse_array_ref_large_index() {
        let arr = parse_array_ref("${arr[999999]}").unwrap();
        assert_eq!(arr.ref_type, ArrayRefType::Index(999999));
    }

    // ===== Integration Tests =====

    #[test]
    fn test_mixed_assignment_and_refs() {
        let assign = parse_array_assignment("items=(apple banana cherry)").unwrap();
        assert_eq!(assign.values.len(), 3);

        let ref0 = parse_array_ref("${items[0]}").unwrap();
        assert_eq!(ref0.ref_type, ArrayRefType::Index(0));

        let ref_all = parse_array_ref("${items[@]}").unwrap();
        assert_eq!(ref_all.ref_type, ArrayRefType::AllWords);
    }
}
