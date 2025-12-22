//! Simple lexer for shell command syntax highlighting
//!
//! Tokenizes shell input to identify:
//! - Commands (first word)
//! - Flags (-f, --flag)
//! - Arguments
//! - Operators (|, &&, ||, ;, &)
//! - Redirections (>, >>)
//! - Strings (quoted text)
//! - Comments (#)

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    /// Command name (first word)
    Command,
    /// Command argument
    Argument,
    /// Flag (-f or --flag)
    Flag,
    /// Pipe operator (|)
    Pipe,
    /// And operator (&&)
    And,
    /// Or operator (||)
    Or,
    /// Semicolon (;)
    Semicolon,
    /// Background operator (&)
    Background,
    /// Output redirection (>, >>)
    Redirect,
    /// String literal (quoted)
    String,
    /// Comment (#...)
    Comment,
    /// Whitespace
    Whitespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl Token {
    fn new(token_type: TokenType, text: String, start: usize, end: usize) -> Self {
        Self { token_type, text, start, end }
    }
}

/// Simple lexer for shell commands
pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    /// Create a new lexer for the given input
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }

    /// Tokenize the entire input into a list of tokens
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut expecting_command = true;

        while self.position < self.input.len() {
            let start = self.position;
            let ch = self.current_char();

            // Check for comments
            if ch == '#' {
                tokens.push(self.read_comment(start));
                break; // Rest of line is comment
            }

            // Check for whitespace
            if ch.is_whitespace() {
                tokens.push(self.read_whitespace(start));
                continue;
            }

            // Check for strings
            if ch == '"' || ch == '\'' {
                tokens.push(self.read_string(start, ch));
                expecting_command = false;
                continue;
            }

            // Check for operators and redirections
            if let Some(token) = self.try_read_operator(start) {
                tokens.push(token);
                expecting_command = true;
                continue;
            }

            // Read a word (command, flag, or argument)
            let word = self.read_word(start);

            // Determine token type based on context
            let token_type = if word.text.starts_with('-') {
                TokenType::Flag
            } else if expecting_command {
                expecting_command = false;
                TokenType::Command
            } else {
                TokenType::Argument
            };

            tokens.push(Token::new(token_type, word.text.clone(), word.start, word.end));
        }

        tokens
    }

    fn current_char(&self) -> char {
        self.input[self.position..].chars().next().unwrap()
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += self.current_char().len_utf8();
        }
    }

    fn read_whitespace(&mut self, start: usize) -> Token {
        while self.position < self.input.len() && self.current_char().is_whitespace() {
            self.advance();
        }
        Token::new(
            TokenType::Whitespace,
            self.input[start..self.position].to_string(),
            start,
            self.position,
        )
    }

    fn read_string(&mut self, start: usize, quote: char) -> Token {
        self.advance(); // Skip opening quote

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == quote {
                self.advance(); // Skip closing quote
                break;
            }
            if ch == '\\' {
                self.advance(); // Skip escape
                if self.position < self.input.len() {
                    self.advance(); // Skip escaped char
                }
            } else {
                self.advance();
            }
        }

        Token::new(
            TokenType::String,
            self.input[start..self.position].to_string(),
            start,
            self.position,
        )
    }

    fn read_comment(&mut self, start: usize) -> Token {
        while self.position < self.input.len() {
            self.advance();
        }
        Token::new(
            TokenType::Comment,
            self.input[start..self.position].to_string(),
            start,
            self.position,
        )
    }

    fn try_read_operator(&mut self, start: usize) -> Option<Token> {
        let remaining = &self.input[self.position..];

        // Check for two-character operators first
        if remaining.starts_with("&&") {
            self.position += 2;
            return Some(Token::new(TokenType::And, "&&".to_string(), start, self.position));
        }
        if remaining.starts_with("||") {
            self.position += 2;
            return Some(Token::new(TokenType::Or, "||".to_string(), start, self.position));
        }
        if remaining.starts_with(">>") {
            self.position += 2;
            return Some(Token::new(TokenType::Redirect, ">>".to_string(), start, self.position));
        }

        // Check for single-character operators
        let ch = self.current_char();
        let token = match ch {
            '|' => Some(Token::new(TokenType::Pipe, "|".to_string(), start, start + 1)),
            ';' => Some(Token::new(TokenType::Semicolon, ";".to_string(), start, start + 1)),
            '&' => Some(Token::new(TokenType::Background, "&".to_string(), start, start + 1)),
            '>' => Some(Token::new(TokenType::Redirect, ">".to_string(), start, start + 1)),
            _ => None,
        };

        if token.is_some() {
            self.advance();
        }

        token
    }

    fn read_word(&mut self, start: usize) -> Token {
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_whitespace() || self.is_operator_char(ch) {
                break;
            }
            self.advance();
        }

        Token::new(
            TokenType::Argument, // Type will be determined by caller
            self.input[start..self.position].to_string(),
            start,
            self.position,
        )
    }

    fn is_operator_char(&self, ch: char) -> bool {
        matches!(ch, '|' | '&' | ';' | '>' | '<' | '#' | '"' | '\'')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let mut lexer = Lexer::new("ls -la".to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 3); // ls, space, -la
        assert_eq!(tokens[0].token_type, TokenType::Command);
        assert_eq!(tokens[0].text, "ls");
        assert_eq!(tokens[2].token_type, TokenType::Flag);
        assert_eq!(tokens[2].text, "-la");
    }

    #[test]
    fn test_command_with_arguments() {
        let mut lexer = Lexer::new("echo hello world".to_string());
        let tokens = lexer.tokenize();

        let non_ws: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type != TokenType::Whitespace)
            .collect();
        assert_eq!(non_ws.len(), 3);
        assert_eq!(non_ws[0].token_type, TokenType::Command);
        assert_eq!(non_ws[1].token_type, TokenType::Argument);
        assert_eq!(non_ws[2].token_type, TokenType::Argument);
    }

    #[test]
    fn test_pipe_operator() {
        let mut lexer = Lexer::new("ls | grep test".to_string());
        let tokens = lexer.tokenize();

        let pipe_token = tokens.iter().find(|t| t.token_type == TokenType::Pipe);
        assert!(pipe_token.is_some());
        assert_eq!(pipe_token.unwrap().text, "|");
    }

    #[test]
    fn test_and_operator() {
        let mut lexer = Lexer::new("make && make test".to_string());
        let tokens = lexer.tokenize();

        let and_token = tokens.iter().find(|t| t.token_type == TokenType::And);
        assert!(and_token.is_some());
        assert_eq!(and_token.unwrap().text, "&&");
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new(r#"echo "hello world""#.to_string());
        let tokens = lexer.tokenize();

        let string_token = tokens.iter().find(|t| t.token_type == TokenType::String);
        assert!(string_token.is_some());
        assert_eq!(string_token.unwrap().text, r#""hello world""#);
    }

    #[test]
    fn test_comment() {
        let mut lexer = Lexer::new("ls # list files".to_string());
        let tokens = lexer.tokenize();

        let comment_token = tokens.iter().find(|t| t.token_type == TokenType::Comment);
        assert!(comment_token.is_some());
        assert!(comment_token.unwrap().text.contains("# list files"));
    }

    #[test]
    fn test_redirect() {
        let mut lexer = Lexer::new("echo test > out.txt".to_string());
        let tokens = lexer.tokenize();

        let redirect = tokens.iter().find(|t| t.token_type == TokenType::Redirect);
        assert!(redirect.is_some());
        assert_eq!(redirect.unwrap().text, ">");
    }

    #[test]
    fn test_background() {
        let mut lexer = Lexer::new("sleep 10 &".to_string());
        let tokens = lexer.tokenize();

        let bg = tokens
            .iter()
            .find(|t| t.token_type == TokenType::Background);
        assert!(bg.is_some());
        assert_eq!(bg.unwrap().text, "&");
    }

    #[test]
    fn test_multiple_flags() {
        let mut lexer = Lexer::new("ls -l -a --color".to_string());
        let tokens = lexer.tokenize();

        let flags: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Flag)
            .collect();
        assert_eq!(flags.len(), 3);
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("".to_string());
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_whitespace_only() {
        let mut lexer = Lexer::new("   \t  \n".to_string());
        let tokens = lexer.tokenize();
        // Should only contain whitespace tokens
        assert!(tokens.iter().all(|t| t.token_type == TokenType::Whitespace));
    }

    #[test]
    fn test_semicolon_operator() {
        let mut lexer = Lexer::new("ls ; pwd".to_string());
        let tokens = lexer.tokenize();

        let semi = tokens.iter().find(|t| t.token_type == TokenType::Semicolon);
        assert!(semi.is_some());
        assert_eq!(semi.unwrap().text, ";");
    }

    #[test]
    fn test_or_operator() {
        let mut lexer = Lexer::new("false || true".to_string());
        let tokens = lexer.tokenize();

        let or_token = tokens.iter().find(|t| t.token_type == TokenType::Or);
        assert!(or_token.is_some());
        assert_eq!(or_token.unwrap().text, "||");
    }

    #[test]
    fn test_append_redirect() {
        let mut lexer = Lexer::new("echo test >> file.txt".to_string());
        let tokens = lexer.tokenize();

        let redirect = tokens.iter().find(|t| t.token_type == TokenType::Redirect);
        assert!(redirect.is_some());
        assert_eq!(redirect.unwrap().text, ">>");
    }

    #[test]
    fn test_single_quote_string() {
        let mut lexer = Lexer::new("echo 'hello world'".to_string());
        let tokens = lexer.tokenize();

        let string_token = tokens.iter().find(|t| t.token_type == TokenType::String);
        assert!(string_token.is_some());
        assert_eq!(string_token.unwrap().text, "'hello world'");
    }

    #[test]
    fn test_escaped_string() {
        let mut lexer = Lexer::new(r#"echo "hello \"world\"""#.to_string());
        let tokens = lexer.tokenize();

        let string_token = tokens.iter().find(|t| t.token_type == TokenType::String);
        assert!(string_token.is_some());
    }

    #[test]
    fn test_complex_command() {
        let mut lexer = Lexer::new("git commit -m \"Initial commit\" && git push".to_string());
        let tokens = lexer.tokenize();

        // Should have command, flags, string, and operators
        // Note: "commit" is an argument to git, not a separate command
        // Only "git" and "git" (after &&) are commands
        let commands: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Command)
            .collect();
        assert_eq!(commands.len(), 2); // git (twice: before and after &&)

        let flags: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Flag)
            .collect();
        assert_eq!(flags.len(), 1); // -m

        let strings: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::String)
            .collect();
        assert_eq!(strings.len(), 1); // "Initial commit"

        let operators: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::And)
            .collect();
        assert_eq!(operators.len(), 1); // &&
    }

    #[test]
    fn test_token_positions() {
        let mut lexer = Lexer::new("ls -la".to_string());
        let tokens = lexer.tokenize();

        // Check that positions are correct
        for token in &tokens {
            assert!(token.start <= token.end);
            assert!(token.end <= "ls -la".len());
        }
    }

    #[test]
    fn test_unicode_input() {
        let mut lexer = Lexer::new("echo 你好世界".to_string());
        let tokens = lexer.tokenize();

        let non_ws: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type != TokenType::Whitespace)
            .collect();
        assert_eq!(non_ws.len(), 2); // echo and 你好世界
    }

    #[test]
    fn test_comment_at_start() {
        let mut lexer = Lexer::new("# this is a comment".to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Comment);
    }

    #[test]
    fn test_mixed_operators() {
        let mut lexer = Lexer::new("cmd1 | cmd2 && cmd3 || cmd4 ; cmd5".to_string());
        let tokens = lexer.tokenize();

        let pipe_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Pipe)
            .count();
        let and_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::And)
            .count();
        let or_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Or)
            .count();
        let semi_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Semicolon)
            .count();

        assert_eq!(pipe_count, 1);
        assert_eq!(and_count, 1);
        assert_eq!(or_count, 1);
        assert_eq!(semi_count, 1);
    }
}
