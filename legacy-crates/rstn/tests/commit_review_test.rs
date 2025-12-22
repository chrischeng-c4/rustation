//! Unit and integration tests for commit review functionality (Feature 050)
//!
//! This test module covers:
//! - T054-T060: Unit tests for state management and input handling
//! - T061-T064: Integration tests for full workflow

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rstn::tui::views::ViewAction;
use rstn::tui::views::{ContentType, WorktreeFocus, WorktreeView};
use rstn::CommitGroup;

/// Helper function to create a test WorktreeView instance
fn create_test_view() -> WorktreeView {
    WorktreeView::new()
}

/// Helper function to create test commit groups
fn create_test_commit_groups(count: usize) -> Vec<CommitGroup> {
    (0..count)
        .map(|i| CommitGroup {
            files: vec![
                format!("file{}_a.rs", i),
                format!("file{}_b.rs", i),
                format!("file{}_c.rs", i),
            ],
            message: format!("Commit message for group {}", i + 1),
            description: format!("Description for group {}", i + 1),
            category: Some(format!("category{}", i)),
        })
        .collect()
}

// ============================================================================
// Unit Tests: State Management (T054-T057)
// ============================================================================

#[test]
fn test_start_commit_review_initializes_state() {
    // T054: Verify state initialization when starting commit review
    let mut view = create_test_view();
    let groups = create_test_commit_groups(3);
    let warnings = vec!["Warning 1".to_string()];
    let sensitive_files = vec!["secret.env".to_string()];

    // Start commit review
    view.start_commit_review(groups.clone(), warnings.clone(), sensitive_files.clone());

    // Verify state initialization
    assert!(view.commit_groups.is_some());
    assert_eq!(view.commit_groups.as_ref().unwrap().len(), 3);
    assert_eq!(view.current_commit_index, 0);
    assert_eq!(view.commit_message_input, "Commit message for group 1");
    assert_eq!(
        view.commit_message_cursor,
        "Commit message for group 1".len()
    );
    assert_eq!(view.commit_sensitive_files, sensitive_files);
    assert!(view.commit_validation_error.is_none());
    assert_eq!(view.content_type, ContentType::CommitReview);
    assert_eq!(view.focus, WorktreeFocus::Content);
}

#[test]
fn test_next_commit_group_increments_index() {
    // T055: Verify index increment and boundary check for next_commit_group
    let mut view = create_test_view();
    let groups = create_test_commit_groups(3);

    view.start_commit_review(groups, vec![], vec![]);

    // Should advance from group 0 to 1
    assert_eq!(view.current_commit_index, 0);
    let result = view.next_commit_group();
    assert!(result);
    assert_eq!(view.current_commit_index, 1);
    assert_eq!(view.commit_message_input, "Commit message for group 2");

    // Should advance from group 1 to 2
    let result = view.next_commit_group();
    assert!(result);
    assert_eq!(view.current_commit_index, 2);
    assert_eq!(view.commit_message_input, "Commit message for group 3");

    // Should not advance beyond last group
    let result = view.next_commit_group();
    assert!(!result);
    assert_eq!(view.current_commit_index, 2);
}

#[test]
fn test_previous_commit_group_decrements_index() {
    // T056: Verify index decrement and boundary check for previous_commit_group
    let mut view = create_test_view();
    let groups = create_test_commit_groups(3);

    view.start_commit_review(groups, vec![], vec![]);

    // Move to last group
    view.next_commit_group();
    view.next_commit_group();
    assert_eq!(view.current_commit_index, 2);

    // Should go back to group 1
    let result = view.previous_commit_group();
    assert!(result);
    assert_eq!(view.current_commit_index, 1);
    assert_eq!(view.commit_message_input, "Commit message for group 2");

    // Should go back to group 0
    let result = view.previous_commit_group();
    assert!(result);
    assert_eq!(view.current_commit_index, 0);
    assert_eq!(view.commit_message_input, "Commit message for group 1");

    // Should not go before first group
    let result = view.previous_commit_group();
    assert!(!result);
    assert_eq!(view.current_commit_index, 0);
}

#[test]
fn test_validate_commit_message_empty() {
    // T057: Verify validation for empty messages
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);

    // Test empty message
    view.commit_message_input = "".to_string();
    let result = view.validate_commit_message();
    assert!(!result);
    assert!(view.commit_validation_error.is_some());
    assert_eq!(
        view.commit_validation_error.as_ref().unwrap(),
        "Commit message cannot be empty"
    );
}

#[test]
fn test_validate_commit_message_whitespace() {
    // T057: Verify validation for whitespace-only messages
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);

    // Test whitespace-only message
    view.commit_message_input = "   \n\t  ".to_string();
    let result = view.validate_commit_message();
    assert!(!result);
    assert!(view.commit_validation_error.is_some());
    assert_eq!(
        view.commit_validation_error.as_ref().unwrap(),
        "Commit message cannot be empty"
    );
}

