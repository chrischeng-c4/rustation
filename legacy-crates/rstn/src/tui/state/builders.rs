//! State builder pattern for easy test setup
//!
//! Builders provide a fluent API for constructing state objects in tests.
//!
//! # Example
//!
//! ```rust
//! use rstn::tui::state::builders::WorktreeViewStateBuilder;
//! use rstn::tui::views::{SpecPhase, PhaseStatus};
//!
//! let state = WorktreeViewStateBuilder::new()
//!     .with_feature("042", "click-function")
//!     .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
//!     .with_spec_content("# Feature 042")
//!     .build();
//! ```

use super::worktree::WorktreeViewState;
use crate::domain::git::{CommitGroup, SecurityWarning};
use crate::tui::event::WorktreeType;
use crate::tui::logging::LogEntry;
use crate::tui::views::{
    Command, ContentType, FeatureInfo, GitCommand, PhaseStatus, SpecPhase,
    SpecifyState, WorktreeFocus,
};
use std::path::PathBuf;

use crate::tui::state::prompt_claude::PromptClaudeStatus;
use crate::tui::state::workflow::WorkflowState;

/// Builder for WorktreeViewState
pub struct WorktreeViewStateBuilder {
    feature_number: Option<String>,
    feature_name: Option<String>,
    worktree_type: WorktreeType,
    spec_content: Option<String>,
    plan_content: Option<String>,
    tasks_content: Option<String>,
    phases: Vec<(SpecPhase, PhaseStatus)>,
    current_phase: Option<SpecPhase>,
    focus: WorktreeFocus,
    content_type: ContentType,
    content_scroll: usize,
    // P2 fields
    commands: Vec<Command>,
    command_state_index: Option<usize>,
    log_entries: Vec<LogEntry>,
    output_scroll: usize,
    // P3 fields
    pending_input_phase: Option<SpecPhase>,
    // Workflow subsystem
    prompt_workflow: WorkflowState<PromptClaudeStatus>,
    // Legacy Progress Subsystem
    progress_step: Option<u32>,
    progress_total: Option<u32>,
    progress_message: Option<String>,
    // P4 fields
    pending_commit_message: Option<String>,
    commit_warnings: Vec<SecurityWarning>,
    commit_groups: Option<Vec<CommitGroup>>,
    current_commit_index: usize,
    commit_message_input: String,
    commit_message_cursor: usize,
    commit_sensitive_files: Vec<String>,
    commit_validation_error: Option<String>,
    // P5 fields
    specify_state: SpecifyState,
    pending_git_command: Option<GitCommand>,
}

