//! Claude Code CLI Integration
//!
//! Spawns and manages Claude Code CLI processes for chat functionality.
//! Parses JSONL streaming output and emits text deltas.
//!
//! ## CLI Invocation
//!
//! ```bash
//! claude -p --verbose --output-format stream-json "prompt"
//! ```
//!
//! ## FSM States
//!
//! - IDLE: No active process
//! - SPAWNING: Process starting
//! - STREAMING: Receiving JSONL events
//! - COMPLETE: message_stop received
//! - ERROR: Error occurred

use serde::Deserialize;
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

// ============================================================================
// Timeout Constants
// ============================================================================

/// Maximum time between events before considering the CLI hung
pub const EVENT_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum total time for a single request
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(300);

// ============================================================================
// JSONL Event Types
// ============================================================================

/// Events emitted by Claude Code CLI in stream-json mode.
///
/// Only the events we care about are fully typed; others use `Other`.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ClaudeStreamEvent {
    /// Start of message - contains metadata
    #[serde(rename = "message_start")]
    MessageStart {
        message: MessageInfo,
    },

    /// Start of a content block
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u32,
        content_block: ContentBlock,
    },

    /// Text delta - the main event for streaming text
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: u32,
        delta: Delta,
    },

    /// End of a content block
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        index: u32,
    },

    /// Final message metadata
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDeltaInfo,
    },

    /// End of message - streaming complete
    #[serde(rename = "message_stop")]
    MessageStop,

    /// Tool use event (for MCP integration)
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },

    /// Claude CLI system event (initialization)
    #[serde(rename = "system")]
    System {
        subtype: String,
        #[serde(flatten)]
        data: serde_json::Value,
    },

    /// Claude CLI assistant message event
    #[serde(rename = "assistant")]
    Assistant {
        message: AssistantMessage,
    },

    /// Claude CLI result event (completion)
    #[serde(rename = "result")]
    Result {
        subtype: String,
        #[serde(flatten)]
        data: serde_json::Value,
    },

    /// Catch-all for unknown event types
    #[serde(other)]
    Other,
}

/// Message metadata from message_start event
#[derive(Debug, Clone, Deserialize)]
pub struct MessageInfo {
    pub id: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub model: String,
}

/// Content block type information
#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
}

/// Delta containing streaming text
#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    #[serde(rename = "type")]
    pub delta_type: String,
    /// The actual text content (only present for text_delta)
    pub text: Option<String>,
}

/// Final message delta with stop reason
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaInfo {
    pub stop_reason: Option<String>,
}

/// Assistant message from Claude CLI
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantMessage {
    pub id: String,
    #[serde(default)]
    pub model: String,
    pub content: Vec<ContentItem>,
}

/// Content item in assistant message
#[derive(Debug, Clone, Deserialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during Claude CLI operations
#[derive(Debug, Clone)]
pub enum ClaudeCliError {
    /// Claude CLI binary not found
    NotFound,
    /// Failed to spawn process
    SpawnFailed(String),
    /// Failed to parse JSONL line
    ParseError(String),
    /// Process exited with error
    ProcessError(String),
    /// Timeout waiting for response
    Timeout,
    /// No active worktree/cwd
    NoCwd,
}

impl std::fmt::Display for ClaudeCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaudeCliError::NotFound => write!(
                f,
                "Claude Code CLI not found. Install from https://claude.ai/code"
            ),
            ClaudeCliError::SpawnFailed(msg) => write!(f, "Failed to start Claude: {}", msg),
            ClaudeCliError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ClaudeCliError::ProcessError(msg) => write!(f, "Process error: {}", msg),
            ClaudeCliError::Timeout => write!(f, "Response timeout"),
            ClaudeCliError::NoCwd => write!(f, "No active project directory"),
        }
    }
}

impl std::error::Error for ClaudeCliError {}

// ============================================================================
// JSONL Parser
// ============================================================================

/// Parse a single JSONL line into a ClaudeStreamEvent.
///
/// Returns `Other` for unrecognized event types (graceful degradation).
pub fn parse_jsonl_line(line: &str) -> Result<ClaudeStreamEvent, ClaudeCliError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(ClaudeStreamEvent::Other);
    }

    serde_json::from_str(trimmed).map_err(|e| ClaudeCliError::ParseError(e.to_string()))
}