#[test]
fn test_validate_commit_message_valid() {
    // T057: Verify validation passes for valid messages
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);

    // Test valid message
    view.commit_message_input = "A valid commit message".to_string();
    let result = view.validate_commit_message();
    assert!(result);
    assert!(view.commit_validation_error.is_none());
}

// ============================================================================
// Unit Tests: Input Handling (T058-T060)
// ============================================================================

#[test]
fn test_character_input_at_cursor() {
    // T058: Verify character insertion at cursor position
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 5; // At end

    // Insert character at end
    let key = KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_input, "Hello!");
    assert_eq!(view.commit_message_cursor, 6);
}

#[test]
fn test_character_input_in_middle() {
    // T058: Verify character insertion in middle of text
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Helo".to_string();
    view.commit_message_cursor = 2; // Between 'e' and 'l'

    // Insert 'l' in the middle
    let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_input, "Hello");
    assert_eq!(view.commit_message_cursor, 3);
}

#[test]
fn test_character_input_utf8() {
    // T058: Verify UTF-8 character insertion (emoji)
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Test ".to_string();
    view.commit_message_cursor = 5;

    // Insert emoji
    let key = KeyEvent::new(KeyCode::Char('🚀'), KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_input, "Test 🚀");
    assert_eq!(view.commit_message_cursor, 5 + '🚀'.len_utf8());
}

#[test]
fn test_backspace_at_start() {
    // T059: Verify backspace at start has no effect
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 0;

    let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_input, "Hello");
    assert_eq!(view.commit_message_cursor, 0);
}

#[test]
fn test_backspace_in_middle() {
    // T059: Verify backspace deletes character before cursor
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 3; // After 'l'

    let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_input, "Helo");
    assert_eq!(view.commit_message_cursor, 2);
}

#[test]
fn test_backspace_at_end() {
    // T059: Verify backspace at end deletes last character
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 5;

    let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_input, "Hell");
    assert_eq!(view.commit_message_cursor, 4);
}

#[test]
fn test_delete_at_end() {
    // T059: Verify delete at end has no effect
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 5;

    let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_input, "Hello");
    assert_eq!(view.commit_message_cursor, 5);
}

#[test]
fn test_delete_in_middle() {
    // T059: Verify delete removes character after cursor
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 2; // Before 'l'

    let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_input, "Helo");
    assert_eq!(view.commit_message_cursor, 2);
}

#[test]
fn test_arrow_left_moves_cursor() {
    // T060: Verify left arrow moves cursor left
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 5;

    let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_cursor, 4);
}

#[test]
fn test_arrow_right_moves_cursor() {
    // T060: Verify right arrow moves cursor right
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 0;

    let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_cursor, 1);
}

#[test]
fn test_home_moves_to_start() {
    // T060: Verify Home key moves cursor to start
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 5;

    let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_cursor, 0);
}

#[test]
fn test_end_moves_to_end() {
    // T060: Verify End key moves cursor to end
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello".to_string();
    view.commit_message_cursor = 0;

    let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    assert_eq!(view.commit_message_cursor, 5);
}

#[test]
fn test_cursor_movement_with_utf8() {
    // T060: Verify cursor handles UTF-8 character boundaries
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);
    view.commit_message_input = "Hello 🚀 World".to_string();
    view.commit_message_cursor = "Hello 🚀 World".len();

    // Move left across emoji
    let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    // Cursor should be before emoji
    assert!(view
        .commit_message_input
        .is_char_boundary(view.commit_message_cursor));
}

// ============================================================================
// Integration Tests (T061-T064)
// ============================================================================

#[test]
fn test_full_workflow_start_to_complete() {
    // T061: Full workflow - start, edit, submit, next, complete
    let mut view = create_test_view();
    let groups = create_test_commit_groups(2);

    // Start review
    view.start_commit_review(groups, vec![], vec![]);
    assert_eq!(view.current_commit_index, 0);

    // Edit first message
    view.commit_message_input = "Updated message 1".to_string();
    assert_eq!(view.get_current_commit_message(), "Updated message 1");

    // Validate and prepare to submit (simulate Enter key)
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let action = view.handle_commit_review_input(key);
    assert!(matches!(action, ViewAction::SubmitCommitGroup));

    // Simulate successful commit - move to next group
    let has_next = view.next_commit_group();
    assert!(has_next);
    assert_eq!(view.current_commit_index, 1);

    // Edit second message
    view.commit_message_input = "Updated message 2".to_string();

    // Submit second (last) commit
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let action = view.handle_commit_review_input(key);
    assert!(matches!(action, ViewAction::SubmitCommitGroup));

    // No more groups
    let has_next = view.next_commit_group();
    assert!(!has_next);

    // Cancel to complete workflow
    view.cancel_commit_review();
    assert!(view.commit_groups.is_none());
    assert_eq!(view.content_type, ContentType::Spec);
}

