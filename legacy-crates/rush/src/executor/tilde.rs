// Tilde expansion module
//
// Expands tilde patterns in shell commands:
// - `~` → user's home directory ($HOME)
// - `~+` → current working directory ($PWD)
// - `~-` → previous working directory ($OLDPWD)
// - `~username` → specified user's home directory

use std::env;

/// Expands tilde patterns in the input string.
///
/// Tilde expansion replaces tilde shortcuts with actual directory paths.
/// Examples:
/// - `~` → `/home/user`
/// - `~/docs` → `/home/user/docs`
/// - `~+` → `/current/working/dir`
/// - `~-` → `/previous/working/dir`
/// - `~root` → `/root`
///
/// Respects quoting and escaping rules:
/// - Single quotes prevent expansion: `'~'` → `'~'`
/// - Double quotes allow expansion: `"~"` → `"/home/user"`
/// - Backslash escapes: `\~` → `\~`
///
/// # Arguments
/// * `input` - The input string potentially containing tilde patterns
///
/// # Returns
/// The expanded string with all tilde patterns replaced by their expansions
pub fn expand_tilde(input: &str) -> String {
    // Quick check: if no tilde, return as-is
    if !input.contains('~') {
        return input.to_string();
    }

    let mut result = String::new();
    let mut current_word = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current_word.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                current_word.push(ch);
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                current_word.push(ch);
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                current_word.push(ch);
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                // Word boundary - expand and add to result
                if !current_word.is_empty() {
                    let expanded = expand_word(&current_word);
                    result.push_str(&expanded);
                    current_word.clear();
                }
                result.push(ch);
            }
            _ => {
                current_word.push(ch);
            }
        }
    }

    // Don't forget the last word
    if !current_word.is_empty() {
        let expanded = expand_word(&current_word);
        result.push_str(&expanded);
    }

    result
}

