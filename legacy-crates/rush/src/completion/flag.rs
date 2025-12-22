//! Flag/option completion for common commands
//!
//! Provides tab completion for command flags and options (--flag, -f).
//! Focuses on stable, commonly-used commands to avoid maintenance burden.
//!
//! Supported commands: git, cargo, ls, cd, cat, echo, grep, find
//! Intentionally excludes dynamic tools like gcloud, kubectl, aws.

use nu_ansi_term::{Color, Style};
use once_cell::sync::Lazy;
use reedline::{Completer, Span, Suggestion};
use std::collections::HashMap;

/// Definition of a command flag/option (T044)
///
/// Represents both long flags (--help) and short flags (-h)
#[derive(Debug, Clone)]
pub struct FlagDefinition {
    /// Long form of the flag (e.g., "--help")
    pub long: String,
    /// Short form of the flag (e.g., "-h"), if any
    pub short: Option<String>,
    /// Description of what the flag does
    pub description: String,
}

impl FlagDefinition {
    /// Create a new flag definition with long and short forms
    pub fn new(long: &str, short: Option<&str>, description: &str) -> Self {
        Self {
            long: long.to_string(),
            short: short.map(|s| s.to_string()),
            description: description.to_string(),
        }
    }

    /// Create a long-only flag (no short form)
    pub fn long_only(long: &str, description: &str) -> Self {
        Self::new(long, None, description)
    }
}

/// Completes command flags and options (T045)
///
/// Features:
/// - Long flags (--help, --version)
/// - Short flags (-h, -v)
/// - Command-specific flag sets
/// - Flag descriptions
/// - Short alternative display
pub struct FlagCompleter {
    /// Registry mapping command names to their flags
    registry: &'static HashMap<String, Vec<FlagDefinition>>,
}

impl FlagCompleter {
    /// Create a new FlagCompleter (T052)
    pub fn new() -> Self {
        Self { registry: &FLAG_REGISTRY }
    }

    /// Extract command name and partial flag from line (T053)
    ///
    /// Examples:
    /// - `git --ve` → Some(("git", "--ve"))
    /// - `ls -a` → Some(("ls", "-a"))
    /// - `unknown --flag` → Some(("unknown", "--flag"))
    /// - `git` → None (no flag being typed)
    fn extract_command_and_flag<'a>(
        &self,
        line: &'a str,
        pos: usize,
    ) -> Option<(&'a str, &'a str)> {
        let before_cursor = &line[..pos];

        // Split into words
        let words: Vec<&str> = before_cursor.split_whitespace().collect();

        if words.is_empty() {
            return None;
        }

        let command = words[0];

        // Find the last word (partial flag being typed)
        let last_word_start = before_cursor
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);

        let partial_flag = &before_cursor[last_word_start..];

        // Only complete if it starts with - (flag indicator)
        if partial_flag.starts_with('-') {
            Some((command, partial_flag))
        } else {
            None
        }
    }

    /// Check if a flag matches the given prefix (T054)
    ///
    /// Matches both long and short forms:
    /// - "--ve" matches "--version"
    /// - "-v" matches "-v"
    fn matches_flag(&self, flag: &FlagDefinition, prefix: &str) -> bool {
        // Check long flag
        if flag.long.starts_with(prefix) {
            return true;
        }

        // Check short flag if it exists
        if let Some(ref short) = flag.short {
            if short.starts_with(prefix) {
                return true;
            }
        }

        false
    }
}

impl Default for FlagCompleter {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement reedline's Completer trait for FlagCompleter (T055)
impl Completer for FlagCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        use std::time::Instant;
        let start = Instant::now();

        tracing::debug!(
            line = %line,
            pos = pos,
            "Flag completion triggered"
        );

        // Extract command and partial flag
        let (command, partial_flag) = match self.extract_command_and_flag(line, pos) {
            Some(result) => result,
            None => {
                tracing::debug!("Not completing flag (no flag prefix found)");
                return vec![];
            }
        };

        tracing::debug!(
            command = %command,
            partial_flag = %partial_flag,
            "Completing flag for command"
        );

        // Get flags for this command
        let flags = match self.registry.get(command) {
            Some(f) => f,
            None => {
                tracing::debug!(
                    command = %command,
                    "No flags registered for command"
                );
                return vec![];
            }
        };

