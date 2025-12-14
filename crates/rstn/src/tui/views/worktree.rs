//! Worktree-focused development workspace view
//!
//! This view provides a focused workspace for feature development by:
//! - Auto-detecting current feature from branch name
//! - Displaying SDD workflow phase status
//! - Loading and showing spec/plan/tasks content
//! - Providing context-aware quick actions
//! - Showing test results for the current feature

use crate::tui::event::WorktreeType;
use crate::tui::logging::{FileChangeTracker, LogBuffer, LogCategory, LogEntry};
use crate::tui::views::{AutoFlowState, ClaudeOptions, PhaseStatus, SpecPhase, View, ViewAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
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

/// Git command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitCommand {
    Commit,
    Push,
    Status,
    AddAll,
    Rebase,
}

impl GitCommand {
    /// Get all git commands in display order
    pub fn all() -> &'static [GitCommand] {
        &[
            GitCommand::Commit,
            GitCommand::Push,
            GitCommand::Status,
            GitCommand::AddAll,
            GitCommand::Rebase,
        ]
    }

    /// Get display name for this git command
    pub fn display_name(&self) -> &'static str {
        match self {
            GitCommand::Commit => "Commit",
            GitCommand::Push => "Push",
            GitCommand::Status => "Status",
            GitCommand::AddAll => "Add All",
            GitCommand::Rebase => "Rebase",
        }
    }
}

/// Unified command that can be either an SDD phase or a Git action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    SddPhase(SpecPhase, PhaseStatus),
    GitAction(GitCommand),
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
    Commands, // Unified panel for SDD phases and Git actions
    Content,
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
    pub command_state: ListState, // Unified command list state
    pub content_scroll: usize,
    pub content_type: ContentType,

    // Unified command list (SDD phases + Git actions)
    pub commands: Vec<Command>,
    pub pending_git_command: Option<GitCommand>,

    // Refresh tracking
    pub tick_count: u64,
    pub last_refresh: u64,

    // Auto-flow state for sequential phase execution
    pub auto_flow: AutoFlowState,

    // Command output display (using LogBuffer from Phase 1)
    pub log_buffer: LogBuffer,
    pub file_tracker: FileChangeTracker,
    pub last_file_check_tick: u64,
    pub output_scroll: usize,
    pub is_running: bool,
    pub running_phase: Option<String>,
    pub spinner_frame: usize,

    // Input handling
    pub pending_input_phase: Option<SpecPhase>,
    pub active_session_id: Option<String>,
    pub pending_follow_up: bool,

    // Progress tracking
    pub progress_step: Option<u32>,
    pub progress_total: Option<u32>,
    pub progress_message: Option<String>,

    // Commit workflow state
    pub pending_commit_message: Option<String>,
    pub commit_warnings: Vec<rstn_core::SecurityWarning>,
}

impl WorktreeView {
    const REFRESH_INTERVAL: u64 = 60; // 6 seconds at 100ms/tick

