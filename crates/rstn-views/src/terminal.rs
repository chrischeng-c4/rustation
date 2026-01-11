//! Terminal view - PTY terminal emulator
//!
//! Based on desktop/src/renderer/src/features/terminal/TerminalPage.tsx
//! Spec: openspec/specs/terminal-pty/spec.md

use gpui::*;
use rstn_ui::{MaterialTheme, PageHeader, Themed};

/// Terminal session state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Idle,
    Spawning,
    Active,
    Resizing,
    Error,
    Terminated,
}

/// Terminal session data
#[derive(Debug, Clone)]
pub struct TerminalSession {
    pub id: String,
    pub state: SessionState,
    pub pid: Option<u32>,
    pub cols: u16,
    pub rows: u16,
    pub working_dir: String,
    pub output_lines: Vec<String>,
    pub error: Option<String>,
}

impl TerminalSession {
    pub fn new(id: String, working_dir: String) -> Self {
        Self {
            id,
            state: SessionState::Idle,
            pid: None,
            cols: 80,
            rows: 24,
            working_dir,
            output_lines: Vec::new(),
            error: None,
        }
    }

    /// Get status color based on state
    pub fn status_color(&self, theme: &MaterialTheme) -> Rgba {
        match self.state {
            SessionState::Idle => theme.text.secondary,
            SessionState::Spawning => rgb(0xFFC107),   // Amber
            SessionState::Active => rgb(0x4CAF50),     // Green
            SessionState::Resizing => rgb(0x2196F3),   // Blue
            SessionState::Error => rgb(0xF44336),      // Red
            SessionState::Terminated => rgb(0x9E9E9E), // Grey
        }
    }

    /// Get status text
    pub fn status_text(&self) -> &str {
        match self.state {
            SessionState::Idle => "Idle",
            SessionState::Spawning => "Spawning...",
            SessionState::Active => "Active",
            SessionState::Resizing => "Resizing...",
            SessionState::Error => "Error",
            SessionState::Terminated => "Terminated",
        }
    }
}

/// Terminal tab component
pub struct TerminalTab {
    session: TerminalSession,
    is_active: bool,
    theme: MaterialTheme,
}

impl TerminalTab {
    pub fn new(session: TerminalSession, is_active: bool, theme: MaterialTheme) -> Self {
        Self {
            session,
            is_active,
            theme,
        }
    }

    pub fn render(&self) -> Div {
        let bg = if self.is_active {
            self.theme.secondary.container
        } else {
            self.theme.background.paper
        };

        div()
            .flex()
            .items_center()
            .gap(self.theme.spacing(0.5))
            .px(self.theme.spacing(1.5))
            .py(self.theme.spacing(0.75))
            .bg(bg)
            .rounded_t(self.theme.shape.border_radius_xs)
            .cursor_pointer()
            .hover(|style| {
                if !self.is_active {
                    style.bg(self.theme.surface.container)
                } else {
                    style
                }
            })
            .child(
                // Status indicator dot
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded_full()
                    .bg(self.session.status_color(&self.theme)),
            )
            .child(
                // Session ID/name
                div()
                    .text_sm()
                    .child(format!("Terminal {}", &self.session.id)),
            )
            .child(
                // Close button
                div()
                    .w(px(16.0))
                    .h(px(16.0))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .hover(|style| style.bg(rgb(0xF44336)))
                    .child("×"),
            )
    }
}

/// Terminal output view with ANSI color support
pub struct TerminalOutput {
    lines: Vec<String>,
    theme: MaterialTheme,
}

impl TerminalOutput {
    pub fn new(lines: Vec<String>, theme: MaterialTheme) -> Self {
        Self { lines, theme }
    }

    pub fn render(&self) -> Div {
        div()
            .flex_1()
            .overflow_hidden()
            .p(self.theme.spacing(2.0))
            .bg(rgb(0x000000)) // Pure black background for terminal
            .font_family("monospace")
            .text_sm()
            .children(if self.lines.is_empty() {
                vec![div()
                    .text_color(rgb(0x666666))
                    .child("Terminal ready. Type a command...")]
            } else {
                self.lines
                    .iter()
                    .map(|line| {
                        div()
                            .text_color(rgb(0x00FF00)) // Green terminal text
                            .child(line.clone())
                    })
                    .collect()
            })
    }
}

/// Terminal input component
pub struct TerminalInput {
    placeholder: String,
    theme: MaterialTheme,
}

impl TerminalInput {
    pub fn new(placeholder: String, theme: MaterialTheme) -> Self {
        Self {
            placeholder,
            theme,
        }
    }

    pub fn render(&self) -> Div {
        div()
            .flex()
            .items_center()
            .px(self.theme.spacing(2.0))
            .py(self.theme.spacing(1.0))
            .bg(rgb(0x1A1A1A)) // Dark terminal input bg
            .border_t_1()
            .border_color(self.theme.border.divider)
            .child(
                // Command prompt indicator
                div()
                    .mr(self.theme.spacing(1.0))
                    .text_color(rgb(0x00FF00))
                    .font_family("monospace")
                    .child("$"),
            )
            .child(
                // Input field placeholder
                div()
                    .flex_1()
                    .text_color(rgb(0x666666))
                    .font_family("monospace")
                    .child(self.placeholder.clone()),
            )
    }
}

