//! TUI views

mod command_runner;
mod dashboard;
mod spec;
mod worktree;

pub use command_runner::CommandRunner;
pub use dashboard::Dashboard;
pub use spec::{AutoFlowState, ClaudeOptions, PhaseStatus, SpecPhase, SpecView};
pub use worktree::WorktreeView;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Actions that views can request
#[derive(Debug, Clone)]
pub enum ViewAction {
    /// No action needed
    None,
    /// Switch to a different view
    SwitchView(ViewType),
    /// Run a command
    RunCommand { name: String, args: Vec<String> },
    /// Run a spec-kit phase (opens Claude Code)
    RunSpecPhase {
        phase: String,
        command: String,
        options: ClaudeOptions,
    },
    /// Start the SDD wizard
    StartWizard,
    /// Show worktree list/manager
    ShowWorktrees,
    /// Quit the application
    Quit,
}

/// View types for switching
#[derive(Debug, Clone, Copy)]
pub enum ViewType {
    Dashboard,
    Commands,
    Spec,
}

/// Trait for views that can be rendered and handle input
pub trait View {
    /// Render the view
    fn render(&self, frame: &mut Frame, area: Rect);

    /// Handle key input, returning an optional action
    fn handle_key(&mut self, key: KeyEvent) -> ViewAction;

    /// Tick update
    fn tick(&mut self) {}
}
