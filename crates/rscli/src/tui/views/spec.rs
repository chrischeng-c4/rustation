//! Spec-Driven Development (SDD) view with workflow phases

use crate::tui::views::{View, ViewAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

/// SDD Workflow phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecPhase {
    Specify,
    Clarify,
    Plan,
    Tasks,
    Analyze,
    Implement,
    Review,
}

impl SpecPhase {
    pub fn all() -> &'static [SpecPhase] {
        &[
            SpecPhase::Specify,
            SpecPhase::Clarify,
            SpecPhase::Plan,
            SpecPhase::Tasks,
            SpecPhase::Analyze,
            SpecPhase::Implement,
            SpecPhase::Review,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SpecPhase::Specify => "specify",
            SpecPhase::Clarify => "clarify",
            SpecPhase::Plan => "plan",
            SpecPhase::Tasks => "tasks",
            SpecPhase::Analyze => "analyze",
            SpecPhase::Implement => "implement",
            SpecPhase::Review => "review",
        }
    }

    pub fn from_name(name: &str) -> Option<SpecPhase> {
        match name.to_lowercase().as_str() {
            "specify" => Some(SpecPhase::Specify),
            "clarify" => Some(SpecPhase::Clarify),
            "plan" => Some(SpecPhase::Plan),
            "tasks" => Some(SpecPhase::Tasks),
            "analyze" => Some(SpecPhase::Analyze),
            "implement" => Some(SpecPhase::Implement),
            "review" => Some(SpecPhase::Review),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SpecPhase::Specify => "Specify",
            SpecPhase::Clarify => "Clarify",
            SpecPhase::Plan => "Plan",
            SpecPhase::Tasks => "Tasks",
            SpecPhase::Analyze => "Analyze",
            SpecPhase::Implement => "Implement",
            SpecPhase::Review => "Review",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SpecPhase::Specify => "Define feature requirements in spec.md",
            SpecPhase::Clarify => "Ask clarifying questions to refine spec",
            SpecPhase::Plan => "Design architecture in plan.md",
            SpecPhase::Tasks => "Generate task breakdown in tasks.md",
            SpecPhase::Analyze => "Validate consistency across artifacts",
            SpecPhase::Implement => "Execute implementation plan",
            SpecPhase::Review => "Review PR against spec requirements",
        }
    }

    pub fn command(&self) -> &'static str {
        match self {
            SpecPhase::Specify => "/speckit.specify",
            SpecPhase::Clarify => "/speckit.clarify",
            SpecPhase::Plan => "/speckit.plan",
            SpecPhase::Tasks => "/speckit.tasks",
            SpecPhase::Analyze => "/speckit.analyze",
            SpecPhase::Implement => "/speckit.implement",
            SpecPhase::Review => "/speckit.review",
        }
    }

    pub fn hotkey(&self) -> char {
        match self {
            SpecPhase::Specify => 's',
            SpecPhase::Clarify => 'c',
            SpecPhase::Plan => 'p',
            SpecPhase::Tasks => 't',
            SpecPhase::Analyze => 'a',
            SpecPhase::Implement => 'i',
            SpecPhase::Review => 'r',
        }
    }
}

/// Phase status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseStatus {
    NotStarted,
    InProgress,
    Completed,
    NeedsUpdate,
}

impl PhaseStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            PhaseStatus::NotStarted => "○",
            PhaseStatus::InProgress => "◐",
            PhaseStatus::Completed => "●",
            PhaseStatus::NeedsUpdate => "◑",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            PhaseStatus::NotStarted => Color::DarkGray,
            PhaseStatus::InProgress => Color::Yellow,
            PhaseStatus::Completed => Color::Green,
            PhaseStatus::NeedsUpdate => Color::Magenta,
        }
    }
}

/// Current feature info
#[derive(Debug, Clone, Default)]
pub struct FeatureInfo {
    pub number: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub branch: Option<String>,
    pub phase_statuses: Vec<(SpecPhase, PhaseStatus)>,
}

