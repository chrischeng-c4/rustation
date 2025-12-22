//! Integration tests for arithmetic expansion feature (029).
//!
//! Tests for:
//! - $((expression)) arithmetic expansion
//! - let builtin command
//! - Arithmetic operators (+, -, *, /, %, **)
//! - Comparison operators (<, >, <=, >=, ==, !=)
//! - Logical operators (&&, ||, !)
//! - Bitwise operators (&, |, ^, ~, <<, >>)
//! - Assignment operators (=, +=, -=, etc.)
//! - Increment/decrement (++, --)
//! - Ternary operator (?:)
//! - Comma operator

use rush::executor::execute::CommandExecutor;

// ===== Basic Arithmetic Expansion Tests =====

#[test]
fn test_arithmetic_expansion_basic_addition() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((5 + 3))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("8"));
}

#[test]
fn test_arithmetic_expansion_subtraction() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((10 - 4))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("6"));
}

#[test]
fn test_arithmetic_expansion_multiplication() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((6 * 7))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("42"));
}

#[test]
fn test_arithmetic_expansion_division() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((17 / 5))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("3"));
}

#[test]
fn test_arithmetic_expansion_modulo() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((17 % 5))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("2"));
}

#[test]
fn test_arithmetic_expansion_power() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((2 ** 8))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("256"));
}

// ===== Precedence Tests =====

#[test]
fn test_arithmetic_expansion_precedence() {
    let mut executor = CommandExecutor::new();
    // 2 + 3 * 4 = 2 + 12 = 14
    executor.execute("x=$((2 + 3 * 4))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("14"));
}

#[test]
fn test_arithmetic_expansion_parentheses() {
    let mut executor = CommandExecutor::new();
    // (2 + 3) * 4 = 5 * 4 = 20
    executor.execute("x=$(((2 + 3) * 4))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("20"));
}

// ===== Variable Tests =====

#[test]
fn test_arithmetic_expansion_with_variable() {
    let mut executor = CommandExecutor::new();
    executor
        .variable_manager_mut()
        .set("y".to_string(), "10".to_string())
        .unwrap();
    executor.execute("x=$((y * 2))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("20"));
}

#[test]
fn test_arithmetic_expansion_undefined_variable() {
    let mut executor = CommandExecutor::new();
    // Undefined variables evaluate to 0
    executor.execute("x=$((undefined + 5))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("5"));
}

// ===== Comparison Operators =====

#[test]
fn test_arithmetic_comparison_greater() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((5 > 3))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("1"));

    executor.execute("y=$((3 > 5))").unwrap();
    assert_eq!(executor.variable_manager().get("y"), Some("0"));
}

#[test]
fn test_arithmetic_comparison_less() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((3 < 5))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("1"));

    executor.execute("y=$((5 < 3))").unwrap();
    assert_eq!(executor.variable_manager().get("y"), Some("0"));
}

#[test]
fn test_arithmetic_comparison_equal() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((5 == 5))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("1"));

    executor.execute("y=$((5 == 3))").unwrap();
    assert_eq!(executor.variable_manager().get("y"), Some("0"));
}

#[test]
fn test_arithmetic_comparison_not_equal() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((5 != 3))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("1"));

    executor.execute("y=$((5 != 5))").unwrap();
    assert_eq!(executor.variable_manager().get("y"), Some("0"));
}

// ===== Logical Operators =====

#[test]
fn test_arithmetic_logical_and() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((1 && 1))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("1"));

    executor.execute("y=$((1 && 0))").unwrap();
    assert_eq!(executor.variable_manager().get("y"), Some("0"));
}

#[test]
fn test_arithmetic_logical_or() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((0 || 1))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("1"));

    executor.execute("y=$((0 || 0))").unwrap();
    assert_eq!(executor.variable_manager().get("y"), Some("0"));
}

#[test]
fn test_arithmetic_logical_not() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((!0))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("1"));

    executor.execute("y=$((!1))").unwrap();
    assert_eq!(executor.variable_manager().get("y"), Some("0"));
}

// ===== Bitwise Operators =====

