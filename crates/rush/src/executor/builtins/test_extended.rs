//! Extended test command `[[` builtin
//!
//! Implements the bash-compatible extended test command with:
//! - String and numeric comparisons
//! - File test operators
//! - Pattern matching (glob and regex)
//! - Logical operators with short-circuit evaluation
//! - No word splitting or pathname expansion

use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;
use crate::executor::test_expr::{BinaryOperator, Expression, LogicalOperator, UnaryOperator};
use regex::Regex;

/// Test expression evaluator
pub struct TestEvaluator<'a> {
    executor: &'a mut CommandExecutor,
}

impl<'a> TestEvaluator<'a> {
    /// Create a new test evaluator
    pub fn new(executor: &'a mut CommandExecutor) -> Self {
        Self { executor }
    }

    /// Evaluate a test expression to a boolean result
    ///
    /// # Arguments
    /// * `expression` - The test expression to evaluate
    ///
    /// # Returns
    /// * `Ok(true)` - Condition is true (exit 0)
    /// * `Ok(false)` - Condition is false (exit 1)
    /// * `Err(RushError)` - Evaluation error (exit 2)
    pub fn evaluate(&mut self, expression: &Expression) -> Result<bool> {
        match expression {
            Expression::Literal(val) => {
                // Non-empty string is true
                Ok(!val.is_empty())
            }
            Expression::UnaryOp { operator, operand } => self.evaluate_unary_op(operator, operand),
            Expression::BinaryOp { left, operator, right } => {
                self.evaluate_binary_op(left, operator, right)
            }
            Expression::LogicalOp { left, operator, right } => {
                self.evaluate_logical_op(left, operator, right)
            }
            Expression::Grouped(inner) => self.evaluate(inner),
        }
    }

    /// Evaluate unary operator
    fn evaluate_unary_op(
        &mut self,
        operator: &UnaryOperator,
        operand: &Expression,
    ) -> Result<bool> {
        match operator {
            UnaryOperator::Negation => {
                let result = self.evaluate(operand)?;
                Ok(!result)
            }
            UnaryOperator::StringEmpty => {
                // Get the operand as a string
                let value = self.get_literal_value(operand)?;
                Ok(value.is_empty())
            }
            UnaryOperator::StringNonEmpty => {
                let value = self.get_literal_value(operand)?;
                Ok(!value.is_empty())
            }
            _ => self.evaluate_file_test(operator, operand),
        }
    }

    /// Get literal value from an expression
    fn get_literal_value(&mut self, expr: &Expression) -> Result<String> {
        match expr {
            Expression::Literal(s) => Ok(s.clone()),
            _ => {
                // Evaluate the expression first
                let result = self.evaluate(expr)?;
                Ok(if result { "1" } else { "" }.to_string())
            }
        }
    }

    /// Evaluate file test operators
    fn evaluate_file_test(
        &mut self,
        operator: &UnaryOperator,
        operand: &Expression,
    ) -> Result<bool> {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let path = self.get_literal_value(operand)?;

        match operator {
            UnaryOperator::FileExists => Ok(std::path::Path::new(&path).exists()),
            UnaryOperator::FileRegular => Ok(std::path::Path::new(&path).is_file()),
            UnaryOperator::FileDirectory => Ok(std::path::Path::new(&path).is_dir()),
            UnaryOperator::FileReadable => match fs::metadata(&path) {
                Ok(metadata) => {
                    let permissions = metadata.permissions();
                    Ok(permissions.mode() & 0o400 != 0)
                }
                Err(_) => Ok(false),
            },
            UnaryOperator::FileWritable => match fs::metadata(&path) {
                Ok(metadata) => {
                    let permissions = metadata.permissions();
                    Ok(permissions.mode() & 0o200 != 0)
                }
                Err(_) => Ok(false),
            },
            UnaryOperator::FileExecutable => match fs::metadata(&path) {
                Ok(metadata) => {
                    let permissions = metadata.permissions();
                    Ok(permissions.mode() & 0o100 != 0)
                }
                Err(_) => Ok(false),
            },
            UnaryOperator::FileNonEmpty => match fs::metadata(&path) {
                Ok(metadata) => Ok(metadata.len() > 0),
                Err(_) => Ok(false),
            },
            _ => Err(RushError::InvalidOperator(format!("unhandled operator: {:?}", operator))),
        }
    }