impl FeatureInfo {
    pub fn new() -> Self {
        let phase_statuses = SpecPhase::all()
            .iter()
            .map(|p| (*p, PhaseStatus::NotStarted))
            .collect();
        Self {
            number: None,
            name: None,
            description: None,
            branch: None,
            phase_statuses,
        }
    }

    pub fn with_detection() -> Self {
        // Try to detect from git branch or spec files
        let mut info = Self::new();
        // In real implementation, this would read from git and spec files
        info.branch = Some("main".to_string());
        info
    }
}

/// Focus area in the spec view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecFocus {
    Phases,
    Info,
    Wizard,
}

/// Wizard state
#[derive(Debug, Clone)]
pub struct WizardState {
    pub active: bool,
    pub current_step: usize,
    pub steps: Vec<WizardStep>,
}

#[derive(Debug, Clone)]
pub struct WizardStep {
    pub phase: SpecPhase,
    pub prompt: String,
    pub completed: bool,
}

impl WizardState {
    pub fn new() -> Self {
        let steps = vec![
            WizardStep {
                phase: SpecPhase::Specify,
                prompt: "First, let's define the feature specification.".to_string(),
                completed: false,
            },
            WizardStep {
                phase: SpecPhase::Clarify,
                prompt: "Let's clarify any ambiguous requirements.".to_string(),
                completed: false,
            },
            WizardStep {
                phase: SpecPhase::Plan,
                prompt: "Now let's design the architecture.".to_string(),
                completed: false,
            },
            WizardStep {
                phase: SpecPhase::Tasks,
                prompt: "Generate the task breakdown.".to_string(),
                completed: false,
            },
            WizardStep {
                phase: SpecPhase::Implement,
                prompt: "Ready to implement the feature!".to_string(),
                completed: false,
            },
        ];
        Self {
            active: false,
            current_step: 0,
            steps,
        }
    }

    pub fn current(&self) -> Option<&WizardStep> {
        self.steps.get(self.current_step)
    }

    pub fn advance(&mut self) {
        if self.current_step < self.steps.len() {
            if let Some(step) = self.steps.get_mut(self.current_step) {
                step.completed = true;
            }
            self.current_step += 1;
        }
    }
}

impl Default for WizardState {
    fn default() -> Self {
        Self::new()
    }
}

/// CLI options for Claude Code execution
#[derive(Debug, Clone)]
pub struct ClaudeOptions {
    /// Maximum agentic turns per phase
    pub max_turns: u32,
    /// Skip permission prompts (use with caution)
    pub skip_permissions: bool,
    /// Continue previous session
    pub continue_session: bool,
    /// Session ID for continuation
    pub session_id: Option<String>,
    /// Allowed tools (empty = all)
    pub allowed_tools: Vec<String>,
}

impl Default for ClaudeOptions {
    fn default() -> Self {
        Self {
            max_turns: 10,
            skip_permissions: false,
            continue_session: false,
            session_id: None,
            allowed_tools: Vec::new(),
        }
    }
}

/// Auto-flow state for running full SDD workflow with review pauses
#[derive(Debug, Clone)]
pub struct AutoFlowState {
    /// Is auto-flow mode active?
    pub active: bool,
    /// Current phase index in the workflow
    pub current_phase_idx: usize,
    /// Awaiting user review before continuing?
    pub awaiting_review: bool,
    /// Output from last completed phase
    pub last_output: Vec<String>,
    /// Was last phase successful?
    pub last_success: bool,
    /// Phases to run in order
    pub phases: Vec<SpecPhase>,
    /// Claude CLI options
    pub options: ClaudeOptions,
}

impl AutoFlowState {
    pub fn new() -> Self {
        Self {
            active: false,
            current_phase_idx: 0,
            awaiting_review: false,
            last_output: Vec::new(),
            last_success: false,
            phases: vec![
                SpecPhase::Specify,
                SpecPhase::Clarify,
                SpecPhase::Plan,
                SpecPhase::Tasks,
                SpecPhase::Analyze,
                SpecPhase::Implement,
                SpecPhase::Review,
            ],
            options: ClaudeOptions::default(),
        }
    }

    pub fn current_phase(&self) -> Option<&SpecPhase> {
        self.phases.get(self.current_phase_idx)
    }

