//! Minimal TUI dialog for collecting user input in CLI mode
//!
//! This component provides a simple centered input dialog that can be shown
//! when CLI mode needs to collect user input (e.g., from MCP needs_input events).

use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};

/// Minimal TUI dialog for collecting user input in CLI mode
///
/// This dialog shows a centered input field with a prompt message.
/// User can type input, submit with Enter, or cancel with Esc.
pub struct MiniTUIDialog {
    /// Prompt message
    prompt: String,
    /// User input buffer
    input: String,
    /// Cursor position within input
    cursor: usize,
}

impl MiniTUIDialog {
    /// Create a new mini dialog with the given prompt
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            input: String::new(),
            cursor: 0,
        }
    }

    /// Run the dialog and return user input (or None if cancelled)
    ///
    /// This method:
    /// 1. Enters raw mode and alternate screen
    /// 2. Shows the input dialog
    /// 3. Collects user input
    /// 4. Restores terminal state
    /// 5. Returns the input (or None if user pressed Esc)
    pub fn run(mut self) -> io::Result<Option<String>> {
        // Setup terminal
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(
            stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Event loop
        let result = self.event_loop(&mut terminal);

        // Restore terminal (always execute, even on error)
        let restore_result = self.restore_terminal(&mut terminal);

        // Return first error if any
        result.and(restore_result)
    }

    /// Main event loop - handle user input until Enter or Esc
    fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<Option<String>> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let CrosstermEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        return Ok(Some(self.input.clone()));
                    }
                    KeyCode::Esc => {
                        return Ok(None);
                    }
                    KeyCode::Char(c) => {
                        self.input.insert(self.cursor, c);
                        self.cursor += 1;
                    }
                    KeyCode::Backspace => {
                        if self.cursor > 0 {
                            self.input.remove(self.cursor - 1);
                            self.cursor -= 1;
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor < self.input.len() {
                            self.input.remove(self.cursor);
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor < self.input.len() {
                            self.cursor += 1;
                        }
                    }
                    KeyCode::Home => {
                        self.cursor = 0;
                    }
                    KeyCode::End => {
                        self.cursor = self.input.len();
                    }
                    _ => {}
                }
            }
        }
    }

    /// Restore terminal to normal state
    fn restore_terminal(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<Option<String>> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(None)
    }

    /// Render the dialog
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Center the dialog (60% width, 7 lines height)
        let dialog_area = Self::centered_rect(60, 7, area);

        // Layout: Prompt (1 line) + Spacer (1 line) + Input (3 lines) + Help (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Prompt
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Input field
                Constraint::Length(1), // Help text
            ])
            .split(dialog_area);

        // Render prompt
        let prompt_widget = Paragraph::new(self.prompt.as_str())
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        frame.render_widget(prompt_widget, chunks[0]);

        // Render input field
        let input_widget = Paragraph::new(self.input.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("MCP Input Request")
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::White));
        frame.render_widget(input_widget, chunks[2]);

        // Render help text
        let help_text = "Enter to submit • Esc to cancel";
        let help_widget = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help_widget, chunks[3]);

        // Set cursor position (inside the input box)
        frame.set_cursor_position((
            chunks[2].x + 1 + self.cursor as u16,
            chunks[2].y + 1,
        ));
    }

    /// Helper to create centered rectangle
    ///
    /// # Arguments
    /// * `percent_x` - Width as percentage of total width (0-100)
    /// * `height` - Absolute height in lines
    /// * `area` - Available area to center within
    fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
        let vertical_margin = (area.height.saturating_sub(height)) / 2;
        let horizontal_margin = (area.width * (100 - percent_x) / 100) / 2;

        Rect {
            x: area.x + horizontal_margin,
            y: area.y + vertical_margin,
            width: area.width.saturating_sub(horizontal_margin * 2),
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mini_dialog_creation() {
        let dialog = MiniTUIDialog::new("Test prompt".to_string());
        assert_eq!(dialog.prompt, "Test prompt");
        assert_eq!(dialog.input, "");
        assert_eq!(dialog.cursor, 0);
    }

    #[test]
    fn test_centered_rect_calculation() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        let centered = MiniTUIDialog::centered_rect(60, 7, area);

        // 60% width = 60, centered at x=20
        assert_eq!(centered.width, 60);
        assert_eq!(centered.x, 20);

        // 7 lines height, centered at y=(50-7)/2=21
        assert_eq!(centered.height, 7);
        assert_eq!(centered.y, 21);
    }

    #[test]
    fn test_centered_rect_small_area() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 10,
        };
        let centered = MiniTUIDialog::centered_rect(80, 5, area);

        // 80% of 40 = 32, margin = (40-32)/2 = 4
        assert_eq!(centered.width, 32);
        assert_eq!(centered.x, 4);

        // 5 lines in 10 total, margin = (10-5)/2 = 2
        assert_eq!(centered.height, 5);
        assert_eq!(centered.y, 2);
    }

    #[test]
    fn test_centered_rect_exact_fit() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 10,
        };
        let centered = MiniTUIDialog::centered_rect(100, 10, area);

        // 100% width = full width
        assert_eq!(centered.width, 50);
        assert_eq!(centered.x, 0);

        // Exact height fit
        assert_eq!(centered.height, 10);
        assert_eq!(centered.y, 0);
    }
}