/// Extract text content from a delta event.
///
/// Returns Some(text) if this is a text_delta with content, None otherwise.
pub fn extract_text_delta(event: &ClaudeStreamEvent) -> Option<&str> {
    match event {
        ClaudeStreamEvent::ContentBlockDelta { delta, .. } => {
            if delta.delta_type == "text_delta" {
                delta.text.as_deref()
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract text content from Claude CLI assistant event.
///
/// Returns Some(text) if this is an Assistant event with text content, None otherwise.
pub fn extract_assistant_text(event: &ClaudeStreamEvent) -> Option<String> {
    match event {
        ClaudeStreamEvent::Assistant { message } => {
            // Extract text from first content item
            message
                .content
                .iter()
                .find(|item| item.content_type == "text")
                .and_then(|item| item.text.clone())
        }
        _ => None,
    }
}

/// Check if event signals end of streaming.
pub fn is_message_stop(event: &ClaudeStreamEvent) -> bool {
    matches!(
        event,
        ClaudeStreamEvent::MessageStop | ClaudeStreamEvent::Result { .. }
    )
}

// ============================================================================
// CLI Process Management
// ============================================================================

/// Check if Claude CLI is available on the system (async version).
pub async fn is_claude_available() -> bool {
    Command::new("claude")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Validate that Claude CLI is available, returning an error if not (async version).
///
/// This should be called before attempting to spawn Claude CLI.
pub async fn validate_claude_cli() -> Result<(), ClaudeCliError> {
    Command::new("claude")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map_err(|_| ClaudeCliError::NotFound)?;
    Ok(())
}

/// Spawn Claude CLI with streaming JSON output (async version).
///
/// Returns a Child process with stdout piped for reading JSONL.
pub fn spawn_claude(prompt: &str, cwd: &Path) -> Result<Child, ClaudeCliError> {
    Command::new("claude")
        .arg("-p")
        .arg("--verbose")
        .arg("--output-format")
        .arg("stream-json")
        .arg(prompt)
        .current_dir(cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ClaudeCliError::NotFound
            } else {
                ClaudeCliError::SpawnFailed(e.to_string())
            }
        })
}

/// Async iterator over JSONL events from a Claude CLI process.
pub struct ClaudeEventStream {
    reader: BufReader<tokio::process::ChildStdout>,
    line_buffer: String,
}

impl ClaudeEventStream {
    /// Create a new event stream from a Child process.
    ///
    /// Takes ownership of the process's stdout.
    pub fn new(child: &mut Child) -> Result<Self, ClaudeCliError> {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ClaudeCliError::SpawnFailed("Failed to capture stdout".to_string()))?;

        Ok(Self {
            reader: BufReader::new(stdout),
            line_buffer: String::new(),
        })
    }

    /// Read the next event from the stream (async version).
    ///
    /// Returns None when the stream ends.
    pub async fn next_event(&mut self) -> Option<Result<ClaudeStreamEvent, ClaudeCliError>> {
        self.line_buffer.clear();

        match self.reader.read_line(&mut self.line_buffer).await {
            Ok(0) => None, // EOF
            Ok(_) => Some(parse_jsonl_line(&self.line_buffer)),
            Err(e) => Some(Err(ClaudeCliError::ProcessError(e.to_string()))),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message_start() {
        let line = r#"{"type":"message_start","message":{"id":"msg_123","role":"assistant","model":"claude-3-5-sonnet"}}"#;
        let event = parse_jsonl_line(line).unwrap();

        match event {
            ClaudeStreamEvent::MessageStart { message } => {
                assert_eq!(message.id, "msg_123");
                assert_eq!(message.role, "assistant");
            }
            _ => panic!("Expected MessageStart"),
        }
    }

    #[test]
    fn test_parse_content_block_delta() {
        let line =
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
        let event = parse_jsonl_line(line).unwrap();

        match event {
            ClaudeStreamEvent::ContentBlockDelta { index, delta } => {
                assert_eq!(index, 0);
                assert_eq!(delta.delta_type, "text_delta");
                assert_eq!(delta.text, Some("Hello".to_string()));
            }
            _ => panic!("Expected ContentBlockDelta"),
        }
    }

    #[test]
    fn test_parse_message_stop() {
        let line = r#"{"type":"message_stop"}"#;
        let event = parse_jsonl_line(line).unwrap();

        assert!(matches!(event, ClaudeStreamEvent::MessageStop));
    }

    #[test]
    fn test_parse_unknown_event() {
        let line = r#"{"type":"some_future_event","data":"whatever"}"#;
        let event = parse_jsonl_line(line).unwrap();

        assert!(matches!(event, ClaudeStreamEvent::Other));
    }

    #[test]
    fn test_parse_empty_line() {
        let event = parse_jsonl_line("").unwrap();
        assert!(matches!(event, ClaudeStreamEvent::Other));

        let event = parse_jsonl_line("   ").unwrap();
        assert!(matches!(event, ClaudeStreamEvent::Other));
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_jsonl_line("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_text_delta() {
        let event = ClaudeStreamEvent::ContentBlockDelta {
            index: 0,
            delta: Delta {
                delta_type: "text_delta".to_string(),
                text: Some("Hello world".to_string()),
            },
        };

        assert_eq!(extract_text_delta(&event), Some("Hello world"));
    }

    #[test]
    fn test_extract_text_delta_no_text() {
        let event = ClaudeStreamEvent::ContentBlockDelta {
            index: 0,
            delta: Delta {
                delta_type: "input_json_delta".to_string(),
                text: None,
            },
        };

        assert_eq!(extract_text_delta(&event), None);
    }

    #[test]
    fn test_is_message_stop() {
        assert!(is_message_stop(&ClaudeStreamEvent::MessageStop));
        assert!(!is_message_stop(&ClaudeStreamEvent::Other));
    }

    #[test]
    fn test_error_display() {
        let err = ClaudeCliError::NotFound;
        assert!(err.to_string().contains("not found"));

        let err = ClaudeCliError::Timeout;
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_parse_content_block_start() {
        let line = r#"{"type":"content_block_start","index":0,"content_block":{"type":"text"}}"#;
        let event = parse_jsonl_line(line).unwrap();

        match event {
            ClaudeStreamEvent::ContentBlockStart {
                index,
                content_block,
            } => {
                assert_eq!(index, 0);
                assert_eq!(content_block.block_type, "text");
            }
            _ => panic!("Expected ContentBlockStart"),
        }
    }

    #[test]
    fn test_parse_message_delta() {
        let line = r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"}}"#;
        let event = parse_jsonl_line(line).unwrap();

        match event {
            ClaudeStreamEvent::MessageDelta { delta } => {
                assert_eq!(delta.stop_reason, Some("end_turn".to_string()));
            }
            _ => panic!("Expected MessageDelta"),
        }
    }
}