/// Terminal view component
///
/// Features:
/// - Multiple terminal sessions with tabs
/// - PTY integration (portable-pty)
/// - ANSI color support
/// - Per-worktree isolation
/// - Session persistence
pub struct TerminalView {
    sessions: Vec<TerminalSession>,
    active_session_index: usize,
    theme: MaterialTheme,
}

impl TerminalView {
    pub fn new(sessions: Vec<TerminalSession>, active_session_index: usize, theme: MaterialTheme) -> Self {
        Self {
            sessions,
            active_session_index,
            theme,
        }
    }

    /// Get active session
    fn active_session(&self) -> Option<&TerminalSession> {
        self.sessions.get(self.active_session_index)
    }

    pub fn render(&self, _window: &mut Window, _cx: &mut App) -> Div {
        let page_header = PageHeader::new(
            "Terminal",
            Some("Integrated shell sessions"),
            self.theme.clone(),
        );

        let active_session = self.active_session();

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                page_header.render(Some(
                    div()
                        .primary_button(&self.theme)
                        .child("New Terminal"),
                )),
            )
            .child(
                // Terminal tabs
                div()
                    .flex()
                    .items_end()
                    .gap(self.theme.spacing(0.5))
                    .px(self.theme.spacing(1.0))
                    .bg(self.theme.background.default)
                    .children(
                        self.sessions
                            .iter()
                            .enumerate()
                            .map(|(i, session)| {
                                TerminalTab::new(
                                    session.clone(),
                                    i == self.active_session_index,
                                    self.theme.clone(),
                                )
                                .render()
                            }),
                    ),
            )
            .child(
                // Terminal content area
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .bg(rgb(0x000000))
                    .children(if let Some(session) = active_session {
                        vec![
                            // Status bar (if error)
                            session.error.as_ref().map(|error| {
                                div()
                                    .flex()
                                    .items_center()
                                    .px(self.theme.spacing(2.0))
                                    .py(self.theme.spacing(1.0))
                                    .bg(rgb(0xF44336))
                                    .text_sm()
                                    .text_color(rgb(0xFFFFFF))
                                    .child(format!("Error: {}", error))
                            }),
                            // Terminal output
                            Some(TerminalOutput::new(
                                session.output_lines.clone(),
                                self.theme.clone(),
                            )
                            .render()),
                            // Terminal input
                            Some(TerminalInput::new(
                                "Type command...".to_string(),
                                self.theme.clone(),
                            )
                            .render()),
                        ]
                        .into_iter()
                        .flatten()
                        .collect()
                    } else {
                        vec![div()
                            .flex()
                            .flex_1()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(0x666666))
                            .child("No terminal session active")]
                    }),
            )
            .child(
                // Info bar
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(self.theme.spacing(2.0))
                    .py(self.theme.spacing(0.75))
                    .bg(self.theme.background.paper)
                    .border_t_1()
                    .border_color(self.theme.border.divider)
                    .text_xs()
                    .text_color(self.theme.text.secondary)
                    .children(if let Some(session) = active_session {
                        vec![
                            div().child(format!(
                                "Working Dir: {}",
                                session.working_dir
                            )),
                            div().child(format!(
                                "Size: {}x{} | Status: {}",
                                session.cols,
                                session.rows,
                                session.status_text()
                            )),
                        ]
                    } else {
                        vec![]
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_session_creation() {
        let session = TerminalSession::new("1".to_string(), "/home/user".to_string());

        assert_eq!(session.id, "1");
        assert_eq!(session.state, SessionState::Idle);
        assert_eq!(session.working_dir, "/home/user");
        assert_eq!(session.cols, 80);
        assert_eq!(session.rows, 24);
        assert!(session.error.is_none());
    }

    #[test]
    fn test_session_state_transitions() {
        let mut session = TerminalSession::new("1".to_string(), "/tmp".to_string());

        // Idle → Spawning
        session.state = SessionState::Spawning;
        assert_eq!(session.status_text(), "Spawning...");

        // Spawning → Active
        session.state = SessionState::Active;
        session.pid = Some(1234);
        assert_eq!(session.status_text(), "Active");
        assert_eq!(session.pid, Some(1234));

        // Active → Terminated
        session.state = SessionState::Terminated;
        assert_eq!(session.status_text(), "Terminated");
    }

    #[test]
    fn test_terminal_view_creation() {
        let theme = MaterialTheme::dark();
        let sessions = vec![
            TerminalSession::new("1".to_string(), "/home/user".to_string()),
            TerminalSession::new("2".to_string(), "/tmp".to_string()),
        ];

        let view = TerminalView::new(sessions, 0, theme);

        assert_eq!(view.sessions.len(), 2);
        assert_eq!(view.active_session_index, 0);
        assert!(view.active_session().is_some());
        assert_eq!(view.active_session().unwrap().id, "1");
    }

    #[test]
    fn test_terminal_output_empty() {
        let theme = MaterialTheme::dark();
        let output = TerminalOutput::new(vec![], theme);

        assert_eq!(output.lines.len(), 0);
    }
}
