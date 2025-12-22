//! Text input widget for interactive user input

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

/// Text input widget for capturing user input
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextInput {
    /// Current text value (single-line mode)
    pub value: String,
    /// Cursor position (character index, single-line mode)
    pub cursor_position: usize,
    /// Prompt text to display before input field
    pub prompt: String,
    /// Placeholder text when value is empty
    pub placeholder: String,
    /// Whether the input is currently active/focused
    pub active: bool,
    /// Whether multiline mode is enabled
    pub multiline: bool,
    /// Line buffer (multiline mode)
    pub lines: Vec<String>,
    /// Current line index (multiline mode)
    pub cursor_line: usize,
    /// Current column in line (multiline mode)
    pub cursor_column: usize,
    /// Maximum lines before scrolling (multiline mode)
    pub max_lines: usize,
    /// Vertical scroll offset (multiline mode)
    pub scroll_offset: usize,
}

/// Strip ANSI escape codes from text
///
/// Removes sequences like:
/// - `\x1b[32m` (color codes)
/// - `\x1b[1m` (bold)
/// - `\x1b[0m` (reset)
fn strip_ansi_codes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Skip escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // Skip '['
                              // Skip until 'm' (end of color code) or other terminator
                while let Some(c) = chars.next() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
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
            multiline: false,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_column: 0,
            max_lines: 10,
            scroll_offset: 0,
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
            multiline: false,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_column: 0,
            max_lines: 10,
            scroll_offset: 0,
        }
    }

    /// Create a multiline text input with a prompt
    pub fn new_multiline(prompt: String, max_lines: usize) -> Self {
        let mut input = Self::new(prompt);
        input.multiline = true;
        input.max_lines = max_lines;
        input
    }

    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        if self.multiline {
            // Insert into current line at cursor column
            let line = &mut self.lines[self.cursor_line];

            // Validate cursor is at valid UTF-8 boundary
            if !line.is_char_boundary(self.cursor_column) {
                // Find next valid boundary
                self.cursor_column = (0..=line.len())
                    .find(|&pos| line.is_char_boundary(pos) && pos >= self.cursor_column)
                    .unwrap_or(line.len());
            }

            line.insert(self.cursor_column, c);
            self.cursor_column += c.len_utf8();
        } else {
            // Ensure cursor is within bounds
            let insert_pos = self.cursor_position.min(self.value.len());
            self.value.insert(insert_pos, c);
            self.cursor_position = insert_pos + 1;
        }
    }

    /// Insert text at cursor position, stripping ANSI codes
    ///
    /// This method safely inserts multi-character text by:
    /// 1. Stripping ANSI escape codes from the input
    /// 2. Validating UTF-8 boundaries before insertion
    /// 3. Updating cursor position correctly
    ///
    /// Use this for pasting text or inserting strings, especially when
    /// the text may contain ANSI color codes or other escape sequences.
    pub fn insert_text(&mut self, text: &str) {
        // Strip ANSI escape codes
        let clean_text = strip_ansi_codes(text);

        if self.multiline {
            // Insert into current line at cursor
            let line = &mut self.lines[self.cursor_line];

            // Validate cursor is at char boundary
            if !line.is_char_boundary(self.cursor_column) {
                // Find next valid boundary
                self.cursor_column = (0..=line.len())
                    .find(|&pos| line.is_char_boundary(pos) && pos >= self.cursor_column)
                    .unwrap_or(line.len());
            }

            // Insert text
            line.insert_str(self.cursor_column, &clean_text);
            self.cursor_column += clean_text.len();
        } else {
            // Single-line mode
            let insert_pos = self.cursor_position.min(self.value.len());
            self.value.insert_str(insert_pos, &clean_text);
            self.cursor_position = insert_pos + clean_text.chars().count();
        }
    }

    /// Delete the character before the cursor (backspace)
    pub fn delete_char(&mut self) {
        if self.multiline {
            if self.cursor_column > 0 {
                // Delete character before cursor in current line
                self.lines[self.cursor_line].remove(self.cursor_column - 1);
                self.cursor_column -= 1;
            } else if self.cursor_line > 0 {
                // At start of line: merge with previous line
                let current_line = self.lines.remove(self.cursor_line);
                self.cursor_line -= 1;
                self.cursor_column = self.lines[self.cursor_line].len();
                self.lines[self.cursor_line].push_str(&current_line);
                self.adjust_scroll();
            }
        } else if self.cursor_position > 0 && !self.value.is_empty() {
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
        if self.multiline {
            if self.cursor_column > 0 {
                self.cursor_column -= 1;
            } else if self.cursor_line > 0 {
                // Move to end of previous line
                self.cursor_line -= 1;
                self.cursor_column = self.lines[self.cursor_line].len();
                self.adjust_scroll();
            }
        } else if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.multiline {
            if self.cursor_column < self.lines[self.cursor_line].len() {
                self.cursor_column += 1;
            } else if self.cursor_line < self.lines.len() - 1 {
                // Move to start of next line
                self.cursor_line += 1;
                self.cursor_column = 0;
                self.adjust_scroll();
            }
        } else if self.cursor_position < self.value.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start of line
    pub fn move_cursor_home(&mut self) {
        if self.multiline {
            self.cursor_column = 0;
        } else {
            self.cursor_position = 0;
        }
    }

    /// Move cursor to end of line
    pub fn move_cursor_end(&mut self) {
        if self.multiline {
            self.cursor_column = self.lines[self.cursor_line].len();
        } else {
            self.cursor_position = self.value.len();
        }
    }

    /// Submit the input and return the value
    pub fn submit(&mut self) -> String {
        let value = if self.multiline {
            self.get_multiline_value()
        } else {
            self.value.clone()
        };

        // Reset state
        if self.multiline {
            self.lines = vec![String::new()];
            self.cursor_line = 0;
            self.cursor_column = 0;
            self.scroll_offset = 0;
        } else {
            self.value.clear();
            self.cursor_position = 0;
        }

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

    /// Insert newline at cursor (multiline mode only)
    pub fn insert_newline(&mut self) {
        if !self.multiline {
            return;
        }

        // Split current line at cursor
        let current = &self.lines[self.cursor_line];
        let before = current[..self.cursor_column].to_string();
        let after = current[self.cursor_column..].to_string();

        // Update current line and insert new line
        self.lines[self.cursor_line] = before;
        self.lines.insert(self.cursor_line + 1, after);

        // Move cursor to start of new line
        self.cursor_line += 1;
        self.cursor_column = 0;

        // Adjust scroll if needed
        self.adjust_scroll();
    }

    /// Move cursor up one line (multiline mode)
    pub fn move_cursor_up(&mut self) {
        if !self.multiline || self.cursor_line == 0 {
            return;
        }

        self.cursor_line -= 1;
        // Clamp column to new line length
        self.cursor_column = self.cursor_column.min(self.lines[self.cursor_line].len());
        self.adjust_scroll();
    }

    /// Move cursor down one line (multiline mode)
    pub fn move_cursor_down(&mut self) {
        if !self.multiline || self.cursor_line >= self.lines.len() - 1 {
            return;
        }

        self.cursor_line += 1;
        // Clamp column to new line length
        self.cursor_column = self.cursor_column.min(self.lines[self.cursor_line].len());
        self.adjust_scroll();
    }

    /// Adjust scroll to keep cursor visible
    fn adjust_scroll(&mut self) {
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + self.max_lines {
            self.scroll_offset = self.cursor_line - self.max_lines + 1;
        }
    }

    /// Get multiline content as single string (joined with \n)
    pub fn get_multiline_value(&self) -> String {
        if self.multiline {
            self.lines.join("\n")
        } else {
            self.value.clone()
        }
    }
}

impl Widget for &TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        if self.multiline {
            self.render_multiline(area, buf);
        } else {
            self.render_singleline(area, buf);
        }
    }
}

