//! Integration tests for prompt command
//!
//! Tests cover:
//! - ClaudeResult data structure serialization/deserialization
//! - JSONL streaming message parsing
//! - Command construction (future)
//! - Error handling (future)
//! - Integration with real Claude CLI (optional, marked #[ignore])

use rstn::commands::prompt::ClaudeResult;
use rstn::tui::claude_stream::ClaudeStreamMessage;

// ============================================================================
// Category 5: Data Structure Tests
// ============================================================================

#[test]
fn test_claude_result_default() {
    let result = ClaudeResult::default();

    assert_eq!(result.session_id, None);
    assert_eq!(result.success, false);
    assert_eq!(result.content, "");
    assert_eq!(result.stderr, "");
    assert_eq!(result.exit_code, None);
}

#[test]
fn test_claude_result_serialization() {
    let result = ClaudeResult {
        session_id: Some("test-session-123".to_string()),
        success: true,
        content: "Response text from Claude".to_string(),
        stderr: String::new(),
        exit_code: Some(0),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&result).expect("Failed to serialize ClaudeResult");

    // Verify JSON contains expected fields
    assert!(json.contains("test-session-123"));
    assert!(json.contains("Response text from Claude"));
    assert!(json.contains("\"success\":true"));
}

#[test]
fn test_claude_result_deserialization() {
    // Create original result
    let original = ClaudeResult {
        session_id: Some("abc-xyz-456".to_string()),
        success: true,
        content: "Hello from Claude".to_string(),
        stderr: "Some warning".to_string(),
        exit_code: Some(0),
    };

    // Serialize
    let json = serde_json::to_string(&original).expect("Failed to serialize");

    // Deserialize
    let loaded: ClaudeResult = serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify round-trip
    assert_eq!(loaded.session_id, original.session_id);
    assert_eq!(loaded.success, original.success);
    assert_eq!(loaded.content, original.content);
    assert_eq!(loaded.stderr, original.stderr);
    assert_eq!(loaded.exit_code, original.exit_code);
}

// ============================================================================
// Category 2: JSONL Streaming Parsing Tests
// ============================================================================

#[test]
fn test_parse_assistant_message() {
    // Mock JSONL response with assistant message
    let mock_jsonl = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hello"}]},"session_id":"xyz-123"}"#;

    // Parse message
    let msg: ClaudeStreamMessage =
        serde_json::from_str(mock_jsonl).expect("Failed to parse assistant message");

    // Verify message type
    assert_eq!(msg.msg_type, "assistant");

    // Verify session ID
    assert_eq!(msg.session_id, Some("xyz-123".to_string()));

    // Verify text content
    assert_eq!(msg.get_text(), Some("Hello".to_string()));
}

#[test]
fn test_parse_partial_messages() {
    // Simulate incremental text streaming
    let partial_1 = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hel"}]}}"#;
    let partial_2 = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"lo"}]}}"#;

    // Parse both messages
    let msg1: ClaudeStreamMessage = serde_json::from_str(partial_1).unwrap();
    let msg2: ClaudeStreamMessage = serde_json::from_str(partial_2).unwrap();

    // Accumulate text
    let mut accumulated = String::new();
    if let Some(text) = msg1.get_text() {
        accumulated.push_str(&text);
    }
    if let Some(text) = msg2.get_text() {
        accumulated.push_str(&text);
    }

    // Verify accumulated text
    assert_eq!(accumulated, "Hello");
}

#[test]
fn test_extract_session_id() {
    // Mock JSONL with session ID
    let mock_jsonl = r#"{"type":"assistant","message":{"role":"assistant","content":[]},"session_id":"session-abc-123"}"#;

    // Parse message
    let msg: ClaudeStreamMessage = serde_json::from_str(mock_jsonl).unwrap();

    // Verify session ID extracted
    assert_eq!(msg.session_id, Some("session-abc-123".to_string()));
}

#[test]
fn test_ignore_non_assistant_messages() {
    // Mock JSONL with user message (should be ignored in streaming)
    let user_msg = r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"User input"}]}}"#;
    let assistant_msg = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Response"}]}}"#;

    // Parse both
    let user: ClaudeStreamMessage = serde_json::from_str(user_msg).unwrap();
    let assistant: ClaudeStreamMessage = serde_json::from_str(assistant_msg).unwrap();

    // In actual streaming, we only process assistant messages
    let mut content = String::new();
    if user.msg_type == "assistant" {
        if let Some(text) = user.get_text() {
            content.push_str(&text);
        }
    }
    if assistant.msg_type == "assistant" {
        if let Some(text) = assistant.get_text() {
            content.push_str(&text);
        }
    }

    // Should only have assistant response, not user input
    assert_eq!(content, "Response");
}

#[test]
fn test_handle_malformed_json() {
    // Malformed JSON should fail to parse
    let malformed = "{invalid json";

    let result = serde_json::from_str::<ClaudeStreamMessage>(malformed);

    // Should return error, not panic
    assert!(result.is_err());
}

// ============================================================================
// Category 4: Integration Tests (Real Claude CLI)
// ============================================================================
// Note: These tests are marked #[ignore] and only run manually when Claude CLI is available

#[tokio::test]
#[ignore]
async fn test_real_prompt_execution() {
    // Simple prompt that should work with Claude CLI
    let result = rstn::commands::prompt::run(
        "Say hello in exactly 3 words",
        1,      // max_turns
        true,   // skip_permissions
        false,  // continue_session
        None,   // session_id
        vec![], // allowed_tools
        vec![], // context
        false,  // verbose
    )
    .await;

    // Should succeed if Claude CLI is available
    assert!(result.is_ok(), "Prompt command failed: {:?}", result.err());

    let result = result.unwrap();
    assert!(result.success, "Command did not succeed");
    assert!(result.session_id.is_some(), "No session_id returned");
    assert!(!result.content.is_empty(), "No content returned");
}

#[tokio::test]
#[ignore]
async fn test_session_continuation() {
    use rstn::commands::prompt;

    // First prompt - should return a session ID
    let result1 = prompt::run(
        "Remember this number: 42",
        1,      // max_turns
        true,   // skip_permissions
        false,  // continue_session
        None,   // session_id
        vec![], // allowed_tools
        vec![], // context
        false,  // verbose
    )
    .await;

    assert!(result1.is_ok(), "First prompt failed: {:?}", result1.err());
    let result1 = result1.unwrap();
    assert!(result1.success, "First prompt did not succeed");
    assert!(
        result1.session_id.is_some(),
        "First prompt did not return a session_id"
    );

    let session_id = result1.session_id.unwrap();

    // Second prompt - resume the session
    let result2 = prompt::run(
        "What number did I ask you to remember?",
        1,                        // max_turns
        true,                     // skip_permissions
        false,                    // continue_session
        Some(session_id.clone()), // session_id - resume previous session
        vec![],                   // allowed_tools
        vec![],                   // context
        false,                    // verbose
    )
    .await;

    assert!(result2.is_ok(), "Second prompt failed: {:?}", result2.err());
    let result2 = result2.unwrap();
    assert!(result2.success, "Second prompt did not succeed");

    // The second prompt should have the same session ID (continuation)
    assert_eq!(
        result2.session_id,
        Some(session_id),
        "Session continuation did not use the same session ID"
    );

    // The response should contain the remembered number (basic context check)
    assert!(
        result2.content.contains("42"),
        "Claude did not remember the number from the previous session"
    );
}
