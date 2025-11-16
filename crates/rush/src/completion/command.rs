//! Command completion from PATH executables
//!
//! Provides tab completion for command names by scanning executables in the PATH
//! environment variable. Uses lazy-loaded caching for performance.

use nu_ansi_term::{Color, Style};
use reedline::{Completer, Span, Suggestion};
use std::collections::HashSet;
use std::env;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Completes command names from executables found in PATH
///
/// This completer scans the PATH environment variable on first use and caches
/// the results for the duration of the session. It performs prefix matching
/// (case-sensitive on Linux, case-insensitive on macOS) and limits results to
/// 50 items to avoid overwhelming the user.
pub struct CommandCompleter {
    /// Lazy-loaded cache of executable names from PATH
    cache: Option<HashSet<String>>,

    /// Whether to perform case-sensitive matching (platform-dependent)
    #[cfg(target_os = "macos")]
    case_sensitive: bool,

    #[cfg(not(target_os = "macos"))]
    case_sensitive: bool,
}

impl CommandCompleter {
    /// Create a new CommandCompleter with platform-specific case sensitivity (T008)
    ///
    /// # Platform behavior
    /// - macOS: Case-insensitive matching (HFS+/APFS default)
    /// - Linux: Case-sensitive matching
    pub fn new() -> Self {
        Self {
            cache: None, // Lazy-loaded on first completion request
            #[cfg(target_os = "macos")]
            case_sensitive: false,
            #[cfg(not(target_os = "macos"))]
            case_sensitive: true,
        }
    }

    /// Scan PATH environment variable and return set of executable names (T009)
    ///
    /// This method:
    /// - Reads PATH environment variable
    /// - Splits into directories
    /// - Scans each directory for executable files
    /// - Returns set of unique command names
    ///
    /// # Performance
    /// Typically takes 50-100ms on first call with ~1000 executables in PATH.
    fn scan_path(&self) -> HashSet<String> {
        use std::time::Instant;
        let start = Instant::now();

        let mut executables = HashSet::new();

        let path = match env::var("PATH") {
            Ok(p) => p,
            Err(_) => {
                tracing::warn!("PATH environment variable not set");
                return executables;
            }
        };

        let dirs: Vec<&str> = path.split(':').collect();
        tracing::debug!(dir_count = dirs.len(), "Scanning PATH directories");

        for dir in dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    // Check if file is executable
                    if self.is_executable(&entry) {
                        if let Ok(name) = entry.file_name().into_string() {
                            executables.insert(name);
                        }
                    }
                }
            }
            // Silently skip directories we can't read (permission denied, etc.)
        }

        let elapsed = start.elapsed();
        tracing::info!(
            count = executables.len(),
            duration_ms = elapsed.as_millis(),
            "PATH scan completed (cached for session)"
        );

        executables
    }

    /// Check if a directory entry is an executable file
    #[cfg(unix)]
    fn is_executable(&self, entry: &fs::DirEntry) -> bool {
        if let Ok(metadata) = entry.metadata() {
            if !metadata.is_file() {
                return false;
            }
            // Check execute permission bits (owner, group, or other)
            let permissions = metadata.permissions();
            permissions.mode() & 0o111 != 0
        } else {
            false
        }
    }

    /// Ensure cache is loaded before use (T010)
    ///
    /// Lazy initialization: cache is populated on first completion request.
    /// Subsequent requests use the cached data.
    fn ensure_cache_loaded(&mut self) {
        if self.cache.is_none() {
            self.cache = Some(self.scan_path());
        }
    }

    /// Check if a command matches the given prefix (T011)
    ///
    /// Matching is case-sensitive or case-insensitive depending on platform.
    fn matches_prefix(&self, command: &str, prefix: &str) -> bool {
        if self.case_sensitive {
            command.starts_with(prefix)
        } else {
            command.to_lowercase().starts_with(&prefix.to_lowercase())
        }
    }

    /// Extract the partial command from the line at cursor position
    ///
    /// For command completion, we only complete the first word.
    fn extract_partial_command(&self, line: &str, pos: usize) -> Option<String> {
        // Only complete if we're in the first word
        let before_cursor = &line[..pos];

        // Check if there are any spaces before cursor (would mean we're not in first word)
        if before_cursor.contains(' ') {
            return None;
        }

        Some(before_cursor.to_string())
    }
}

