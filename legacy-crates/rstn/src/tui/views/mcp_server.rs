//! MCP Server Info View
//!
//! Displays real-time MCP server status and Claude Code interaction metrics.

use crate::tui::mcp_server::McpState;
use crate::tui::views::{View, ViewAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// MCP Server info view
pub struct McpServerView {
    /// Shared state from MCP server
    state: Arc<Mutex<McpState>>,
    /// Scroll position for events
    scroll: usize,
}

impl McpServerView {
    /// Create a new MCP Server view
    pub fn new(state: Arc<Mutex<McpState>>) -> Self {
        Self { state, scroll: 0 }
    }

    /// Render the server status section
    fn render_server_status(&self, frame: &mut Frame, area: Rect, state: &McpState) {
        let uptime = state.server_start_time.elapsed();
        let uptime_str = format!(
            "{}h {}m {}s",
            uptime.as_secs() / 3600,
            (uptime.as_secs() % 3600) / 60,
            uptime.as_secs() % 60
        );

        let lines = vec![
            Line::from(vec![
                Span::styled("• Status:        ", Style::default()),
                Span::styled("Running ✓", Style::default().fg(Color::Green)),
            ]),
            Line::from("• URL:           http://127.0.0.1:19560"),
            Line::from("• Transport:     SSE"),
            Line::from(format!("• Uptime:        {}", uptime_str)),
            Line::from("• Config:        ~/.rstn/mcp-session.json"),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Server Status ");
        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Render the feature context section
    fn render_feature_context(&self, frame: &mut Frame, area: Rect, state: &McpState) {
        let lines = vec![
            Line::from(format!(
                "• Feature:       {}",
                state.feature_number.as_deref().unwrap_or("None")
            )),
            Line::from(format!(
                "• Branch:        {}",
                state.branch.as_deref().unwrap_or("None")
            )),
            Line::from(format!(
                "• Phase:         {}",
                state.phase.as_deref().unwrap_or("None")
            )),
            Line::from(format!(
                "• Spec Dir:      {}",
                state.spec_dir.as_deref().unwrap_or("None")
            )),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Feature Context ");
        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Render the tool usage section
    fn render_tool_usage(&self, frame: &mut Frame, area: Rect, state: &McpState) {
        let mut lines = vec![
            Line::from(format!(
                "• rstn_report_status:    {} calls",
                state
                    .tool_call_counts
                    .get("rstn_report_status")
                    .unwrap_or(&0)
            )),
            Line::from(format!(
                "• rstn_read_spec:        {} calls",
                state.tool_call_counts.get("rstn_read_spec").unwrap_or(&0)
            )),
            Line::from(format!(
                "• rstn_get_context:      {} calls",
                state.tool_call_counts.get("rstn_get_context").unwrap_or(&0)
            )),
            Line::from(format!(
                "• rstn_complete_task:    {} calls",
                state
                    .tool_call_counts
                    .get("rstn_complete_task")
                    .unwrap_or(&0)
            )),
        ];

        if let Some((tool, instant)) = &state.last_tool_call {
            let elapsed = instant.elapsed().as_secs();
            lines.push(Line::from(format!(
                "• Last called:           {} ({}s ago)",
                tool, elapsed
            )));
        }

        let block = Block::default().borders(Borders::ALL).title(" Tool Usage ");
        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Render the recent events section
    fn render_recent_events(&self, frame: &mut Frame, area: Rect, state: &McpState) {
        let items: Vec<ListItem> = state
            .recent_events
            .iter()
            .rev()
            .take(10)
            .map(|event| {
                let elapsed = event.timestamp.elapsed().as_secs();
                let time_str = format!("{}s ago", elapsed);
                let line = format!("{:<8} [{}] {}", time_str, event.event_type, event.details);
                ListItem::new(line)
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Recent Events (last 10) ");
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}

impl View for McpServerView {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Try to lock state (non-blocking)
        let state_guard = match self.state.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                // If can't lock, show loading message
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" MCP Server Info ");
                let text = Paragraph::new("Loading...").block(block);
                frame.render_widget(text, area);
                return;
            }
        };

        // Split into 4 sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Server Status
                Constraint::Length(7), // Feature Context
                Constraint::Length(8), // Tool Usage
                Constraint::Min(8),    // Recent Events
            ])
            .split(area);

        self.render_server_status(frame, chunks[0], &state_guard);
        self.render_feature_context(frame, chunks[1], &state_guard);
        self.render_tool_usage(frame, chunks[2], &state_guard);
        self.render_recent_events(frame, chunks[3], &state_guard);
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
                ViewAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll = self.scroll.saturating_add(1);
                ViewAction::None
            }
            _ => ViewAction::None,
        }
    }

    fn tick(&mut self) {
        // Auto-refresh every tick to update uptime and "X seconds ago" displays
    }
}
