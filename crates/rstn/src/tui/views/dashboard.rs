//! Dashboard view showing project status

use crate::tui::event::WorktreeType;
use crate::tui::views::{View, ViewAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
use std::path::PathBuf;

/// Git refresh interval in ticks (30 ticks = 3 seconds at 100ms/tick)
const GIT_REFRESH_INTERVAL: u64 = 30;

/// Dashboard panel types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardPanel {
    ProjectInfo,
    RecentTests,
    GitStatus,
    QuickActions,
}

/// Dashboard view state
pub struct Dashboard {
    /// Currently focused panel
    pub focused_panel: DashboardPanel,
    /// Git status info
    pub git_branch: String,
    pub git_status: Vec<String>,
    pub worktree_count: usize,
    /// Worktree information
    pub worktree_path: Option<PathBuf>,
    pub is_git_repo: bool,
    pub worktree_type: WorktreeType,
    pub git_error: Option<String>,
    pub last_git_refresh: u64,
    /// Test results
    pub test_results: Option<TestResults>,
    /// Project info
    pub project_name: String,
    pub rust_version: String,
    /// Tick counter for refresh
    pub tick_count: u64,
    /// Quick action index
    quick_action_index: usize,
}

/// Test results summary
#[derive(Debug, Clone)]
pub struct TestResults {
    pub passed: u32,
    pub failed: u32,
    pub ignored: u32,
    pub duration_ms: u64,
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            focused_panel: DashboardPanel::QuickActions,
            git_branch: "main".to_string(),
            git_status: vec!["Loading...".to_string()],
            worktree_count: 1,
            worktree_path: None,
            is_git_repo: true, // Assume yes until proven otherwise
            worktree_type: WorktreeType::MainRepository,
            git_error: None,
            last_git_refresh: 0,
            test_results: None,
            project_name: "rustation".to_string(),
            rust_version: "1.75+".to_string(),
            tick_count: 0,
            quick_action_index: 0,
        }
    }

    /// Check if git info should be refreshed
    pub fn should_refresh_git(&self) -> bool {
        // Refresh on first tick or every GIT_REFRESH_INTERVAL ticks
        self.tick_count == 1
            || (self.is_git_repo
                && (self.tick_count - self.last_git_refresh) >= GIT_REFRESH_INTERVAL)
    }

    fn get_quick_actions() -> Vec<(&'static str, &'static str)> {
        vec![
            ("t", "Run Tests"),
            ("b", "Build Project"),
            ("c", "Check (fast)"),
            ("l", "Run Lint"),
            ("f", "Format Code"),
            ("d", "Doctor Check"),
            ("w", "Worktrees"),
            ("s", "SDD Wizard"),
        ]
    }

    fn run_action_by_key(&self, key: char) -> ViewAction {
        match key {
            't' => ViewAction::RunCommand {
                name: "test".to_string(),
                args: vec!["--lib".to_string()],
            },
            'b' => ViewAction::RunCommand {
                name: "build".to_string(),
                args: vec![],
            },
            'c' => ViewAction::RunCommand {
                name: "check".to_string(),
                args: vec![],
            },
            'l' => ViewAction::RunCommand {
                name: "lint".to_string(),
                args: vec![],
            },
            'f' => ViewAction::RunCommand {
                name: "fmt".to_string(),
                args: vec![],
            },
            'd' => ViewAction::RunCommand {
                name: "doctor".to_string(),
                args: vec![],
            },
            'w' => ViewAction::ShowWorktrees,
            's' => ViewAction::StartWizard,
            _ => ViewAction::None,
        }
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Dashboard {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Create 2x2 grid layout
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        // Project Info Panel (top-left)
        self.render_project_info(frame, top_cols[0]);

        // Git Status Panel (top-right)
        self.render_git_status(frame, top_cols[1]);

        // Recent Tests Panel (bottom-left)
        self.render_test_results(frame, bottom_cols[0]);

        // Quick Actions Panel (bottom-right)
        self.render_quick_actions(frame, bottom_cols[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            // Panel navigation with arrow keys
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                self.focused_panel = match (self.focused_panel, key.code) {
                    (DashboardPanel::ProjectInfo, KeyCode::Right) => DashboardPanel::GitStatus,
                    (DashboardPanel::ProjectInfo, KeyCode::Down) => DashboardPanel::RecentTests,
                    (DashboardPanel::GitStatus, KeyCode::Left) => DashboardPanel::ProjectInfo,
                    (DashboardPanel::GitStatus, KeyCode::Down) => DashboardPanel::QuickActions,
                    (DashboardPanel::RecentTests, KeyCode::Right) => DashboardPanel::QuickActions,
                    (DashboardPanel::RecentTests, KeyCode::Up) => DashboardPanel::ProjectInfo,
                    (DashboardPanel::QuickActions, KeyCode::Left) => DashboardPanel::RecentTests,
                    (DashboardPanel::QuickActions, KeyCode::Up) => DashboardPanel::GitStatus,
                    _ => self.focused_panel,
                };
                ViewAction::None
            }
            // Quick action shortcuts
            KeyCode::Char(c @ ('t' | 'b' | 'c' | 'l' | 'f' | 'd' | 'w' | 's')) => {
                self.run_action_by_key(c)
            }
            // Navigate quick actions in focused panel
            KeyCode::Char('j') if self.focused_panel == DashboardPanel::QuickActions => {
                let actions = Self::get_quick_actions();
                self.quick_action_index = (self.quick_action_index + 1) % actions.len();
                ViewAction::None
            }
            KeyCode::Char('k') if self.focused_panel == DashboardPanel::QuickActions => {
                let actions = Self::get_quick_actions();
                self.quick_action_index = if self.quick_action_index == 0 {
                    actions.len() - 1
                } else {
                    self.quick_action_index - 1
                };
                ViewAction::None
            }
            KeyCode::Enter if self.focused_panel == DashboardPanel::QuickActions => {
                let actions = Self::get_quick_actions();
                if let Some((key, _)) = actions.get(self.quick_action_index) {
                    self.run_action_by_key(key.chars().next().unwrap_or('t'))
                } else {
                    ViewAction::None
                }
            }
            _ => ViewAction::None,
        }
    }

    fn tick(&mut self) {
        self.tick_count += 1;
    }
}

