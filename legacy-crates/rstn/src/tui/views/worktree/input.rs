//! Inline input widget for Claude follow-up questions
//!
//! This module provides the InlineInput struct which displays prompts
//! directly in the content area instead of a popup dialog.

use crate::tui::widgets::TextInput;

/// Inline input state for Claude follow-up questions
/// Displayed directly in the content area instead of a popup dialog
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InlineInput {
    /// Claude's prompt/question to the user
    pub prompt: String,
    /// Text input widget for user response
    pub text_input: TextInput,
}

impl InlineInput {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            text_input: TextInput::new(String::new()), // Empty prompt for inline input
        }
    }

    /// Get the current input value
    pub fn value(&self) -> &str {
        &self.text_input.value
    }
}
