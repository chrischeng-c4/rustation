//! Path completion for files and directories
//!
//! Provides tab completion for file and directory paths when typing arguments.
//! Supports relative paths, absolute paths, tilde expansion, hidden files, and
//! proper handling of paths with spaces.

use nu_ansi_term::{Color, Style};
use reedline::{Completer, Span, Suggestion};
use std::fs;

/// Completes file and directory paths for command arguments
///
/// Features:
/// - Relative and absolute paths
/// - Tilde expansion (~/ → home directory)
/// - Hidden files (shown only when prefix starts with '.')
/// - Directory markers (appends '/' to directories)
/// - Quoted paths with spaces
/// - Platform-specific case sensitivity
pub struct PathCompleter {
    /// Whether to perform case-sensitive matching (platform-dependent)
    #[cfg(target_os = "macos")]
    case_sensitive: bool,

    #[cfg(not(target_os = "macos"))]
    case_sensitive: bool,
}

impl PathCompleter {
    /// Create a new PathCompleter with platform-specific case sensitivity (T023)
    ///
    /// # Platform behavior
    /// - macOS: Case-insensitive matching (HFS+/APFS default)
    /// - Linux: Case-sensitive matching
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            case_sensitive: false,
            #[cfg(not(target_os = "macos"))]
            case_sensitive: true,
        }
    }

    /// Extract the partial path from the line at cursor position (T024)
    ///
    /// Extracts the path fragment being completed, handling:
    /// - Relative paths: `src/m` → `src/m`
    /// - Absolute paths: `/usr/l` → `/usr/l`
    /// - Tilde paths: `~/D` → `~/D`
    /// - Multiple arguments: `ls src/m other` (cursor at 8) → `src/m`
    fn extract_partial_path(&self, line: &str, pos: usize) -> Option<String> {
        let before_cursor = &line[..pos];

        // Find the start of the current argument (after last space)
        let start = before_cursor.rfind(' ').map(|i| i + 1).unwrap_or(0);

        // Extract from start to cursor
        let partial = before_cursor[start..].to_string();

        // Only complete if we're not in the first word (that's command completion)
        if start == 0 {
            return None;
        }

        Some(partial)
    }

    /// Split path into parent directory and filename prefix (T025)
    ///
    /// Examples:
    /// - `src/main.rs` → (`src/`, `main.rs`)
    /// - `main.rs` → (`./`, `main.rs`)
    /// - `/usr/bin/git` → (`/usr/bin/`, `git`)
    /// - `~/Documents/test` → (`~/Documents/`, `test`)
    fn split_path_and_prefix(&self, path: &str) -> (String, String) {
        // Handle empty path
        if path.is_empty() {
            return ("./".to_string(), String::new());
        }

        // Find the last path separator
        let sep_idx = path.rfind('/');

        match sep_idx {
            Some(idx) => {
                // Has a directory component
                let parent = &path[..=idx]; // Include the /
                let prefix = &path[idx + 1..];
                (parent.to_string(), prefix.to_string())
            }
            None => {
                // No directory component, use current directory
                ("./".to_string(), path.to_string())
            }
        }
    }

    /// Expand tilde (~) to home directory (T030)
    ///
    /// Examples:
    /// - `~/Documents` → `/Users/username/Documents`
    /// - `~` → `/Users/username`
    /// - `./test` → `./test` (no change)
    fn expand_tilde(&self, path: &str) -> String {
        if path.starts_with("~/") || path == "~" {
            if let Some(home) = dirs::home_dir() {
                if path == "~" {
                    return home.to_string_lossy().to_string();
                } else {
                    return path.replacen("~", &home.to_string_lossy(), 1);
                }
            }
        }
        path.to_string()
    }

    /// Check if filename matches prefix (case-sensitive or insensitive)
    fn matches_prefix(&self, name: &str, prefix: &str) -> bool {
        if self.case_sensitive {
            name.starts_with(prefix)
        } else {
            name.to_lowercase().starts_with(&prefix.to_lowercase())
        }
    }

    /// List directory entries matching the given prefix (T026)
    ///
    /// Returns matching files and directories with appropriate formatting:
    /// - Directories have '/' appended
    /// - Paths with spaces are quoted
    /// - Hidden files only shown if prefix starts with '.'
    fn list_directory_entries(
        &self,
        parent: &str,
        prefix: &str,
    ) -> Result<Vec<String>, std::io::Error> {
        // Expand tilde in parent directory
        let parent_expanded = self.expand_tilde(parent);

        // Read directory
        let entries = fs::read_dir(&parent_expanded)?;

        let mut matches = Vec::new();

        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                // T027: Skip hidden files unless prefix starts with '.'
                if name.starts_with('.') && !prefix.starts_with('.') {
                    continue;
                }

                // Check if name matches prefix
                if self.matches_prefix(&name, prefix) {
                    // T028: Append '/' to directories
                    let mut display_name = name.clone();
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            display_name.push('/');
                        }
                    }

                    // T028: Quote paths with spaces
                    if display_name.contains(' ') {
                        display_name = format!("\"{}\"", display_name);
                    }

                    matches.push(display_name);
                }
            }
        }

        // Sort alphabetically
        matches.sort();

        Ok(matches)
    }
}

