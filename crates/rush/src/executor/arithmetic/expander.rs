//! Expander for arithmetic expressions.
//!
//! Integrates arithmetic expansion into the shell's expansion pipeline.

use super::evaluator::{evaluate, VariableContext};
use super::parser::parse;
use super::{ArithmeticError, Result};

/// Check if a string contains arithmetic expansion syntax.
pub fn contains_arithmetic(input: &str) -> bool {
    input.contains("$((")
}

/// Expand all arithmetic expressions in the input string.
///
/// Replaces `$((expression))` with the evaluated result.
/// Supports nested arithmetic expansions.
pub fn expand_arithmetic<C: VariableContext>(input: &str, ctx: &mut C) -> Result<String> {
    if !contains_arithmetic(input) {
        return Ok(input.to_string());
    }

    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut i = 0;
    let input_chars: Vec<char> = input.chars().collect();

    while i < input_chars.len() {
        // Check for $((
        if i + 2 < input_chars.len()
            && input_chars[i] == '$'
            && input_chars[i + 1] == '('
            && input_chars[i + 2] == '('
        {
            // Find the matching ))
            let start = i + 3; // After $((
            let end = find_closing_parens(&input_chars, start)?;

            // Extract the expression
            let expr_str: String = input_chars[start..end].iter().collect();

            // Recursively expand any nested arithmetic
            let expanded_expr = expand_arithmetic(&expr_str, ctx)?;

            // Parse and evaluate
            let expr = parse(&expanded_expr)?;
            let value = evaluate(&expr, ctx)?;

            // Append the result
            result.push_str(&value.to_string());

            // Skip past ))
            i = end + 2;
        } else {
            result.push(input_chars[i]);
            i += 1;
        }
    }

    Ok(result)
}

/// Find the position of the matching )) for a $(( at the given start position.
fn find_closing_parens(chars: &[char], start: usize) -> Result<usize> {
    let mut depth = 1;
    let mut i = start;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '(' && chars[i + 1] == '(' {
            depth += 1;
            i += 2;
        } else if i + 1 < chars.len() && chars[i] == ')' && chars[i + 1] == ')' {
            depth -= 1;
            if depth == 0 {
                return Ok(i);
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    Err(ArithmeticError::SyntaxError("unmatched $((".to_string()))
}

/// Evaluate an arithmetic expression string and return the result.
///
/// This is a convenience function for use by the `let` builtin.
pub fn evaluate_expression<C: VariableContext>(input: &str, ctx: &mut C) -> Result<i64> {
    let expr = parse(input)?;
    evaluate(&expr, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::arithmetic::evaluator::SimpleContext;

    fn expand(input: &str) -> Result<String> {
        let mut ctx = SimpleContext::new();
        expand_arithmetic(input, &mut ctx)
    }

    fn expand_with_var(input: &str, name: &str, value: i64) -> Result<String> {
        let mut ctx = SimpleContext::new();
        ctx.set(name, value);
        expand_arithmetic(input, &mut ctx)
    }

    #[test]
    fn test_no_expansion() {
        assert_eq!(expand("hello world").unwrap(), "hello world");
    }

    #[test]
    fn test_simple_expansion() {
        assert_eq!(expand("$((2 + 3))").unwrap(), "5");
    }

    #[test]
    fn test_expansion_in_text() {
        assert_eq!(expand("result: $((5 * 3))").unwrap(), "result: 15");
    }

    #[test]
    fn test_multiple_expansions() {
        assert_eq!(expand("$((1 + 1)) and $((2 + 2))").unwrap(), "2 and 4");
    }

    #[test]
    fn test_with_variable() {
        assert_eq!(expand_with_var("$((x * 2))", "x", 10).unwrap(), "20");
    }

    #[test]
    fn test_nested_expansion() {
        assert_eq!(expand("$((1 + $((2 * 3))))").unwrap(), "7");
    }

    #[test]
    fn test_empty_expression() {
        assert_eq!(expand("$(())").unwrap(), "0");
    }

    #[test]
    fn test_unmatched_parens() {
        let result = expand("$((2 + 3");
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_arithmetic() {
        assert!(contains_arithmetic("$((5))"));
        assert!(contains_arithmetic("echo $((2+3))"));
        assert!(!contains_arithmetic("echo hello"));
        assert!(!contains_arithmetic("$( cmd )"));
    }
}
