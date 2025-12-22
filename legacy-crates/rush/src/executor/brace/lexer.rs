// Brace expansion lexer
// Tokenizes brace patterns and handles quote/escape states

/// Represents the state of quote context during lexing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteState {
    None,
    Single,
    Double,
}

/// Token types identified during brace lexing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BraceToken {
    /// Opening brace at given position
    OpenBrace(usize),
    /// Closing brace at given position
    CloseBrace(usize),
    /// Comma separator at given position
    Comma(usize),
    /// Two dots (sequence operator) at given position
    DoubleDot(usize),
    /// Regular text content
    Text(String),
}

/// Find matching closing brace for an opening brace, respecting quotes and escapes
///
/// # Arguments
/// * `input` - The input string
/// * `start` - Position of the opening brace (must be '{')
///
/// # Returns
/// The position of the matching closing brace, or None if not found
pub fn find_matching_brace(input: &str, start: usize) -> Option<usize> {
    let chars: Vec<char> = input.chars().collect();

    if start >= chars.len() || chars[start] != '{' {
        return None;
    }

    let mut depth = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for (i, &ch) in chars.iter().enumerate().skip(start) {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '{' if !in_single_quote && !in_double_quote => {
                depth += 1;
            }
            '}' if !in_single_quote && !in_double_quote => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

/// Split a string on commas, respecting nested braces
///
/// # Arguments
/// * `content` - The string content to split (without outer braces)
///
/// # Returns
/// Vector of strings split by top-level commas
pub fn split_on_commas(content: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in content.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                current.push(ch);
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                current.push(ch);
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                current.push(ch);
                in_double_quote = !in_double_quote;
            }
            '{' if !in_single_quote && !in_double_quote => {
                current.push(ch);
                depth += 1;
            }
            '}' if !in_single_quote && !in_double_quote => {
                current.push(ch);
                depth -= 1;
            }
            ',' if depth == 0 && !in_single_quote && !in_double_quote => {
                result.push(current.clone());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    result.push(current);
    result
}

/// Check if input contains any unquoted, unescaped braces
pub fn contains_braces(input: &str) -> bool {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '{' | '}' if !in_single_quote && !in_double_quote => {
                return true;
            }
            _ => {}
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_brace_simple() {
        assert_eq!(find_matching_brace("{a,b}", 0), Some(4));
        assert_eq!(find_matching_brace("{hello}", 0), Some(6));
    }

    #[test]
    fn test_find_matching_brace_nested() {
        assert_eq!(find_matching_brace("{a,{b,c}}", 0), Some(8));
        assert_eq!(find_matching_brace("{a,{b,c}}", 3), Some(7));
    }

    #[test]
    fn test_find_matching_brace_quoted() {
        // Braces inside quotes don't count
        // {'{'}  positions: 0='{' 1='\'' 2='{' 3='\'' 4='}'
        assert_eq!(find_matching_brace("{'{'}}", 0), Some(4));
        // {"{"} positions: 0='{' 1='"' 2='{' 3='"' 4='}'
        assert_eq!(find_matching_brace("{\"{\"}", 0), Some(4));
    }

    #[test]
    fn test_find_matching_brace_escaped() {
        // Escaped braces don't count
        assert_eq!(find_matching_brace("{\\{}", 0), Some(3));
    }

    #[test]
    fn test_find_matching_brace_unmatched() {
        assert_eq!(find_matching_brace("{a,b", 0), None);
        assert_eq!(find_matching_brace("{a,{b}", 0), None);
    }

    #[test]
    fn test_split_on_commas_simple() {
        assert_eq!(split_on_commas("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(split_on_commas("one,two"), vec!["one", "two"]);
    }

    #[test]
    fn test_split_on_commas_nested() {
        let result = split_on_commas("a,{b,c},d");
        assert_eq!(result, vec!["a", "{b,c}", "d"]);
    }

    #[test]
    fn test_split_on_commas_empty() {
        assert_eq!(split_on_commas("a,,b"), vec!["a", "", "b"]);
    }

    #[test]
    fn test_split_on_commas_single() {
        assert_eq!(split_on_commas("single"), vec!["single"]);
    }

    #[test]
    fn test_contains_braces() {
        assert!(contains_braces("{a,b}"));
        assert!(contains_braces("pre{a}post"));
        assert!(!contains_braces("no braces"));
        assert!(!contains_braces("'{a,b}'")); // Quoted
        assert!(!contains_braces("\\{escaped\\}")); // Escaped
    }
}
