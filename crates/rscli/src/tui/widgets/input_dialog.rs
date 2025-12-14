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

    /// Calculate the dialog area centered in the given area
    pub fn dialog_area(&self, area: Rect) -> Rect {
        // Dialog is 60% width, max 80 chars, min 40 chars
        let width = (area.width as f32 * 0.6) as u16;
        let width = width.max(40).min(80).min(area.width.saturating_sub(4));

        // Height: title(1) + border(2) + description(0-3) + input(1) + help(1) + padding(2)
        let desc_lines = self.description.as_ref().map(|d| {
            // Estimate line count
            (d.len() as u16 / width.saturating_sub(4)).max(1).min(3)
        }).unwrap_or(0);
        let height = (5 + desc_lines + 2).min(area.height.saturating_sub(4));

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
        let constraints = if has_desc {
            vec![
                Constraint::Length(3), // Description
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Input
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Help text
            ]
        } else {
            vec![
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Input
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Help text
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
        let help = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" Submit  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
        ]);
        let help_para = Paragraph::new(help).alignment(Alignment::Center);
        help_para.render(chunks[chunk_idx], buf);
    }

    /// Render just the input field portion
    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        let prompt_len = self.input.prompt.len() as u16;

        // Split area for prompt and input
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(prompt_len + 1),
                Constraint::Min(1),
            ])
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
                cell.set_style(Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK));
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
        let dialog = InputDialog::with_description(
            "Title",
            "Some description",
            "Prompt:",
        );
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
}
