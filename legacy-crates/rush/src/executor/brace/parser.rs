// Brace expansion parser
// Parses tokenized brace content into expansion types

use super::lexer;

/// Represents a parsed brace expression ready for expansion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BraceExpr {
    /// Comma-separated list: {a,b,c}
    List(Vec<String>),

    /// Numeric sequence: {1..10..2}
    NumericSeq {
        start: i64,
        end: i64,
        step: i64,
        width: usize, // For zero-padding
    },

    /// Character sequence: {a..z..2}
    CharSeq { start: char, end: char, step: i64 },

    /// Literal (invalid brace pattern, no expansion)
    Literal(String),
}

/// Check if a brace pattern content is valid for expansion
///
/// Valid patterns:
/// - Contains comma: {a,b} → valid
/// - Contains ..: {1..5} → check sequence validity
/// - Otherwise: invalid (e.g., {a}, {})
pub fn is_valid_brace_pattern(content: &str) -> bool {
    if content.is_empty() {
        return false; // {} is invalid
    }

    // Check for comma (list pattern)
    if content.contains(',') {
        // Single comma with empty content around it: {,} is valid (two empty elements)
        // But just a single element like {a} is invalid
        let elements = lexer::split_on_commas(content);
        return elements.len() > 1;
    }

    // Check for sequence pattern (..)
    if content.contains("..") {
        return is_valid_sequence(content);
    }

    false // Single element like {a} is invalid
}

/// Check if a sequence pattern is valid
fn is_valid_sequence(content: &str) -> bool {
    let parts: Vec<&str> = content.split("..").collect();

    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }

    let start = parts[0].trim();
    let end = parts[1].trim();

    let start_is_num = start.parse::<i64>().is_ok();
    let end_is_num = end.parse::<i64>().is_ok();
    let start_is_char = start.len() == 1 && start.chars().next().unwrap().is_ascii();
    let end_is_char = end.len() == 1 && end.chars().next().unwrap().is_ascii();

    // Both must be numbers OR both must be characters (not mixed)
    if start_is_num && end_is_num {
        // Valid numeric sequence: {1..10} or {1..10..2}
        if parts.len() == 3 {
            // Must have valid step
            return parts[2].trim().parse::<i64>().is_ok();
        }
        return true;
    } else if start_is_char && end_is_char && !start_is_num && !end_is_num {
        // Valid character sequence: {a..z} or {a..z..2}
        // Make sure they're not numbers (like '5' is both char and num)
        if parts.len() == 3 {
            // Must have valid step
            return parts[2].trim().parse::<i64>().is_ok();
        }
        return true;
    }

    false // Mixed types or invalid format
}

/// Parse brace content into a BraceExpr
///
/// # Arguments
/// * `content` - The content inside braces (without the {} delimiters)
///
/// # Returns
/// A BraceExpr representing the expansion type
pub fn parse_brace_content(content: &str) -> BraceExpr {
    if !is_valid_brace_pattern(content) {
        return BraceExpr::Literal(format!("{{{}}}", content));
    }

    // Try parsing as sequence first (more specific)
    if content.contains("..") {
        if let Some(expr) = try_parse_sequence(content) {
            return expr;
        }
    }

    // Parse as comma-separated list
    parse_list(content)
}

/// Try to parse content as a sequence
fn try_parse_sequence(content: &str) -> Option<BraceExpr> {
    let parts: Vec<&str> = content.split("..").collect();

    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }

    let start_str = parts[0].trim();
    let end_str = parts[1].trim();

    // Try numeric sequence
    if let (Ok(start), Ok(end)) = (start_str.parse::<i64>(), end_str.parse::<i64>()) {
        // Determine default step based on direction
        let default_step = if start <= end { 1 } else { -1 };
        let step_str = if parts.len() == 3 {
            parts[2].trim()
        } else {
            if start <= end {
                "1"
            } else {
                "-1"
            }
        };

        if let Ok(step) = step_str.parse::<i64>() {
            if step == 0 {
                return None; // Invalid step
            }

            // Calculate width for zero-padding
            let width = start_str.len().max(end_str.len());

            return Some(BraceExpr::NumericSeq { start, end, step, width });
        }
    }

    // Try character sequence
    if start_str.len() == 1 && end_str.len() == 1 {
        let start_char = start_str.chars().next().unwrap();
        let end_char = end_str.chars().next().unwrap();
        if start_char.is_ascii() && end_char.is_ascii() {
            // Determine default step based on direction
            let default_step = if start_char <= end_char { 1 } else { -1 };
            let step_str = if parts.len() == 3 {
                parts[2].trim()
            } else {
                if start_char <= end_char {
                    "1"
                } else {
                    "-1"
                }
            };

            if let Ok(step) = step_str.parse::<i64>() {
                if step != 0 {
                    return Some(BraceExpr::CharSeq { start: start_char, end: end_char, step });
                }
            }
        }
    }

    None
}

/// Parse content as a comma-separated list
pub fn parse_list(content: &str) -> BraceExpr {
    let elements = lexer::split_on_commas(content);
    BraceExpr::List(elements)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brace_expr_list() {
        let expr = BraceExpr::List(vec!["a".to_string(), "b".to_string()]);
        assert!(matches!(expr, BraceExpr::List(_)));
    }

    #[test]
    fn test_is_valid_brace_pattern_list() {
        assert!(is_valid_brace_pattern("a,b,c"));
        assert!(is_valid_brace_pattern("a,b"));
        assert!(is_valid_brace_pattern("a,,b")); // Empty element is valid
    }

    #[test]
    fn test_is_valid_brace_pattern_single() {
        assert!(!is_valid_brace_pattern("a")); // Single element is invalid
        assert!(!is_valid_brace_pattern("")); // Empty is invalid
    }

    #[test]
    fn test_is_valid_brace_pattern_numeric() {
        assert!(is_valid_brace_pattern("1..5"));
        assert!(is_valid_brace_pattern("1..10..2"));
        assert!(is_valid_brace_pattern("-5..5"));
    }

    #[test]
    fn test_is_valid_brace_pattern_char() {
        assert!(is_valid_brace_pattern("a..z"));
        assert!(is_valid_brace_pattern("a..z..2"));
    }

    #[test]
    fn test_is_valid_brace_pattern_mixed() {
        assert!(!is_valid_brace_pattern("a..5")); // Mixed types invalid
        assert!(!is_valid_brace_pattern("1..z")); // Mixed types invalid
    }

    #[test]
    fn test_parse_list() {
        let expr = parse_list("a,b,c");
        assert_eq!(expr, BraceExpr::List(vec!["a".to_string(), "b".to_string(), "c".to_string()]));
    }

    #[test]
    fn test_parse_numeric_sequence() {
        let expr = parse_brace_content("1..5");
        assert!(matches!(expr, BraceExpr::NumericSeq { start: 1, end: 5, step: 1, width: 1 }));
    }

    #[test]
    fn test_parse_char_sequence() {
        let expr = parse_brace_content("a..e");
        assert!(matches!(expr, BraceExpr::CharSeq { start: 'a', end: 'e', step: 1 }));
    }

    #[test]
    fn test_parse_zero_padded() {
        let expr = parse_brace_content("01..05");
        assert!(matches!(expr, BraceExpr::NumericSeq { start: 1, end: 5, step: 1, width: 2 }));
    }

    #[test]
    fn test_parse_invalid() {
        let expr = parse_brace_content("a");
        assert!(matches!(expr, BraceExpr::Literal(_)));
    }
}
