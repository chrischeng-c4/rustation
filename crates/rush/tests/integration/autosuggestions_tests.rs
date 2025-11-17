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
        let result1 = hinter.hint("g", 1, &history);
        assert_eq!(
            result1,
            Some("it status".to_string()),
            "After 'g': should suggest 'it status'"
        );

        let result2 = hinter.hint("gi", 2, &history);
        assert_eq!(
            result2,
            Some("t status".to_string()),
            "After 'gi': should suggest 't status'"
        );

        let result3 = hinter.hint("git", 3, &history);
        assert_eq!(
            result3,
            Some(" status".to_string()),
            "After 'git': should suggest ' status'"
        );

        let result4 = hinter.hint("git ", 4, &history);
        assert_eq!(
            result4,
            Some("status".to_string()),
            "After 'git ': should suggest 'status'"
        );

        let result5 = hinter.hint("git s", 5, &history);
        assert_eq!(
            result5,
            Some("tatus".to_string()),
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
        let result = hinter.hint("cargo", 5, &history);
        assert_eq!(
            result,
            Some(" test".to_string()),
            "Should suggest most recent cargo command"
        );

        // Should suggest most recent "git" command
        let result = hinter.hint("git", 3, &history);
        assert_eq!(
            result,
            Some(" push".to_string()),
            "Should suggest most recent git command"
        );
    }

    #[test]
    fn test_no_suggestion_after_exact_match() {
        let mut hinter = RushHinter::new();
        let mut history = TestHistory::new();
        history.add("ls");

        // Typing "ls" should not suggest anything (exact match)
        let result = hinter.hint("ls", 2, &history);
        assert_eq!(result, None, "Should not suggest for exact match");
    }

    #[test]
    fn test_backspace_updates_suggestion() {
        let mut hinter = RushHinter::new();
        let mut history = TestHistory::new();
        history.add("git status");
        history.add("git stash");

        // Type "git st"
        let result = hinter.hint("git st", 6, &history);
        assert_eq!(
            result,
            Some("ash".to_string()),
            "Should suggest 'ash' for 'git st'"
        );

        // Simulate backspace to "git s"
        let result = hinter.hint("git s", 5, &history);
        assert_eq!(
            result,
            Some("tash".to_string()),
            "After backspace: should suggest 'tash' for 'git s'"
        );
    }

    #[test]
    fn test_suggestion_with_long_commands() {
        let mut hinter = RushHinter::new();
        let mut history = TestHistory::new();
        history.add("cargo build --release --features \"feature1 feature2 feature3\" --target x86_64-apple-darwin");

        let result = hinter.hint("cargo b", 7, &history);
        assert!(result.is_some(), "Should suggest for long command");

        let suggestion = result.unwrap();
        assert!(
            suggestion.starts_with("uild --release"),
            "Should start with correct suffix"
        );
        assert!(
            suggestion.contains("features"),
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
        let result = hinter.hint("git", 3, &history);
        let duration = start.elapsed();

        assert_eq!(
            result,
            Some(" status".to_string()),
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
