// Unit tests for RushHinter (autosuggestions)

use rush::repl::suggest::RushHinter;
use reedline::{Hinter, History, HistoryItem, HistoryItemId, HistorySessionId, SearchQuery, Result as ReedlineResult, CommandLineSearch};

/// Mock history implementation for testing
struct MockHistory {
    entries: Vec<HistoryItem>,
    next_id: i64,
}

impl MockHistory {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    fn add(&mut self, cmd: impl Into<String>) {
        let item = HistoryItem {
            id: Some(HistoryItemId::new(self.next_id)),
            start_timestamp: None,
            command_line: cmd.into(),
            session_id: None,
            hostname: None,
            cwd: None,
            duration: None,
            exit_status: None,
            more_info: None,
        };
        self.next_id += 1;
        self.entries.push(item);
    }
}

impl History for MockHistory {
    fn save(&mut self, mut h: HistoryItem) -> ReedlineResult<HistoryItem> {
        if h.id.is_none() {
            h.id = Some(HistoryItemId::new(self.next_id));
            self.next_id += 1;
        }
        self.entries.push(h.clone());
        Ok(h)
    }

    fn load(&self, id: HistoryItemId) -> ReedlineResult<HistoryItem> {
        self.entries
            .iter()
            .find(|item| item.id == Some(id))
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Item not found").into())
    }

    fn count(&self, query: SearchQuery) -> ReedlineResult<i64> {
        Ok(self.search(query)?.len() as i64)
    }

    fn search(&self, query: SearchQuery) -> ReedlineResult<Vec<HistoryItem>> {
        let prefix = match query.filter.command_line {
            Some(CommandLineSearch::Prefix(s)) => s,
            Some(CommandLineSearch::Substring(s)) => s,
            Some(CommandLineSearch::Exact(s)) => s,
            None => String::new(),
        };

        let results: Vec<HistoryItem> = self.entries
            .iter()
            .filter(|item| {
                if prefix.is_empty() {
                    true
                } else {
                    item.command_line.starts_with(&prefix)
                }
            })
            .rev()  // Most recent first
            .cloned()
            .collect();
        Ok(results)
    }

    fn update(&mut self, id: HistoryItemId, updater: &dyn Fn(HistoryItem) -> HistoryItem) -> ReedlineResult<()> {
        if let Some(item) = self.entries.iter_mut().find(|item| item.id == Some(id)) {
            *item = updater(item.clone());
        }
        Ok(())
    }

    fn clear(&mut self) -> ReedlineResult<()> {
        self.entries.clear();
        Ok(())
    }

    fn delete(&mut self, id: HistoryItemId) -> ReedlineResult<()> {
        self.entries.retain(|item| item.id != Some(id));
        Ok(())
    }

    fn sync(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn session(&self) -> Option<HistorySessionId> {
        None
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
