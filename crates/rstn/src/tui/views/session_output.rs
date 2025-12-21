//! Session Output View - Display Claude streaming session output in real-time

use crate::tui::claude_stream::ClaudeStreamMessage;
use crate::tui::views::{View, ViewAction};
use serde_json;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use std::collections::HashMap;
use std::time::Instant;

/// Tool execution tracking
#[derive(Debug, Clone)]
struct ActiveTool {
    _name: String,
    start_time: Instant,
}

/// Session Output View state
#[derive(Debug, Clone)]
pub struct SessionOutputView {
    /// All output lines (formatted for display)
    output_lines: Vec<String>,
    /// Current scroll offset
    scroll_offset: usize,
    /// Current turn number
    current_turn: usize,
    /// Maximum turns allowed
    max_turns: usize,
    /// Session start time (for duration calculation)
    start_time: Option<std::time::Instant>,
    /// Session completion status
    completion_status: Option<CompletionStatus>,
    /// Cumulative cost in USD (updated in real-time)
    cumulative_cost_usd: f64,
    /// Budget warning threshold in USD (warn if exceeded)
    budget_warning_threshold: f64,
    /// Track active tool executions (tool_id → tool info)
    active_tools: HashMap<String, ActiveTool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompletionStatus {
    Complete { turns: usize, duration_secs: u64 },
    MaxTurns { turns: usize, duration_secs: u64 },
    Error { message: String, duration_secs: u64 },
}

impl SessionOutputView {
    pub fn new(max_turns: usize) -> Self {
        Self {
            output_lines: vec![],
            scroll_offset: 0,
            current_turn: 0,
            max_turns,
            start_time: None,
            completion_status: None,
            cumulative_cost_usd: 0.0,
            budget_warning_threshold: 0.5, // Default: warn at $0.50
            active_tools: HashMap::new(),
        }
    }

    /// Create a new session with custom budget threshold
    pub fn with_budget(max_turns: usize, budget_threshold: f64) -> Self {
        Self {
            output_lines: vec![],
            scroll_offset: 0,
            current_turn: 0,
            max_turns,
            start_time: None,
            completion_status: None,
            cumulative_cost_usd: 0.0,
            budget_warning_threshold: budget_threshold,
            active_tools: HashMap::new(),
        }
    }

    /// Start a new session
    pub fn start_session(&mut self, user_prompt: &str, max_turns: usize) {
        self.output_lines.clear();
        self.scroll_offset = 0;
        self.current_turn = 0;
        self.max_turns = max_turns;
        self.start_time = Some(std::time::Instant::now());
        self.completion_status = None;
        self.cumulative_cost_usd = 0.0; // Reset cost tracking
        self.active_tools.clear(); // Reset tool tracking

        // Add user prompt
        self.output_lines.push(format!("> User: {}", user_prompt));
        self.output_lines.push(String::new()); // Blank line
    }