    /// Evaluate binary operator
    fn evaluate_binary_op(
        &mut self,
        left: &str,
        operator: &BinaryOperator,
        right: &str,
    ) -> Result<bool> {
        match operator {
            // String operators
            BinaryOperator::StringEqual => Ok(left == right),
            BinaryOperator::StringNotEqual => Ok(left != right),
            BinaryOperator::StringLess => Ok(left < right),
            BinaryOperator::StringGreater => Ok(left > right),

            // Numeric operators
            BinaryOperator::NumericEqual => {
                self.evaluate_numeric_comparison(left, right, |a, b| a == b)
            }
            BinaryOperator::NumericNotEqual => {
                self.evaluate_numeric_comparison(left, right, |a, b| a != b)
            }
            BinaryOperator::NumericLess => {
                self.evaluate_numeric_comparison(left, right, |a, b| a < b)
            }
            BinaryOperator::NumericLessEqual => {
                self.evaluate_numeric_comparison(left, right, |a, b| a <= b)
            }
            BinaryOperator::NumericGreater => {
                self.evaluate_numeric_comparison(left, right, |a, b| a > b)
            }
            BinaryOperator::NumericGreaterEqual => {
                self.evaluate_numeric_comparison(left, right, |a, b| a >= b)
            }

            // Pattern operators
            BinaryOperator::GlobMatch => self.glob_match(left, right),
            BinaryOperator::GlobNotMatch => self.glob_match(left, right).map(|result| !result),
            BinaryOperator::RegexMatch => self.regex_match(left, right),
        }
    }

    /// Evaluate numeric comparison
    fn evaluate_numeric_comparison<F>(&self, left: &str, right: &str, compare: F) -> Result<bool>
    where
        F: Fn(i64, i64) -> bool,
    {
        let left_num = left
            .parse::<i64>()
            .map_err(|_| RushError::TypeMismatch(format!("'{}' is not a valid integer", left)))?;
        let right_num = right
            .parse::<i64>()
            .map_err(|_| RushError::TypeMismatch(format!("'{}' is not a valid integer", right)))?;
        Ok(compare(left_num, right_num))
    }

    /// Evaluate glob pattern matching
    ///
    /// Supports: * (any chars), ? (single char), [...] (character class), [!...] (negated class)
    fn glob_match(&self, text: &str, pattern: &str) -> Result<bool> {
        // Validate pattern length (10KB limit from spec)
        if pattern.len() > 10240 {
            return Err(RushError::PatternTooLong);
        }

        Ok(glob_match_impl(text, pattern))
    }

