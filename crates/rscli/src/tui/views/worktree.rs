//! Worktree-focused development workspace view
//!
//! This view provides a focused workspace for feature development by:
//! - Auto-detecting current feature from branch name
//! - Displaying SDD workflow phase status
//! - Loading and showing spec/plan/tasks content
//! - Providing context-aware quick actions
//! - Showing test results for the current feature

use crate::tui::event::WorktreeType;
use crate::tui::views::{AutoFlowState, ClaudeOptions, PhaseStatus, SpecPhase, View, ViewAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use std::fs;
use std::path::{Path, PathBuf};

/// Feature information parsed from branch and verified
#[derive(Debug, Clone)]
pub struct FeatureInfo {
    pub number: String,
    pub name: String,
    pub branch: String,
    pub spec_dir: PathBuf,
}

/// Quick action item
#[derive(Debug, Clone)]
pub struct QuickAction {
    pub key: &'static str,
    pub description: &'static str,
}

impl QuickAction {
    fn new(key: &'static str, description: &'static str) -> Self {
        Self { key, description }
    }
}

/// Content type to display in middle panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Spec,
    Plan,
    Tasks,
}

impl ContentType {
    fn name(&self) -> &'static str {
        match self {
            ContentType::Spec => "Spec",
            ContentType::Plan => "Plan",
            ContentType::Tasks => "Tasks",
        }
    }
}

/// Focus area in the worktree view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorktreeFocus {
    Phases,
    Content,
    Actions,
}

/// Worktree-focused development workspace view
pub struct WorktreeView {
    // Feature context
    pub feature_info: Option<FeatureInfo>,
    pub worktree_type: WorktreeType,

    // Spec content (cached)
    pub spec_content: Option<String>,
    pub plan_content: Option<String>,
    pub tasks_content: Option<String>,

    // Phase tracking
    pub phases: Vec<(SpecPhase, PhaseStatus)>,
    pub current_phase: Option<SpecPhase>,

    // UI state
    pub focus: WorktreeFocus,
    pub phase_state: ListState,
    pub action_state: ListState,
    pub content_scroll: usize,
    pub content_type: ContentType,

    // Refresh tracking
    pub tick_count: u64,
    pub last_refresh: u64,

    // Auto-flow state for sequential phase execution
    pub auto_flow: AutoFlowState,
}

impl WorktreeView {
    const REFRESH_INTERVAL: u64 = 60; // 6 seconds at 100ms/tick

    pub fn new() -> Self {
        let mut phase_state = ListState::default();
        phase_state.select(Some(0));

        let mut action_state = ListState::default();
        action_state.select(Some(0));

        let phases = SpecPhase::all()
            .iter()
            .map(|&p| (p, PhaseStatus::NotStarted))
            .collect();

        Self {
            feature_info: None,
            worktree_type: WorktreeType::NotGit,
            spec_content: None,
            plan_content: None,
            tasks_content: None,
            phases,
            current_phase: None,
            focus: WorktreeFocus::Content,
            phase_state,
            action_state,
            content_scroll: 0,
            content_type: ContentType::Spec,
            tick_count: 0,
            last_refresh: 0,
            auto_flow: AutoFlowState::new(),
        }
    }

    /// Refresh feature detection and spec loading based on current branch
    pub fn refresh_feature(&mut self, number: String, name: String, branch: Option<String>) {
        // Try to find spec directory
        if let Ok(repo_root) = self.get_repo_root() {
            // Try both naming conventions: {number}-{name} and {number}
            let spec_dir_with_name = repo_root.join(format!("specs/{}-{}", number, name));
            let spec_dir_number_only = repo_root.join(format!("specs/{}", number));

            let spec_dir = if spec_dir_with_name.exists() {
                spec_dir_with_name
            } else if spec_dir_number_only.exists() {
                spec_dir_number_only
            } else {
                // No spec directory found
                self.clear_feature();
                return;
            };

            // Feature detected!
            self.feature_info = Some(FeatureInfo {
                number: number.clone(),
                name: name.clone(),
                branch: branch.unwrap_or_else(|| format!("{}-{}", number, name)),
                spec_dir: spec_dir.clone(),
            });

            // Load spec files
            self.load_spec_files(&spec_dir);

            // Detect phase statuses
            self.detect_phase_statuses(&spec_dir);

            // Determine current phase
            self.update_current_phase();
        } else {
            self.clear_feature();
        }
    }

