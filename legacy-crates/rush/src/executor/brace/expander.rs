// Brace expansion main entry point

use super::lexer;
use super::parser::{self, BraceExpr};

/// Expands brace patterns in the input string.
///
/// Brace expansion generates multiple text strings from patterns containing braces.
/// Examples:
/// - `{a,b,c}` → `a b c`
/// - `{1..5}` → `1 2 3 4 5`
/// - `file{1,2}.txt` → `file1.txt file2.txt`
///
/// # Arguments
/// * `input` - The input string potentially containing brace patterns
///
/// # Returns
/// The expanded string with all brace patterns replaced by their expansions
pub fn expand_brace(input: &str) -> String {
    // Quick check: if no braces, return as-is
    if !lexer::contains_braces(input) {
        return input.to_string();
    }

    // Split input into words (respecting quotes), expand each word, then flatten
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current_word.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                current_word.push(ch);
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                current_word.push(ch);
                in_single_quote = !in_single_quote;
            }
            '"' if !in_double_quote => {
                current_word.push(ch);
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                // Word boundary - expand and add to result
                if !current_word.is_empty() {
                    let expanded = expand_word(&current_word);
                    result.extend(expanded);
                    current_word.clear();
                }
            }
            _ => {
                current_word.push(ch);
            }
        }
    }

    // Don't forget the last word
    if !current_word.is_empty() {
        let expanded = expand_word(&current_word);
        result.extend(expanded);
    }

    result.join(" ")
}

/// Expand a single word containing brace patterns
///
/// Recursively expands all brace patterns in the word, handling:
/// - Multiple adjacent braces (Cartesian product)
/// - Nested braces (inner first)
/// - Preamble and postscript text
fn expand_word(word: &str) -> Vec<String> {
    // Find the first unquoted, unescaped opening brace
    let chars: Vec<char> = word.chars().collect();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for (i, &ch) in chars.iter().enumerate() {
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
                // Found an opening brace - try to expand it
                if let Some(close_pos) = lexer::find_matching_brace(word, i) {
                    let content = &word[i + 1..close_pos];
                    let expr = parser::parse_brace_content(content);

                    match expr {
                        BraceExpr::Literal(_) => {
                            // Invalid pattern, return as-is
                            return vec![word.to_string()];
                        }
                        _ => {
                            // Valid pattern - expand it
                            let preamble = &word[..i];
                            let postscript = &word[close_pos + 1..];
                            let alternatives = expand_expr(&expr);

                            // Combine preamble + alternatives + postscript
                            let mut results = Vec::new();
                            for alt in alternatives {
                                let combined = format!("{}{}{}", preamble, alt, postscript);
                                // Recursively expand the combined result (for nested braces)
                                let expanded = expand_word(&combined);
                                results.extend(expanded);
                            }
                            return results;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // No braces found or all quoted/escaped
    vec![word.to_string()]
}

/// Expand a BraceExpr into its alternatives
fn expand_expr(expr: &BraceExpr) -> Vec<String> {
    match expr {
        BraceExpr::List(elements) => elements.clone(),
        BraceExpr::NumericSeq { start, end, step, width } => {
            expand_numeric_seq(*start, *end, *step, *width)
        }
        BraceExpr::CharSeq { start, end, step } => expand_char_seq(*start, *end, *step),
        BraceExpr::Literal(s) => vec![s.clone()],
    }
}

/// Expand a numeric sequence
fn expand_numeric_seq(start: i64, end: i64, step: i64, width: usize) -> Vec<String> {
    let mut result = Vec::new();

    if step > 0 {
        let mut current = start;
        while current <= end {
            result.push(format!("{:0width$}", current, width = width));
            current += step;
        }
    } else if step < 0 {
        let mut current = start;
        while current >= end {
            result.push(format!("{:0width$}", current, width = width));
            current += step;
        }
    }

    result
}

/// Expand a character sequence
fn expand_char_seq(start: char, end: char, step: i64) -> Vec<String> {
    let mut result = Vec::new();
    let start_val = start as i64;
    let end_val = end as i64;

    if step > 0 {
        let mut current = start_val;
        while current <= end_val {
            if let Some(ch) = char::from_u32(current as u32) {
                result.push(ch.to_string());
            }
            current += step;
        }
    } else if step < 0 {
        let mut current = start_val;
        while current >= end_val {
            if let Some(ch) = char::from_u32(current as u32) {
                result.push(ch.to_string());
            }
            current += step;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_braces() {
        assert_eq!(expand_brace("hello"), "hello");
    }

    #[test]
    fn test_basic_list() {
        assert_eq!(expand_brace("{a,b,c}"), "a b c");
    }

    #[test]
    fn test_preamble_postscript() {
        assert_eq!(expand_brace("file{1,2,3}.txt"), "file1.txt file2.txt file3.txt");
    }

    #[test]
    fn test_numeric_sequence() {
        assert_eq!(expand_brace("{1..5}"), "1 2 3 4 5");
    }

    #[test]
    fn test_reverse_sequence() {
        assert_eq!(expand_brace("{5..1}"), "5 4 3 2 1");
    }

    #[test]
    fn test_char_sequence() {
        assert_eq!(expand_brace("{a..e}"), "a b c d e");
    }

    #[test]
    fn test_zero_padded() {
        assert_eq!(expand_brace("{01..05}"), "01 02 03 04 05");
    }

    #[test]
    fn test_single_element_no_expand() {
        assert_eq!(expand_brace("{a}"), "{a}");
    }

    #[test]
    fn test_quoted_no_expand() {
        // Note: This test will pass once we integrate with the parser that handles quotes
        // For now, the lexer already detects quoted braces
        assert_eq!(expand_brace("'{a,b}'"), "'{a,b}'");
    }
}
