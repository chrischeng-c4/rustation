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
use crate::executor::test_expr::{
    BinaryOperator, Expression, LogicalOperator, TestExpression, UnaryOperator,
};

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
            Expression::UnaryOp { operator, operand } => {
                self.evaluate_unary_op(operator, operand)
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => self.evaluate_binary_op(left, operator, right),
            Expression::LogicalOp {
                left,
                operator,
                right,
            } => self.evaluate_logical_op(left, operator, right),
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
            UnaryOperator::FileRegular => {
                Ok(std::path::Path::new(&path).is_file())
            }
            UnaryOperator::FileDirectory => {
                Ok(std::path::Path::new(&path).is_dir())
            }
            UnaryOperator::FileReadable => {
                match fs::metadata(&path) {
                    Ok(metadata) => {
                        let permissions = metadata.permissions();
                        Ok(permissions.mode() & 0o400 != 0)
                    }
                    Err(_) => Ok(false),
                }
            }
            UnaryOperator::FileWritable => {
                match fs::metadata(&path) {
                    Ok(metadata) => {
                        let permissions = metadata.permissions();
                        Ok(permissions.mode() & 0o200 != 0)
                    }
                    Err(_) => Ok(false),
                }
            }
            UnaryOperator::FileExecutable => {
                match fs::metadata(&path) {
                    Ok(metadata) => {
                        let permissions = metadata.permissions();
                        Ok(permissions.mode() & 0o100 != 0)
                    }
                    Err(_) => Ok(false),
                }
            }
            UnaryOperator::FileNonEmpty => {
                match fs::metadata(&path) {
                    Ok(metadata) => Ok(metadata.len() > 0),
                    Err(_) => Ok(false),
                }
            }
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
            BinaryOperator::NumericEqual => self.evaluate_numeric_comparison(left, right, |a, b| a == b),
            BinaryOperator::NumericNotEqual => self.evaluate_numeric_comparison(left, right, |a, b| a != b),
            BinaryOperator::NumericLess => self.evaluate_numeric_comparison(left, right, |a, b| a < b),
            BinaryOperator::NumericLessEqual => self.evaluate_numeric_comparison(left, right, |a, b| a <= b),
            BinaryOperator::NumericGreater => self.evaluate_numeric_comparison(left, right, |a, b| a > b),
            BinaryOperator::NumericGreaterEqual => self.evaluate_numeric_comparison(left, right, |a, b| a >= b),

            // Pattern operators (to be implemented in Phase 4)
            BinaryOperator::GlobMatch | BinaryOperator::GlobNotMatch | BinaryOperator::RegexMatch => {
                Err(RushError::InvalidOperator("pattern matching not yet implemented".to_string()))
            }
        }
    }

    /// Evaluate numeric comparison
    fn evaluate_numeric_comparison<F>(&self, left: &str, right: &str, compare: F) -> Result<bool>
    where
        F: Fn(i64, i64) -> bool,
    {
        let left_num = left.parse::<i64>()
            .map_err(|_| RushError::TypeMismatch(format!("'{}' is not a valid integer", left)))?;
        let right_num = right.parse::<i64>()
            .map_err(|_| RushError::TypeMismatch(format!("'{}' is not a valid integer", right)))?;
        Ok(compare(left_num, right_num))
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
        Ok(true) => Ok(0),   // Condition is true
        Ok(false) => Ok(1),  // Condition is false
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