    /// Evaluate regex pattern matching with BASH_REMATCH support
    fn regex_match(&mut self, text: &str, pattern: &str) -> Result<bool> {
        // Validate pattern length (10KB limit from spec)
        if pattern.len() > 10240 {
            return Err(RushError::PatternTooLong);
        }

        // Compile regex pattern
        let re = Regex::new(pattern)
            .map_err(|e| RushError::InvalidPattern(format!("invalid regex: {}", e)))?;

        // Match and populate BASH_REMATCH array if successful
        if let Some(captures) = re.captures(text) {
            // Store captures in BASH_REMATCH array
            // BASH_REMATCH[0] is the full match, [1]+ are capture groups
            let mut rematch = Vec::new();
            for cap in captures.iter() {
                rematch.push(cap.map(|m| m.as_str().to_string()).unwrap_or_default());
            }

            // Store in variable manager as array
            // For now, just store BASH_REMATCH[0] as a simple variable
            // TODO: Proper array support when arrays are implemented
            if let Some(full_match) = rematch.first() {
                let _ = self
                    .executor
                    .variable_manager_mut()
                    .set("BASH_REMATCH".to_string(), full_match.clone());
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Evaluate logical operator with short-circuit evaluation
    fn evaluate_logical_op(
        &mut self,
        left: &Expression,
        operator: &LogicalOperator,
        right: &Expression,
    ) -> Result<bool> {
        match operator {
            LogicalOperator::And => {
                let left_result = self.evaluate(left)?;
                if !left_result {
                    return Ok(false); // Short-circuit: AND is false if left is false
                }
                self.evaluate(right)
            }
            LogicalOperator::Or => {
                let left_result = self.evaluate(left)?;
                if left_result {
                    return Ok(true); // Short-circuit: OR is true if left is true
                }
                self.evaluate(right)
            }
        }
    }
}

/// Execute the extended test command `[[`
///
/// # Arguments
/// * `executor` - Command executor with shell state
/// * `args` - Arguments to the test command (tokens between [[ and ]])
///
/// # Returns
/// * `Ok(0)` - Condition is true
/// * `Ok(1)` - Condition is false
/// * `Ok(2)` - Syntax error or evaluation failure
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("[[: empty test expression");
        return Ok(2);
    }

    // Step 1: Expand variables in arguments (but no word splitting/globbing)
    let expanded_args: Vec<String> = args
        .iter()
        .map(|arg| expand_variable(executor, arg))
        .collect();

    // Step 2: Parse the test expression
    let test_expr = match crate::executor::test_expr::parse_test_expression(&expanded_args) {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("[[: {}", e);
            return Ok(2); // Syntax error
        }
    };

    // Step 3: Evaluate the expression
    let mut evaluator = TestEvaluator::new(executor);
    match evaluator.evaluate(&test_expr.expression) {
        Ok(true) => Ok(0),  // Condition is true
        Ok(false) => Ok(1), // Condition is false
        Err(e) => {
            eprintln!("[[: {}", e);
            Ok(2) // Evaluation error
        }
    }
}

/// Expand variables in a token without word splitting or pathname expansion
///
/// This is a simplified variable expansion for test expressions.
/// Full expansion is handled by the expansion module, but here we just do
/// basic variable substitution.
fn expand_variable(executor: &CommandExecutor, token: &str) -> String {
    if !token.contains('$') {
        return token.to_string();
    }

    let mut result = token.to_string();

    // Simple $VAR expansion (not full ${VAR} syntax)
    // TODO: Use proper expansion module when available
    if let Some(var_start) = token.find('$') {
        let var_end = token[var_start + 1..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| var_start + 1 + i)
            .unwrap_or(token.len());

        let var_name = &token[var_start + 1..var_end];
        if !var_name.is_empty() {
            let value = executor.variable_manager().get(var_name).unwrap_or("");
            result = token[..var_start].to_string() + value + &token[var_end..];
        }
    }

    result
}

/// Glob pattern matching implementation
///
/// Supports:
/// - `*` - matches any sequence of characters (including empty)
/// - `?` - matches exactly one character
/// - `[abc]` - matches any character in the set
/// - `[!abc]` or `[^abc]` - matches any character NOT in the set
/// - `[a-z]` - matches any character in the range
///
/// # Arguments
/// * `text` - The text to match against
/// * `pattern` - The glob pattern
///
/// # Returns
/// * `true` if the text matches the pattern, `false` otherwise
fn glob_match_impl(text: &str, pattern: &str) -> bool {
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    glob_match_recursive(&text_chars, &pattern_chars, 0, 0)
}

