//! Settings view for configuring rscli behavior

use crate::session;
use crate::settings::Settings;
use crate::tui::views::{View, ViewAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

/// Settings menu items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsItem {
    AutoRun,
    MaxTurns,
    SkipPermissions,
    ClearCurrentSession,
    ClearAllSessions,
}

impl SettingsItem {
    fn all() -> Vec<Self> {
        vec![
            Self::AutoRun,
            Self::MaxTurns,
            Self::SkipPermissions,
            Self::ClearCurrentSession,
            Self::ClearAllSessions,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::AutoRun => "SDD Auto-run",
            Self::MaxTurns => "Max turns",
            Self::SkipPermissions => "Skip permissions",
            Self::ClearCurrentSession => "Clear current session",
            Self::ClearAllSessions => "Clear all sessions",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::AutoRun => "Run SDD phases automatically in sequence",
            Self::MaxTurns => "Maximum turns for Claude CLI (use +/- to adjust)",
            Self::SkipPermissions => "Skip permission prompts in Claude CLI",
            Self::ClearCurrentSession => "Clear session for current feature",
            Self::ClearAllSessions => "Clear all cached sessions",
        }
    }
}

/// Settings view state
pub struct SettingsView {
    /// Current settings
    pub settings: Settings,
    /// Selected menu item
    selected_index: usize,
    /// Current feature number (for session clearing)
    pub current_feature: Option<String>,
    /// Status message
    status_message: Option<String>,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            settings: Settings::load(),
            selected_index: 0,
            current_feature: None,
            status_message: None,
        }
    }

    /// Set the current feature for session management
    pub fn set_current_feature(&mut self, feature: Option<String>) {
        self.current_feature = feature;
    }

    /// Get current settings (for use by other components)
    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    fn items() -> Vec<SettingsItem> {
        SettingsItem::all()
    }

    fn selected_item(&self) -> SettingsItem {
        Self::items()[self.selected_index]
    }

    fn handle_enter(&mut self) {
        match self.selected_item() {
            SettingsItem::AutoRun => {
                self.settings.toggle_auto_run();
                let _ = self.settings.save();
                self.status_message = Some(format!(
                    "Auto-run {}",
                    if self.settings.auto_run { "enabled" } else { "disabled" }
                ));
            }
            SettingsItem::SkipPermissions => {
                self.settings.toggle_skip_permissions();
                let _ = self.settings.save();
                self.status_message = Some(format!(
                    "Skip permissions {}",
                    if self.settings.skip_permissions { "enabled" } else { "disabled" }
                ));
            }
            SettingsItem::MaxTurns => {
                // Enter doesn't do anything for max turns, use +/-
            }
            SettingsItem::ClearCurrentSession => {
                if let Some(ref feature) = self.current_feature {
                    if session::clear_session(feature).is_ok() {
                        self.status_message = Some(format!("Cleared session for feature {}", feature));
                    } else {
                        self.status_message = Some("Failed to clear session".to_string());
                    }
                } else {
                    self.status_message = Some("No feature selected".to_string());
                }
            }
            SettingsItem::ClearAllSessions => {
                if session::clear_all_sessions().is_ok() {
                    self.status_message = Some("Cleared all sessions".to_string());
                } else {
                    self.status_message = Some("Failed to clear sessions".to_string());
                }
            }
        }
    }

    fn handle_increment(&mut self) {
        if self.selected_item() == SettingsItem::MaxTurns {
            self.settings.increment_max_turns();
            let _ = self.settings.save();
            self.status_message = Some(format!("Max turns: {}", self.settings.max_turns));
        }
    }

    fn handle_decrement(&mut self) {
        if self.selected_item() == SettingsItem::MaxTurns {
            self.settings.decrement_max_turns();
            let _ = self.settings.save();
            self.status_message = Some(format!("Max turns: {}", self.settings.max_turns));
        }
    }

    fn render_value(&self, item: SettingsItem) -> Span<'static> {
        match item {
            SettingsItem::AutoRun => {
                if self.settings.auto_run {
                    Span::styled("[ON]", Style::default().fg(Color::Green))
                } else {
                    Span::styled("[OFF]", Style::default().fg(Color::Red))
                }
            }
            SettingsItem::MaxTurns => {
                Span::styled(
                    format!("[{}]", self.settings.max_turns),
                    Style::default().fg(Color::Cyan),
                )
            }
            SettingsItem::SkipPermissions => {
                if self.settings.skip_permissions {
                    Span::styled("[ON]", Style::default().fg(Color::Green))
                } else {
                    Span::styled("[OFF]", Style::default().fg(Color::Red))
                }
            }
            SettingsItem::ClearCurrentSession => {
                if let Some(ref feature) = self.current_feature {
                    Span::styled(
                        format!("[Feature {}]", feature),
                        Style::default().fg(Color::Yellow),
                    )
                } else {
                    Span::styled("[No feature]", Style::default().fg(Color::DarkGray))
                }
            }
            SettingsItem::ClearAllSessions => {
                Span::styled("[Action]", Style::default().fg(Color::Red))
            }
        }
    }
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for SettingsView {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(10),    // Settings list
                Constraint::Length(3),  // Status/help
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled("Settings", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]))
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Settings list
        let items: Vec<ListItem> = Self::items()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == self.selected_index;
                let prefix = if is_selected { "▶ " } else { "  " };

                let style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let value = self.render_value(*item);

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(item.label(), style),
                    Span::raw("  "),
                    value,
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Options "));
        frame.render_widget(list, chunks[1]);

        // Status/help bar
        let help_text = if let Some(ref msg) = self.status_message {
            Line::from(vec![
                Span::styled(msg.as_str(), Style::default().fg(Color::Green)),
            ])
        } else {
            let item = self.selected_item();
            Line::from(vec![
                Span::styled(item.description(), Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled(
                    match item {
                        SettingsItem::MaxTurns => "[+/-] Adjust  [Enter] Toggle",
                        _ => "[Enter] Toggle/Execute",
                    },
                    Style::default().fg(Color::Cyan),
                ),
            ])
        };

        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[2]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        // Clear status message on any key
        self.status_message = None;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < Self::items().len() - 1 {
                    self.selected_index += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.handle_enter();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.handle_increment();
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.handle_decrement();
            }
            KeyCode::Char('q') => {
                return ViewAction::Quit;
            }
            _ => {}
        }
        ViewAction::None
    }

    fn tick(&mut self) {
        // Could add periodic refresh if needed
    }
}