/// Expand a single word containing tilde patterns
///
/// Handles quotes internally - expands tilde in double quotes but not in single quotes.
fn expand_word(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    let mut result = String::new();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    while i < chars.len() {
        if escape_next {
            result.push(chars[i]);
            escape_next = false;
            i += 1;
            continue;
        }

        match chars[i] {
            '\\' => {
                result.push(chars[i]);
                escape_next = true;
                i += 1;
            }
            '\'' if !in_double_quote => {
                result.push(chars[i]);
                in_single_quote = !in_single_quote;
                i += 1;
            }
            '"' if !in_single_quote => {
                result.push(chars[i]);
                in_double_quote = !in_double_quote;
                i += 1;
            }
            '~' if !in_single_quote => {
                // Only expand if at start of word or right after opening double quote
                let should_expand = i == 0 || (i > 0 && chars[i - 1] == '"');

                if should_expand {
                    // Collect tilde prefix (up to '/' or next special char)
                    let mut prefix_end = i + 1;
                    while prefix_end < chars.len() {
                        match chars[prefix_end] {
                            '/' => break, // Slash ends the prefix but is part of remainder
                            '"' | '\'' | ' ' | '\t' | '\\' => break, // Special chars end the prefix
                            _ => prefix_end += 1,
                        }
                    }

                    // Collect the tilde prefix and remainder (up to special char, not including it)
                    let tilde_part: String = chars[i..prefix_end].iter().collect();
                    let remainder_start = if prefix_end < chars.len() && chars[prefix_end] == '/' {
                        // Include the slash and everything after it
                        let mut rem_end = prefix_end;
                        while rem_end < chars.len()
                            && !matches!(chars[rem_end], '"' | '\'' | ' ' | '\t' | '\\')
                        {
                            rem_end += 1;
                        }
                        (prefix_end, rem_end)
                    } else {
                        (prefix_end, prefix_end)
                    };

                    let full_tilde_expr = if remainder_start.1 > remainder_start.0 {
                        let rem: String =
                            chars[remainder_start.0..remainder_start.1].iter().collect();
                        format!("{}{}", tilde_part, rem)
                    } else {
                        tilde_part
                    };

                    if let Some(expanded) = expand_tilde_prefix(&full_tilde_expr) {
                        result.push_str(&expanded);
                        i = remainder_start.1;
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            _ => {
                result.push(chars[i]);
                i += 1;
            }
        }
    }

    result
}

/// Expand a tilde prefix pattern
///
/// Handles:
/// - `~` → $HOME
/// - `~+` → $PWD
/// - `~-` → $OLDPWD
/// - `~username` → user's home directory
fn expand_tilde_prefix(word: &str) -> Option<String> {
    if !word.starts_with('~') {
        return None;
    }

    // Parse the tilde prefix
    let (prefix, remainder) = parse_tilde_prefix(word)?;

    let expanded_path = match prefix {
        "~" => get_home_dir()?,
        "~+" => env::var("PWD").ok()?,
        "~-" => env::var("OLDPWD").ok()?,
        _ if prefix.starts_with("~") => {
            // ~username pattern
            let username = &prefix[1..];
            get_user_home(username)?
        }
        _ => return None,
    };

    Some(format!("{}{}", expanded_path, remainder))
}

/// Parse a tilde prefix from a word
///
/// Returns (prefix, remainder) where:
/// - prefix: "~", "~+", "~-", or "~username"
/// - remainder: the rest of the word after the prefix (including leading /)
fn parse_tilde_prefix(word: &str) -> Option<(&str, &str)> {
    if !word.starts_with('~') {
        return None;
    }

    // Find where the tilde prefix ends (at '/' or end of word)
    if let Some(slash_pos) = word.find('/') {
        Some((&word[..slash_pos], &word[slash_pos..]))
    } else {
        Some((word, ""))
    }
}

/// Get the user's home directory from $HOME
fn get_home_dir() -> Option<String> {
    env::var("HOME").ok()
}

/// Get a user's home directory by username
///
/// Currently returns None - will be implemented in Phase 5 (US3)
#[allow(unused_variables)]
fn get_user_home(username: &str) -> Option<String> {
    // TODO: Implement user lookup using nix crate or libc
    // For now, return None to leave ~username unexpanded
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    // Helper to set/unset HOME for testing
    // NOTE: Tests using this must be marked #[serial] to avoid race conditions
    fn with_home<F>(home: Option<&str>, f: F)
    where
        F: FnOnce(),
    {
        let original = env::var("HOME").ok();

        if let Some(h) = home {
            env::set_var("HOME", h);
        } else {
            env::remove_var("HOME");
        }

        f();

        // Restore original
        if let Some(h) = original {
            env::set_var("HOME", h);
        } else {
            env::remove_var("HOME");
        }
    }

    // === User Story 1: Basic ~ Expansion Tests ===

    #[test]
    fn test_no_tilde() {
        assert_eq!(expand_tilde("hello world"), "hello world");
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(expand_tilde(""), "");
    }

    #[test]
    #[serial]
    fn test_basic_tilde() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("~"), "/home/user");
        });
    }

    #[test]
    #[serial]
    fn test_tilde_with_path() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("~/documents"), "/home/user/documents");
            assert_eq!(expand_tilde("~/docs/file.txt"), "/home/user/docs/file.txt");
        });
    }

    #[test]
    #[serial]
    fn test_tilde_in_command() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("cd ~"), "cd /home/user");
            assert_eq!(expand_tilde("ls ~/projects"), "ls /home/user/projects");
        });
    }

    #[test]
    #[serial]
    fn test_multiple_tildes() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("~ ~"), "/home/user /home/user");
            assert_eq!(expand_tilde("diff ~/a ~/b"), "diff /home/user/a /home/user/b");
        });
    }

    #[test]
    #[serial]
    fn test_tilde_with_trailing_slash() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("~/"), "/home/user/");
        });
    }

    #[test]
    #[serial]
    fn test_missing_home() {
        with_home(None, || {
            // When HOME is unset, leave tilde unexpanded
            assert_eq!(expand_tilde("~"), "~");
            assert_eq!(expand_tilde("~/documents"), "~/documents");
        });
    }

    // === Quote/Escape Handling Tests ===

    #[test]
    #[serial]
    fn test_single_quotes_no_expand() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("'~'"), "'~'");
            assert_eq!(expand_tilde("echo '~'"), "echo '~'");
            assert_eq!(expand_tilde("'~/path'"), "'~/path'");
        });
    }

    #[test]
    #[serial]
    fn test_double_quotes_expand() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("\"~\""), "\"/home/user\"");
            assert_eq!(expand_tilde("echo \"~\""), "echo \"/home/user\"");
            assert_eq!(expand_tilde("\"~/path\""), "\"/home/user/path\"");
        });
    }

    #[test]
    #[serial]
    fn test_escaped_tilde() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("\\~"), "\\~");
            assert_eq!(expand_tilde("echo \\~"), "echo \\~");
        });
    }

    #[test]
    #[serial]
    fn test_mixed_quotes() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("'~' \"~\""), "'~' \"/home/user\"");
            assert_eq!(expand_tilde("~ '~' ~"), "/home/user '~' /home/user");
        });
    }

    // === Edge Cases ===

    #[test]
    #[serial]
    fn test_tilde_mid_word() {
        with_home(Some("/home/user"), || {
            // Tilde only expands at word start
            assert_eq!(expand_tilde("a~b"), "a~b");
            assert_eq!(expand_tilde("file~txt"), "file~txt");
        });
    }

    #[test]
    #[serial]
    fn test_tilde_after_word_boundary() {
        with_home(Some("/home/user"), || {
            // Each word boundary resets, so tilde at start of new word should expand
            assert_eq!(expand_tilde("echo ~"), "echo /home/user");
            assert_eq!(expand_tilde("cd ~"), "cd /home/user");
        });
    }

    #[test]
    #[serial]
    fn test_empty_home() {
        with_home(Some(""), || {
            // Empty HOME should expand to empty string
            assert_eq!(expand_tilde("~"), "");
            assert_eq!(expand_tilde("~/path"), "/path");
        });
    }

    #[test]
    #[serial]
    fn test_whitespace_preservation() {
        with_home(Some("/home/user"), || {
            assert_eq!(expand_tilde("  ~  "), "  /home/user  ");
            assert_eq!(expand_tilde("~\t~/"), "/home/user\t/home/user/");
        });
    }

    #[test]
    #[serial]
    fn test_complex_path() {
        with_home(Some("/home/user"), || {
            assert_eq!(
                expand_tilde("~/projects/rust-station/src/main.rs"),
                "/home/user/projects/rust-station/src/main.rs"
            );
        });
    }

    // === User Story 2: Working Directory Shortcuts Tests ===

    // Helper to set/unset PWD for testing
    fn with_pwd<F>(pwd: Option<&str>, f: F)
    where
        F: FnOnce(),
    {
        let original = env::var("PWD").ok();

        if let Some(p) = pwd {
            env::set_var("PWD", p);
        } else {
            env::remove_var("PWD");
        }

        f();

        // Restore original
        if let Some(p) = original {
            env::set_var("PWD", p);
        } else {
            env::remove_var("PWD");
        }
    }

    // Helper to set/unset OLDPWD for testing
    fn with_oldpwd<F>(oldpwd: Option<&str>, f: F)
    where
        F: FnOnce(),
    {
        let original = env::var("OLDPWD").ok();

        if let Some(o) = oldpwd {
            env::set_var("OLDPWD", o);
        } else {
            env::remove_var("OLDPWD");
        }

        f();

        // Restore original
        if let Some(o) = original {
            env::set_var("OLDPWD", o);
        } else {
            env::remove_var("OLDPWD");
        }
    }

    #[test]
    #[serial]
    fn test_tilde_plus() {
        with_pwd(Some("/current/directory"), || {
            assert_eq!(expand_tilde("~+"), "/current/directory");
            assert_eq!(expand_tilde("echo ~+"), "echo /current/directory");
        });
    }

    #[test]
    #[serial]
    fn test_tilde_plus_with_path() {
        with_pwd(Some("/current/directory"), || {
            assert_eq!(expand_tilde("~+/file.txt"), "/current/directory/file.txt");
            assert_eq!(expand_tilde("ls ~+/src"), "ls /current/directory/src");
        });
    }

    #[test]
    #[serial]
    fn test_tilde_minus() {
        with_oldpwd(Some("/previous/directory"), || {
            assert_eq!(expand_tilde("~-"), "/previous/directory");
            assert_eq!(expand_tilde("cd ~-"), "cd /previous/directory");
        });
    }

    #[test]
    #[serial]
    fn test_tilde_minus_with_path() {
        with_oldpwd(Some("/previous/directory"), || {
            assert_eq!(expand_tilde("~-/file.txt"), "/previous/directory/file.txt");
        });
    }

    #[test]
    #[serial]
    fn test_missing_pwd() {
        with_pwd(None, || {
            assert_eq!(expand_tilde("~+"), "~+");
        });
    }

    #[test]
    #[serial]
    fn test_missing_oldpwd() {
        with_oldpwd(None, || {
            assert_eq!(expand_tilde("~-"), "~-");
        });
    }
}