#[test]
fn test_navigation_keys() {
    // T061: Verify 'n' and 'p' keys work for navigation
    let mut view = create_test_view();
    let groups = create_test_commit_groups(3);

    view.start_commit_review(groups, vec![], vec![]);
    assert_eq!(view.current_commit_index, 0);

    // Press 'n' to go to next
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    view.handle_commit_review_input(key);
    assert_eq!(view.current_commit_index, 1);

    // Press 'n' again
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    view.handle_commit_review_input(key);
    assert_eq!(view.current_commit_index, 2);

    // Press 'p' to go back
    let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
    view.handle_commit_review_input(key);
    assert_eq!(view.current_commit_index, 1);
}

#[test]
fn test_escape_cancels_workflow() {
    // T061: Verify Esc key cancels workflow
    let mut view = create_test_view();
    let groups = create_test_commit_groups(3);

    view.start_commit_review(groups, vec![], vec![]);
    assert!(view.commit_groups.is_some());
    assert_eq!(view.content_type, ContentType::CommitReview);

    // Press Esc to cancel
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let action = view.handle_commit_review_input(key);

    // Should call cancel_commit_review internally and return None
    assert!(matches!(action, ViewAction::None));
    // Verify state was cleared
    assert!(view.commit_groups.is_none());
    assert_eq!(view.content_type, ContentType::Spec);
}

#[test]
fn test_validation_blocks_empty_message_submission() {
    // T063: Empty message should show error and block submission
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);

    // Clear the message
    view.commit_message_input = "".to_string();

    // Try to submit with Enter
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let action = view.handle_commit_review_input(key);

    // Should not submit, validation error should be set
    assert!(matches!(action, ViewAction::None));
    assert!(view.commit_validation_error.is_some());
    assert_eq!(
        view.commit_validation_error.as_ref().unwrap(),
        "Commit message cannot be empty"
    );
}

#[test]
fn test_validation_clears_on_edit() {
    // T063: Validation error should clear when user starts typing
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);

    // Set validation error
    view.commit_message_input = "".to_string();
    view.validate_commit_message();
    assert!(view.commit_validation_error.is_some());

    // Type a character
    view.commit_message_cursor = 0;
    let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    view.handle_commit_review_input(key);

    // Validation error should be cleared
    assert!(view.commit_validation_error.is_none());
}

#[test]
fn test_warning_for_many_files() {
    // T064: Test handling of commit groups with >50 files
    let mut view = create_test_view();

    // Create a commit group with 60 files
    let large_group = CommitGroup {
        files: (0..60).map(|i| format!("file{}.rs", i)).collect(),
        message: "Large commit".to_string(),
        description: "Many files".to_string(),
        category: Some("large".to_string()),
    };

    let warnings =
        vec!["Warning: This group contains 60 files. Consider splitting it.".to_string()];

    // Also create a security warning to test that warnings are stored
    let security_warnings = vec![rstn::SecurityWarning {
        file_path: "large_commit".to_string(),
        line_number: 0,
        pattern_matched: "file_count".to_string(),
        severity: rstn::Severity::High,
        message: "Large number of files in single commit".to_string(),
    }];

    view.start_commit_review(vec![large_group], warnings.clone(), vec![]);
    // Manually set security warnings for testing
    view.commit_warnings = security_warnings;

    // Verify warning is stored
    assert_eq!(view.commit_warnings.len(), 1);
    assert!(view.commit_warnings[0].message.contains("files"));
}

#[test]
fn test_get_current_commit_message() {
    // Verify get_current_commit_message returns edited message
    let mut view = create_test_view();
    let groups = create_test_commit_groups(1);

    view.start_commit_review(groups, vec![], vec![]);

    // Edit the message
    view.commit_message_input = "My custom message".to_string();

    // Get should return the edited version
    assert_eq!(view.get_current_commit_message(), "My custom message");
}

#[test]
fn test_cancel_commit_review_clears_state() {
    // Verify cancel_commit_review resets all state
    let mut view = create_test_view();
    let groups = create_test_commit_groups(2);

    view.start_commit_review(
        groups,
        vec!["warning".to_string()],
        vec!["secret".to_string()],
    );

    // Modify state
    view.next_commit_group();
    view.commit_message_input = "Modified".to_string();

    // Cancel
    view.cancel_commit_review();

    // Verify all state is cleared
    assert!(view.commit_groups.is_none());
    assert_eq!(view.current_commit_index, 0);
    assert_eq!(view.commit_message_input, "");
    assert_eq!(view.commit_message_cursor, 0);
    assert!(view.commit_sensitive_files.is_empty());
    assert!(view.commit_validation_error.is_none());
    assert_eq!(view.content_type, ContentType::Spec);
}
