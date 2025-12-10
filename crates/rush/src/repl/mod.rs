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
pub mod validator;

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
use self::suggest::RushHinter;

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

        // Create hinter for autosuggestions (US1: Autosuggestions)
        let hinter = Box::new(RushHinter::new());

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

        // Set up keybinding for Right Arrow to accept full autosuggestion (US2)
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Right,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::HistoryHintComplete,
                ReedlineEvent::Edit(vec![reedline::EditCommand::MoveRight { select: false }]),
            ]),
        );

        // Set up keybinding for Alt+Right Arrow to accept word from autosuggestion (US3)
        keybindings.add_binding(
            KeyModifiers::ALT,
            KeyCode::Right,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::HistoryHintWordComplete,
                ReedlineEvent::Edit(vec![reedline::EditCommand::MoveWordRight { select: false }]),
            ]),
        );

        // Create edit mode with keybindings (T015)
        let edit_mode = Box::new(Emacs::new(keybindings));

        // Build editor with history, syntax highlighting, autosuggestions, and tab completion
        let editor = Reedline::create()
            .with_history(history)
            .with_highlighter(highlighter)
            .with_hinter(hinter) // US1: Enable autosuggestions
            .with_completer(completer)
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_edit_mode(edit_mode);

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

            // Check for background jobs status
            self.executor.check_background_jobs();

            // Create prompt with current exit code
            let prompt = RushPrompt::new(self.last_exit_code);

            // Read input - potentially across multiple lines for incomplete statements
            let sig = self.read_complete_statement_signal(&prompt)?;

            match sig {
                Signal::Success(line) => {
                    tracing::debug!(cmd = %line, iteration, "User input received");

                    // Skip empty lines
                    if line.trim().is_empty() {
                        tracing::trace!("Empty line, skipping");
                        continue;
                    }

                    // Execute the command
                    match self.executor.execute(&line) {
                        Ok(exit_code) => {
                            tracing::info!(
                                cmd = %line,
                                exit_code,
                                prev_exit_code = self.last_exit_code,
                                "Command completed"
                            );
                            self.last_exit_code = exit_code;
                        }
                        Err(crate::RushError::ExitRequest(code)) => {
                            // Exit builtin requested shell termination
                            tracing::info!(exit_code = code, "Exit command received");
                            return Ok(code);
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
                Signal::CtrlD => {
                    // Exit on Ctrl+D
                    tracing::info!(exit_code = self.last_exit_code, "Ctrl+D received, exiting");
                    return Ok(self.last_exit_code);
                }
                Signal::CtrlC => {
                    // Clear current line and show new prompt
                    tracing::debug!(iteration, "Ctrl+C received, clearing line");
                    // reedline handles this automatically
                    continue;
                }
            }
        }
    }

    /// Read a complete statement, potentially across multiple lines
    /// Accumulates lines until we have a syntactically complete statement
    /// Also handles heredoc input collection
    fn read_complete_statement_signal(
        &mut self,
        prompt: &RushPrompt,
    ) -> std::result::Result<Signal, std::io::Error> {
        use crate::executor::parser::{get_pending_heredocs, is_heredoc_complete};

        let mut accumulated = String::new();
        let mut line_count = 0;
        let mut in_heredoc = false;

        loop {
            line_count += 1;

            // Use different prompt based on context
            let current_prompt = if line_count == 1 {
                prompt.clone()
            } else if in_heredoc {
                RushPrompt::new_heredoc()
            } else {
                RushPrompt::new_continuation()
            };

            // Read line from user
            tracing::trace!("Waiting for user input (line {})", line_count);
            let sig = self.editor.read_line(&current_prompt);

            match sig {
                Ok(Signal::Success(buffer)) => {
                    // Add the line to accumulated input
                    if !accumulated.is_empty() {
                        accumulated.push('\n');
                    }
                    accumulated.push_str(&buffer);

                    // On first line, check if we're starting a heredoc
                    if line_count == 1 {
                        if let Ok(pending) = get_pending_heredocs(&buffer) {
                            if !pending.is_empty() {
                                in_heredoc = true;
                                tracing::debug!(
                                    heredocs = pending.len(),
                                    "Detected heredocs, starting collection"
                                );
                            }
                        }
                    }

                    // Check if both statement and heredocs are complete
                    let statement_complete =
                        crate::executor::conditional::is_statement_complete(&accumulated);
                    let heredoc_complete = is_heredoc_complete(&accumulated);

                    if statement_complete && heredoc_complete {
                        return Ok(Signal::Success(accumulated));
                    }

                    // Update heredoc tracking
                    if in_heredoc && heredoc_complete {
                        in_heredoc = false;
                    }
                }
                Ok(signal) => {
                    // Ctrl+D, Ctrl+C, or other signals
                    return Ok(signal);
                }
                Err(err) => {
                    return Err(err);
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
