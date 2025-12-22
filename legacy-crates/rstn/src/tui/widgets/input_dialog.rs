//! Centered input dialog widget for prompting user input
//!
//! Displays a modal dialog in the center of the screen with:
//! - Title/prompt
//! - Text input field
//! - Instructions for submit/cancel

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use super::TextInput;

/// Centered input dialog for capturing user input
#[derive(Debug, Clone)]
pub struct InputDialog {
    /// The underlying text input
    pub input: TextInput,
    /// Dialog title
    pub title: String,
    /// Optional description/context shown above the input
    pub description: Option<String>,
}

impl InputDialog {
    /// Create a new input dialog with title and prompt
    pub fn new(title: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            input: TextInput::new(prompt.into()),
            title: title.into(),
            description: None,
        }
    }

    /// Create with a description shown above the input
    pub fn with_description(
        title: impl Into<String>,
        description: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            input: TextInput::new(prompt.into()),
            title: title.into(),
            description: Some(description.into()),
        }
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.input.placeholder = placeholder.into();
        self
    }

    /// Create multiline input dialog
    pub fn new_multiline(
        title: impl Into<String>,
        prompt: impl Into<String>,
        max_lines: usize,
    ) -> Self {
        Self {
            input: TextInput::new_multiline(prompt.into(), max_lines),
            title: title.into(),
            description: None,
        }
    }

    /// Get the current input value
    pub fn value(&self) -> &str {
        &self.input.value
    }

    /// Forward character insertion to input
    pub fn insert_char(&mut self, c: char) {
        self.input.insert_char(c);
    }

    /// Forward backspace to input
    pub fn delete_char(&mut self) {
        self.input.delete_char();
    }

    /// Forward cursor movement
    pub fn move_cursor_left(&mut self) {
        self.input.move_cursor_left();
    }

    /// Forward cursor movement
    pub fn move_cursor_right(&mut self) {
        self.input.move_cursor_right();
    }

    /// Forward home key
    pub fn move_cursor_start(&mut self) {
        self.input.move_cursor_home();
    }

    /// Forward end key
    pub fn move_cursor_end(&mut self) {
        self.input.move_cursor_end();
    }

    /// Insert newline (multiline mode)
    pub fn insert_newline(&mut self) {
        self.input.insert_newline();
    }

    /// Move cursor up (multiline mode)
    pub fn move_cursor_up(&mut self) {
        self.input.move_cursor_up();
    }

    /// Move cursor down (multiline mode)
    pub fn move_cursor_down(&mut self) {
        self.input.move_cursor_down();
    }

    /// Check if in multiline mode
    pub fn is_multiline(&self) -> bool {
        self.input.multiline
    }

    /// Calculate the dialog area centered in the given area
    pub fn dialog_area(&self, area: Rect) -> Rect {
        // Dialog is 60% width, max 80 chars, min 40 chars
        let width = (area.width as f32 * 0.6) as u16;
        let width = width.max(40).min(80).min(area.width.saturating_sub(4));

        // Calculate input height based on mode
        let input_height = if self.input.multiline {
            self.input.lines.len().min(self.input.max_lines) as u16
        } else {
            1
        };

        // Height: title(1) + border(2) + description(0-3) + input(dynamic) + help(1) + padding(2)
        let desc_lines = self
            .description
            .as_ref()
            .map(|d| {
                // Estimate line count
                (d.len() as u16 / width.saturating_sub(4)).max(1).min(3)
            })
            .unwrap_or(0);
        let height = (4 + desc_lines + input_height + 2).min(area.height.saturating_sub(4));

        // Center the dialog
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;

        Rect::new(x, y, width, height)
    }

    /// Render the dialog to a frame at the calculated centered position
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let dialog_area = self.dialog_area(area);

        // Clear the area behind the dialog
        Clear.render(dialog_area, buf);

        // Create the dialog block
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        // Layout inside the dialog
        let has_desc = self.description.is_some();
        // Calculate input height: for multiline, we need 1 line for prompt + lines for actual input
        let input_height = if self.input.multiline {
            // prompt line + at least 1 input line (capped by max_lines)
            1 + (self.input.max_lines as u16).min(5)
        } else {
            1
        };
        let constraints = if has_desc {
            vec![
                Constraint::Length(3),            // Description
                Constraint::Length(1),            // Spacer
                Constraint::Length(input_height), // Input (dynamic for multiline)
                Constraint::Length(1),            // Spacer
                Constraint::Length(1),            // Help text
            ]
        } else {
            vec![
                Constraint::Length(1),            // Spacer
                Constraint::Length(input_height), // Input (dynamic for multiline)
                Constraint::Length(1),            // Spacer
                Constraint::Length(1),            // Help text
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let mut chunk_idx = 0;

        // Render description if present
        if let Some(ref desc) = self.description {
            let desc_para = Paragraph::new(desc.as_str())
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Left);
            desc_para.render(chunks[chunk_idx], buf);
            chunk_idx += 1;
            chunk_idx += 1; // Skip spacer
        } else {
            chunk_idx += 1; // Skip spacer
        }

        // Render input field
        self.render_input(chunks[chunk_idx], buf);
        chunk_idx += 1;
        chunk_idx += 1; // Skip spacer

        // Render help text
        let help = if self.input.multiline {
            Line::from(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Submit  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Ctrl+Enter",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" New line  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
            ])
        } else {
            Line::from(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Submit  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
            ])
        };
        let help_para = Paragraph::new(help).alignment(Alignment::Center);
        help_para.render(chunks[chunk_idx], buf);
    }

    /// Render just the input field portion
    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        if self.input.multiline {
            // For multiline mode, render the prompt above and use full area for TextInput
            if area.height > 0 {
                // Render prompt on first line
                let prompt_span =
                    Span::styled(&self.input.prompt, Style::default().fg(Color::Yellow));
                buf.set_span(area.x, area.y, &prompt_span, area.width);

                // Render multiline input using TextInput's widget implementation
                if area.height > 1 {
                    let input_area = Rect::new(
                        area.x,
                        area.y + 1,
                        area.width,
                        area.height.saturating_sub(1),
                    );
                    Widget::render(&self.input, input_area, buf);
                }
            }
        } else {
            // Single-line mode: render horizontally
            let prompt_len = self.input.prompt.len() as u16;

            // Split area for prompt and input
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(prompt_len + 1), Constraint::Min(1)])
                .split(area);

            // Render prompt
            let prompt_span = Span::styled(
                format!("{} ", self.input.prompt),
                Style::default().fg(Color::Yellow),
            );
            buf.set_span(chunks[0].x, chunks[0].y, &prompt_span, chunks[0].width);

            // Render input value or placeholder
            let input_area = chunks[1];
            let display_text = if self.input.value.is_empty() {
                &self.input.placeholder
            } else {
                &self.input.value
            };

            let text_style = if self.input.value.is_empty() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            // Render the text
            let visible_width = input_area.width as usize;
            let cursor_pos = self.input.cursor_position;

            // Calculate scroll offset to keep cursor visible
            let scroll_offset = if cursor_pos >= visible_width {
                cursor_pos - visible_width + 1
            } else {
                0
            };

            let visible_text: String = display_text
                .chars()
                .skip(scroll_offset)
                .take(visible_width)
                .collect();

            buf.set_string(input_area.x, input_area.y, &visible_text, text_style);

            // Render cursor
            if self.input.active && !self.input.value.is_empty() {
                let cursor_x = input_area.x + (cursor_pos - scroll_offset) as u16;
                if cursor_x < input_area.x + input_area.width {
                    if let Some(cell) = buf.cell_mut((cursor_x, input_area.y)) {
                        cell.set_style(Style::default().bg(Color::White).fg(Color::Black));
                    }
                }
            } else if self.input.active && self.input.value.is_empty() {
                // Show cursor at start when empty
                if let Some(cell) = buf.cell_mut((input_area.x, input_area.y)) {
                    cell.set_char('_');
                    cell.set_style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::SLOW_BLINK),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dialog = InputDialog::new("Test Title", "Enter value:");
        assert_eq!(dialog.title, "Test Title");
        assert_eq!(dialog.input.prompt, "Enter value:");
        assert!(dialog.description.is_none());
    }

    #[test]
    fn test_with_description() {
        let dialog = InputDialog::with_description("Title", "Some description", "Prompt:");
        assert_eq!(dialog.description, Some("Some description".to_string()));
    }

    #[test]
    fn test_placeholder() {
        let dialog = InputDialog::new("Title", "Prompt:").placeholder("Type here...");
        assert_eq!(dialog.input.placeholder, "Type here...");
    }

    #[test]
    fn test_input_forwarding() {
        let mut dialog = InputDialog::new("Title", ">");
        dialog.insert_char('a');
        dialog.insert_char('b');
        assert_eq!(dialog.value(), "ab");
        dialog.delete_char();
        assert_eq!(dialog.value(), "a");
    }

    // T018: Verify new multiline dialog creates active input
    #[test]
    fn test_new_creates_active_input() {
        let dialog = InputDialog::new("Title", "Prompt:");
        assert!(dialog.input.active, "TextInput should be active by default");

        let multiline_dialog = InputDialog::new_multiline("Title", "Prompt:", 5);
        assert!(
            multiline_dialog.input.active,
            "Multiline TextInput should be active"
        );
        assert!(
            multiline_dialog.input.multiline,
            "Multiline flag should be set"
        );
        assert_eq!(multiline_dialog.input.max_lines, 5);
    }

    // T019: Verify insert_char forwards to input correctly
    #[test]
    fn test_insert_char_forwards_to_input() {
        let mut dialog = InputDialog::new("Title", ">");
        assert_eq!(dialog.value(), "");

        dialog.insert_char('h');
        assert_eq!(dialog.value(), "h");

        dialog.insert_char('i');
        assert_eq!(dialog.value(), "hi");

        // Verify cursor position advances
        assert_eq!(dialog.input.cursor_position, 2);
    }

    // T020: Verify multiline character insertion
    #[test]
    fn test_multiline_insert_char() {
        let mut dialog = InputDialog::new_multiline("Title", ">", 10);

        dialog.insert_char('a');
        dialog.insert_char('b');
        // In multiline mode, use get_multiline_value() or check lines directly
        assert_eq!(dialog.input.lines[0], "ab");
        assert_eq!(dialog.input.get_multiline_value(), "ab");

        // Insert newline and more text
        dialog.insert_newline();
        dialog.insert_char('c');
        dialog.insert_char('d');

        // Value should contain newline
        assert!(dialog.input.get_multiline_value().contains('\n'));
        assert_eq!(dialog.input.lines.len(), 2);
        assert_eq!(dialog.input.lines[0], "ab");
        assert_eq!(dialog.input.lines[1], "cd");
    }

    // T021: Verify all cursor movement methods work
    #[test]
    fn test_cursor_movement_methods() {
        let mut dialog = InputDialog::new("Title", ">");
        dialog.insert_char('a');
        dialog.insert_char('b');
        dialog.insert_char('c');

        // Initial cursor is at end
        assert_eq!(dialog.input.cursor_position, 3);

        // Move left
        dialog.move_cursor_left();
        assert_eq!(dialog.input.cursor_position, 2);

        // Move left again
        dialog.move_cursor_left();
        assert_eq!(dialog.input.cursor_position, 1);

        // Move right
        dialog.move_cursor_right();
        assert_eq!(dialog.input.cursor_position, 2);

        // Move to start
        dialog.move_cursor_start();
        assert_eq!(dialog.input.cursor_position, 0);

        // Move to end
        dialog.move_cursor_end();
        assert_eq!(dialog.input.cursor_position, 3);
    }

    // T022: Verify backspace deletes characters correctly
    #[test]
    fn test_backspace_deletes_char() {
        let mut dialog = InputDialog::new("Title", ">");
        dialog.insert_char('a');
        dialog.insert_char('b');
        dialog.insert_char('c');
        assert_eq!(dialog.value(), "abc");

        // Delete from end
        dialog.delete_char();
        assert_eq!(dialog.value(), "ab");

        // Delete another
        dialog.delete_char();
        assert_eq!(dialog.value(), "a");

        // Delete last char
        dialog.delete_char();
        assert_eq!(dialog.value(), "");

        // Delete on empty should not panic
        dialog.delete_char();
        assert_eq!(dialog.value(), "");
    }

    // T023: Verify multiline newline insertion
    #[test]
    fn test_multiline_newline_insert() {
        let mut dialog = InputDialog::new_multiline("Title", ">", 5);

        // Type first line
        dialog.insert_char('l');
        dialog.insert_char('i');
        dialog.insert_char('n');
        dialog.insert_char('e');
        dialog.insert_char('1');

        // Insert newline
        dialog.insert_newline();
        assert_eq!(dialog.input.lines.len(), 2);
        assert_eq!(dialog.input.cursor_line, 1);

        // Type second line
        dialog.insert_char('l');
        dialog.insert_char('i');
        dialog.insert_char('n');
        dialog.insert_char('e');
        dialog.insert_char('2');

        assert_eq!(dialog.input.lines[0], "line1");
        assert_eq!(dialog.input.lines[1], "line2");
    }

    // Additional: Verify multiline mode detection
    #[test]
    fn test_is_multiline() {
        let single = InputDialog::new("Title", ">");
        assert!(!single.is_multiline());

        let multi = InputDialog::new_multiline("Title", ">", 5);
        assert!(multi.is_multiline());
    }

    // Additional: Verify multiline cursor navigation
    #[test]
    fn test_multiline_cursor_navigation() {
        let mut dialog = InputDialog::new_multiline("Title", ">", 5);

        // Type two lines
        dialog.insert_char('a');
        dialog.insert_char('b');
        dialog.insert_newline();
        dialog.insert_char('c');
        dialog.insert_char('d');

        // Cursor should be on line 1
        assert_eq!(dialog.input.cursor_line, 1);

        // Move up
        dialog.move_cursor_up();
        assert_eq!(dialog.input.cursor_line, 0);

        // Move down
        dialog.move_cursor_down();
        assert_eq!(dialog.input.cursor_line, 1);
    }
}