        // Find matching flags (T054)
        let matches: Vec<&FlagDefinition> = flags
            .iter()
            .filter(|flag| self.matches_flag(flag, partial_flag))
            .collect();

        tracing::debug!(
            match_count = matches.len(),
            partial_flag = %partial_flag,
            "Found flag matches"
        );

        if matches.is_empty() {
            let elapsed = start.elapsed();
            tracing::info!(
                partial_flag = %partial_flag,
                command = %command,
                duration_ms = elapsed.as_millis(),
                "No matching flags found"
            );
            return vec![];
        }

        let elapsed = start.elapsed();
        tracing::info!(
            count = matches.len(),
            partial_flag = %partial_flag,
            command = %command,
            duration_ms = elapsed.as_millis(),
            "Flag completion successful"
        );

        // Convert to Suggestion objects
        let start_pos = line[..pos]
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);

        // Style for flag completions (cyan to distinguish from commands/paths)
        let flag_style = Style::new().fg(Color::Cyan);

        matches
            .into_iter()
            .map(|flag| {
                // Use long form as the primary value
                let value = flag.long.clone();

                // T056: Include description
                let description = Some(flag.description.clone());

                // T057: Include short alternative in extra field (wrapped in Vec)
                let extra = flag.short.as_ref().map(|s| vec![s.clone()]);

                Suggestion {
                    value,
                    description,
                    extra,
                    span: Span { start: start_pos, end: pos },
                    append_whitespace: true, // Append space after flag
                    style: Some(flag_style),
                }
            })
            .collect()
    }
}