    pub fn advance(&mut self) {
        if self.current_phase_idx < self.phases.len() {
            self.current_phase_idx += 1;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_phase_idx >= self.phases.len()
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.current_phase_idx = 0;
        self.awaiting_review = false;
        self.last_output.clear();
        self.last_success = false;
        // Keep options but clear session for fresh start
        self.options.session_id = None;
        self.options.continue_session = false;
    }

    pub fn progress_percent(&self) -> u16 {
        if self.phases.is_empty() {
            return 100;
        }
        ((self.current_phase_idx as f32 / self.phases.len() as f32) * 100.0) as u16
    }
}

impl Default for AutoFlowState {
    fn default() -> Self {
        Self::new()
    }
}

/// Spec-Driven Development view
pub struct SpecView {
    /// Current feature info
    pub feature: FeatureInfo,
    /// Phase list state
    pub phase_state: ListState,
    /// Current focus
    pub focus: SpecFocus,
    /// Wizard state
    pub wizard: WizardState,
    /// Auto-flow state for full workflow execution
    pub auto_flow: AutoFlowState,
    /// Output/log lines
    pub output_lines: Vec<String>,
    /// Output scroll
    pub output_scroll: usize,
}

impl SpecView {
    pub fn new() -> Self {
        let mut phase_state = ListState::default();
        phase_state.select(Some(0));
        Self {
            feature: FeatureInfo::with_detection(),
            phase_state,
            focus: SpecFocus::Phases,
            wizard: WizardState::new(),
            auto_flow: AutoFlowState::new(),
            output_lines: vec![
                "Spec-Driven Development Workflow".to_string(),
                "─".repeat(40),
                "Hotkeys:".to_string(),
                "  A - Start Auto-Flow (full workflow)".to_string(),
                "  w - Start guided wizard".to_string(),
                "  s/c/p/t/a/i/r - Run individual phases".to_string(),
                "".to_string(),
                "Settings (before starting Auto-Flow):".to_string(),
                "  m - Toggle max turns (5/10/20)".to_string(),
                "  P - Toggle skip permissions".to_string(),
                "".to_string(),
                "Auto-Flow runs all phases in sequence,".to_string(),
                "pausing after each for your review.".to_string(),
            ],
            output_scroll: 0,
        }
    }

    /// Handle phase completion from auto-flow
    pub fn handle_phase_completed(&mut self, _phase: String, success: bool, output: Vec<String>) {
        if !self.auto_flow.active {
            return;
        }

        self.auto_flow.last_output = output;
        self.auto_flow.last_success = success;
        self.auto_flow.awaiting_review = true;

        // Update phase status in feature info
        if let Some(idx) = self.auto_flow.current_phase_idx.checked_sub(0) {
            if idx < self.feature.phase_statuses.len() {
                self.feature.phase_statuses[idx].1 = if success {
                    PhaseStatus::Completed
                } else {
                    PhaseStatus::NeedsUpdate
                };
            }
        }
    }

    /// Get next phase action for auto-flow continuation
    pub fn get_next_auto_flow_action(&mut self) -> ViewAction {
        if !self.auto_flow.active || self.auto_flow.awaiting_review {
            return ViewAction::None;
        }

        if let Some(phase) = self.auto_flow.current_phase().copied() {
            // Mark current phase as in progress
            if self.auto_flow.current_phase_idx < self.feature.phase_statuses.len() {
                self.feature.phase_statuses[self.auto_flow.current_phase_idx].1 =
                    PhaseStatus::InProgress;
            }
            return self.run_phase(phase);
        }

        // All phases complete
        self.auto_flow.active = false;
        ViewAction::None
    }

    fn get_selected_phase(&self) -> Option<SpecPhase> {
        self.phase_state
            .selected()
            .and_then(|i| SpecPhase::all().get(i).copied())
    }

