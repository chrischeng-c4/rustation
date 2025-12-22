//! Worktree view state
//!
//! This module defines the serializable state for WorktreeView.
//! Phase 1: P1 fields (10 fields) - feature context, content cache, phase tracking
//! Phase 3A: P1+P2 fields (19 fields) - added commands and logging/output state
//! Phase 3B: P1+P2+P3+P4+P5 fields (36 fields) - complete state-first architecture
//! Phase 4 (Workflow-Driven): Refactored to use WorkflowState container

use crate::domain::git::{CommitGroup, SecurityWarning};
use crate::tui::event::WorktreeType;
use crate::tui::logging::LogEntry;
use crate::tui::views::{
    Command, ContentType, FeatureInfo, GitCommand, PhaseStatus, SpecPhase,
    SpecifyState, WorktreeFocus,
};
use serde::{Deserialize, Serialize};

use super::prompt_claude::PromptClaudeStatus;
use super::workflow::WorkflowState;
use super::StateInvariants;

/// Worktree view state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorktreeViewState {
    // ========================================
    // Feature Context
    // ========================================
    /// Current feature information (if in feature worktree)
    pub feature_info: Option<FeatureInfo>,

    /// Worktree type (main repo vs feature worktree)
    pub worktree_type: WorktreeType,

    // ========================================
    // Content Cache
    // ========================================
    /// Cached spec.md content
    pub spec_content: Option<String>,

    /// Cached plan.md content
    pub plan_content: Option<String>,

    /// Cached tasks.md content
    pub tasks_content: Option<String>,

    // ========================================
    // Phase Tracking
    // ========================================
    /// SDD workflow phases with status
    pub phases: Vec<(SpecPhase, PhaseStatus)>,

    /// Currently active phase
    pub current_phase: Option<SpecPhase>,

    // ========================================
    // UI State
    // ========================================
    /// Current focus area
    pub focus: WorktreeFocus,

    /// Content type being displayed
    pub content_type: ContentType,

    /// Content scroll position (line number)
    pub content_scroll: usize,

    // ========================================
    // Commands Subsystem
    // ========================================
    /// Unified command list (SDD phases + Git actions)
    pub commands: Vec<Command>,

    /// Selected command index (derived from ListState)
    pub command_state_index: Option<usize>,

    // ========================================
    // Logging/Output Subsystem
    // ========================================
    /// Log entries (serializable form of LogBuffer)
    pub log_entries: Vec<LogEntry>,

    /// Output scroll position
    pub output_scroll: usize,

    // ========================================
    // Workflow Subsystem (Replaces scattered fields)
    // ========================================
    /// Prompt Claude workflow state
    pub prompt_workflow: WorkflowState<PromptClaudeStatus>,

    // ========================================
    // Input Subsystem
    // ========================================
    /// Pending input request for a specific phase
    pub pending_input_phase: Option<SpecPhase>,

    // ========================================
    // Progress Subsystem (Legacy - to be moved to Workflow)
    // ========================================
    /// Current progress step
    pub progress_step: Option<u32>,

    /// Total progress steps
    pub progress_total: Option<u32>,

    /// Progress status message
    pub progress_message: Option<String>,

    // ========================================
    // Commit Workflow Subsystem
    // ========================================
    /// Pending commit message from intelligent commit workflow
    pub pending_commit_message: Option<String>,

    /// Security warnings found during commit scanning
    pub commit_warnings: Vec<SecurityWarning>,

    /// Grouped changes for staged commit workflow
    pub commit_groups: Option<Vec<CommitGroup>>,

    /// Current group index in commit review
    pub current_commit_index: usize,

    /// User input for commit message
    pub commit_message_input: String,

    /// Cursor position in commit message input
    pub commit_message_cursor: usize,

    /// Sensitive files found during commit
    pub commit_sensitive_files: Vec<String>,

    /// Commit validation error message
    pub commit_validation_error: Option<String>,

    // ========================================
    // Specify Workflow Subsystem
    // ========================================
    /// SDD workflow state (Specify/Plan/Tasks phases)
    pub specify_state: SpecifyState,
    
    // Note: Pending git command kept for now, but should move to a GitWorkflow later
    pub pending_git_command: Option<GitCommand>,
}

impl Default for WorktreeViewState {
    fn default() -> Self {
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
            // Feature context
            feature_info: None,
            worktree_type: WorktreeType::NotGit,

            // Content cache
            spec_content: None,
            plan_content: None,
            tasks_content: None,

            // Phase tracking
            phases,
            current_phase: None,

            // UI state
            focus: WorktreeFocus::Commands,
            content_type: ContentType::Spec, // Default to Spec view
            content_scroll: 0,

            // Commands subsystem
            commands,
            command_state_index: Some(1), // Start on "Prompt Claude"

            // Logging/Output subsystem
            log_entries: Vec::new(),
            output_scroll: 0,

            // Workflow subsystem
            prompt_workflow: WorkflowState::default(),
            pending_git_command: None,
            pending_input_phase: None,

            // Progress subsystem
            progress_step: None,
            progress_total: None,
            progress_message: None,

            // Commit workflow
            pending_commit_message: None,
            commit_warnings: Vec::new(),
            commit_groups: None,
            current_commit_index: 0,
            commit_message_input: String::new(),
            commit_message_cursor: 0,
            commit_sensitive_files: Vec::new(),
            commit_validation_error: None,

            // Specify workflow
            specify_state: SpecifyState::default(),
        }
    }
}

impl StateInvariants for WorktreeViewState {
    fn assert_invariants(&self) {
        // Invariant 1: Feature info present for feature worktrees
        if matches!(self.worktree_type, WorktreeType::FeatureWorktree { .. }) {
            assert!(
                self.feature_info.is_some(),
                "Feature worktree requires feature_info"
            );
        }

        // Invariant 2: Content scroll within reasonable bounds (0..100000)
        assert!(
            self.content_scroll < 100000,
            "Content scroll position unreasonably large: {}",
            self.content_scroll
        );

        // Invariant 3: Output scroll within reasonable bounds (0..100000)
        assert!(
            self.output_scroll < 100000,
            "Output scroll position unreasonably large: {}",
            self.output_scroll
        );

        // Invariant 4: Command state index within bounds
        if let Some(idx) = self.command_state_index {
            assert!(
                idx < self.commands.len(),
                "Command state index {} out of bounds (commands.len = {})",
                idx,
                self.commands.len()
            );
        }
    }
}

impl WorktreeViewState {
    /// Get phase status for a specific phase
    pub fn get_phase_status(&self, phase: SpecPhase) -> PhaseStatus {
        self.phases
            .iter()
            .find(|(p, _)| *p == phase)
            .map(|(_, status)| *status)
            .unwrap_or(PhaseStatus::NotStarted)
    }

    /// Set phase status for a specific phase
    pub fn set_phase_status(&mut self, phase: SpecPhase, status: PhaseStatus) {
        for (p, s) in &mut self.phases {
            if *p == phase {
                *s = status;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_valid() {
        let state = WorktreeViewState::default();
        state.assert_invariants();
    }

    #[test]
    fn test_get_phase_status() {
        let state = WorktreeViewState::default();
        assert_eq!(
            state.get_phase_status(SpecPhase::Specify),
            PhaseStatus::NotStarted
        );
    }

    #[test]
    fn test_set_phase_status() {
        let mut state = WorktreeViewState::default();
        state.set_phase_status(SpecPhase::Specify, PhaseStatus::Completed);
        assert_eq!(
            state.get_phase_status(SpecPhase::Specify),
            PhaseStatus::Completed
        );
    }
}