    /// Add a streaming message from Claude
    pub fn add_message(&mut self, message: &ClaudeStreamMessage) {
        // Update cumulative cost if available (real-time tracking)
        if let Some(cost) = message.total_cost_usd {
            self.cumulative_cost_usd = cost;
        }

        match message.msg_type.as_str() {
            "init" => {
                // Session initialized
                self.output_lines.push("─".repeat(60));
                self.output_lines
                    .push(format!("🤖 Session started (max {} turns)", self.max_turns));
                self.output_lines.push("─".repeat(60));
                self.output_lines.push(String::new());
            }
            "assistant" => {
                // Assistant message
                if let Some(text) = message.get_text() {
                    // Check if this is a new turn (assistant text after tools)
                    if self
                        .output_lines
                        .last()
                        .map(|l| l.starts_with("  ✓"))
                        .unwrap_or(false)
                    {
                        self.current_turn += 1;
                    }

                    self.output_lines
                        .push(format!("▸ Assistant (Turn {}): ", self.current_turn.max(1)));
                    for line in text.lines() {
                        self.output_lines.push(format!("  {}", line));
                    }
                    self.output_lines.push(String::new());
                }
            }
            "tool_use" => {
                // Tool usage (extract from message content)
                if let Some(msg) = &message.message {
                    for content in &msg.content {
                        if content.content_type == "tool_use" {
                            let tool_name = content.name.as_deref().unwrap_or("Unknown");
                            let tool_id = content
                                .id
                                .as_deref()
                                .and_then(|id| id.get(..8))
                                .unwrap_or("????????");

                            // Track tool execution start
                            if let Some(id) = &content.id {
                                self.active_tools.insert(
                                    id.clone(),
                                    ActiveTool {
                                        _name: tool_name.to_string(),
                                        start_time: Instant::now(),
                                    },
                                );
                            }

                            // Special handling for Edit tool - show diff preview
                            if tool_name == "Edit" {
                                if let Some(input) = &content.input {
                                    let file_path = input
                                        .get("file_path")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");
                                    let old_string = input
                                        .get("old_string")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    let new_string = input
                                        .get("new_string")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");

                                    self.output_lines.push(format!(
                                        "🔧 Tool: {} [{}] ⏳ Running...",
                                        tool_name, tool_id
                                    ));

                                    // Add diff preview
                                    let preview_lines =
                                        Self::format_edit_preview(file_path, old_string, new_string);
                                    self.output_lines.extend(preview_lines);
                                } else {
                                    // Fallback if input is missing
                                    self.output_lines.push(format!(
                                        "🔧 Tool: {} [{}] ⏳ Running... (no input)",
                                        tool_name, tool_id
                                    ));
                                }
                            } else {
                                // Generic tool handling for non-Edit tools
                                // Format input parameters
                                let params = if let Some(input) = &content.input {
                                    let json_str = serde_json::to_string(input).unwrap_or_default();
                                    if json_str.len() > 60 {
                                        format!("{}...", &json_str[..60])
                                    } else {
                                        json_str
                                    }
                                } else {
                                    "{}".to_string()
                                };

                                self.output_lines.push(format!(
                                    "🔧 Tool: {} [{}] ⏳ Running... {}",
                                    tool_name, tool_id, params
                                ));
                            }
                        }
                    }
                }
            }
            "tool_result" => {
                // Tool result - check if we have execution time tracking
                let mut elapsed_ms: Option<u128> = None;

                // Try to find the tool from content blocks (has tool_use_id)
                if let Some(msg) = &message.message {
                    for content in &msg.content {
                        if content.content_type == "tool_result" {
                            if let Some(tool_use_id) = &content.tool_use_id {
                                // Find and remove from active tools
                                if let Some(active_tool) = self.active_tools.remove(tool_use_id) {
                                    elapsed_ms = Some(active_tool.start_time.elapsed().as_millis());
                                }
                            }
                        }
                    }
                }

                // Display result with elapsed time if available
                if let Some(result) = &message.result {
                    let truncated = if result.len() > 200 {
                        format!("{}... (truncated)", &result[..200])
                    } else {
                        result.clone()
                    };

                    if let Some(ms) = elapsed_ms {
                        self.output_lines.push(format!("  ✓ Result: {} ({}ms)", truncated, ms));
                    } else {
                        self.output_lines.push(format!("  ✓ Result: {}", truncated));
                    }
                    self.output_lines.push(String::new());
                }
            }
            "result" => {
                // Final result
                let duration = self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
                let turns = self.current_turn.max(1);

                if message.is_error.unwrap_or(false) {
                    let error_msg = message.result.as_deref().unwrap_or("Unknown error");

                    // Extract suggestion if present
                    let suggestion = Self::extract_suggestion(error_msg);
                    let clean_error = Self::strip_suggestion(error_msg);

                    self.completion_status = Some(CompletionStatus::Error {
                        message: clean_error.clone(),
                        duration_secs: duration,
                    });
                    self.output_lines.push("─".repeat(60));
                    self.output_lines
                        .push(format!("❌ Session failed: {}", clean_error));
                    self.output_lines
                        .push(format!("   Duration: {}s", duration));

                    // Display suggestion if available
                    if let Some(suggestion_text) = suggestion {
                        self.output_lines.push(String::new());
                        self.output_lines
                            .push(format!("💡 Suggestion: {}", suggestion_text));
                    }

                    self.output_lines.push("─".repeat(60));
                } else if turns >= self.max_turns {
                    self.completion_status = Some(CompletionStatus::MaxTurns {
                        turns,
                        duration_secs: duration,
                    });
                    self.output_lines.push("─".repeat(60));
                    self.output_lines.push(format!(
                        "⚠️  Max turns reached ({}/{})",
                        turns, self.max_turns
                    ));
                    self.output_lines
                        .push(format!("   Duration: {}s", duration));
                    self.output_lines.push("─".repeat(60));
                } else {
                    self.completion_status = Some(CompletionStatus::Complete {
                        turns,
                        duration_secs: duration,
                    });
                    self.output_lines.push("─".repeat(60));
                    self.output_lines.push(format!(
                        "✓ Session complete ({} turn{}, {}s)",
                        turns,
                        if turns == 1 { "" } else { "s" },
                        duration
                    ));
                    if let Some(cost) = message.total_cost_usd {
                        self.output_lines.push(format!("   Cost: ${:.4}", cost));
                    }
                    self.output_lines.push("─".repeat(60));
                }
            }
            "stream_event" => {
                // Streaming progress events - log but don't display in session view
                // (these appear in the Log column for debugging)
                tracing::trace!(target: "claude_stream", "stream_event: {:?}", message);
            }
            _ => {
                // Unknown message type - log for debugging (not stream_event)
                tracing::debug!(
                    "Unknown message type in SessionOutputView: {}",
                    message.msg_type
                );
            }
        }

        // Auto-scroll to bottom
        self.scroll_to_bottom();
    }

