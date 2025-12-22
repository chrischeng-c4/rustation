//! Test expression parser for `[[ ]]` extended test command
//!
//! This module provides parsing functionality for bash-compatible extended test
//! expressions. Unlike the traditional `[` command, `[[` is a shell keyword that
//! provides enhanced features:
//! - No word splitting or pathname expansion on variables
//! - Pattern matching with glob and regex support
//! - Logical operators (&&, ||, !) with proper precedence
//! - Parentheses for grouping

use crate::error::{Result, RushError};

/// Expression types in test command
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Unary operation: ! expr, -z str, -f file
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    /// Binary operation: a == b, x -lt y
    BinaryOp {
        left: String,
        operator: BinaryOperator,
        right: String,
    },
    /// Logical operation: expr && expr, expr || expr
    LogicalOp {
        left: Box<Expression>,
        operator: LogicalOperator,
        right: Box<Expression>,
    },
    /// Grouped expression: ( expr )
    Grouped(Box<Expression>),
    /// Literal value
    Literal(String),
}

/// Unary operators (single operand)
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    /// ! - Logical negation
    Negation,
    /// -z - String is empty
    StringEmpty,
    /// -n - String is non-empty
    StringNonEmpty,
    /// -e - File exists
    FileExists,
    /// -f - Regular file exists
    FileRegular,
    /// -d - Directory exists
    FileDirectory,
    /// -r - File is readable
    FileReadable,
    /// -w - File is writable
    FileWritable,
    /// -x - File is executable
    FileExecutable,
    /// -s - File exists and is non-empty
    FileNonEmpty,
}

/// Binary operators (two operands)
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // String operators
    /// == - String equality
    StringEqual,
    /// != - String inequality
    StringNotEqual,
    /// < - Lexicographic less than
    StringLess,
    /// > - Lexicographic greater than
    StringGreater,

    // Numeric operators
    /// -eq - Numeric equal
    NumericEqual,
    /// -ne - Numeric not equal
    NumericNotEqual,
    /// -lt - Numeric less than
    NumericLess,
    /// -le - Numeric less or equal
    NumericLessEqual,
    /// -gt - Numeric greater than
    NumericGreater,
    /// -ge - Numeric greater or equal
    NumericGreaterEqual,

    // Pattern operators
    /// == with glob pattern
    GlobMatch,
    /// != with glob pattern
    GlobNotMatch,
    /// =~ - Regex match
    RegexMatch,
}

/// Logical operators
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    /// && - Logical AND (short-circuit)
    And,
    /// || - Logical OR (short-circuit)
    Or,
}

/// Represents a complete test expression from [[ ]] command
#[derive(Debug, Clone, PartialEq)]
pub struct TestExpression {
    pub expression: Expression,
}

impl TestExpression {
    /// Create a new test expression
    pub fn new(expression: Expression) -> Self {
        Self { expression }
    }
}

/// Parse a test expression from tokens between [[ and ]]
///
/// # Arguments
/// * `tokens` - Tokens from the test expression (excluding [[ and ]])
///
/// # Returns
/// * `Ok(TestExpression)` - Successfully parsed expression
/// * `Err(RushError)` - Syntax error or invalid expression
pub fn parse_test_expression(tokens: &[String]) -> Result<TestExpression> {
    if tokens.is_empty() {
        return Err(RushError::Syntax("empty test expression".to_string()));
    }

    let mut parser = Parser::new(tokens);
    let expression = parser.parse_or()?;

    if parser.current < tokens.len() {
        return Err(RushError::Syntax(format!("unexpected token: {}", tokens[parser.current])));
    }

    Ok(TestExpression::new(expression))
}