    /// Clear feature info when not on a feature branch
    pub fn clear_feature(&mut self) {
        self.feature_info = None;
        self.spec_content = None;
        self.plan_content = None;
        self.tasks_content = None;
        let phases = SpecPhase::all()
            .iter()
            .map(|&p| (p, PhaseStatus::NotStarted))
            .collect();
        self.phases = phases;
        self.current_phase = None;
        self.content_scroll = 0;
    }

    /// Load spec files from the spec directory
    fn load_spec_files(&mut self, spec_dir: &Path) {
        // Load spec.md
        if let Ok(content) = fs::read_to_string(spec_dir.join("spec.md")) {
            self.spec_content = Some(content);
        } else {
            self.spec_content = None;
        }

        // Load plan.md
        if let Ok(content) = fs::read_to_string(spec_dir.join("plan.md")) {
            self.plan_content = Some(content);
        } else {
            self.plan_content = None;
        }

        // Load tasks.md
        if let Ok(content) = fs::read_to_string(spec_dir.join("tasks.md")) {
            self.tasks_content = Some(content);
        } else {
            self.tasks_content = None;
        }
    }

    /// Detect phase statuses based on file existence
    fn detect_phase_statuses(&mut self, spec_dir: &Path) {
        let mut phases = Vec::new();

        for &phase in SpecPhase::all() {
            let status = match phase {
                SpecPhase::Specify => {
                    if spec_dir.join("spec.md").exists() {
                        PhaseStatus::Completed
                    } else {
                        PhaseStatus::NotStarted
                    }
                }
                SpecPhase::Clarify => {
                    // Assume clarify is done if spec exists (simplified)
                    if spec_dir.join("spec.md").exists() {
                        PhaseStatus::Completed
                    } else {
                        PhaseStatus::NotStarted
                    }
                }
                SpecPhase::Plan => {
                    if spec_dir.join("plan.md").exists() {
                        PhaseStatus::Completed
                    } else {
                        PhaseStatus::NotStarted
                    }
                }
                SpecPhase::Tasks => {
                    if spec_dir.join("tasks.md").exists() {
                        PhaseStatus::Completed
                    } else {
                        PhaseStatus::NotStarted
                    }
                }
                SpecPhase::Analyze => {
                    // Optional phase - mark as completed if tasks exist
                    if spec_dir.join("tasks.md").exists() {
                        PhaseStatus::Completed
                    } else {
                        PhaseStatus::NotStarted
                    }
                }
                SpecPhase::Implement => {
                    // In progress if we have tasks but not done
                    // (simplified: always in progress if tasks exist)
                    if spec_dir.join("tasks.md").exists() {
                        PhaseStatus::InProgress
                    } else {
                        PhaseStatus::NotStarted
                    }
                }
                SpecPhase::Review => {
                    // Not started by default (would need PR detection)
                    PhaseStatus::NotStarted
                }
            };
            phases.push((phase, status));
        }

        self.phases = phases;
    }

    /// Update current phase based on phase statuses
    fn update_current_phase(&mut self) {
        // Find first non-completed phase
        for (phase, status) in &self.phases {
            if *status != PhaseStatus::Completed {
                self.current_phase = Some(*phase);
                return;
            }
        }
        // All completed - current phase is Review
        self.current_phase = Some(SpecPhase::Review);
    }

