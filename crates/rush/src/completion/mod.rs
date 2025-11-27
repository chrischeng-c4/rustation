//! Tab completion module for rush shell
//!
//! Provides completions for:
//! - Command names from PATH (P1 - CommandCompleter)
//! - File and directory paths (P2 - PathCompleter)
//! - Flags for common commands (P3 - FlagCompleter)
//!
//! Uses reedline's Completer trait for integration with the REPL.

// Re-export reedline types for convenience (T005)
pub use reedline::{Completer, Span, Suggestion};

// Module declarations
pub mod command; // US1 - Command completion from PATH
pub mod flag;
pub mod path; // US2 - File and directory path completion // US3 - Flag completion for common commands

// Re-export completers for convenience
pub use command::CommandCompleter;
pub use flag::FlagCompleter;
pub use path::PathCompleter; // T033 // T058

/// CompletionRegistry routes completion requests to appropriate completers (T006)
///
/// This is the main entry point for tab completion. It analyzes the input
/// context and delegates to the appropriate completer based on what's being
/// completed (command, path, or flag).
pub struct CompletionRegistry {
    // US1: Command completion
    command_completer: CommandCompleter,
    // US2: Path completion
    path_completer: PathCompleter,
    // US3: Flag completion
    flag_completer: FlagCompleter,
}

impl CompletionRegistry {
    /// Create a new CompletionRegistry with all completers
    pub fn new() -> Self {
        Self {
            command_completer: CommandCompleter::new(), // US1
            path_completer: PathCompleter::new(),       // US2
            flag_completer: FlagCompleter::new(),       // US3
        }
    }
}

impl Default for CompletionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// T034: Determine what type of completion is needed based on context
#[derive(Debug, Clone, Copy, PartialEq)]
enum CompletionContext {
    Command, // First word - complete command names
    Path,    // After first word - complete file/directory paths
    Flag,    // T059: Arguments starting with - or -- - complete flags
}

impl CompletionRegistry {
    /// T034: Analyze line and cursor position to determine completion context
    /// T059: Updated to detect flag completion context
    fn determine_context(&self, line: &str, pos: usize) -> CompletionContext {
        let before_cursor = &line[..pos];

        // Check if there's a space before the cursor
        // If no space, we're in the first word (command position)
        if !before_cursor.contains(' ') {
            CompletionContext::Command
        } else {
            // T059: Detect if current word starts with - (flag indicator)
            let last_word_start = before_cursor
                .rfind(|c: char| c.is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(0);

            let current_word = &before_cursor[last_word_start..];

            if current_word.starts_with('-') {
                CompletionContext::Flag
            } else {
                CompletionContext::Path
            }
        }
    }
}

/// T035: Route completion requests to appropriate completer
/// T060: Updated to include flag completion routing
impl Completer for CompletionRegistry {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // Determine what type of completion is needed
        let context = self.determine_context(line, pos);

        tracing::debug!(
            context = ?context,
            line = %line,
            pos = pos,
            "Routing completion request"
        );

        // Delegate to appropriate completer
        match context {
            CompletionContext::Command => self.command_completer.complete(line, pos),
            CompletionContext::Path => self.path_completer.complete(line, pos),
            CompletionContext::Flag => self.flag_completer.complete(line, pos), // T060
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_registry_new() {
        let mut registry = CompletionRegistry::new();
        // Verify it compiles and creates successfully
        assert_eq!(registry.complete("", 0).len(), 0); // Empty input returns no completions
    }

    #[test]
    fn test_completion_registry_default() {
        let mut registry = CompletionRegistry::default();
        // Empty input returns no completions
        assert_eq!(registry.complete("", 0).len(), 0);
    }

    #[test]
    fn test_completion_registry_delegates_to_command_completer() {
        let mut registry = CompletionRegistry::new();
        // Should delegate to CommandCompleter for command completion
        // Try a common command prefix
        let _suggestions = registry.complete("ca", 2);
        // Should either find matches or return empty (depending on PATH)
        // Main thing is it doesn't crash
    }
}
