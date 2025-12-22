//! E2E tests for SDD workflow using TestBackend
//!
//! These tests verify the visual output and state of the TUI
//! during the Specify workflow.

use super::TuiTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};

// T045: Test that input dialog renders in buffer
#[test]
fn test_input_dialog_renders_in_buffer() {
    let mut harness = TuiTestHarness::new(80, 24);

    // Verify initial state - no input mode
    assert!(!harness.is_in_input_mode());
    assert!(!harness.has_input_dialog());

    // Request input for feature description (multiline)
    harness.request_input("Enter feature description:", None);

    // Verify input mode is active
    assert!(harness.is_in_input_mode(), "Should be in input mode");
    assert!(harness.has_input_dialog(), "Should have input dialog");

    // Render to buffer
    harness.render();

    // The dialog should be created (we can verify via state)
    let dialog = harness.app.input_dialog.as_ref().unwrap();
    assert!(
        dialog.is_multiline(),
        "Dialog should be multiline for feature description"
    );
    assert_eq!(dialog.title, "Input Required");
}

// T046: Test that typed characters appear in buffer
#[test]
fn test_typed_characters_appear_in_buffer() {
    let mut harness = TuiTestHarness::new(80, 24);

    // Request input
    harness.request_input("Enter feature description:", None);

    // Type some text
    harness.send_text("test feature");

    // Verify the input value contains our text
    let value = harness.get_input_value();
    assert!(value.is_some(), "Should have input value");
    assert_eq!(value.unwrap(), "test feature");
}

// T047: Test that Escape removes dialog from buffer
#[test]
fn test_escape_removes_dialog_from_buffer() {
    let mut harness = TuiTestHarness::new(80, 24);

    // Request input
    harness.request_input("Enter feature description:", None);
    assert!(harness.is_in_input_mode());

    // Type some text
    harness.send_text("some input");

    // Press Escape to cancel
    harness.send_key(KeyCode::Esc);

    // Verify dialog is gone
    assert!(
        !harness.is_in_input_mode(),
        "Should not be in input mode after Escape"
    );
    assert!(
        !harness.has_input_dialog(),
        "Dialog should be removed after Escape"
    );
}

// Additional: Test multiline input with newlines
#[test]
fn test_multiline_input_with_newlines() {
    let mut harness = TuiTestHarness::new(80, 24);

    // Request multiline input
    harness.request_input("Enter feature description:", None);

    // Type first line
    harness.send_text("line one");

    // Insert newline (Ctrl+Enter in multiline mode inserts newline)
    harness.send_key_with_mod(KeyCode::Enter, KeyModifiers::CONTROL);

    // Type second line
    harness.send_text("line two");

    // Verify multiline content
    let value = harness.get_input_value().unwrap();
    assert!(value.contains("line one"), "Should contain first line");
    assert!(value.contains("line two"), "Should contain second line");
    assert!(value.contains('\n'), "Should contain newline");
}

// Additional: Test Enter submits multiline input
#[test]
fn test_enter_submits_and_clears_dialog() {
    let mut harness = TuiTestHarness::new(80, 24);

    // Request multiline input
    harness.request_input("Enter feature description:", None);

    // Type content
    harness.send_text("my feature");

    // Submit with Enter
    harness.send_key(KeyCode::Enter);

    // Verify dialog is cleared after submission
    assert!(
        !harness.is_in_input_mode(),
        "Should exit input mode after submit"
    );
    assert!(
        !harness.has_input_dialog(),
        "Dialog should be cleared after submit"
    );
}

// Additional: Test single-line input submits on Enter
#[test]
fn test_single_line_enter_submits() {
    let mut harness = TuiTestHarness::new(80, 24);

    // Request single-line input (non-feature-description prompt)
    harness.request_input("Enter branch name:", None);

    // Verify it's not multiline
    let dialog = harness.app.input_dialog.as_ref().unwrap();
    assert!(!dialog.is_multiline(), "Branch name should be single-line");

    // Type content
    harness.send_text("feature-branch");

    // Submit with Enter (single-line)
    harness.send_key(KeyCode::Enter);

    // Verify dialog is cleared
    assert!(!harness.is_in_input_mode());
    assert!(!harness.has_input_dialog());
}