    /// Get context-aware quick actions based on current phase
    fn get_quick_actions(&self) -> Vec<QuickAction> {
        let mut actions = vec![];

        if self.feature_info.is_none() {
            actions.push(QuickAction::new("n", "Create new feature"));
            return actions;
        }

        match self.current_phase {
            Some(SpecPhase::Specify) | Some(SpecPhase::Clarify) => {
                actions.push(QuickAction::new("e", "Edit spec.md"));
                actions.push(QuickAction::new("c", "Run /speckit.clarify"));
                actions.push(QuickAction::new("p", "Run /speckit.plan"));
            }
            Some(SpecPhase::Plan) => {
                actions.push(QuickAction::new("v", "View plan.md"));
                actions.push(QuickAction::new("t", "Run /speckit.tasks"));
                actions.push(QuickAction::new("e", "Edit plan.md"));
            }
            Some(SpecPhase::Tasks) => {
                actions.push(QuickAction::new("v", "View tasks.md"));
                actions.push(QuickAction::new("i", "Run /speckit.implement"));
                actions.push(QuickAction::new("t", "Run tests"));
            }
            Some(SpecPhase::Implement) | Some(SpecPhase::Analyze) => {
                actions.push(QuickAction::new("t", "Run tests"));
                actions.push(QuickAction::new("b", "Build project"));
                actions.push(QuickAction::new("l", "Run lint"));
                actions.push(QuickAction::new("i", "Run /speckit.implement"));
            }
            Some(SpecPhase::Review) => {
                actions.push(QuickAction::new("r", "Run /speckit.review"));
                actions.push(QuickAction::new("t", "Run tests"));
                actions.push(QuickAction::new("p", "Create PR"));
            }
            None => {
                actions.push(QuickAction::new("n", "Create new feature"));
            }
        }

        // Common actions
        actions.push(QuickAction::new("s", "Switch content"));

        actions
    }

    /// Get repository root
    fn get_repo_root(&self) -> Result<PathBuf, std::io::Error> {
        std::env::current_dir()
    }

    /// Get current content to display
    fn get_current_content(&self) -> Option<&str> {
        match self.content_type {
            ContentType::Spec => self.spec_content.as_deref(),
            ContentType::Plan => self.plan_content.as_deref(),
            ContentType::Tasks => self.tasks_content.as_deref(),
        }
    }

    /// Move focus left
    fn focus_left(&mut self) {
        self.focus = match self.focus {
            WorktreeFocus::Content => WorktreeFocus::Phases,
            WorktreeFocus::Actions => WorktreeFocus::Content,
            WorktreeFocus::Phases => WorktreeFocus::Actions,
        };
    }

    /// Move focus right
    fn focus_right(&mut self) {
        self.focus = match self.focus {
            WorktreeFocus::Phases => WorktreeFocus::Content,
            WorktreeFocus::Content => WorktreeFocus::Actions,
            WorktreeFocus::Actions => WorktreeFocus::Phases,
        };
    }

    /// Move to next pane (same as focus_right, for Tab key compatibility)
    pub fn next_pane(&mut self) {
        self.focus_right();
    }

    /// Scroll content down
    fn scroll_down(&mut self) {
        match self.focus {
            WorktreeFocus::Phases => {
                let i = self.phase_state.selected().unwrap_or(0);
                let new_i = (i + 1).min(self.phases.len().saturating_sub(1));
                self.phase_state.select(Some(new_i));
            }
            WorktreeFocus::Content => {
                if let Some(content) = self.get_current_content() {
                    let line_count = content.lines().count();
                    if self.content_scroll < line_count.saturating_sub(1) {
                        self.content_scroll += 1;
                    }
                }
            }
            WorktreeFocus::Actions => {
                let actions = self.get_quick_actions();
                let i = self.action_state.selected().unwrap_or(0);
                let new_i = (i + 1).min(actions.len().saturating_sub(1));
                self.action_state.select(Some(new_i));
            }
        }
    }

    /// Scroll content up
    fn scroll_up(&mut self) {
        match self.focus {
            WorktreeFocus::Phases => {
                let i = self.phase_state.selected().unwrap_or(0);
                let new_i = i.saturating_sub(1);
                self.phase_state.select(Some(new_i));
            }
            WorktreeFocus::Content => {
                self.content_scroll = self.content_scroll.saturating_sub(1);
            }
            WorktreeFocus::Actions => {
                let i = self.action_state.selected().unwrap_or(0);
                let new_i = i.saturating_sub(1);
                self.action_state.select(Some(new_i));
            }
        }
    }

    /// Switch content type (cycle through Spec -> Plan -> Tasks)
    fn switch_content(&mut self) {
        self.content_type = match self.content_type {
            ContentType::Spec => ContentType::Plan,
            ContentType::Plan => ContentType::Tasks,
            ContentType::Tasks => ContentType::Spec,
        };
        self.content_scroll = 0;
    }

