//! State management for rstn TUI
//!
//! This module implements Feature 079's core principle:
//! "At any time, at any moment, rstn's entire state MUST be representable as JSON/YAML."
//!
//! ## Architecture
//!
//! - **State**: Single source of truth (JSON/YAML serializable)
//! - **UI**: Pure function of state (`UI = render(State)`)
//! - **Testing**: State tests (95%) vs UI tests (5%)
//!
//! ## State Types
//!
//! - `AppState`: Top-level application state
//! - `WorktreeViewState`: Worktree view state
//! - (More views to be added in future phases)

pub mod builders;
pub mod dashboard;
pub mod prompt_claude;
pub mod session_history;
pub mod settings;
pub mod workflow;
pub mod worktree;

use serde::{Deserialize, Serialize};

/// Trait for types that have state invariants
///
/// Invariants are consistency rules that must always hold for valid state.
/// Implementing this trait allows state to be validated after deserialization
/// or mutation.
///
/// # Example
///
/// ```rust
/// impl StateInvariants for WorktreeViewState {
///     fn assert_invariants(&self) {
///         // Invariant: selection within bounds
///         if let Some(selected) = self.command_state_index {
///             assert!(selected < self.commands.len(),
///                 "Selection out of bounds");
///         }
///     }
/// }
/// ```
pub trait StateInvariants {
    /// Assert all invariants hold for this state
    ///
    /// # Panics
    ///
    /// Panics if any invariant is violated
    fn assert_invariants(&self);
}

/// Application-wide state
///
/// This wraps all view states and provides session persistence.
/// Phase 4: Complete with all view states (Worktree, Dashboard, Settings, SessionHistory)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    /// State schema version (e.g., "0.3.0")
    pub version: String,

    /// Worktree view state
    pub worktree_view: worktree::WorktreeViewState,

    /// Dashboard view state
    pub dashboard_view: dashboard::DashboardState,

    /// Settings view state
    pub settings_view: settings::SettingsState,

    /// Session history view state
    pub session_history_view: session_history::SessionHistoryState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            worktree_view: worktree::WorktreeViewState::default(),
            dashboard_view: dashboard::DashboardState::default(),
            settings_view: settings::SettingsState::default(),
            session_history_view: session_history::SessionHistoryState::default(),
        }
    }
}

impl StateInvariants for AppState {
    fn assert_invariants(&self) {
        // Delegate to all view states
        self.worktree_view.assert_invariants();
        self.dashboard_view.assert_invariants();
        self.settings_view.assert_invariants();
        self.session_history_view.assert_invariants();
    }
}

impl AppState {
    /// Save state to a file in JSON format
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rstn::tui::state::AppState;
    /// use std::path::Path;
    ///
    /// let state = AppState::default();
    /// state.save_to_file(Path::new("state.json")).unwrap();
    /// ```
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Save state to a file in YAML format
    pub fn save_to_yaml_file(
        &self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Load state from a file (auto-detects JSON or YAML)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rstn::tui::state::AppState;
    /// use std::path::Path;
    ///
    /// let state = AppState::load_from_file(Path::new("state.json")).unwrap();
    /// ```
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;

        // Try JSON first
        if let Ok(state) = serde_json::from_str::<AppState>(&content) {
            // Verify version compatibility
            Self::check_version(&state.version)?;
            return Ok(state);
        }

        // Try YAML
        if let Ok(state) = serde_yaml::from_str::<AppState>(&content) {
            Self::check_version(&state.version)?;
            return Ok(state);
        }

        Err("Failed to parse state file as JSON or YAML".into())
    }

    /// Check if the state version is compatible with the current version
    fn check_version(state_version: &str) -> Result<(), Box<dyn std::error::Error>> {
        let current_version = env!("CARGO_PKG_VERSION");

        if state_version != current_version {
            eprintln!(
                "Warning: State version mismatch. State: {}, Current: {}",
                state_version, current_version
            );
            // For Phase 2, we allow version mismatch with a warning
            // Phase 5 will implement proper migration logic
        }

        Ok(())
    }

    /// Get the current state schema version
    pub fn schema_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
