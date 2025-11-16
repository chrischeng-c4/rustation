//! REPL (Read-Eval-Print Loop) module
//!
//! Provides interactive command line interface with:
//! - Line editing
//! - Syntax highlighting
//! - Autosuggestions
//! - History navigation
//! - Tab completion

pub mod highlight;
pub mod input;
pub mod lexer;
pub mod prompt;
pub mod suggest;

use crate::completion::CompletionRegistry; // US1: Add tab completion
use crate::config::Config;
use crate::error::Result;
use crate::executor::execute::CommandExecutor;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Emacs, FileBackedHistory, KeyCode, KeyModifiers,
    MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};
use std::path::PathBuf;

use self::highlight::RushHighlighter;
use self::prompt::RushPrompt;

/// Interactive REPL for rush shell
pub struct Repl {
    /// Line editor from reedline
    editor: Reedline,

    /// Shell configuration
    #[allow(dead_code)] // Reserved for future REPL configuration
    config: Config,

    /// Command executor
    executor: CommandExecutor,

    /// Last command exit code (for prompt display)
    last_exit_code: i32,
}

impl Repl {
    /// Create new REPL with default configuration
    pub fn new() -> Result<Self> {
        let config = Config::load();
        Self::with_config(config)
    }

    /// Create REPL with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        // Set up history file path
        let history_path = Self::history_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = history_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Initialize reedline with file-backed history
        // TODO(v0.2.0): Add file locking to prevent corruption from concurrent writes
        // when multiple rush instances are running. Consider using fs2 crate for
        // cross-platform file locking.
        let history = Box::new(
            FileBackedHistory::with_file(config.history_size, history_path)
                .map_err(|e| crate::RushError::History(e.to_string()))?,
        );

        // Create highlighter
        let highlighter = Box::new(RushHighlighter::new());

        // Create tab completer (US1: Command completion)
        let completer = Box::new(CompletionRegistry::new());

        // Create completion menu for visual display (T015)
        let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));

        // Set up keybindings for Tab key to trigger completion (T015)
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu("completion_menu".to_string()),
                ReedlineEvent::MenuNext,
            ]),
        );

        // Create edit mode with keybindings (T015)
        let edit_mode = Box::new(Emacs::new(keybindings));

        // Build editor with history, syntax highlighting, and tab completion (T015)
        let editor = Reedline::create()
            .with_history(history)
            .with_highlighter(highlighter)
            .with_completer(completer)
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_edit_mode(edit_mode); // US1: Enable tab completion with menu

        Ok(Self { editor, config, executor: CommandExecutor::new(), last_exit_code: 0 })
    }

    /// Run the REPL loop until exit
    /// Returns exit code for the shell process
    pub fn run(&mut self) -> Result<i32> {
        tracing::info!("Starting REPL loop");
        let mut iteration = 0;

        loop {
            iteration += 1;
            tracing::trace!(iteration, "REPL iteration start");

            // Create prompt with current exit code
            let prompt = RushPrompt::new(self.last_exit_code);

            // Read line from user
            tracing::trace!("Waiting for user input");
            let sig = self.editor.read_line(&prompt);

            match sig {
                Ok(Signal::Success(buffer)) => {
                    // User entered a command
                    let line = buffer.trim();
                    tracing::debug!(cmd = %line, iteration, "User input received");

                    // Skip empty lines
                    if line.is_empty() {
                        tracing::trace!("Empty line, skipping");
                        continue;
                    }

                    // Check for exit command
                    if line == "exit" || line == "quit" {
                        tracing::info!(exit_code = self.last_exit_code, "Exit command received");
                        return Ok(self.last_exit_code);
                    }

                    // Execute the command
                    match self.executor.execute(line) {
                        Ok(exit_code) => {
                            tracing::info!(
                                cmd = %line,
                                exit_code,
                                prev_exit_code = self.last_exit_code,
                                "Command completed"
                            );
                            self.last_exit_code = exit_code;
                        }
                        Err(err) => {
                            tracing::error!(
                                cmd = %line,
                                error = %err,
                                "Command execution error"
                            );
                            eprintln!("rush: error: {}", err);
                            self.last_exit_code = 1;
                        }
                    }
                }
                Ok(Signal::CtrlD) => {
                    // Exit on Ctrl+D
                    tracing::info!(exit_code = self.last_exit_code, "Ctrl+D received, exiting");
                    return Ok(self.last_exit_code);
                }
                Ok(Signal::CtrlC) => {
                    // Clear current line and show new prompt
                    tracing::debug!(iteration, "Ctrl+C received, clearing line");
                    // reedline handles this automatically
                    continue;
                }
                Err(err) => {
                    // REPL error - log and return error code
                    tracing::error!(error = %err, iteration, "REPL error");
                    eprintln!("rush: REPL error: {}", err);
                    return Ok(1);
                }
            }
        }
    }

    /// Get history file path
    fn history_path() -> Result<PathBuf> {
        // Try data_dir first, then home_dir, then error
        let base = dirs::data_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("share")))
            .ok_or_else(|| {
                crate::RushError::Config(
                    "Unable to determine home directory for history file".to_string(),
                )
            })?;

        Ok(base.join("rush").join("history.txt"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_new() {
        // Basic test that REPL can be created
        let result = Repl::new();
        assert!(result.is_ok(), "REPL should initialize successfully");
    }

    #[test]
    fn test_repl_with_config() {
        let config = Config::default();
        let result = Repl::with_config(config);
        assert!(result.is_ok(), "REPL should initialize with custom config");
    }

    #[test]
    fn test_history_path() {
        let path = Repl::history_path().expect("Should get history path");
        assert!(path.to_string_lossy().contains("rush"));
        assert!(path.to_string_lossy().ends_with("history.txt"));
    }
}
