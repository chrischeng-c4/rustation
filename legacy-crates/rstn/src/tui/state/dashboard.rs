//! Dashboard view state
//!
//! This module defines the serializable state for Dashboard view.

use crate::tui::event::WorktreeType;
use crate::tui::views::{DashboardPanel, TestResults};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::StateInvariants;

/// Dashboard view state
///
/// Contains all serializable fields for the Dashboard view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardState {
    /// Currently focused panel
    pub focused_panel: DashboardPanel,

    /// Git status information
    pub git_branch: String,
    pub git_status: Vec<String>,
    pub worktree_count: usize,

    /// Worktree information
    pub worktree_path: Option<PathBuf>,
    pub is_git_repo: bool,
    pub worktree_type: WorktreeType,
    pub git_error: Option<String>,

    /// Test results
    pub test_results: Option<TestResults>,

    /// Project information
    pub project_name: String,
    pub rust_version: String,

    /// Quick action selection
    pub quick_action_index: usize,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            focused_panel: DashboardPanel::QuickActions,
            git_branch: "main".to_string(),
            git_status: vec!["Loading...".to_string()],
            worktree_count: 1,
            worktree_path: None,
            is_git_repo: true,
            worktree_type: WorktreeType::MainRepository,
            git_error: None,
            test_results: None,
            project_name: "rustation".to_string(),
            rust_version: "1.75+".to_string(),
            quick_action_index: 0,
        }
    }
}

impl StateInvariants for DashboardState {
    fn assert_invariants(&self) {
        // Invariant: worktree_count should be at least 1
        assert!(self.worktree_count > 0, "Worktree count must be at least 1");
    }
}
