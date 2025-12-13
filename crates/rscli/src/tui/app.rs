//! Application state and main loop for the TUI

use crate::tui::event::{Event, EventHandler};
use crate::tui::protocol::{OutputParser, ProtocolMessage};
use crate::tui::views::{CommandRunner, Dashboard, SpecView, View, ViewAction, ViewType, WorktreeView};
use crate::tui::widgets::TextInput;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::sync::mpsc;

/// Result type for the app
pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Available views in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    Worktree,
    Commands,
    Spec,
    Dashboard,
}

/// Main application state
pub struct App {
    /// Is the app running?
    pub running: bool,
    /// Current active view
    pub current_view: CurrentView,
    /// Worktree-focused development workspace view state
    pub worktree_view: WorktreeView,
    /// Dashboard view state
    pub dashboard: Dashboard,
    /// Command runner view state
    pub command_runner: CommandRunner,
    /// Spec-driven development view state
    pub spec_view: SpecView,
    /// Status message to show at bottom
    pub status_message: Option<String>,
    /// Event sender for command output
    pub event_sender: Option<mpsc::Sender<Event>>,
    /// Currently running spec phase (for auto-flow tracking)
    pub running_spec_phase: Option<String>,
    /// Flag to trigger visual view copy on next render
    pub copy_visual_view: bool,
    /// Protocol parser for Claude Code ↔ TUI communication
    pub protocol_parser: OutputParser,
    /// Text input widget for interactive user input
    pub text_input: Option<TextInput>,
    /// Whether the app is in input mode (capturing text input)
    pub input_mode: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            running: true,
            current_view: CurrentView::Worktree,
            worktree_view: WorktreeView::new(),
            dashboard: Dashboard::new(),
            command_runner: CommandRunner::new(),
            spec_view: SpecView::new(),
            status_message: None,
            event_sender: None,
            running_spec_phase: None,
            copy_visual_view: false,
            protocol_parser: OutputParser::new(),
            text_input: None,
            input_mode: false,
        }
    }

    /// Handle key events
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // If in input mode, handle input separately
        if self.input_mode {
            self.handle_key_event_in_input_mode(key);
            return;
        }

        // Global key bindings
        match key.code {
            // Quit on Ctrl+C or q (when not in command view with running command)
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
                return;
            }
            KeyCode::Char('q')
                if (self.current_view == CurrentView::Dashboard || self.current_view == CurrentView::Worktree)
                    && !self.command_runner.is_running() =>
            {
                self.running = false;
                return;
            }
            // Switch tabs/views with [ and ]
            KeyCode::Char('[') => {
                self.current_view = match self.current_view {
                    CurrentView::Worktree => CurrentView::Dashboard,
                    CurrentView::Commands => CurrentView::Worktree,
                    CurrentView::Spec => CurrentView::Commands,
                    CurrentView::Dashboard => CurrentView::Spec,
                };
                self.status_message = Some(format!("Switched to {} view", match self.current_view {
                    CurrentView::Worktree => "Worktree",
                    CurrentView::Commands => "Commands",
                    CurrentView::Spec => "Spec",
                    CurrentView::Dashboard => "Dashboard",
                }));
                return;
            }
            KeyCode::Char(']') => {
                self.current_view = match self.current_view {
                    CurrentView::Worktree => CurrentView::Commands,
                    CurrentView::Commands => CurrentView::Spec,
                    CurrentView::Spec => CurrentView::Dashboard,
                    CurrentView::Dashboard => CurrentView::Worktree,
                };
                self.status_message = Some(format!("Switched to {} view", match self.current_view {
                    CurrentView::Worktree => "Worktree",
                    CurrentView::Commands => "Commands",
                    CurrentView::Spec => "Spec",
                    CurrentView::Dashboard => "Dashboard",
                }));
                return;
            }
            // Update rscli: build and install to ~/.local/bin
            KeyCode::Char('U') => {
                self.run_update();
                return;
            }
            // Switch panes within current view with Tab
            KeyCode::Tab => {
                match self.current_view {
                    CurrentView::Worktree => {
                        self.worktree_view.next_pane();
                        self.status_message = Some("Switched to next pane".to_string());
                    }
                    CurrentView::Dashboard => {
                        self.dashboard.next_pane();
                        self.status_message = Some("Switched to next pane".to_string());
                    }
                    CurrentView::Commands => {
                        self.command_runner.next_pane();
                        self.status_message = Some("Switched to next pane".to_string());
                    }
                    CurrentView::Spec => {
                        self.spec_view.next_pane();
                        self.status_message = Some("Switched to next pane".to_string());
                    }
                }
                return;
            }
            // Number keys for quick view switch
            KeyCode::Char('1') => {
                self.current_view = CurrentView::Worktree;
                return;
            }
            KeyCode::Char('2') => {
                self.current_view = CurrentView::Commands;
                return;
            }
            KeyCode::Char('3') => {
                self.current_view = CurrentView::Spec;
                return;
            }
            KeyCode::Char('4') => {
                self.current_view = CurrentView::Dashboard;
                return;
            }
            // Copy current pane content with y
            KeyCode::Char('y') => {
                self.copy_current_pane();
                return;
            }
            // Copy current tab with styling with Y (Shift+y) - visual screenshot mode
            KeyCode::Char('Y') => {
                self.copy_visual_view = true;
                return;
            }
            _ => {}
        }

        // Delegate to current view and handle returned action
        let action = match self.current_view {
            CurrentView::Worktree => self.worktree_view.handle_key(key),
            CurrentView::Dashboard => self.dashboard.handle_key(key),
            CurrentView::Commands => self.command_runner.handle_key(key),
            CurrentView::Spec => self.spec_view.handle_key(key),
        };

        self.handle_view_action(action);
    }

    /// Handle actions returned from views
    fn handle_view_action(&mut self, action: ViewAction) {
        match action {
            ViewAction::None => {}
            ViewAction::SwitchView(view_type) => {
                self.current_view = match view_type {
                    ViewType::Dashboard => CurrentView::Dashboard,
                    ViewType::Commands => CurrentView::Commands,
                    ViewType::Spec => CurrentView::Spec,
                };
            }
            ViewAction::RunCommand { name, args } => {
                self.command_runner.start_command(&name, &args);
                self.current_view = CurrentView::Commands;

                // Spawn the actual command
                let sender = self.event_sender.clone();
                let cmd_name = name.clone();
                let cmd_args = args.clone();

                tokio::spawn(async move {
                    let result =
                        crate::runners::cargo::run_cargo_command(&cmd_name, &cmd_args).await;
                    if let Some(sender) = sender {
                        match result {
                            Ok(output) => {
                                let _ = sender.send(Event::CommandDone {
                                    success: output.success,
                                    lines: output.lines,
                                });
                            }
                            Err(_) => {
                                let _ = sender.send(Event::CommandDone {
                                    success: false,
                                    lines: vec!["Command failed to execute".to_string()],
                                });
                            }
                        }
                    }
                });
            }
            ViewAction::RunSpecPhase {
                phase,
                command,
                options,
            } => {
                // Track running phase for auto-flow
                self.running_spec_phase = Some(phase.clone());

                // Start the spec phase command
                self.command_runner.start_command(&format!("spec:{}", phase), &[]);

                // Only switch to Commands view if NOT in auto-flow mode
                if !self.spec_view.auto_flow.active {
                    self.current_view = CurrentView::Commands;
                }

                let max_turns = options.max_turns;
                self.status_message = Some(format!(
                    "Running {} phase via Claude CLI (max {} turns)...",
                    phase, max_turns
                ));

                // Convert view options to CLI options
                let cli_options = crate::runners::cargo::ClaudeCliOptions {
                    max_turns: Some(options.max_turns),
                    skip_permissions: options.skip_permissions,
                    continue_session: options.continue_session,
                    session_id: options.session_id.clone(),
                    allowed_tools: options.allowed_tools.clone(),
                };

                // Spawn the Claude CLI command
                let sender = self.event_sender.clone();
                let cmd = command.clone();
                let phase_name = phase.clone();
                let is_auto_flow = self.spec_view.auto_flow.active;

                tokio::spawn(async move {
                    let result =
                        crate::runners::cargo::run_claude_command_with_options(&cmd, &cli_options)
                            .await;
                    if let Some(sender) = sender {
                        match result {
                            Ok(output) => {
                                if is_auto_flow {
                                    // Send phase completed event for auto-flow
                                    let _ = sender.send(Event::SpecPhaseCompleted {
                                        phase: phase_name,
                                        success: output.success,
                                        output: output.lines,
                                    });
                                } else {
                                    let _ = sender.send(Event::CommandDone {
                                        success: output.success,
                                        lines: output.lines,
                                    });
                                }
                            }
                            Err(e) => {
                                let error_lines = vec![
                                    format!("─ SDD Phase: {} ─", phase_name),
                                    format!("Failed to run Claude CLI: {}", e),
                                    String::new(),
                                    "Make sure 'claude' CLI is installed and available in PATH."
                                        .to_string(),
                                    "Install: npm install -g @anthropic-ai/claude-code".to_string(),
                                ];
                                if is_auto_flow {
                                    let _ = sender.send(Event::SpecPhaseCompleted {
                                        phase: phase_name,
                                        success: false,
                                        output: error_lines,
                                    });
                                } else {
                                    let _ = sender.send(Event::CommandDone {
                                        success: false,
                                        lines: error_lines,
                                    });
                                }
                            }
                        }
                    }
                });
            }
            ViewAction::StartWizard => {
                // Switch to spec view and activate wizard
                self.current_view = CurrentView::Spec;
                self.spec_view.wizard.active = true;
                self.spec_view.wizard.current_step = 0;
                self.status_message = Some("SDD Wizard started - follow the guided workflow".to_string());
            }
            ViewAction::ShowWorktrees => {
                // Run worktree list command
                self.command_runner.start_command("worktree-list", &[]);
                self.current_view = CurrentView::Commands;
                self.status_message = Some("Listing git worktrees...".to_string());

                // Spawn worktree list command
                let sender = self.event_sender.clone();
                tokio::spawn(async move {
                    // Run rscli worktree list via command
                    let result = tokio::process::Command::new("rscli")
                        .args(&["--cli", "worktree", "list", "--verbose"])
                        .output()
                        .await;

                    if let Some(sender) = sender {
                        match result {
                            Ok(output) => {
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                let combined = format!("{}{}", stdout, stderr);
                                let lines: Vec<String> = combined.lines().map(|s| s.to_string()).collect();

                                let _ = sender.send(Event::CommandDone {
                                    success: output.status.success(),
                                    lines,
                                });
                            }
                            Err(e) => {
                                let _ = sender.send(Event::CommandDone {
                                    success: false,
                                    lines: vec![format!("Failed to run worktree command: {}", e)],
                                });
                            }
                        }
                    }
                });
            }
            ViewAction::Quit => {
                self.running = false;
            }
        }
    }

    /// Handle command output events
    pub fn handle_command_output(&mut self, line: String) {
        // Add line to protocol parser for potential protocol messages
        self.protocol_parser.add_line(line.clone());

        // Try to parse protocol messages
        if let Some(msg) = self.protocol_parser.try_parse() {
            // Protocol message detected - handle it!
            self.handle_protocol_message(msg);
        }

        // Always add to command runner for display
        // (protocol markers will be filtered out later if needed)
        self.command_runner.add_output(line);
    }

    /// Handle command completion with output
    pub fn handle_command_done(&mut self, success: bool, lines: Vec<String>) {
        // Add all output lines
        for line in lines {
            self.command_runner.add_output(line);
        }
        self.command_runner.command_finished(success);
        self.status_message = Some(if success {
            "Command completed successfully".to_string()
        } else {
            "Command failed".to_string()
        });
    }

    /// Build rscli and install to ~/.local/bin
    pub fn run_update(&mut self) {
        self.status_message = Some("Building rscli (release)...".to_string());
        self.command_runner.start_command("update", &[]);
        self.current_view = CurrentView::Commands;

        let sender = self.event_sender.clone();

        tokio::spawn(async move {
            use std::process::Command;

            // Step 1: Build release
            let build_result = Command::new("cargo")
                .args(["build", "--release", "--bin", "rscli"])
                .output();

            match build_result {
                Ok(output) if output.status.success() => {
                    // Step 2: Copy to ~/.local/bin
                    let home = std::env::var("HOME").unwrap_or_default();
                    let target = format!("{home}/.local/bin/rscli");
                    let source = "target/release/rscli";

                    // Ensure directory exists
                    let _ = std::fs::create_dir_all(format!("{home}/.local/bin"));

                    match std::fs::copy(source, &target) {
                        Ok(_) => {
                            if let Some(sender) = sender {
                                let _ = sender.send(Event::CommandDone {
                                    success: true,
                                    lines: vec![
                                        "Build successful!".to_string(),
                                        format!("Installed to: {target}"),
                                        "Restart rscli to use the new version.".to_string(),
                                    ],
                                });
                            }
                        }
                        Err(e) => {
                            if let Some(sender) = sender {
                                let _ = sender.send(Event::CommandDone {
                                    success: false,
                                    lines: vec![format!("Failed to copy: {e}")],
                                });
                            }
                        }
                    }
                }
                Ok(output) => {
                    // Build failed
                    if let Some(sender) = sender {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let _ = sender.send(Event::CommandDone {
                            success: false,
                            lines: stderr.lines().map(String::from).collect(),
                        });
                    }
                }
                Err(e) => {
                    if let Some(sender) = sender {
                        let _ = sender.send(Event::CommandDone {
                            success: false,
                            lines: vec![format!("Failed to run cargo: {e}")],
                        });
                    }
                }
            }
        });
    }

    /// Handle key events when in input mode
    pub fn handle_key_event_in_input_mode(&mut self, key: KeyEvent) {
        if let Some(ref mut input) = self.text_input {
            match key.code {
                KeyCode::Char(c) => {
                    input.insert_char(c);
                }
                KeyCode::Backspace => {
                    input.delete_char();
                }
                KeyCode::Delete => {
                    input.delete_char_forward();
                }
                KeyCode::Left => {
                    input.move_cursor_left();
                }
                KeyCode::Right => {
                    input.move_cursor_right();
                }
                KeyCode::Home => {
                    input.move_cursor_home();
                }
                KeyCode::End => {
                    input.move_cursor_end();
                }
                KeyCode::Enter => {
                    let value = input.submit();
                    self.submit_user_input(value);
                    self.text_input = None;
                    self.input_mode = false;
                }
                KeyCode::Esc => {
                    input.cancel();
                    self.text_input = None;
                    self.input_mode = false;
                    self.status_message = Some("Input cancelled".to_string());
                }
                _ => {}
            }
        }
    }

    /// Submit user input back to the running process
    pub fn submit_user_input(&mut self, value: String) {
        // TODO: Send input back to Claude CLI
        // For now, just show a status message
        self.status_message = Some(format!("Submitted: {}", value));

        // In the future, this will send the input to the running Claude CLI process
        // via stdin or by re-running with the input as an argument
    }

    /// Handle protocol messages from Claude Code
    pub fn handle_protocol_message(&mut self, msg: ProtocolMessage) {
        match msg {
            ProtocolMessage::RequestInput {
                prompt,
                placeholder,
                next_action: _,
            } => {
                // Show input widget at bottom status bar
                let mut input = TextInput::new(prompt);
                if let Some(ph) = placeholder {
                    input.placeholder = ph;
                }
                self.text_input = Some(input);
                self.input_mode = true;
                self.status_message = Some("Input requested - type your response".to_string());
            }
            ProtocolMessage::PhaseCompleted {
                phase,
                next_phase,
                auto_continue,
            } => {
                // Update phase status in WorktreeView (if applicable)
                // For now, just show a message
                let mut message = format!("Phase '{}' completed", phase);
                if auto_continue && next_phase.is_some() {
                    let next = next_phase.unwrap();
                    message.push_str(&format!(" - auto-continuing to '{}'", next));
                    // TODO: Auto-start next phase
                }
                self.status_message = Some(message);
            }
            ProtocolMessage::DisplayInfo { message, details } => {
                // Show info message
                let mut full_message = message;
                if !details.is_empty() {
                    full_message.push_str(&format!(" ({})", details.join(", ")));
                }
                self.status_message = Some(full_message);
            }
        }
    }

    /// Handle spec phase completion for auto-flow mode
    pub fn handle_spec_phase_completed(&mut self, phase: String, success: bool, output: Vec<String>) {
        // Clear running phase
        self.running_spec_phase = None;

        // Also add output to command runner for reference
        for line in &output {
            self.command_runner.add_output(line.clone());
        }
        self.command_runner.command_finished(success);

        // Update spec view with phase completion
        self.spec_view.handle_phase_completed(phase.clone(), success, output);
        self.spec_view.output_scroll = 0; // Reset scroll for new output

        // Update WorktreeView phase status (if on Worktree tab)
        let status = if success {
            crate::tui::views::PhaseStatus::Completed
        } else {
            crate::tui::views::PhaseStatus::NeedsUpdate
        };
        self.worktree_view.update_phase_status(&phase, status);

        // Check for auto-flow continuation in WorktreeView
        if self.current_view == CurrentView::Worktree && self.worktree_view.auto_flow.active {
            if !self.worktree_view.auto_flow.is_complete() && success {
                // Auto-advance to next phase
                self.worktree_view.auto_flow.advance();
                if let Some(next_phase) = self.worktree_view.auto_flow.current_phase() {
                    self.status_message = Some(format!(
                        "{} phase completed - auto-continuing to {}",
                        phase, next_phase.display_name()
                    ));
                    // TODO: Auto-start next phase
                } else {
                    self.status_message = Some("Auto-flow workflow completed!".to_string());
                    self.worktree_view.auto_flow.reset();
                }
            } else {
                self.status_message = Some(if success {
                    "Phase completed - auto-flow paused".to_string()
                } else {
                    "Phase failed - auto-flow stopped".to_string()
                });
            }
        } else {
            // Normal status message for SpecView
            self.status_message = Some(if success {
                format!("{} phase completed - review and press Enter to continue", phase)
            } else {
                format!("{} phase failed - review output and press Enter to continue or Esc to stop", phase)
            });
        }
    }

    /// Refresh git worktree information
    fn refresh_git_info(&mut self) {
        use rscli_core::git::worktree;
        use crate::tui::event::{Event, WorktreeType};
        use tokio::time::{timeout, Duration};

        let sender = self.event_sender.clone();

        tokio::spawn(async move {
            // Helper to run git command with timeout
            async fn with_timeout<F, T>(f: F) -> Option<T>
            where
                F: std::future::Future<Output = rscli_core::Result<T>>,
            {
                match timeout(Duration::from_secs(5), f).await {
                    Ok(Ok(result)) => Some(result),
                    _ => None,
                }
            }

            // Try to get current worktree path
            let path = with_timeout(worktree::get_current_worktree()).await;

            // If we got a path, we're in a git repo
            let is_git_repo = path.is_some();

            if !is_git_repo {
                // Not a git repo, send event with minimal info
                if let Some(sender) = sender {
                    let _ = sender.send(Event::GitInfoUpdated {
                        branch: None,
                        worktree_path: None,
                        worktree_count: 0,
                        worktree_type: WorktreeType::NotGit,
                        is_git_repo: false,
                        error: None,
                    });
                }
                return;
            }

            // Get branch name
            let branch = with_timeout(worktree::get_current_branch()).await.flatten();

            // List all worktrees
            let worktrees = with_timeout(worktree::list_worktrees()).await;
            let count = worktrees.as_ref().map(|w| w.len()).unwrap_or(1);

            // Determine worktree type
            let wt_type = if let Some(ref b) = branch {
                if let Some(feature) = worktree::parse_feature_branch(b) {
                    WorktreeType::FeatureWorktree {
                        number: feature.number,
                        name: feature.name,
                    }
                } else {
                    WorktreeType::MainRepository
                }
            } else {
                WorktreeType::MainRepository
            };

            // Send the event
            if let Some(sender) = sender {
                let _ = sender.send(Event::GitInfoUpdated {
                    branch,
                    worktree_path: path,
                    worktree_count: count,
                    worktree_type: wt_type,
                    is_git_repo: true,
                    error: None,
                });
            }
        });
    }

    /// Handle git information update event
    pub fn handle_git_info_updated(
        &mut self,
        branch: Option<String>,
        worktree_path: Option<std::path::PathBuf>,
        worktree_count: usize,
        worktree_type: crate::tui::event::WorktreeType,
        is_git_repo: bool,
        error: Option<String>,
    ) {
        self.dashboard.git_branch = branch.clone().unwrap_or_else(|| "HEAD".to_string());
        self.dashboard.worktree_path = worktree_path;
        self.dashboard.worktree_count = worktree_count;
        self.dashboard.worktree_type = worktree_type.clone();
        self.dashboard.is_git_repo = is_git_repo;
        self.dashboard.git_error = error;
        self.dashboard.last_git_refresh = self.dashboard.tick_count;

        // Update worktree view based on worktree type
        match worktree_type {
            crate::tui::event::WorktreeType::FeatureWorktree { number, name } => {
                self.worktree_view.refresh_feature(number, name, branch);
            }
            _ => {
                self.worktree_view.clear_feature();
            }
        }
    }

    /// Tick update (for animations, status refresh, etc.)
    pub fn tick(&mut self) {
        self.worktree_view.tick();
        self.dashboard.tick();
        self.command_runner.tick();
        self.spec_view.tick();

        // Check if git refresh is needed
        if self.dashboard.should_refresh_git() {
            self.refresh_git_info();
        }
    }

    /// Copy current pane content to clipboard
    pub fn copy_current_pane(&mut self) {
        let (content, pane_name) = match self.current_view {
            CurrentView::Worktree => (self.worktree_view.get_focused_pane_text(), "current pane"),
            CurrentView::Dashboard => (self.dashboard.get_focused_pane_text(), "current pane"),
            CurrentView::Commands => (self.command_runner.get_focused_pane_text(), "current pane"),
            CurrentView::Spec => (self.spec_view.get_focused_pane_text(), "current pane"),
        };

        if content.is_empty() {
            self.status_message = Some(format!("No content in {} pane", pane_name));
            return;
        }

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(&content) {
                Ok(_) => {
                    let lines = content.lines().count();
                    self.status_message =
                        Some(format!("Copied {} pane ({} lines)", pane_name, lines));
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to copy: {}", e));
                }
            },
            Err(e) => {
                self.status_message = Some(format!("Clipboard error: {}", e));
            }
        }
    }

    /// Copy current tab with ANSI styling preserved
    pub fn copy_current_tab_styled(&mut self) {
        let (content, tab_name) = match self.current_view {
            CurrentView::Worktree => (self.worktree_view.get_styled_output(), "Worktree"),
            CurrentView::Dashboard => (self.dashboard.get_styled_output(), "Dashboard"),
            CurrentView::Commands => (self.command_runner.get_styled_output(), "Commands"),
            CurrentView::Spec => (self.spec_view.get_styled_output(), "Spec"),
        };

        if content.is_empty() {
            self.status_message = Some(format!("No content in {} tab", tab_name));
            return;
        }

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(&content) {
                Ok(_) => {
                    let lines = content.lines().count();
                    self.status_message =
                        Some(format!("Copied {} tab with styling ({} lines)", tab_name, lines));
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to copy: {}", e));
                }
            },
            Err(e) => {
                self.status_message = Some(format!("Clipboard error: {}", e));
            }
        }
    }

    /// Capture visual view as it appears on screen (with box-drawing characters)
    fn capture_visual_view(&self, terminal: &Terminal<CrosstermBackend<Stdout>>) -> AppResult<String> {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal as TestTerminal;

        // Get current terminal size
        let size = terminal.size()?;

        // Create a test backend with the same size
        let backend = TestBackend::new(size.width, size.height);
        let mut test_terminal = TestTerminal::new(backend)?;

        // Render to the test backend
        test_terminal.draw(|f| self.render(f))?;

        // Get the buffer content
        let buffer = test_terminal.backend().buffer().clone();
        let mut lines = Vec::new();

        // Extract characters from buffer line by line
        for y in 0..size.height {
            let mut line = String::new();
            for x in 0..size.width {
                let idx = (y * size.width + x) as usize;
                if idx < buffer.content.len() {
                    line.push_str(buffer.content[idx].symbol());
                }
            }
            // Trim trailing whitespace from this line only
            lines.push(line.trim_end().to_string());
        }

        // Join lines with newlines
        Ok(lines.join("\n"))
    }

    /// Run the TUI application
    pub fn run(&mut self) -> AppResult<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // Create event handler
        let event_handler = EventHandler::new(100); // 100ms tick rate
        self.event_sender = Some(event_handler.sender());

        // Main loop
        let result = self.main_loop(&mut terminal, &event_handler);

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        event_handler: &EventHandler,
    ) -> AppResult<()> {
        while self.running {
            // Draw UI
            terminal.draw(|frame| {
                self.render(frame);
            })?;

            // Check if visual copy was requested
            if self.copy_visual_view {
                self.copy_visual_view = false; // Reset flag

                // Capture the visual buffer
                match self.capture_visual_view(terminal) {
                    Ok(content) => {
                        // Copy to clipboard
                        match arboard::Clipboard::new() {
                            Ok(mut clipboard) => match clipboard.set_text(&content) {
                                Ok(_) => {
                                    let lines = content.lines().count();
                                    let tab_name = match self.current_view {
                                        CurrentView::Worktree => "Worktree",
                                        CurrentView::Dashboard => "Dashboard",
                                        CurrentView::Commands => "Commands",
                                        CurrentView::Spec => "Spec",
                                    };
                                    self.status_message = Some(format!(
                                        "Copied {} visual view ({} lines)",
                                        tab_name, lines
                                    ));
                                }
                                Err(e) => {
                                    self.status_message = Some(format!("Failed to copy: {}", e));
                                }
                            },
                            Err(e) => {
                                self.status_message = Some(format!("Clipboard error: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Failed to capture view: {}", e));
                    }
                }
            }

            // Handle events
            match event_handler.next()? {
                Event::Tick => self.tick(),
                Event::Key(key) => self.handle_key_event(key),
                Event::Mouse(_) => {} // Could add mouse support later
                Event::Resize(_, _) => {} // Terminal handles resize automatically
                Event::CommandOutput(line) => self.handle_command_output(line),
                Event::CommandDone { success, lines } => self.handle_command_done(success, lines),
                Event::SpecPhaseCompleted {
                    phase,
                    success,
                    output,
                } => {
                    self.handle_spec_phase_completed(phase, success, output);
                }
                Event::GitInfoUpdated {
                    branch,
                    worktree_path,
                    worktree_count,
                    worktree_type,
                    is_git_repo,
                    error,
                } => {
                    self.handle_git_info_updated(
                        branch,
                        worktree_path,
                        worktree_count,
                        worktree_type,
                        is_git_repo,
                        error,
                    );
                }
            }
        }
        Ok(())
    }

    /// Render the current view
    fn render(&self, frame: &mut ratatui::Frame) {
        use ratatui::layout::{Constraint, Direction, Layout};
        use ratatui::style::{Color, Style};
        use ratatui::widgets::{Block, Borders, Paragraph, Tabs};

        let size = frame.area();

        // Create main layout: tabs at top, content in middle, footer at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(2), // Footer (shortcuts + status)
            ])
            .split(size);

        // Render tabs
        let tab_titles = vec!["[1] Worktree", "[2] Commands", "[3] Spec-Kit", "[4] Dashboard"];
        let selected_tab = match self.current_view {
            CurrentView::Worktree => 0,
            CurrentView::Commands => 1,
            CurrentView::Spec => 2,
            CurrentView::Dashboard => 3,
        };
        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" rscli - Rust Station Dev Toolkit "),
            )
            .select(selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));
        frame.render_widget(tabs, chunks[0]);

        // Render current view
        match self.current_view {
            CurrentView::Worktree => self.worktree_view.render(frame, chunks[1]),
            CurrentView::Commands => self.command_runner.render(frame, chunks[1]),
            CurrentView::Spec => self.spec_view.render(frame, chunks[1]),
            CurrentView::Dashboard => self.dashboard.render(frame, chunks[1]),
        }

        // Render footer with shortcuts and status
        use ratatui::text::{Line, Span};
        use ratatui::style::Modifier;

        let footer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Shortcuts
                Constraint::Length(1), // Status message
            ])
            .split(chunks[2]);

        // Shortcuts bar (always visible)
        let shortcuts = Line::from(vec![
            Span::styled("[", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("/"),
            Span::styled("]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Switch Tab", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Switch Pane", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("y", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Copy", Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled("Y", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Copy+Style", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("U", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Update", Style::default().fg(Color::DarkGray)),
        ]);
        let shortcuts_bar = Paragraph::new(shortcuts);
        frame.render_widget(shortcuts_bar, footer_chunks[0]);

        // Status message bar OR input field
        if self.input_mode && self.text_input.is_some() {
            // Render input field
            if let Some(ref input) = self.text_input {
                frame.render_widget(input, footer_chunks[1]);
            }
        } else {
            // Render normal status message
            let status = self
                .status_message
                .as_deref()
                .unwrap_or("");
            let status_bar = Paragraph::new(status).style(Style::default().fg(Color::Cyan));
            frame.render_widget(status_bar, footer_chunks[1]);
        }
    }
}