    /// Get the currently selected phase from the phases list
    fn get_selected_phase(&self) -> Option<SpecPhase> {
        self.phase_state
            .selected()
            .and_then(|i| self.phases.get(i).map(|(phase, _status)| *phase))
    }

    /// Run the selected phase and switch to Commands view
    fn run_phase(&self, phase: SpecPhase) -> ViewAction {
        ViewAction::RunSpecPhase {
            phase: phase.name().to_string(),
            command: phase.command().to_string(),
            options: ClaudeOptions {
                max_turns: 50,
                skip_permissions: false,
                continue_session: false,
                session_id: None,
                allowed_tools: Vec::new(),
            },
        }
    }

    /// Update the status of a specific phase
    pub fn update_phase_status(&mut self, phase_name: &str, status: PhaseStatus) {
        if let Some((_, existing_status)) = self.phases.iter_mut()
            .find(|(p, _)| p.name() == phase_name) {
            *existing_status = status;
        }
    }

    /// Run phase with auto-flow support
    pub fn run_phase_with_auto_flow(&self, phase: SpecPhase) -> ViewAction {
        if self.auto_flow.active {
            // Auto-flow mode: use auto-flow options
            ViewAction::RunSpecPhase {
                phase: phase.name().to_string(),
                command: phase.command().to_string(),
                options: self.auto_flow.options.clone(),
            }
        } else {
            // Interactive mode: run single phase with default options
            self.run_phase(phase)
        }
    }