impl Dashboard {
    fn render_project_info(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focused_panel == DashboardPanel::ProjectInfo;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Project Info ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let text = vec![
            Line::from(vec![
                Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.project_name, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Rust: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.rust_version, Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "rush - A shell in Rust",
                Style::default().fg(Color::White),
            )]),
        ];

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_git_status(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focused_panel == DashboardPanel::GitStatus;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Git Status ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let mut text = vec![];

        // Show worktree path if available
        if let Some(ref path) = self.worktree_path {
            let path_str = path.to_string_lossy();
            // Shorten path if too long
            let display_path = if path_str.len() > 40 {
                format!("...{}", &path_str[path_str.len() - 37..])
            } else {
                path_str.to_string()
            };
            text.push(Line::from(vec![
                Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
                Span::styled(display_path, Style::default().fg(Color::Cyan)),
            ]));
        }

        // Show branch
        text.push(Line::from(vec![
            Span::styled("Branch: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&self.git_branch, Style::default().fg(Color::Magenta)),
        ]));

        // Show worktree type
        let (type_label, type_color) = match &self.worktree_type {
            WorktreeType::NotGit => ("Not a git repository".to_string(), Color::DarkGray),
            WorktreeType::MainRepository => ("Main Repository".to_string(), Color::Green),
            WorktreeType::FeatureWorktree { number, .. } => {
                (format!("Feature #{}", number), Color::Yellow)
            }
        };
        text.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
            Span::styled(type_label, Style::default().fg(type_color)),
        ]));

        // Show worktree count
        text.push(Line::from(vec![
            Span::styled("Worktrees: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", self.worktree_count),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        // Show error if present
        if let Some(ref error) = self.git_error {
            text.push(Line::from(""));
            text.push(Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red)),
                Span::styled(error, Style::default().fg(Color::DarkGray)),
            ]));
        }

        // Add hint
        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "w",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to manage worktrees", Style::default().fg(Color::DarkGray)),
        ]));

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_test_results(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focused_panel == DashboardPanel::RecentTests;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Test Results ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let text = if let Some(ref results) = self.test_results {
            vec![
                Line::from(vec![
                    Span::styled("Passed: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}", results.passed),
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Failed: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}", results.failed),
                        if results.failed > 0 {
                            Style::default().fg(Color::Red)
                        } else {
                            Style::default().fg(Color::Green)
                        },
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Ignored: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}", results.ignored),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}ms", results.duration_ms),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
            ]
        } else {
            vec![Line::from(vec![Span::styled(
                "No tests run yet. Press 't' to run tests.",
                Style::default().fg(Color::DarkGray),
            )])]
        };

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_quick_actions(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focused_panel == DashboardPanel::QuickActions;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Quick Actions ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let actions = Self::get_quick_actions();
        let items: Vec<ListItem> = actions
            .iter()
            .enumerate()
            .map(|(i, (key, label))| {
                let style = if is_focused && i == self.quick_action_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("[{}] ", key), Style::default().fg(Color::Cyan)),
                    Span::styled(*label, style),
                ]))
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// Switch to next pane
    pub fn next_pane(&mut self) {
        self.focused_panel = match self.focused_panel {
            DashboardPanel::ProjectInfo => DashboardPanel::GitStatus,
            DashboardPanel::GitStatus => DashboardPanel::RecentTests,
            DashboardPanel::RecentTests => DashboardPanel::QuickActions,
            DashboardPanel::QuickActions => DashboardPanel::ProjectInfo,
        };
    }

    /// Get only the focused pane content as text
    pub fn get_focused_pane_text(&self) -> String {
        match self.focused_panel {
            DashboardPanel::ProjectInfo => vec![
                "=== Project Info ===".to_string(),
                format!("Project: {}", self.project_name),
                format!("Rust: {}", self.rust_version),
                String::new(),
                "rush - A shell in Rust".to_string(),
            ]
            .join("\n"),
            DashboardPanel::GitStatus => {
                let mut lines = vec![
                    "=== Git Status ===".to_string(),
                    format!("Branch: {}", self.git_branch),
                    format!("Worktrees: {}", self.worktree_count),
                ];
                for status in &self.git_status {
                    lines.push(format!("  {}", status));
                }
                lines.join("\n")
            }
            DashboardPanel::RecentTests => {
                let mut lines = vec!["=== Test Results ===".to_string()];
                if let Some(ref results) = self.test_results {
                    lines.push(format!("Passed: {}", results.passed));
                    lines.push(format!("Failed: {}", results.failed));
                    lines.push(format!("Ignored: {}", results.ignored));
                    lines.push(format!("Duration: {}ms", results.duration_ms));
                } else {
                    lines.push("No test results available".to_string());
                }
                lines.join("\n")
            }
            DashboardPanel::QuickActions => {
                let mut lines = vec!["=== Quick Actions ===".to_string()];
                for (key, desc) in Self::get_quick_actions() {
                    lines.push(format!("  {} - {}", key, desc));
                }
                lines.join("\n")
            }
        }
    }

    /// Get dashboard content as text
    pub fn get_output_text(&self) -> String {
        let mut lines = vec![
            "=== Project Info ===".to_string(),
            format!("Project: {}", self.project_name),
            format!("Rust: {}", self.rust_version),
            String::new(),
            "=== Git Status ===".to_string(),
        ];

        // Add worktree path if available
        if let Some(ref path) = self.worktree_path {
            lines.push(format!("Path: {}", path.display()));
        }

        lines.push(format!("Branch: {}", self.git_branch));

        // Add worktree type
        let type_label = match &self.worktree_type {
            WorktreeType::NotGit => "Not a git repository".to_string(),
            WorktreeType::MainRepository => "Main Repository".to_string(),
            WorktreeType::FeatureWorktree { number, .. } => {
                format!("Feature #{}", number)
            }
        };
        lines.push(format!("Type: {}", type_label));
        lines.push(format!("Worktrees: {}", self.worktree_count));

        for status in &self.git_status {
            lines.push(format!("  {}", status));
        }

        lines.push(String::new());
        lines.push("=== Test Results ===".to_string());
        if let Some(ref results) = self.test_results {
            lines.push(format!("Passed: {}", results.passed));
            lines.push(format!("Failed: {}", results.failed));
            lines.push(format!("Ignored: {}", results.ignored));
            lines.push(format!("Duration: {}ms", results.duration_ms));
        } else {
            lines.push("No test results available".to_string());
        }

        lines.push(String::new());
        lines.push("=== Quick Actions ===".to_string());
        for (key, desc) in Self::get_quick_actions() {
            lines.push(format!("  {} - {}", key, desc));
        }

        lines.join("\n")
    }

    /// Get styled output (Dashboard uses ANSI codes for colors)
    pub fn get_styled_output(&self) -> String {
        let mut lines = vec![
            "\x1b[36m=== Project Info ===\x1b[0m".to_string(),
            format!("Project: \x1b[36m{}\x1b[0m", self.project_name),
            format!("Rust: \x1b[32m{}\x1b[0m", self.rust_version),
            String::new(),
            "\x1b[36m=== Git Status ===\x1b[0m".to_string(),
        ];

        // Add worktree path if available
        if let Some(ref path) = self.worktree_path {
            lines.push(format!("Path: \x1b[36m{}\x1b[0m", path.display()));
        }

        lines.push(format!("Branch: \x1b[35m{}\x1b[0m", self.git_branch));

        // Add worktree type with color coding
        let (type_label, type_color) = match &self.worktree_type {
            WorktreeType::NotGit => ("Not a git repository".to_string(), "\x1b[90m"),
            WorktreeType::MainRepository => ("Main Repository".to_string(), "\x1b[32m"),
            WorktreeType::FeatureWorktree { number, .. } => {
                (format!("Feature #{}", number), "\x1b[33m")
            }
        };
        lines.push(format!("Type: {}{}\x1b[0m", type_color, type_label));
        lines.push(format!("Worktrees: \x1b[36m{}\x1b[0m", self.worktree_count));

        for status in &self.git_status {
            lines.push(format!("  {}", status));
        }

        lines.push(String::new());
        lines.push("\x1b[36m=== Test Results ===\x1b[0m".to_string());
        if let Some(ref results) = self.test_results {
            let failed_color = if results.failed > 0 {
                "\x1b[31m"
            } else {
                "\x1b[32m"
            };
            lines.push(format!("Passed: \x1b[32m{}\x1b[0m", results.passed));
            lines.push(format!("Failed: {}{}\x1b[0m", failed_color, results.failed));
            lines.push(format!("Ignored: \x1b[33m{}\x1b[0m", results.ignored));
            lines.push(format!("Duration: {}ms", results.duration_ms));
        } else {
            lines.push("No test results available".to_string());
        }

        lines.push(String::new());
        lines.push("\x1b[36m=== Quick Actions ===\x1b[0m".to_string());
        for (key, desc) in Self::get_quick_actions() {
            lines.push(format!("  \x1b[33m{}\x1b[0m - {}", key, desc));
        }

        lines.join("\n")
    }
}
