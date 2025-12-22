//! Type definitions for the Worktree view
//!
//! This module contains enums and structs used by WorktreeView:
//! - Feature information
//! - Command types (SDD phases and Git actions)
//! - Content display types
//! - Focus management

use std::path::PathBuf;

use crate::tui::views::{PhaseStatus, SpecPhase};

/// Feature information extracted from git branch
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FeatureInfo {
    pub number: String,
    pub name: String,
    pub branch: String,
    pub spec_dir: PathBuf,
}

/// Git command types
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
            GitCommand::Commit => "Intelligent Commit",
            GitCommand::Push => "Push",
            GitCommand::Status => "Status",
            GitCommand::AddAll => "Add All",
            GitCommand::Rebase => "Rebase",
        }
    }
}

/// Unified command that can be a workflow command, SDD phase, or Git action
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Command {
    PromptClaude,                     // Primary workflow command
    SddPhase(SpecPhase, PhaseStatus), // SDD workflow phases
    GitAction(GitCommand),            // Git actions
}

/// Content type to display in middle panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContentType {
    Spec,
    Plan,
    Tasks,
    CommitReview,
    SpecifyInput,  // Feature 051: Input feature description
    SpecifyReview, // Feature 051: Review/edit generated spec
    PromptInput,   // Prompt Claude: Multi-line prompt input
    PromptRunning, // Prompt Claude: Streaming output display
}

impl ContentType {
    pub(super) fn name(&self) -> &'static str {
        match self {
            ContentType::Spec => "Spec",
            ContentType::Plan => "Plan",
            ContentType::Tasks => "Tasks",
            ContentType::CommitReview => "Commit Review",
            ContentType::SpecifyInput => "Specify Input",
            ContentType::SpecifyReview => "Specify Review",
            ContentType::PromptInput => "Prompt Claude",
            ContentType::PromptRunning => "Claude Running",
        }
    }
}

/// Focus state for the Worktree view
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorktreeFocus {
    Commands, // Unified panel for SDD phases and Git actions
    Content,  // Content display (spec, plan, tasks)
    Output,   // Output/log panel
}
