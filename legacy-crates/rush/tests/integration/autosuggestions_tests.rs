// Integration tests for autosuggestions feature

use rush::repl::suggest::RushHinter;
use reedline::{Hinter, History, HistoryItem, HistoryItemId, SearchQuery};

/// Mock history for integration testing
struct TestHistory {
    entries: Vec<String>,
}

impl TestHistory {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add(&mut self, cmd: impl Into<String>) {
        self.entries.push(cmd.into());
    }
}

impl History for TestHistory {
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
    fn test_realtime_suggestion_updates() {
        let mut hinter = RushHinter::new();
        let mut history = TestHistory::new();
        history.add("git status");

        // Simulate user typing "git s" character by character
        let result1 = hinter.handle("g", 1, &history, false, "");
        assert_eq!(
            result1,
            "it status",
            "After 'g': should suggest 'it status'"
        );

        let result2 = hinter.handle("gi", 2, &history, false, "");
        assert_eq!(
            result2,
            "t status",
            "After 'gi': should suggest 't status'"
        );

        let result3 = hinter.handle("git", 3, &history, false, "");
        assert_eq!(
            result3,
            " status",
            "After 'git': should suggest ' status'"
        );

        let result4 = hinter.handle("git ", 4, &history, false, "");
        assert_eq!(
            result4,
            "status",
            "After 'git ': should suggest 'status'"
        );

        let result5 = hinter.handle("git s", 5, &history, false, "");
        assert_eq!(
            result5,
            "tatus",
            "After 'git s': should suggest 'tatus'"
        );
    }

    #[test]
    fn test_suggestion_with_multiple_commands() {
        let mut hinter = RushHinter::new();
        let mut history = TestHistory::new();

        // Add various commands
        history.add("git status");
        history.add("git commit -m \"test\"");
        history.add("git push");
        history.add("cargo build");
        history.add("cargo test");

        // Should suggest most recent "cargo" command
        let result = hinter.handle("cargo", 5, &history, false, "");
        assert_eq!(
            result,
            " test",
            "Should suggest most recent cargo command"
        );

        // Should suggest most recent "git" command
        let result = hinter.handle("git", 3, &history, false, "");
        assert_eq!(
            result,
            " push",
            "Should suggest most recent git command"
        );
    }

    #[test]
    fn test_no_suggestion_after_exact_match() {
        let mut hinter = RushHinter::new();
        let mut history = TestHistory::new();
        history.add("ls");

        // Typing "ls" should not suggest anything (exact match)
        let result = hinter.handle("ls", 2, &history, false, "");
        assert_eq!(result, "", "Should not suggest for exact match");
    }

    #[test]
    fn test_backspace_updates_suggestion() {
        let mut hinter = RushHinter::new();
        let mut history = TestHistory::new();
        history.add("git status");
        history.add("git stash");

        // Type "git st"
        let result = hinter.handle("git st", 6, &history, false, "");
        assert_eq!(
            result,
            "ash",
            "Should suggest 'ash' for 'git st'"
        );

        // Simulate backspace to "git s"
        let result = hinter.handle("git s", 5, &history, false, "");
        assert_eq!(
            result,
            "tash",
            "After backspace: should suggest 'tash' for 'git s'"
        );
    }

    #[test]
    fn test_suggestion_with_long_commands() {
        let mut hinter = RushHinter::new();
        let mut history = TestHistory::new();
        history.add("cargo build --release --features \"feature1 feature2 feature3\" --target x86_64-apple-darwin");

        let result = hinter.handle("cargo b", 7, &history, false, "");
        assert!(!result.is_empty(), "Should suggest for long command");

        assert!(
            result.starts_with("uild --release"),
            "Should start with correct suffix"
        );
        assert!(
            result.contains("features"),
            "Should include features in suggestion"
        );
    }

    #[test]
    fn test_performance_with_many_history_entries() {
        let mut hinter = RushHinter::new();
        let mut history = TestHistory::new();

        // Add 1000 history entries
        for i in 0..1000 {
            history.add(format!("command_{}", i));
        }

        // Add target command at the end (most recent)
        history.add("git status");

        // Should still find suggestion quickly
        let start = std::time::Instant::now();
        let result = hinter.handle("git", 3, &history, false, "");
        let duration = start.elapsed();

        assert_eq!(
            result,
            " status",
            "Should find suggestion with many entries"
        );

        // Should complete in reasonable time (much less than 50ms)
        assert!(
            duration.as_millis() < 50,
            "Suggestion should complete within 50ms, took {:?}",
            duration
        );
    }
}