    /// Get only the focused pane content as text
    pub fn get_focused_pane_text(&self) -> String {
        match self.focus {
            SpecFocus::Phases => {
                // Return list of phases with their statuses
                let mut lines = vec!["=== Workflow Phases ===".to_string()];
                for (phase, status) in &self.feature.phase_statuses {
                    let status_char = match status {
                        PhaseStatus::NotStarted => ' ',
                        PhaseStatus::InProgress => '⋯',
                        PhaseStatus::Completed => '✓',
                        PhaseStatus::NeedsUpdate => '!',
                    };
                    lines.push(format!("  [{}] {} - {}", status_char, phase.hotkey(), phase.name()));
                }
                lines.join("\n")
            }
            SpecFocus::Info => {
                // Return feature info and settings
                vec![
                    "=== Feature Info ===".to_string(),
                    format!("Number: {}", self.feature.number.as_deref().unwrap_or("N/A")),
                    format!("Name: {}", self.feature.name.as_deref().unwrap_or("N/A")),
                    format!("Branch: {}", self.feature.branch.as_deref().unwrap_or("N/A")),
                    String::new(),
                    "=== Auto-Flow Settings ===".to_string(),
                    format!("Max Turns: {}", self.auto_flow.options.max_turns),
                    format!("Skip Permissions: {}", if self.auto_flow.options.skip_permissions { "Yes" } else { "No" }),
                ].join("\n")
            }
            SpecFocus::Wizard => {
                // Return wizard state if active, otherwise output
                if self.wizard.active {
                    let mut lines = vec![
                        "=== SDD Wizard ===".to_string(),
                        format!("Step {}/{}", self.wizard.current_step + 1, self.wizard.steps.len()),
                    ];
                    if let Some(step) = self.wizard.steps.get(self.wizard.current_step) {
                        lines.push(format!("Phase: {}", step.phase.name()));
                        lines.push(format!("Prompt: {}", step.prompt));
                        lines.push(format!("Completed: {}", if step.completed { "Yes" } else { "No" }));
                    }
                    lines.join("\n")
                } else if self.auto_flow.active && !self.auto_flow.last_output.is_empty() {
                    self.auto_flow.last_output.join("\n")
                } else {
                    self.output_lines.join("\n")
                }
            }
        }
    }

    /// Get all visible output as a single string for copying
    pub fn get_output_text(&self) -> String {
        // If in auto-flow, return the last phase output
        if self.auto_flow.active && !self.auto_flow.last_output.is_empty() {
            return self.auto_flow.last_output.join("\n");
        }
        // Otherwise return the guide/help text
        self.output_lines.join("\n")
    }

    /// Get styled output (for spec view, just return plain text since we don't track colors)
    pub fn get_styled_output(&self) -> String {
        self.get_output_text()
    }

    /// Switch to next pane
    pub fn next_pane(&mut self) {
        self.focus = match self.focus {
            SpecFocus::Phases => SpecFocus::Info,
            SpecFocus::Info => SpecFocus::Wizard,
            SpecFocus::Wizard => SpecFocus::Phases,
        };
    }

    fn run_phase(&self, phase: SpecPhase) -> ViewAction {
        // Use auto_flow options if active, otherwise defaults
        let options = if self.auto_flow.active {
            self.auto_flow.options.clone()
        } else {
            ClaudeOptions::default()
        };

        ViewAction::RunSpecPhase {
            phase: phase.name().to_string(),
            command: phase.command().to_string(),
            options,
        }
    }
}

impl Default for SpecView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for SpecView {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Layout: left (phases + info), right (output/wizard/auto-flow)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Left side: phases on top, feature info below
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(main_chunks[0]);

        self.render_phases(frame, left_chunks[0]);
        self.render_feature_info(frame, left_chunks[1]);