impl Default for PathCompleter {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement reedline's Completer trait for PathCompleter (T029)
impl Completer for PathCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        use std::time::Instant;
        let start = Instant::now();

        tracing::debug!(
            line = %line,
            pos = pos,
            "Path completion triggered"
        );

        // Extract partial path
        let partial = match self.extract_partial_path(line, pos) {
            Some(p) => p,
            None => {
                tracing::debug!("Not completing path (in first word - command position)");
                return vec![];
            }
        };

        tracing::debug!(
            partial = %partial,
            "Completing partial path"
        );

        // Split into parent directory and filename prefix
        let (parent, prefix) = self.split_path_and_prefix(&partial);

        tracing::debug!(
            parent = %parent,
            prefix = %prefix,
            "Split path into parent and prefix"
        );

        // List matching entries
        let matches = match self.list_directory_entries(&parent, &prefix) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    parent = %parent,
                    "Failed to read directory for path completion"
                );
                return vec![];
            }
        };

        tracing::debug!(
            match_count = matches.len(),
            sample = ?matches.iter().take(5).collect::<Vec<_>>(),
            "Found path matches"
        );

        // T032: Limit to 50 matches
        if matches.len() > 50 {
            let elapsed = start.elapsed();
            tracing::warn!(
                count = matches.len(),
                partial = %partial,
                duration_ms = elapsed.as_millis(),
                "Too many path matches - returning empty vec"
            );
            return vec![];
        }

        if matches.is_empty() {
            let elapsed = start.elapsed();
            tracing::info!(
                partial = %partial,
                duration_ms = elapsed.as_millis(),
                "No matching paths found"
            );
            return vec![];
        }

        let elapsed = start.elapsed();
        tracing::info!(
            count = matches.len(),
            partial = %partial,
            duration_ms = elapsed.as_millis(),
            "Path completion successful"
        );

        // Convert to Suggestion objects
        // Calculate the span to replace (entire partial path)
        let start_pos = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);

        // Style for path completions (green to distinguish from commands)
        let path_style = Style::new().fg(Color::Green);

        matches
            .into_iter()
            .map(|path| {
                // Reconstruct full path for display
                let mut value = parent.clone();
                value.push_str(&path);

                Suggestion {
                    value,
                    description: None,
                    extra: None,
                    span: Span { start: start_pos, end: pos },
                    append_whitespace: false, // Don't append space after paths (user might add more path)
                    style: Some(path_style),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_completer_new() {
        let completer = PathCompleter::new();

        #[cfg(target_os = "macos")]
        assert!(!completer.case_sensitive);

        #[cfg(not(target_os = "macos"))]
        assert!(completer.case_sensitive);
    }

    #[test]
    fn test_split_path_and_prefix() {
        let completer = PathCompleter::new();

        assert_eq!(
            completer.split_path_and_prefix("src/main.rs"),
            ("src/".to_string(), "main.rs".to_string())
        );

        assert_eq!(
            completer.split_path_and_prefix("main.rs"),
            ("./".to_string(), "main.rs".to_string())
        );

        assert_eq!(
            completer.split_path_and_prefix("/usr/bin/git"),
            ("/usr/bin/".to_string(), "git".to_string())
        );

        assert_eq!(completer.split_path_and_prefix(""), ("./".to_string(), String::new()));
    }

    #[test]
    fn test_expand_tilde() {
        let completer = PathCompleter::new();

        // Tilde should expand to home directory
        let expanded = completer.expand_tilde("~/Documents");
        assert!(expanded.contains("Documents"));
        assert!(!expanded.starts_with("~"));

        // Non-tilde paths should remain unchanged
        assert_eq!(completer.expand_tilde("./test"), "./test");
        assert_eq!(completer.expand_tilde("/usr/local"), "/usr/local");
    }

    #[test]
    fn test_extract_partial_path() {
        let completer = PathCompleter::new();

        // Should extract path from argument position
        assert_eq!(completer.extract_partial_path("ls src/m", 8), Some("src/m".to_string()));

        // Should return None for first word (command position)
        assert_eq!(completer.extract_partial_path("ls", 2), None);

        // Should handle cursor in middle of argument
        assert_eq!(
            completer.extract_partial_path("ls src/main.rs other", 14),
            Some("src/main.rs".to_string())
        );
    }
}
