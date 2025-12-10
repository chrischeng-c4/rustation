//! Lexer for arithmetic expressions.
//!
//! Tokenizes arithmetic expressions into a sequence of tokens that can be
//! parsed into an AST.

use super::{ArithmeticError, Result};

/// Tokens produced by the arithmetic lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(i64),
    Identifier(String),

    // Arithmetic operators
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Percent,  // %
    StarStar, // **

    // Comparison operators
    Lt,   // <
    Gt,   // >
    Le,   // <=
    Ge,   // >=
    EqEq, // ==
    Ne,   // !=

    // Logical operators
    AndAnd, // &&
    OrOr,   // ||
    Bang,   // !

    // Bitwise operators
    And,   // &
    Or,    // |
    Caret, // ^
    Tilde, // ~
    LtLt,  // <<
    GtGt,  // >>

    // Assignment operators
    Eq,        // =
    PlusEq,    // +=
    MinusEq,   // -=
    StarEq,    // *=
    SlashEq,   // /=
    PercentEq, // %=
    AndEq,     // &=
    OrEq,      // |=
    CaretEq,   // ^=
    LtLtEq,    // <<=
    GtGtEq,    // >>=

    // Increment/Decrement
    PlusPlus,   // ++
    MinusMinus, // --

    // Ternary
    Question, // ?
    Colon,    // :

    // Comma
    Comma, // ,

    // Grouping
    LParen, // (
    RParen, // )

    // End of input
    Eof,
}

