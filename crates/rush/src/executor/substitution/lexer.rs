//! Lexer for command substitution expressions
//!
//! Scans input text and identifies $(...) regions, producing a token stream.
//! Handles:
//! - Basic substitution: $(command)
//! - Nested substitution: $(echo $(date))
//! - Quote handling: Single quotes disable substitution, double quotes allow it
//! - Escape sequences: \$ prevents substitution
//! - Parenthesis balancing: Tracks depth to find matching closing paren

use super::SubstitutionError;
use super::SubstitutionToken;
use crate::error::Result;

const MAX_INPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB limit

/// Lexer for command substitution expressions
pub struct SubstitutionLexer;

impl SubstitutionLexer {
    /// Tokenize input, identifying $(...) regions
    ///
    /// # Arguments
    /// * `input` - The input string to tokenize
    ///
    /// # Returns
    /// * `Ok(tokens)` - Vec of SubstitutionToken representing the input
    /// * `Err(e)` - SubstitutionError if parsing fails (mismatched parens, unclosed quotes)
    ///
    /// # Examples
    /// ```ignore
    /// let tokens = SubstitutionLexer::tokenize("echo $(date)")?;
    /// // Returns: [Literal("echo "), Substitution("date")]
    /// ```
    pub fn tokenize(input: &str) -> Result<Vec<SubstitutionToken>> {
        if input.len() > MAX_INPUT_SIZE {
            return Err(crate::error::RushError::Execution(format!(
                "substitution: input too large: {} bytes (limit: {} bytes)",
                input.len(),
                MAX_INPUT_SIZE
            )));
        }

        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Try to find a substitution starting at current position
            if let Some((end_pos, command)) = Self::find_substitution(&chars, i)? {
                // Add any literal text before the substitution
                if i > 0 && !matches!(tokens.last(), Some(SubstitutionToken::Literal(_))) {
                    // No literal before, or last token was substitution
                }

                tokens.push(SubstitutionToken::Substitution(command));
                i = end_pos;
            } else {
                // Not a substitution start, collect literal text
                let start = i;
                loop {
                    if i >= chars.len() {
                        break;
                    }

                    // Check if we're at the start of a substitution
                    if i < chars.len() - 1
                        && chars[i] == '$'
                        && chars[i + 1] == '('
                        && (i == 0 || chars[i - 1] != '\\')
                    {
                        // Check if we're in quotes
                        if !Self::is_in_single_quotes(&chars, i) {
                            break;
                        }
                    }

                    i += 1;
                }

                if i > start {
                    let literal = chars[start..i].iter().collect::<String>();
                    tokens.push(SubstitutionToken::Literal(literal));
                }
            }
        }

