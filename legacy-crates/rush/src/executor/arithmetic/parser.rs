//! Parser for arithmetic expressions.
//!
//! Uses a Pratt parser (operator-precedence parsing) to build an AST
//! from a sequence of tokens.

use super::lexer::Token;
use super::{ArithmeticError, Result};

/// AST nodes for arithmetic expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Number(i64),
    Variable(String),

    // Unary operators
    Negate(Box<Expr>),
    Positive(Box<Expr>),
    LogicalNot(Box<Expr>),
    BitwiseNot(Box<Expr>),

    // Binary arithmetic operators
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),

    // Comparison operators
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),

    // Logical operators
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),

    // Bitwise operators
    BitAnd(Box<Expr>, Box<Expr>),
    BitOr(Box<Expr>, Box<Expr>),
    BitXor(Box<Expr>, Box<Expr>),
    Shl(Box<Expr>, Box<Expr>),
    Shr(Box<Expr>, Box<Expr>),

    // Assignment
    Assign(String, Box<Expr>),
    AddAssign(String, Box<Expr>),
    SubAssign(String, Box<Expr>),
    MulAssign(String, Box<Expr>),
    DivAssign(String, Box<Expr>),
    ModAssign(String, Box<Expr>),
    AndAssign(String, Box<Expr>),
    OrAssign(String, Box<Expr>),
    XorAssign(String, Box<Expr>),
    ShlAssign(String, Box<Expr>),
    ShrAssign(String, Box<Expr>),

    // Increment/Decrement
    PreIncrement(String),
    PostIncrement(String),
    PreDecrement(String),
    PostDecrement(String),

    // Ternary
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),

    // Comma (sequence)
    Comma(Vec<Expr>),
}

/// Parser for arithmetic expressions using Pratt parsing.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

// Operator precedence levels (higher = binds tighter)
// Based on C operator precedence
const PREC_COMMA: u8 = 1;
const PREC_ASSIGN: u8 = 2;
const PREC_TERNARY: u8 = 3;
const PREC_OR: u8 = 4;
const PREC_AND: u8 = 5;
const PREC_BIT_OR: u8 = 6;
const PREC_BIT_XOR: u8 = 7;
const PREC_BIT_AND: u8 = 8;
const PREC_EQUALITY: u8 = 9;
const PREC_COMPARISON: u8 = 10;
const PREC_SHIFT: u8 = 11;
const PREC_ADDITIVE: u8 = 12;
const PREC_MULTIPLICATIVE: u8 = 13;
const PREC_POWER: u8 = 14;
const PREC_UNARY: u8 = 15;
const PREC_POSTFIX: u8 = 16;

impl Parser {
    /// Create a new parser from tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse the tokens into an expression AST.
    pub fn parse(&mut self) -> Result<Expr> {
        if self.is_at_end() || self.peek() == &Token::Eof {
            // Empty expression evaluates to 0
            return Ok(Expr::Number(0));
        }
        let expr = self.parse_expression(PREC_COMMA)?;
        if !self.is_at_end() && self.peek() != &Token::Eof {
            return Err(ArithmeticError::UnexpectedToken(format!("{:?}", self.peek())));
        }
        Ok(expr)
    }