impl TextInput {
    /// Render single-line mode
    fn render_singleline(&self, area: Rect, buf: &mut Buffer) {
        let mut spans = Vec::new();

        // Add prompt
        spans.push(Span::styled(
            format!("{} ", self.prompt),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        // Add input value or placeholder
        if self.value.is_empty() && !self.placeholder.is_empty() {
            // Show placeholder
            spans.push(Span::styled(
                &self.placeholder,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ));
        } else {
            // Show value with cursor
            let before_cursor = self
                .value
                .chars()
                .take(self.cursor_position)
                .collect::<String>();
            let cursor_char = self.value.chars().nth(self.cursor_position).unwrap_or(' ');
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

    /// Render multiline mode
    fn render_multiline(&self, area: Rect, buf: &mut Buffer) {
        // Calculate visible lines
        let visible_height = area.height as usize;
        let visible_lines = self
            .lines
            .iter()
            .skip(self.scroll_offset)
            .take(visible_height)
            .enumerate();

        for (idx, line) in visible_lines {
            let y = area.y + idx as u16;
            let line_idx = self.scroll_offset + idx;

            // Render line with cursor if this is the active line
            if line_idx == self.cursor_line && self.active {
                self.render_line_with_cursor(area, buf, y, line);
            } else {
                buf.set_string(area.x, y, line, Style::default().fg(Color::White));
            }
        }
    }

    /// Render a single line with cursor
    fn render_line_with_cursor(&self, area: Rect, buf: &mut Buffer, y: u16, line: &str) {
        // Build text with cursor
        let before = line.chars().take(self.cursor_column).collect::<String>();
        let cursor_char = line.chars().nth(self.cursor_column).unwrap_or(' ');
        let after = line
            .chars()
            .skip(self.cursor_column + 1)
            .collect::<String>();

        let mut x_offset = 0;

        // Render text before cursor
        if !before.is_empty() {
            buf.set_string(area.x, y, &before, Style::default().fg(Color::White));
            x_offset += before.len() as u16;
        }

        // Render cursor
        buf.set_string(
            area.x + x_offset,
            y,
            cursor_char.to_string(),
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
        x_offset += 1;

        // Render text after cursor
        if !after.is_empty() {
            buf.set_string(
                area.x + x_offset,
                y,
                &after,
                Style::default().fg(Color::White),
            );
        }
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
        let input =
            TextInput::with_placeholder("Enter:".to_string(), "Type something...".to_string());
        assert_eq!(input.placeholder, "Type something...");
        assert!(input.is_empty());
    }

    #[test]
    fn test_multiline_insert_newline() {
        let mut input = TextInput::new_multiline("Prompt:".to_string(), 10);
        assert!(input.multiline);
        assert_eq!(input.lines.len(), 1);

        // Type "hello"
        input.insert_char('h');
        input.insert_char('e');
        input.insert_char('l');
        input.insert_char('l');
        input.insert_char('o');
        assert_eq!(input.lines[0], "hello");
        assert_eq!(input.cursor_line, 0);
        assert_eq!(input.cursor_column, 5);

        // Insert newline
        input.insert_newline();
        assert_eq!(input.lines.len(), 2);
        assert_eq!(input.lines[0], "hello");
        assert_eq!(input.lines[1], "");
        assert_eq!(input.cursor_line, 1);
        assert_eq!(input.cursor_column, 0);

        // Type "world"
        input.insert_char('w');
        input.insert_char('o');
        input.insert_char('r');
        input.insert_char('l');
        input.insert_char('d');
        assert_eq!(input.lines[1], "world");
        assert_eq!(input.cursor_column, 5);
    }

    #[test]
    fn test_multiline_navigation() {
        let mut input = TextInput::new_multiline("Prompt:".to_string(), 10);

        // Create 3 lines
        input.insert_char('l');
        input.insert_char('1');
        input.insert_newline();
        input.insert_char('l');
        input.insert_char('2');
        input.insert_newline();
        input.insert_char('l');
        input.insert_char('3');

        assert_eq!(input.cursor_line, 2);
        assert_eq!(input.cursor_column, 2);

        // Move up
        input.move_cursor_up();
        assert_eq!(input.cursor_line, 1);
        assert_eq!(input.cursor_column, 2);

        input.move_cursor_up();
        assert_eq!(input.cursor_line, 0);
        assert_eq!(input.cursor_column, 2);

        // Move up at top (should stay at top)
        input.move_cursor_up();
        assert_eq!(input.cursor_line, 0);

        // Move down
        input.move_cursor_down();
        assert_eq!(input.cursor_line, 1);

        input.move_cursor_down();
        assert_eq!(input.cursor_line, 2);

        // Move down at bottom (should stay at bottom)
        input.move_cursor_down();
        assert_eq!(input.cursor_line, 2);
    }

    #[test]
    fn test_multiline_backspace_merge_lines() {
        let mut input = TextInput::new_multiline("Prompt:".to_string(), 10);

        // Create "hello\nworld"
        input.insert_char('h');
        input.insert_char('e');
        input.insert_char('l');
        input.insert_char('l');
        input.insert_char('o');
        input.insert_newline();
        input.insert_char('w');
        input.insert_char('o');
        input.insert_char('r');
        input.insert_char('l');
        input.insert_char('d');

        assert_eq!(input.lines.len(), 2);
        assert_eq!(input.lines[0], "hello");
        assert_eq!(input.lines[1], "world");
        assert_eq!(input.cursor_line, 1);
        assert_eq!(input.cursor_column, 5);

        // Move to start of second line
        input.move_cursor_home();
        assert_eq!(input.cursor_column, 0);

        // Backspace should merge lines
        input.delete_char();
        assert_eq!(input.lines.len(), 1);
        assert_eq!(input.lines[0], "helloworld");
        assert_eq!(input.cursor_line, 0);
        assert_eq!(input.cursor_column, 5);
    }

    #[test]
    fn test_multiline_scroll() {
        let mut input = TextInput::new_multiline("Prompt:".to_string(), 3); // max 3 lines

        // Create 5 lines
        for i in 0..5 {
            if i > 0 {
                input.insert_newline();
            }
            input.insert_char('l');
            input.insert_char(char::from_digit(i, 10).unwrap());
        }

        assert_eq!(input.lines.len(), 5);
        assert_eq!(input.cursor_line, 4);

        // Scroll should have adjusted to show last 3 lines
        assert_eq!(input.scroll_offset, 2); // Lines 2, 3, 4 visible

        // Move up should adjust scroll
        input.move_cursor_up();
        input.move_cursor_up();
        assert_eq!(input.cursor_line, 2);
        assert_eq!(input.scroll_offset, 2); // Still at offset 2

        input.move_cursor_up();
        assert_eq!(input.cursor_line, 1);
        assert_eq!(input.scroll_offset, 1); // Adjusted to show lines 1, 2, 3
    }

    #[test]
    fn test_multiline_submit() {
        let mut input = TextInput::new_multiline("Prompt:".to_string(), 10);

        // Create "line1\nline2\nline3"
        input.insert_char('l');
        input.insert_char('i');
        input.insert_char('n');
        input.insert_char('e');
        input.insert_char('1');
        input.insert_newline();
        input.insert_char('l');
        input.insert_char('i');
        input.insert_char('n');
        input.insert_char('e');
        input.insert_char('2');
        input.insert_newline();
        input.insert_char('l');
        input.insert_char('i');
        input.insert_char('n');
        input.insert_char('e');
        input.insert_char('3');

        let value = input.get_multiline_value();
        assert_eq!(value, "line1\nline2\nline3");

        // Submit should reset
        let submitted = input.submit();
        assert_eq!(submitted, "line1\nline2\nline3");
        assert_eq!(input.lines.len(), 1);
        assert_eq!(input.lines[0], "");
        assert_eq!(input.cursor_line, 0);
        assert_eq!(input.cursor_column, 0);
    }
}
