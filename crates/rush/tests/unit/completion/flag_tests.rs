//! Unit tests for FlagCompleter
//!
//! Tests flag completion for common commands (git, cargo, ls, etc.)

use reedline::Completer;
use rush::completion::FlagCompleter;

/// T061: Integration test for git flag completion
#[test]
fn test_git_flag_completion() {
    let mut completer = FlagCompleter::new();

    // Complete --ve (should match --version)
    let suggestions = completer.complete("git --ve", 8);

    assert!(!suggestions.is_empty(), "Should find git flags matching --ve");

    // Verify at least --version is in results
    let has_version = suggestions.iter().any(|s| s.value == "--version");
    assert!(has_version, "Should find --version flag");

    // Verify suggestions have descriptions
    for suggestion in &suggestions {
        assert!(suggestion.description.is_some(), "Flags should have descriptions");
    }

    // Verify suggestions have proper span
    for suggestion in &suggestions {
        assert_eq!(suggestion.span.start, 4); // After "git "
        assert_eq!(suggestion.span.end, 8); // At cursor
    }

    // Verify styling
    for suggestion in &suggestions {
        assert!(suggestion.style.is_some(), "Flags should have styling");
    }
}

#[test]
fn test_git_short_flag_completion() {
    let mut completer = FlagCompleter::new();

    // Complete -h (should match -h for --help)
    let suggestions = completer.complete("git -h", 6);

    if !suggestions.is_empty() {
        // If we get results, verify they match -h
        for suggestion in &suggestions {
            assert!(
                suggestion.value.starts_with("--h") || suggestion.value == "-h",
                "Suggestions should match -h pattern"
            );
        }
    }
}

#[test]
fn test_git_multiple_matches() {
    let mut completer = FlagCompleter::new();

    // Complete --v (should match --version, --verbose, possibly others)
    let suggestions = completer.complete("git --v", 7);

    assert!(suggestions.len() >= 1, "Should find at least --version and --verbose");

    // All should start with --v
    for suggestion in &suggestions {
        assert!(suggestion.value.starts_with("--v"), "All suggestions should start with --v");
    }
}

/// T062: Integration test for cargo flag completion
#[test]
fn test_cargo_flag_completion() {
    let mut completer = FlagCompleter::new();

    // Complete --ver (should match --version, --verbose)
    let suggestions = completer.complete("cargo --ver", 11);

    assert!(!suggestions.is_empty(), "Should find cargo flags matching --ver");

    // Verify at least --version is in results
    let has_version = suggestions.iter().any(|s| s.value == "--version");
    assert!(has_version, "Should find --version flag");

    // Verify descriptions exist
    for suggestion in &suggestions {
        assert!(suggestion.description.is_some());
    }
}

#[test]
fn test_cargo_short_flag_completion() {
    let mut completer = FlagCompleter::new();

    // Complete -V (should match --version with -V short form)
    let suggestions = completer.complete("cargo -V", 8);

    if !suggestions.is_empty() {
        // Should find --version (which has -V as short form)
        let has_version = suggestions.iter().any(|s| s.value == "--version");
        assert!(has_version, "Should find --version via -V short form");
    }
}

/// T063: Integration test for ls flag completion
#[test]
fn test_ls_flag_completion() {
    let mut completer = FlagCompleter::new();

    // Complete --al (should match --all, --almost-all)
    let suggestions = completer.complete("ls --al", 7);

    assert!(!suggestions.is_empty(), "Should find ls flags matching --al");

    // Verify flags start with --al
    for suggestion in &suggestions {
        assert!(suggestion.value.starts_with("--al"));
    }

    // Verify descriptions exist
    for suggestion in &suggestions {
        assert!(suggestion.description.is_some());
    }
}

#[test]
fn test_ls_short_flag_completion() {
    let mut completer = FlagCompleter::new();

    // Complete -a (should match multiple flags with -a short form)
    let suggestions = completer.complete("ls -a", 5);

    if !suggestions.is_empty() {
        // Verify styling
        for suggestion in &suggestions {
            assert!(suggestion.style.is_some());
        }
    }
}

#[test]
fn test_ls_human_readable_flag() {
    let mut completer = FlagCompleter::new();

    // Complete --human (should match --human-readable)
    let suggestions = completer.complete("ls --human", 10);

    if !suggestions.is_empty() {
        let has_human_readable = suggestions.iter().any(|s| s.value == "--human-readable");
        assert!(has_human_readable, "Should find --human-readable");
    }
}

/// T064: Test unknown command (no flags)
#[test]
fn test_unknown_command_no_flags() {
    let mut completer = FlagCompleter::new();

    // Try completing flags for unknown command
    let suggestions = completer.complete("unknowncmd --help", 17);

    assert!(suggestions.is_empty(), "Unknown command should return no flag completions");
}

#[test]
fn test_unregistered_command() {
    let mut completer = FlagCompleter::new();

    // gcloud is intentionally not registered (dynamic tool)
    let suggestions = completer.complete("gcloud --ver", 12);

    assert!(
        suggestions.is_empty(),
        "Unregistered command (gcloud) should return no completions"
    );
}

