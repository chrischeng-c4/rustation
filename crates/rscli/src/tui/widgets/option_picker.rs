//! Option picker widget for selecting from structured choices

use crate::tui::protocol::SelectOptionItem;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};
use std::collections::HashSet;

/// Option picker widget for selecting from a list of options
#[derive(Debug, Clone)]
pub struct OptionPicker {
    /// Prompt text to display
    pub prompt: String,
    /// Available options
    pub options: Vec<SelectOptionItem>,
    /// Currently highlighted index
    pub selected_index: usize,
    /// Allow multiple selections
    pub multi_select: bool,
    /// Selected option IDs (for multi-select)
    pub selected_ids: HashSet<String>,
    /// Whether the picker is active
    pub active: bool,
}

impl OptionPicker {
    /// Create a new option picker
    pub fn new(prompt: String, options: Vec<SelectOptionItem>) -> Self {
        Self {
            prompt,
            options,
            selected_index: 0,
            multi_select: false,
            selected_ids: HashSet::new(),
            active: true,
        }
    }

    /// Create with multi-select enabled
    pub fn with_multi_select(prompt: String, options: Vec<SelectOptionItem>) -> Self {
        Self {
            prompt,
            options,
            selected_index: 0,
            multi_select: true,
            selected_ids: HashSet::new(),
            active: true,
        }
    }

    /// Set the default selection by ID
    pub fn set_default(&mut self, id: &str) {
        if let Some(idx) = self.options.iter().position(|o| o.id == id) {
            self.selected_index = idx;
            if self.multi_select {
                self.selected_ids.insert(id.to_string());
            }
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_index < self.options.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Toggle selection (for multi-select)
    pub fn toggle_current(&mut self) {
        if self.multi_select {
            if let Some(option) = self.options.get(self.selected_index) {
                let id = option.id.clone();
                if self.selected_ids.contains(&id) {
                    self.selected_ids.remove(&id);
                } else {
                    self.selected_ids.insert(id);
                }
            }
        }
    }

    /// Submit the selection and return the selected ID(s)
    pub fn submit(&self) -> String {
        if self.multi_select {
            self.selected_ids
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(",")
        } else {
            self.options
                .get(self.selected_index)
                .map(|o| o.id.clone())
                .unwrap_or_default()
        }
    }

    /// Cancel the picker
    pub fn cancel(&mut self) {
        self.active = false;
    }

    /// Check if any option is selected
    pub fn has_selection(&self) -> bool {
        if self.multi_select {
            !self.selected_ids.is_empty()
        } else {
            !self.options.is_empty()
        }
    }

    /// Get the currently highlighted option
    pub fn current_option(&self) -> Option<&SelectOptionItem> {
        self.options.get(self.selected_index)
    }
}

impl Widget for &OptionPicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || self.options.is_empty() {
            return;
        }

        let mut y = area.y;

        // Render prompt on first line
        let prompt_line = Line::from(vec![
            Span::styled(
                &self.prompt,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if self.multi_select {
                    " [Space: toggle | Enter: confirm | Esc: cancel]"
                } else {
                    " [↑↓: navigate | Enter: select | Esc: cancel]"
                },
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        buf.set_line(area.x, y, &prompt_line, area.width);
        y += 1;

        // Render options
        for (i, option) in self.options.iter().enumerate() {
            if y >= area.y + area.height {
                break;
            }

            let is_highlighted = i == self.selected_index;
            let is_selected = self.multi_select && self.selected_ids.contains(&option.id);

            let prefix = if is_highlighted {
                if is_selected { "▶ [✓] " } else { "▶ [ ] " }
            } else if is_selected {
                "  [✓] "
            } else if self.multi_select {
                "  [ ] "
            } else {
                "  "
            };

            let style = if is_highlighted {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let mut spans = vec![
                Span::styled(prefix, style),
                Span::styled(&option.label, style),
            ];

            // Add description if present
            if let Some(ref desc) = option.description {
                spans.push(Span::styled(
                    format!(" - {}", desc),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let line = Line::from(spans);
            buf.set_line(area.x, y, &line, area.width);
            y += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_options() -> Vec<SelectOptionItem> {
        vec![
            SelectOptionItem {
                id: "a".to_string(),
                label: "Option A".to_string(),
                description: Some("First option".to_string()),
            },
            SelectOptionItem {
                id: "b".to_string(),
                label: "Option B".to_string(),
                description: None,
            },
            SelectOptionItem {
                id: "c".to_string(),
                label: "Option C".to_string(),
                description: Some("Third option".to_string()),
            },
        ]
    }

    #[test]
    fn test_new() {
        let picker = OptionPicker::new("Select:".to_string(), test_options());
        assert_eq!(picker.selected_index, 0);
        assert!(!picker.multi_select);
        assert!(picker.active);
    }

    #[test]
    fn test_navigation() {
        let mut picker = OptionPicker::new("Select:".to_string(), test_options());
        assert_eq!(picker.selected_index, 0);

        picker.move_down();
        assert_eq!(picker.selected_index, 1);

        picker.move_down();
        assert_eq!(picker.selected_index, 2);

        picker.move_down(); // Should not go beyond last
        assert_eq!(picker.selected_index, 2);

        picker.move_up();
        assert_eq!(picker.selected_index, 1);

        picker.move_up();
        assert_eq!(picker.selected_index, 0);

        picker.move_up(); // Should not go below 0
        assert_eq!(picker.selected_index, 0);
    }

    #[test]
    fn test_single_select_submit() {
        let mut picker = OptionPicker::new("Select:".to_string(), test_options());
        picker.move_down(); // Select option B

        let result = picker.submit();
        assert_eq!(result, "b");
    }

    #[test]
    fn test_multi_select() {
        let mut picker = OptionPicker::with_multi_select("Select:".to_string(), test_options());

        picker.toggle_current(); // Select A
        picker.move_down();
        picker.move_down();
        picker.toggle_current(); // Select C

        assert!(picker.selected_ids.contains("a"));
        assert!(!picker.selected_ids.contains("b"));
        assert!(picker.selected_ids.contains("c"));

        let result = picker.submit();
        // Order may vary due to HashSet
        assert!(result.contains("a"));
        assert!(result.contains("c"));
    }

    #[test]
    fn test_set_default() {
        let mut picker = OptionPicker::new("Select:".to_string(), test_options());
        picker.set_default("b");
        assert_eq!(picker.selected_index, 1);
    }

    #[test]
    fn test_cancel() {
        let mut picker = OptionPicker::new("Select:".to_string(), test_options());
        assert!(picker.active);
        picker.cancel();
        assert!(!picker.active);
    }
}
