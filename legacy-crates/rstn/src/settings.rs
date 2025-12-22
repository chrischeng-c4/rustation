//! Settings persistence for rstn
//!
//! Stores settings in ~/.rstn/settings.json

use crate::session::get_data_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Run SDD phases automatically in sequence
    #[serde(default = "default_auto_run")]
    pub auto_run: bool,

    /// Maximum turns for Claude CLI
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,

    /// Skip permission prompts in Claude CLI
    #[serde(default = "default_skip_permissions")]
    pub skip_permissions: bool,

    /// Enable logging to file
    #[serde(default = "default_logging_enabled")]
    pub logging_enabled: bool,

    /// Log level: "error", "warn", "info", "debug", "trace"
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Also output logs to console (stderr)
    #[serde(default = "default_log_to_console")]
    pub log_to_console: bool,

    /// Path to Claude CLI executable (auto-discovered or user-specified)
    /// If None, will be auto-discovered on first use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_path: Option<String>,
}

fn default_auto_run() -> bool {
    true
}

fn default_max_turns() -> u32 {
    50
}

fn default_skip_permissions() -> bool {
    true
}

fn default_logging_enabled() -> bool {
    true // Enabled by default for development
}

fn default_log_level() -> String {
    // Use trace logging for debug builds, info for release builds
    #[cfg(debug_assertions)]
    {
        "trace".to_string()
    }
    #[cfg(not(debug_assertions))]
    {
        "info".to_string()
    }
}

fn default_log_to_console() -> bool {
    false
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_run: default_auto_run(),
            max_turns: default_max_turns(),
            skip_permissions: default_skip_permissions(),
            logging_enabled: default_logging_enabled(),
            log_level: default_log_level(),
            log_to_console: default_log_to_console(),
            claude_path: None,
        }
    }
}

impl Settings {
    /// Get the path to the settings file
    pub fn path() -> PathBuf {
        get_data_dir().join("settings.json")
    }

    /// Load settings from disk, or return defaults if not found
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(settings) => return settings,
                    Err(e) => {
                        eprintln!("Warning: Failed to parse settings: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Failed to read settings: {}", e);
                }
            }
        }
        Self::default()
    }

    /// Save settings to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| std::io::Error::other(e))?;
        std::fs::write(path, content)
    }

    /// Toggle auto-run mode
    pub fn toggle_auto_run(&mut self) {
        self.auto_run = !self.auto_run;
    }

    /// Toggle skip permissions
    pub fn toggle_skip_permissions(&mut self) {
        self.skip_permissions = !self.skip_permissions;
    }

    /// Increment max turns (by 10)
    pub fn increment_max_turns(&mut self) {
        self.max_turns = self.max_turns.saturating_add(10).min(200);
    }

    /// Decrement max turns (by 10)
    pub fn decrement_max_turns(&mut self) {
        self.max_turns = self.max_turns.saturating_sub(10).max(10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.auto_run);
        assert_eq!(settings.max_turns, 50);
        assert!(settings.skip_permissions);
    }

    #[test]
    fn test_toggle_auto_run() {
        let mut settings = Settings::default();
        assert!(settings.auto_run);
        settings.toggle_auto_run();
        assert!(!settings.auto_run);
        settings.toggle_auto_run();
        assert!(settings.auto_run);
    }
}
