//! Text input widget for interactive user input

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

/// Text input widget for capturing user input
#[derive(Debug, Clone)]
pub struct TextInput {
    /// Current text value
    pub value: String,
    /// Cursor position (character index)
    pub cursor_position: usize,
    /// Prompt text to display before input field
    pub prompt: String,
    /// Placeholder text when value is empty
    pub placeholder: String,
    /// Whether the input is currently active/focused
    pub active: bool,
}

impl TextInput {
    /// Create a new text input with a prompt
    pub fn new(prompt: String) -> Self {
        Self {
            value: String::new(),
            cursor_position: 0,
            prompt,
            placeholder: String::new(),
            active: true,
        }
    }

    /// Create a text input with prompt and placeholder
    pub fn with_placeholder(prompt: String, placeholder: String) -> Self {
        Self {
            value: String::new(),
            cursor_position: 0,
            prompt,
            placeholder,
            active: true,
        }
    }

    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        // Ensure cursor is within bounds
        let insert_pos = self.cursor_position.min(self.value.len());
        self.value.insert(insert_pos, c);
        self.cursor_position = insert_pos + 1;
    }

    /// Delete the character before the cursor (backspace)
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 && !self.value.is_empty() {
            let delete_pos = self.cursor_position - 1;
            if delete_pos < self.value.len() {
                self.value.remove(delete_pos);
                self.cursor_position = delete_pos;
            }
        }
    }

    /// Delete the character at the cursor (delete key)
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start of line
    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end of line
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.value.len();
    }

    /// Submit the input and return the value
    pub fn submit(&mut self) -> String {
        let value = self.value.clone();
        self.value.clear();
        self.cursor_position = 0;
        value
    }

    /// Cancel the input and clear the value
    pub fn cancel(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
        self.active = false;
    }

    /// Set the value programmatically
    pub fn set_value(&mut self, value: String) {
        self.cursor_position = value.len();
        self.value = value;
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if the input is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl Widget for &TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let mut spans = Vec::new();

        // Add prompt
        spans.push(Span::styled(
            format!("{} ", self.prompt),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));

        // Add input value or placeholder
        if self.value.is_empty() && !self.placeholder.is_empty() {
            // Show placeholder
            spans.push(Span::styled(
                &self.placeholder,
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            ));
        } else {
            // Show value with cursor
            let before_cursor = self.value.chars().take(self.cursor_position).collect::<String>();
            let cursor_char = self
                .value
                .chars()
                .nth(self.cursor_position)
                .unwrap_or(' ');
            let after_cursor = self
                .value
                .chars()
                .skip(self.cursor_position + 1)
                .collect::<String>();

            // Text before cursor
            if !before_cursor.is_empty() {
                spans.push(Span::styled(
                    before_cursor,
                    Style::default().fg(Color::White),
                ));
            }

            // Cursor character (inverted)
            spans.push(Span::styled(
                cursor_char.to_string(),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));

            // Text after cursor
            if !after_cursor.is_empty() {
                spans.push(Span::styled(
                    after_cursor,
                    Style::default().fg(Color::White),
                ));
            }
        }

        // Add help text
        spans.push(Span::styled(
            " [Enter: submit | Esc: cancel]",
            Style::default().fg(Color::DarkGray),
        ));

        let line = Line::from(spans);

        // Render at the first row of the area
        buf.set_line(area.x, area.y, &line, area.width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_char() {
        let mut input = TextInput::new("Prompt:".to_string());
        input.insert_char('a');
        assert_eq!(input.value(), "a");
        assert_eq!(input.cursor_position, 1);

        input.insert_char('b');
        assert_eq!(input.value(), "ab");
        assert_eq!(input.cursor_position, 2);
    }

    #[test]
    fn test_delete_char() {
        let mut input = TextInput::new("Prompt:".to_string());
        input.set_value("hello".to_string());
        assert_eq!(input.cursor_position, 5);

        input.delete_char();
        assert_eq!(input.value(), "hell");
        assert_eq!(input.cursor_position, 4);

        input.delete_char();
        assert_eq!(input.value(), "hel");
        assert_eq!(input.cursor_position, 3);
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = TextInput::new("Prompt:".to_string());
        input.set_value("hello".to_string());
        assert_eq!(input.cursor_position, 5);

        input.move_cursor_left();
        assert_eq!(input.cursor_position, 4);

        input.move_cursor_left();
        assert_eq!(input.cursor_position, 3);

        input.move_cursor_right();
        assert_eq!(input.cursor_position, 4);

        input.move_cursor_home();
        assert_eq!(input.cursor_position, 0);

        input.move_cursor_end();
        assert_eq!(input.cursor_position, 5);
    }

    #[test]
    fn test_insert_middle() {
        let mut input = TextInput::new("Prompt:".to_string());
        input.set_value("helo".to_string());
        input.cursor_position = 2; // Between 'e' and 'l'

        input.insert_char('l');
        assert_eq!(input.value(), "hello");
        assert_eq!(input.cursor_position, 3);
    }

    #[test]
    fn test_submit() {
        let mut input = TextInput::new("Prompt:".to_string());
        input.set_value("test value".to_string());

        let value = input.submit();
        assert_eq!(value, "test value");
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor_position, 0);
    }

    #[test]
    fn test_cancel() {
        let mut input = TextInput::new("Prompt:".to_string());
        input.set_value("test value".to_string());

        input.cancel();
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor_position, 0);
        assert!(!input.active);
    }

    #[test]
    fn test_placeholder() {
        let input = TextInput::with_placeholder(
            "Enter:".to_string(),
            "Type something...".to_string(),
        );
        assert_eq!(input.placeholder, "Type something...");
        assert!(input.is_empty());
    }
}
