//! Syntax highlighting implementation using custom lexer

use super::lexer::{Lexer, TokenType};
use super::validator::Validator;
use nu_ansi_term::{Color, Style};
use reedline::Highlighter;

/// Syntax highlighter for rush shell
///
/// Provides real-time syntax highlighting optimized for dark terminals:
/// - Commands: Green
/// - Flags: Cyan (changed from Blue for better visibility on dark backgrounds)
/// - Arguments: Default (white/terminal default)
/// - Operators: Cyan (pipes, redirects, etc.)
/// - Strings: Yellow
/// - Comments: Gray
pub struct RushHighlighter;

impl RushHighlighter {
    /// Create a new syntax highlighter
    pub fn new() -> Self {
        Self
    }

    /// Get Style for a token type
    /// Colors optimized for dark terminal backgrounds (out-of-box experience)
    fn get_style(token_type: &TokenType, text: &str) -> Style {
        match token_type {
            TokenType::Command => {
                if Validator::validate_command(text) {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::Red)
                }
            }
            TokenType::Flag => Style::new().fg(Color::Cyan),
            TokenType::Argument => {
                // Check if it looks like a path (contains / or is . or ..)
                if text.contains('/') || text == "." || text == ".." {
                    if Validator::validate_path(text) {
                        Style::new().underline()
                    } else {
                        // Invalid path - keep default or maybe red?
                        // Fish makes it red if it looks like a path but isn't
                        // For now, let's keep default to avoid false positives on non-path args
                        Style::default()
                    }
                } else {
                    Style::default()
                }
            }
            TokenType::Pipe => Style::new().fg(Color::Cyan),
            TokenType::And => Style::new().fg(Color::Cyan),
            TokenType::Or => Style::new().fg(Color::Cyan),
            TokenType::Semicolon => Style::new().fg(Color::Cyan),
            TokenType::Background => Style::new().fg(Color::Cyan),
            TokenType::Redirect => Style::new().fg(Color::Cyan),
            TokenType::String => Style::new().fg(Color::Yellow),
            TokenType::Comment => Style::new().fg(Color::DarkGray),
            TokenType::Whitespace => Style::default(),
        }
    }
}

impl Default for RushHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for RushHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> reedline::StyledText {
        let mut lexer = Lexer::new(line.to_string());
        let tokens = lexer.tokenize();

        let mut styled_text = reedline::StyledText::new();

        for token in tokens {
            let style = Self::get_style(&token.token_type, &token.text);
            styled_text.push((style, token.text));
        }

        styled_text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_new() {
        let highlighter = RushHighlighter::new();
        // Should compile and create successfully
        drop(highlighter);
    }

    #[test]
    fn test_highlighter_default() {
        let highlighter = RushHighlighter::default();
        drop(highlighter);
    }

    #[test]
    fn test_highlight_simple_command() {
        let highlighter = RushHighlighter::new();
        let styled = highlighter.highlight("ls -la", 0);

        // Should have styled text (exact format depends on reedline)
        assert!(!styled.buffer.is_empty());
    }

    #[test]
    fn test_highlight_with_pipe() {
        let highlighter = RushHighlighter::new();
        let styled = highlighter.highlight("ls | grep test", 0);

        assert!(!styled.buffer.is_empty());
    }

    #[test]
    fn test_highlight_with_string() {
        let highlighter = RushHighlighter::new();
        let styled = highlighter.highlight(r#"echo "hello""#, 0);

        assert!(!styled.buffer.is_empty());
    }

    #[test]
    fn test_get_style_for_command() {
        // "ls" should be valid (Green)
        let style = RushHighlighter::get_style(&TokenType::Command, "ls");
        assert_eq!(style.foreground, Some(Color::Green));

        // "invalid_cmd" should be invalid (Red)
        let style = RushHighlighter::get_style(&TokenType::Command, "invalid_cmd_xyz");
        assert_eq!(style.foreground, Some(Color::Red));
    }

    #[test]
    fn test_get_style_for_flag() {
        let style = RushHighlighter::get_style(&TokenType::Flag, "-f");
        assert_eq!(style.foreground, Some(Color::Cyan));
    }

    #[test]
    fn test_get_style_for_string() {
        let style = RushHighlighter::get_style(&TokenType::String, "\"hello\"");
        assert_eq!(style.foreground, Some(Color::Yellow));
    }
}