    fn parse_expression(&mut self, min_prec: u8) -> Result<Expr> {
        let mut left = self.parse_prefix()?;

        while !self.is_at_end() {
            let token = self.peek().clone();

            // Check for postfix operators first
            if matches!(token, Token::PlusPlus | Token::MinusMinus) {
                if let Expr::Variable(name) = &left {
                    self.advance();
                    left = match token {
                        Token::PlusPlus => Expr::PostIncrement(name.clone()),
                        Token::MinusMinus => Expr::PostDecrement(name.clone()),
                        _ => unreachable!(),
                    };
                    continue;
                }
            }

            let (prec, right_assoc) = self.infix_precedence(&token);
            if prec < min_prec {
                break;
            }

            self.advance();

            // Handle ternary operator specially
            if token == Token::Question {
                let then_expr = self.parse_expression(PREC_COMMA)?;
                self.expect(Token::Colon)?;
                let else_expr = self.parse_expression(PREC_TERNARY)?;
                left = Expr::Ternary(Box::new(left), Box::new(then_expr), Box::new(else_expr));
                continue;
            }

            // Handle comma operator specially (builds a list)
            if token == Token::Comma {
                let right = self.parse_expression(PREC_ASSIGN)?;
                left = match left {
                    Expr::Comma(mut exprs) => {
                        exprs.push(right);
                        Expr::Comma(exprs)
                    }
                    _ => Expr::Comma(vec![left, right]),
                };
                continue;
            }

            // Handle assignment operators
            if self.is_assignment_op(&token) {
                let name = match left {
                    Expr::Variable(ref name) => name.clone(),
                    _ => return Err(ArithmeticError::InvalidAssignmentTarget),
                };
                let right = self.parse_expression(PREC_ASSIGN)?;
                left = self.make_assignment(name, &token, right);
                continue;
            }

            // Regular binary operators
            let next_prec = if right_assoc { prec } else { prec + 1 };
            let right = self.parse_expression(next_prec)?;
            left = self.make_binary(left, &token, right)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr> {
        let token = self.peek().clone();

        match token {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(Expr::Variable(name))
            }
            Token::Plus => {
                self.advance();
                let expr = self.parse_expression(PREC_UNARY)?;
                Ok(Expr::Positive(Box::new(expr)))
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_expression(PREC_UNARY)?;
                Ok(Expr::Negate(Box::new(expr)))
            }
            Token::Bang => {
                self.advance();
                let expr = self.parse_expression(PREC_UNARY)?;
                Ok(Expr::LogicalNot(Box::new(expr)))
            }
            Token::Tilde => {
                self.advance();
                let expr = self.parse_expression(PREC_UNARY)?;
                Ok(Expr::BitwiseNot(Box::new(expr)))
            }
            Token::PlusPlus => {
                self.advance();
                let name = self.expect_identifier()?;
                Ok(Expr::PreIncrement(name))
            }
            Token::MinusMinus => {
                self.advance();
                let name = self.expect_identifier()?;
                Ok(Expr::PreDecrement(name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression(PREC_COMMA)?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::Eof => Err(ArithmeticError::UnexpectedEnd),
            _ => Err(ArithmeticError::UnexpectedToken(format!("{:?}", token))),
        }
    }

    fn infix_precedence(&self, token: &Token) -> (u8, bool) {
        match token {
            Token::Comma => (PREC_COMMA, false),

            Token::Eq
            | Token::PlusEq
            | Token::MinusEq
            | Token::StarEq
            | Token::SlashEq
            | Token::PercentEq
            | Token::AndEq
            | Token::OrEq
            | Token::CaretEq
            | Token::LtLtEq
            | Token::GtGtEq => (PREC_ASSIGN, true),

            Token::Question => (PREC_TERNARY, true),

            Token::OrOr => (PREC_OR, false),
            Token::AndAnd => (PREC_AND, false),

            Token::Or => (PREC_BIT_OR, false),
            Token::Caret => (PREC_BIT_XOR, false),
            Token::And => (PREC_BIT_AND, false),

            Token::EqEq | Token::Ne => (PREC_EQUALITY, false),
            Token::Lt | Token::Gt | Token::Le | Token::Ge => (PREC_COMPARISON, false),

            Token::LtLt | Token::GtGt => (PREC_SHIFT, false),

            Token::Plus | Token::Minus => (PREC_ADDITIVE, false),
            Token::Star | Token::Slash | Token::Percent => (PREC_MULTIPLICATIVE, false),

            Token::StarStar => (PREC_POWER, true), // Right associative

            Token::PlusPlus | Token::MinusMinus => (PREC_POSTFIX, false),

            _ => (0, false),
        }
    }

    fn is_assignment_op(&self, token: &Token) -> bool {
        matches!(
            token,
            Token::Eq
                | Token::PlusEq
                | Token::MinusEq
                | Token::StarEq
                | Token::SlashEq
                | Token::PercentEq
                | Token::AndEq
                | Token::OrEq
                | Token::CaretEq
                | Token::LtLtEq
                | Token::GtGtEq
        )
    }

    fn make_assignment(&self, name: String, op: &Token, value: Expr) -> Expr {
        match op {
            Token::Eq => Expr::Assign(name, Box::new(value)),
            Token::PlusEq => Expr::AddAssign(name, Box::new(value)),
            Token::MinusEq => Expr::SubAssign(name, Box::new(value)),
            Token::StarEq => Expr::MulAssign(name, Box::new(value)),
            Token::SlashEq => Expr::DivAssign(name, Box::new(value)),
            Token::PercentEq => Expr::ModAssign(name, Box::new(value)),
            Token::AndEq => Expr::AndAssign(name, Box::new(value)),
            Token::OrEq => Expr::OrAssign(name, Box::new(value)),
            Token::CaretEq => Expr::XorAssign(name, Box::new(value)),
            Token::LtLtEq => Expr::ShlAssign(name, Box::new(value)),
            Token::GtGtEq => Expr::ShrAssign(name, Box::new(value)),
            _ => unreachable!(),
        }
    }

    fn make_binary(&self, left: Expr, op: &Token, right: Expr) -> Result<Expr> {
        Ok(match op {
            Token::Plus => Expr::Add(Box::new(left), Box::new(right)),
            Token::Minus => Expr::Sub(Box::new(left), Box::new(right)),
            Token::Star => Expr::Mul(Box::new(left), Box::new(right)),
            Token::Slash => Expr::Div(Box::new(left), Box::new(right)),
            Token::Percent => Expr::Mod(Box::new(left), Box::new(right)),
            Token::StarStar => Expr::Pow(Box::new(left), Box::new(right)),

            Token::Lt => Expr::Lt(Box::new(left), Box::new(right)),
            Token::Gt => Expr::Gt(Box::new(left), Box::new(right)),
            Token::Le => Expr::Le(Box::new(left), Box::new(right)),
            Token::Ge => Expr::Ge(Box::new(left), Box::new(right)),
            Token::EqEq => Expr::Eq(Box::new(left), Box::new(right)),
            Token::Ne => Expr::Ne(Box::new(left), Box::new(right)),

            Token::AndAnd => Expr::And(Box::new(left), Box::new(right)),
            Token::OrOr => Expr::Or(Box::new(left), Box::new(right)),

            Token::And => Expr::BitAnd(Box::new(left), Box::new(right)),
            Token::Or => Expr::BitOr(Box::new(left), Box::new(right)),
            Token::Caret => Expr::BitXor(Box::new(left), Box::new(right)),
            Token::LtLt => Expr::Shl(Box::new(left), Box::new(right)),
            Token::GtGt => Expr::Shr(Box::new(left), Box::new(right)),

            _ => {
                return Err(ArithmeticError::UnexpectedToken(format!(
                    "unexpected operator: {:?}",
                    op
                )))
            }
        })
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens.get(self.pos - 1).unwrap_or(&Token::Eof)
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.peek() == &Token::Eof
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.peek() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(ArithmeticError::UnexpectedToken(format!(
                "expected {:?}, got {:?}",
                expected,
                self.peek()
            )))
        }
    }

    fn expect_identifier(&mut self) -> Result<String> {
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            _ => Err(ArithmeticError::UnexpectedToken(format!(
                "expected identifier, got {:?}",
                self.peek()
            ))),
        }
    }
}

/// Parse an arithmetic expression string into an AST.
pub fn parse(input: &str) -> Result<Expr> {
    let mut lexer = super::lexer::Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        let expr = parse("42").unwrap();
        assert_eq!(expr, Expr::Number(42));
    }

    #[test]
    fn test_variable() {
        let expr = parse("x").unwrap();
        assert_eq!(expr, Expr::Variable("x".to_string()));
    }

    #[test]
    fn test_addition() {
        let expr = parse("2 + 3").unwrap();
        assert_eq!(expr, Expr::Add(Box::new(Expr::Number(2)), Box::new(Expr::Number(3))));
    }

    #[test]
    fn test_precedence() {
        // 2 + 3 * 4 = 2 + (3 * 4) = 14
        let expr = parse("2 + 3 * 4").unwrap();
        assert_eq!(
            expr,
            Expr::Add(
                Box::new(Expr::Number(2)),
                Box::new(Expr::Mul(Box::new(Expr::Number(3)), Box::new(Expr::Number(4))))
            )
        );
    }

    #[test]
    fn test_parentheses() {
        // (2 + 3) * 4
        let expr = parse("(2 + 3) * 4").unwrap();
        assert_eq!(
            expr,
            Expr::Mul(
                Box::new(Expr::Add(Box::new(Expr::Number(2)), Box::new(Expr::Number(3)))),
                Box::new(Expr::Number(4))
            )
        );
    }

    #[test]
    fn test_unary_minus() {
        let expr = parse("-5").unwrap();
        assert_eq!(expr, Expr::Negate(Box::new(Expr::Number(5))));
    }

    #[test]
    fn test_power_right_assoc() {
        // 2 ** 3 ** 2 = 2 ** (3 ** 2) = 2 ** 9 = 512
        let expr = parse("2 ** 3 ** 2").unwrap();
        assert_eq!(
            expr,
            Expr::Pow(
                Box::new(Expr::Number(2)),
                Box::new(Expr::Pow(Box::new(Expr::Number(3)), Box::new(Expr::Number(2))))
            )
        );
    }

    #[test]
    fn test_comparison() {
        let expr = parse("5 > 3").unwrap();
        assert_eq!(expr, Expr::Gt(Box::new(Expr::Number(5)), Box::new(Expr::Number(3))));
    }

    #[test]
    fn test_logical_and() {
        let expr = parse("1 && 0").unwrap();
        assert_eq!(expr, Expr::And(Box::new(Expr::Number(1)), Box::new(Expr::Number(0))));
    }

    #[test]
    fn test_logical_not() {
        let expr = parse("!0").unwrap();
        assert_eq!(expr, Expr::LogicalNot(Box::new(Expr::Number(0))));
    }

    #[test]
    fn test_bitwise() {
        let expr = parse("5 & 3").unwrap();
        assert_eq!(expr, Expr::BitAnd(Box::new(Expr::Number(5)), Box::new(Expr::Number(3))));
    }

    #[test]
    fn test_assignment() {
        let expr = parse("x = 5").unwrap();
        assert_eq!(expr, Expr::Assign("x".to_string(), Box::new(Expr::Number(5))));
    }

    #[test]
    fn test_compound_assignment() {
        let expr = parse("x += 3").unwrap();
        assert_eq!(expr, Expr::AddAssign("x".to_string(), Box::new(Expr::Number(3))));
    }

    #[test]
    fn test_pre_increment() {
        let expr = parse("++x").unwrap();
        assert_eq!(expr, Expr::PreIncrement("x".to_string()));
    }

    #[test]
    fn test_post_increment() {
        let expr = parse("x++").unwrap();
        assert_eq!(expr, Expr::PostIncrement("x".to_string()));
    }

    #[test]
    fn test_ternary() {
        let expr = parse("1 ? 2 : 3").unwrap();
        assert_eq!(
            expr,
            Expr::Ternary(
                Box::new(Expr::Number(1)),
                Box::new(Expr::Number(2)),
                Box::new(Expr::Number(3))
            )
        );
    }

    #[test]
    fn test_comma() {
        let expr = parse("1, 2, 3").unwrap();
        assert_eq!(expr, Expr::Comma(vec![Expr::Number(1), Expr::Number(2), Expr::Number(3)]));
    }

    #[test]
    fn test_empty() {
        let expr = parse("").unwrap();
        assert_eq!(expr, Expr::Number(0));
    }

    #[test]
    fn test_whitespace_only() {
        let expr = parse("   ").unwrap();
        assert_eq!(expr, Expr::Number(0));
    }
}