    /// Scroll to bottom of output
    fn scroll_to_bottom(&mut self) {
        if self.output_lines.len() > 20 {
            self.scroll_offset = self.output_lines.len().saturating_sub(20);
        } else {
            self.scroll_offset = 0;
        }
    }

    /// Scroll up
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll down
    pub fn scroll_down(&mut self, amount: usize) {
        let max_scroll = self.output_lines.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
    }

    /// Get current status line for display
    pub fn status_line(&self) -> String {
        // Budget warning indicator
        let budget_warning = if self.cumulative_cost_usd > self.budget_warning_threshold {
            "⚠️ "
        } else {
            ""
        };

        match &self.completion_status {
            Some(CompletionStatus::Complete {
                turns,
                duration_secs,
            }) => {
                format!(
                    "✓ Complete ({} turn{}, {}s, {}${:.4})",
                    turns,
                    if *turns == 1 { "" } else { "s" },
                    duration_secs,
                    budget_warning,
                    self.cumulative_cost_usd
                )
            }
            Some(CompletionStatus::MaxTurns {
                turns,
                duration_secs,
            }) => {
                format!(
                    "⚠️  Max turns ({}/{}, {}s, {}${:.4})",
                    turns,
                    self.max_turns,
                    duration_secs,
                    budget_warning,
                    self.cumulative_cost_usd
                )
            }
            Some(CompletionStatus::Error {
                message,
                duration_secs,
            }) => {
                format!(
                    "❌ Error: {} ({}s, ${:.4})",
                    message, duration_secs, self.cumulative_cost_usd
                )
            }
            None if self.start_time.is_some() => {
                let duration = self.start_time.unwrap().elapsed().as_secs();
                format!(
                    "🤖 Running... (Turn {}/{}, {}s, {}${:.4})",
                    self.current_turn.max(1),
                    self.max_turns,
                    duration,
                    budget_warning,
                    self.cumulative_cost_usd
                )
            }
            None => "Ready".to_string(),
        }
    }

    /// Check if session is complete
    pub fn is_complete(&self) -> bool {
        self.completion_status.is_some()
    }

    /// Extract suggestion from error message
    /// Format: "{error message} | Suggestion: {suggestion}"
    fn extract_suggestion(error_msg: &str) -> Option<String> {
        error_msg
            .split(" | Suggestion: ")
            .nth(1)
            .map(|s| s.to_string())
    }