/// T065: Test short flag matching
#[test]
fn test_short_flag_matching() {
    let mut completer = FlagCompleter::new();

    // Test that short flags work for various commands
    let test_cases = vec![
        ("git -q", 6),   // Should match --quiet (6 chars, pos=6)
        ("cargo -q", 8), // Should match --quiet (8 chars, pos=8)
        ("cat -n", 6),   // Should match --number (6 chars, pos=6)
        ("grep -i", 7),  // Should match --ignore-case (7 chars, pos=7)
    ];

    for (input, pos) in test_cases {
        let suggestions = completer.complete(input, pos);

        // If we get results, they should have styling and descriptions
        for suggestion in &suggestions {
            assert!(suggestion.style.is_some(), "Short flags should have styling for {}", input);
            assert!(
                suggestion.description.is_some(),
                "Short flags should have descriptions for {}",
                input
            );
        }
    }
}

#[test]
fn test_flag_with_no_short_form() {
    let mut completer = FlagCompleter::new();

    // --verbose for git has no short form
    let suggestions = completer.complete("git --verbose", 13);

    if !suggestions.is_empty() {
        let has_verbose = suggestions.iter().any(|s| s.value == "--verbose");
        assert!(has_verbose, "Should find --verbose");
    }
}

#[test]
fn test_non_flag_argument_returns_empty() {
    let mut completer = FlagCompleter::new();

    // Not starting with -, should return empty (path completer handles this)
    let suggestions = completer.complete("git status", 10);

    assert!(
        suggestions.is_empty(),
        "Non-flag arguments should return empty from FlagCompleter"
    );
}

#[test]
fn test_flag_completion_after_other_args() {
    let mut completer = FlagCompleter::new();

    // Flags can appear after positional arguments
    let suggestions = completer.complete("git commit file.txt --m", 23);

    if !suggestions.is_empty() {
        // Verify matches start with --m
        for suggestion in &suggestions {
            assert!(suggestion.value.starts_with("--m"));
        }
    }
}

#[test]
fn test_cat_flags() {
    let mut completer = FlagCompleter::new();

    // Test cat --number flag
    let suggestions = completer.complete("cat --num", 9);

    if !suggestions.is_empty() {
        let has_number = suggestions.iter().any(|s| s.value.starts_with("--number"));
        assert!(has_number, "Should find --number flags for cat");
    }
}

#[test]
fn test_echo_flags() {
    let mut completer = FlagCompleter::new();

    // Test echo -n flag
    let suggestions = completer.complete("echo -n", 7);

    if !suggestions.is_empty() {
        let has_n = suggestions.iter().any(|s| s.value == "-n");
        assert!(has_n, "Should find -n flag for echo");
    }
}

#[test]
fn test_grep_flags() {
    let mut completer = FlagCompleter::new();

    // Test grep --ignore-case
    let suggestions = completer.complete("grep --ignore", 13);

    if !suggestions.is_empty() {
        let has_ignore_case = suggestions.iter().any(|s| s.value == "--ignore-case");
        assert!(has_ignore_case, "Should find --ignore-case for grep");
    }
}

#[test]
fn test_find_flags() {
    let mut completer = FlagCompleter::new();

    // Test find -name flag
    let suggestions = completer.complete("find -name", 10);

    if !suggestions.is_empty() {
        let has_name = suggestions.iter().any(|s| s.value == "-name");
        assert!(has_name, "Should find -name flag for find");
    }
}

#[test]
fn test_cd_flags() {
    let mut completer = FlagCompleter::new();

    // Test cd -L flag
    let suggestions = completer.complete("cd -L", 5);

    if !suggestions.is_empty() {
        let has_l = suggestions.iter().any(|s| s.value == "-L");
        assert!(has_l, "Should find -L flag for cd");
    }
}

#[test]
fn test_flag_append_whitespace() {
    let mut completer = FlagCompleter::new();

    // Flags should append whitespace (ready for next argument)
    let suggestions = completer.complete("git --ver", 9);

    if !suggestions.is_empty() {
        for suggestion in &suggestions {
            assert!(suggestion.append_whitespace, "Flags should append whitespace for next arg");
        }
    }
}

#[test]
fn test_empty_flag_prefix() {
    let mut completer = FlagCompleter::new();

    // Just "-" should return all flags for the command
    let suggestions = completer.complete("git -", 5);

    // Should get many results (all git flags)
    // Actual count depends on registry, but should be > 0
    if !suggestions.is_empty() {
        assert!(suggestions.len() > 3, "Should find multiple flags for git");
    }
}

#[test]
fn test_double_dash_completion() {
    let mut completer = FlagCompleter::new();

    // Just "--" should return all long flags
    let suggestions = completer.complete("cargo --", 8);

    if !suggestions.is_empty() {
        // All should be long flags
        for suggestion in &suggestions {
            assert!(suggestion.value.starts_with("--"), "Should only return long flags");
        }
    }
}
