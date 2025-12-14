//! Protocol for bidirectional TUI ↔ Claude Code communication
//!
//! This module defines structured messages that Claude Code can embed in its
//! output to control the TUI's behavior (e.g., request user input, signal
//! phase completion, display info).
//!
//! ## Protocol Format
//!
//! Messages are JSON embedded between markers:
//! ```text
//! @@@ RSCLI_PROTOCOL_START @@@
//! {"type":"request_input","prompt":"Enter feature description:","placeholder":"..."}
//! @@@ RSCLI_PROTOCOL_END @@@
//! ```

use serde::{Deserialize, Serialize};

/// Protocol message markers
pub const PROTOCOL_START_MARKER: &str = "@@@ RSCLI_PROTOCOL_START @@@";
pub const PROTOCOL_END_MARKER: &str = "@@@ RSCLI_PROTOCOL_END @@@";

/// Protocol messages from Claude Code to TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolMessage {
    /// Request user input via bottom status bar
    #[serde(rename = "request_input")]
    RequestInput {
        /// Prompt text to display
        prompt: String,
        /// Optional placeholder text
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        /// Internal identifier for what to do with the input
        #[serde(skip_serializing_if = "Option::is_none")]
        next_action: Option<String>,
    },

    /// Signal that a phase has completed
    #[serde(rename = "phase_completed")]
    PhaseCompleted {
        /// Name of the completed phase
        phase: String,
        /// Next phase to run (if any)
        #[serde(skip_serializing_if = "Option::is_none")]
        next_phase: Option<String>,
        /// Whether to auto-continue to next phase
        #[serde(default)]
        auto_continue: bool,
    },

    /// Display informational message
    #[serde(rename = "display_info")]
    DisplayInfo {
        /// Main message to display
        message: String,
        /// Optional additional details
        #[serde(default)]
        details: Vec<String>,
    },

    /// Present options for user to select (structured choices)
    #[serde(rename = "select_option")]
    SelectOption {
        /// Prompt text to display
        prompt: String,
        /// Available options
        options: Vec<SelectOptionItem>,
        /// Allow multiple selections
        #[serde(default)]
        multi_select: bool,
        /// Default option ID
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<String>,
    },

    /// Auto-continue to next phase without user input
    #[serde(rename = "auto_continue")]
    AutoContinue {
        /// Next phase to run
        next_phase: String,
        /// Delay in milliseconds before starting
        #[serde(default)]
        delay_ms: u64,
        /// Optional status message
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },

    /// Yes/No confirmation prompt
    #[serde(rename = "confirm")]
    Confirm {
        /// Prompt text
        prompt: String,
        /// Default value (true = yes)
        #[serde(default = "default_true")]
        default: bool,
    },

    /// Progress update
    #[serde(rename = "progress")]
    Progress {
        /// Current phase name
        phase: String,
        /// Current step number
        step: u32,
        /// Total steps
        total_steps: u32,
        /// Status message
        message: String,
    },

    /// Session info for persistence
    #[serde(rename = "session_info")]
    SessionInfo {
        /// Session ID for resuming
        session_id: String,
        /// Feature number (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        feature: Option<String>,
    },
}

/// Default function for true
fn default_true() -> bool {
    true
}

/// Option item for select_option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOptionItem {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Parse a protocol message from output lines
///
/// Scans for protocol markers and extracts JSON between them.
/// Returns None if no valid protocol message is found.
pub fn parse_protocol_message(output_lines: &[String]) -> Option<ProtocolMessage> {
    // Find start and end markers
    let start_idx = output_lines
        .iter()
        .position(|line| line.contains(PROTOCOL_START_MARKER))?;
    let end_idx = output_lines
        .iter()
        .position(|line| line.contains(PROTOCOL_END_MARKER))?;

    if end_idx <= start_idx {
        return None;
    }

    // Extract JSON lines between markers
    let json_lines: Vec<&str> = output_lines[start_idx + 1..end_idx]
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if json_lines.is_empty() {
        return None;
    }

    let json_str = json_lines.join(" ");

    // Parse JSON
    serde_json::from_str(&json_str).ok()
}

/// Filter out protocol markers from output lines for display
pub fn filter_protocol_markers(output_lines: &[String]) -> Vec<String> {
    output_lines
        .iter()
        .filter(|line| {
            !line.contains(PROTOCOL_START_MARKER) && !line.contains(PROTOCOL_END_MARKER)
        })
        .cloned()
        .collect()
}

/// Stateful parser for incremental output processing
///
/// Buffers output lines and parses protocol messages as they arrive.
/// Maintains separate views for protocol messages and display-ready text.
#[derive(Debug, Default)]
pub struct OutputParser {
    /// Buffer of all output lines received
    buffer: Vec<String>,
    /// Last successfully parsed protocol message
    last_message: Option<ProtocolMessage>,
}