/// Recursive glob matching implementation
fn glob_match_recursive(
    text: &[char],
    pattern: &[char],
    text_idx: usize,
    pattern_idx: usize,
) -> bool {
    // If we've reached the end of both text and pattern, it's a match
    if pattern_idx >= pattern.len() {
        return text_idx >= text.len();
    }

    // Handle backslash escaping
    if pattern.get(pattern_idx) == Some(&'\\') && pattern_idx + 1 < pattern.len() {
        // Escaped character - treat next char as literal
        let escaped_char = pattern[pattern_idx + 1];
        if text.get(text_idx) == Some(&escaped_char) {
            return glob_match_recursive(text, pattern, text_idx + 1, pattern_idx + 2);
        } else {
            return false;
        }
    }

    // Handle different pattern characters
    match pattern.get(pattern_idx) {
        Some('*') => {
            // Try matching zero or more characters
            // First, try matching zero characters (skip the *)
            if glob_match_recursive(text, pattern, text_idx, pattern_idx + 1) {
                return true;
            }
            // Then, try matching one or more characters
            for i in text_idx..text.len() {
                if glob_match_recursive(text, pattern, i + 1, pattern_idx + 1) {
                    return true;
                }
            }
            false
        }
        Some('?') => {
            // Match exactly one character
            if text_idx < text.len() {
                glob_match_recursive(text, pattern, text_idx + 1, pattern_idx + 1)
            } else {
                false
            }
        }
        Some('[') => {
            // Character class matching
            if text_idx >= text.len() {
                return false;
            }

            let text_char = text[text_idx];

            // Find the closing ]
            let mut class_end = pattern_idx + 1;
            while class_end < pattern.len() && pattern[class_end] != ']' {
                class_end += 1;
            }

            if class_end >= pattern.len() {
                // No closing ], treat [ as literal
                if text.get(text_idx) == Some(&'[') {
                    return glob_match_recursive(text, pattern, text_idx + 1, pattern_idx + 1);
                }
                return false;
            }

            // Extract the character class
            let class_chars: Vec<char> = pattern[pattern_idx + 1..class_end].to_vec();

            // Check for negation
            let (negated, class_chars) =
                if !class_chars.is_empty() && (class_chars[0] == '!' || class_chars[0] == '^') {
                    (true, &class_chars[1..])
                } else {
                    (false, &class_chars[..])
                };

            // Check if text_char matches any character in the class
            let mut matches = false;
            let mut i = 0;
            while i < class_chars.len() {
                if i + 2 < class_chars.len() && class_chars[i + 1] == '-' {
                    // Range: a-z
                    let start = class_chars[i];
                    let end = class_chars[i + 2];
                    if text_char >= start && text_char <= end {
                        matches = true;
                        break;
                    }
                    i += 3;
                } else {
                    // Single character
                    if text_char == class_chars[i] {
                        matches = true;
                        break;
                    }
                    i += 1;
                }
            }

            // Apply negation if needed
            let result = if negated { !matches } else { matches };

            if result {
                glob_match_recursive(text, pattern, text_idx + 1, class_end + 1)
            } else {
                false
            }
        }
        Some(&ch) => {
            // Literal character match
            if text.get(text_idx) == Some(&ch) {
                glob_match_recursive(text, pattern, text_idx + 1, pattern_idx + 1)
            } else {
                false
            }
        }
        None => {
            // Shouldn't reach here
            text_idx >= text.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_args() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2); // Syntax error
    }

    #[test]
    fn test_evaluator_literal_empty() {
        let mut executor = CommandExecutor::new();
        let mut evaluator = TestEvaluator::new(&mut executor);
        let expr = Expression::Literal("".to_string());
        let result = evaluator.evaluate(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // Empty string is false
    }

    #[test]
    fn test_evaluator_literal_non_empty() {
        let mut executor = CommandExecutor::new();
        let mut evaluator = TestEvaluator::new(&mut executor);
        let expr = Expression::Literal("value".to_string());
        let result = evaluator.evaluate(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true); // Non-empty string is true
    }
}
