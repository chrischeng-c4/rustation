// Unit tests for RushHinter (autosuggestions)

use rush::repl::suggest::RushHinter;
use reedline::{Hinter, History, HistoryItem, HistoryItemId, SearchQuery};

/// Mock history implementation for testing
struct MockHistory {
    entries: Vec<String>,
}

impl MockHistory {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add(&mut self, cmd: impl Into<String>) {
        self.entries.push(cmd.into());
    }
}

impl History for MockHistory {
    fn save(&mut self, _entry: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn load(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(self.entries.clone())
    }

    fn clear(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn iter_chronologic(&self) -> impl Iterator<Item = HistoryItem> {
        self.entries.iter().enumerate().map(|(id, cmd)| HistoryItem {
            id: Some(HistoryItemId::new(id as i64)),
            start_timestamp: None,
            command_line: cmd.clone(),
            session_id: None,
            hostname: None,
            cwd: None,
            duration: None,
            exit_status: None,
            more_info: None,
        })
    }

    fn count(&self, _query: SearchQuery) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(self.entries.len() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let _hinter = RushHinter::new();
        // Should create without panic
    }

    #[test]
    fn test_default() {
        let _hinter = RushHinter::default();
        // Should create without panic
    }

    #[test]
    fn test_prefix_matching() {
        let mut hinter = RushHinter::new();
        let mut history = MockHistory::new();
        history.add("git status");
        history.add("git commit");
        history.add("cargo build");

        // Should match "git" prefix
        let result = hinter.handle("git", 3, &history, false, "");
        assert!(!result.is_empty(), "Should find match for 'git'");

        // Should not match non-existent prefix
        let result = hinter.handle("xyz", 3, &history, false, "");
        assert_eq!(result, "", "Should not match 'xyz'");
    }

    #[test]
    fn test_most_recent_match() {
        let mut hinter = RushHinter::new();
        let mut history = MockHistory::new();
        history.add("git status"); // Older
        history.add("git stash"); // Newer

        let result = hinter.handle("git s", 5, &history, false, "");
        assert_eq!(
            result,
            "tash",
            "Should suggest 'tash' from most recent 'git stash'"
        );
    }

    #[test]
    fn test_cursor_position_check() {
        let mut hinter = RushHinter::new();
        let mut history = MockHistory::new();
        history.add("git status");

        // Cursor at end: should suggest
        let result = hinter.handle("git s", 5, &history, false, "");
        assert!(!result.is_empty(), "Should suggest when cursor at end");

        // Cursor in middle: should not suggest
        let result = hinter.handle("git s", 3, &history, false, "");
        assert_eq!(result, "", "Should not suggest when cursor in middle");

        // Cursor at start: should not suggest
        let result = hinter.handle("git s", 0, &history, false, "");
        assert_eq!(result, "", "Should not suggest when cursor at start");
    }

    #[test]
    fn test_empty_input_handling() {
        let mut hinter = RushHinter::new();
        let mut history = MockHistory::new();
        history.add("git status");

        // Empty input should not suggest
        let result = hinter.handle("", 0, &history, false, "");
        assert_eq!(result, "", "Should not suggest for empty input");
    }

    #[test]
    fn test_no_matching_history() {
        let mut hinter = RushHinter::new();
        let mut history = MockHistory::new();
        history.add("git status");

        // No match for "cargo"
        let result = hinter.handle("cargo", 5, &history, false, "");
        assert_eq!(result, "", "Should not suggest when no match");
    }

    #[test]
    fn test_empty_history() {
        let mut hinter = RushHinter::new();
        let history = MockHistory::new(); // Empty

        let result = hinter.handle("git s", 5, &history, false, "");
        assert_eq!(result, "", "Should handle empty history gracefully");
    }

    #[test]
    fn test_exact_match_not_suggested() {
        let mut hinter = RushHinter::new();
        let mut history = MockHistory::new();
        history.add("git status");

        // Exact match should not be suggested
        let result = hinter.handle("git status", 10, &history, false, "");
        assert_eq!(result, "", "Should not suggest exact matches");
    }

    #[test]
    fn test_suffix_only_returned() {
        let mut hinter = RushHinter::new();
        let mut history = MockHistory::new();
        history.add("cargo build --release");

        let result = hinter.handle("cargo b", 7, &history, false, "");
        assert_eq!(
            result,
            "uild --release",
            "Should return suffix only"
        );
    }

    #[test]
    fn test_special_characters() {
        let mut hinter = RushHinter::new();
        let mut history = MockHistory::new();
        history.add("echo \"hello world\"");
        history.add("git commit -m 'fix: bug'");

        // Should handle quotes
        let result = hinter.handle("echo \"", 6, &history, false, "");
        assert_eq!(
            result,
            "hello world\"",
            "Should handle double quotes"
        );

        let result = hinter.handle("git commit -m '", 15, &history, false, "");
        assert_eq!(
            result,
            "fix: bug'",
            "Should handle single quotes"
        );
    }
}