    pub fn new() -> Self {
        let mut phase_state = ListState::default();
        phase_state.select(Some(0));

        let mut command_state = ListState::default();
        command_state.select(Some(1)); // Start on first SDD phase (Specify), not header

        let phases = SpecPhase::all()
            .iter()
            .map(|&p| (p, PhaseStatus::NotStarted))
            .collect::<Vec<_>>();

        // Build unified command list (SDD phases + Git commands)
        let mut commands = Vec::new();
        for (phase, status) in &phases {
            commands.push(Command::SddPhase(*phase, *status));
        }
        for git_cmd in GitCommand::all() {
            commands.push(Command::GitAction(*git_cmd));
        }

        Self {
            feature_info: None,
            worktree_type: WorktreeType::NotGit,
            spec_content: None,
            plan_content: None,
            tasks_content: None,
            phases,
            current_phase: None,
            focus: WorktreeFocus::Commands,
            phase_state,
            command_state,
            content_scroll: 0,
            content_type: ContentType::Spec,
            commands,
            pending_git_command: None,
            tick_count: 0,
            last_refresh: 0,
            auto_flow: AutoFlowState::new(),
            log_buffer: LogBuffer::new(),
            file_tracker: FileChangeTracker::new(),
            last_file_check_tick: 0,
            output_scroll: 0,
            is_running: false,
            running_phase: None,
            spinner_frame: 0,
            pending_input_phase: None,
            active_session_id: None,
            pending_follow_up: false,
            progress_step: None,
            progress_total: None,
            progress_message: None,
            pending_commit_message: None,
            commit_warnings: Vec::new(),
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

    /// Map display index to command index (accounting for headers and separators)
    fn display_index_to_command_index(&self, display_idx: usize) -> Option<usize> {
        // Display indices:
        // 0: "SDD WORKFLOW" header (not selectable)
        // 1-7: SDD phases (commands 0-6)
        // 8: separator (not selectable)
        // 9: "GIT ACTIONS" header (not selectable)
        // 10+: Git commands (commands 7+)

        let num_sdd_phases = self.phases.len();

        if display_idx == 0 {
            // "SDD WORKFLOW" header - not selectable
            None
        } else if display_idx <= num_sdd_phases {
            // SDD phases: display index 1-7 maps to commands 0-6
            Some(display_idx - 1)
        } else if display_idx == num_sdd_phases + 1 {
            // Separator - not selectable
            None
        } else if display_idx == num_sdd_phases + 2 {
            // "GIT ACTIONS" header - not selectable
            None
        } else {
            // Git commands: display index 10+ maps to commands 7+
            Some(display_idx - 3)
        }
    }

    /// Move focus left
    fn focus_left(&mut self) {
        self.focus = match self.focus {
            WorktreeFocus::Content => WorktreeFocus::Commands,
            WorktreeFocus::Commands => WorktreeFocus::Content,
        };
    }

    /// Move focus right
    fn focus_right(&mut self) {
        self.focus = match self.focus {
            WorktreeFocus::Commands => WorktreeFocus::Content,
            WorktreeFocus::Content => WorktreeFocus::Commands,
        };
    }

    /// Move to next pane (same as focus_right, for Tab key compatibility)
    pub fn next_pane(&mut self) {
        self.focus_right();
    }

    /// Scroll content down
    fn scroll_down(&mut self) {
        match self.focus {
            WorktreeFocus::Commands => {
                let current_idx = self.command_state.selected().unwrap_or(0);
                // Calculate total display items: header + phases + separator + header + git commands
                let num_sdd_phases = self.phases.len();
                let total_display_items = 1 + num_sdd_phases + 1 + 1 + GitCommand::all().len();

                // Find next selectable item
                let mut new_idx = current_idx + 1;
                while new_idx < total_display_items {
                    if self.display_index_to_command_index(new_idx).is_some() {
                        self.command_state.select(Some(new_idx));
                        return;
                    }
                    new_idx += 1;
                }
                // If we couldn't find a next selectable item, stay at current
            }
            WorktreeFocus::Content => {
                if let Some(content) = self.get_current_content() {
                    let line_count = content.lines().count();
                    if self.content_scroll < line_count.saturating_sub(1) {
                        self.content_scroll += 1;
                    }
                }
            }
        }
    }

    /// Scroll content up
    fn scroll_up(&mut self) {
        match self.focus {
            WorktreeFocus::Commands => {
                let current_idx = self.command_state.selected().unwrap_or(0);

                // Find previous selectable item
                if current_idx > 0 {
                    let mut new_idx = current_idx - 1;
                    loop {
                        if self.display_index_to_command_index(new_idx).is_some() {
                            self.command_state.select(Some(new_idx));
                            return;
                        }
                        if new_idx == 0 {
                            break;
                        }
                        new_idx -= 1;
                    }
                }
                // If we couldn't find a previous selectable item, stay at current
            }
            WorktreeFocus::Content => {
                self.content_scroll = self.content_scroll.saturating_sub(1);
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
        if let Some((_, existing_status)) =
            self.phases.iter_mut().find(|(p, _)| p.name() == phase_name)
        {
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

    /// Get default Claude CLI options
    pub fn get_claude_options(&self) -> ClaudeOptions {
        ClaudeOptions {
            max_turns: 50,
            skip_permissions: false,
            continue_session: false,
            session_id: None,
            allowed_tools: Vec::new(),
        }
    }

    /// Start a command and track it
    pub fn start_command(&mut self, phase: SpecPhase, session_id: Option<&str>) {
        self.is_running = true;
        self.running_phase = Some(phase.name().to_string());
        // Note: We keep log_buffer for history, don't clear it
        self.output_scroll = 0;
        self.active_session_id = session_id.map(|s| s.to_string());
    }

    /// Get the current session ID if active
    pub fn get_session_id(&self) -> Option<String> {
        self.active_session_id.clone()
    }

    /// Check if output is being shown
    pub fn is_showing_output(&self) -> bool {
        !self.log_buffer.is_empty() || self.is_running
    }

    /// Add output line (logs it with ClaudeStream category)
    pub fn add_output(&mut self, line: String) {
        self.log(LogCategory::ClaudeStream, line);
    }

    /// Mark command as done
    pub fn command_done(&mut self) {
        self.is_running = false;
        self.running_phase = None;
        self.active_session_id = None;
    }

    /// Update progress display
    pub fn update_progress(&mut self, _phase: &str, step: u32, total: u32, message: &str) {
        self.progress_step = Some(step);
        self.progress_total = Some(total);
        self.progress_message = Some(message.to_string());
    }

    /// Clear progress display
    pub fn clear_progress(&mut self) {
        self.progress_step = None;
        self.progress_total = None;
        self.progress_message = None;
    }

    /// Log a message with timestamp and category
    pub fn log(&mut self, category: LogCategory, content: String) {
        let entry = LogEntry::new(category, content);
        self.log_buffer.push(entry);

        // Auto-scroll to bottom on new entries
        let total_lines = self.log_buffer.len();
        if total_lines > 20 {
            self.output_scroll = total_lines.saturating_sub(20);
        }
    }

    /// Log slash command execution
    pub fn log_slash_command(&mut self, command: &str) {
        self.log(LogCategory::SlashCommand, command.to_string());
        self.log(LogCategory::System, "─".repeat(60)); // Separator
    }

    /// Log file change
    pub fn log_file_change(&mut self, path: &Path) {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        self.log(
            LogCategory::FileChange,
            format!("File updated: {}", filename)
        );
    }

    /// Log shell command
    pub fn log_shell_command(&mut self, script: &str, exit_code: i32) {
        self.log(
            LogCategory::ShellOutput,
            format!("{} completed (exit: {})", script, exit_code)
        );
    }

    /// Clear output (note: this clears the log buffer)
    pub fn clear_output(&mut self) {
        // Note: We keep the log buffer for history
        // This method is kept for backward compatibility but doesn't clear logs
        self.output_scroll = 0;
    }

    /// Handle git command execution
    fn handle_git_command(&mut self, git_cmd: GitCommand) -> ViewAction {
        match git_cmd {
            GitCommand::Commit => {
                // NEW: Enhanced commit workflow with security scanning
                ViewAction::RunEnhancedCommit
            }
            GitCommand::Push => {
                // Run git push directly
                ViewAction::RunCommand {
                    name: "git".to_string(),
                    args: vec!["push".to_string()],
                }
            }
            GitCommand::Status => {
                // Run git status directly
                ViewAction::RunCommand {
                    name: "git".to_string(),
                    args: vec!["status".to_string()],
                }
            }
            GitCommand::AddAll => {
                // Run git add --all directly
                ViewAction::RunCommand {
                    name: "git".to_string(),
                    args: vec!["add".to_string(), "--all".to_string()],
                }
            }
            GitCommand::Rebase => {
                // Store pending git command and request branch name
                self.pending_git_command = Some(git_cmd);
                ViewAction::RequestInput {
                    prompt: "Rebase onto branch:".to_string(),
                    placeholder: Some("main".to_string()),
                }
            }
        }
    }

    /// Get focused pane text for copying
    pub fn get_focused_pane_text(&self) -> String {
        match self.focus {
            WorktreeFocus::Commands => {
                // Return command list (SDD phases + Git actions)
                let mut lines = Vec::new();
                lines.push("SDD WORKFLOW".to_string());
                for (phase, status) in &self.phases {
                    lines.push(format!("{} {}", status.symbol(), phase.display_name()));
                }
                lines.push(String::new());
                lines.push("GIT ACTIONS".to_string());
                for git_cmd in GitCommand::all() {
                    lines.push(format!("• {}", git_cmd.display_name()));
                }
                lines.join("\n")
            }
            WorktreeFocus::Content => {
                // Return current content
                self.get_current_content().unwrap_or("").to_string()
            }
        }
    }

    /// Get styled output for copying (with ANSI codes)
    pub fn get_styled_output(&self) -> String {
        // For now, just return the focused pane text
        // Could add ANSI color codes later
        self.get_focused_pane_text()
    }

    /// Render left panel (commands - unified SDD phases and Git actions)
    fn render_commands(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == WorktreeFocus::Commands;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Commands ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let mut items = Vec::new();

        // SDD WORKFLOW section header
        items.push(ListItem::new(vec![Line::from(vec![Span::styled(
            "SDD WORKFLOW",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])]));

        // SDD phase commands
        for (phase, status) in &self.phases {
            let symbol = status.symbol();
            let color = status.color();
            items.push(ListItem::new(vec![Line::from(vec![
                Span::styled(symbol, Style::default().fg(color)),
                Span::raw(" "),
                Span::styled(phase.display_name(), Style::default().fg(Color::White)),
            ])]));
        }

        // Separator
        items.push(ListItem::new(Line::from("")));

        // GIT ACTIONS section header
        items.push(ListItem::new(vec![Line::from(vec![Span::styled(
            "GIT ACTIONS",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])]));

        // Git commands
        for git_cmd in GitCommand::all() {
            items.push(ListItem::new(vec![Line::from(vec![
                Span::raw("• "),
                Span::styled(git_cmd.display_name(), Style::default().fg(Color::White)),
            ])]));
        }

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

        frame.render_stateful_widget(list, area, &mut self.command_state.clone());

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

        // Split area: Tabs (3 lines) + Content (remaining)
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tab bar with border
                Constraint::Min(0),     // Content area
            ])
            .split(area);

        // Determine selected tab index
        let selected_idx = match self.content_type {
            ContentType::Spec => 0,
            ContentType::Plan => 1,
            ContentType::Tasks => 2,
        };

        // Render tab bar
        let tab_titles = vec!["Spec", "Plan", "Tasks"];
        let tab_title = if let Some(ref info) = self.feature_info {
            format!(" Content - Feature #{} ", info.number)
        } else {
            " Content ".to_string()
        };

        let tabs = Tabs::new(tab_titles)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(tab_title)
                .border_style(if is_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }))
            .select(selected_idx)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD));

        frame.render_widget(tabs, sections[0]);

        // Render content area
        let content_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let content_lines: Vec<Line> = if let Some(content) = self.get_current_content() {
            content
                .lines()
                .skip(self.content_scroll)
                .take(sections[1].height.saturating_sub(2) as usize)
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
                    "Press ← → or h/l to switch tabs",
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
            .block(content_block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, sections[1]);
    }

    /// Render output panel (bottom-right section) with comprehensive logging
    fn render_output(&self, frame: &mut Frame, area: Rect) {
        // Dynamic title based on running state
        let title = if self.is_running {
            let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];
            let spinner_char = spinner[self.spinner_frame % spinner.len()];
            if let Some(ref phase) = self.running_phase {
                format!(" Output {} Running: {} ", spinner_char, phase)
            } else {
                format!(" Output {} Running... ", spinner_char)
            }
        } else {
            format!(" Output (1000 line history) ")
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default());

        // Build output lines with timestamps, icons, and category-based styling
        let lines: Vec<Line> = if self.log_buffer.is_empty() && !self.is_running {
            // Show placeholder when no output
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No output yet",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Run a command or SDD phase to see output here",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            let visible_height = area.height.saturating_sub(2) as usize;
            self.log_buffer
                .entries()
                .skip(self.output_scroll)
                .take(visible_height)
                .map(|entry| {
                    let timestamp = entry.format_timestamp();
                    let icon = entry.category_icon();
                    let color = entry.category.color();

                    Line::from(vec![
                        Span::styled(
                            format!("[{}] ", timestamp),
                            Style::default().fg(Color::DarkGray)
                        ),
                        Span::raw(format!("{} ", icon)),
                        Span::styled(&entry.content, Style::default().fg(color)),
                    ])
                })
                .collect()
        };

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Check for file changes and reload content if modified
    fn check_file_changes(&mut self) {
        if let Some(ref info) = self.feature_info {
            let files = vec![
                info.spec_dir.join("spec.md"),
                info.spec_dir.join("plan.md"),
                info.spec_dir.join("tasks.md"),
            ];

            let changed = self.file_tracker.check_files(&files);

            for path in changed {
                // Reload file content
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    match filename {
                        "spec.md" => self.spec_content = Some(content),
                        "plan.md" => self.plan_content = Some(content),
                        "tasks.md" => self.tasks_content = Some(content),
                        _ => {}
                    }
                }

                // Log the change
                self.log_file_change(&path);
            }
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
        // Create two columns: Commands (30%) | Right panel (70%)
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Commands
                Constraint::Percentage(70), // Right panel (to be split vertically)
            ])
            .split(area);

        // Split right column vertically: Content (70%) | Output (30%)
        let right_sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70), // Content
                Constraint::Percentage(30), // Output
            ])
            .split(columns[1]);

        // Render all three panels
        self.render_commands(frame, columns[0]);
        self.render_content(frame, right_sections[0]);
        self.render_output(frame, right_sections[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => {
                if self.focus == WorktreeFocus::Content {
                    // Cycle tabs left when Content is focused
                    self.content_type = match self.content_type {
                        ContentType::Spec => ContentType::Tasks,
                        ContentType::Plan => ContentType::Spec,
                        ContentType::Tasks => ContentType::Plan,
                    };
                    self.content_scroll = 0;
                } else {
                    self.focus_left();
                }
                ViewAction::None
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.focus == WorktreeFocus::Content {
                    // Cycle tabs right when Content is focused
                    self.switch_content();
                } else {
                    self.focus_right();
                }
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
                        self.content_scroll =
                            (self.content_scroll + 10).min(line_count.saturating_sub(1));
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
                // Execute selected command (SDD phase or Git action)
                if self.focus == WorktreeFocus::Commands {
                    if let Some(display_idx) = self.command_state.selected() {
                        // Map display index to actual command index
                        if let Some(cmd_idx) = self.display_index_to_command_index(display_idx) {
                            if let Some(command) = self.commands.get(cmd_idx) {
                                return match command {
                                    Command::SddPhase(phase, _) => {
                                        // Specify phase needs user input first
                                        if *phase == SpecPhase::Specify {
                                            self.pending_input_phase = Some(*phase);
                                            ViewAction::RequestInput {
                                                prompt: "Enter feature description:".to_string(),
                                                placeholder: Some(
                                                    "e.g., Add user authentication with OAuth2"
                                                        .to_string(),
                                                ),
                                            }
                                        } else {
                                            self.run_phase(*phase)
                                        }
                                    }
                                    Command::GitAction(git_cmd) => {
                                        self.handle_git_command(*git_cmd)
                                    }
                                };
                            }
                        }
                    }
                }
                ViewAction::None
            }
            _ => ViewAction::None,
        }
    }

    fn tick(&mut self) {
        self.tick_count += 1;

        // Update spinner animation when running
        if self.is_running {
            self.spinner_frame = (self.spinner_frame + 1) % 8;
        }

        // Check for file changes every 10 ticks (1 second at 100ms/tick)
        if self.tick_count - self.last_file_check_tick >= 10 {
            self.check_file_changes();
            self.last_file_check_tick = self.tick_count;
        }

        // Refresh feature detection periodically
        if self.tick_count % Self::REFRESH_INTERVAL == 0 {
            // Refresh will be triggered by GitInfoUpdated event
            // No action needed here
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Helper to create a test worktree view
    fn create_test_view() -> WorktreeView {
        WorktreeView::new()
    }

    /// Helper to simulate key press
    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_display_index_to_command_index_mapping() {
        let view = create_test_view();
        let num_phases = view.phases.len();

        // Index 0: "SDD WORKFLOW" header - should return None
        assert_eq!(view.display_index_to_command_index(0), None);

        // Indices 1-7: SDD phases - should map to commands 0-6
        for i in 1..=num_phases {
            assert_eq!(
                view.display_index_to_command_index(i),
                Some(i - 1),
                "Display index {} should map to command index {}",
                i,
                i - 1
            );
        }

        // Index 8: separator - should return None
        assert_eq!(view.display_index_to_command_index(num_phases + 1), None);

        // Index 9: "GIT ACTIONS" header - should return None
        assert_eq!(view.display_index_to_command_index(num_phases + 2), None);

        // Indices 10+: Git commands - should map to commands 7+
        let git_count = GitCommand::all().len();
        for i in 0..git_count {
            let display_idx = num_phases + 3 + i;
            let expected_cmd_idx = num_phases + i;
            assert_eq!(
                view.display_index_to_command_index(display_idx),
                Some(expected_cmd_idx),
                "Display index {} should map to command index {}",
                display_idx,
                expected_cmd_idx
            );
        }
    }

    #[test]
    fn test_initial_selection_on_selectable_item() {
        let view = create_test_view();

        // Initial selection should be on index 1 (first SDD phase - Specify)
        assert_eq!(view.command_state.selected(), Some(1));

        // Verify index 1 maps to a valid command (command 0 = Specify)
        assert_eq!(view.display_index_to_command_index(1), Some(0));

        // Verify command 0 is indeed Specify
        match &view.commands[0] {
            Command::SddPhase(phase, _) => {
                assert_eq!(phase.name(), "specify");
            }
            _ => panic!("First command should be SDD phase Specify"),
        }
    }

    #[test]
    fn test_scroll_down_skips_headers_and_separators() {
        let mut view = create_test_view();
        let num_phases = view.phases.len();

        // Set focus to Commands panel
        view.focus = WorktreeFocus::Commands;

        // Start at first SDD phase (Specify, index 1)
        view.command_state.select(Some(1));

        // Scroll down through all SDD phases
        for i in 2..=num_phases {
            view.scroll_down();
            assert_eq!(view.command_state.selected(), Some(i));
            // Verify it's a selectable item
            assert!(view.display_index_to_command_index(i).is_some());
        }

        // Next scroll should skip separator (index 8) and header (index 9)
        // and land on first git command (index 10)
        view.scroll_down();
        let expected_git_start = num_phases + 3;
        assert_eq!(view.command_state.selected(), Some(expected_git_start));
        assert!(view
            .display_index_to_command_index(expected_git_start)
            .is_some());

        // Verify it's a git command
        if let Some(cmd_idx) = view.display_index_to_command_index(expected_git_start) {
            match &view.commands[cmd_idx] {
                Command::GitAction(_) => {} // Expected
                _ => panic!("Should be a git command"),
            }
        }
    }

    #[test]
    fn test_scroll_up_skips_headers_and_separators() {
        let mut view = create_test_view();
        let num_phases = view.phases.len();

        // Set focus to Commands panel
        view.focus = WorktreeFocus::Commands;

        // Start at first git command (index 10)
        let git_start_idx = num_phases + 3;
        view.command_state.select(Some(git_start_idx));

        // Scroll up should skip header (index 9) and separator (index 8)
        // and land on last SDD phase (index 7)
        view.scroll_up();
        assert_eq!(view.command_state.selected(), Some(num_phases));
        assert!(view.display_index_to_command_index(num_phases).is_some());

        // Verify it's an SDD phase
        if let Some(cmd_idx) = view.display_index_to_command_index(num_phases) {
            match &view.commands[cmd_idx] {
                Command::SddPhase(_, _) => {} // Expected
                _ => panic!("Should be an SDD phase"),
            }
        }
    }

    #[test]
    fn test_scroll_down_stops_at_last_item() {
        let mut view = create_test_view();
        let num_phases = view.phases.len();
        let git_count = GitCommand::all().len();
        let last_idx = num_phases + 3 + git_count - 1;

        // Set focus to Commands panel
        view.focus = WorktreeFocus::Commands;

        // Move to last item
        view.command_state.select(Some(last_idx));

        // Try to scroll down - should stay at last item
        view.scroll_down();
        assert_eq!(view.command_state.selected(), Some(last_idx));
    }

    #[test]
    fn test_scroll_up_stops_at_first_selectable_item() {
        let mut view = create_test_view();

        // Set focus to Commands panel
        view.focus = WorktreeFocus::Commands;

        // Start at first SDD phase (index 1)
        view.command_state.select(Some(1));

        // Try to scroll up - should stay at first selectable item
        view.scroll_up();
        assert_eq!(view.command_state.selected(), Some(1));
    }

    #[test]
    fn test_enter_on_specify_requests_input() {
        let mut view = create_test_view();

        // Set focus to Commands panel
        view.focus = WorktreeFocus::Commands;

        // Select Specify (display index 1 = command index 0)
        view.command_state.select(Some(1));

        // Press Enter
        let action = view.handle_key(key_event(KeyCode::Enter));

        // Should request input
        match action {
            ViewAction::RequestInput {
                prompt,
                placeholder,
            } => {
                assert!(prompt.contains("feature description"));
                assert!(placeholder.is_some());
            }
            _ => panic!("Expected RequestInput action for Specify phase"),
        }

        // Verify pending_input_phase is set
        assert_eq!(view.pending_input_phase, Some(SpecPhase::Specify));
    }

    #[test]
    fn test_enter_on_clarify_runs_phase() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Select Clarify (display index 2 = command index 1)
        view.command_state.select(Some(2));

        let action = view.handle_key(key_event(KeyCode::Enter));

        // Should run the phase
        match action {
            ViewAction::RunSpecPhase { phase, command, .. } => {
                assert_eq!(phase, "clarify");
                assert!(command.contains("clarify"));
            }
            _ => panic!("Expected RunSpecPhase action for Clarify phase"),
        }
    }

    #[test]
    fn test_enter_on_git_commit_requests_input() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Find and select Git Commit command
        let num_phases = view.phases.len();
        let commit_display_idx = num_phases + 3; // First git command (Commit)

        view.command_state.select(Some(commit_display_idx));

        let action = view.handle_key(key_event(KeyCode::Enter));

        // Should trigger enhanced commit workflow
        match action {
            ViewAction::RunEnhancedCommit => {
                // Expected behavior - triggers security scanning and commit workflow
            }
            _ => panic!("Expected RunEnhancedCommit action for Git Commit"),
        }
    }

    #[test]
    fn test_enter_on_git_push_runs_command() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Find and select Git Push command (second git command)
        let num_phases = view.phases.len();
        let push_display_idx = num_phases + 4;

        view.command_state.select(Some(push_display_idx));

        let action = view.handle_key(key_event(KeyCode::Enter));

        // Should run git push command
        match action {
            ViewAction::RunCommand { name, args } => {
                assert_eq!(name, "git");
                assert_eq!(args, vec!["push".to_string()]);
            }
            _ => panic!("Expected RunCommand action for Git Push"),
        }
    }

    #[test]
    fn test_enter_on_git_status_runs_command() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Find and select Git Status command (third git command)
        let num_phases = view.phases.len();
        let status_display_idx = num_phases + 5;

        view.command_state.select(Some(status_display_idx));

        let action = view.handle_key(key_event(KeyCode::Enter));

        // Should run git status command
        match action {
            ViewAction::RunCommand { name, args } => {
                assert_eq!(name, "git");
                assert_eq!(args, vec!["status".to_string()]);
            }
            _ => panic!("Expected RunCommand action for Git Status"),
        }
    }

    #[test]
    fn test_enter_on_git_add_all_runs_command() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Find and select Git Add All command (fourth git command)
        let num_phases = view.phases.len();
        let add_all_display_idx = num_phases + 6;

        view.command_state.select(Some(add_all_display_idx));

        let action = view.handle_key(key_event(KeyCode::Enter));

        // Should run git add --all command
        match action {
            ViewAction::RunCommand { name, args } => {
                assert_eq!(name, "git");
                assert_eq!(args, vec!["add".to_string(), "--all".to_string()]);
            }
            _ => panic!("Expected RunCommand action for Git Add All"),
        }
    }

    #[test]
    fn test_enter_on_git_rebase_requests_input() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Find and select Git Rebase command (fifth git command)
        let num_phases = view.phases.len();
        let rebase_display_idx = num_phases + 7;

        view.command_state.select(Some(rebase_display_idx));

        let action = view.handle_key(key_event(KeyCode::Enter));

        // Should request input for branch name
        match action {
            ViewAction::RequestInput {
                prompt,
                placeholder,
            } => {
                assert!(prompt.contains("Rebase onto branch"));
                assert_eq!(placeholder, Some("main".to_string()));
            }
            _ => panic!("Expected RequestInput action for Git Rebase"),
        }

        // Verify pending_git_command is set
        assert_eq!(view.pending_git_command, Some(GitCommand::Rebase));
    }

    #[test]
    fn test_enter_on_header_does_nothing() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Try to select header (index 0) - this shouldn't happen in practice
        // due to scroll methods, but test the safety
        view.command_state.select(Some(0));

        let action = view.handle_key(key_event(KeyCode::Enter));

        // Should return None since header is not selectable
        match action {
            ViewAction::None => {} // Expected
            _ => panic!("Expected None action for header"),
        }
    }

    #[test]
    fn test_focus_navigation() {
        let mut view = create_test_view();

        // Start with Commands focus (default)
        assert_eq!(view.focus, WorktreeFocus::Commands);

        // Move right to Content
        view.focus_right();
        assert_eq!(view.focus, WorktreeFocus::Content);

        // Move left back to Commands
        view.focus_left();
        assert_eq!(view.focus, WorktreeFocus::Commands);

        // Test toggle behavior (left and right both toggle)
        view.focus_left();
        assert_eq!(view.focus, WorktreeFocus::Content);

        view.focus_left();
        assert_eq!(view.focus, WorktreeFocus::Commands);
    }

    #[test]
    fn test_j_k_navigation() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Start at first SDD phase (index 1)
        view.command_state.select(Some(1));

        // Press 'j' to move down
        view.handle_key(key_event(KeyCode::Char('j')));
        assert_eq!(view.command_state.selected(), Some(2));

        // Press 'k' to move up
        view.handle_key(key_event(KeyCode::Char('k')));
        assert_eq!(view.command_state.selected(), Some(1));
    }

    #[test]
    fn test_commands_vector_matches_phases_and_git() {
        let view = create_test_view();

        // Commands should contain all SDD phases + all git commands
        let expected_count = view.phases.len() + GitCommand::all().len();
        assert_eq!(view.commands.len(), expected_count);

        // First N commands should be SDD phases
        for i in 0..view.phases.len() {
            match &view.commands[i] {
                Command::SddPhase(_, _) => {} // Expected
                _ => panic!("Command {} should be an SDD phase", i),
            }
        }

        // Remaining commands should be git commands
        for i in view.phases.len()..view.commands.len() {
            match &view.commands[i] {
                Command::GitAction(_) => {} // Expected
                _ => panic!("Command {} should be a git command", i),
            }
        }
    }
}