#[test]
fn test_arithmetic_bitwise_and() {
    let mut executor = CommandExecutor::new();
    // 5 & 3 = 101 & 011 = 001 = 1
    executor.execute("x=$((5 & 3))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("1"));
}

#[test]
fn test_arithmetic_bitwise_or() {
    let mut executor = CommandExecutor::new();
    // 5 | 3 = 101 | 011 = 111 = 7
    executor.execute("x=$((5 | 3))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("7"));
}

#[test]
fn test_arithmetic_bitwise_xor() {
    let mut executor = CommandExecutor::new();
    // 5 ^ 3 = 101 ^ 011 = 110 = 6
    executor.execute("x=$((5 ^ 3))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("6"));
}

#[test]
fn test_arithmetic_bitwise_not() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((~0))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("-1"));
}

#[test]
fn test_arithmetic_shift_left() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((1 << 4))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("16"));
}

#[test]
fn test_arithmetic_shift_right() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((16 >> 2))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("4"));
}

// ===== Assignment Operators =====

#[test]
fn test_arithmetic_assignment() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((y = 10))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("10"));
    assert_eq!(executor.variable_manager().get("y"), Some("10"));
}

#[test]
fn test_arithmetic_add_assign() {
    let mut executor = CommandExecutor::new();
    executor
        .variable_manager_mut()
        .set("x".to_string(), "5".to_string())
        .unwrap();
    executor.execute("y=$((x += 3))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("8"));
    assert_eq!(executor.variable_manager().get("y"), Some("8"));
}

// ===== Increment/Decrement =====

#[test]
fn test_arithmetic_pre_increment() {
    let mut executor = CommandExecutor::new();
    executor
        .variable_manager_mut()
        .set("x".to_string(), "5".to_string())
        .unwrap();
    executor.execute("y=$((++x))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("6"));
    assert_eq!(executor.variable_manager().get("y"), Some("6"));
}

#[test]
fn test_arithmetic_post_increment() {
    let mut executor = CommandExecutor::new();
    executor
        .variable_manager_mut()
        .set("x".to_string(), "5".to_string())
        .unwrap();
    executor.execute("y=$((x++))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("6"));
    assert_eq!(executor.variable_manager().get("y"), Some("5")); // Returns old value
}

// ===== Ternary Operator =====

#[test]
fn test_arithmetic_ternary_true() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((5 > 3 ? 10 : 20))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("10"));
}

#[test]
fn test_arithmetic_ternary_false() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((5 < 3 ? 10 : 20))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("20"));
}

// ===== Comma Operator =====

#[test]
fn test_arithmetic_comma() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((1, 2, 3))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("3"));
}

// ===== Number Formats =====

#[test]
fn test_arithmetic_hex() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((0x10))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("16"));

    executor.execute("y=$((0xFF))").unwrap();
    assert_eq!(executor.variable_manager().get("y"), Some("255"));
}

#[test]
fn test_arithmetic_octal() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((010))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("8"));
}

// ===== Edge Cases =====

#[test]
fn test_arithmetic_empty_expression() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$(())").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("0"));
}

#[test]
fn test_arithmetic_negative() {
    let mut executor = CommandExecutor::new();
    executor.execute("x=$((-5))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("-5"));
}

// ===== let Builtin Tests =====

#[test]
fn test_let_simple() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("let x=5+3");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
    assert_eq!(executor.variable_manager().get("x"), Some("8"));
}

#[test]
fn test_let_multiple() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("let x=5 y=10");
    assert!(result.is_ok());
    assert_eq!(executor.variable_manager().get("x"), Some("5"));
    assert_eq!(executor.variable_manager().get("y"), Some("10"));
}

#[test]
fn test_let_increment() {
    let mut executor = CommandExecutor::new();
    executor
        .variable_manager_mut()
        .set("x".to_string(), "5".to_string())
        .unwrap();
    let result = executor.execute("let x++");
    assert!(result.is_ok());
    assert_eq!(executor.variable_manager().get("x"), Some("6"));
}

#[test]
fn test_let_exit_status_nonzero() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("let x=5");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // Non-zero result = exit 0
}

#[test]
fn test_let_exit_status_zero() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("let x=0");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // Zero result = exit 1
}

// ===== Nested Arithmetic =====

#[test]
fn test_arithmetic_nested() {
    let mut executor = CommandExecutor::new();
    // $((1 + $((2 * 3)))) = $((1 + 6)) = 7
    executor.execute("x=$((1 + $((2 * 3))))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("7"));
}

// ===== Complex Expression =====

#[test]
fn test_arithmetic_complex() {
    let mut executor = CommandExecutor::new();
    // (2 + 3) * 4 - 5 / 2 = 5 * 4 - 2 = 20 - 2 = 18
    executor.execute("x=$(((2 + 3) * 4 - 5 / 2))").unwrap();
    assert_eq!(executor.variable_manager().get("x"), Some("18"));
}