impl WorktreeViewStateBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        let phases = SpecPhase::all()
            .iter()
            .map(|&p| (p, PhaseStatus::NotStarted))
            .collect::<Vec<_>>();

        // Build default command list
        let mut commands = Vec::new();
        commands.push(Command::PromptClaude);

        Self {
            feature_number: None,
            feature_name: None,
            worktree_type: WorktreeType::NotGit,
            spec_content: None,
            plan_content: None,
            tasks_content: None,
            phases,
            current_phase: None,
            focus: WorktreeFocus::Commands,
            content_type: ContentType::Spec, // Default to Spec view
            content_scroll: 0,
            // P2 fields
            commands,
            command_state_index: Some(1), // Start on "Prompt Claude"
            log_entries: Vec::new(),
            output_scroll: 0,
            pending_input_phase: None,
            // Workflow subsystem
            prompt_workflow: WorkflowState::default(),
            // Progress subsystem
            progress_step: None,
            progress_total: None,
            progress_message: None,
            // P4 fields
            pending_commit_message: None,
            commit_warnings: Vec::new(),
            commit_groups: None,
            current_commit_index: 0,
            commit_message_input: String::new(),
            commit_message_cursor: 0,
            commit_sensitive_files: Vec::new(),
            commit_validation_error: None,
            // P5 fields
            specify_state: SpecifyState::default(),
            pending_git_command: None,
        }
    }

    /// Set feature information
    pub fn with_feature(mut self, number: &str, name: &str) -> Self {
        self.feature_number = Some(number.to_string());
        self.feature_name = Some(name.to_string());
        self.worktree_type = WorktreeType::FeatureWorktree {
            number: number.to_string(),
            name: name.to_string(),
        };
        self
    }

    /// Set feature number only (name derived as "test-feature")
    pub fn with_feature_number(self, number: &str) -> Self {
        self.with_feature(number, "test-feature")
    }

    /// Set worktree type
    pub fn with_worktree_type(mut self, worktree_type: WorktreeType) -> Self {
        self.worktree_type = worktree_type;
        self
    }

    /// Set spec content
    pub fn with_spec_content(mut self, content: &str) -> Self {
        self.spec_content = Some(content.to_string());
        self
    }

    /// Set plan content
    pub fn with_plan_content(mut self, content: &str) -> Self {
        self.plan_content = Some(content.to_string());
        self
    }

    /// Set tasks content
    pub fn with_tasks_content(mut self, content: &str) -> Self {
        self.tasks_content = Some(content.to_string());
        self
    }

    /// Set phase status
    pub fn with_phase(mut self, phase: SpecPhase, status: PhaseStatus) -> Self {
        for (p, s) in &mut self.phases {
            if *p == phase {
                *s = status;
                break;
            }
        }
        self
    }

    /// Set current phase
    pub fn with_current_phase(mut self, phase: SpecPhase) -> Self {
        self.current_phase = Some(phase);
        self
    }

    /// Set focus
    pub fn with_focus(mut self, focus: WorktreeFocus) -> Self {
        self.focus = focus;
        self
    }

    /// Set content type
    pub fn with_content_type(mut self, content_type: ContentType) -> Self {
        self.content_type = content_type;
        self
    }

    /// Set content scroll position
    pub fn with_content_scroll(mut self, scroll: usize) -> Self {
        self.content_scroll = scroll;
        self
    }

    /// Build the final WorktreeViewState
    pub fn build(self) -> WorktreeViewState {
        let feature_info =
            if let (Some(number), Some(name)) = (self.feature_number, self.feature_name) {
                Some(FeatureInfo {
                    number: number.clone(),
                    name: name.clone(),
                    branch: format!("{}-{}", number, name),
                    spec_dir: PathBuf::from(format!("specs/{}-{}", number, name)),
                })
            } else {
                None
            };

        WorktreeViewState {
            feature_info,
            worktree_type: self.worktree_type,
            spec_content: self.spec_content,
            plan_content: self.plan_content,
            tasks_content: self.tasks_content,
            phases: self.phases,
            current_phase: self.current_phase,
            focus: self.focus,
            content_type: self.content_type,
            content_scroll: self.content_scroll,
            // P2 fields
            commands: self.commands,
            command_state_index: self.command_state_index,
            log_entries: self.log_entries,
            output_scroll: self.output_scroll,
            // Workflow subsystem
            prompt_workflow: self.prompt_workflow,
            // P3 fields
            pending_input_phase: self.pending_input_phase,
            progress_step: self.progress_step,
            progress_total: self.progress_total,
            progress_message: self.progress_message,
            // P4 fields
            pending_commit_message: self.pending_commit_message,
            commit_warnings: self.commit_warnings,
            commit_groups: self.commit_groups,
            current_commit_index: self.current_commit_index,
            commit_message_input: self.commit_message_input,
            commit_message_cursor: self.commit_message_cursor,
            commit_sensitive_files: self.commit_sensitive_files,
            commit_validation_error: self.commit_validation_error,
            // P5 fields
            specify_state: self.specify_state,
            pending_git_command: self.pending_git_command,
        }
    }

    // ========================================
    // Preset Configurations
    // ========================================

    /// Create state for Specify phase in progress
    pub fn specify_in_progress(number: &str) -> WorktreeViewState {
        Self::new()
            .with_feature_number(number)
            .with_phase(SpecPhase::Specify, PhaseStatus::InProgress)
            .with_current_phase(SpecPhase::Specify)
            .with_focus(WorktreeFocus::Output)
            .with_content_type(ContentType::Spec)
            .build()
    }

    /// Create state for Specify phase completed
    pub fn specify_completed(number: &str) -> WorktreeViewState {
        Self::new()
            .with_feature_number(number)
            .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
            .with_spec_content("# Feature Specification\n\nGenerated spec content")
            .with_focus(WorktreeFocus::Content)
            .with_content_type(ContentType::Spec)
            .build()
    }

    /// Create state for Plan phase completed
    pub fn plan_completed(number: &str) -> WorktreeViewState {
        Self::new()
            .with_feature_number(number)
            .with_phase(SpecPhase::Specify, PhaseStatus::Completed)
            .with_phase(SpecPhase::Plan, PhaseStatus::Completed)
            .with_spec_content("# Feature Specification")
            .with_plan_content("# Implementation Plan")
            .with_focus(WorktreeFocus::Content)
            .with_content_type(ContentType::Plan)
            .build()
    }
}

impl Default for WorktreeViewStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_empty() {
        let state = WorktreeViewStateBuilder::new().build();
        assert_eq!(state.feature_info, None);
        assert_eq!(state.worktree_type, WorktreeType::NotGit);
        assert_eq!(state.focus, WorktreeFocus::Commands);
    }

    #[test]
    fn test_builder_with_feature() {
        let state = WorktreeViewStateBuilder::new()
            .with_feature("042", "click-function")
            .build();

        assert!(state.feature_info.is_some());
        let feature = state.feature_info.unwrap();
        assert_eq!(feature.number, "042");
        assert_eq!(feature.name, "click-function");
        assert_eq!(feature.branch, "042-click-function");
    }

    #[test]
    fn test_builder_preset_specify_in_progress() {
        let state = WorktreeViewStateBuilder::specify_in_progress("042");

        assert_eq!(
            state.get_phase_status(SpecPhase::Specify),
            PhaseStatus::InProgress
        );
        assert_eq!(state.current_phase, Some(SpecPhase::Specify));
        assert_eq!(state.focus, WorktreeFocus::Output);
    }

    #[test]
    fn test_builder_preset_specify_completed() {
        let state = WorktreeViewStateBuilder::specify_completed("042");

        assert_eq!(
            state.get_phase_status(SpecPhase::Specify),
            PhaseStatus::Completed
        );
        assert!(state.spec_content.is_some());
        assert_eq!(state.content_type, ContentType::Spec);
    }
}
