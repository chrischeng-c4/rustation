//! Default configuration values for rush
//!
//! These defaults ensure rush works perfectly with zero configuration.

use crossterm::style::Color;
use std::path::PathBuf;

/// Default configuration for rush shell
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum number of history entries (default: 10,000)
    pub history_size: usize,

    /// Command prompt string (default: "$ ")
    pub prompt: String,

    /// Color theme for syntax highlighting
    pub theme: Theme,

    /// Tab completion timeout in milliseconds (default: 100ms)
    pub completion_timeout_ms: u64,

    /// Autosuggestion delay in milliseconds (default: 50ms)
    pub suggestion_delay_ms: u64,
}

impl Config {
    /// Load configuration from TOML file, falling back to defaults if file doesn't exist
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if !config_path.exists() {
            tracing::debug!("Config file not found, using defaults");
            return Self::default();
        }

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
                Ok(config_file) => {
                    tracing::info!("Loaded configuration from {:?}", config_path);
                    Self::from_config_file(config_file)
                }
                Err(e) => {
                    tracing::warn!("Failed to parse config file: {}, using defaults", e);
                    Self::default()
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read config file: {}, using defaults", e);
                Self::default()
            }
        }
    }

    /// Get the path to the configuration file: ~/.config/rush/rush.toml
    pub fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("rush");
        path.push("rush.toml");
        path
    }

    /// Convert from TOML config file format to Config struct
    fn from_config_file(file: ConfigFile) -> Self {
        let mut config = Self::default();

        if let Some(behavior) = file.behavior {
            if let Some(history_size) = behavior.history_size {
                config.history_size = history_size;
            }
            if let Some(completion_timeout_ms) = behavior.completion_timeout_ms {
                config.completion_timeout_ms = completion_timeout_ms;
            }
            if let Some(suggestion_delay_ms) = behavior.suggestion_delay_ms {
                config.suggestion_delay_ms = suggestion_delay_ms;
            }
        }

        if let Some(appearance) = file.appearance {
            if let Some(prompt) = appearance.prompt {
                config.prompt = prompt;
            }
        }

        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            history_size: 10_000,
            prompt: "$ ".to_string(),
            theme: Theme::default(),
            completion_timeout_ms: 100,
            suggestion_delay_ms: 50,
        }
    }
}

/// TOML configuration file format
#[derive(Debug, serde::Deserialize)]
struct ConfigFile {
    appearance: Option<AppearanceConfig>,
    behavior: Option<BehaviorConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct AppearanceConfig {
    prompt: Option<String>,
    #[allow(dead_code)] // Reserved for future theme support
    theme: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct BehaviorConfig {
    history_size: Option<usize>,
    completion_timeout_ms: Option<u64>,
    suggestion_delay_ms: Option<u64>,
}

/// Color theme for syntax highlighting
#[derive(Debug, Clone)]
pub struct Theme {
    pub command_color: Color,
    pub flag_color: Color,
    pub path_color: Color,
    pub string_color: Color,
    pub error_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            command_color: Color::Green,
            flag_color: Color::Cyan, // Cyan for better visibility on dark terminals
            path_color: Color::Cyan,
            string_color: Color::Yellow,
            error_color: Color::Red,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.history_size, 10_000);
        assert_eq!(config.prompt, "$ ");
        assert_eq!(config.completion_timeout_ms, 100);
        assert_eq!(config.suggestion_delay_ms, 50);
    }

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        // Verify all colors are set (just checking they're Color types)
        match theme.command_color {
            Color::Green => (),
            _ => panic!("Expected command color to be Green"),
        }
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::default();
        let config2 = config1.clone();
        assert_eq!(config1.history_size, config2.history_size);
        assert_eq!(config1.prompt, config2.prompt);
    }

    #[test]
    fn test_config_load_missing_file() {
        // When config file doesn't exist, should return defaults
        let config = Config::load();
        assert_eq!(config.history_size, 10_000);
        assert_eq!(config.prompt, "$ ");
    }
}
