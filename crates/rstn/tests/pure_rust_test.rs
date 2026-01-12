//! Pure Rust test without any GPUI imports
//!
//! This test verifies if the SIGBUS error is related to GPUI dependencies.

#[test]
fn test_basic_addition() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_string_concatenation() {
    let result = format!("{} {}", "Hello", "World");
    assert_eq!(result, "Hello World");
}
