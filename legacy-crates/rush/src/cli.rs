//! CLI argument parsing for rush shell

use clap::Parser;
use std::path::PathBuf;

/// Full version string including git hash, date, and build profile
/// Example: "0.35.0 (8ca3000, 2024-12-14) [debug]"
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_HASH"),
    ", ",
    env!("BUILD_DATE"),
    ") [",
    env!("BUILD_PROFILE"),
    "]"
);

/// rush - A modern, fast, fish-like shell written in Rust
#[derive(Parser, Debug)]
#[command(name = "rush")]
#[command(version = VERSION)]
#[command(about = "A modern, fast, fish-like shell", long_about = None)]
#[command(author)]
pub struct Cli {
    /// Increase logging verbosity (-v for debug, -vv for trace)
    ///
    /// Enables detailed logging to help diagnose issues during alpha testing.
    /// Logs are written to ~/.local/share/rush/rush-v{version}.log
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-essential output
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Use custom config file
    ///
    /// Specify a custom TOML configuration file instead of the default
    /// ~/.config/rush/rush.toml
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Ignore config file and use built-in defaults
    #[arg(long, conflicts_with = "config")]
    pub no_config: bool,

    /// Custom log file path (requires --verbose)
    ///
    /// Override the default log file location. Only applies when verbose
    /// logging is enabled.
    #[arg(long, value_name = "FILE", requires = "verbose")]
    pub log_file: Option<PathBuf>,

    /// Log format: pretty (default) or json
    ///
    /// - pretty: Human-readable colored output
    /// - json: Structured JSON for parsing/analysis
    #[arg(long, value_name = "FORMAT", default_value = "pretty")]
    pub log_format: String,

    /// Override history size limit
    #[arg(long, value_name = "N")]
    pub history_size: Option<usize>,

    /// Disable command history persistence
    ///
    /// Useful for privacy or temporary sessions
    #[arg(long)]
    pub no_history: bool,

    /// Execute command and exit (like bash -c)
    ///
    /// Run a single command and exit instead of starting interactive shell
    #[arg(short = 'c', long, value_name = "COMMAND")]
    pub command: Option<String>,

    /// Print resolved configuration and exit
    ///
    /// Shows the complete configuration that rush will use,
    /// including defaults and loaded values
    #[arg(long)]
    pub dump_config: bool,

    /// Disable syntax highlighting
    #[arg(long)]
    pub no_highlight: bool,

    /// Disable autosuggestions
    #[arg(long)]
    pub no_suggestions: bool,

    /// Disable tab completion
    #[arg(long)]
    pub no_completion: bool,

    /// Run health check and print diagnostics
    ///
    /// Verifies config files, history file, permissions, and dependencies
    #[arg(long)]
    pub doctor: bool,
}

impl Cli {
    /// Get the log level based on verbosity flag
    pub fn log_level(&self) -> &str {
        match self.verbose {
            0 => "info",
            1 => "debug",
            _ => "trace", // 2 or more
        }
    }

    /// Check if verbose logging is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose > 0
    }

    /// Get the log file path, using default if not specified
    pub fn get_log_file_path(&self) -> PathBuf {
        if let Some(ref path) = self.log_file {
            path.clone()
        } else {
            // Default: ~/.local/share/rush/rush-v{version}.log
            let version = env!("CARGO_PKG_VERSION");
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("rush")
                .join(format!("rush-v{}.log", version))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_default() {
        let cli = Cli {
            verbose: 0,
            quiet: false,
            config: None,
            no_config: false,
            log_file: None,
            log_format: "pretty".to_string(),
            history_size: None,
            no_history: false,
            command: None,
            dump_config: false,
            no_highlight: false,
            no_suggestions: false,
            no_completion: false,
            doctor: false,
        };

        assert_eq!(cli.log_level(), "info");
        assert!(!cli.is_verbose());
    }

    #[test]
    fn test_log_level_verbose() {
        let mut cli = Cli {
            verbose: 1,
            quiet: false,
            config: None,
            no_config: false,
            log_file: None,
            log_format: "pretty".to_string(),
            history_size: None,
            no_history: false,
            command: None,
            dump_config: false,
            no_highlight: false,
            no_suggestions: false,
            no_completion: false,
            doctor: false,
        };

        assert_eq!(cli.log_level(), "debug");
        assert!(cli.is_verbose());

        cli.verbose = 2;
        assert_eq!(cli.log_level(), "trace");
    }

    #[test]
    fn test_log_file_path_default() {
        let cli = Cli {
            verbose: 1,
            quiet: false,
            config: None,
            no_config: false,
            log_file: None,
            log_format: "pretty".to_string(),
            history_size: None,
            no_history: false,
            command: None,
            dump_config: false,
            no_highlight: false,
            no_suggestions: false,
            no_completion: false,
            doctor: false,
        };

        let path = cli.get_log_file_path();
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("rush"));
        assert!(path_str.contains(&format!("rush-v{}.log", env!("CARGO_PKG_VERSION"))));
    }

    #[test]
    fn test_log_file_path_custom() {
        let custom_path = PathBuf::from("/tmp/custom.log");
        let cli = Cli {
            verbose: 1,
            quiet: false,
            config: None,
            no_config: false,
            log_file: Some(custom_path.clone()),
            log_format: "pretty".to_string(),
            history_size: None,
            no_history: false,
            command: None,
            dump_config: false,
            no_highlight: false,
            no_suggestions: false,
            no_completion: false,
            doctor: false,
        };

        assert_eq!(cli.get_log_file_path(), custom_path);
    }
}
