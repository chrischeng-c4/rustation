//! Session history view state
//!
//! This module defines the serializable state for Session History view.

use serde::{Deserialize, Serialize};

use super::StateInvariants;

/// Focus area in Session History view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionHistoryFocus {
    /// Focus on session list (left pane)
    List,
    /// Focus on session details/log preview (right pane)
    Details,
}

impl Default for SessionHistoryFocus {
    fn default() -> Self {
        Self::List
    }
}

/// Session history view state
///
/// Contains all serializable fields for the Session History view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionHistoryState {
    /// Selected session index in the list
    pub selected_index: Option<usize>,

    /// Current focus area (List or Details)
    pub focus: SessionHistoryFocus,

    /// Maximum number of sessions to display
    pub max_sessions: usize,

    /// Whether to show log preview in details pane
    pub show_log_preview: bool,

    /// Log preview scroll position (line number)
    pub log_scroll: usize,

    /// Filter by command type (e.g., "prompt", "specify")
    pub filter_type: Option<String>,

    /// Filter by status (e.g., "completed", "active", "error")
    pub filter_status: Option<String>,
}

impl Default for SessionHistoryState {
    fn default() -> Self {
        Self {
            selected_index: Some(0), // Start with first session selected
            focus: SessionHistoryFocus::List,
            max_sessions: 50, // Default limit
            show_log_preview: true,
            log_scroll: 0,
            filter_type: None,
            filter_status: None,
        }
    }
}

impl StateInvariants for SessionHistoryState {
    fn assert_invariants(&self) {
        // Invariant 1: max_sessions should be reasonable (1..=1000)
        assert!(
            self.max_sessions > 0 && self.max_sessions <= 1000,
            "max_sessions must be between 1 and 1000, got: {}",
            self.max_sessions
        );

        // Invariant 2: log_scroll within reasonable bounds (0..100000)
        assert!(
            self.log_scroll < 100000,
            "log_scroll position unreasonably large: {}",
            self.log_scroll
        );

        // Note: We can't validate selected_index bounds here because we don't
        // know the session list length. This will be validated in the View layer.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_valid() {
        let state = SessionHistoryState::default();
        state.assert_invariants();
    }

    #[test]
    fn test_default_values() {
        let state = SessionHistoryState::default();
        assert_eq!(state.selected_index, Some(0));
        assert_eq!(state.focus, SessionHistoryFocus::List);
        assert_eq!(state.max_sessions, 50);
        assert!(state.show_log_preview);
        assert_eq!(state.log_scroll, 0);
        assert_eq!(state.filter_type, None);
        assert_eq!(state.filter_status, None);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let state = SessionHistoryState {
            selected_index: Some(5),
            focus: SessionHistoryFocus::Details,
            max_sessions: 100,
            show_log_preview: false,
            log_scroll: 42,
            filter_type: Some("prompt".to_string()),
            filter_status: Some("completed".to_string()),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&state).unwrap();

        // Deserialize back
        let deserialized: SessionHistoryState = serde_json::from_str(&json).unwrap();

        // Should match original
        assert_eq!(state, deserialized);
        deserialized.assert_invariants();
    }

    #[test]
    #[should_panic(expected = "max_sessions must be between 1 and 1000")]
    fn test_invariant_max_sessions_too_large() {
        let state = SessionHistoryState {
            max_sessions: 5000,
            ..Default::default()
        };
        state.assert_invariants();
    }

    #[test]
    #[should_panic(expected = "max_sessions must be between 1 and 1000")]
    fn test_invariant_max_sessions_zero() {
        let state = SessionHistoryState {
            max_sessions: 0,
            ..Default::default()
        };
        state.assert_invariants();
    }

    #[test]
    #[should_panic(expected = "log_scroll position unreasonably large")]
    fn test_invariant_log_scroll_too_large() {
        let state = SessionHistoryState {
            log_scroll: 200000,
            ..Default::default()
        };
        state.assert_invariants();
    }
}
