//! Custom prompt rendering with exit code indicators

use reedline::{Prompt, PromptHistorySearch, PromptHistorySearchStatus};
use std::borrow::Cow;
use std::env;
use std::path::PathBuf;

/// Custom prompt for rush shell
///
/// Displays:
/// - Current working directory (shortened to ~/ for home)
/// - Exit code indicator (green for success, red for failure)
/// - Prompt symbol (❯)
pub struct RushPrompt {
    /// Last command exit code (0 = success)
    exit_code: i32,
    /// Whether this is a continuation prompt for multiline input
    is_continuation: bool,
}

impl Clone for RushPrompt {
    fn clone(&self) -> Self {
        Self {
            exit_code: self.exit_code,
            is_continuation: self.is_continuation,
        }
    }
}

impl RushPrompt {
    /// Create a new prompt with the given exit code
    pub fn new(exit_code: i32) -> Self {
        Self {
            exit_code,
            is_continuation: false,
        }
    }

    /// Create a continuation prompt for multiline input
    pub fn new_continuation() -> Self {
        Self {
            exit_code: 0,
            is_continuation: true,
        }
    }

    /// Get shortened current directory path
    fn get_current_dir(&self) -> String {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Try to shorten home directory to ~
        if let Some(home) = dirs::home_dir() {
            if cwd.starts_with(&home) {
                if cwd == home {
                    return "~".to_string();
                }
                if let Ok(relative) = cwd.strip_prefix(&home) {
                    return format!("~/{}", relative.display());
                }
            }
        }

        cwd.display().to_string()
    }

    /// Get prompt color based on exit code
    fn get_prompt_indicator(&self) -> &'static str {
        if self.exit_code == 0 {
            "\x1b[32m❯\x1b[0m" // Green ❯
        } else {
            "\x1b[31m❯\x1b[0m" // Red ❯
        }
    }
}

impl Prompt for RushPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Owned(String::new())
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Owned(String::new())
    }

    fn render_prompt_indicator(&self, _prompt_mode: reedline::PromptEditMode) -> Cow<'_, str> {
        if self.is_continuation {
            // Continuation prompt - just show "> "
            Cow::Borrowed("> ")
        } else {
            // Format: "~/path/to/dir ❯ "
            let dir = self.get_current_dir();
            let indicator = self.get_prompt_indicator();
            Cow::Owned(format!("{} {} ", dir, indicator))
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        Cow::Owned(format!("({}reverse-search: {}) ", prefix, history_search.term))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_new() {
        let prompt = RushPrompt::new(0);
        assert_eq!(prompt.exit_code, 0);

        let prompt_fail = RushPrompt::new(1);
        assert_eq!(prompt_fail.exit_code, 1);
    }

    #[test]
    fn test_get_current_dir() {
        let prompt = RushPrompt::new(0);
        let dir = prompt.get_current_dir();
        // Should return a non-empty string
        assert!(!dir.is_empty());
    }

    #[test]
    fn test_prompt_indicator_colors() {
        let prompt_success = RushPrompt::new(0);
        let indicator_success = prompt_success.get_prompt_indicator();
        assert!(indicator_success.contains("32m")); // Green color code

        let prompt_failure = RushPrompt::new(1);
        let indicator_failure = prompt_failure.get_prompt_indicator();
        assert!(indicator_failure.contains("31m")); // Red color code
    }

    #[test]
    fn test_render_prompt_indicator() {
        let prompt = RushPrompt::new(0);
        let rendered = prompt.render_prompt_indicator(reedline::PromptEditMode::Default);
        // Should contain directory and prompt symbol
        assert!(rendered.contains("❯"));
    }

    #[test]
    fn test_home_directory_shortening() {
        let prompt = RushPrompt::new(0);
        let dir = prompt.get_current_dir();

        // If we're in home or a subdirectory, should use ~
        if let Some(home) = dirs::home_dir() {
            if env::current_dir().unwrap_or_default().starts_with(&home) {
                assert!(dir.starts_with("~"));
            }
        }
    }
}
