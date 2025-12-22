//! Settings view state
//!
//! This module defines the serializable state for Settings view.

use crate::settings::Settings;
use serde::{Deserialize, Serialize};

use super::StateInvariants;

/// Settings view state
///
/// Contains all serializable fields for the Settings view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsState {
    /// Current settings configuration
    pub settings: Settings,

    /// Selected menu item index
    pub selected_index: usize,

    /// Current feature number (for session clearing)
    pub current_feature: Option<String>,

    /// Status message
    pub status_message: Option<String>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            selected_index: 0,
            current_feature: None,
            status_message: None,
        }
    }
}

impl StateInvariants for SettingsState {
    fn assert_invariants(&self) {
        // Invariant: selected_index should be within bounds (0..5)
        // There are 5 settings items in SettingsItem::all()
        assert!(
            self.selected_index < 5,
            "Settings selected_index {} out of bounds (max 4)",
            self.selected_index
        );
    }
}