/// Global flag registry (T047)
///
/// Maps command names to their available flags.
/// Uses once_cell::sync::Lazy for lazy initialization.
static FLAG_REGISTRY: Lazy<HashMap<String, Vec<FlagDefinition>>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    // T048: Git flags
    registry.insert(
        "git".to_string(),
        vec![
            FlagDefinition::new("--version", Some("-v"), "Show git version"),
            FlagDefinition::new("--help", Some("-h"), "Show help information"),
            FlagDefinition::new("--verbose", None, "Be more verbose"),
            FlagDefinition::new("--quiet", Some("-q"), "Be more quiet"),
            FlagDefinition::long_only("--git-dir", "Set the path to the repository"),
            FlagDefinition::long_only("--work-tree", "Set the path to the working tree"),
            FlagDefinition::new("--no-pager", Some("-P"), "Do not pipe output into a pager"),
            FlagDefinition::long_only("--bare", "Treat the repository as a bare repository"),
        ],
    );

    // T049: Cargo flags
    registry.insert(
        "cargo".to_string(),
        vec![
            FlagDefinition::new("--version", Some("-V"), "Print version info and exit"),
            FlagDefinition::new("--help", Some("-h"), "Print help information"),
            FlagDefinition::new("--verbose", Some("-v"), "Use verbose output"),
            FlagDefinition::new("--quiet", Some("-q"), "Do not print cargo log messages"),
            FlagDefinition::long_only("--color", "Coloring: auto, always, never"),
            FlagDefinition::long_only("--frozen", "Require Cargo.lock and cache are up to date"),
            FlagDefinition::long_only("--locked", "Require Cargo.lock is up to date"),
            FlagDefinition::long_only("--offline", "Run without accessing the network"),
            FlagDefinition::long_only("--config", "Override a configuration value"),
        ],
    );

    // T050: ls flags (common POSIX flags)
    registry.insert(
        "ls".to_string(),
        vec![
            FlagDefinition::new("--all", Some("-a"), "Do not ignore entries starting with ."),
            FlagDefinition::new("--almost-all", Some("-A"), "Do not list . and .."),
            FlagDefinition::long_only("--author", "Print the author of each file"),
            FlagDefinition::new("--escape", Some("-b"), "Print C-style escapes"),
            FlagDefinition::long_only("--block-size", "Scale sizes by SIZE before printing"),
            FlagDefinition::new(
                "--ignore-backups",
                Some("-B"),
                "Do not list entries ending with ~",
            ),
            FlagDefinition::new("--directory", Some("-d"), "List directories themselves"),
            FlagDefinition::new("--classify", Some("-F"), "Append indicator to entries"),
            FlagDefinition::long_only("--file-type", "Likewise, except do not append *"),
            FlagDefinition::long_only(
                "--format",
                "Across, commas, horizontal, long, verbose, vertical",
            ),
            FlagDefinition::new("--no-group", Some("-G"), "Do not print group names"),
            FlagDefinition::new("--human-readable", Some("-h"), "Print human readable sizes"),
            FlagDefinition::new(
                "--dereference-command-line",
                Some("-H"),
                "Follow symbolic links on command line",
            ),
            FlagDefinition::new("--inode", Some("-i"), "Print the index number of each file"),
            FlagDefinition::new("--size", Some("-s"), "Print the allocated size of each file"),
            FlagDefinition::new("--sort", Some("-S"), "Sort by file size, largest first"),
            FlagDefinition::new("--time", Some("-t"), "Sort by time, newest first"),
            FlagDefinition::new("--recursive", Some("-R"), "List subdirectories recursively"),
            FlagDefinition::new("--reverse", Some("-r"), "Reverse order while sorting"),
            FlagDefinition::new("-l", None, "Use a long listing format"),
        ],
    );

    // T051: cd, cat, echo, grep, find flags

    // cd has very few flags (mostly shell built-in)
    registry.insert(
        "cd".to_string(),
        vec![
            FlagDefinition::new("-L", None, "Follow symbolic links (default)"),
            FlagDefinition::new("-P", None, "Use the physical directory structure"),
        ],
    );

    // cat flags
    registry.insert(
        "cat".to_string(),
        vec![
            FlagDefinition::new("--number", Some("-n"), "Number all output lines"),
            FlagDefinition::new("--number-nonblank", Some("-b"), "Number nonempty output lines"),
            FlagDefinition::new("--show-all", Some("-A"), "Equivalent to -vET"),
            FlagDefinition::new("--show-ends", Some("-E"), "Display $ at end of each line"),
            FlagDefinition::new("--show-tabs", Some("-T"), "Display TAB characters as ^I"),
            FlagDefinition::new("--show-nonprinting", Some("-v"), "Use ^ and M- notation"),
            FlagDefinition::new(
                "--squeeze-blank",
                Some("-s"),
                "Suppress repeated empty output lines",
            ),
        ],
    );

    // echo flags (minimal, mostly built-in)
    registry.insert(
        "echo".to_string(),
        vec![
            FlagDefinition::new("-n", None, "Do not output the trailing newline"),
            FlagDefinition::new("-e", None, "Enable interpretation of backslash escapes"),
            FlagDefinition::new(
                "-E",
                None,
                "Disable interpretation of backslash escapes (default)",
            ),
        ],
    );

    // grep flags
    registry.insert(
        "grep".to_string(),
        vec![
            FlagDefinition::new(
                "--extended-regexp",
                Some("-E"),
                "PATTERNS are extended regular expressions",
            ),
            FlagDefinition::new("--fixed-strings", Some("-F"), "PATTERNS are strings"),
            FlagDefinition::new(
                "--basic-regexp",
                Some("-G"),
                "PATTERNS are basic regular expressions",
            ),
            FlagDefinition::new(
                "--perl-regexp",
                Some("-P"),
                "PATTERNS are Perl regular expressions",
            ),
            FlagDefinition::new("--regexp", Some("-e"), "Use PATTERNS for matching"),
            FlagDefinition::new("--file", Some("-f"), "Take PATTERNS from FILE"),
            FlagDefinition::new("--ignore-case", Some("-i"), "Ignore case distinctions"),
            FlagDefinition::new("--invert-match", Some("-v"), "Select non-matching lines"),
            FlagDefinition::new("--word-regexp", Some("-w"), "Match only whole words"),
            FlagDefinition::new("--line-regexp", Some("-x"), "Match only whole lines"),
            FlagDefinition::new("--count", Some("-c"), "Print only a count of matching lines"),
            FlagDefinition::long_only("--color", "Use markers to highlight matching strings"),
            FlagDefinition::new("--line-number", Some("-n"), "Print line number with output lines"),
            FlagDefinition::new("--with-filename", Some("-H"), "Print file name with output lines"),
            FlagDefinition::new("--no-filename", Some("-h"), "Suppress file name prefix on output"),
            FlagDefinition::new("--recursive", Some("-r"), "Read all files under each directory"),
            FlagDefinition::new("--only-matching", Some("-o"), "Show only matching part of lines"),
        ],
    );

    // find flags (common ones)
    registry.insert(
        "find".to_string(),
        vec![
            FlagDefinition::new("-name", None, "Base of file name matches shell pattern"),
            FlagDefinition::new("-iname", None, "Like -name, but case insensitive"),
            FlagDefinition::new("-type", None, "File is of type (f=file, d=directory, l=link)"),
            FlagDefinition::new("-size", None, "File uses n units of space"),
            FlagDefinition::new("-mtime", None, "File's data was last modified n*24 hours ago"),
            FlagDefinition::new("-atime", None, "File was last accessed n*24 hours ago"),
            FlagDefinition::new("-user", None, "File is owned by user"),
            FlagDefinition::new("-group", None, "File belongs to group"),
            FlagDefinition::new("-perm", None, "File's permission bits are exactly mode"),
            FlagDefinition::new("-exec", None, "Execute command; true if returns 0"),
            FlagDefinition::new("-print", None, "Print the full file name on the standard output"),
            FlagDefinition::new("-delete", None, "Delete files; true if removal succeeded"),
            FlagDefinition::new("-maxdepth", None, "Descend at most levels deep"),
            FlagDefinition::new("-mindepth", None, "Do not apply tests at levels less than"),
        ],
    );

    registry
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_definition_new() {
        let flag = FlagDefinition::new("--help", Some("-h"), "Show help");
        assert_eq!(flag.long, "--help");
        assert_eq!(flag.short, Some("-h".to_string()));
        assert_eq!(flag.description, "Show help");
    }

    #[test]
    fn test_flag_definition_long_only() {
        let flag = FlagDefinition::long_only("--verbose", "Be verbose");
        assert_eq!(flag.long, "--verbose");
        assert_eq!(flag.short, None);
        assert_eq!(flag.description, "Be verbose");
    }

    #[test]
    fn test_flag_completion_suggestions_have_correct_format() {
        let mut completer = FlagCompleter::new();
        let suggestions = completer.complete("git --ver", 9);

        assert!(!suggestions.is_empty());
        let suggestion = &suggestions[0];
        assert_eq!(suggestion.value, "--version");
        assert_eq!(suggestion.description, Some("Show git version".to_string()));
    }

    #[test]
    fn test_flag_completer_new() {
        let _completer = FlagCompleter::new();
        // Should create successfully
    }

    #[test]
    fn test_extract_command_and_flag() {
        let completer = FlagCompleter::new();

        assert_eq!(completer.extract_command_and_flag("git --ve", 8), Some(("git", "--ve")));

        assert_eq!(completer.extract_command_and_flag("ls -a", 5), Some(("ls", "-a")));

        // No flag prefix
        assert_eq!(completer.extract_command_and_flag("git status", 10), None);

        // Just command, no args
        assert_eq!(completer.extract_command_and_flag("git", 3), None);
    }

    #[test]
    fn test_matches_flag() {
        let completer = FlagCompleter::new();
        let flag = FlagDefinition::new("--version", Some("-v"), "Show version");

        assert!(completer.matches_flag(&flag, "--v"));
        assert!(completer.matches_flag(&flag, "--ver"));
        assert!(completer.matches_flag(&flag, "--version"));
        assert!(completer.matches_flag(&flag, "-v"));
        assert!(!completer.matches_flag(&flag, "--h"));
        assert!(!completer.matches_flag(&flag, "-h"));
    }

    #[test]
    fn test_flag_registry_contains_commands() {
        assert!(FLAG_REGISTRY.contains_key("git"));
        assert!(FLAG_REGISTRY.contains_key("cargo"));
        assert!(FLAG_REGISTRY.contains_key("ls"));
        assert!(FLAG_REGISTRY.contains_key("cd"));
        assert!(FLAG_REGISTRY.contains_key("cat"));
        assert!(FLAG_REGISTRY.contains_key("echo"));
        assert!(FLAG_REGISTRY.contains_key("grep"));
        assert!(FLAG_REGISTRY.contains_key("find"));
    }

    #[test]
    fn test_flag_registry_git_flags() {
        let git_flags = FLAG_REGISTRY.get("git").unwrap();
        assert!(!git_flags.is_empty());

        // Check for --version flag
        let has_version = git_flags.iter().any(|f| f.long == "--version");
        assert!(has_version);
    }
}