/// Lexer for arithmetic expressions.
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input.
    pub fn new(input: &'a str) -> Self {
        Self { input, chars: input.char_indices().peekable(), current_pos: 0 }
    }

    /// Tokenize the entire input into a vector of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    /// Get the next token from the input.
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        match self.peek_char() {
            None => Ok(Token::Eof),
            Some(c) => match c {
                '0'..='9' => self.read_number(),
                'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),
                '+' => self.read_plus(),
                '-' => self.read_minus(),
                '*' => self.read_star(),
                '/' => self.read_slash(),
                '%' => self.read_percent(),
                '<' => self.read_less(),
                '>' => self.read_greater(),
                '=' => self.read_equal(),
                '!' => self.read_bang(),
                '&' => self.read_ampersand(),
                '|' => self.read_pipe(),
                '^' => self.read_caret(),
                '~' => {
                    self.advance();
                    Ok(Token::Tilde)
                }
                '?' => {
                    self.advance();
                    Ok(Token::Question)
                }
                ':' => {
                    self.advance();
                    Ok(Token::Colon)
                }
                ',' => {
                    self.advance();
                    Ok(Token::Comma)
                }
                '(' => {
                    self.advance();
                    Ok(Token::LParen)
                }
                ')' => {
                    self.advance();
                    Ok(Token::RParen)
                }
                _ => {
                    Err(ArithmeticError::UnexpectedToken(format!("unexpected character: '{}'", c)))
                }
            },
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next().map(|(pos, c)| {
            self.current_pos = pos + c.len_utf8();
            c
        })
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<Token> {
        let start = self.current_pos;

        // Check for hex (0x) or octal (0o or leading 0)
        if self.peek_char() == Some('0') {
            self.advance();
            match self.peek_char() {
                Some('x') | Some('X') => {
                    self.advance();
                    return self.read_hex_number();
                }
                Some('o') | Some('O') => {
                    self.advance();
                    return self.read_octal_number();
                }
                Some('0'..='7') => {
                    // Traditional octal (leading 0)
                    return self.read_traditional_octal();
                }
                _ => {
                    // Just the number 0
                    return Ok(Token::Number(0));
                }
            }
        }

        // Decimal number
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let num_str = &self.input[start..self.current_pos];
        num_str
            .parse::<i64>()
            .map(Token::Number)
            .map_err(|_| ArithmeticError::InvalidNumber(num_str.to_string()))
    }

    fn read_hex_number(&mut self) -> Result<Token> {
        let start = self.current_pos;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_hexdigit() {
                self.advance();
            } else {
                break;
            }
        }

        let hex_str = &self.input[start..self.current_pos];
        if hex_str.is_empty() {
            return Err(ArithmeticError::InvalidNumber("0x".to_string()));
        }

        i64::from_str_radix(hex_str, 16)
            .map(Token::Number)
            .map_err(|_| ArithmeticError::InvalidNumber(format!("0x{}", hex_str)))
    }

    fn read_octal_number(&mut self) -> Result<Token> {
        let start = self.current_pos;

        while let Some(c) = self.peek_char() {
            if matches!(c, '0'..='7') {
                self.advance();
            } else {
                break;
            }
        }

        let octal_str = &self.input[start..self.current_pos];
        if octal_str.is_empty() {
            return Err(ArithmeticError::InvalidNumber("0o".to_string()));
        }

        i64::from_str_radix(octal_str, 8)
            .map(Token::Number)
            .map_err(|_| ArithmeticError::InvalidNumber(format!("0o{}", octal_str)))
    }

    fn read_traditional_octal(&mut self) -> Result<Token> {
        let start = self.current_pos;

        while let Some(c) = self.peek_char() {
            if matches!(c, '0'..='7') {
                self.advance();
            } else if c.is_ascii_digit() {
                // Invalid octal digit
                return Err(ArithmeticError::InvalidNumber(format!("invalid octal digit: {}", c)));
            } else {
                break;
            }
        }

        let octal_str = &self.input[start..self.current_pos];
        // Include leading 0 that was already consumed
        i64::from_str_radix(octal_str, 8)
            .map(Token::Number)
            .map_err(|_| ArithmeticError::InvalidNumber(format!("0{}", octal_str)))
    }

    fn read_identifier(&mut self) -> Result<Token> {
        let start = self.current_pos;

        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let ident = &self.input[start..self.current_pos];
        Ok(Token::Identifier(ident.to_string()))
    }

    fn read_plus(&mut self) -> Result<Token> {
        self.advance(); // consume '+'
        match self.peek_char() {
            Some('+') => {
                self.advance();
                Ok(Token::PlusPlus)
            }
            Some('=') => {
                self.advance();
                Ok(Token::PlusEq)
            }
            _ => Ok(Token::Plus),
        }
    }

    fn read_minus(&mut self) -> Result<Token> {
        self.advance(); // consume '-'
        match self.peek_char() {
            Some('-') => {
                self.advance();
                Ok(Token::MinusMinus)
            }
            Some('=') => {
                self.advance();
                Ok(Token::MinusEq)
            }
            _ => Ok(Token::Minus),
        }
    }

    fn read_star(&mut self) -> Result<Token> {
        self.advance(); // consume '*'
        match self.peek_char() {
            Some('*') => {
                self.advance();
                Ok(Token::StarStar)
            }
            Some('=') => {
                self.advance();
                Ok(Token::StarEq)
            }
            _ => Ok(Token::Star),
        }
    }

    fn read_slash(&mut self) -> Result<Token> {
        self.advance(); // consume '/'
        match self.peek_char() {
            Some('=') => {
                self.advance();
                Ok(Token::SlashEq)
            }
            _ => Ok(Token::Slash),
        }
    }

    fn read_percent(&mut self) -> Result<Token> {
        self.advance(); // consume '%'
        match self.peek_char() {
            Some('=') => {
                self.advance();
                Ok(Token::PercentEq)
            }
            _ => Ok(Token::Percent),
        }
    }

    fn read_less(&mut self) -> Result<Token> {
        self.advance(); // consume '<'
        match self.peek_char() {
            Some('<') => {
                self.advance();
                match self.peek_char() {
                    Some('=') => {
                        self.advance();
                        Ok(Token::LtLtEq)
                    }
                    _ => Ok(Token::LtLt),
                }
            }
            Some('=') => {
                self.advance();
                Ok(Token::Le)
            }
            _ => Ok(Token::Lt),
        }
    }

    fn read_greater(&mut self) -> Result<Token> {
        self.advance(); // consume '>'
        match self.peek_char() {
            Some('>') => {
                self.advance();
                match self.peek_char() {
                    Some('=') => {
                        self.advance();
                        Ok(Token::GtGtEq)
                    }
                    _ => Ok(Token::GtGt),
                }
            }
            Some('=') => {
                self.advance();
                Ok(Token::Ge)
            }
            _ => Ok(Token::Gt),
        }
    }

    fn read_equal(&mut self) -> Result<Token> {
        self.advance(); // consume '='
        match self.peek_char() {
            Some('=') => {
                self.advance();
                Ok(Token::EqEq)
            }
            _ => Ok(Token::Eq),
        }
    }

    fn read_bang(&mut self) -> Result<Token> {
        self.advance(); // consume '!'
        match self.peek_char() {
            Some('=') => {
                self.advance();
                Ok(Token::Ne)
            }
            _ => Ok(Token::Bang),
        }
    }

    fn read_ampersand(&mut self) -> Result<Token> {
        self.advance(); // consume '&'
        match self.peek_char() {
            Some('&') => {
                self.advance();
                Ok(Token::AndAnd)
            }
            Some('=') => {
                self.advance();
                Ok(Token::AndEq)
            }
            _ => Ok(Token::And),
        }
    }

    fn read_pipe(&mut self) -> Result<Token> {
        self.advance(); // consume '|'
        match self.peek_char() {
            Some('|') => {
                self.advance();
                Ok(Token::OrOr)
            }
            Some('=') => {
                self.advance();
                Ok(Token::OrEq)
            }
            _ => Ok(Token::Or),
        }
    }

    fn read_caret(&mut self) -> Result<Token> {
        self.advance(); // consume '^'
        match self.peek_char() {
            Some('=') => {
                self.advance();
                Ok(Token::CaretEq)
            }
            _ => Ok(Token::Caret),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 0 123");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(42),
                Token::Number(0),
                Token::Number(123),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_hex_numbers() {
        let mut lexer = Lexer::new("0x10 0xFF 0X1a");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(16),
                Token::Number(255),
                Token::Number(26),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_octal_numbers() {
        let mut lexer = Lexer::new("010 0o17 0O10");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(8),
                Token::Number(15),
                Token::Number(8),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("x foo_bar _test x123");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Identifier("foo_bar".to_string()),
                Token::Identifier("_test".to_string()),
                Token::Identifier("x123".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_arithmetic_operators() {
        let mut lexer = Lexer::new("+ - * / % **");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Percent,
                Token::StarStar,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_comparison_operators() {
        let mut lexer = Lexer::new("< > <= >= == !=");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Lt,
                Token::Gt,
                Token::Le,
                Token::Ge,
                Token::EqEq,
                Token::Ne,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_logical_operators() {
        let mut lexer = Lexer::new("&& || !");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::AndAnd, Token::OrOr, Token::Bang, Token::Eof]);
    }

    #[test]
    fn test_bitwise_operators() {
        let mut lexer = Lexer::new("& | ^ ~ << >>");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::And,
                Token::Or,
                Token::Caret,
                Token::Tilde,
                Token::LtLt,
                Token::GtGt,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_assignment_operators() {
        let mut lexer = Lexer::new("= += -= *= /= %= &= |= ^= <<= >>=");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Eq,
                Token::PlusEq,
                Token::MinusEq,
                Token::StarEq,
                Token::SlashEq,
                Token::PercentEq,
                Token::AndEq,
                Token::OrEq,
                Token::CaretEq,
                Token::LtLtEq,
                Token::GtGtEq,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_increment_decrement() {
        let mut lexer = Lexer::new("++ --");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::PlusPlus, Token::MinusMinus, Token::Eof]);
    }

    #[test]
    fn test_ternary_comma() {
        let mut lexer = Lexer::new("? : ,");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Question, Token::Colon, Token::Comma, Token::Eof]);
    }

    #[test]
    fn test_parentheses() {
        let mut lexer = Lexer::new("( )");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::LParen, Token::RParen, Token::Eof]);
    }

    #[test]
    fn test_expression() {
        let mut lexer = Lexer::new("x + 5 * (y - 3)");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Plus,
                Token::Number(5),
                Token::Star,
                Token::LParen,
                Token::Identifier("y".to_string()),
                Token::Minus,
                Token::Number(3),
                Token::RParen,
                Token::Eof
            ]
        );
    }
}
