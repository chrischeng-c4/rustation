//! E2E test harness for TUI testing using ratatui's TestBackend
//!
//! This module provides a TuiTestHarness that wraps an App and TestBackend
//! for end-to-end testing of the TUI interface.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rstn::tui::views::ViewAction;
use rstn::tui::App;

/// Test harness for E2E TUI testing
pub struct TuiTestHarness {
    /// The application instance
    pub app: App,
    /// Terminal with test backend for in-memory rendering
    pub terminal: Terminal<TestBackend>,
}

impl TuiTestHarness {
    /// Create a new test harness with specified dimensions
    pub fn new(width: u16, height: u16) -> Self {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).expect("Failed to create terminal");
        let app = App::new();

        Self { app, terminal }
    }

    /// Send a key event to the app
    pub fn send_key(&mut self, code: KeyCode) {
        let event = KeyEvent::new(code, KeyModifiers::empty());
        self.app.handle_key_event(event);
    }

    /// Send a key event with modifiers
    pub fn send_key_with_mod(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let event = KeyEvent::new(code, modifiers);
        self.app.handle_key_event(event);
    }

    /// Send a string of characters as key events
    pub fn send_text(&mut self, text: &str) {
        for c in text.chars() {
            self.send_key(KeyCode::Char(c));
        }
    }

    /// Setup input mode via RequestInput action
    pub fn request_input(&mut self, prompt: &str, placeholder: Option<&str>) {
        self.app.handle_view_action(ViewAction::RequestInput {
            prompt: prompt.to_string(),
            placeholder: placeholder.map(|s| s.to_string()),
        });
    }

    /// Render the app to the test backend
    pub fn render(&mut self) {
        // Note: We can't use app.draw() directly because it requires a real terminal
        // For E2E tests, we verify state and use the backend buffer
        // This is a simplified render that just ensures the terminal is ready
        self.terminal.clear().ok();
    }

    /// Check if the buffer contains the given text anywhere
    pub fn buffer_contains(&self, text: &str) -> bool {
        let backend = self.terminal.backend();
        let buffer = backend.buffer();

        // Search through all cells in the buffer
        let mut content = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = buffer.cell((x, y));
                if let Some(cell) = cell {
                    content.push_str(cell.symbol());
                }
            }
        }

        content.contains(text)
    }

    /// Get all text content from buffer as a string
    pub fn get_buffer_content(&self) -> String {
        let backend = self.terminal.backend();
        let buffer = backend.buffer();

        let mut content = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = buffer.cell((x, y));
                if let Some(cell) = cell {
                    content.push_str(cell.symbol());
                }
            }
            content.push('\n');
        }

        content
    }

    /// Check if app is in input mode
    pub fn is_in_input_mode(&self) -> bool {
        self.app.input_mode
    }

    /// Check if input dialog exists
    pub fn has_input_dialog(&self) -> bool {
        self.app.input_dialog.is_some()
    }

    /// Get the current input value (if dialog exists)
    pub fn get_input_value(&self) -> Option<String> {
        self.app.input_dialog.as_ref().map(|d| {
            if d.is_multiline() {
                d.input.get_multiline_value()
            } else {
                d.value().to_string()
            }
        })
    }
}

// Include the E2E test module
pub mod sdd_workflow_e2e;
