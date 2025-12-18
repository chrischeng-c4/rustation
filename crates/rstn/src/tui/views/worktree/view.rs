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
use crate::tui::widgets::TextInput; // Feature 051: Multi-line edit mode (User Story 3)

// Import extracted types from sibling modules
use super::{
    Command, ContentType, FeatureInfo, GitCommand, InlineInput, ParsedTask, SpecifyState,
    TaskListState, WorktreeFocus,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use ratatui::Frame;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub commit_warnings: Vec<crate::SecurityWarning>,

    // Commit review state (Feature 050)
    pub commit_groups: Option<Vec<crate::CommitGroup>>,
    pub current_commit_index: usize,
    pub commit_message_input: String,
    pub commit_message_cursor: usize,
    pub commit_sensitive_files: Vec<String>,
    pub commit_validation_error: Option<String>,

    // Specify workflow state (Feature 051)
    pub specify_state: SpecifyState,

    // Prompt Claude workflow state
    pub prompt_input: Option<TextInput>,  // Multi-line prompt input
    pub prompt_edit_mode: bool,           // Track if in edit mode (i/Esc toggle)
    pub prompt_output: String,            // Accumulated streaming output

    // Layout rects for mouse click detection
    pub commands_pane_rect: Option<Rect>,
    pub content_pane_rect: Option<Rect>,
    pub output_pane_rect: Option<Rect>,

    // Inline input mode for Claude follow-up questions (replaces dialog)
    pub inline_input: Option<InlineInput>,
}

impl WorktreeView {
    const REFRESH_INTERVAL: u64 = 60; // 6 seconds at 100ms/tick