    /// Get error message without suggestion part
    fn strip_suggestion(error_msg: &str) -> String {
        error_msg
            .split(" | Suggestion: ")
            .next()
            .unwrap_or(error_msg)
            .to_string()
    }

    /// Generate a unified diff preview for Edit tool
    /// Returns formatted diff lines with +/- markers
    fn generate_diff_preview(old_text: &str, new_text: &str) -> Vec<String> {
        use similar::{ChangeTag, TextDiff};

        let diff = TextDiff::from_lines(old_text, new_text);
        let mut result = Vec::new();

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            result.push(format!("{} {}", sign, change.value().trim_end()));
        }

        result
    }

    /// Format Edit tool parameters for display
    fn format_edit_preview(file_path: &str, old_string: &str, new_string: &str) -> Vec<String> {
        let mut lines = Vec::new();

        lines.push("─".repeat(60));
        lines.push(format!("📝 Edit Preview: {}", file_path));
        lines.push("─".repeat(60));
        lines.push(String::new());

        let diff_lines = Self::generate_diff_preview(old_string, new_string);

        if diff_lines.len() > 20 {
            // Show first 10 lines
            for line in &diff_lines[..10] {
                lines.push(line.clone());
            }
            lines.push(format!("... ({} more lines) ...", diff_lines.len() - 20));
            // Show last 10 lines
            for line in &diff_lines[diff_lines.len() - 10..] {
                lines.push(line.clone());
            }
        } else {
            lines.extend(diff_lines);
        }

        lines.push(String::new());
        lines.push("─".repeat(60));

        lines
    }
}

impl View for SessionOutputView {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let title = if self.is_complete() {
            format!(" Claude Session {} [Esc to close] ", self.status_line())
        } else {
            format!(" Claude Session {} [Esc to close] ", self.status_line())
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default());

        // Render output lines with syntax highlighting
        let lines: Vec<Line> = self
            .output_lines
            .iter()
            .skip(self.scroll_offset)
            .map(|line| {
                let style = if line.starts_with("─") {
                    // Separator lines
                    Style::default().fg(Color::DarkGray)
                } else if line.starts_with("+ ") {
                    // Diff added line
                    Style::default().fg(Color::Green)
                } else if line.starts_with("- ") {
                    // Diff deleted line
                    Style::default().fg(Color::Red)
                } else if line.starts_with("  ") && !line.starts_with("  ✓") {
                    // Diff context line (but not tool result)
                    Style::default().fg(Color::DarkGray)
                } else if line.starts_with("📝 Edit Preview:") {
                    // Edit preview header
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if line.starts_with("> User:") {
                    // User prompt
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if line.starts_with("▸ Assistant") {
                    // Assistant label
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if line.starts_with("🔧 Tool:") && line.contains("⏳ Running...") {
                    // Tool execution in progress
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if line.starts_with("🔧 Tool:") {
                    // Tool execution (completed)
                    Style::default().fg(Color::Yellow)
                } else if line.starts_with("  ✓ Result:") {
                    // Tool result
                    Style::default().fg(Color::Blue)
                } else if line.starts_with("✓ Session complete") {
                    // Success
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if line.starts_with("❌") {
                    // Error
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else if line.starts_with("⚠️") {
                    // Warning
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if line.starts_with("🤖") {
                    // Status
                    Style::default().fg(Color::Magenta)
                } else {
                    // Regular text
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(line.as_str(), style))
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Close session output view
                ViewAction::None // Parent will handle closing
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_down(1);
                ViewAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up(1);
                ViewAction::None
            }
            KeyCode::Char('d') => {
                self.scroll_down(10);
                ViewAction::None
            }
            KeyCode::Char('u') => {
                self.scroll_up(10);
                ViewAction::None
            }
            KeyCode::Char('g') => {
                self.scroll_offset = 0;
                ViewAction::None
            }
            KeyCode::Char('G') => {
                self.scroll_to_bottom();
                ViewAction::None
            }
            _ => ViewAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_tracking_real_time() {
        let mut view = SessionOutputView::new(5);
        view.start_session("Test prompt", 5);

        // Simulate receiving messages with increasing cost
        let msg1 = ClaudeStreamMessage {
            msg_type: "assistant".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![],
            }),
            session_id: None,
            result: None,
            total_cost_usd: Some(0.001), // First turn
            is_error: None,
        };
        view.add_message(&msg1);
        assert_eq!(view.cumulative_cost_usd, 0.001);

        let msg2 = ClaudeStreamMessage {
            msg_type: "assistant".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![],
            }),
            session_id: None,
            result: None,
            total_cost_usd: Some(0.025), // Second turn
            is_error: None,
        };
        view.add_message(&msg2);
        assert_eq!(view.cumulative_cost_usd, 0.025);

        // Verify status line includes cost
        let status = view.status_line();
        assert!(status.contains("$0.0250"), "Status should show cost: {}", status);
    }

    #[test]
    fn test_budget_warning() {
        let mut view = SessionOutputView::with_budget(5, 0.01); // Low threshold
        view.start_session("Test prompt", 5);

        // Cost below threshold
        let msg1 = ClaudeStreamMessage {
            msg_type: "assistant".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![],
            }),
            session_id: None,
            result: None,
            total_cost_usd: Some(0.005),
            is_error: None,
        };
        view.add_message(&msg1);
        let status = view.status_line();
        assert!(!status.contains("⚠️ ⚠️"), "Should not have double warning: {}", status);

        // Cost exceeds threshold
        let msg2 = ClaudeStreamMessage {
            msg_type: "assistant".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![],
            }),
            session_id: None,
            result: None,
            total_cost_usd: Some(0.025), // Above threshold
            is_error: None,
        };
        view.add_message(&msg2);
        let status = view.status_line();
        assert!(status.contains("⚠️"), "Status should show budget warning: {}", status);
    }

