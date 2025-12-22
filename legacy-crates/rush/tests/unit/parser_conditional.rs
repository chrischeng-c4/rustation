//! Unit tests for conditional control flow parser (Feature 017)
//!
//! Tests parser functions for if/then/elif/else/fi constructs

#[cfg(test)]
mod conditional_parser_tests {
    use rush::executor::conditional;

    #[test]
    fn test_parse_if_clause_basic() {
        // Test basic if/then/fi structure
        let result = conditional::parse_if_clause("if true; then echo hello; fi");
        assert!(result.is_ok(), "Should parse basic if statement");
        let if_block = result.unwrap();
        assert!(!if_block.condition.is_empty(), "Condition should not be empty");
        assert!(!if_block.then_block.is_empty(), "Then block should not be empty");
    }

    #[test]
    fn test_parse_if_clause_missing_then() {
        // Test missing "then" keyword
        let result = conditional::parse_if_clause("if true; fi");
        assert!(result.is_err(), "Should fail without 'then' keyword");
    }

    #[test]
    fn test_parse_if_clause_missing_fi() {
        // Test missing "fi" keyword
        let result = conditional::parse_if_clause("if true; then echo hello");
        assert!(result.is_err(), "Should fail without 'fi' keyword");
    }

    #[test]
    fn test_parse_if_clause_empty_condition() {
        // Test empty condition
        let result = conditional::parse_if_clause("if then echo hello; fi");
        assert!(result.is_err(), "Should fail with empty condition");
    }

    #[test]
    fn test_parse_if_clause_empty_then_block() {
        // Test empty then block (should be valid)
        let result = conditional::parse_if_clause("if true; then; fi");
        assert!(result.is_ok(), "Should allow empty then block");
        let if_block = result.unwrap();
        assert!(if_block.then_block.is_empty(), "Then block should be empty");
    }

    #[test]
    fn test_parse_if_clause_with_else() {
        // Test if/then/else/fi structure (US2)
        let result = conditional::parse_if_clause("if true; then echo yes; else echo no; fi");
        assert!(result.is_ok(), "Should parse if/else statement");
        let if_block = result.unwrap();
        assert!(!if_block.then_block.is_empty(), "Then block should not be empty");
        assert!(if_block.else_block.is_some(), "Should have else block");
    }

    #[test]
    fn test_parse_if_clause_else_only() {
        // Test parsing when else block is present (US2)
        let result =
            conditional::parse_if_clause("if false; then echo fail; else echo fallback; fi");
        assert!(result.is_ok(), "Should parse with else block");
        let if_block = result.unwrap();
        assert!(if_block.else_block.is_some(), "Should have else block");
    }

    #[test]
    fn test_parse_if_clause_empty_else_block() {
        // Test empty else block (should be valid) (US2)
        let result = conditional::parse_if_clause("if true; then true; else; fi");
        assert!(result.is_ok(), "Should allow else block");
        let if_block = result.unwrap();
        assert!(if_block.else_block.is_some(), "Should have else block");
    }

    #[test]
    fn test_parse_compound_list_empty() {
        let result = conditional::parse_compound_list("");
        assert!(result.is_ok());
        let list = result.unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_parse_compound_list_single_command() {
        let result = conditional::parse_compound_list("echo hello");
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_parse_compound_list_multiple_commands() {
        let result = conditional::parse_compound_list("echo hello; echo world");
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 2);
    }
}
