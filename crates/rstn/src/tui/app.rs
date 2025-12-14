//! Application state and main loop for the TUI

use crate::tui::claude_stream::{ClaudeStreamMessage, RscliStatus};
use crate::tui::event::{Event, EventHandler};
use crate::tui::protocol::{OutputParser, ProtocolMessage};
use crate::tui::views::{
    CommandRunner, Dashboard, SettingsView, SpecPhase, SpecView, View, ViewAction, ViewType,
    WorktreeView,
};
use crate::tui::widgets::{InputDialog, OptionPicker, TextInput};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::sync::mpsc;
use tracing::debug;

/// Result type for the app
pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Available views in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    Worktree,
    Settings,
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
    /// Settings view state
    pub settings_view: SettingsView,
    /// Command runner (internal, for running commands)
    pub command_runner: CommandRunner,
    /// Spec-driven development view state (internal)
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
    /// Text input widget for interactive user input (footer)
    pub text_input: Option<TextInput>,
    /// Input dialog for modal prompts (centered popup)
    pub input_dialog: Option<InputDialog>,
    /// Whether the app is in input mode (capturing text input)
    pub input_mode: bool,
    /// Option picker widget for structured choices
    pub option_picker: Option<OptionPicker>,
    /// Whether the app is in picker mode (selecting options)
    pub picker_mode: bool,
    /// Pending auto-continue: (next_phase, delay_ms)
    pub pending_auto_continue: Option<(String, u64)>,
    /// Path to restart executable (None = normal quit, Some = exec restart)
    pub restart_path: Option<String>,
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
            settings_view: SettingsView::new(),
            command_runner: CommandRunner::new(),
            spec_view: SpecView::new(),
            status_message: None,
            event_sender: None,
            running_spec_phase: None,
            copy_visual_view: false,
            protocol_parser: OutputParser::new(),
            text_input: None,
            input_dialog: None,
            input_mode: false,
            option_picker: None,
            picker_mode: false,
            pending_auto_continue: None,
            restart_path: None,
        }
    }

    /// Handle key events
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Debug logging for input mode investigation
        debug!(
            "handle_key_event: key={:?}, input_mode={}, input_dialog_exists={}",
            key.code,
            self.input_mode,
            self.input_dialog.is_some()
        );

        // Defensive check: Ensure input_mode and input_dialog state are consistent
        // If input_mode is true but no dialog exists, reset input_mode
        if self.input_mode && self.input_dialog.is_none() {
            debug!("State inconsistency detected: input_mode=true but input_dialog=None, resetting input_mode");
            self.input_mode = false;
        }
        // If input_dialog exists but input_mode is false, enable input_mode
        if self.input_dialog.is_some() && !self.input_mode {
            debug!("State inconsistency detected: input_dialog exists but input_mode=false, enabling input_mode");
            self.input_mode = true;
        }

        // If in input mode, handle input separately
        if self.input_mode {
            self.handle_key_event_in_input_mode(key);
            return;
        }

        // Global key bindings
        match key.code {
            // Quit on Ctrl+C or q (when not in command view with running command)
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                debug!("Quit triggered: Ctrl+C");
                self.running = false;
                return;
            }
            KeyCode::Char('q') if !self.worktree_view.is_running => {
                debug!("Quit triggered: 'q' key");
                self.running = false;
                return;
            }
            // Switch tabs/views with [ and ]
            KeyCode::Char('[') => {
                self.current_view = match self.current_view {
                    CurrentView::Worktree => CurrentView::Dashboard,
                    CurrentView::Settings => CurrentView::Worktree,
                    CurrentView::Dashboard => CurrentView::Settings,
                };
                self.status_message = Some(format!(
                    "Switched to {} view",
                    match self.current_view {
                        CurrentView::Worktree => "Worktree",
                        CurrentView::Settings => "Settings",
                        CurrentView::Dashboard => "Dashboard",
                    }
                ));
                return;
            }
            KeyCode::Char(']') => {
                self.current_view = match self.current_view {
                    CurrentView::Worktree => CurrentView::Settings,
                    CurrentView::Settings => CurrentView::Dashboard,
                    CurrentView::Dashboard => CurrentView::Worktree,
                };
                self.status_message = Some(format!(
                    "Switched to {} view",
                    match self.current_view {
                        CurrentView::Worktree => "Worktree",
                        CurrentView::Settings => "Settings",
                        CurrentView::Dashboard => "Dashboard",
                    }
                ));
                return;
            }
            // Update rstn: build and install to ~/.local/bin
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
                    CurrentView::Settings => {
                        // Settings view doesn't have panes
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
                self.current_view = CurrentView::Settings;
                return;
            }
            KeyCode::Char('3') => {
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
            CurrentView::Settings => self.settings_view.handle_key(key),
            CurrentView::Dashboard => self.dashboard.handle_key(key),
        };

        self.handle_view_action(action);
    }

    /// Handle actions returned from views
    /// Handle a view action (public for testing)
    pub fn handle_view_action(&mut self, action: ViewAction) {
        match action {
            ViewAction::None => {}
            ViewAction::SwitchView(view_type) => {
                self.current_view = match view_type {
                    ViewType::Dashboard => CurrentView::Dashboard,
                    ViewType::Commands => CurrentView::Worktree, // Commands now inline
                    ViewType::Spec => CurrentView::Worktree,     // Spec now inline
                };
            }
            ViewAction::RunCommand { name, args } => {
                // Keep command_runner for backwards compat, but don't switch views
                self.command_runner.start_command(&name, &args);
                // Commands now run inline - no view switch needed

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

                // Parse phase enum from name
                let spec_phase = SpecPhase::from_name(&phase);

                // Start inline output in WorktreeView
                if let Some(phase_enum) = spec_phase {
                    self.worktree_view.start_command(phase_enum, Some(&command));
                }

                let max_turns = options.max_turns;
                self.status_message = Some(format!(
                    "Running {} phase via Claude CLI (max {} turns)...",
                    phase, max_turns
                ));

                // Get session ID from WorktreeView (feature-specific)
                let session_id = self.worktree_view.get_session_id();

                // Convert view options to CLI options
                let cli_options = crate::runners::cargo::ClaudeCliOptions {
                    max_turns: Some(options.max_turns),
                    skip_permissions: options.skip_permissions,
                    continue_session: options.continue_session,
                    session_id: session_id.or(options.session_id.clone()),
                    allowed_tools: options.allowed_tools.clone(),
                };

                // Spawn the Claude CLI command
                let sender = self.event_sender.clone();
                let cmd = command.clone();
                let phase_name = phase.clone();
                let _is_auto_flow = self.worktree_view.auto_flow.active;

                tokio::spawn(async move {
                    // Use streaming function with sender for real-time output
                    let result = crate::runners::cargo::run_claude_command_streaming(
                        &cmd,
                        &cli_options,
                        sender.clone(),
                    )
                    .await;

                    if let Some(sender) = sender {
                        match result {
                            Ok(claude_result) => {
                                // Send completion event with parsed status
                                let _ = sender.send(Event::ClaudeCompleted {
                                    phase: phase_name,
                                    success: claude_result.success,
                                    session_id: claude_result.session_id,
                                    status: claude_result.status,
                                });
                            }
                            Err(e) => {
                                // Send error as ClaudeCompleted with error status
                                let error_status = RscliStatus {
                                    status: "error".to_string(),
                                    prompt: None,
                                    message: Some(format!(
                                        "Failed to run Claude CLI: {}. Make sure 'claude' CLI is installed.",
                                        e
                                    )),
                                };
                                let _ = sender.send(Event::ClaudeCompleted {
                                    phase: phase_name,
                                    success: false,
                                    session_id: None,
                                    status: Some(error_status),
                                });
                            }
                        }
                    }
                });
            }
            ViewAction::RunEnhancedCommit => {
                // Run enhanced commit workflow with security scanning
                self.status_message =
                    Some("Scanning staged changes for security issues...".to_string());

                let sender = self.event_sender.clone();
                tokio::spawn(async move {
                    let result = rstn_core::git::interactive_commit().await;

                    if let Some(sender) = sender {
                        match result {
                            Ok(rstn_core::CommitResult::Blocked(scan)) => {
                                let _ = sender.send(Event::CommitBlocked { scan });
                            }
                            Ok(rstn_core::CommitResult::ReadyToCommit {
                                message,
                                warnings,
                                sensitive_files,
                            }) => {
                                let _ = sender.send(Event::CommitReady {
                                    message,
                                    warnings,
                                    sensitive_files,
                                });
                            }
                            Err(e) => {
                                let _ = sender.send(Event::CommitError {
                                    error: e.to_string(),
                                });
                            }
                        }
                    }
                });
            }
            ViewAction::StartWizard => {
                // Start wizard mode in worktree view
                self.worktree_view.auto_flow.active = true;
                self.status_message =
                    Some("SDD Workflow started - phases will run sequentially".to_string());
            }
            ViewAction::ShowWorktrees => {
                // Just show a status message - worktrees are shown in worktree view
                self.status_message = Some("Worktree info shown in Worktree tab".to_string());

                // Spawn worktree list command
                let sender = self.event_sender.clone();
                tokio::spawn(async move {
                    // Run rstn worktree list via command
                    let result = tokio::process::Command::new("rstn")
                        .args(&["--cli", "worktree", "list", "--verbose"])
                        .output()
                        .await;

                    if let Some(sender) = sender {
                        match result {
                            Ok(output) => {
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                let combined = format!("{}{}", stdout, stderr);
                                let lines: Vec<String> =
                                    combined.lines().map(|s| s.to_string()).collect();

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
                debug!("Quit triggered: ViewAction::Quit");
                self.running = false;
            }
            ViewAction::RequestInput {
                prompt,
                placeholder,
            } => {
                debug!(
                    "ViewAction::RequestInput received: prompt='{}', placeholder={:?}",
                    prompt, placeholder
                );

                // Detect if this is the Specify phase (requires multiline input)
                let is_multiline = prompt.contains("feature description");

                let mut dialog = if is_multiline {
                    InputDialog::new_multiline("Input Required", prompt, 10)
                } else {
                    InputDialog::new("Input Required", prompt)
                };

                if let Some(ph) = placeholder {
                    dialog = dialog.placeholder(ph);
                }
                self.input_dialog = Some(dialog);
                self.input_mode = true;

                // State validation: Both must be set for input to work
                debug_assert!(
                    self.input_mode && self.input_dialog.is_some(),
                    "RequestInput: input_mode and input_dialog must both be set"
                );

                debug!(
                    "RequestInput handled: input_mode={}, input_dialog_exists={}, is_multiline={}",
                    self.input_mode,
                    self.input_dialog.is_some(),
                    is_multiline
                );
            }
            ViewAction::RunGitCommand(_) => {
                // Git commands are handled via handle_git_command() which returns
                // ViewAction::RunCommand, so this case should never be reached
                // but we handle it for exhaustiveness
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

        // Add to WorktreeView's inline output if it's showing output
        if self.worktree_view.is_showing_output() {
            self.worktree_view.add_output(line.clone());
        }

        // Also add to command runner for backwards compatibility
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

    /// Build rstn and install to ~/.local/bin
    pub fn run_update(&mut self) {
        self.status_message = Some("Building rstn (release)...".to_string());
        self.command_runner.start_command("update", &[]);
        // Commands now run inline, no view switch needed

        let sender = self.event_sender.clone();

        tokio::spawn(async move {
            use std::process::Command;

            // Step 1: Build release
            let build_result = Command::new("cargo")
                .args(["build", "--release", "--bin", "rstn"])
                .output();

            match build_result {
                Ok(output) if output.status.success() => {
                    // Step 2: Copy to ~/.local/bin
                    let home = std::env::var("HOME").unwrap_or_default();
                    let target = format!("{home}/.local/bin/rstn");
                    let source = "target/release/rstn";

                    // Ensure directory exists
                    let _ = std::fs::create_dir_all(format!("{home}/.local/bin"));

                    match std::fs::copy(source, &target) {
                        Ok(_) => {
                            if let Some(sender) = sender {
                                let _ = sender.send(Event::UpdateCompleted {
                                    installed_path: target.clone(),
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

    /// Prepare for restart: validate binary and set restart flag
    /// Returns true if restart should happen, false to continue normally
    fn prepare_restart(&mut self, installed_path: &str) -> bool {
        // Verify installed binary exists
        if !std::path::Path::new(installed_path).exists() {
            self.status_message = Some(format!(
                "Warning: {} not found, restart cancelled",
                installed_path
            ));
            return false;
        }

        // Show brief message before restart
        self.status_message = Some("Restarting with new version...".to_string());

        // Store restart path for post-cleanup execution
        self.restart_path = Some(installed_path.to_string());

        // Set running=false to exit main loop cleanly
        self.running = false;

        true
    }

    /// Handle key events when in input mode
    pub fn handle_key_event_in_input_mode(&mut self, key: KeyEvent) {
        debug!(
            "handle_key_event_in_input_mode: key={:?}, input_dialog_exists={}, text_input_exists={}",
            key.code,
            self.input_dialog.is_some(),
            self.text_input.is_some()
        );

        // Prioritize input_dialog (modal) over text_input (footer)
        if let Some(ref mut dialog) = self.input_dialog {
            match key.code {
                KeyCode::Char(c) => {
                    debug!("Inserting char '{}' into input dialog", c);
                    dialog.insert_char(c);
                }
                KeyCode::Backspace => {
                    dialog.delete_char();
                }
                KeyCode::Left => {
                    dialog.move_cursor_left();
                }
                KeyCode::Right => {
                    dialog.move_cursor_right();
                }
                KeyCode::Up => {
                    if dialog.is_multiline() {
                        dialog.move_cursor_up();
                    }
                }
                KeyCode::Down => {
                    if dialog.is_multiline() {
                        dialog.move_cursor_down();
                    }
                }
                KeyCode::Home => {
                    dialog.move_cursor_start();
                }
                KeyCode::End => {
                    dialog.move_cursor_end();
                }
                KeyCode::Enter => {
                    if dialog.is_multiline() {
                        // Multiline mode: Alt+Enter submits, Enter creates newline
                        if key.modifiers.contains(KeyModifiers::ALT) {
                            let value = dialog.input.get_multiline_value();
                            self.submit_user_input(value);
                            self.input_dialog = None;
                            self.input_mode = false;
                        } else {
                            dialog.insert_newline();
                        }
                    } else {
                        // Single-line mode: Enter submits
                        let value = dialog.value().to_string();
                        self.submit_user_input(value);
                        self.input_dialog = None;
                        self.input_mode = false;
                    }
                }
                KeyCode::Esc => {
                    self.input_dialog = None;
                    self.input_mode = false;
                    self.worktree_view.pending_follow_up = false;
                    self.status_message = Some("Input cancelled".to_string());
                }
                _ => {}
            }
        } else if let Some(ref mut input) = self.text_input {
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
        // Check if this is a commit message
        if self.worktree_view.pending_commit_message.is_some() {
            self.worktree_view.pending_commit_message = None;

            // Execute commit
            self.handle_view_action(ViewAction::RunCommand {
                name: "git".to_string(),
                args: vec!["commit".to_string(), "-m".to_string(), value],
            });
            return;
        }

        // Check for pending git command (non-commit commands)
        if let Some(git_cmd) = self.worktree_view.pending_git_command.take() {
            use crate::tui::views::GitCommand;
            match git_cmd {
                GitCommand::Commit => {
                    self.handle_view_action(ViewAction::RunCommand {
                        name: "git".to_string(),
                        args: vec!["commit".to_string(), "-m".to_string(), value],
                    });
                }
                GitCommand::Rebase => {
                    self.handle_view_action(ViewAction::RunCommand {
                        name: "git".to_string(),
                        args: vec!["rebase".to_string(), value],
                    });
                }
                _ => {}
            }
            return;
        }

        // Check if this is a follow-up response to Claude's question
        if self.worktree_view.pending_follow_up {
            // Resume conversation with user response
            self.worktree_view.pending_follow_up = false;
            let session_id = self.worktree_view.active_session_id.clone();

            // Build options with session ID for resume
            let mut options = self.worktree_view.get_claude_options();
            options.session_id = session_id;

            // User's response becomes the prompt, resumed via --resume
            self.handle_view_action(ViewAction::RunSpecPhase {
                phase: "follow-up".to_string(),
                command: value,
                options,
            });
        } else if let Some(phase) = self.worktree_view.pending_input_phase.take() {
            // Initial phase input - construct command with user input
            let command = format!("{} {}", phase.command(), value);
            let options = self.worktree_view.get_claude_options();

            // Run the phase with user input
            self.handle_view_action(ViewAction::RunSpecPhase {
                phase: phase.name().to_string(),
                command,
                options,
            });
        } else {
            // Fallback: just show a status message
            self.status_message = Some(format!("Submitted: {}", value));
        }
    }

    /// Handle protocol messages from Claude Code
    pub fn handle_protocol_message(&mut self, msg: ProtocolMessage) {
        match msg {
            ProtocolMessage::RequestInput {
                prompt,
                placeholder,
                next_action: _,
            } => {
                // Use centered dialog for better UX
                let mut dialog = InputDialog::new("Input Required", prompt);
                if let Some(ph) = placeholder {
                    dialog = dialog.placeholder(ph);
                }
                self.input_dialog = Some(dialog);
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
            ProtocolMessage::SelectOption {
                prompt,
                options,
                multi_select,
                default,
            } => {
                // Show option picker widget
                let mut picker = if multi_select {
                    OptionPicker::with_multi_select(prompt, options)
                } else {
                    OptionPicker::new(prompt, options)
                };
                if let Some(ref default_id) = default {
                    picker.set_default(default_id);
                }
                self.option_picker = Some(picker);
                self.picker_mode = true;
                self.status_message = Some("Select an option".to_string());
            }
            ProtocolMessage::AutoContinue {
                next_phase,
                delay_ms,
                message,
            } => {
                // Show message and schedule next phase
                if let Some(msg) = message {
                    self.status_message = Some(msg);
                }
                // Schedule next phase start after delay
                self.pending_auto_continue = Some((next_phase, delay_ms));
            }
            ProtocolMessage::Confirm { prompt, default } => {
                // Show confirmation dialog as option picker with Yes/No
                let options = vec![
                    crate::tui::protocol::SelectOptionItem {
                        id: "yes".to_string(),
                        label: "Yes".to_string(),
                        description: None,
                    },
                    crate::tui::protocol::SelectOptionItem {
                        id: "no".to_string(),
                        label: "No".to_string(),
                        description: None,
                    },
                ];
                let mut picker = OptionPicker::new(prompt, options);
                picker.set_default(if default { "yes" } else { "no" });
                self.option_picker = Some(picker);
                self.picker_mode = true;
            }
            ProtocolMessage::Progress {
                phase,
                step,
                total_steps,
                message,
            } => {
                // Update progress in WorktreeView
                self.worktree_view
                    .update_progress(&phase, step, total_steps, &message);
                self.status_message = Some(format!("[{}/{}] {}", step, total_steps, message));
            }
            ProtocolMessage::SessionInfo {
                session_id,
                feature,
            } => {
                // Save session ID
                self.worktree_view.active_session_id = Some(session_id.clone());
                if let Some(feat) = feature {
                    let _ = crate::session::save_session_id(&feat, &session_id);
                    self.status_message = Some(format!("Session saved for feature {}", feat));
                }
            }
        }
    }

    /// Handle spec phase completion for auto-flow mode
    pub fn handle_spec_phase_completed(
        &mut self,
        phase: String,
        success: bool,
        output: Vec<String>,
    ) {
        // Clear running phase
        self.running_spec_phase = None;

        // Add output to WorktreeView's inline output panel
        for line in &output {
            self.worktree_view.add_output(line.clone());
        }
        self.worktree_view.command_done();

        // Also add output to command runner for reference (kept for backwards compat)
        for line in &output {
            self.command_runner.add_output(line.clone());
        }
        self.command_runner.command_finished(success);

        // Update spec view with phase completion
        self.spec_view
            .handle_phase_completed(phase.clone(), success, output);
        self.spec_view.output_scroll = 0; // Reset scroll for new output

        // Update WorktreeView phase status
        let status = if success {
            crate::tui::views::PhaseStatus::Completed
        } else {
            crate::tui::views::PhaseStatus::NeedsUpdate
        };
        self.worktree_view.update_phase_status(&phase, status);

        // Check for auto-flow continuation in WorktreeView
        if self.worktree_view.auto_flow.active {
            if !self.worktree_view.auto_flow.is_complete() && success {
                // Auto-advance to next phase
                self.worktree_view.auto_flow.advance();
                if let Some(next_phase) = self.worktree_view.auto_flow.current_phase() {
                    self.status_message = Some(format!(
                        "{} phase completed - auto-continuing to {}",
                        phase,
                        next_phase.display_name()
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
            // Normal status message
            self.status_message = Some(if success {
                format!("{} phase completed - press Esc to dismiss output", phase)
            } else {
                format!("{} phase failed - press Esc to dismiss output", phase)
            });
        }
    }

    /// Handle Claude streaming JSON message (real-time output)
    fn handle_claude_stream(&mut self, msg: ClaudeStreamMessage) {
        // Display assistant messages in output panel (strip status block)
        if msg.msg_type == "assistant" {
            if let Some(text) = msg.get_display_text() {
                for line in text.lines() {
                    self.worktree_view.add_output(line.to_string());
                }
            }
        }
    }

    /// Handle Claude command completed with parsed status
    fn handle_claude_completed(
        &mut self,
        phase: String,
        success: bool,
        session_id: Option<String>,
        status: Option<RscliStatus>,
    ) {
        // Save session ID for this feature
        if let Some(sid) = session_id {
            self.worktree_view.active_session_id = Some(sid.clone());
            if let Some(ref info) = self.worktree_view.feature_info {
                let _ = crate::session::save_session_id(&info.number, &sid);
            }
        }

        // Handle based on parsed JSON status
        if let Some(status) = status {
            match status.status.as_str() {
                "needs_input" => {
                    // Use the prompt from JSON, or fallback
                    let prompt = status
                        .prompt
                        .unwrap_or_else(|| "Enter your response:".to_string());
                    self.worktree_view.pending_follow_up = true;
                    // Use centered input dialog for better UX
                    self.input_dialog = Some(InputDialog::new("Claude Input", prompt));
                    self.input_mode = true;
                    self.status_message = Some("Waiting for your response...".to_string());
                }
                "error" => {
                    let msg = status
                        .message
                        .unwrap_or_else(|| "Unknown error".to_string());
                    self.worktree_view.command_done();
                    self.status_message = Some(format!("{} error: {}", phase, msg));
                }
                "completed" | _ => {
                    self.worktree_view.command_done();
                    self.status_message = Some(format!("{} phase completed", phase));
                }
            }
        } else {
            // No status block - use heuristic detection
            // Check if the last non-empty output line looks like a question
            let needs_input = self
                .worktree_view
                .output_lines
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .map(|line| {
                    let text = line.trim().to_lowercase();
                    text.ends_with('?')
                        || text.contains("please describe")
                        || text.contains("what feature")
                        || text.contains("please provide")
                        || text.contains("could you")
                        || text.contains("would you like")
                })
                .unwrap_or(false);

            if needs_input && self.worktree_view.active_session_id.is_some() {
                // Looks like Claude asked a question - prompt for input
                self.worktree_view.pending_follow_up = true;
                // Use centered input dialog for better UX
                self.input_dialog = Some(InputDialog::new("Claude Input", "Enter your response:"));
                self.input_mode = true;
                self.status_message = Some("Claude is waiting for your input...".to_string());
            } else {
                // Truly completed
                self.worktree_view.command_done();
                self.status_message = Some(format!("{} phase finished", phase));
            }
        }

        self.running_spec_phase = None;
    }

    /// Refresh git worktree information
    fn refresh_git_info(&mut self) {
        use crate::tui::event::{Event, WorktreeType};
        use rstn_core::git::worktree;
        use tokio::time::{timeout, Duration};

        let sender = self.event_sender.clone();

        tokio::spawn(async move {
            // Helper to run git command with timeout
            async fn with_timeout<F, T>(f: F) -> Option<T>
            where
                F: std::future::Future<Output = rstn_core::Result<T>>,
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
            CurrentView::Settings => ("".to_string(), "settings"), // Settings doesn't have copyable panes
            CurrentView::Dashboard => (self.dashboard.get_focused_pane_text(), "current pane"),
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
            CurrentView::Settings => ("".to_string(), "Settings"), // Settings doesn't have styled output
            CurrentView::Dashboard => (self.dashboard.get_styled_output(), "Dashboard"),
        };

        if content.is_empty() {
            self.status_message = Some(format!("No content in {} tab", tab_name));
            return;
        }

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(&content) {
                Ok(_) => {
                    let lines = content.lines().count();
                    self.status_message = Some(format!(
                        "Copied {} tab with styling ({} lines)",
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

    /// Capture visual view as it appears on screen (with box-drawing characters)
    fn capture_visual_view(
        &self,
        terminal: &Terminal<CrosstermBackend<Stdout>>,
    ) -> AppResult<String> {
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
        debug!("App::run() starting");

        // Setup terminal
        debug!("enable_raw_mode()...");
        enable_raw_mode()?;
        debug!("enable_raw_mode() OK");

        let mut stdout = stdout();
        debug!("EnterAlternateScreen...");
        execute!(stdout, EnterAlternateScreen)?;
        debug!("EnterAlternateScreen OK");

        debug!("Creating CrosstermBackend...");
        let backend = CrosstermBackend::new(stdout);
        debug!("CrosstermBackend OK");

        debug!("Creating Terminal...");
        let mut terminal = Terminal::new(backend)?;
        debug!("Terminal OK");

        debug!("terminal.clear()...");
        terminal.clear()?;
        debug!("terminal.clear() OK");

        // Create event handler
        debug!("Creating EventHandler...");
        let event_handler = EventHandler::new(100); // 100ms tick rate
        debug!("EventHandler OK");

        self.event_sender = Some(event_handler.sender());

        // Main loop
        debug!("Entering main_loop...");
        let result = self.main_loop(&mut terminal, &event_handler);
        debug!("main_loop returned: {:?}", result.as_ref().map(|_| "Ok"));

        // Restore terminal
        debug!("Restoring terminal...");
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        debug!("Terminal restored");

        // Check if we should restart
        if let Some(restart_path) = &self.restart_path {
            debug!("Restart requested: {}", restart_path);
            self.exec_restart(restart_path)?;
            // If exec_restart returns, it failed - fall through to normal exit
        }

        result
    }

    /// Execute restart using Unix exec syscall
    /// This replaces the current process with the new binary
    fn exec_restart(&self, binary_path: &str) -> AppResult<()> {
        use std::ffi::CString;

        debug!("exec_restart: binary_path={}", binary_path);

        // Get current command-line arguments
        let args: Vec<String> = std::env::args().collect();
        debug!("exec_restart: original args: {:?}", args);

        // Convert binary path to CString for exec
        let path_cstr =
            CString::new(binary_path).map_err(|e| anyhow::anyhow!("Invalid path: {}", e))?;

        // Convert arguments to CStrings
        // First arg should be the program name (convention)
        let mut exec_args = vec![path_cstr.clone()];

        // Add remaining arguments (skip argv[0])
        for arg in args.iter().skip(1) {
            exec_args.push(
                CString::new(arg.as_str()).map_err(|e| anyhow::anyhow!("Invalid arg: {}", e))?,
            );
        }

        debug!("exec_restart: exec_args count={}", exec_args.len());

        // Perform exec - this replaces current process
        // If it returns, it failed
        #[cfg(unix)]
        {
            // Convert CStrings to raw pointers for execv
            // execv expects a NULL-terminated array of char pointers
            let mut argv: Vec<*const libc::c_char> =
                exec_args.iter().map(|s| s.as_ptr()).collect();
            argv.push(std::ptr::null()); // NULL-terminate the array

            unsafe {
                libc::execv(path_cstr.as_ptr(), argv.as_ptr());
            }

            // If we get here, exec failed
            let err = std::io::Error::last_os_error();
            debug!("exec_restart: FAILED: {}", err);
            Err(anyhow::anyhow!("Failed to exec: {}", err).into())
        }

        #[cfg(not(unix))]
        {
            debug!("exec_restart: Not on Unix, cannot exec");
            Err(anyhow::anyhow!("Restart only supported on Unix systems").into())
        }
    }

    fn main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        event_handler: &EventHandler,
    ) -> AppResult<()> {
        debug!("main_loop: starting, running={}", self.running);
        let mut iteration = 0;
        while self.running {
            iteration += 1;
            debug!("main_loop iteration {}: drawing UI", iteration);
            // Draw UI
            terminal.draw(|frame| {
                self.render(frame);
            })?;
            debug!("main_loop iteration {}: UI drawn", iteration);

            // Check if visual copy was requested
            if self.copy_visual_view {
                debug!("main_loop iteration {}: processing visual copy", iteration);
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
                                        CurrentView::Settings => "Settings",
                                        CurrentView::Dashboard => "Dashboard",
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
            debug!("main_loop iteration {}: waiting for event", iteration);
            match event_handler.next()? {
                Event::Tick => {
                    debug!("main_loop iteration {}: Event::Tick", iteration);
                    self.tick();
                }
                Event::Key(key) => {
                    debug!("main_loop iteration {}: Event::Key({:?})", iteration, key);
                    self.handle_key_event(key);
                }
                Event::Mouse(_) => {
                    debug!("main_loop iteration {}: Event::Mouse", iteration);
                } // Could add mouse support later
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
                Event::ClaudeStream(msg) => {
                    self.handle_claude_stream(msg);
                }
                Event::ClaudeCompleted {
                    phase,
                    success,
                    session_id,
                    status,
                } => {
                    debug!("main_loop iteration {}: Event::ClaudeCompleted", iteration);
                    self.handle_claude_completed(phase, success, session_id, status);
                }
                Event::CommitStarted => {
                    self.status_message = Some("Commit workflow started...".to_string());
                }
                Event::CommitBlocked { scan } => {
                    self.show_commit_blocked_dialog(scan);
                }
                Event::CommitReady {
                    message,
                    warnings,
                    sensitive_files,
                } => {
                    // Store in worktree view
                    self.worktree_view.pending_commit_message = Some(message.clone());
                    self.worktree_view.commit_warnings = warnings.clone();

                    // Show editable dialog
                    self.input_dialog = Some(
                        InputDialog::with_description(
                            "Commit Changes",
                            Self::format_warnings(&warnings, &sensitive_files),
                            "Message:",
                        )
                        .placeholder(message),
                    );
                    self.input_mode = true;
                }
                Event::CommitCompleted { success, output } => {
                    self.worktree_view.add_output(output);
                    if success {
                        self.worktree_view
                            .add_output("✓ Commit successful".to_string());
                        self.status_message = Some("Commit successful!".to_string());
                    } else {
                        self.status_message =
                            Some("Commit failed - see output for details".to_string());
                    }
                }
                Event::CommitError { error } => {
                    self.worktree_view
                        .add_output(format!("❌ Error: {}", error));
                    self.status_message = Some(format!("Commit error: {}", error));
                }
                Event::UpdateCompleted { installed_path } => {
                    // Show success message
                    self.worktree_view
                        .add_output("✓ Build successful!".to_string());
                    self.worktree_view
                        .add_output(format!("Installed to: {}", installed_path));
                    self.worktree_view.add_output("Restarting...".to_string());

                    // Prepare for restart
                    if self.prepare_restart(&installed_path) {
                        return Ok(()); // Exit loop to trigger restart
                    }
                }
            }
            debug!(
                "main_loop iteration {}: event handled, running={}",
                iteration, self.running
            );
        }
        debug!("main_loop: exited (running={})", self.running);
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
        let tab_titles = vec!["[1] Worktree", "[2] Settings", "[3] Dashboard"];
        let selected_tab = match self.current_view {
            CurrentView::Worktree => 0,
            CurrentView::Settings => 1,
            CurrentView::Dashboard => 2,
        };
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title(format!(
                " rstn {} - Rustation Dev Toolkit ",
                crate::version::short_version()
            )))
            .select(selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));
        frame.render_widget(tabs, chunks[0]);

        // Render current view
        match self.current_view {
            CurrentView::Worktree => self.worktree_view.render(frame, chunks[1]),
            CurrentView::Settings => self.settings_view.render(frame, chunks[1]),
            CurrentView::Dashboard => self.dashboard.render(frame, chunks[1]),
        }

        // Render footer with shortcuts and status
        use ratatui::style::Modifier;
        use ratatui::text::{Line, Span};

        let footer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Shortcuts
                Constraint::Length(1), // Status message
            ])
            .split(chunks[2]);

        // Shortcuts bar (always visible)
        let shortcuts = Line::from(vec![
            Span::styled(
                "[",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/"),
            Span::styled(
                "]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Switch Tab", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Switch Pane", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                "y",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Copy", Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(
                "Y",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Copy+Style", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                "q",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                "U",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Update", Style::default().fg(Color::DarkGray)),
        ]);
        let shortcuts_bar = Paragraph::new(shortcuts);
        frame.render_widget(shortcuts_bar, footer_chunks[0]);

        // Status message bar OR input field (footer input, not dialog)
        if self.input_mode && self.text_input.is_some() && self.input_dialog.is_none() {
            // Render footer input field (only if no dialog is open)
            if let Some(ref input) = self.text_input {
                frame.render_widget(input, footer_chunks[1]);
            }
        } else {
            // Render normal status message
            let status = self.status_message.as_deref().unwrap_or("");
            let status_bar = Paragraph::new(status).style(Style::default().fg(Color::Cyan));
            frame.render_widget(status_bar, footer_chunks[1]);
        }

        // Render input dialog as overlay (on top of everything)
        if let Some(ref dialog) = self.input_dialog {
            dialog.render(size, frame.buffer_mut());
        }
    }

    /// Show commit blocked dialog with security details
    fn show_commit_blocked_dialog(&mut self, scan: rstn_core::SecurityScanResult) {
        use rstn_core::Severity;

        let details = scan
            .warnings
            .iter()
            .filter(|w| matches!(w.severity, Severity::Critical))
            .map(|w| format!("{}:{} - {}", w.file_path, w.line_number, w.message))
            .collect::<Vec<_>>()
            .join("\n");

        self.worktree_view
            .add_output("❌ COMMIT BLOCKED".to_string());
        self.worktree_view.add_output("".to_string());
        self.worktree_view
            .add_output("Critical security issues detected:".to_string());
        for line in details.lines() {
            self.worktree_view.add_output(format!("  {}", line));
        }
        self.status_message = Some("Commit blocked due to security issues".to_string());
    }

    /// Format warnings and sensitive files for display
    fn format_warnings(
        warnings: &[rstn_core::SecurityWarning],
        sensitive_files: &[rstn_core::SensitiveFile],
    ) -> String {
        let mut desc = String::new();

        if !warnings.is_empty() {
            desc.push_str("⚠️ Security Warnings:\n");
            for w in warnings.iter().take(3) {
                desc.push_str(&format!(
                    "  • {}:{} - {}\n",
                    w.file_path, w.line_number, w.message
                ));
            }
            if warnings.len() > 3 {
                desc.push_str(&format!("  ... and {} more\n", warnings.len() - 3));
            }
            desc.push('\n');
        }

        if !sensitive_files.is_empty() {
            desc.push_str("📁 Sensitive Files:\n");
            for f in sensitive_files.iter().take(3) {
                desc.push_str(&format!("  • {} ({})\n", f.path, f.reason));
            }
        }

        desc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    // Helper to create key events
    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn key_event_with_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    // T024: Test that RequestInput sets input_mode to true
    #[test]
    fn test_request_input_sets_input_mode() {
        let mut app = App::new();

        // Verify initial state
        assert!(!app.input_mode, "input_mode should be false initially");
        assert!(
            app.input_dialog.is_none(),
            "input_dialog should be None initially"
        );

        // Simulate ViewAction::RequestInput
        app.handle_view_action(ViewAction::RequestInput {
            prompt: "Enter feature description:".to_string(),
            placeholder: Some("e.g., A new login system".to_string()),
        });

        // Verify input mode is enabled
        assert!(
            app.input_mode,
            "input_mode should be true after RequestInput"
        );
    }

    // T025: Test that RequestInput creates an input dialog
    #[test]
    fn test_request_input_creates_dialog() {
        let mut app = App::new();

        // Simulate ViewAction::RequestInput
        app.handle_view_action(ViewAction::RequestInput {
            prompt: "Enter test input:".to_string(),
            placeholder: None,
        });

        // Verify dialog is created
        assert!(
            app.input_dialog.is_some(),
            "input_dialog should be Some after RequestInput"
        );

        let dialog = app.input_dialog.as_ref().unwrap();
        assert_eq!(dialog.title, "Input Required");
    }

    // T026: Test that key events in input mode route to input handler
    #[test]
    fn test_key_event_routes_to_input_mode_handler() {
        let mut app = App::new();

        // Setup input mode
        app.handle_view_action(ViewAction::RequestInput {
            prompt: "Enter:".to_string(),
            placeholder: None,
        });

        assert!(app.input_mode);
        assert!(app.input_dialog.is_some());

        // Send character key events
        app.handle_key_event(key_event(KeyCode::Char('h')));
        app.handle_key_event(key_event(KeyCode::Char('i')));

        // Verify characters were inserted into dialog
        let dialog = app.input_dialog.as_ref().unwrap();
        assert_eq!(dialog.value(), "hi");
    }

    // T027: Test that Enter submits single-line input
    #[test]
    fn test_enter_submits_single_line_input() {
        let mut app = App::new();

        // Create a single-line (non-multiline) dialog
        app.handle_view_action(ViewAction::RequestInput {
            prompt: "Enter branch name:".to_string(), // Not "feature description", so single-line
            placeholder: None,
        });

        // Type some text
        app.handle_key_event(key_event(KeyCode::Char('t')));
        app.handle_key_event(key_event(KeyCode::Char('e')));
        app.handle_key_event(key_event(KeyCode::Char('s')));
        app.handle_key_event(key_event(KeyCode::Char('t')));

        // Verify it's not multiline
        let dialog = app.input_dialog.as_ref().unwrap();
        assert!(!dialog.is_multiline(), "Should be single-line dialog");

        // Submit with Enter
        app.handle_key_event(key_event(KeyCode::Enter));

        // After submit, input_mode should be false and dialog cleared
        assert!(!app.input_mode, "input_mode should be false after submit");
        assert!(
            app.input_dialog.is_none(),
            "input_dialog should be None after submit"
        );
    }

    // T028: Test that Alt+Enter submits multiline input
    #[test]
    fn test_alt_enter_submits_multiline_input() {
        let mut app = App::new();

        // Create multiline dialog (feature description triggers multiline)
        app.handle_view_action(ViewAction::RequestInput {
            prompt: "Enter feature description:".to_string(),
            placeholder: None,
        });

        // Verify it's multiline
        let dialog = app.input_dialog.as_ref().unwrap();
        assert!(dialog.is_multiline(), "Should be multiline dialog");

        // Type some text
        app.handle_key_event(key_event(KeyCode::Char('t')));
        app.handle_key_event(key_event(KeyCode::Char('e')));
        app.handle_key_event(key_event(KeyCode::Char('s')));
        app.handle_key_event(key_event(KeyCode::Char('t')));

        // Submit with Alt+Enter (multiline submit)
        app.handle_key_event(key_event_with_mod(KeyCode::Enter, KeyModifiers::ALT));

        // After submit, input_mode should be false and dialog cleared
        assert!(
            !app.input_mode,
            "input_mode should be false after Alt+Enter submit"
        );
        assert!(
            app.input_dialog.is_none(),
            "input_dialog should be None after submit"
        );
    }

    // T029: Test that Escape cancels input
    #[test]
    fn test_escape_cancels_input() {
        let mut app = App::new();

        // Setup input mode
        app.handle_view_action(ViewAction::RequestInput {
            prompt: "Enter:".to_string(),
            placeholder: None,
        });

        // Type some text
        app.handle_key_event(key_event(KeyCode::Char('x')));

        // Cancel with Escape
        app.handle_key_event(key_event(KeyCode::Esc));

        // Verify input mode is disabled and dialog is cleared
        assert!(!app.input_mode, "input_mode should be false after Escape");
        assert!(
            app.input_dialog.is_none(),
            "input_dialog should be None after Escape"
        );
    }

    // Additional: Test defensive state check syncs input_mode
    #[test]
    fn test_defensive_state_check_resets_orphan_input_mode() {
        let mut app = App::new();

        // Manually create inconsistent state: input_mode=true but no dialog
        app.input_mode = true;
        app.input_dialog = None;

        // The defensive check in handle_key_event should fix this
        app.handle_key_event(key_event(KeyCode::Char('a')));

        // Should have been reset
        assert!(
            !app.input_mode,
            "input_mode should be reset when dialog is None"
        );
    }

    // Additional: Test defensive state check enables input_mode when dialog exists
    #[test]
    fn test_defensive_state_check_enables_input_mode_for_orphan_dialog() {
        let mut app = App::new();

        // Manually create inconsistent state: dialog exists but input_mode=false
        app.input_mode = false;
        app.input_dialog = Some(InputDialog::new("Test", "Enter:"));

        // Send a key event - defensive check should enable input_mode
        app.handle_key_event(key_event(KeyCode::Char('a')));

        // Character should have been inserted (proving input_mode was enabled)
        let dialog = app.input_dialog.as_ref().unwrap();
        assert_eq!(dialog.value(), "a");
    }
}
