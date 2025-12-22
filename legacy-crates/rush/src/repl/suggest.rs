// suggest.rs - History-based autosuggestions for rush shell
//
// This module implements the Hinter trait from reedline to provide fish-like
// autosuggestions based on command history. Suggestions are displayed as
// grayed-out text after the cursor and can be accepted with Right Arrow.

use nu_ansi_term::{Color, Style};
use reedline::{Hinter, History, SearchQuery};

/// Provides history-based autosuggestions for the rush shell.
///
/// `RushHinter` implements reedline's `Hinter` trait to suggest commands from
/// history as the user types. Suggestions are displayed as dimmed text and can
/// be accepted with Right Arrow (full suggestion) or Alt+Right Arrow (word-by-word).
///
/// # Behavior
///
/// - Only suggests when cursor is at end of line
/// - Searches history in reverse chronological order (most recent first)
/// - Returns the most recent command that starts with current input
/// - Handles empty history and no-match scenarios gracefully
///
/// # Example
///
/// ```no_run
/// use rush::repl::suggest::RushHinter;
/// use reedline::Reedline;
///
/// let hinter = Box::new(RushHinter::new());
/// let editor = Reedline::create()
///     .with_hinter(hinter);
/// ```
pub struct RushHinter {
    /// Current hint text (stored for complete_hint and next_hint_token)
    current_hint: String,
    /// Style for rendering hints
    style: Style,
}

impl RushHinter {
    /// Creates a new `RushHinter` instance.
    ///
    /// The hinter uses a light gray style for suggestions.
    ///
    /// # Returns
    ///
    /// A new `RushHinter` ready to provide suggestions.
    ///
    /// # Example
    ///
    /// ```
    /// use rush::repl::suggest::RushHinter;
    ///
    /// let hinter = RushHinter::new();
    /// ```
    pub fn new() -> Self {
        Self { current_hint: String::new(), style: Style::new().fg(Color::DarkGray).dimmed() }
    }
}

impl Default for RushHinter {
    fn default() -> Self {
        Self::new()
    }
}

impl Hinter for RushHinter {
    /// Provides a suggestion for the current line and cursor position.
    ///
    /// This is called by reedline on every keystroke to update suggestions.
    ///
    /// # Arguments
    ///
    /// * `line` - The current input buffer content
    /// * `pos` - The current cursor position (0-indexed)
    /// * `history` - Access to command history
    /// * `use_ansi_coloring` - Whether to apply color styling
    /// * `_cwd` - Current working directory (unused)
    ///
    /// # Returns
    ///
    /// Formatted suggestion string (empty if no suggestion)
    fn handle(
        &mut self,
        line: &str,
        pos: usize,
        history: &dyn History,
        use_ansi_coloring: bool,
        _cwd: &str,
    ) -> String {
        // Only suggest when cursor is at end of line
        if pos != line.len() {
            self.current_hint.clear();
            return String::new();
        }

        // Don't suggest for empty input
        if line.is_empty() {
            self.current_hint.clear();
            return String::new();
        }

        // Search history for most recent match
        self.current_hint = history
            .search(SearchQuery::last_with_prefix(line.to_string(), history.session()))
            .unwrap_or_default()
            .first()
            .and_then(|entry| {
                let cmd = &entry.command_line;
                // Skip exact matches
                if cmd == line {
                    None
                } else {
                    // Return suffix (everything after the input)
                    cmd.get(line.len()..).map(|s| s.to_string())
                }
            })
            .unwrap_or_default();

        // Apply styling if requested and hint is not empty
        if use_ansi_coloring && !self.current_hint.is_empty() {
            self.style.paint(&self.current_hint).to_string()
        } else {
            self.current_hint.clone()
        }
    }

    /// Return the current hint unformatted for full completion
    fn complete_hint(&self) -> String {
        self.current_hint.clone()
    }

    /// Return the first token of the hint for incremental completion
    fn next_hint_token(&self) -> String {
        // Return the first whitespace-delimited token
        self.current_hint
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let hinter = RushHinter::new();
        assert_eq!(hinter.current_hint, "");
    }

    #[test]
    fn test_default() {
        let hinter = RushHinter::default();
        assert_eq!(hinter.current_hint, "");
    }
}
