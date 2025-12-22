//! Integration tests for the SDD (Spec-Driven Development) workflow
//!
//! These tests verify the complete Specify workflow from the Worktree view,
//! including input dialog creation, character handling, and submission.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rstn::tui::mcp_server::McpState;
use rstn::tui::views::ViewAction;
use rstn::tui::widgets::InputDialog;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Create a test App with default MCP state
fn create_test_app() -> rstn::tui::App {
    let mcp_state = Arc::new(Mutex::new(McpState::default()));
    rstn::tui::App::new(mcp_state)
}

// Helper to create key events
fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

fn key_event_with_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

// T032: Test that Specify phase returns RequestInput action (via App)
#[test]
fn test_specify_returns_request_input_action() {
    // Test via App since WorktreeView.handle_key is private
    // We verify that when a RequestInput action is handled, it sets up input mode
    let mut app = create_test_app();

    // Simulate the Specify workflow requesting input
    let action = ViewAction::RequestInput {
        prompt: "Enter feature description:".to_string(),
        placeholder: Some("e.g., A new login system".to_string()),
    };

    app.handle_view_action(action);

    // Verify the request input action was processed correctly
    assert!(app.input_mode, "Input mode should be enabled");
    assert!(app.input_dialog.is_some(), "Input dialog should be created");

    let dialog = app.input_dialog.as_ref().unwrap();
    assert!(
        dialog.is_multiline(),
        "Feature description dialog should be multiline"
    );
}

// T033: Test that RequestInput creates a multiline dialog for feature description
#[test]
fn test_request_input_creates_multiline_dialog() {
    // Create a multiline dialog like the one created for Specify
    let dialog = InputDialog::new_multiline("Input Required", "Enter feature description:", 10);

    assert!(dialog.is_multiline(), "Specify dialog should be multiline");
    assert_eq!(dialog.input.max_lines, 10);
    assert!(dialog.input.active, "Dialog input should be active");
}

// T034: Test that input dialog accepts characters
#[test]
fn test_input_dialog_accepts_characters() {
    let mut dialog = InputDialog::new_multiline("Test", "Enter:", 5);

    // Insert characters
    dialog.insert_char('h');
    dialog.insert_char('e');
    dialog.insert_char('l');
    dialog.insert_char('l');
    dialog.insert_char('o');

    // Verify characters were accepted
    assert_eq!(dialog.input.lines[0], "hello");
    assert_eq!(dialog.input.get_multiline_value(), "hello");
}

// T035: Test that input dialog handles backspace
#[test]
fn test_input_dialog_handles_backspace() {
    let mut dialog = InputDialog::new_multiline("Test", "Enter:", 5);

    // Type some text
    dialog.insert_char('t');
    dialog.insert_char('e');
    dialog.insert_char('s');
    dialog.insert_char('t');
    assert_eq!(dialog.input.lines[0], "test");

    // Delete characters
    dialog.delete_char(); // Remove 't'
    assert_eq!(dialog.input.lines[0], "tes");

    dialog.delete_char(); // Remove 's'
    assert_eq!(dialog.input.lines[0], "te");
}

// T036: Test that multiline input dialog submits on Enter
#[test]
fn test_input_dialog_submits_on_enter() {
    let mut app = create_test_app();

    // Setup multiline input mode (feature description)
    app.handle_view_action(ViewAction::RequestInput {
        prompt: "Enter feature description:".to_string(),
        placeholder: None,
    });

    assert!(app.input_mode);
    assert!(app.input_dialog.is_some());

    // Type some content
    for c in "new feature".chars() {
        app.handle_key_event(key_event(KeyCode::Char(c)));
    }

    // Submit with Enter
    app.handle_key_event(key_event(KeyCode::Enter));

    // Dialog should be cleared after submission
    assert!(!app.input_mode, "Input mode should be false after Enter");
    assert!(
        app.input_dialog.is_none(),
        "Dialog should be None after submission"
    );
}

// T037: Test that input dialog cancels on Escape
#[test]
fn test_input_dialog_cancels_on_escape() {
    let mut app = create_test_app();

    // Setup input mode
    app.handle_view_action(ViewAction::RequestInput {
        prompt: "Enter something:".to_string(),
        placeholder: None,
    });

    // Type some text
    app.handle_key_event(key_event(KeyCode::Char('x')));
    app.handle_key_event(key_event(KeyCode::Char('y')));

    // Cancel with Escape
    app.handle_key_event(key_event(KeyCode::Esc));

    // Dialog should be cleared
    assert!(!app.input_mode, "Input mode should be false after Escape");
    assert!(
        app.input_dialog.is_none(),
        "Dialog should be None after Escape"
    );
}

// Additional: Test multiline with multiple lines
#[test]
fn test_multiline_input_multiple_lines() {
    let mut dialog = InputDialog::new_multiline("Test", "Enter:", 5);

    // Type first line
    for c in "line one".chars() {
        dialog.insert_char(c);
    }

    // Insert newline
    dialog.insert_newline();

    // Type second line
    for c in "line two".chars() {
        dialog.insert_char(c);
    }

    // Verify two lines exist
    assert_eq!(dialog.input.lines.len(), 2);
    assert_eq!(dialog.input.lines[0], "line one");
    assert_eq!(dialog.input.lines[1], "line two");

    // Verify full value
    let full_value = dialog.input.get_multiline_value();
    assert!(full_value.contains('\n'));
    assert_eq!(full_value, "line one\nline two");
}

// Additional: Test input dialog cursor navigation in multiline
#[test]
fn test_multiline_cursor_navigation() {
    let mut dialog = InputDialog::new_multiline("Test", "Enter:", 5);

    // Type two lines
    for c in "abc".chars() {
        dialog.insert_char(c);
    }
    dialog.insert_newline();
    for c in "def".chars() {
        dialog.insert_char(c);
    }

    // Currently on line 1 (index 1)
    assert_eq!(dialog.input.cursor_line, 1);

    // Move up
    dialog.move_cursor_up();
    assert_eq!(dialog.input.cursor_line, 0);

    // Move down
    dialog.move_cursor_down();
    assert_eq!(dialog.input.cursor_line, 1);
}