        // Right side: auto-flow review, wizard, or output
        if self.auto_flow.active {
            self.render_auto_flow(frame, main_chunks[1]);
        } else if self.wizard.active {
            self.render_wizard(frame, main_chunks[1]);
        } else {
            self.render_output(frame, main_chunks[1]);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        // Auto-flow mode takes highest priority
        if self.auto_flow.active {
            return self.handle_auto_flow_key(key);
        }

        // Wizard mode takes priority
        if self.wizard.active {
            return self.handle_wizard_key(key);
        }

        match key.code {
            // Start auto-flow with 'A' (shift+a)
            KeyCode::Char('A') => {
                self.auto_flow.reset();
                self.auto_flow.active = true;
                self.focus = SpecFocus::Wizard; // Reuse wizard focus
                // Start first phase immediately
                return self.get_next_auto_flow_action();
            }
            // Start wizard
            KeyCode::Char('w') => {
                self.wizard.active = true;
                self.wizard.current_step = 0;
                self.focus = SpecFocus::Wizard;
                ViewAction::None
            }
            // Phase navigation
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.phase_state.selected().unwrap_or(0);
                let new_i = if i == 0 {
                    SpecPhase::all().len() - 1
                } else {
                    i - 1
                };
                self.phase_state.select(Some(new_i));
                ViewAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.phase_state.selected().unwrap_or(0);
                let new_i = (i + 1) % SpecPhase::all().len();
                self.phase_state.select(Some(new_i));
                ViewAction::None
            }
            // Run selected phase
            KeyCode::Enter => {
                if let Some(phase) = self.get_selected_phase() {
                    return self.run_phase(phase);
                }
                ViewAction::None
            }
            // Settings: toggle max turns
            KeyCode::Char('m') => {
                self.auto_flow.options.max_turns = match self.auto_flow.options.max_turns {
                    5 => 10,
                    10 => 20,
                    20 => 50,
                    _ => 5,
                };
                self.output_lines.push(format!(
                    "Max turns set to: {}",
                    self.auto_flow.options.max_turns
                ));
                ViewAction::None
            }
            // Settings: toggle skip permissions
            KeyCode::Char('P') => {
                self.auto_flow.options.skip_permissions = !self.auto_flow.options.skip_permissions;
                self.output_lines.push(format!(
                    "Skip permissions: {}",
                    if self.auto_flow.options.skip_permissions {
                        "ON (dangerous!)"
                    } else {
                        "OFF (safe)"
                    }
                ));
                ViewAction::None
            }
            // Hotkeys for phases
            KeyCode::Char(c) => {
                for phase in SpecPhase::all() {
                    if phase.hotkey() == c {
                        return self.run_phase(*phase);
                    }
                }
                ViewAction::None
            }
            _ => ViewAction::None,
        }
    }

    fn tick(&mut self) {
        // Could refresh feature status periodically
    }
}

