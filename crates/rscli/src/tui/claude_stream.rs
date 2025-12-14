//! Parse Claude Code streaming JSON output (JSONL format)
//!
//! When Claude Code runs with `--output-format stream-json`, it outputs one JSON
//! object per line. This module parses that output and extracts RSCLI status blocks.

use serde::Deserialize;

/// A single line from Claude's stream-json output
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeStreamMessage {
    #[serde(rename = "type")]
    pub msg_type: String, // "init", "assistant", "user", "result"

    #[serde(default)]
    pub message: Option<ClaudeMessage>,

    #[serde(default)]
    pub session_id: Option<String>,

    #[serde(default)]
    pub result: Option<String>,

    #[serde(default)]
    pub total_cost_usd: Option<f64>,

    #[serde(default)]
    pub is_error: Option<bool>,
}

/// Message content from Claude
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: Vec<ClaudeContent>,
}

/// Content block within a message
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

/// RSCLI status block parsed from Claude's output
///
/// Claude is instructed (via --append-system-prompt) to output this at the end
/// of responses when it needs to signal state to the TUI.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RscliStatus {
    /// Status type: "needs_input", "completed", "error"
    pub status: String,
    /// Prompt to show user (for needs_input status)
    #[serde(default)]
    pub prompt: Option<String>,
    /// Error message (for error status)
    #[serde(default)]
    pub message: Option<String>,
}

/// Markers for the JSON status block
const STATUS_BLOCK_START: &str = "```rscli-status";
const STATUS_BLOCK_END: &str = "```";

impl ClaudeStreamMessage {
    /// Extract all text content from message
    pub fn get_text(&self) -> Option<String> {
        let msg = self.message.as_ref()?;
        let texts: Vec<&str> = msg.content.iter().filter_map(|c| c.text.as_deref()).collect();
        if texts.is_empty() {
            None
        } else {
            Some(texts.join("\n"))
        }
    }

    /// Parse RSCLI status block from message
    ///
    /// Looks for a code block with language `rscli-status` containing JSON.
    pub fn parse_status(&self) -> Option<RscliStatus> {
        let text = self.get_text()?;

        // Find status block markers
        let start = text.find(STATUS_BLOCK_START)?;
        let json_start = start + STATUS_BLOCK_START.len();
        let json_text = &text[json_start..];

        // Find the closing ``` but make sure it's not the start of another block
        let end = json_text.find(STATUS_BLOCK_END)?;
        let json_str = json_text[..end].trim();

        // Parse JSON
        serde_json::from_str(json_str).ok()
    }

    /// Check if Claude needs user input
    pub fn needs_input(&self) -> bool {
        self.parse_status()
            .map(|s| s.status == "needs_input")
            .unwrap_or(false)
    }

    /// Check if phase completed successfully
    pub fn is_completed(&self) -> bool {
        self.parse_status()
            .map(|s| s.status == "completed")
            .unwrap_or(false)
    }

    /// Check if there was an error
    pub fn has_error(&self) -> bool {
        self.parse_status()
            .map(|s| s.status == "error")
            .unwrap_or(false)
    }

    /// Get the prompt text for user input
    pub fn get_input_prompt(&self) -> Option<String> {
        self.parse_status().and_then(|s| s.prompt)
    }

    /// Get error message
    pub fn get_error_message(&self) -> Option<String> {
        self.parse_status().and_then(|s| s.message)
    }

    /// Strip status block from text for display
    ///
    /// Users shouldn't see the raw status block JSON, so we strip it.
    pub fn get_display_text(&self) -> Option<String> {
        let text = self.get_text()?;

        // Remove status block if present
        if let Some(start) = text.find(STATUS_BLOCK_START) {
            let before = &text[..start];
            // Find the end of the status block
            let after_start = &text[start..];
            if let Some(block_start_end) = after_start.find('\n') {
                let remaining = &after_start[block_start_end + 1..];
                if let Some(block_end) = remaining.find(STATUS_BLOCK_END) {
                    let after_block_end = block_end + STATUS_BLOCK_END.len();
                    let after = if after_block_end < remaining.len() {
                        &remaining[after_block_end..]
                    } else {
                        ""
                    };
                    return Some(format!("{}{}", before.trim_end(), after.trim_start()));
                }
            }
        }
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_assistant_message(text: &str) -> ClaudeStreamMessage {
        ClaudeStreamMessage {
            msg_type: "assistant".to_string(),
            message: Some(ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![ClaudeContent {
                    content_type: "text".to_string(),
                    text: Some(text.to_string()),
                }],
            }),
            session_id: Some("test-session".to_string()),
            result: None,
            total_cost_usd: None,
            is_error: None,
        }
    }