    pub fn new() -> Self {
        let mut phase_state = ListState::default();
        phase_state.select(Some(0));

        let mut command_state = ListState::default();
        command_state.select(Some(1)); // Start on "Prompt Claude" (after WORKFLOW header)

        let phases = SpecPhase::all()
            .iter()
            .map(|&p| (p, PhaseStatus::NotStarted))
            .collect::<Vec<_>>();

        // Build unified command list (Workflow + SDD phases + Git commands)
        let mut commands = Vec::new();
        // WORKFLOW section
        commands.push(Command::PromptClaude);
        // SDD section
        for (phase, status) in &phases {
            commands.push(Command::SddPhase(*phase, *status));
        }
        // GIT section
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
            // Commit review state initialization (Feature 050)
            commit_groups: None,
            current_commit_index: 0,
            commit_message_input: String::new(),
            commit_message_cursor: 0,
            commit_sensitive_files: Vec::new(),
            commit_validation_error: None,
            // Specify workflow state initialization (Feature 051)
            specify_state: SpecifyState::new(),
            // Prompt Claude workflow state initialization
            prompt_input: None,
            prompt_edit_mode: false,
            prompt_output: String::new(),
            // Mouse click detection
            commands_pane_rect: None,
            content_pane_rect: None,
            output_pane_rect: None,
            // Inline input (replaces dialog for Claude follow-ups)
            inline_input: None,
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
                SpecPhase::Checklist => {
                    // Checklist is optional - check if file exists
                    if spec_dir.join("checklist.md").exists() {
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
            ContentType::CommitReview => None, // Rendered separately via render_commit_review()
            ContentType::SpecifyInput | ContentType::SpecifyReview => None, // Feature 051: Rendered separately via render_specify_input()
            ContentType::PromptInput | ContentType::PromptRunning => None, // Rendered separately via render_prompt_input/running()
        }
    }

    /// Map display index to command index (accounting for headers and separators)
    fn display_index_to_command_index(&self, display_idx: usize) -> Option<usize> {
        // Display indices for three-section layout:
        // 0: "WORKFLOW" header (not selectable)
        // 1: "Prompt Claude" (command 0)
        // 2: separator (not selectable)
        // 3: "SDD" header (not selectable)
        // 4-10: SDD phases (commands 1-7)
        // 11: separator (not selectable)
        // 12: "GIT" header (not selectable)
        // 13+: Git commands (commands 8+)

        let num_sdd_phases = self.phases.len();

        if display_idx == 0 {
            // "WORKFLOW" header - not selectable
            None
        } else if display_idx == 1 {
            // "Prompt Claude" - command 0
            Some(0)
        } else if display_idx == 2 {
            // Separator - not selectable
            None
        } else if display_idx == 3 {
            // "SDD" header - not selectable
            None
        } else if display_idx <= 3 + num_sdd_phases {
            // SDD phases: display index 4-10 maps to commands 1-7
            Some(display_idx - 3)
        } else if display_idx == 4 + num_sdd_phases {
            // Separator - not selectable
            None
        } else if display_idx == 5 + num_sdd_phases {
            // "GIT" header - not selectable
            None
        } else {
            // Git commands: display index 13+ maps to commands 8+
            Some(display_idx - 6)
        }
    }

    /// Move focus left
    fn focus_left(&mut self) {
        self.focus = match self.focus {
            WorktreeFocus::Content => WorktreeFocus::Commands,
            WorktreeFocus::Commands => WorktreeFocus::Output,
            WorktreeFocus::Output => WorktreeFocus::Content,
        };
    }

    /// Move to previous pane (Shift+Tab: Commands → Output → Content → Commands)
    pub fn prev_pane(&mut self) {
        self.focus = match self.focus {
            WorktreeFocus::Commands => WorktreeFocus::Output,
            WorktreeFocus::Content => WorktreeFocus::Commands,
            WorktreeFocus::Output => WorktreeFocus::Content,
        };
    }

    /// Move focus right
    fn focus_right(&mut self) {
        self.focus = match self.focus {
            WorktreeFocus::Commands => WorktreeFocus::Content,
            WorktreeFocus::Content => WorktreeFocus::Output,
            WorktreeFocus::Output => WorktreeFocus::Commands,
        };
    }

    /// Move to next pane (Tab key: Commands → Content → Output → Commands)
    pub fn next_pane(&mut self) {
        self.focus = match self.focus {
            WorktreeFocus::Commands => WorktreeFocus::Content,
            WorktreeFocus::Content => WorktreeFocus::Output,
            WorktreeFocus::Output => WorktreeFocus::Commands,
        };
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
            WorktreeFocus::Output => {
                // Output scrolling is handled separately in scroll_output_down()
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
            WorktreeFocus::Output => {
                // Output scrolling is handled separately in scroll_output_up()
            }
        }
    }

    /// Switch content type (cycle through Spec -> Plan -> Tasks)
    fn switch_content(&mut self) {
        self.content_type = match self.content_type {
            ContentType::Spec => ContentType::Plan,
            ContentType::Plan => ContentType::Tasks,
            ContentType::Tasks => ContentType::Spec,
            ContentType::CommitReview => ContentType::CommitReview, // Don't allow switching during review
            ContentType::SpecifyInput | ContentType::SpecifyReview => self.content_type, // Feature 051: Don't allow switching during specify workflow
            ContentType::PromptInput | ContentType::PromptRunning => self.content_type, // Don't allow switching during Prompt Claude workflow
        };
        self.content_scroll = 0;
    }

    /// Scroll output panel down by one line
    fn scroll_output_down(&mut self) {
        let max_scroll = self.log_buffer.len().saturating_sub(1);
        self.output_scroll = (self.output_scroll + 1).min(max_scroll);
    }

    /// Scroll output panel up by one line
    fn scroll_output_up(&mut self) {
        self.output_scroll = self.output_scroll.saturating_sub(1);
    }

    /// Scroll output panel down by one page (10 lines)
    fn scroll_output_page_down(&mut self) {
        let max_scroll = self.log_buffer.len().saturating_sub(1);
        self.output_scroll = (self.output_scroll + 10).min(max_scroll);
    }

    /// Scroll output panel up by one page (10 lines)
    fn scroll_output_page_up(&mut self) {
        self.output_scroll = self.output_scroll.saturating_sub(10);
    }

    /// Scroll output panel to the bottom
    fn scroll_output_to_bottom(&mut self) {
        self.output_scroll = self.log_buffer.len().saturating_sub(1);
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

    // ─── Inline Input Methods (replaces dialog for Claude follow-ups) ───

    /// Set inline input mode with Claude's prompt
    pub fn set_inline_input(&mut self, prompt: String) {
        self.inline_input = Some(InlineInput::new(prompt));
        // Switch to content tab to show the input
        self.focus = WorktreeFocus::Content;
    }

    /// Submit inline input and return the value
    pub fn submit_inline_input(&mut self) -> Option<String> {
        self.inline_input.take().map(|input| input.text_input.value)
    }

    /// Cancel inline input mode
    pub fn cancel_inline_input(&mut self) {
        self.inline_input = None;
    }

    /// Check if inline input is active
    pub fn has_inline_input(&self) -> bool {
        self.inline_input.is_some()
    }

    /// Handle key event for inline input
    pub fn handle_inline_input_key(&mut self, key: KeyEvent) {
        if let Some(ref mut input) = self.inline_input {
            match key.code {
                KeyCode::Char(c) => input.text_input.insert_char(c),
                KeyCode::Backspace => input.text_input.delete_char(),
                KeyCode::Left => input.text_input.move_cursor_left(),
                KeyCode::Right => input.text_input.move_cursor_right(),
                KeyCode::Home => input.text_input.move_cursor_home(),
                KeyCode::End => input.text_input.move_cursor_end(),
                KeyCode::Up => input.text_input.move_cursor_up(),
                KeyCode::Down => input.text_input.move_cursor_down(),
                _ => {}
            }
        }
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
        self.log(LogCategory::Command, command.to_string());
        self.log(LogCategory::System, "─".repeat(60)); // Separator
    }

    /// Log file change
    pub fn log_file_change(&mut self, path: &Path) {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        self.log(
            LogCategory::FileChange,
            format!("File updated: {}", filename),
        );
    }

    /// Log shell command
    pub fn log_shell_command(&mut self, script: &str, exit_code: i32) {
        self.log(
            LogCategory::Hook,
            format!("{} completed (exit: {})", script, exit_code),
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
                ViewAction::RunIntelligentCommit
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
            WorktreeFocus::Output => {
                // Return output log entries with timestamps
                if self.log_buffer.is_empty() {
                    return String::new();
                }

                self.log_buffer
                    .entries()
                    .map(|entry| {
                        format!(
                            "[{}] {} {}",
                            entry.format_timestamp(),
                            entry.category_icon(),
                            entry.content
                        )
                    })
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

        // WORKFLOW section header
        items.push(ListItem::new(vec![Line::from(vec![Span::styled(
            "WORKFLOW",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])]));

        // Prompt Claude command
        items.push(ListItem::new(vec![Line::from(vec![
            Span::raw("✨ "),
            Span::styled(
                "Prompt Claude",
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            ),
        ])]));

        // Separator
        items.push(ListItem::new(Line::from("")));

        // SDD section header
        items.push(ListItem::new(vec![Line::from(vec![Span::styled(
            "SDD",
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

        // GIT section header
        items.push(ListItem::new(vec![Line::from(vec![Span::styled(
            "GIT",
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
                Constraint::Length(3), // Tab bar with border
                Constraint::Min(0),    // Content area
            ])
            .split(area);

        // Determine selected tab index
        let selected_idx = match self.content_type {
            ContentType::Spec => 0,
            ContentType::Plan => 1,
            ContentType::Tasks => 2,
            ContentType::CommitReview => 3,
            ContentType::SpecifyInput | ContentType::SpecifyReview => 0, // Feature 051: Highlight Spec tab during specify workflow
            ContentType::PromptInput | ContentType::PromptRunning => 0, // Highlight Spec tab during Prompt Claude workflow
        };

        // Render tab bar
        let tab_titles = vec!["Spec", "Plan", "Tasks", "Commit Review"];
        let tab_title = if let Some(ref info) = self.feature_info {
            format!(" Content - Feature #{} ", info.number)
        } else {
            " Content ".to_string()
        };

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(tab_title)
                    .border_style(if is_focused {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .select(selected_idx)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, sections[0]);

        // Render inline input if active (Claude follow-up questions)
        if self.inline_input.is_some() {
            self.render_inline_input(frame, sections[1]);
            return;
        }

        // Render content area - dispatch to commit review if in that mode (Feature 050)
        if self.content_type == ContentType::CommitReview {
            self.render_commit_review(frame, sections[1]);
            return;
        }

        // Render content area - dispatch to specify input if in that mode (Feature 051 - T021)
        if self.content_type == ContentType::SpecifyInput {
            self.render_specify_input(frame, sections[1]);
            return;
        }

        // Render content area - dispatch to specify review/edit if in that mode (Feature 051)
        if self.content_type == ContentType::SpecifyReview {
            // T067: Route to edit mode if active (User Story 3)
            if self.specify_state.edit_mode {
                self.render_specify_edit(frame, sections[1]);
                return;
            }
            // T035: Otherwise show review mode
            self.render_specify_review(frame, sections[1]);
            return;
        }

        // Render content area - dispatch to Prompt Claude input if in that mode (Task 1.6)
        if self.content_type == ContentType::PromptInput {
            self.render_prompt_input(frame, sections[1]);
            return;
        }

        // Standard content rendering for Spec/Plan/Tasks
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
                format!(" Log {} Running: {} ", spinner_char, phase)
            } else {
                format!(" Log {} Running... ", spinner_char)
            }
        } else {
            " Log ".to_string()
        };

        // Change border color when focused
        let is_focused = self.focus == WorktreeFocus::Output;
        let border_style = if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

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
                            Style::default().fg(Color::DarkGray),
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

    // ========== Commit Review Methods (Feature 050) ==========

    /// Start commit review workflow with analyzed commit groups
    pub fn start_commit_review(
        &mut self,
        groups: Vec<crate::CommitGroup>,
        warnings: Vec<String>,
        sensitive_files: Vec<String>,
    ) {
        #[cfg(debug_assertions)]
        assert!(!groups.is_empty(), "Commit groups cannot be empty");

        // Check file count warnings
        for (i, group) in groups.iter().enumerate() {
            if group.files.len() > 50 {
                let entry = LogEntry::new(
                    LogCategory::System,
                    format!(
                        "Group {} has {} files (>50). Consider splitting this commit.",
                        i + 1,
                        group.files.len()
                    ),
                );
                self.log_buffer.push(entry);
            }
        }

        // Initialize first group
        let first_message = groups[0].message.clone();

        self.commit_groups = Some(groups);
        self.current_commit_index = 0;
        self.commit_message_input = first_message.clone();
        self.commit_message_cursor = first_message.len(); // Cursor at end
        self.commit_sensitive_files = sensitive_files;
        self.commit_validation_error = None;
        self.content_type = ContentType::CommitReview;
        self.focus = WorktreeFocus::Content; // Auto-focus Content

        // Log warnings
        for warning in warnings {
            let entry = LogEntry::new(LogCategory::System, warning);
            self.log_buffer.push(entry);
        }

        let entry = LogEntry::new(
            LogCategory::System,
            format!(
                "Starting commit review workflow with {} groups",
                self.commit_groups.as_ref().unwrap().len()
            ),
        );
        self.log_buffer.push(entry);
    }

    /// Move to next commit group
    pub fn next_commit_group(&mut self) -> bool {
        if let Some(groups) = &self.commit_groups {
            if self.current_commit_index + 1 < groups.len() {
                self.current_commit_index += 1;
                self.load_current_group_message();
                return true;
            }
        }
        false // No more groups
    }

    /// Move to previous commit group
    pub fn previous_commit_group(&mut self) -> bool {
        if self.current_commit_index > 0 {
            self.current_commit_index -= 1;
            self.load_current_group_message();
            return true;
        }
        false // Already at first group
    }

    /// Cancel commit review workflow and return to normal view
    pub fn cancel_commit_review(&mut self) {
        let entry = LogEntry::new(
            LogCategory::System,
            "Commit review workflow cancelled".to_string(),
        );
        self.log_buffer.push(entry);

        self.commit_groups = None;
        self.current_commit_index = 0;
        self.commit_message_input.clear();
        self.commit_message_cursor = 0;
        self.commit_sensitive_files.clear();
        self.commit_validation_error = None;
        self.content_type = ContentType::Spec; // Return to Spec view
    }

    /// Get current commit message (with user edits)
    pub fn get_current_commit_message(&self) -> String {
        self.commit_message_input.clone()
    }

    // ========================================================================
    // Interactive phase workflow methods (Features 051, 053-058)
    // ========================================================================

    /// Enter interactive workflow for any SDD phase (Features 051, 053-058)
    ///
    /// Initiates the interactive workflow for a specific phase by:
    /// - Setting content type to `SpecifyInput`
    /// - Setting the current phase in specify_state
    /// - Switching focus to content area
    /// - Clearing any previous state
    ///
    /// # Parameters
    /// - `phase`: The SDD phase to run interactively
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rstn::tui::views::{WorktreeView, ContentType, SpecPhase};
    /// # let mut view = WorktreeView::new();
    /// view.start_interactive_phase(SpecPhase::Plan);
    /// assert_eq!(view.content_type, ContentType::SpecifyInput);
    /// assert_eq!(view.specify_state.current_phase, SpecPhase::Plan);
    /// ```
    pub fn start_interactive_phase(&mut self, phase: SpecPhase) {
        self.specify_state = SpecifyState::for_phase(phase);
        self.content_type = ContentType::SpecifyInput;
        self.focus = WorktreeFocus::Content; // Auto-focus Content area
    }

    /// Enter specify Input Phase (legacy wrapper for Feature 051)
    ///
    /// This is a convenience wrapper around `start_interactive_phase(SpecPhase::Specify)`.
    pub fn start_specify_input(&mut self) {
        self.start_interactive_phase(SpecPhase::Specify);
    }

    /// Start Prompt Claude input workflow
    ///
    /// Initializes the prompt input UI for direct Claude Code interaction.
    /// User can type a multi-line prompt, then submit with Ctrl+Enter.
    ///
    /// # Workflow
    /// 1. Show input textarea (read-only initially)
    /// 2. Press 'i' to enter edit mode
    /// 3. Type prompt (multi-line, Ctrl+Enter for newlines in edit mode)
    /// 4. Press Esc to exit edit mode
    /// 5. Press Ctrl+Enter to submit prompt
    pub fn start_prompt_input(&mut self) {
        // Initialize TextInput widget for multi-line prompt (20 lines max)
        self.prompt_input = Some(TextInput::new_multiline(String::new(), 20));
        self.prompt_edit_mode = false; // Start in view mode (press 'i' to edit)
        self.prompt_output.clear();
        self.content_type = ContentType::PromptInput;
        self.focus = WorktreeFocus::Content; // Auto-focus Content area
    }

    /// Handle key input for Prompt Claude workflow (Task 1.5)
    ///
    /// Vim-style modal editing:
    /// - View mode (default): 'i' to enter edit, 'Esc' to cancel
    /// - Edit mode: Full editing, 'Esc' to exit edit, 'Ctrl+Enter' to submit
    pub fn handle_prompt_input_key(&mut self, key: KeyEvent) -> ViewAction {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Get mutable reference to input widget
        let Some(input) = self.prompt_input.as_mut() else {
            return ViewAction::None;
        };

        // Handle keys based on edit mode
        if !self.prompt_edit_mode {
            // VIEW MODE: Only handle mode switching and cancel
            match key.code {
                KeyCode::Char('i') => {
                    // Enter edit mode
                    self.prompt_edit_mode = true;
                    input.active = true;
                    ViewAction::None
                }
                KeyCode::Esc => {
                    // Cancel prompt workflow
                    self.prompt_input = None;
                    self.prompt_edit_mode = false;
                    self.prompt_output.clear();
                    self.content_type = ContentType::Spec; // Return to default content
                    self.focus = WorktreeFocus::Commands; // Return focus to commands
                    ViewAction::None
                }
                KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Allow Ctrl+Enter submit in view mode too
                    self.submit_prompt_input()
                }
                _ => ViewAction::None,
            }
        } else {
            // EDIT MODE: Full editing capabilities
            match key.code {
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    input.insert_char(c);
                    ViewAction::None
                }
                KeyCode::Backspace => {
                    input.delete_char();
                    ViewAction::None
                }
                KeyCode::Delete => {
                    input.delete_char_forward();
                    ViewAction::None
                }
                KeyCode::Left => {
                    input.move_cursor_left();
                    ViewAction::None
                }
                KeyCode::Right => {
                    input.move_cursor_right();
                    ViewAction::None
                }
                KeyCode::Up => {
                    input.move_cursor_up();
                    ViewAction::None
                }
                KeyCode::Down => {
                    input.move_cursor_down();
                    ViewAction::None
                }
                KeyCode::Home => {
                    input.move_cursor_home();
                    ViewAction::None
                }
                KeyCode::End => {
                    input.move_cursor_end();
                    ViewAction::None
                }
                KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+Enter: Submit prompt
                    self.submit_prompt_input()
                }
                KeyCode::Enter => {
                    // Regular Enter: Insert newline
                    input.insert_newline();
                    ViewAction::None
                }
                KeyCode::Esc => {
                    // Exit edit mode (don't cancel, just return to view mode)
                    self.prompt_edit_mode = false;
                    input.active = false;
                    ViewAction::None
                }
                _ => ViewAction::None,
            }
        }
    }

    /// Submit the prompt and trigger Claude CLI execution (Task 1.5)
    ///
    /// Validates minimum length, then returns RunPromptClaude action
    fn submit_prompt_input(&mut self) -> ViewAction {
        let Some(input) = self.prompt_input.as_mut() else {
            return ViewAction::None;
        };

        // Get prompt text
        let prompt = input.get_multiline_value();

        // Validate minimum length (at least 3 characters)
        if prompt.trim().len() < 3 {
            // TODO: Show validation error in UI (for now, just ignore)
            return ViewAction::None;
        };

        // Clear input state
        self.prompt_input = None;
        self.prompt_edit_mode = false;

        // Return action to execute Claude CLI
        ViewAction::RunPromptClaude { prompt }
    }

    /// Render Prompt Claude input UI (Task 1.6)
    ///
    /// Layout:
    /// - Title and instructions (3 lines)
    /// - TextInput widget (multiline, fills most space)
    /// - Status and keybindings (4 lines)
    fn render_prompt_input(&self, frame: &mut Frame, area: Rect) {
        // Split area: Header (3 lines) + Input (fill) + Footer (4 lines)
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title + instructions
                Constraint::Min(5),    // Input area (minimum 5 lines)
                Constraint::Length(4), // Status + keybindings
            ])
            .split(area);

        // === HEADER: Title and instructions ===
        let title_lines = vec![
            Line::from(Span::styled(
                "Enter prompt for Claude",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                if self.prompt_edit_mode {
                    "Type your prompt below (multi-line):"
                } else {
                    "Press 'i' to enter edit mode:"
                },
                Style::default().fg(Color::Yellow),
            )),
        ];

        let title_paragraph = Paragraph::new(title_lines)
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(title_paragraph, sections[0]);

        // === INPUT: TextInput widget ===
        if let Some(ref input) = self.prompt_input {
            // Render TextInput widget in bordered box
            let input_block = Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Magenta));

            let input_inner = input_block.inner(sections[1]);
            frame.render_widget(input_block, sections[1]);
            frame.render_widget(input, input_inner);
        }

        // === FOOTER: Status and keybindings ===
        let edit_mode_status = if self.prompt_edit_mode {
            Span::styled("EDIT", Style::default().fg(Color::Green))
        } else {
            Span::styled("VIEW", Style::default().fg(Color::Yellow))
        };

        let footer_lines = vec![
            Line::from(vec![
                Span::raw("Edit mode: "),
                edit_mode_status,
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("[i]", Style::default().fg(Color::Green)),
                Span::raw(" Edit  "),
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(if self.prompt_edit_mode {
                    " Exit edit  "
                } else {
                    " Cancel     "
                }),
                Span::styled("[Ctrl+Enter]", Style::default().fg(Color::Cyan)),
                Span::raw(" Submit"),
            ]),
        ];

        let footer_paragraph = Paragraph::new(footer_lines)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(footer_paragraph, sections[2]);
    }

    /// Start implement mode to execute tasks (Feature 056)
    ///
    /// Loads existing tasks.md and enters task execution mode.
    /// Unlike other phases, this doesn't use the generate workflow.
    ///
    /// # Returns
    /// - `ViewAction::DisplayMessage` with error if tasks.md doesn't exist or is empty
    /// - `ViewAction::None` on success
    pub fn start_implement_mode(&mut self) -> ViewAction {
        // Get feature info
        let (spec_dir, number, name) = if let Some(ref info) = self.feature_info {
            (info.spec_dir.clone(), info.number.clone(), info.name.clone())
        } else {
            return ViewAction::DisplayMessage("No feature context".to_string());
        };

        // Check if tasks.md exists
        let tasks_path = spec_dir.join("tasks.md");
        if !tasks_path.exists() {
            return ViewAction::DisplayMessage("No tasks.md found. Run Tasks phase first.".to_string());
        }

        // Read and parse tasks
        match std::fs::read_to_string(&tasks_path) {
            Ok(content) => {
                self.specify_state = SpecifyState::for_phase(SpecPhase::Implement);
                self.specify_state.load_tasks_from_file(&content, number, name);

                if self.specify_state.task_list_state.is_none() {
                    return ViewAction::DisplayMessage("No tasks found in tasks.md".to_string());
                }

                // Enable list mode for navigation
                if let Some(ref mut tl) = self.specify_state.task_list_state {
                    tl.list_mode = true;
                }

                self.content_type = ContentType::SpecifyReview;
                self.focus = WorktreeFocus::Content;

                let entry = LogEntry::new(
                    LogCategory::System,
                    format!("Implement mode started with {} tasks",
                        self.specify_state.task_list_state.as_ref().map(|t| t.len()).unwrap_or(0)),
                );
                self.log_buffer.push(entry);

                ViewAction::None
            }
            Err(e) => ViewAction::DisplayMessage(format!("Failed to read tasks.md: {}", e)),
        }
    }

    /// Save tasks to tasks.md file (Feature 056)
    ///
    /// Persists the current task list state to the tasks.md file.
    pub fn save_tasks_to_file(&mut self) -> ViewAction {
        // Get file path
        let tasks_path = if let Some(ref info) = self.feature_info {
            info.spec_dir.join("tasks.md")
        } else {
            return ViewAction::DisplayMessage("No feature context".to_string());
        };

        // Get task content
        let content = if let Some(ref task_list) = self.specify_state.task_list_state {
            task_list.to_markdown()
        } else {
            return ViewAction::DisplayMessage("No tasks to save".to_string());
        };

        // Write to file
        match std::fs::write(&tasks_path, &content) {
            Ok(()) => {
                let entry = LogEntry::new(
                    LogCategory::System,
                    format!("Tasks saved to {}", tasks_path.display()),
                );
                self.log_buffer.push(entry);
                ViewAction::None
            }
            Err(e) => ViewAction::DisplayMessage(format!("Failed to save tasks: {}", e)),
        }
    }

    /// Execute the currently selected task (Feature 056)
    ///
    /// Returns a ViewAction::ExecuteTask with the task details.
    pub fn execute_selected_task(&mut self) -> ViewAction {
        // Get task info
        let (task_id, task_description) = if let Some(task) = self.specify_state.get_selected_task() {
            (task.id.clone(), task.description.clone())
        } else {
            return ViewAction::DisplayMessage("No task selected".to_string());
        };

        // Get feature info
        let (number, name) = if let (Some(num), Some(n)) = (
            &self.specify_state.feature_number,
            &self.specify_state.feature_name,
        ) {
            (num.clone(), n.clone())
        } else {
            return ViewAction::DisplayMessage("No feature context".to_string());
        };

        // Log execution start
        let entry = LogEntry::new(
            LogCategory::System,
            format!("Executing task {}: {}", task_id, task_description),
        );
        self.log_buffer.push(entry);

        // Set executing state
        if let Some(ref task_list) = self.specify_state.task_list_state {
            self.specify_state.executing_task_index = Some(task_list.selected);
        }

        ViewAction::ExecuteTask {
            task_id,
            task_description,
            feature_number: number,
            feature_name: name,
        }
    }

    /// Mark task as complete after execution (Feature 056)
    ///
    /// Called when task execution completes successfully.
    pub fn complete_task_execution(&mut self) {
        // Mark task complete
        if let Some(idx) = self.specify_state.executing_task_index {
            if let Some(ref mut task_list) = self.specify_state.task_list_state {
                if let Some(task) = task_list.tasks.get_mut(idx) {
                    task.completed = true;
                }
            }
        }

        // Clear executing state
        self.specify_state.executing_task_index = None;

        // Auto-advance if enabled
        if self.specify_state.auto_advance {
            self.specify_state.advance_to_next_incomplete();
        }

        // Auto-save
        let _ = self.save_tasks_to_file();

        // Log completion
        let (completed, total) = self.specify_state.get_task_progress();
        let entry = LogEntry::new(
            LogCategory::System,
            format!("Task complete. Progress: {}/{}", completed, total),
        );
        self.log_buffer.push(entry);
    }

    /// Mark a task complete by its ID via MCP tool (Feature 063)
    ///
    /// Called when Claude Code calls rstn_complete_task tool.
    pub fn complete_task_by_id(&mut self, task_id: &str) -> Result<(usize, usize), String> {
        // Mark task complete
        self.specify_state.complete_task_by_id(task_id)?;

        // Save to file
        match self.save_tasks_to_file() {
            ViewAction::None => {
                // Success - get progress and return
                let (completed, total) = self.specify_state.get_task_progress();

                // Log completion
                let entry = LogEntry::new(
                    LogCategory::System,
                    format!("Task {} complete via MCP. Progress: {}/{}", task_id, completed, total),
                );
                self.log_buffer.push(entry);

                Ok((completed, total))
            }
            ViewAction::DisplayMessage(msg) => Err(msg),
            _ => Err("Unexpected result from save_tasks_to_file".to_string()),
        }
    }

    /// Cancel specify workflow and return to normal Spec view (T016)
    ///
    /// Resets all specify state to defaults and returns to regular content view.
    /// Safe to call from any specify phase (Input, Review, Edit).
    pub fn cancel_specify(&mut self) {
        let entry = LogEntry::new(
            LogCategory::System,
            "Specify workflow cancelled".to_string(),
        );
        self.log_buffer.push(entry);

        self.specify_state.clear();
        self.content_type = ContentType::Spec; // Return to Spec view
        self.focus = WorktreeFocus::Commands; // Return focus to commands
    }

    /// Check if currently in SpecifyInput mode that needs input isolation
    pub fn is_in_specify_input_mode(&self) -> bool {
        self.content_type == ContentType::SpecifyInput
            && self.focus == WorktreeFocus::Content
            && !self.specify_state.is_generating
    }

    /// Handle keyboard input during Input Phase (T017)
    ///
    /// # Parameters
    /// - `key`: The key event to process
    ///
    /// # Returns
    /// - `ViewAction::None` for navigation keys (handled by parent)
    /// - `ViewAction::GenerateSpec` when user submits valid input
    pub fn handle_specify_input(&mut self, key: KeyEvent) -> ViewAction {
        use crossterm::event::{KeyCode, KeyModifiers};

        match key.code {
            KeyCode::Char(c) => {
                // Insert character at cursor position
                self.specify_state.input_buffer.insert(self.specify_state.input_cursor, c);
                self.specify_state.input_cursor += 1;
                self.specify_state.validation_error = None; // Clear error on input
                ViewAction::None
            }
            KeyCode::Backspace => {
                if self.specify_state.input_cursor > 0 {
                    self.specify_state.input_cursor -= 1;
                    self.specify_state.input_buffer.remove(self.specify_state.input_cursor);
                    self.specify_state.validation_error = None;
                }
                ViewAction::None
            }
            KeyCode::Delete => {
                if self.specify_state.input_cursor < self.specify_state.input_buffer.len() {
                    self.specify_state.input_buffer.remove(self.specify_state.input_cursor);
                }
                ViewAction::None
            }
            KeyCode::Left => {
                if self.specify_state.input_cursor > 0 {
                    self.specify_state.input_cursor -= 1;
                }
                ViewAction::None
            }
            KeyCode::Right => {
                if self.specify_state.input_cursor < self.specify_state.input_buffer.len() {
                    self.specify_state.input_cursor += 1;
                }
                ViewAction::None
            }
            KeyCode::Home => {
                self.specify_state.input_cursor = 0;
                ViewAction::None
            }
            KeyCode::End => {
                self.specify_state.input_cursor = self.specify_state.input_buffer.len();
                ViewAction::None
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+Enter: Insert newline
                self.specify_state.input_buffer.insert(self.specify_state.input_cursor, '\n');
                self.specify_state.input_cursor += 1;
                self.specify_state.validation_error = None;
                ViewAction::None
            }
            KeyCode::Enter => {
                // Submit description
                self.submit_specify_description()
            }
            KeyCode::Esc => {
                // Cancel specify workflow
                self.cancel_specify();
                ViewAction::None
            }
            _ => ViewAction::None,
        }
    }

    /// Validate input and trigger spec generation (T019)
    ///
    /// Validates that the description is at least 10 characters, then:
    /// - Sets `is_generating = true`
    /// - Returns `GenerateSpec` action with description
    ///
    /// # Returns
    /// - `ViewAction::GenerateSpec` if valid
    /// - `ViewAction::None` if invalid (sets `validation_error`)
    pub fn submit_specify_description(&mut self) -> ViewAction {
        // Validate input (T018)
        if let Err(error) = self.specify_state.validate_input() {
            self.specify_state.validation_error = Some(error);
            return ViewAction::None;
        }

        // Clear validation error
        self.specify_state.validation_error = None;

        // Mark as generating
        self.specify_state.is_generating = true;

        // Return action to trigger async generation
        ViewAction::GenerateSpec {
            phase: self.specify_state.current_phase,
            description: self.specify_state.input_buffer.clone(),
        }
    }

    /// Trigger save workflow for generated or edited spec (T036)
    ///
    /// Returns a `SaveSpec` action that will be handled by app.rs to write the
    /// spec to `specs/{number}-{name}/spec.md`.
    ///
    /// # Returns
    /// - `ViewAction::SaveSpec` with content, number, and name
    /// - `ViewAction::None` if spec/metadata is missing (sets `validation_error`)
    pub fn save_specify_spec(&mut self) -> ViewAction {
        // Feature 055: Use task list content if in task list mode
        let content = self.specify_state.get_current_content();

        // Defensive checks
        if let (Some(content), Some(number), Some(name)) = (
            content,
            &self.specify_state.feature_number,
            &self.specify_state.feature_name,
        ) {
            ViewAction::SaveSpec {
                phase: self.specify_state.current_phase,
                content,
                number: number.clone(),
                name: name.clone(),
            }
        } else {
            // Should never happen, but handle gracefully
            self.specify_state.validation_error =
                Some("Invalid state: missing spec content or feature info".to_string());
            ViewAction::None
        }
    }

    /// Enter Edit Phase from Review Phase (T052, User Story 3)
    ///
    /// Creates a multi-line TextInput widget pre-populated with the generated spec.
    /// The user can then edit the spec before saving.
    ///
    /// # Behavior
    /// - Only activates if `generated_spec` exists
    /// - Initializes TextInput with spec content split into lines
    /// - Sets cursor to (0, 0) at start of spec
    /// - Sets `edit_mode = true`
    pub fn toggle_specify_edit_mode(&mut self) {
        // Only allow entering edit mode if we have a generated spec
        if let Some(spec_content) = &self.specify_state.generated_spec {
            // Create multi-line text input with reasonable height
            let mut input = TextInput::new_multiline(String::new(), 25);

            // Load spec content into input lines
            input.lines = spec_content.lines().map(|s| s.to_string()).collect();

            // Position cursor at the start
            input.cursor_line = 0;
            input.cursor_column = 0;

            // Store the input widget and enable edit mode
            self.specify_state.edit_text_input = Some(input);
            self.specify_state.edit_mode = true;
        }
    }

    /// Handle keyboard input during Edit Phase (T054-T067, User Story 3)
    ///
    /// Routes all keypresses to the TextInput widget or Edit Phase controls:
    /// - `Ctrl+S`: Save edited spec
    /// - `Esc`: Cancel editing, return to Review
    /// - `Enter`: Insert newline (NOT save)
    /// - Arrow keys, Home, End: Navigate cursor
    /// - Backspace, Delete: Delete characters
    /// - Regular chars: Insert at cursor
    ///
    /// # Parameters
    /// - `key`: The key event to process
    ///
    /// # Returns
    /// - `ViewAction::SaveSpec` if user pressed Ctrl+S
    /// - `ViewAction::None` otherwise
    pub fn handle_specify_edit_input(&mut self, key: KeyEvent) -> ViewAction {
        // Get mutable reference to the text input
        if let Some(input) = &mut self.specify_state.edit_text_input {
            match key.code {
                // Ctrl+S - Save edited spec
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return self.save_from_edit();
                }

                // Enter - Insert newline (NOT save - prevents accidental saves)
                KeyCode::Enter => {
                    input.insert_newline();
                }

                // Backspace - Delete character before cursor
                KeyCode::Backspace => {
                    input.delete_char();
                }

                // Delete - Delete character after cursor
                KeyCode::Delete => {
                    input.delete_char_forward();
                }

                // Arrow keys - Navigate cursor
                KeyCode::Up => {
                    input.move_cursor_up();
                }
                KeyCode::Down => {
                    input.move_cursor_down();
                }
                KeyCode::Left => {
                    input.move_cursor_left();
                }
                KeyCode::Right => {
                    input.move_cursor_right();
                }

                // Home - Move to line start
                KeyCode::Home => {
                    input.move_cursor_home();
                }

                // End - Move to line end
                KeyCode::End => {
                    input.move_cursor_end();
                }

                // Esc - Cancel editing and return to review mode
                KeyCode::Esc => {
                    self.cancel_edit();
                }

                // Regular character input
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    input.insert_char(c);
                }

                // Ignore other keys
                _ => {}
            }
        }
        ViewAction::None
    }

    /// Extract edited content and trigger save workflow (T065, User Story 3)
    ///
    /// Called when user presses Ctrl+S in Edit Phase. Extracts the multi-line
    /// content from TextInput, updates `generated_spec`, exits Edit Phase, and
    /// triggers the save workflow.
    ///
    /// # Returns
    /// - `ViewAction::SaveSpec` with edited content
    /// - `ViewAction::None` if no TextInput exists (shouldn't happen)
    pub fn save_from_edit(&mut self) -> ViewAction {
        // Extract edited content from TextInput
        if let Some(input) = &self.specify_state.edit_text_input {
            let edited_content = input.get_multiline_value();

            // Update the generated spec with edited content
            self.specify_state.generated_spec = Some(edited_content);

            // Clear edit mode and return to review
            self.specify_state.edit_mode = false;
            self.specify_state.edit_text_input = None;

            // Trigger save workflow (will write to file)
            return self.save_specify_spec();
        }

        ViewAction::None
    }

    /// Discard edits and return to Review Phase (T066, User Story 3)
    ///
    /// Called when user presses Esc in Edit Phase. Discards any changes made in
    /// the editor and returns to Review Phase with the original spec unchanged.
    pub fn cancel_edit(&mut self) {
        // Clear edit mode state and return to review
        self.specify_state.edit_mode = false;
        self.specify_state.edit_text_input = None;
        // Original spec content remains in generated_spec, unchanged
    }

    /// Handle keyboard input during Review Phase (T037)
    ///
    /// # Keybindings
    /// - `Enter`: Save spec to file (or execute task in Implement mode)
    /// - `e`: Enter Edit Phase for inline editing
    /// - `x`: Toggle task completion (Implement mode only)
    /// - `Esc`: Cancel workflow and discard spec
    /// - Other keys: No action (scrolling handled by parent)
    ///
    /// # Parameters
    /// - `key`: The key event to process
    ///
    /// # Returns
    /// - `ViewAction::SaveSpec` if user pressed Enter (non-implement mode)
    /// - `ViewAction::ExecuteTask` if user pressed Enter in implement mode
    /// - `ViewAction::None` otherwise
    pub fn handle_specify_review_input(&mut self, key: KeyEvent) -> ViewAction {
        // Feature 056: Implement mode key handling
        if self.specify_state.is_implement_mode() {
            match (key.code, key.modifiers) {
                // j - Select next task
                (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                    if let Some(ref mut task_list) = self.specify_state.task_list_state {
                        task_list.select_next();
                    }
                    return ViewAction::None;
                }
                // k - Select previous task
                (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                    if let Some(ref mut task_list) = self.specify_state.task_list_state {
                        task_list.select_previous();
                    }
                    return ViewAction::None;
                }
                // x - Toggle task completion
                (KeyCode::Char('x'), KeyModifiers::NONE) => {
                    self.specify_state.toggle_task_completion();
                    // Auto-save after toggling completion
                    return self.save_tasks_to_file();
                }
                // Enter - Execute selected task
                (KeyCode::Enter, _) => {
                    return self.execute_selected_task();
                }
                // Esc - Exit implement mode
                (KeyCode::Esc, _) => {
                    self.cancel_specify();
                    return ViewAction::None;
                }
                // a - Toggle auto-advance
                (KeyCode::Char('a'), KeyModifiers::NONE) => {
                    self.specify_state.auto_advance = !self.specify_state.auto_advance;
                    let entry = LogEntry::new(
                        LogCategory::System,
                        format!("Auto-advance: {}", if self.specify_state.auto_advance { "ON" } else { "OFF" }),
                    );
                    self.log_buffer.push(entry);
                    return ViewAction::None;
                }
                _ => return ViewAction::None,
            }
        }

        // Feature 055: Task list mode key handling (for Tasks phase generation)
        if self.specify_state.is_task_list_mode() {
            match (key.code, key.modifiers) {
                // Shift+J - Move task down in list
                (KeyCode::Char('J'), _) | (KeyCode::Char('j'), KeyModifiers::SHIFT) => {
                    if let Some(ref mut task_list) = self.specify_state.task_list_state {
                        task_list.reorder_down();
                    }
                    return ViewAction::None;
                }
                // Shift+K - Move task up in list
                (KeyCode::Char('K'), _) | (KeyCode::Char('k'), KeyModifiers::SHIFT) => {
                    if let Some(ref mut task_list) = self.specify_state.task_list_state {
                        task_list.reorder_up();
                    }
                    return ViewAction::None;
                }
                // j - Select next task
                (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                    if let Some(ref mut task_list) = self.specify_state.task_list_state {
                        task_list.select_next();
                    }
                    return ViewAction::None;
                }
                // k - Select previous task
                (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                    if let Some(ref mut task_list) = self.specify_state.task_list_state {
                        task_list.select_previous();
                    }
                    return ViewAction::None;
                }
                // t - Toggle between list mode and raw text mode
                (KeyCode::Char('t'), KeyModifiers::NONE) => {
                    if let Some(ref mut task_list) = self.specify_state.task_list_state {
                        task_list.list_mode = !task_list.list_mode;
                    }
                    return ViewAction::None;
                }
                _ => {}
            }
        }

        match key.code {
            // Enter - Save spec
            KeyCode::Enter => self.save_specify_spec(),

            // 'e' - Edit mode (User Story 3, T052)
            KeyCode::Char('e') => {
                self.toggle_specify_edit_mode();
                ViewAction::None
            }

            // Esc - Cancel review and return to normal view
            KeyCode::Esc => {
                self.cancel_specify();
                ViewAction::None
            }

            // Other keys - no action (scrolling handled by main handler)
            _ => ViewAction::None,
        }
    }

    /// Validate commit message before submission
    pub fn validate_commit_message(&mut self) -> bool {
        let trimmed = self.commit_message_input.trim();
        if trimmed.is_empty() {
            self.commit_validation_error = Some("Commit message cannot be empty".to_string());
            return false;
        }
        self.commit_validation_error = None;
        true
    }

    /// Load message from current group into input (private helper)
    fn load_current_group_message(&mut self) {
        if let Some(groups) = &self.commit_groups {
            let message = groups[self.current_commit_index].message.clone();
            self.commit_message_input = message.clone();
            self.commit_message_cursor = message.len();
            self.commit_validation_error = None;
        }
    }

    /// Copy current commit review content to clipboard (Feature 050, T046)
    fn copy_commit_review(&mut self) {
        if let Some(groups) = &self.commit_groups {
            if let Some(group) = groups.get(self.current_commit_index) {
                let total_groups = groups.len();
                let current_group = self.current_commit_index + 1;

                // Format content for clipboard (T046)
                let mut content = format!(
                    "Commit Group {}/{}\n\nMessage:\n{}\n\nFiles:\n",
                    current_group, total_groups, self.commit_message_input
                );

                for file in &group.files {
                    content.push_str(&format!("  - {}\n", file));
                }

                // Add warnings if present
                if !self.commit_sensitive_files.is_empty() {
                    content.push_str("\nWarnings:\n");
                    for sensitive_file in &self.commit_sensitive_files {
                        content.push_str(&format!("  ⚠ Sensitive file: {}\n", sensitive_file));
                    }
                }

                // Copy to clipboard using arboard (T046, T048)
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(content) {
                        Ok(_) => {
                            let entry = LogEntry::new(
                                LogCategory::System,
                                format!(
                                    "Copied commit group {}/{} to clipboard",
                                    current_group, total_groups
                                ),
                            );
                            self.log_buffer.push(entry);
                        }
                        Err(e) => {
                            // T048: Handle clipboard errors gracefully
                            let entry = LogEntry::new(
                                LogCategory::System,
                                format!("Failed to copy to clipboard: {}", e),
                            );
                            self.log_buffer.push(entry);
                        }
                    },
                    Err(e) => {
                        // T048: Handle clipboard initialization errors
                        let entry = LogEntry::new(
                            LogCategory::System,
                            format!("Failed to initialize clipboard: {}", e),
                        );
                        self.log_buffer.push(entry);
                    }
                }
            }
        }
    }

    /// Handle keyboard input during commit review mode
    /// Returns ViewAction if action needed, None if handled internally
    pub fn handle_commit_review_input(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            // Character input - insert at cursor (T018) or navigation (T023, T024)
            KeyCode::Char(c) => {
                // Special keys for navigation and clipboard
                match c {
                    // 'n' - next group (T023)
                    'n' => {
                        self.next_commit_group();
                        ViewAction::None
                    }
                    // 'p' - previous group (T024)
                    'p' => {
                        self.previous_commit_group();
                        ViewAction::None
                    }
                    // 'y' - copy to clipboard (T047)
                    'y' => {
                        self.copy_commit_review();
                        ViewAction::None
                    }
                    // All other characters - insert at cursor
                    _ => {
                        // Clear validation error when user edits message
                        self.commit_validation_error = None;

                        // Insert character at cursor position
                        if self.commit_message_cursor <= self.commit_message_input.len()
                            && self
                                .commit_message_input
                                .is_char_boundary(self.commit_message_cursor)
                        {
                            self.commit_message_input
                                .insert(self.commit_message_cursor, c);
                            self.commit_message_cursor += c.len_utf8();
                        }
                        ViewAction::None
                    }
                }
            }

            // Backspace - delete character before cursor (T019)
            KeyCode::Backspace => {
                if self.commit_message_cursor > 0 {
                    // Find the previous character boundary
                    let mut new_cursor = self.commit_message_cursor - 1;
                    while !self.commit_message_input.is_char_boundary(new_cursor) && new_cursor > 0
                    {
                        new_cursor -= 1;
                    }

                    self.commit_message_input.remove(new_cursor);
                    self.commit_message_cursor = new_cursor;
                    self.commit_validation_error = None;
                }
                ViewAction::None
            }

            // Delete - delete character after cursor (T020)
            KeyCode::Delete => {
                if self.commit_message_cursor < self.commit_message_input.len() {
                    // Remove character at cursor position
                    if self
                        .commit_message_input
                        .is_char_boundary(self.commit_message_cursor)
                    {
                        self.commit_message_input.remove(self.commit_message_cursor);
                        self.commit_validation_error = None;
                    }
                }
                ViewAction::None
            }

            // Arrow keys - cursor movement (T021)
            KeyCode::Left => {
                if self.commit_message_cursor > 0 {
                    // Move cursor left one character
                    let mut new_cursor = self.commit_message_cursor - 1;
                    while !self.commit_message_input.is_char_boundary(new_cursor) && new_cursor > 0
                    {
                        new_cursor -= 1;
                    }
                    self.commit_message_cursor = new_cursor;
                }
                ViewAction::None
            }
            KeyCode::Right => {
                if self.commit_message_cursor < self.commit_message_input.len() {
                    // Move cursor right one character
                    let mut new_cursor = self.commit_message_cursor + 1;
                    while !self.commit_message_input.is_char_boundary(new_cursor)
                        && new_cursor < self.commit_message_input.len()
                    {
                        new_cursor += 1;
                    }
                    self.commit_message_cursor = new_cursor;
                }
                ViewAction::None
            }

            // Home/End - cursor to start/end (T022)
            KeyCode::Home => {
                self.commit_message_cursor = 0;
                ViewAction::None
            }
            KeyCode::End => {
                self.commit_message_cursor = self.commit_message_input.len();
                ViewAction::None
            }

            // Enter - validate and submit (T025)
            KeyCode::Enter => {
                if self.validate_commit_message() {
                    ViewAction::SubmitCommitGroup
                } else {
                    // Validation error is already set, will be displayed
                    ViewAction::None
                }
            }

            // Esc - cancel workflow (T026)
            KeyCode::Esc => {
                self.cancel_commit_review();
                ViewAction::None
            }

            // Other keys - no action
            _ => ViewAction::None,
        }
    }

    /// Render commit review UI in Content pane
    fn render_commit_review(&self, frame: &mut Frame, area: Rect) {
        if let Some(groups) = &self.commit_groups {
            let group = &groups[self.current_commit_index];
            let total_groups = groups.len();
            let current_group = self.current_commit_index + 1;

            // Build content lines
            let mut lines = vec![];

            // Group number header
            lines.push(Line::from(Span::styled(
                format!("Commit Group {}/{}", current_group, total_groups),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            // Message section
            lines.push(Line::from(Span::styled(
                "Message:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!("  {}", self.commit_message_input)));

            // Validation error if present
            if let Some(ref error) = self.commit_validation_error {
                lines.push(Line::from(Span::styled(
                    format!("  ⚠ {}", error),
                    Style::default().fg(Color::Red),
                )));
            }
            lines.push(Line::from(""));

            // Files section
            lines.push(Line::from(Span::styled(
                "Files:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            for file in &group.files {
                lines.push(Line::from(format!("  - {}", file)));
            }
            lines.push(Line::from(""));

            // Warnings section
            if !self.commit_sensitive_files.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Warnings:",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
                for sensitive_file in &self.commit_sensitive_files {
                    lines.push(Line::from(Span::styled(
                        format!("  ⚠ Sensitive file: {}", sensitive_file),
                        Style::default().fg(Color::Red),
                    )));
                }
                lines.push(Line::from(""));
            }

            // Navigation controls
            lines.push(Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Green)),
                Span::raw(" Submit  "),
                Span::styled("[n]", Style::default().fg(Color::Cyan)),
                Span::raw(" Next  "),
                Span::styled("[p]", Style::default().fg(Color::Cyan)),
                Span::raw(" Previous  "),
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" Cancel"),
            ]));

            // Render as paragraph
            let paragraph = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Commit Review")
                        .border_style(Style::default().fg(Color::Green)),
                )
                .wrap(Wrap { trim: false });

            frame.render_widget(paragraph, area);
        }
    }

    /// Render specify Input Phase dialog (T020)
    /// Render inline input for Claude follow-up questions
    fn render_inline_input(&self, frame: &mut Frame, area: Rect) {
        let inline = match &self.inline_input {
            Some(input) => input,
            None => return,
        };

        let mut lines = vec![];

        // Title
        lines.push(Line::from(Span::styled(
            "Claude Input",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Claude's prompt/question (split by newlines for markdown-like display)
        for line in inline.prompt.lines() {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::Yellow),
            )));
        }
        lines.push(Line::from(""));

        // Input area header
        lines.push(Line::from(Span::styled(
            "Your Response:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));

        // Input buffer with cursor indicator
        let input_value = &inline.text_input.value;
        let cursor_pos = inline.text_input.cursor_position;
        let input_display = if input_value.is_empty() {
            Line::from(Span::styled(
                "Type your response here..._",
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            // Show cursor position
            let (before, after) = input_value.split_at(cursor_pos.min(input_value.len()));
            Line::from(vec![
                Span::raw(before),
                Span::styled("█", Style::default().fg(Color::White)),
                Span::raw(after),
            ])
        };
        lines.push(input_display);

        lines.push(Line::from(""));

        // Action hints
        lines.push(Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Submit  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]));

        // Render as paragraph
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Claude Input ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn render_specify_input(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![];

        // Title
        lines.push(Line::from(Span::styled(
            "Specify Feature",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Instructions
        lines.push(Line::from(Span::styled(
            "Enter feature description:",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(""));

        // Input buffer with cursor
        let input_display = if self.specify_state.input_buffer.is_empty() {
            Span::styled(
                "Type your description here...",
                Style::default().fg(Color::DarkGray),
            )
        } else {
            Span::raw(&self.specify_state.input_buffer)
        };
        lines.push(Line::from(input_display));

        // Validation error (T030)
        if let Some(ref error) = self.specify_state.validation_error {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  ⚠ {}", error),
                Style::default().fg(Color::Red),
            )));
        }

        lines.push(Line::from(""));

        // Action hints
        lines.push(Line::from(vec![
            Span::styled("[Ctrl+Enter]", Style::default().fg(Color::Green)),
            Span::raw(" New line  "),
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Submit  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]));

        // Render as paragraph
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Specify Input")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);

        // Calculate cursor position for visible input cursor
        // Get inner area (excluding borders)
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);

        // Layout inside block:
        // Line 0: Title "Specify Feature"
        // Line 1: Empty
        // Line 2: "Enter feature description:"
        // Line 3: Empty
        // Line 4: Input buffer <- cursor should be here
        let input_line_offset = 4;
        let cursor_x = inner.x + self.specify_state.input_cursor as u16;
        let cursor_y = inner.y + input_line_offset;

        // Only set cursor if within bounds
        if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    /// Render specify Review Phase with spec preview (T034, T038, T039)
    /// Feature 055: Enhanced with structured task list rendering
    fn render_specify_review(&self, frame: &mut Frame, area: Rect) {
        // Build ALL lines with styling
        let mut all_lines: Vec<Line> = vec![];

        // Header: Feature number and name (T038)
        if let (Some(number), Some(name)) = (
            &self.specify_state.feature_number,
            &self.specify_state.feature_name,
        ) {
            all_lines.push(Line::from(Span::styled(
                format!("Feature {} - {}", number, name),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            all_lines.push(Line::from(""));
        }

        // Feature 056: Check if in implement mode
        let is_implement_mode = self.specify_state.is_implement_mode();

        // Feature 055/056: Check if in task list mode or implement mode
        if self.specify_state.is_task_list_mode() || is_implement_mode {
            // Render header based on mode
            if is_implement_mode {
                // Implement mode header with progress
                let (completed, total) = self.specify_state.get_task_progress();
                all_lines.push(Line::from(vec![
                    Span::styled(
                        "Implementation Mode",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  [{}/{}]", completed, total),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
            } else {
                // Task generation mode header
                all_lines.push(Line::from(Span::styled(
                    "Generated Tasks (List Mode):",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            all_lines.push(Line::from(""));

            if let Some(ref task_list) = self.specify_state.task_list_state {
                for (idx, task) in task_list.tasks.iter().enumerate() {
                    let is_selected = idx == task_list.selected;

                    // Build task line with markers
                    let mut spans = vec![];

                    // Selection indicator
                    if is_selected {
                        spans.push(Span::styled("▶ ", Style::default().fg(Color::Green)));
                    } else {
                        spans.push(Span::raw("  "));
                    }

                    // Checkbox
                    let checkbox = if task.completed { "[X]" } else { "[ ]" };
                    spans.push(Span::styled(
                        checkbox,
                        Style::default().fg(if task.completed { Color::Green } else { Color::Gray }),
                    ));
                    spans.push(Span::raw(" "));

                    // Task ID
                    spans.push(Span::styled(
                        &task.id,
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::raw(" "));

                    // Parallel marker
                    if task.is_parallel {
                        spans.push(Span::styled("[P]", Style::default().fg(Color::Magenta)));
                        spans.push(Span::raw(" "));
                    }

                    // User story marker
                    if let Some(ref us) = task.user_story {
                        spans.push(Span::styled(
                            format!("[{}]", us),
                            Style::default().fg(Color::Blue),
                        ));
                        spans.push(Span::raw(" "));
                    }

                    // Description
                    let desc_style = if is_selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(&task.description, desc_style));

                    all_lines.push(Line::from(spans));
                }

                // Task count
                all_lines.push(Line::from(""));
                all_lines.push(Line::from(Span::styled(
                    format!("{} tasks total", task_list.tasks.len()),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            // Action hints (different for implement mode vs task list mode)
            all_lines.push(Line::from(""));
            if is_implement_mode {
                // Implement mode hints
                all_lines.push(Line::from(vec![
                    Span::styled("[j/k]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Select  "),
                    Span::styled("[x]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Toggle done  "),
                    Span::styled("[Enter]", Style::default().fg(Color::Green)),
                    Span::raw(" Execute  "),
                    Span::styled("[a]", Style::default().fg(Color::Magenta)),
                    Span::raw(if self.specify_state.auto_advance { " Auto:ON  " } else { " Auto:OFF " }),
                    Span::styled("[Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" Exit"),
                ]));
            } else {
                // Task generation mode hints
                all_lines.push(Line::from(vec![
                    Span::styled("[j/k]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Select  "),
                    Span::styled("[J/K]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Reorder  "),
                    Span::styled("[t]", Style::default().fg(Color::Magenta)),
                    Span::raw(" Toggle mode  "),
                    Span::styled("[Enter]", Style::default().fg(Color::Green)),
                    Span::raw(" Save  "),
                    Span::styled("[Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" Cancel"),
                ]));
            }
        } else {
            // Standard spec content section
            all_lines.push(Line::from(Span::styled(
                "Generated Specification:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            all_lines.push(Line::from(""));

            // Spec markdown content
            if let Some(spec) = &self.specify_state.generated_spec {
                for line in spec.lines() {
                    all_lines.push(Line::from(line.to_string()));
                }
            }

            // Action hints (T039)
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Green)),
                Span::raw(" Save  "),
                Span::styled("[e]", Style::default().fg(Color::Cyan)),
                Span::raw(" Edit  "),
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" Cancel"),
            ]));
        }

        // Apply scrolling based on content_scroll
        let visible_height = area.height.saturating_sub(2) as usize;
        let visible_lines: Vec<Line> = all_lines
            .into_iter()
            .skip(self.content_scroll)
            .take(visible_height)
            .collect();

        // Determine title based on mode
        let title = if self.specify_state.is_task_list_mode() {
            "Review Tasks (Reorder with J/K)"
        } else {
            "Review Generated Spec"
        };

        // Render paragraph
        let paragraph = Paragraph::new(visible_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render specify Edit Phase with multi-line editor (T067, User Story 3)
    fn render_specify_edit(&self, frame: &mut Frame, area: Rect) {
        // Create 3-section layout: Header | Content | Footer
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header: Title
                Constraint::Min(10),   // Content: Text editor
                Constraint::Length(3), // Footer: Action hints
            ])
            .split(area);

        // === Header: Feature number and name with [EDIT MODE] indicator ===
        let mut header_lines: Vec<Line> = vec![];
        if let (Some(number), Some(name)) = (
            &self.specify_state.feature_number,
            &self.specify_state.feature_name,
        ) {
            header_lines.push(Line::from(Span::styled(
                format!("Feature {} - {} [EDIT MODE]", number, name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            header_lines.push(Line::from(Span::styled(
                "Edit Mode",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let header = Paragraph::new(header_lines)
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(header, sections[0]);

        // === Content: Render TextInput widget ===
        if let Some(input) = &self.specify_state.edit_text_input {
            frame.render_widget(input, sections[1]);
        }

        // === Footer: Action hints ===
        let footer_lines = vec![Line::from(vec![
            Span::styled("[Ctrl+S]", Style::default().fg(Color::Green)),
            Span::raw(" Save  "),
            Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
            Span::raw(" New line  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel  "),
            Span::styled("[Arrow keys]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Navigate"),
        ])];

        let footer = Paragraph::new(footer_lines)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(footer, sections[2]);
    }
}

impl Default for WorktreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for WorktreeView {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Create three columns: Commands (20%) | Content (40%) | Log (40%)
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Commands
                Constraint::Percentage(40), // Content
                Constraint::Percentage(40), // Log
            ])
            .split(area);

        // Store layout rects for mouse click detection
        self.commands_pane_rect = Some(columns[0]);
        self.content_pane_rect = Some(columns[1]);
        self.output_pane_rect = Some(columns[2]);

        // Render all three panels
        self.render_commands(frame, columns[0]);
        self.render_content(frame, columns[1]);
        self.render_output(frame, columns[2]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        // Route to commit review input handler when in CommitReview mode (Feature 050)
        if self.content_type == ContentType::CommitReview && self.focus == WorktreeFocus::Content {
            return self.handle_commit_review_input(key);
        }

        // Route to specify input handler when in SpecifyInput mode (Feature 051 - T022, T023)
        if self.content_type == ContentType::SpecifyInput && self.focus == WorktreeFocus::Content {
            return self.handle_specify_input(key);
        }

        // Route to specify review/edit input handler when in SpecifyReview mode (Feature 051)
        if self.content_type == ContentType::SpecifyReview && self.focus == WorktreeFocus::Content
        {
            // T054-T067: Route to edit handler if in edit mode (User Story 3)
            if self.specify_state.edit_mode {
                return self.handle_specify_edit_input(key);
            }
            // T037: Otherwise route to review handler
            return self.handle_specify_review_input(key);
        }

        // Route to Prompt Claude input handler when in PromptInput mode (Task 1.5)
        if self.content_type == ContentType::PromptInput && self.focus == WorktreeFocus::Content {
            return self.handle_prompt_input_key(key);
        }

        match key.code {
            KeyCode::Char('h') | KeyCode::Left => {
                if self.focus == WorktreeFocus::Content {
                    // Cycle tabs left when Content is focused
                    self.content_type = match self.content_type {
                        ContentType::Spec => ContentType::Tasks,
                        ContentType::Plan => ContentType::Spec,
                        ContentType::Tasks => ContentType::Plan,
                        ContentType::CommitReview => ContentType::CommitReview, // No tab cycling during review
                        ContentType::SpecifyInput => ContentType::SpecifyInput, // No tab cycling during specify
                        ContentType::SpecifyReview => ContentType::SpecifyReview, // No tab cycling during specify
                        ContentType::PromptInput => ContentType::PromptInput, // No tab cycling during prompt input
                        ContentType::PromptRunning => ContentType::PromptRunning, // No tab cycling during prompt running
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
                if self.focus == WorktreeFocus::Output {
                    self.scroll_output_down();
                } else {
                    self.scroll_down();
                }
                ViewAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.focus == WorktreeFocus::Output {
                    self.scroll_output_up();
                } else {
                    self.scroll_up();
                }
                ViewAction::None
            }
            KeyCode::Char('s') => {
                self.switch_content();
                ViewAction::None
            }
            KeyCode::PageDown => {
                if self.focus == WorktreeFocus::Output {
                    self.scroll_output_page_down();
                } else if self.focus == WorktreeFocus::Content {
                    if let Some(content) = self.get_current_content() {
                        let line_count = content.lines().count();
                        self.content_scroll =
                            (self.content_scroll + 10).min(line_count.saturating_sub(1));
                    }
                }
                ViewAction::None
            }
            KeyCode::PageUp => {
                if self.focus == WorktreeFocus::Output {
                    self.scroll_output_page_up();
                } else if self.focus == WorktreeFocus::Content {
                    self.content_scroll = self.content_scroll.saturating_sub(10);
                }
                ViewAction::None
            }
            KeyCode::Home | KeyCode::Char('g') => {
                if self.focus == WorktreeFocus::Output {
                    self.output_scroll = 0;
                } else if self.focus == WorktreeFocus::Content {
                    self.content_scroll = 0;
                }
                ViewAction::None
            }
            KeyCode::End | KeyCode::Char('G') => {
                if self.focus == WorktreeFocus::Output {
                    self.scroll_output_to_bottom();
                } else if self.focus == WorktreeFocus::Content {
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
                                    Command::PromptClaude => {
                                        self.start_prompt_input();
                                        ViewAction::None
                                    }
                                    Command::SddPhase(phase, _) => {
                                        match phase {
                                            // Specify phase uses new interactive flow (Feature 051)
                                            SpecPhase::Specify => {
                                                self.start_specify_input();
                                                ViewAction::None
                                            }
                                            // Implement phase uses task execution mode (Feature 056)
                                            SpecPhase::Implement => self.start_implement_mode(),
                                            // All other phases use interactive flow (Feature 057, 058)
                                            SpecPhase::Clarify
                                            | SpecPhase::Plan
                                            | SpecPhase::Tasks
                                            | SpecPhase::Analyze
                                            | SpecPhase::Checklist
                                            | SpecPhase::Review => {
                                                self.start_interactive_phase(*phase);
                                                ViewAction::None
                                            }
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
        if self.tick_count.is_multiple_of(Self::REFRESH_INTERVAL) {
            // Refresh will be triggered by GitInfoUpdated event
            // No action needed here
        }
    }
}

// Mouse handling (outside View trait)
impl WorktreeView {
    /// Handle mouse click events
    pub fn handle_mouse(&mut self, col: u16, row: u16) {
        // Helper function for point-in-rect check
        fn point_in_rect(col: u16, row: u16, rect: &Rect) -> bool {
            col >= rect.x
                && col < rect.x + rect.width
                && row >= rect.y
                && row < rect.y + rect.height
        }

        // Check which pane was clicked and switch focus
        if let Some(rect) = self.commands_pane_rect {
            if point_in_rect(col, row, &rect) {
                self.focus = WorktreeFocus::Commands;
                return;
            }
        }

        if let Some(rect) = self.content_pane_rect {
            if point_in_rect(col, row, &rect) {
                self.focus = WorktreeFocus::Content;
                return;
            }
        }

        if let Some(rect) = self.output_pane_rect {
            if point_in_rect(col, row, &rect) {
                self.focus = WorktreeFocus::Output;
            }
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
    fn test_enter_on_specify_starts_input_mode() {
        let mut view = create_test_view();

        // Set focus to Commands panel
        view.focus = WorktreeFocus::Commands;

        // Select Specify (display index 1 = command index 0)
        view.command_state.select(Some(1));

        // Press Enter
        let action = view.handle_key(key_event(KeyCode::Enter));

        // Feature 051: Should start specify input mode directly, not request input dialog
        assert_eq!(action, ViewAction::None);

        // Verify specify input mode is active
        assert_eq!(view.content_type, ContentType::SpecifyInput);
        assert_eq!(view.specify_state.input_buffer, "");
        assert!(!view.specify_state.is_generating);
    }

    #[test]
    fn test_enter_on_clarify_uses_interactive_flow() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Select Clarify (display index 2 = command index 1)
        view.command_state.select(Some(2));

        let action = view.handle_key(key_event(KeyCode::Enter));

        // Feature 057: Should start interactive flow (Input → Generate → Review)
        assert_eq!(action, ViewAction::None);

        // Verify interactive input mode is active for Clarify phase
        assert_eq!(view.content_type, ContentType::SpecifyInput);
        assert_eq!(view.specify_state.current_phase, SpecPhase::Clarify);
        assert_eq!(view.specify_state.input_buffer, "");
        assert!(!view.specify_state.is_generating);
    }

    #[test]
    fn test_enter_on_git_commit_requests_input() {
        let mut view = create_test_view();
        view.focus = WorktreeFocus::Commands;

        // Find and select Git Intelligent Commit command
        let num_phases = view.phases.len();
        let commit_display_idx = num_phases + 3; // First git command (Intelligent Commit)

        view.command_state.select(Some(commit_display_idx));

        let action = view.handle_key(key_event(KeyCode::Enter));

        // Should trigger intelligent commit workflow
        match action {
            ViewAction::RunIntelligentCommit => {
                // Expected behavior - triggers security scanning and commit workflow
            }
            _ => panic!("Expected RunIntelligentCommit action for Git Intelligent Commit"),
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

        // Test cycle behavior (left goes reverse: Commands → Output → Content)
        view.focus_left();
        assert_eq!(view.focus, WorktreeFocus::Output);

        view.focus_left();
        assert_eq!(view.focus, WorktreeFocus::Content);
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

    // Feature 051: Tests for specify workflow (T031-T032)

    #[test]
    fn test_specify_state_default() {
        let state = SpecifyState::default();
        assert_eq!(state.current_phase, SpecPhase::Specify);
        assert_eq!(state.input_buffer, "");
        assert_eq!(state.input_cursor, 0);
        assert!(!state.is_generating);
        assert!(state.generation_error.is_none());
        assert!(state.generated_spec.is_none());
        assert!(state.feature_number.is_none());
        assert!(state.feature_name.is_none());
        assert!(!state.edit_mode);
        assert!(state.edit_text_input.is_none());
        assert!(state.validation_error.is_none());
    }

    #[test]
    fn test_specify_state_for_phase() {
        // Test for_phase creates state with correct phase
        let state = SpecifyState::for_phase(SpecPhase::Plan);
        assert_eq!(state.current_phase, SpecPhase::Plan);
        assert_eq!(state.input_buffer, "");
        assert!(!state.is_generating);

        let state = SpecifyState::for_phase(SpecPhase::Tasks);
        assert_eq!(state.current_phase, SpecPhase::Tasks);
    }

    #[test]
    fn test_start_interactive_phase() {
        let mut view = create_test_view();

        // Test starting Plan phase
        view.start_interactive_phase(SpecPhase::Plan);
        assert_eq!(view.content_type, ContentType::SpecifyInput);
        assert_eq!(view.specify_state.current_phase, SpecPhase::Plan);
        assert_eq!(view.focus, WorktreeFocus::Content);

        // Test starting Tasks phase
        view.start_interactive_phase(SpecPhase::Tasks);
        assert_eq!(view.specify_state.current_phase, SpecPhase::Tasks);

        // Test starting Clarify phase
        view.start_interactive_phase(SpecPhase::Clarify);
        assert_eq!(view.specify_state.current_phase, SpecPhase::Clarify);
    }

    #[test]
    fn test_specify_state_target_filename() {
        let mut state = SpecifyState::default();

        state.current_phase = SpecPhase::Specify;
        assert_eq!(state.target_filename(), "spec.md");

        state.current_phase = SpecPhase::Clarify;
        assert_eq!(state.target_filename(), "spec.md");

        state.current_phase = SpecPhase::Plan;
        assert_eq!(state.target_filename(), "plan.md");

        state.current_phase = SpecPhase::Tasks;
        assert_eq!(state.target_filename(), "tasks.md");

        state.current_phase = SpecPhase::Analyze;
        assert_eq!(state.target_filename(), "analysis.md");
    }

    #[test]
    fn test_specify_state_input_prompt() {
        let mut state = SpecifyState::default();

        state.current_phase = SpecPhase::Specify;
        assert!(state.input_prompt().contains("feature description"));

        state.current_phase = SpecPhase::Plan;
        assert!(state.input_prompt().contains("implementation notes"));

        state.current_phase = SpecPhase::Tasks;
        assert!(state.input_prompt().contains("task generation"));
    }

    #[test]
    fn test_specify_state_validation_by_phase() {
        let mut state = SpecifyState::default();

        // Specify phase requires non-empty input
        state.current_phase = SpecPhase::Specify;
        state.input_buffer = "".to_string();
        assert!(state.validate_input().is_err());

        state.input_buffer = "ab".to_string();
        assert!(state.validate_input().is_err()); // Too short

        state.input_buffer = "valid description".to_string();
        assert!(state.validate_input().is_ok());

        // Other phases allow empty input
        state.current_phase = SpecPhase::Plan;
        state.input_buffer = "".to_string();
        assert!(state.validate_input().is_ok());

        state.current_phase = SpecPhase::Tasks;
        state.input_buffer = "".to_string();
        assert!(state.validate_input().is_ok());
    }

    #[test]
    fn test_start_specify_input() {
        let mut view = create_test_view();
        view.start_specify_input();

        assert_eq!(view.content_type, ContentType::SpecifyInput);
        assert_eq!(view.specify_state.input_buffer, "");
        assert_eq!(view.specify_state.input_cursor, 0);
        assert!(!view.specify_state.is_generating);
        assert!(view.specify_state.generation_error.is_none());
    }

    #[test]
    fn test_cancel_specify() {
        let mut view = create_test_view();

        // Set up specify state
        view.content_type = ContentType::SpecifyInput;
        view.specify_state.input_buffer = "test input".to_string();
        view.specify_state.input_cursor = 5;
        view.specify_state.is_generating = true;

        // Cancel should reset everything
        view.cancel_specify();

        assert_eq!(view.content_type, ContentType::Spec);
        assert_eq!(view.specify_state.input_buffer, "");
        assert_eq!(view.specify_state.input_cursor, 0);
        assert!(!view.specify_state.is_generating);
    }

    #[test]
    fn test_specify_state_transition_input_to_generating() {
        let mut view = create_test_view();

        // Start in input mode
        view.start_specify_input();
        assert_eq!(view.content_type, ContentType::SpecifyInput);
        assert!(!view.specify_state.is_generating);

        // Simulate generation started
        view.specify_state.is_generating = true;
        assert!(view.specify_state.is_generating);
        assert_eq!(view.content_type, ContentType::SpecifyInput);
    }

    #[test]
    fn test_specify_state_transition_generating_to_review() {
        let mut view = create_test_view();

        // Start with generating state
        view.content_type = ContentType::SpecifyInput;
        view.specify_state.is_generating = true;

        // Simulate generation completed
        view.specify_state.is_generating = false;
        view.specify_state.generated_spec = Some("# Test Spec".to_string());
        view.specify_state.feature_number = Some("052".to_string());
        view.specify_state.feature_name = Some("test-feature".to_string());
        view.content_type = ContentType::SpecifyReview;

        // Verify state
        assert!(!view.specify_state.is_generating);
        assert_eq!(view.content_type, ContentType::SpecifyReview);
        assert!(view.specify_state.generated_spec.is_some());
        assert_eq!(
            view.specify_state.feature_number.as_deref(),
            Some("052")
        );
        assert_eq!(
            view.specify_state.feature_name.as_deref(),
            Some("test-feature")
        );
    }

    #[test]
    fn test_specify_state_transition_generating_to_error() {
        let mut view = create_test_view();

        // Start with generating state
        view.content_type = ContentType::SpecifyInput;
        view.specify_state.is_generating = true;

        // Simulate generation failed
        view.specify_state.is_generating = false;
        view.specify_state.generation_error = Some("Test error".to_string());

        // Verify state
        assert!(!view.specify_state.is_generating);
        assert_eq!(view.content_type, ContentType::SpecifyInput);
        assert_eq!(
            view.specify_state.generation_error.as_deref(),
            Some("Test error")
        );
    }

    #[test]
    fn test_specify_input_handling_character() {
        let mut view = create_test_view();
        view.start_specify_input();

        // Simulate typing 'a'
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        let action = view.handle_specify_input(key);

        assert_eq!(action, ViewAction::None);
        assert_eq!(view.specify_state.input_buffer, "a");
        assert_eq!(view.specify_state.input_cursor, 1);
    }

    #[test]
    fn test_specify_input_handling_backspace() {
        let mut view = create_test_view();
        view.start_specify_input();
        view.specify_state.input_buffer = "test".to_string();
        view.specify_state.input_cursor = 4;

        // Simulate backspace
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty());
        let action = view.handle_specify_input(key);

        assert_eq!(action, ViewAction::None);
        assert_eq!(view.specify_state.input_buffer, "tes");
        assert_eq!(view.specify_state.input_cursor, 3);
    }

    #[test]
    fn test_specify_input_validation_empty() {
        let mut view = create_test_view();
        view.start_specify_input();

        // Try to submit empty input
        let action = view.submit_specify_description();

        assert_eq!(action, ViewAction::None);
        assert!(view.specify_state.validation_error.is_some());
        assert!(view
            .specify_state
            .validation_error
            .as_ref()
            .unwrap()
            .contains("cannot be empty"));
    }

    #[test]
    fn test_specify_input_validation_too_short() {
        let mut view = create_test_view();
        view.start_specify_input();
        view.specify_state.input_buffer = "ab".to_string();

        // Try to submit too-short input
        let action = view.submit_specify_description();

        assert_eq!(action, ViewAction::None);
        assert!(view.specify_state.validation_error.is_some());
        assert!(view
            .specify_state
            .validation_error
            .as_ref()
            .unwrap()
            .contains("at least 3 characters"));
    }

    #[test]
    fn test_specify_input_validation_success() {
        let mut view = create_test_view();
        view.start_specify_input();
        view.specify_state.input_buffer = "Valid feature description".to_string();

        // Submit valid input
        let action = view.submit_specify_description();

        // Should return GenerateSpec action
        match action {
            ViewAction::GenerateSpec { phase, description } => {
                assert_eq!(phase, SpecPhase::Specify);
                assert_eq!(description, "Valid feature description");
            }
            _ => panic!("Expected GenerateSpec action"),
        }
        assert!(view.specify_state.validation_error.is_none());
    }

    // Feature 051: Tests for specify review mode (T050-T051)

    #[test]
    fn test_specify_review_state_transition() {
        let mut view = create_test_view();

        // Simulate generation completed
        view.specify_state.generated_spec = Some("# Test Spec\n\nContent here".to_string());
        view.specify_state.feature_number = Some("052".to_string());
        view.specify_state.feature_name = Some("test-feature".to_string());
        view.content_type = ContentType::SpecifyReview;
        view.focus = WorktreeFocus::Content;

        // Verify state
        assert_eq!(view.content_type, ContentType::SpecifyReview);
        assert!(view.specify_state.generated_spec.is_some());
        assert_eq!(
            view.specify_state.feature_number.as_deref(),
            Some("052")
        );
        assert_eq!(
            view.specify_state.feature_name.as_deref(),
            Some("test-feature")
        );
    }

    #[test]
    fn test_save_specify_spec_action() {
        let mut view = create_test_view();

        // Set up review state
        view.specify_state.generated_spec = Some("# Test".to_string());
        view.specify_state.feature_number = Some("052".to_string());
        view.specify_state.feature_name = Some("test".to_string());

        // Trigger save
        let action = view.save_specify_spec();

        // Verify action
        match action {
            ViewAction::SaveSpec {
                phase,
                content,
                number,
                name,
            } => {
                assert_eq!(phase, SpecPhase::Specify);
                assert_eq!(content, "# Test");
                assert_eq!(number, "052");
                assert_eq!(name, "test");
            }
            _ => panic!("Expected SaveSpec action"),
        }
    }

    #[test]
    fn test_specify_review_enter_key_saves() {
        let mut view = create_test_view();

        // Set up review state
        view.specify_state.generated_spec = Some("# Spec".to_string());
        view.specify_state.feature_number = Some("052".to_string());
        view.specify_state.feature_name = Some("test".to_string());
        view.content_type = ContentType::SpecifyReview;
        view.focus = WorktreeFocus::Content;

        // Press Enter
        let action = view.handle_specify_review_input(key_event(KeyCode::Enter));

        // Should trigger save
        match action {
            ViewAction::SaveSpec { .. } => {} // Expected
            _ => panic!("Expected SaveSpec action on Enter"),
        }
    }

    #[test]
    fn test_specify_review_esc_cancels() {
        let mut view = create_test_view();

        // Set up review state
        view.content_type = ContentType::SpecifyReview;
        view.specify_state.generated_spec = Some("# Test".to_string());

        // Press Esc
        let action = view.handle_specify_review_input(key_event(KeyCode::Esc));

        // Should cancel and return None
        assert_eq!(action, ViewAction::None);
        assert_eq!(view.content_type, ContentType::Spec);
        assert!(view.specify_state.generated_spec.is_none());
    }

    // === User Story 3: Edit Mode Tests ===

    #[test]
    fn test_toggle_specify_edit_mode() {
        let mut view = create_test_view();

        // Set up review state with generated spec
        view.content_type = ContentType::SpecifyReview;
        view.specify_state.generated_spec = Some("# Test Spec\n\nContent here".to_string());
        view.specify_state.feature_number = Some("051".to_string());
        view.specify_state.feature_name = Some("edit-mode".to_string());

        // Initially not in edit mode
        assert!(!view.specify_state.edit_mode);
        assert!(view.specify_state.edit_text_input.is_none());

        // Toggle edit mode
        view.toggle_specify_edit_mode();

        // Should now be in edit mode with TextInput initialized
        assert!(view.specify_state.edit_mode);
        assert!(view.specify_state.edit_text_input.is_some());

        // Verify TextInput contains the spec content
        if let Some(input) = &view.specify_state.edit_text_input {
            assert_eq!(input.lines.len(), 3); // "# Test Spec", "", "Content here"
            assert_eq!(input.lines[0], "# Test Spec");
            assert_eq!(input.lines[1], "");
            assert_eq!(input.lines[2], "Content here");
            assert_eq!(input.cursor_line, 0);
            assert_eq!(input.cursor_column, 0);
        }
    }

    #[test]
    fn test_save_from_edit() {
        let mut view = create_test_view();

        // Set up edit mode with modified content
        view.content_type = ContentType::SpecifyReview;
        view.specify_state.edit_mode = true;
        view.specify_state.feature_number = Some("051".to_string());
        view.specify_state.feature_name = Some("edit-mode".to_string());

        // Create TextInput with edited content
        let mut input = TextInput::new_multiline(String::new(), 25);
        input.lines = vec![
            "# Edited Spec".to_string(),
            "".to_string(),
            "Modified content".to_string(),
        ];
        view.specify_state.edit_text_input = Some(input);

        // Save from edit
        let action = view.save_from_edit();

        // Should exit edit mode
        assert!(!view.specify_state.edit_mode);
        assert!(view.specify_state.edit_text_input.is_none());

        // Should update generated_spec with edited content
        assert!(view.specify_state.generated_spec.is_some());
        let spec = view.specify_state.generated_spec.as_ref().unwrap();
        assert!(spec.contains("# Edited Spec"));
        assert!(spec.contains("Modified content"));

        // Should return SaveSpec action
        match action {
            ViewAction::SaveSpec { phase, content, number, name } => {
                assert_eq!(phase, SpecPhase::Specify);
                assert!(content.contains("# Edited Spec"));
                assert_eq!(number, "051");
                assert_eq!(name, "edit-mode");
            }
            _ => panic!("Expected SaveSpec action"),
        }
    }

    #[test]
    fn test_cancel_edit() {
        let mut view = create_test_view();

        // Set up edit mode
        view.content_type = ContentType::SpecifyReview;
        view.specify_state.edit_mode = true;
        view.specify_state.generated_spec = Some("# Original Spec".to_string());

        // Create TextInput with modified content
        let mut input = TextInput::new_multiline(String::new(), 25);
        input.lines = vec!["# Modified Spec".to_string()];
        view.specify_state.edit_text_input = Some(input);

        // Cancel edit
        view.cancel_edit();

        // Should exit edit mode and discard changes
        assert!(!view.specify_state.edit_mode);
        assert!(view.specify_state.edit_text_input.is_none());

        // Original spec should remain unchanged
        assert_eq!(
            view.specify_state.generated_spec.as_deref(),
            Some("# Original Spec")
        );
    }

    #[test]
    fn test_edit_mode_key_handling() {
        let mut view = create_test_view();

        // Set up edit mode
        view.content_type = ContentType::SpecifyReview;
        view.specify_state.edit_mode = true;
        view.specify_state.feature_number = Some("051".to_string());
        view.specify_state.feature_name = Some("edit-mode".to_string());

        // Create TextInput
        let mut input = TextInput::new_multiline(String::new(), 25);
        input.lines = vec!["Test".to_string()];
        view.specify_state.edit_text_input = Some(input);

        // Test Ctrl+S (save)
        let action = view.handle_specify_edit_input(key_event_with_modifiers(
            KeyCode::Char('s'),
            KeyModifiers::CONTROL,
        ));
        assert!(matches!(action, ViewAction::SaveSpec { .. }));
        assert!(!view.specify_state.edit_mode); // Should exit edit mode

        // Reset for next test
        view.specify_state.edit_mode = true;
        let mut input = TextInput::new_multiline(String::new(), 25);
        input.lines = vec!["Test".to_string()];
        view.specify_state.edit_text_input = Some(input);

        // Test Esc (cancel)
        let action = view.handle_specify_edit_input(key_event(KeyCode::Esc));
        assert_eq!(action, ViewAction::None);
        assert!(!view.specify_state.edit_mode); // Should exit edit mode
        assert!(view.specify_state.edit_text_input.is_none());
    }

    #[test]
    fn test_edit_multiline_content() {
        let mut view = create_test_view();

        // Set up edit mode with multi-line content
        view.content_type = ContentType::SpecifyReview;
        view.specify_state.edit_mode = true;

        // Create TextInput with multiple lines
        let mut input = TextInput::new_multiline(String::new(), 25);
        input.lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];
        input.cursor_line = 1;
        input.cursor_column = 0;
        view.specify_state.edit_text_input = Some(input);

        // Test Enter (insert newline)
        view.handle_specify_edit_input(key_event(KeyCode::Enter));
        if let Some(input) = &view.specify_state.edit_text_input {
            assert_eq!(input.lines.len(), 4); // Should have 4 lines now
        }

        // Test character insertion
        view.handle_specify_edit_input(key_event(KeyCode::Char('X')));
        if let Some(input) = &view.specify_state.edit_text_input {
            assert!(input.lines[2].contains('X')); // Character added
        }

        // Test arrow key navigation
        view.handle_specify_edit_input(key_event(KeyCode::Down));
        if let Some(input) = &view.specify_state.edit_text_input {
            assert_eq!(input.cursor_line, 3); // Moved down
        }

        view.handle_specify_edit_input(key_event(KeyCode::Up));
        if let Some(input) = &view.specify_state.edit_text_input {
            assert_eq!(input.cursor_line, 2); // Moved up
        }
    }

    // Helper function for creating key events with modifiers
    fn key_event_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    // ===== Feature 055: Interactive Task Generation Tests =====

    #[test]
    fn test_parsed_task_parse_basic() {
        let line = "- [ ] T001 Create User model in src/models/user.rs";
        let task = ParsedTask::parse(line).expect("Should parse basic task");

        assert_eq!(task.id, "T001");
        assert!(!task.is_parallel);
        assert!(task.user_story.is_none());
        assert_eq!(task.description, "Create User model in src/models/user.rs");
        assert!(!task.completed);
    }

    #[test]
    fn test_parsed_task_parse_with_parallel_marker() {
        let line = "- [ ] T002 [P] Implement UserService in src/services/";
        let task = ParsedTask::parse(line).expect("Should parse task with [P]");

        assert_eq!(task.id, "T002");
        assert!(task.is_parallel);
        assert!(task.user_story.is_none());
        assert_eq!(task.description, "Implement UserService in src/services/");
        assert!(!task.completed);
    }

    #[test]
    fn test_parsed_task_parse_with_user_story() {
        let line = "- [ ] T003 [US1] Add validation logic";
        let task = ParsedTask::parse(line).expect("Should parse task with [US]");

        assert_eq!(task.id, "T003");
        assert!(!task.is_parallel);
        assert_eq!(task.user_story, Some("US1".to_string()));
        assert_eq!(task.description, "Add validation logic");
    }

    #[test]
    fn test_parsed_task_parse_with_all_markers() {
        let line = "- [X] T004 [P] [US2] Complete implementation";
        let task = ParsedTask::parse(line).expect("Should parse task with all markers");

        assert_eq!(task.id, "T004");
        assert!(task.is_parallel);
        assert_eq!(task.user_story, Some("US2".to_string()));
        assert_eq!(task.description, "Complete implementation");
        assert!(task.completed);
    }

    #[test]
    fn test_parsed_task_parse_rejects_non_task() {
        assert!(ParsedTask::parse("# Section Header").is_none());
        assert!(ParsedTask::parse("Regular paragraph text").is_none());
        assert!(ParsedTask::parse("- [ ] No task ID").is_none());
        assert!(ParsedTask::parse("").is_none());
    }

    #[test]
    fn test_parsed_task_to_markdown() {
        let task = ParsedTask {
            id: "T001".to_string(),
            is_parallel: false,
            user_story: None,
            description: "Test description".to_string(),
            file_path: None,
            completed: false,
            raw_line: String::new(),
        };

        assert_eq!(task.to_markdown(), "- [ ] T001 Test description");

        let task_parallel = ParsedTask {
            id: "T002".to_string(),
            is_parallel: true,
            user_story: Some("US1".to_string()),
            description: "Parallel task".to_string(),
            file_path: None,
            completed: true,
            raw_line: String::new(),
        };

        assert_eq!(task_parallel.to_markdown(), "- [X] T002 [P] [US1] Parallel task");
    }

    #[test]
    fn test_task_list_state_from_markdown() {
        let content = r#"# Tasks

- [ ] T001 First task
- [ ] T002 [P] Parallel task
- [ ] T003 [US1] User story task

## Dependencies

Some other content
"#;
        let state = TaskListState::from_markdown(content);

        assert_eq!(state.len(), 3);
        assert_eq!(state.tasks[0].id, "T001");
        assert_eq!(state.tasks[1].id, "T002");
        assert!(state.tasks[1].is_parallel);
        assert_eq!(state.tasks[2].id, "T003");
        assert_eq!(state.tasks[2].user_story, Some("US1".to_string()));
    }

    #[test]
    fn test_task_list_state_selection() {
        let content = "- [ ] T001 First\n- [ ] T002 Second\n- [ ] T003 Third";
        let mut state = TaskListState::from_markdown(content);

        assert_eq!(state.selected, 0);

        state.select_next();
        assert_eq!(state.selected, 1);

        state.select_next();
        assert_eq!(state.selected, 2);

        // Should not go past last item
        state.select_next();
        assert_eq!(state.selected, 2);

        state.select_previous();
        assert_eq!(state.selected, 1);

        state.select_previous();
        assert_eq!(state.selected, 0);

        // Should not go before first item
        state.select_previous();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_task_list_state_reorder_up() {
        let content = "- [ ] T001 First\n- [ ] T002 Second\n- [ ] T003 Third";
        let mut state = TaskListState::from_markdown(content);

        // Select second task
        state.select_next();
        assert_eq!(state.selected, 1);
        assert_eq!(state.tasks[1].id, "T002");

        // Reorder up
        state.reorder_up();

        // T002 should now be first, selection follows
        assert_eq!(state.selected, 0);
        assert_eq!(state.tasks[0].id, "T002");
        assert_eq!(state.tasks[1].id, "T001");
        assert_eq!(state.tasks[2].id, "T003");

        // Reorder up at top should do nothing
        state.reorder_up();
        assert_eq!(state.selected, 0);
        assert_eq!(state.tasks[0].id, "T002");
    }

    #[test]
    fn test_task_list_state_reorder_down() {
        let content = "- [ ] T001 First\n- [ ] T002 Second\n- [ ] T003 Third";
        let mut state = TaskListState::from_markdown(content);

        // Selection starts at first task
        assert_eq!(state.selected, 0);
        assert_eq!(state.tasks[0].id, "T001");

        // Reorder down
        state.reorder_down();

        // T001 should now be second, selection follows
        assert_eq!(state.selected, 1);
        assert_eq!(state.tasks[0].id, "T002");
        assert_eq!(state.tasks[1].id, "T001");
        assert_eq!(state.tasks[2].id, "T003");

        // Reorder down again
        state.reorder_down();
        assert_eq!(state.selected, 2);
        assert_eq!(state.tasks[2].id, "T001");

        // Reorder down at bottom should do nothing
        state.reorder_down();
        assert_eq!(state.selected, 2);
        assert_eq!(state.tasks[2].id, "T001");
    }

    #[test]
    fn test_task_list_state_to_markdown() {
        let content = "- [ ] T001 First\n- [ ] T002 [P] Second\n- [X] T003 [US1] Third";
        let mut state = TaskListState::from_markdown(content);

        // Reorder: move T003 to top
        state.select_next();
        state.select_next(); // Select T003
        state.reorder_up();  // T003 is now second
        state.reorder_up();  // T003 is now first

        let markdown = state.to_markdown();
        let lines: Vec<&str> = markdown.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("T003"));
        assert!(lines[1].contains("T001"));
        assert!(lines[2].contains("T002"));
    }

    #[test]
    fn test_specify_state_task_list_mode() {
        let mut state = SpecifyState::for_phase(SpecPhase::Tasks);

        // Initially not in task list mode
        assert!(!state.is_task_list_mode());

        // Set content with tasks
        let content = "- [ ] T001 First\n- [ ] T002 Second".to_string();
        state.set_generated_content(content, "055".to_string(), "test-feature".to_string());

        // Should now have task list state
        assert!(state.task_list_state.is_some());
        let task_list = state.task_list_state.as_ref().unwrap();
        assert_eq!(task_list.len(), 2);

        // Enable list mode
        if let Some(ref mut tl) = state.task_list_state {
            tl.list_mode = true;
        }
        assert!(state.is_task_list_mode());

        // get_current_content should return task list markdown
        let content = state.get_current_content();
        assert!(content.is_some());
        assert!(content.unwrap().contains("T001"));
    }

    #[test]
    fn test_specify_state_task_list_get_content_after_reorder() {
        let mut state = SpecifyState::for_phase(SpecPhase::Tasks);

        let content = "- [ ] T001 First\n- [ ] T002 Second\n- [ ] T003 Third".to_string();
        state.set_generated_content(content, "055".to_string(), "test".to_string());

        // Reorder tasks
        if let Some(ref mut tl) = state.task_list_state {
            tl.list_mode = true;
            tl.reorder_down(); // Move T001 down
        }

        // Content should reflect reordered tasks
        let result = state.get_current_content().unwrap();
        let lines: Vec<&str> = result.lines().collect();

        // T002 should now be first
        assert!(lines[0].contains("T002"));
        assert!(lines[1].contains("T001"));
        assert!(lines[2].contains("T003"));
    }
}