    /// Get focused pane text for copying
    pub fn get_focused_pane_text(&self) -> String {
        match self.focus {
            WorktreeFocus::Phases => {
                // Return phase list
                self.phases
                    .iter()
                    .map(|(phase, status)| {
                        format!("{} {}", status.symbol(), phase.display_name())
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            WorktreeFocus::Content => {
                // Return current content
                self.get_current_content().unwrap_or("").to_string()
            }
            WorktreeFocus::Actions => {
                // Return quick actions list
                self.get_quick_actions()
                    .iter()
                    .map(|action| format!("[{}] {}", action.key, action.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }

    /// Get styled output for copying (with ANSI codes)
    pub fn get_styled_output(&self) -> String {
        // For now, just return the focused pane text
        // Could add ANSI color codes later
        self.get_focused_pane_text()
    }

    /// Render left panel (phases)
    fn render_phases(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == WorktreeFocus::Phases;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" SDD Phases ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let items: Vec<ListItem> = self
            .phases
            .iter()
            .map(|(phase, status)| {
                let symbol = status.symbol();
                let color = status.color();
                ListItem::new(vec![Line::from(vec![
                    Span::styled(symbol, Style::default().fg(color)),
                    Span::raw(" "),
                    Span::styled(phase.display_name(), Style::default().fg(Color::White)),
                ])])
            })
            .collect();

        // Add feature info at bottom
        let mut footer_lines = vec![];
        if let Some(ref info) = self.feature_info {
            footer_lines.push(Line::from(""));
            footer_lines.push(Line::from(vec![
                Span::styled("Feature: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("#{}", info.number),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
            footer_lines.push(Line::from(vec![
                Span::styled("Branch: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&info.branch, Style::default().fg(Color::Green)),
            ]));
        }

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.phase_state.clone());

        // Render footer with feature info
        if !footer_lines.is_empty() && area.height > 10 {
            let footer_area = Rect {
                x: area.x + 1,
                y: area.y + area.height.saturating_sub(4),
                width: area.width.saturating_sub(2),
                height: 3,
            };
            let footer = Paragraph::new(footer_lines);
            frame.render_widget(footer, footer_area);
        }
    }

    /// Render middle panel (content)
    fn render_content(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == WorktreeFocus::Content;

        let title = if let Some(ref info) = self.feature_info {
            format!(
                " {} - Feature #{} ",
                self.content_type.name(),
                info.number
            )
        } else {
            format!(" {} ", self.content_type.name())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let content_lines: Vec<Line> = if let Some(content) = self.get_current_content() {
            content
                .lines()
                .skip(self.content_scroll)
                .take(area.height.saturating_sub(2) as usize)
                .map(|line| Line::from(line.to_string()))
                .collect()
        } else if self.feature_info.is_some() {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("No {} file found", self.content_type.name().to_lowercase()),
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 's' to switch to another content type",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No feature detected",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("To work on a feature:"),
                Line::from(""),
                Line::from(Span::styled(
                    "1. Switch to feature branch: git checkout NNN-feature-name",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "2. Or create new feature: press 'n'",
                    Style::default().fg(Color::Cyan),
                )),
            ]
        };

        let paragraph = Paragraph::new(content_lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render right panel (actions)
    fn render_actions(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == WorktreeFocus::Actions;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Quick Actions ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let actions = self.get_quick_actions();
        let items: Vec<ListItem> = actions
            .iter()
            .map(|action| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("[{}]", action.key),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(action.description, Style::default().fg(Color::White)),
                    ]),
                ])
            })
            .collect();

        // Add current phase info
        let mut footer_lines = vec![Line::from("")];
        if let Some(phase) = self.current_phase {
            let default_status = PhaseStatus::NotStarted;
            let status = self
                .phases
                .iter()
                .find(|(p, _)| p == &phase)
                .map(|(_, s)| s)
                .unwrap_or(&default_status);

            footer_lines.push(Line::from(vec![
                Span::styled("Current: ", Style::default().fg(Color::DarkGray)),
                Span::styled(status.symbol(), Style::default().fg(status.color())),
                Span::raw(" "),
                Span::styled(phase.display_name(), Style::default().fg(Color::Yellow)),
            ]));
        }

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.action_state.clone());

        // Render footer with current phase
        if !footer_lines.is_empty() && area.height > 10 {
            let footer_area = Rect {
                x: area.x + 1,
                y: area.y + area.height.saturating_sub(3),
                width: area.width.saturating_sub(2),
                height: 2,
            };
            let footer = Paragraph::new(footer_lines);
            frame.render_widget(footer, footer_area);
        }
    }
}

impl Default for WorktreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for WorktreeView {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Split into 3 columns
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Phases
                Constraint::Percentage(50), // Content
                Constraint::Percentage(25), // Actions
            ])
            .split(area);

        self.render_phases(frame, columns[0]);
        self.render_content(frame, columns[1]);
        self.render_actions(frame, columns[2]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => {
                self.focus_left();
                ViewAction::None
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.focus_right();
                ViewAction::None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_down();
                ViewAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up();
                ViewAction::None
            }
            KeyCode::Char('s') => {
                self.switch_content();
                ViewAction::None
            }
            KeyCode::PageDown => {
                if self.focus == WorktreeFocus::Content {
                    if let Some(content) = self.get_current_content() {
                        let line_count = content.lines().count();
                        self.content_scroll = (self.content_scroll + 10)
                            .min(line_count.saturating_sub(1));
                    }
                }
                ViewAction::None
            }
            KeyCode::PageUp => {
                if self.focus == WorktreeFocus::Content {
                    self.content_scroll = self.content_scroll.saturating_sub(10);
                }
                ViewAction::None
            }
            KeyCode::Home | KeyCode::Char('g') => {
                if self.focus == WorktreeFocus::Content {
                    self.content_scroll = 0;
                }
                ViewAction::None
            }
            KeyCode::End | KeyCode::Char('G') => {
                if self.focus == WorktreeFocus::Content {
                    if let Some(content) = self.get_current_content() {
                        let line_count = content.lines().count();
                        self.content_scroll = line_count.saturating_sub(1);
                    }
                }
                ViewAction::None
            }
            KeyCode::Enter => {
                // Start the selected phase if in Phases panel
                if self.focus == WorktreeFocus::Phases {
                    if let Some(phase) = self.get_selected_phase() {
                        return self.run_phase(phase);
                    }
                }
                ViewAction::None
            }
            _ => ViewAction::None,
        }
    }

    fn tick(&mut self) {
        self.tick_count += 1;

        // Refresh feature detection periodically
        if self.tick_count % Self::REFRESH_INTERVAL == 0 {
            // Refresh will be triggered by GitInfoUpdated event
            // No action needed here
        }
    }
}