/// Recursive descent parser for test expressions
struct Parser<'a> {
    tokens: &'a [String],
    current: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    const MAX_DEPTH: usize = 32;

    fn new(tokens: &'a [String]) -> Self {
        Self { tokens, current: 0, depth: 0 }
    }

    fn check_depth(&self) -> Result<()> {
        if self.depth >= Self::MAX_DEPTH {
            return Err(RushError::Syntax(format!(
                "expression nesting too deep (max {} levels)",
                Self::MAX_DEPTH
            )));
        }
        Ok(())
    }

    fn peek(&self) -> Option<&String> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&String> {
        let token = self.tokens.get(self.current);
        if token.is_some() {
            self.current += 1;
        }
        token
    }

    /// Parse OR expression (lowest precedence)
    /// expr ::= and_expr ( '||' and_expr )*
    fn parse_or(&mut self) -> Result<Expression> {
        let mut left = self.parse_and()?;

        while self.peek().map(|s| s.as_str()) == Some("||") {
            self.advance(); // consume ||
            let right = self.parse_and()?;
            left = Expression::LogicalOp {
                left: Box::new(left),
                operator: LogicalOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse AND expression
    /// and_expr ::= comparison ( '&&' comparison )*
    fn parse_and(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;

        while self.peek().map(|s| s.as_str()) == Some("&&") {
            self.advance(); // consume &&
            let right = self.parse_comparison()?;
            left = Expression::LogicalOp {
                left: Box::new(left),
                operator: LogicalOperator::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse comparison expression
    /// comparison ::= unary ( binary_op unary )?
    fn parse_comparison(&mut self) -> Result<Expression> {
        // Check for negation or unary operators
        if let Some(token_str) = self.peek().map(|s| s.clone()) {
            if token_str == "!" {
                self.advance();
                let operand = self.parse_comparison()?;
                return Ok(Expression::UnaryOp {
                    operator: UnaryOperator::Negation,
                    operand: Box::new(operand),
                });
            }

            // Check for unary file/string operators
            if let Some(unary_op) = self.parse_unary_operator(&token_str) {
                self.advance();
                let operand_token = self.advance().ok_or_else(|| {
                    RushError::Syntax(format!("missing operand for {}", token_str))
                })?;
                return Ok(Expression::UnaryOp {
                    operator: unary_op,
                    operand: Box::new(Expression::Literal(operand_token.clone())),
                });
            }

            // Check for grouped expression
            if token_str == "(" {
                self.check_depth()?;
                self.depth += 1;
                self.advance();
                let inner = self.parse_or()?;
                self.depth -= 1;

                if self.advance().map(|s| s.as_str()) != Some(")") {
                    return Err(RushError::Syntax("missing ')'".to_string()));
                }

                return Ok(Expression::Grouped(Box::new(inner)));
            }
        }

        // Parse binary operation: left op right
        let left = self
            .advance()
            .ok_or_else(|| RushError::Syntax("unexpected end of expression".to_string()))?
            .clone();

        // Check if there's a binary operator
        if let Some(token_str) = self.peek().map(|s| s.clone()) {
            if let Some(binary_op) = self.parse_binary_operator(&token_str) {
                self.advance(); // consume operator
                let right = self
                    .advance()
                    .ok_or_else(|| {
                        RushError::Syntax(format!("missing right operand for {}", token_str))
                    })?
                    .clone();

                return Ok(Expression::BinaryOp { left, operator: binary_op, right });
            }
        }

        // Just a literal value
        Ok(Expression::Literal(left))
    }

    fn parse_unary_operator(&self, token: &str) -> Option<UnaryOperator> {
        match token {
            "-z" => Some(UnaryOperator::StringEmpty),
            "-n" => Some(UnaryOperator::StringNonEmpty),
            "-e" => Some(UnaryOperator::FileExists),
            "-f" => Some(UnaryOperator::FileRegular),
            "-d" => Some(UnaryOperator::FileDirectory),
            "-r" => Some(UnaryOperator::FileReadable),
            "-w" => Some(UnaryOperator::FileWritable),
            "-x" => Some(UnaryOperator::FileExecutable),
            "-s" => Some(UnaryOperator::FileNonEmpty),
            _ => None,
        }
    }

    fn parse_binary_operator(&self, token: &str) -> Option<BinaryOperator> {
        match token {
            "==" => Some(BinaryOperator::StringEqual),
            "!=" => Some(BinaryOperator::StringNotEqual),
            "<" => Some(BinaryOperator::StringLess),
            ">" => Some(BinaryOperator::StringGreater),
            "-eq" => Some(BinaryOperator::NumericEqual),
            "-ne" => Some(BinaryOperator::NumericNotEqual),
            "-lt" => Some(BinaryOperator::NumericLess),
            "-le" => Some(BinaryOperator::NumericLessEqual),
            "-gt" => Some(BinaryOperator::NumericGreater),
            "-ge" => Some(BinaryOperator::NumericGreaterEqual),
            "=~" => Some(BinaryOperator::RegexMatch),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_expression_error() {
        let result = parse_test_expression(&[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RushError::Syntax(_)));
    }

    #[test]
    fn test_test_expression_creation() {
        let expr = Expression::Literal("test".to_string());
        let test_expr = TestExpression::new(expr.clone());
        assert_eq!(test_expr.expression, expr);
    }
}