impl SpecView {
    fn handle_auto_flow_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            // Stop auto-flow
            KeyCode::Esc => {
                self.auto_flow.reset();
                self.focus = SpecFocus::Phases;
                ViewAction::None
            }
            // Approve and continue to next phase
            KeyCode::Enter | KeyCode::Char('y') if self.auto_flow.awaiting_review => {
                self.auto_flow.awaiting_review = false;
                self.auto_flow.advance();

                if self.auto_flow.is_complete() {
                    // All done!
                    self.auto_flow.active = false;
                    self.focus = SpecFocus::Phases;
                    ViewAction::None
                } else {
                    // Run next phase
                    self.get_next_auto_flow_action()
                }
            }
            // Skip current phase and continue
            KeyCode::Char('n') if self.auto_flow.awaiting_review => {
                self.auto_flow.awaiting_review = false;
                self.auto_flow.advance();

                if self.auto_flow.is_complete() {
                    self.auto_flow.active = false;
                    self.focus = SpecFocus::Phases;
                    ViewAction::None
                } else {
                    self.get_next_auto_flow_action()
                }
            }
            // Scroll output up
            KeyCode::Up | KeyCode::Char('k') if self.auto_flow.awaiting_review => {
                if self.output_scroll > 0 {
                    self.output_scroll = self.output_scroll.saturating_sub(1);
                }
                ViewAction::None
            }
            // Scroll output down
            KeyCode::Down | KeyCode::Char('j') if self.auto_flow.awaiting_review => {
                if self.output_scroll < self.auto_flow.last_output.len().saturating_sub(10) {
                    self.output_scroll += 1;
                }
                ViewAction::None
            }
            _ => ViewAction::None,
        }
    }

    fn handle_wizard_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            KeyCode::Esc => {
                self.wizard.active = false;
                self.focus = SpecFocus::Phases;
                ViewAction::None
            }
            KeyCode::Enter => {
                if let Some(step) = self.wizard.current() {
                    let phase = step.phase;
                    self.wizard.advance();
                    if self.wizard.current_step >= self.wizard.steps.len() {
                        self.wizard.active = false;
                        self.focus = SpecFocus::Phases;
                    }
                    return self.run_phase(phase);
                }
                ViewAction::None
            }
            KeyCode::Char('n') => {
                // Skip to next step
                self.wizard.advance();
                if self.wizard.current_step >= self.wizard.steps.len() {
                    self.wizard.active = false;
                    self.focus = SpecFocus::Phases;
                }
                ViewAction::None
            }
            _ => ViewAction::None,
        }
    }

    fn render_phases(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == SpecFocus::Phases && !self.wizard.active;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" SDD Workflow Phases ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let items: Vec<ListItem> = SpecPhase::all()
            .iter()
            .enumerate()
            .map(|(i, phase)| {
                let status = self
                    .feature
                    .phase_statuses
                    .get(i)
                    .map(|(_, s)| *s)
                    .unwrap_or(PhaseStatus::NotStarted);

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("{} ", status.symbol()),
                            Style::default().fg(status.color()),
                        ),
                        Span::styled(
                            format!("[{}] ", phase.hotkey()),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(phase.display_name(), Style::default().fg(Color::White)),
                    ]),
                    Line::from(vec![Span::styled(
                        format!("    {}", phase.description()),
                        Style::default().fg(Color::DarkGray),
                    )]),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.phase_state.clone());
    }

    fn render_feature_info(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == SpecFocus::Info;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Current Feature ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let mut lines = vec![];

        if let Some(ref num) = self.feature.number {
            lines.push(Line::from(vec![
                Span::styled("Feature: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("#{}", num), Style::default().fg(Color::Cyan)),
            ]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "No feature selected",
                Style::default().fg(Color::DarkGray),
            )]));
        }

        if let Some(ref name) = self.feature.name {
            lines.push(Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(name.clone(), Style::default().fg(Color::White)),
            ]));
        }

        if let Some(ref branch) = self.feature.branch {
            lines.push(Line::from(vec![
                Span::styled("Branch: ", Style::default().fg(Color::DarkGray)),
                Span::styled(branch.clone(), Style::default().fg(Color::Magenta)),
            ]));
        }

        lines.push(Line::from(""));

        // Show current auto-flow settings
        lines.push(Line::from(vec![Span::styled(
            "Auto-Flow Settings:",
            Style::default().fg(Color::DarkGray),
        )]));
        lines.push(Line::from(vec![
            Span::styled("  Max turns: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                self.auto_flow.options.max_turns.to_string(),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(" (m to change)", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Skip perms: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if self.auto_flow.options.skip_permissions {
                    "ON"
                } else {
                    "OFF"
                },
                if self.auto_flow.options.skip_permissions {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
            Span::styled(" (P to toggle)", Style::default().fg(Color::DarkGray)),
        ]));

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_wizard(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" SDD Wizard ")
            .border_style(Style::default().fg(Color::Cyan));

        let mut lines = vec![
            Line::from(vec![Span::styled(
                "Guided Spec-Driven Development",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        // Show progress
        for (i, step) in self.wizard.steps.iter().enumerate() {
            let is_current = i == self.wizard.current_step;
            let symbol = if step.completed {
                "✓"
            } else if is_current {
                "▶"
            } else {
                "○"
            };
            let color = if step.completed {
                Color::Green
            } else if is_current {
                Color::Yellow
            } else {
                Color::DarkGray
            };

            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", symbol), Style::default().fg(color)),
                Span::styled(
                    step.phase.display_name(),
                    Style::default().fg(if is_current { Color::White } else { color }),
                ),
            ]));
        }

        lines.push(Line::from(""));

        // Current step instructions
        if let Some(step) = self.wizard.current() {
            lines.push(Line::from(vec![Span::styled(
                &step.prompt,
                Style::default().fg(Color::White),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Command: ", Style::default().fg(Color::DarkGray)),
                Span::styled(step.phase.command(), Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Press Enter to run | n to skip | Esc to exit wizard",
                Style::default().fg(Color::DarkGray),
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "✓ All phases complete!",
                Style::default().fg(Color::Green),
            )]));
        }

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" SDD Guide ")
            .border_style(Style::default());

        let lines: Vec<Line> = self
            .output_lines
            .iter()
            .skip(self.output_scroll)
            .map(|line| {
                let style = if line.starts_with("─") {
                    Style::default().fg(Color::DarkGray)
                } else if line.starts_with("  /") {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(line.as_str(), style))
            })
            .collect();

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn render_auto_flow(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Gauge;

        // Split into progress bar, phase info, and output/review area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Progress bar
                Constraint::Length(5), // Current phase info
                Constraint::Min(10),   // Output/review area
            ])
            .split(area);

        // Progress bar with settings info
        let progress = self.auto_flow.progress_percent();
        let settings_info = format!(
            " Auto-Flow [max:{} {}] ",
            self.auto_flow.options.max_turns,
            if self.auto_flow.options.skip_permissions {
                "UNSAFE"
            } else {
                "safe"
            }
        );
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(settings_info))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(progress)
            .label(format!(
                "{}/{} phases",
                self.auto_flow.current_phase_idx,
                self.auto_flow.phases.len()
            ));
        frame.render_widget(gauge, chunks[0]);

        // Current phase info
        let phase_block = Block::default()
            .borders(Borders::ALL)
            .title(if self.auto_flow.awaiting_review {
                " Review Phase Output "
            } else {
                " Running Phase "
            })
            .border_style(if self.auto_flow.awaiting_review {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            });

        let phase_lines = if self.auto_flow.is_complete() {
            vec![
                Line::from(vec![Span::styled(
                    "✓ All phases complete!",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press Esc to exit",
                    Style::default().fg(Color::DarkGray),
                )]),
            ]
        } else if let Some(phase) = self.auto_flow.current_phase() {
            let status_line = if self.auto_flow.awaiting_review {
                if self.auto_flow.last_success {
                    Line::from(vec![
                        Span::styled("✓ ", Style::default().fg(Color::Green)),
                        Span::styled(
                            format!("{} completed successfully", phase.display_name()),
                            Style::default().fg(Color::White),
                        ),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("✗ ", Style::default().fg(Color::Red)),
                        Span::styled(
                            format!("{} failed", phase.display_name()),
                            Style::default().fg(Color::White),
                        ),
                    ])
                }
            } else {
                Line::from(vec![
                    Span::styled("◐ ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("Running {}...", phase.display_name()),
                        Style::default().fg(Color::White),
                    ),
                ])
            };

            let action_line = if self.auto_flow.awaiting_review {
                Line::from(vec![Span::styled(
                    "Enter/y: continue | n: skip | j/k: scroll | Esc: stop",
                    Style::default().fg(Color::DarkGray),
                )])
            } else {
                Line::from(vec![Span::styled(
                    "Waiting for phase to complete...",
                    Style::default().fg(Color::DarkGray),
                )])
            };

            vec![status_line, Line::from(""), action_line]
        } else {
            vec![]
        };

        let phase_paragraph = Paragraph::new(phase_lines)
            .block(phase_block)
            .wrap(Wrap { trim: false });
        frame.render_widget(phase_paragraph, chunks[1]);

        // Output area
        let output_block = Block::default()
            .borders(Borders::ALL)
            .title(" Phase Output ")
            .border_style(Style::default());

        let output_lines: Vec<Line> = self
            .auto_flow
            .last_output
            .iter()
            .skip(self.output_scroll)
            .take(chunks[2].height as usize - 2) // Account for borders
            .map(|line| {
                let style = if line.contains("error") || line.contains("Error") {
                    Style::default().fg(Color::Red)
                } else if line.contains("warning") || line.contains("Warning") {
                    Style::default().fg(Color::Yellow)
                } else if line.starts_with("✓") || line.contains("success") {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(line.as_str(), style))
            })
            .collect();

        let output_paragraph = Paragraph::new(output_lines)
            .block(output_block)
            .wrap(Wrap { trim: false });
        frame.render_widget(output_paragraph, chunks[2]);
    }
}