impl Default for CommandCompleter {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement reedline's Completer trait for CommandCompleter (T012, T013)
impl Completer for CommandCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        use std::time::Instant;
        let start = Instant::now();

        tracing::debug!(
            line = %line,
            pos = pos,
            "Tab completion triggered for command"
        );

        // Extract partial command
        let partial = match self.extract_partial_command(line, pos) {
            Some(p) => p,
            None => {
                tracing::debug!(
                    line = %line,
                    pos = pos,
                    "Not completing command (cursor in arguments or after space)"
                );
                return vec![]; // Not completing a command (in arguments)
            }
        };

        tracing::debug!(
            partial = %partial,
            partial_len = partial.len(),
            "Completing partial command"
        );

        // Ensure cache is loaded
        let was_cached = self.cache.is_some();
        self.ensure_cache_loaded();

        // Get cache reference
        let cache = match &self.cache {
            Some(c) => c,
            None => {
                tracing::warn!("Cache load failed unexpectedly");
                return vec![];
            }
        };

        tracing::debug!(cache_size = cache.len(), was_cached = was_cached, "Using PATH cache");

        // Find matching commands
        let mut matches: Vec<String> = cache
            .iter()
            .filter(|cmd| self.matches_prefix(cmd, &partial))
            .cloned()
            .collect();

        // Sort alphabetically for consistent ordering
        matches.sort();

        // Log match count and sample
        let sample: Vec<&str> = matches.iter().take(10).map(|s| s.as_str()).collect();
        tracing::debug!(
            match_count = matches.len(),
            sample = ?sample,
            "Found matching commands"
        );

        // Limit to 50 matches (T013)
        if matches.len() > 50 {
            let elapsed = start.elapsed();
            let sample: Vec<&str> = matches.iter().take(5).map(|s| s.as_str()).collect();
            tracing::warn!(
                count = matches.len(),
                partial = %partial,
                sample = ?sample,
                duration_ms = elapsed.as_millis(),
                "Too many matches for command completion - returning empty vec (user should type more characters)"
            );

            // Return empty vec when too many matches to avoid overwhelming the completion menu
            // User will need to type more characters to narrow down results
            // TODO(polish): Consider showing first 50 with a status message in the menu
            return vec![];
        }

        // Handle no matches case
        if matches.is_empty() {
            let elapsed = start.elapsed();
            tracing::info!(
                partial = %partial,
                duration_ms = elapsed.as_millis(),
                "No matching commands found - returning empty vec"
            );
            return vec![];
        }

        let elapsed = start.elapsed();
        let result_preview: Vec<&str> = matches.iter().take(5).map(|s| s.as_str()).collect();
        tracing::info!(
            count = matches.len(),
            partial = %partial,
            results = ?result_preview,
            duration_ms = elapsed.as_millis(),
            "Command completion successful - returning {} suggestions",
            matches.len()
        );

        // Convert to Suggestion objects with warm tone styling for dark terminals
        let completion_style = Style::new().fg(Color::Yellow).bold();
        matches
            .into_iter()
            .map(|cmd| Suggestion {
                value: cmd,
                description: None, // No descriptions for commands
                extra: None,
                span: Span { start: 0, end: pos }, // Replace entire first word
                append_whitespace: true,           // Add space after command
                style: Some(completion_style), // Warm yellow + bold for visibility on dark backgrounds
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_completer_new() {
        let completer = CommandCompleter::new();
        assert!(completer.cache.is_none()); // Cache is lazy-loaded
    }

    #[test]
    fn test_command_completer_default() {
        let completer = CommandCompleter::default();
        assert!(completer.cache.is_none());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_case_insensitive_on_macos() {
        let completer = CommandCompleter::new();
        assert!(!completer.case_sensitive);
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_case_sensitive_on_linux() {
        let completer = CommandCompleter::new();
        assert!(completer.case_sensitive);
    }
}
