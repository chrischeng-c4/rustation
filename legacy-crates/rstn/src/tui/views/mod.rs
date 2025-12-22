//! TUI views

mod command_runner;
mod dashboard;
mod mcp_server;
mod session_history;
mod session_output;
mod settings;
mod spec;
mod worktree;

pub use command_runner::{CommandRunner, OutputLine, OutputLineType};
pub use dashboard::{Dashboard, DashboardPanel, TestResults};
pub use mcp_server::McpServerView;
pub use session_history::SessionHistoryView;
pub use session_output::{CompletionStatus, SessionOutputView};
pub use settings::SettingsView;
pub use spec::{AutoFlowState, ClaudeOptions, PhaseStatus, SpecPhase, SpecView};
pub use worktree::{
    Command, ContentType, FeatureInfo, GitCommand, InlineInput, SpecifyState, WorktreeFocus,
    WorktreeView,
};

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Actions that views can request
#[derive(Debug, Clone, PartialEq)]
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
    /// Run Prompt Claude command with user prompt (Primary workflow command)
    RunPromptClaude { prompt: String },
    /// Start the SDD wizard
    StartWizard,
    /// Show worktree list/manager
    ShowWorktrees,
    /// Quit the application
    Quit,
    /// Request text input from user
    RequestInput {
        prompt: String,
        placeholder: Option<String>,
    },
    /// Run a git command
    RunGitCommand(GitCommand),
    /// Run intelligent commit workflow with AI-powered grouping
    RunIntelligentCommit,
    /// Submit current commit group in review workflow (Feature 050)
    SubmitCommitGroup,
    /// Generate content for an SDD phase (Features 051, 053-058)
    GenerateSpec {
        phase: SpecPhase,
        description: String,
    },
    /// Save generated content to file (Features 051, 053-058)
    SaveSpec {
        phase: SpecPhase,
        content: String,
        number: String,
        name: String,
    },
    /// Execute a task with Claude CLI (Feature 056)
    ExecuteTask {
        task_id: String,
        task_description: String,
        feature_number: String,
        feature_name: String,
    },
    /// Display a message to the user (Feature 056)
    DisplayMessage(String),
}

/// View types for switching
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewType {
    Dashboard,
    Commands,
    Spec,
}

/// Trait for views that can be rendered and handle input
pub trait View {
    /// Render the view
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Handle key input, returning an optional action
    fn handle_key(&mut self, key: KeyEvent) -> ViewAction;

    /// Tick update
    fn tick(&mut self) {}
}