    #[test]
    fn test_cost_reset_on_new_session() {
        let mut view = SessionOutputView::new(5);
        view.start_session("First prompt", 5);

        // Add cost
        let msg = ClaudeStreamMessage {
            msg_type: "assistant".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![],
            }),
            session_id: None,
            result: None,
            total_cost_usd: Some(0.05),
            is_error: None,
        };
        view.add_message(&msg);
        assert_eq!(view.cumulative_cost_usd, 0.05);

        // Start new session - cost should reset
        view.start_session("Second prompt", 5);
        assert_eq!(view.cumulative_cost_usd, 0.0);
    }

    #[test]
    fn test_completion_status_shows_cost() {
        let mut view = SessionOutputView::new(5);
        view.start_session("Test prompt", 5);
        view.cumulative_cost_usd = 0.123;

        // Complete
        let result_msg = ClaudeStreamMessage {
            msg_type: "result".to_string(),
            message: None,
            session_id: Some("test123".to_string()),
            result: Some("Done".to_string()),
            total_cost_usd: Some(0.123),
            is_error: Some(false),
        };
        view.add_message(&result_msg);

        let status = view.status_line();
        assert!(status.contains("$0.1230"), "Completion status should show cost: {}", status);
        assert!(status.contains("✓ Complete"), "Should show completion: {}", status);
    }

    #[test]
    fn test_extract_suggestion() {
        // With suggestion
        let msg = "spec.md not found | Suggestion: Run /speckit.specify to generate spec first";
        let suggestion = SessionOutputView::extract_suggestion(msg);
        assert_eq!(
            suggestion,
            Some("Run /speckit.specify to generate spec first".to_string())
        );

        // Without suggestion
        let msg_no_suggestion = "spec.md not found";
        let suggestion = SessionOutputView::extract_suggestion(msg_no_suggestion);
        assert_eq!(suggestion, None);
    }

    #[test]
    fn test_strip_suggestion() {
        // With suggestion
        let msg = "spec.md not found | Suggestion: Run /speckit.specify to generate spec first";
        let clean = SessionOutputView::strip_suggestion(msg);
        assert_eq!(clean, "spec.md not found");

        // Without suggestion
        let msg_no_suggestion = "spec.md not found";
        let clean = SessionOutputView::strip_suggestion(msg_no_suggestion);
        assert_eq!(clean, "spec.md not found");
    }

    #[test]
    fn test_error_display_with_suggestion() {
        let mut view = SessionOutputView::new(5);
        view.start_session("Test prompt", 5);

        // Error message with suggestion
        let error_msg = ClaudeStreamMessage {
            msg_type: "result".to_string(),
            message: None,
            session_id: Some("test123".to_string()),
            result: Some(
                "spec.md not found | Suggestion: Run /speckit.specify to generate spec first"
                    .to_string(),
            ),
            total_cost_usd: Some(0.01),
            is_error: Some(true),
        };
        view.add_message(&error_msg);

        // Verify output contains error and suggestion
        let output = view.output_lines.join("\n");
        assert!(
            output.contains("❌ Session failed: spec.md not found"),
            "Should show clean error message"
        );
        assert!(
            output.contains("💡 Suggestion: Run /speckit.specify"),
            "Should show suggestion"
        );
        assert!(
            !output.contains("| Suggestion:"),
            "Should not show raw separator in error line"
        );
    }

    #[test]
    fn test_error_display_without_suggestion() {
        let mut view = SessionOutputView::new(5);
        view.start_session("Test prompt", 5);

        // Error message without suggestion
        let error_msg = ClaudeStreamMessage {
            msg_type: "result".to_string(),
            message: None,
            session_id: Some("test123".to_string()),
            result: Some("Connection timeout".to_string()),
            total_cost_usd: Some(0.01),
            is_error: Some(true),
        };
        view.add_message(&error_msg);

        // Verify output contains error but no suggestion
        let output = view.output_lines.join("\n");
        assert!(
            output.contains("❌ Session failed: Connection timeout"),
            "Should show error message"
        );
        assert!(
            !output.contains("💡 Suggestion:"),
            "Should not show suggestion when not present"
        );
    }

    #[test]
    fn test_generate_diff_preview() {
        let old_text = "Hello\nWorld\nFoo";
        let new_text = "Hello\nRust\nFoo";

        let diff = SessionOutputView::generate_diff_preview(old_text, new_text);

        // Should show unchanged lines with space prefix
        assert!(
            diff.iter().any(|line| line.starts_with("  Hello")),
            "Should have context line for Hello"
        );
        assert!(
            diff.iter().any(|line| line.starts_with("  Foo")),
            "Should have context line for Foo"
        );

        // Should show deleted line with - prefix
        assert!(
            diff.iter().any(|line| line.starts_with("- World")),
            "Should have deletion for World"
        );

        // Should show added line with + prefix
        assert!(
            diff.iter().any(|line| line.starts_with("+ Rust")),
            "Should have addition for Rust"
        );
    }

    #[test]
    fn test_format_edit_preview() {
        let file_path = "src/main.rs";
        let old_string = "fn main() {\n    println!(\"Hello\");\n}";
        let new_string = "fn main() {\n    println!(\"Hello, World!\");\n}";

        let preview = SessionOutputView::format_edit_preview(file_path, old_string, new_string);

        // Check header
        assert!(
            preview.iter().any(|line| line.contains("📝 Edit Preview: src/main.rs")),
            "Should have preview header with file path"
        );

        // Check separator lines
        assert!(
            preview.iter().any(|line| line.starts_with("─")),
            "Should have separator lines"
        );

        // Check diff content
        assert!(
            preview.iter().any(|line| line.contains("println!")),
            "Should have diff content"
        );
    }

    #[test]
    fn test_format_edit_preview_long_diff() {
        let file_path = "test.txt";
        let old_lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
        let mut new_lines = old_lines.clone();
        new_lines[25] = "Line 26 MODIFIED".to_string();

        let old_string = old_lines.join("\n");
        let new_string = new_lines.join("\n");

        let preview = SessionOutputView::format_edit_preview(file_path, &old_string, &new_string);

        // Should be truncated (show first 10 and last 10 lines)
        assert!(
            preview.iter().any(|line| line.contains("... (") && line.contains("more lines) ...")),
            "Should show truncation message for long diff"
        );
    }

    #[test]
    fn test_edit_tool_interception() {
        use serde_json::json;

        let mut view = SessionOutputView::new(5);
        view.start_session("Test prompt", 5);

        // Simulate Edit tool_use message
        let edit_msg = ClaudeStreamMessage {
            msg_type: "tool_use".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![crate::tui::claude_stream::ClaudeContent {
                    content_type: "tool_use".to_string(),
                    text: None,
                    id: Some("toolu_12345678".to_string()),
                    name: Some("Edit".to_string()),
                    input: Some(json!({
                        "file_path": "/tmp/test.rs",
                        "old_string": "let x = 1;",
                        "new_string": "let x = 2;",
                    })),
                    tool_use_id: None,
                    content: None,
                    is_error: None,
                }],
            }),
            session_id: None,
            result: None,
            total_cost_usd: None,
            is_error: None,
        };

        view.add_message(&edit_msg);

        // Verify output contains diff preview
        let output = view.output_lines.join("\n");
        assert!(
            output.contains("📝 Edit Preview: /tmp/test.rs"),
            "Should show Edit preview header"
        );
        assert!(
            output.contains("- let x = 1;"),
            "Should show deletion"
        );
        assert!(
            output.contains("+ let x = 2;"),
            "Should show addition"
        );
    }

    #[test]
    fn test_non_edit_tool_no_diff() {
        use serde_json::json;

        let mut view = SessionOutputView::new(5);
        view.start_session("Test prompt", 5);

        // Simulate Read tool_use message (not Edit)
        let read_msg = ClaudeStreamMessage {
            msg_type: "tool_use".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![crate::tui::claude_stream::ClaudeContent {
                    content_type: "tool_use".to_string(),
                    text: None,
                    id: Some("toolu_12345678".to_string()),
                    name: Some("Read".to_string()),
                    input: Some(json!({
                        "file_path": "/tmp/test.rs",
                    })),
                    tool_use_id: None,
                    content: None,
                    is_error: None,
                }],
            }),
            session_id: None,
            result: None,
            total_cost_usd: None,
            is_error: None,
        };

        view.add_message(&read_msg);

        // Verify output does NOT contain diff preview
        let output = view.output_lines.join("\n");
        assert!(
            !output.contains("📝 Edit Preview:"),
            "Should not show Edit preview for non-Edit tools"
        );
        assert!(
            output.contains("🔧 Tool: Read"),
            "Should show generic tool header for Read"
        );
    }

    #[test]
    fn test_tool_execution_progress_indicator() {
        use serde_json::json;

        let mut view = SessionOutputView::new(5);
        view.start_session("Test prompt", 5);

        // Simulate tool_use message
        let tool_use_msg = ClaudeStreamMessage {
            msg_type: "tool_use".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![crate::tui::claude_stream::ClaudeContent {
                    content_type: "tool_use".to_string(),
                    text: None,
                    id: Some("toolu_12345678".to_string()),
                    name: Some("Read".to_string()),
                    input: Some(json!({
                        "file_path": "/tmp/test.rs",
                    })),
                    tool_use_id: None,
                    content: None,
                    is_error: None,
                }],
            }),
            session_id: None,
            result: None,
            total_cost_usd: None,
            is_error: None,
        };

        view.add_message(&tool_use_msg);

        // Verify Running indicator is shown
        let output = view.output_lines.join("\n");
        assert!(
            output.contains("⏳ Running..."),
            "Should show running indicator for active tool"
        );
        assert!(
            output.contains("🔧 Tool: Read"),
            "Should show tool name"
        );

        // Verify tool is tracked
        assert_eq!(
            view.active_tools.len(),
            1,
            "Should have one active tool"
        );
        assert!(
            view.active_tools.contains_key("toolu_12345678"),
            "Should track tool by ID"
        );
    }

    #[test]
    fn test_tool_execution_completion_with_timing() {
        use serde_json::json;
        use std::thread;
        use std::time::Duration;

        let mut view = SessionOutputView::new(5);
        view.start_session("Test prompt", 5);

        // Simulate tool_use message
        let tool_use_msg = ClaudeStreamMessage {
            msg_type: "tool_use".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![crate::tui::claude_stream::ClaudeContent {
                    content_type: "tool_use".to_string(),
                    text: None,
                    id: Some("toolu_TESTID123".to_string()),
                    name: Some("Grep".to_string()),
                    input: Some(json!({
                        "pattern": "test",
                    })),
                    tool_use_id: None,
                    content: None,
                    is_error: None,
                }],
            }),
            session_id: None,
            result: None,
            total_cost_usd: None,
            is_error: None,
        };

        view.add_message(&tool_use_msg);

        // Wait a bit to ensure measurable time
        thread::sleep(Duration::from_millis(10));

        // Simulate tool_result message
        let tool_result_msg = ClaudeStreamMessage {
            msg_type: "tool_result".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![crate::tui::claude_stream::ClaudeContent {
                    content_type: "tool_result".to_string(),
                    text: None,
                    id: None,
                    name: None,
                    input: None,
                    tool_use_id: Some("toolu_TESTID123".to_string()),
                    content: Some("Match found".to_string()),
                    is_error: None,
                }],
            }),
            session_id: None,
            result: Some("Match found".to_string()),
            total_cost_usd: None,
            is_error: None,
        };

        view.add_message(&tool_result_msg);

        // Verify tool was removed from active tools
        assert_eq!(
            view.active_tools.len(),
            0,
            "Should have no active tools after completion"
        );

        // Verify elapsed time is shown
        let output = view.output_lines.join("\n");
        assert!(
            output.contains("✓ Result:") && output.contains("ms)"),
            "Should show elapsed time in result: {}",
            output
        );
    }

    #[test]
    fn test_multiple_active_tools() {
        use serde_json::json;

        let mut view = SessionOutputView::new(5);
        view.start_session("Test prompt", 5);

        // Start first tool
        let tool1_msg = ClaudeStreamMessage {
            msg_type: "tool_use".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![crate::tui::claude_stream::ClaudeContent {
                    content_type: "tool_use".to_string(),
                    text: None,
                    id: Some("toolu_FIRST".to_string()),
                    name: Some("Read".to_string()),
                    input: Some(json!({"file_path": "/tmp/1.rs"})),
                    tool_use_id: None,
                    content: None,
                    is_error: None,
                }],
            }),
            session_id: None,
            result: None,
            total_cost_usd: None,
            is_error: None,
        };

        view.add_message(&tool1_msg);

        // Start second tool
        let tool2_msg = ClaudeStreamMessage {
            msg_type: "tool_use".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![crate::tui::claude_stream::ClaudeContent {
                    content_type: "tool_use".to_string(),
                    text: None,
                    id: Some("toolu_SECOND".to_string()),
                    name: Some("Grep".to_string()),
                    input: Some(json!({"pattern": "test"})),
                    tool_use_id: None,
                    content: None,
                    is_error: None,
                }],
            }),
            session_id: None,
            result: None,
            total_cost_usd: None,
            is_error: None,
        };

        view.add_message(&tool2_msg);

        // Both should be tracked
        assert_eq!(
            view.active_tools.len(),
            2,
            "Should have two active tools"
        );

        // Complete first tool
        let result1_msg = ClaudeStreamMessage {
            msg_type: "tool_result".to_string(),
            message: Some(crate::tui::claude_stream::ClaudeMessage {
                role: "assistant".to_string(),
                content: vec![crate::tui::claude_stream::ClaudeContent {
                    content_type: "tool_result".to_string(),
                    text: None,
                    id: None,
                    name: None,
                    input: None,
                    tool_use_id: Some("toolu_FIRST".to_string()),
                    content: Some("File read".to_string()),
                    is_error: None,
                }],
            }),
            session_id: None,
            result: Some("File read".to_string()),
            total_cost_usd: None,
            is_error: None,
        };

        view.add_message(&result1_msg);

        // Only second tool should remain active
        assert_eq!(
            view.active_tools.len(),
            1,
            "Should have one active tool"
        );
        assert!(
            view.active_tools.contains_key("toolu_SECOND"),
            "Second tool should still be active"
        );
    }
}