        Ok(tokens)
    }

    /// Find the next substitution starting at position `start`
    ///
    /// # Arguments
    /// * `chars` - The character array of the input
    /// * `start` - The starting position to search from
    ///
    /// # Returns
    /// * `Ok(Some((end_pos, command)))` - Found a substitution, returns position after ) and the command
    /// * `Ok(None)` - No substitution at this position
    /// * `Err(e)` - Parsing error (mismatched parens, unclosed quote)
    fn find_substitution(
        chars: &[char],
        start: usize,
    ) -> Result<Option<(usize, String)>> {
        // Check for $( pattern and not in single quotes
        if start >= chars.len() - 1
            || chars[start] != '$'
            || chars[start + 1] != '('
            || (start > 0 && chars[start - 1] == '\\')
            || Self::is_in_single_quotes(chars, start)
        {
            return Ok(None);
        }

        // We're in double quotes or unquoted context, proceed with matching parens
        let command_start = start + 2; // Position after $(
        let closing_paren = Self::find_closing_paren(chars, command_start - 1)?;

        let command = chars[command_start..closing_paren]
            .iter()
            .collect::<String>();

        Ok(Some((closing_paren + 1, command)))
    }

    /// Find the closing ) for the opening ( at position `paren_pos`
    ///
    /// Handles nested parentheses and quoted strings within the command.
    ///
    /// # Arguments
    /// * `chars` - The character array
    /// * `paren_pos` - Position of the opening $( - should be at the (
    ///
    /// # Returns
    /// * `Ok(pos)` - Position of the closing )
    /// * `Err(e)` - MismatchedParentheses or UnclosedQuote error
    fn find_closing_paren(chars: &[char], paren_pos: usize) -> Result<usize> {
        let mut depth = 1;
        let mut i = paren_pos + 1; // Start after the (

        while i < chars.len() && depth > 0 {
            let ch = chars[i];

            match ch {
                '\'' => {
                    // Single quote: skip to closing quote
                    i += 1;
                    while i < chars.len() && chars[i] != '\'' {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            i += 2; // Skip escaped character
                        } else {
                            i += 1;
                        }
                    }
                    if i >= chars.len() {
                        return Err(crate::error::RushError::Execution(
                            "substitution: unclosed single quote".to_string(),
                        ));
                    }
                    i += 1; // Skip closing quote
                }
                '"' => {
                    // Double quote: skip to closing quote
                    i += 1;
                    while i < chars.len() && chars[i] != '"' {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            i += 2; // Skip escaped character
                        } else {
                            i += 1;
                        }
                    }
                    if i >= chars.len() {
                        return Err(crate::error::RushError::Execution(
                            "substitution: unclosed double quote".to_string(),
                        ));
                    }
                    i += 1; // Skip closing quote
                }
                '(' => {
                    depth += 1;
                    i += 1;
                }
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(i);
                    }
                    i += 1;
                }
                '\\' => {
                    // Skip escaped character
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        if depth > 0 {
            return Err(crate::error::RushError::Execution(
                "substitution: mismatched parentheses in command substitution".to_string(),
            ));
        }

        Err(crate::error::RushError::Execution(
            "substitution: unexpected end of input while scanning parentheses".to_string(),
        ))
    }

    /// Check if a position is inside single quotes (not escaped)
    fn is_in_single_quotes(chars: &[char], pos: usize) -> bool {
        let mut in_single = false;
        let mut in_double = false;
        let mut i = 0;

        while i < pos && i < chars.len() {
            if i > 0 && chars[i - 1] == '\\' {
                i += 1;
                continue;
            }

            match chars[i] {
                '\'' if !in_double => in_single = !in_single,
                '"' if !in_single => in_double = !in_double,
                _ => {}
            }
            i += 1;
        }

        in_single
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_substitution() {
        let tokens = SubstitutionLexer::tokenize("echo $(date)").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], SubstitutionToken::Literal("echo ".to_string()));
        assert_eq!(tokens[1], SubstitutionToken::Substitution("date".to_string()));
    }

    #[test]
    fn test_no_substitution() {
        let tokens = SubstitutionLexer::tokenize("echo hello world").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            SubstitutionToken::Literal("echo hello world".to_string())
        );
    }

    #[test]
    fn test_substitution_only() {
        let tokens = SubstitutionLexer::tokenize("$(date)").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], SubstitutionToken::Substitution("date".to_string()));
    }

    #[test]
    fn test_multiple_substitutions() {
        let tokens = SubstitutionLexer::tokenize("$(cmd1) and $(cmd2)").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], SubstitutionToken::Substitution("cmd1".to_string()));
        assert_eq!(tokens[1], SubstitutionToken::Literal(" and ".to_string()));
        assert_eq!(tokens[2], SubstitutionToken::Substitution("cmd2".to_string()));
    }

    #[test]
    fn test_nested_substitution() {
        let tokens = SubstitutionLexer::tokenize("echo $(echo $(date))").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], SubstitutionToken::Literal("echo ".to_string()));
        assert_eq!(
            tokens[1],
            SubstitutionToken::Substitution("echo $(date)".to_string())
        );
    }

    #[test]
    fn test_deeply_nested_substitution() {
        let tokens = SubstitutionLexer::tokenize("$(echo $(echo $(date)))").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            SubstitutionToken::Substitution("echo $(echo $(date))".to_string())
        );
    }

    #[test]
    fn test_substitution_in_double_quotes() {
        let tokens = SubstitutionLexer::tokenize("echo \"$(date)\"").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], SubstitutionToken::Literal("echo \"".to_string()));
        assert_eq!(tokens[1], SubstitutionToken::Substitution("date".to_string()));
        assert_eq!(tokens[2], SubstitutionToken::Literal("\"".to_string()));
    }

    #[test]
    fn test_substitution_not_in_single_quotes() {
        let tokens = SubstitutionLexer::tokenize("echo '$(date)'").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            SubstitutionToken::Literal("echo '$(date)'".to_string())
        );
    }

    #[test]
    fn test_escaped_dollar_sign() {
        let tokens = SubstitutionLexer::tokenize("echo \\$(date)").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            SubstitutionToken::Literal("echo \\$(date)".to_string())
        );
    }

    #[test]
    fn test_command_with_quoted_args() {
        let tokens = SubstitutionLexer::tokenize("$(echo \"hello world\")").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            SubstitutionToken::Substitution("echo \"hello world\"".to_string())
        );
    }

    #[test]
    fn test_command_with_parentheses_in_quotes() {
        let tokens = SubstitutionLexer::tokenize("$(echo \"(test)\")").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            SubstitutionToken::Substitution("echo \"(test)\"".to_string())
        );
    }

    #[test]
    fn test_mismatched_parentheses_unclosed() {
        let result = SubstitutionLexer::tokenize("echo $(date");
        assert!(result.is_err());
    }

    #[test]
    fn test_unclosed_quote_in_substitution() {
        let result = SubstitutionLexer::tokenize("$(echo \"hello)");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_substitution() {
        let tokens = SubstitutionLexer::tokenize("echo $()").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], SubstitutionToken::Literal("echo ".to_string()));
        assert_eq!(tokens[1], SubstitutionToken::Substitution("".to_string()));
    }

    #[test]
    fn test_literal_with_spaces() {
        let tokens = SubstitutionLexer::tokenize("  hello  ").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            SubstitutionToken::Literal("  hello  ".to_string())
        );
    }

    #[test]
    fn test_complex_real_world_example() {
        let tokens = SubstitutionLexer::tokenize("ls $(find . -name '*.txt')").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], SubstitutionToken::Literal("ls ".to_string()));
        assert_eq!(
            tokens[1],
            SubstitutionToken::Substitution("find . -name '*.txt'".to_string())
        );
    }

    #[test]
    fn test_substitution_in_variable_assignment() {
        let tokens = SubstitutionLexer::tokenize("var=$(date)").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], SubstitutionToken::Literal("var=".to_string()));
        assert_eq!(tokens[1], SubstitutionToken::Substitution("date".to_string()));
    }

    #[test]
    fn test_multiple_nested_levels() {
        let tokens = SubstitutionLexer::tokenize("$(a $(b $(c)))").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            SubstitutionToken::Substitution("a $(b $(c))".to_string())
        );
    }

    #[test]
    fn test_mixed_quotes() {
        let tokens = SubstitutionLexer::tokenize("echo \"$(cmd)\" '$(not_cmd)'").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], SubstitutionToken::Literal("echo \"".to_string()));
        assert_eq!(tokens[1], SubstitutionToken::Substitution("cmd".to_string()));
        assert_eq!(
            tokens[2],
            SubstitutionToken::Literal("\" '$(not_cmd)'".to_string())
        );
    }
}
