use ratatui::style::Color;
use std::time::SystemTime;

/// Category of log entry for styling and filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCategory {
    /// Slash command execution (e.g., /speckit.specify)
    SlashCommand,
    /// Claude Code streaming output
    ClaudeStream,
    /// File change in spec directory
    FileChange,
    /// Shell script execution
    ShellOutput,
    /// System/TUI internal messages
    System,
}

impl LogCategory {
    /// Get emoji icon for this category
    pub fn icon(&self) -> &'static str {
        match self {
            Self::SlashCommand => "⚡",
            Self::ClaudeStream => "🤖",
            Self::FileChange => "📝",
            Self::ShellOutput => "🔧",
            Self::System => "ℹ️",
        }
    }

    /// Get ratatui Color for this category
    pub fn color(&self) -> Color {
        match self {
            Self::SlashCommand => Color::Cyan,
            Self::ClaudeStream => Color::White,
            Self::FileChange => Color::Green,
            Self::ShellOutput => Color::Yellow,
            Self::System => Color::DarkGray,
        }
    }
}

/// A single timestamped log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub category: LogCategory,
    pub content: String,
}

impl LogEntry {
    /// Create a new log entry with current timestamp
    pub fn new(category: LogCategory, content: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            category,
            content,
        }
    }

    /// Format timestamp as HH:MM:SS
    pub fn format_timestamp(&self) -> String {
        use chrono::prelude::*;

        if let Ok(duration) = self.timestamp.duration_since(SystemTime::UNIX_EPOCH) {
            let datetime = Local.timestamp_opt(duration.as_secs() as i64, 0);
            if let Some(dt) = datetime.single() {
                return dt.format("%H:%M:%S").to_string();
            }
        }

        // Fallback if timestamp conversion fails
        "00:00:00".to_string()
    }

    /// Get emoji icon for this entry's category
    pub fn category_icon(&self) -> &'static str {
        self.category.icon()
    }
}
