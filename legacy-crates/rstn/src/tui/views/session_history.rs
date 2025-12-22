//! Session History view - display and manage Claude CLI sessions
//!
//! This view provides a 5th tab in the TUI for session management.
//! Layout: Split-pane (30% session list | 70% session details + log preview)

use crate::session_manager::{SessionManager, SessionRecord};
use crate::tui::state::session_history::{SessionHistoryFocus, SessionHistoryState};
use crate::tui::views::{View, ViewAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Session History view
pub struct SessionHistoryView {
    // Serializable state (persisted)
    /// Selected session index
    selected_index: Option<usize>,
    /// Current focus area
    focus: SessionHistoryFocus,
    /// Maximum sessions to load
    max_sessions: usize,
    /// Show log preview toggle
    show_log_preview: bool,
    /// Log scroll position
    log_scroll: usize,
    /// Filter by command type
    filter_type: Option<String>,
    /// Filter by status
    filter_status: Option<String>,

    // Non-serializable runtime state
    /// Loaded sessions from database
    sessions: Vec<SessionRecord>,
    /// Currently displayed session details
    selected_session: Option<SessionRecord>,
    /// Cached log file content
    log_content: Option<Vec<String>>,
    /// Error message if any
    error_message: Option<String>,
    /// Last refresh time (for auto-refresh)
    last_refresh_tick: u64,
    /// Tick counter
    tick_count: u64,
}

impl SessionHistoryView {
    /// Create a new Session History view
    pub fn new() -> Self {
        let mut view = Self {
            selected_index: Some(0),
            focus: SessionHistoryFocus::List,
            max_sessions: 50,
            show_log_preview: true,
            log_scroll: 0,
            filter_type: None,
            filter_status: None,
            sessions: Vec::new(),
            selected_session: None,
            log_content: None,
            error_message: None,
            last_refresh_tick: 0,
            tick_count: 0,
        };

        // Load sessions on initialization
        if let Err(e) = view.refresh_sessions() {
            view.error_message = Some(format!("Failed to load sessions: {}", e));
        }

        view
    }

    /// Extract persistent state (for session persistence)
    pub fn to_state(&self) -> SessionHistoryState {
        SessionHistoryState {
            selected_index: self.selected_index,
            focus: self.focus,
            max_sessions: self.max_sessions,
            show_log_preview: self.show_log_preview,
            log_scroll: self.log_scroll,
            filter_type: self.filter_type.clone(),
            filter_status: self.filter_status.clone(),
        }
    }

    /// Restore from persistent state (for session restoration)
    pub fn from_state(state: SessionHistoryState) -> Self {
        // Save the original selected_index to preserve it
        let original_selected_index = state.selected_index;

        let mut view = Self {
            selected_index: state.selected_index,
            focus: state.focus,
            max_sessions: state.max_sessions,
            show_log_preview: state.show_log_preview,
            log_scroll: state.log_scroll,
            filter_type: state.filter_type,
            filter_status: state.filter_status,
            sessions: Vec::new(),
            selected_session: None,
            log_content: None,
            error_message: None,
            last_refresh_tick: 0,
            tick_count: 0,
        };

        // Load sessions after restoring state
        if let Err(e) = view.refresh_sessions() {
            view.error_message = Some(format!("Failed to load sessions: {}", e));
            // If refresh failed, restore the original selected_index
            view.selected_index = original_selected_index;
        } else if view.sessions.is_empty() {
            // If no sessions loaded, restore the original selected_index
            // (refresh_sessions sets it to None when empty, but we want to preserve state)
            view.selected_index = original_selected_index;
        }

        view
    }

    /// Refresh session list from database
    pub fn refresh_sessions(&mut self) -> crate::Result<()> {
        debug!("Refreshing session list (max: {})", self.max_sessions);
        let manager = SessionManager::open()?;
        self.sessions = manager.list_recent_sessions(self.max_sessions)?;
        debug!("Loaded {} sessions", self.sessions.len());

        // Update selected session if we have one selected
        if let Some(idx) = self.selected_index {
            if idx < self.sessions.len() {
                self.selected_session = Some(self.sessions[idx].clone());
                self.load_log_preview()?;
            } else if !self.sessions.is_empty() {
                // Selection out of bounds, reset to first
                self.selected_index = Some(0);
                self.selected_session = Some(self.sessions[0].clone());
                self.load_log_preview()?;
            } else {
                // No sessions
                self.selected_index = None;
                self.selected_session = None;
                self.log_content = None;
            }
        }

        self.error_message = None;
        Ok(())
    }

    /// Load log file preview for currently selected session
    fn load_log_preview(&mut self) -> crate::Result<()> {
        if !self.show_log_preview {
            self.log_content = None;
            return Ok(());
        }

        if let Some(ref session) = self.selected_session {
            if let Some(ref log_file) = session.log_file {
                match std::fs::read_to_string(log_file) {
                    Ok(content) => {
                        self.log_content = Some(content.lines().map(|s| s.to_string()).collect());
                    }
                    Err(e) => {
                        self.log_content = Some(vec![format!("Error reading log file: {}", e)]);
                    }
                }
            } else {
                self.log_content = Some(vec!["No log file available".to_string()]);
            }
        }

        Ok(())
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if let Some(idx) = self.selected_index {
            if idx > 0 {
                self.selected_index = Some(idx - 1);
                if idx - 1 < self.sessions.len() {
                    self.selected_session = Some(self.sessions[idx - 1].clone());
                    if let Err(e) = self.load_log_preview() {
                        debug!("Failed to load log preview: {}", e);
                        // Don't show error to user for navigation - too noisy
                    }
                    self.log_scroll = 0; // Reset scroll when changing session
                }
            }
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if let Some(idx) = self.selected_index {
            if idx + 1 < self.sessions.len() {
                self.selected_index = Some(idx + 1);
                self.selected_session = Some(self.sessions[idx + 1].clone());
                if let Err(e) = self.load_log_preview() {
                    debug!("Failed to load log preview: {}", e);
                    // Don't show error to user for navigation - too noisy
                }
                self.log_scroll = 0; // Reset scroll when changing session
            }
        } else if !self.sessions.is_empty() {
            // No selection, select first
            self.selected_index = Some(0);
            self.selected_session = Some(self.sessions[0].clone());
            if let Err(e) = self.load_log_preview() {
                debug!("Failed to load log preview: {}", e);
            }
        }
    }

    /// Scroll log preview up
    pub fn scroll_log_up(&mut self) {
        if self.log_scroll > 0 {
            self.log_scroll -= 1;
        }
    }

    /// Scroll log preview down
    pub fn scroll_log_down(&mut self, max_scroll: usize) {
        if self.log_scroll < max_scroll {
            self.log_scroll += 1;
        }
    }

    /// Toggle focus between list and details
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            SessionHistoryFocus::List => SessionHistoryFocus::Details,
            SessionHistoryFocus::Details => SessionHistoryFocus::List,
        };
    }

    /// Delete currently selected session
    pub fn delete_selected_session(&mut self) -> crate::Result<()> {
        if let Some(ref session) = self.selected_session {
            let manager = SessionManager::open()?;
            manager.delete_session(&session.session_id)?;

            // Delete log file if it exists
            if let Some(ref log_file) = session.log_file {
                let _ = std::fs::remove_file(log_file); // Ignore error if file doesn't exist
            }

            // Refresh list
            self.refresh_sessions()?;
        }
        Ok(())
    }

    /// Get currently selected session ID
    pub fn get_selected_session_id(&self) -> Option<String> {
        self.selected_session.as_ref().map(|s| s.session_id.clone())
    }

    /// Get currently selected session's log file path
    pub fn get_selected_log_path(&self) -> Option<PathBuf> {
        self.selected_session
            .as_ref()
            .and_then(|s| s.log_file.as_ref().map(PathBuf::from))
    }

    /// Render the session list (left pane, 30%)
    fn render_session_list(&self, frame: &mut Frame, area: Rect) {
        let title = format!("Sessions ({})", self.sessions.len());
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focus == SessionHistoryFocus::List {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        if self.sessions.is_empty() {
            let empty_msg =
                Paragraph::new("No sessions found.\n\nRun `rstn prompt` to create a session.")
                    .block(block)
                    .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty_msg, area);
            return;
        }

        let items: Vec<ListItem> = self
            .sessions
            .iter()
            .enumerate()
            .map(|(idx, session)| {
                let is_selected = self.selected_index == Some(idx);

                // Format timestamp (just date and time for now)
                let dt = chrono::DateTime::from_timestamp(session.created_at, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                // Status icon and color
                let (status_icon, status_color) = match session.status.as_str() {
                    "completed" => ("✓", Color::Green),
                    "active" => ("⏵", Color::Yellow),
                    "error" => ("✗", Color::Red),
                    _ => ("•", Color::Gray),
                };

                // Build line
                let line = Line::from(vec![
                    Span::styled(
                        if is_selected { "▶ " } else { "  " },
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(dt),
                    Span::raw("  "),
                    Span::styled(status_icon, Style::default().fg(status_color)),
                    Span::raw(" "),
                    Span::raw(&session.command_type),
                ]);

                let style = if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(line).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// Render session details and log preview (right pane, 70%)
    fn render_session_details(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Session Details")
            .borders(Borders::ALL)
            .border_style(if self.focus == SessionHistoryFocus::Details {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        if let Some(ref session) = self.selected_session {
            // Split into details (top 30%) and log preview (bottom 70%)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(block.inner(area));

            // Render block border
            frame.render_widget(block, area);

            // Render session information
            let info_block = Block::default().title("Information").borders(Borders::ALL);

            let dt = chrono::DateTime::from_timestamp(session.created_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let info_text = vec![
                Line::from(vec![
                    Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&session.session_id),
                ]),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&session.command_type),
                ]),
                Line::from(vec![
                    Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(dt),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        &session.status,
                        Style::default().fg(match session.status.as_str() {
                            "completed" => Color::Green,
                            "active" => Color::Yellow,
                            "error" => Color::Red,
                            _ => Color::Gray,
                        }),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Log: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(session.log_file.as_deref().unwrap_or("N/A")),
                ]),
            ];

            let info_paragraph = Paragraph::new(info_text).block(info_block);
            frame.render_widget(info_paragraph, chunks[0]);

            // Render log preview
            if self.show_log_preview {
                let log_block = Block::default()
                    .title("Log Preview (scroll: j/k)")
                    .borders(Borders::ALL);

                if let Some(ref lines) = self.log_content {
                    let visible_lines: Vec<Line> = lines
                        .iter()
                        .skip(self.log_scroll)
                        .take(chunks[1].height.saturating_sub(2) as usize)
                        .map(|line| Line::from(line.clone()))
                        .collect();

                    let log_paragraph = Paragraph::new(visible_lines).block(log_block);
                    frame.render_widget(log_paragraph, chunks[1]);
                } else {
                    let empty = Paragraph::new("No log content")
                        .block(log_block)
                        .style(Style::default().fg(Color::DarkGray));
                    frame.render_widget(empty, chunks[1]);
                }
            }
        } else {
            let empty = Paragraph::new("No session selected")
                .block(block)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
        }
    }
}

impl View for SessionHistoryView {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Split into left (30%) and right (70%)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        self.render_session_list(frame, chunks[0]);
        self.render_session_details(frame, chunks[1]);

        // Show error message if any
        if let Some(ref error) = self.error_message {
            // Render error at the top
            let error_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 3,
            };
            let error_block = Block::default()
                .title("Error")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red));
            let error_text = Paragraph::new(error.as_str())
                .block(error_block)
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error_text, error_area);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        match (self.focus, key.code) {
            // Navigation in List focus
            (SessionHistoryFocus::List, KeyCode::Char('j') | KeyCode::Down) => {
                self.move_down();
                ViewAction::None
            }
            (SessionHistoryFocus::List, KeyCode::Char('k') | KeyCode::Up) => {
                self.move_up();
                ViewAction::None
            }

            // Navigation in Details focus (scroll log)
            (SessionHistoryFocus::Details, KeyCode::Char('j') | KeyCode::Down) => {
                if let Some(ref lines) = self.log_content {
                    let max_scroll = lines.len().saturating_sub(1);
                    self.scroll_log_down(max_scroll);
                }
                ViewAction::None
            }
            (SessionHistoryFocus::Details, KeyCode::Char('k') | KeyCode::Up) => {
                self.scroll_log_up();
                ViewAction::None
            }

            // Tab: Switch focus
            (_, KeyCode::Tab) => {
                self.toggle_focus();
                ViewAction::None
            }

            // Enter: Load selected session (same as selection change)
            (_, KeyCode::Enter) => {
                if let Err(e) = self.load_log_preview() {
                    self.error_message = Some(format!("Failed to load log: {}", e));
                }
                ViewAction::None
            }

            // r: Refresh
            (_, KeyCode::Char('r')) => {
                if let Err(e) = self.refresh_sessions() {
                    self.error_message = Some(format!("Failed to refresh: {}", e));
                }
                ViewAction::None
            }

            // c: Continue session (TODO: Implement in Phase 2G)
            (_, KeyCode::Char('c')) => {
                if let Some(session_id) = self.get_selected_session_id() {
                    // TODO: Return ViewAction::ContinueSession in Phase 2G
                    self.error_message = Some(format!(
                        "Continue session not yet implemented (session: {})",
                        session_id
                    ));
                }
                ViewAction::None
            }

            // o: Open log in external viewer
            (_, KeyCode::Char('o')) => {
                if let Some(log_path) = self.get_selected_log_path() {
                    // Open with system default (TODO: cross-platform)
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open").arg(log_path).spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open").arg(log_path).spawn();
                    }
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("cmd")
                            .args(&["/C", "start", log_path.to_str().unwrap()])
                            .spawn();
                    }
                }
                ViewAction::None
            }

            // d: Delete session (with confirmation TODO)
            (_, KeyCode::Char('d')) => {
                // TODO: Show confirmation dialog in Phase 2G
                if let Err(e) = self.delete_selected_session() {
                    self.error_message = Some(format!("Failed to delete session: {}", e));
                } else {
                    self.error_message = Some("Session deleted".to_string());
                }
                ViewAction::None
            }

            // Esc or q: Clear error message (if any)
            (_, KeyCode::Esc | KeyCode::Char('q')) if self.error_message.is_some() => {
                self.error_message = None;
                ViewAction::None
            }

            _ => ViewAction::None,
        }
    }

    fn tick(&mut self) {
        self.tick_count += 1;

        // Auto-refresh every 30 seconds (300 ticks at 100ms/tick)
        if self.tick_count - self.last_refresh_tick > 300 {
            if let Err(e) = self.refresh_sessions() {
                warn!("Auto-refresh failed: {}", e);
                self.error_message = Some(format!("Refresh failed: {}", e));
            }
            self.last_refresh_tick = self.tick_count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_view() {
        let view = SessionHistoryView::new();
        assert_eq!(view.max_sessions, 50);
        assert_eq!(view.focus, SessionHistoryFocus::List);
        assert!(view.show_log_preview);
    }

    #[test]
    fn test_toggle_focus() {
        let mut view = SessionHistoryView::new();
        assert_eq!(view.focus, SessionHistoryFocus::List);
        view.toggle_focus();
        assert_eq!(view.focus, SessionHistoryFocus::Details);
        view.toggle_focus();
        assert_eq!(view.focus, SessionHistoryFocus::List);
    }

    #[test]
    fn test_state_persistence_roundtrip() {
        let mut view = SessionHistoryView::new();
        view.selected_index = Some(5);
        view.focus = SessionHistoryFocus::Details;
        view.max_sessions = 100;
        view.log_scroll = 42;

        let state = view.to_state();
        let restored = SessionHistoryView::from_state(state);

        assert_eq!(restored.selected_index, Some(5));
        assert_eq!(restored.focus, SessionHistoryFocus::Details);
        assert_eq!(restored.max_sessions, 100);
        assert_eq!(restored.log_scroll, 42);
    }
}