    #[test]
    fn test_parse_needs_input_status() {
        let msg = make_assistant_message(
            r#"I need more information about the feature.

```rscli-status
{"status":"needs_input","prompt":"Please describe the feature in detail"}
```"#,
        );

        assert!(msg.needs_input());
        assert!(!msg.is_completed());
        assert!(!msg.has_error());

        let status = msg.parse_status().unwrap();
        assert_eq!(status.status, "needs_input");
        assert_eq!(
            status.prompt,
            Some("Please describe the feature in detail".to_string())
        );
    }

    #[test]
    fn test_parse_completed_status() {
        let msg = make_assistant_message(
            r#"I've created the specification file.

```rscli-status
{"status":"completed"}
```"#,
        );

        assert!(!msg.needs_input());
        assert!(msg.is_completed());
        assert!(!msg.has_error());
    }

    #[test]
    fn test_parse_error_status() {
        let msg = make_assistant_message(
            r#"Something went wrong.

```rscli-status
{"status":"error","message":"Could not find spec directory"}
```"#,
        );

        assert!(!msg.needs_input());
        assert!(!msg.is_completed());
        assert!(msg.has_error());

        let status = msg.parse_status().unwrap();
        assert_eq!(
            status.message,
            Some("Could not find spec directory".to_string())
        );
    }

    #[test]
    fn test_get_display_text_strips_status() {
        let msg = make_assistant_message(
            r#"Here is the spec content.

```rscli-status
{"status":"completed"}
```"#,
        );

        let display = msg.get_display_text().unwrap();
        assert!(!display.contains("rscli-status"));
        assert!(!display.contains("completed"));
        assert!(display.contains("Here is the spec content"));
    }

    #[test]
    fn test_no_status_block() {
        let msg = make_assistant_message("Just a regular message without any status block.");

        assert!(msg.parse_status().is_none());
        assert!(!msg.needs_input());
        assert!(!msg.is_completed());
        assert!(!msg.has_error());
    }

    #[test]
    fn test_get_text_multiple_content() {
        let msg = ClaudeStreamMessage {
            msg_type: "assistant".to_string(),
            message: Some(ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![
                    ClaudeContent {
                        content_type: "text".to_string(),
                        text: Some("First part".to_string()),
                    },
                    ClaudeContent {
                        content_type: "text".to_string(),
                        text: Some("Second part".to_string()),
                    },
                ],
            }),
            session_id: None,
            result: None,
            total_cost_usd: None,
            is_error: None,
        };

        let text = msg.get_text().unwrap();
        assert!(text.contains("First part"));
        assert!(text.contains("Second part"));
    }

    #[test]
    fn test_result_message() {
        let json = r#"{"type":"result","result":"Done","session_id":"abc123","total_cost_usd":0.05,"is_error":false}"#;
        let msg: ClaudeStreamMessage = serde_json::from_str(json).unwrap();

        assert_eq!(msg.msg_type, "result");
        assert_eq!(msg.session_id, Some("abc123".to_string()));
        assert_eq!(msg.total_cost_usd, Some(0.05));
        assert_eq!(msg.is_error, Some(false));
    }

    #[test]
    fn test_init_message() {
        let json = r#"{"type":"init","apiKeySource":"ANTHROPIC_API_KEY","model":"claude-sonnet-4-20250514"}"#;
        let msg: ClaudeStreamMessage = serde_json::from_str(json).unwrap();

        assert_eq!(msg.msg_type, "init");
    }
}