impl OutputParser {
    /// Create a new output parser
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            last_message: None,
        }
    }

    /// Add a new output line to the buffer
    pub fn add_line(&mut self, line: String) {
        self.buffer.push(line);
    }

    /// Try to parse a protocol message from the current buffer
    ///
    /// Returns the most recent protocol message if found.
    /// Caches the result to avoid redundant parsing.
    pub fn try_parse(&mut self) -> Option<ProtocolMessage> {
        if let Some(msg) = parse_protocol_message(&self.buffer) {
            self.last_message = Some(msg.clone());
            Some(msg)
        } else {
            None
        }
    }

    /// Get the last successfully parsed message
    pub fn last_message(&self) -> Option<&ProtocolMessage> {
        self.last_message.as_ref()
    }

    /// Get display lines with protocol markers filtered out
    pub fn get_display_lines(&self) -> Vec<String> {
        filter_protocol_markers(&self.buffer)
    }

    /// Clear the buffer and reset state
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.last_message = None;
    }

    /// Get the raw buffer
    pub fn buffer(&self) -> &[String] {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_input() {
        let lines = vec![
            "Some regular output".to_string(),
            PROTOCOL_START_MARKER.to_string(),
            r#"{"type":"request_input","prompt":"Enter description:"}"#.to_string(),
            PROTOCOL_END_MARKER.to_string(),
            "More output".to_string(),
        ];

        let msg = parse_protocol_message(&lines).unwrap();
        match msg {
            ProtocolMessage::RequestInput { prompt, .. } => {
                assert_eq!(prompt, "Enter description:");
            }
            _ => panic!("Expected RequestInput"),
        }
    }

    #[test]
    fn test_parse_phase_completed() {
        let lines = vec![
            PROTOCOL_START_MARKER.to_string(),
            r#"{"type":"phase_completed","phase":"specify","next_phase":"clarify","auto_continue":true}"#
                .to_string(),
            PROTOCOL_END_MARKER.to_string(),
        ];

        let msg = parse_protocol_message(&lines).unwrap();
        match msg {
            ProtocolMessage::PhaseCompleted {
                phase,
                next_phase,
                auto_continue,
            } => {
                assert_eq!(phase, "specify");
                assert_eq!(next_phase, Some("clarify".to_string()));
                assert!(auto_continue);
            }
            _ => panic!("Expected PhaseCompleted"),
        }
    }

    #[test]
    fn test_parse_display_info() {
        let lines = vec![
            PROTOCOL_START_MARKER.to_string(),
            r#"{"type":"display_info","message":"Created spec.md","details":["5 user stories","3 acceptance criteria"]}"#
                .to_string(),
            PROTOCOL_END_MARKER.to_string(),
        ];

        let msg = parse_protocol_message(&lines).unwrap();
        match msg {
            ProtocolMessage::DisplayInfo { message, details } => {
                assert_eq!(message, "Created spec.md");
                assert_eq!(details.len(), 2);
            }
            _ => panic!("Expected DisplayInfo"),
        }
    }

    #[test]
    fn test_no_protocol_message() {
        let lines = vec![
            "Just regular output".to_string(),
            "No protocol here".to_string(),
        ];

        assert!(parse_protocol_message(&lines).is_none());
    }

    #[test]
    fn test_filter_markers() {
        let lines = vec![
            "Regular line 1".to_string(),
            PROTOCOL_START_MARKER.to_string(),
            r#"{"type":"display_info","message":"test"}"#.to_string(),
            PROTOCOL_END_MARKER.to_string(),
            "Regular line 2".to_string(),
        ];

        let filtered = filter_protocol_markers(&lines);
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0], "Regular line 1");
        assert_eq!(filtered[2], "Regular line 2");
    }

    #[test]
    fn test_output_parser_incremental() {
        let mut parser = OutputParser::new();

        // Add regular lines
        parser.add_line("Regular output".to_string());
        assert!(parser.try_parse().is_none());

        // Add protocol start
        parser.add_line(PROTOCOL_START_MARKER.to_string());
        assert!(parser.try_parse().is_none());

        // Add protocol message
        parser.add_line(r#"{"type":"display_info","message":"test"}"#.to_string());

        // Add protocol end
        parser.add_line(PROTOCOL_END_MARKER.to_string());

        // Now should parse successfully
        let msg = parser.try_parse().unwrap();
        match msg {
            ProtocolMessage::DisplayInfo { message, .. } => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected DisplayInfo"),
        }

        // Display lines should filter markers
        let display = parser.get_display_lines();
        assert_eq!(display.len(), 2); // Regular output + JSON line
    }

    #[test]
    fn test_output_parser_clear() {
        let mut parser = OutputParser::new();
        parser.add_line("test".to_string());
        assert_eq!(parser.buffer().len(), 1);

        parser.clear();
        assert_eq!(parser.buffer().len(), 0);
        assert!(parser.last_message().is_none());
    }

    #[test]
    fn test_output_parser_last_message() {
        let mut parser = OutputParser::new();
        parser.add_line(PROTOCOL_START_MARKER.to_string());
        parser.add_line(r#"{"type":"request_input","prompt":"Test"}"#.to_string());
        parser.add_line(PROTOCOL_END_MARKER.to_string());

        parser.try_parse();

        // Should cache the message
        assert!(parser.last_message().is_some());
        match parser.last_message().unwrap() {
            ProtocolMessage::RequestInput { prompt, .. } => {
                assert_eq!(prompt, "Test");
            }
            _ => panic!("Expected RequestInput"),
        }
    }
}
