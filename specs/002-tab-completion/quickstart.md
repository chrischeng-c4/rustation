# Developer Quickstart: Tab Completion

**Feature**: 002-tab-completion
**Date**: 2025-11-16
**Purpose**: Guide for developers implementing and testing tab completion

---

## Overview

This guide covers how to implement, test, and extend the tab completion system in rush. The completion system is built on reedline's `Completer` trait with three independent completers (command, path, flag) coordinated by a registry.

---

## Quick Start

### 1. Build and Test

```bash
# From workspace root
cd /Users/chrischeng/projects/rust-station

# Build rush with completion support
cargo build -p rush --release

# Run unit tests for completion module
cargo test -p rush completion

# Run integration tests
cargo test -p rush -- --test-threads=1 integration::completion

# Run rush with completion enabled
./target/release/rush
```

### 2. Try It Out

```bash
# In rush shell:
$ gi<TAB>           # Should complete to "git"
$ ls src/re<TAB>    # Should complete to "src/repl/"
$ git --ver<TAB>    # Should complete to "git --version"
```

---

## Project Structure

```
crates/rush/src/
├── completion/              # NEW: Completion module
│   ├── mod.rs              # Module exports + CompletionRegistry
│   ├── command.rs          # P1: CommandCompleter
│   ├── path.rs             # P2: PathCompleter
│   ├── flag.rs             # P3: FlagCompleter
│   └── registry.rs         # Routing logic
├── repl/mod.rs             # MODIFIED: Integrate completer
└── main.rs                 # Entry point (minimal changes)

tests/
├── unit/completion/        # Unit tests
│   ├── command_tests.rs
│   ├── path_tests.rs
│   └── flag_tests.rs
└── integration/
    └── completion_tests.rs  # End-to-end tests
```

---

## Implementation Phases

### Phase 1: P1 - Command Completion (Required for v0.2)

**Goal**: Complete command names from PATH executables

**Files to Create**:
1. `src/completion/mod.rs`
2. `src/completion/command.rs`
3. `tests/unit/completion/command_tests.rs`

**Implementation Steps**:

1. **Create Module Structure**:

```rust
// src/completion/mod.rs
mod command;

pub use command::CommandCompleter;

use reedline::{Completer, Suggestion};

// Re-export reedline types for convenience
pub use reedline::{Completer, Suggestion, Span};
```

2. **Implement CommandCompleter**:

```rust
// src/completion/command.rs
use std::collections::HashSet;
use std::env;
use std::fs;
use reedline::{Completer, Suggestion, Span};

pub struct CommandCompleter {
    cache: Option<HashSet<String>>,
    #[cfg(target_os = "macos")]
    case_sensitive: bool,
}

impl CommandCompleter {
    pub fn new() -> Self {
        Self {
            cache: None,
            #[cfg(target_os = "macos")]
            case_sensitive: false,
            #[cfg(not(target_os = "macos"))]
            case_sensitive: true,
        }
    }

    fn ensure_cache_loaded(&mut self) {
        if self.cache.is_none() {
            self.cache = Some(self.scan_path());
        }
    }

    fn scan_path(&self) -> HashSet<String> {
        // TODO: Implement PATH scanning
        // See data-model.md for algorithm
        HashSet::new()
    }
}

impl Completer for CommandCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // TODO: Implement completion logic
        // See contracts/completer-trait.md for requirements
        vec![]
    }
}
```

3. **Write Tests**:

```rust
// tests/unit/completion/command_tests.rs
use rush::completion::CommandCompleter;
use reedline::Completer;

#[test]
fn test_single_match() {
    let mut completer = CommandCompleter::new();
    let suggestions = completer.complete("gi", 2);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].value, "git");
}

#[test]
fn test_multiple_matches() {
    let mut completer = CommandCompleter::new();
    let suggestions = completer.complete("ca", 2);

    assert!(suggestions.len() > 1);
    assert!(suggestions.iter().any(|s| s.value == "cat"));
    assert!(suggestions.iter().any(|s| s.value == "cargo"));
}

#[test]
fn test_no_matches() {
    let mut completer = CommandCompleter::new();
    let suggestions = completer.complete("zzz", 3);

    assert_eq!(suggestions.len(), 0);
}
```

4. **Integrate with REPL**:

```rust
// src/repl/mod.rs
use crate::completion::CommandCompleter;

impl Repl {
    pub fn with_config(config: Config) -> Result<Self> {
        let completer = Box::new(CommandCompleter::new());

        let line_editor = Reedline::create()
            .with_history(history)
            .with_highlighter(highlighter)
            .with_completer(completer);  // <-- Add here

        Ok(Self { line_editor, config })
    }
}
```

### Phase 2: P2 - Path Completion (Optional for v0.2)

**Goal**: Complete file and directory paths

**Files to Create**:
1. `src/completion/path.rs`
2. `tests/unit/completion/path_tests.rs`

**Implementation Steps**:

1. **Implement PathCompleter**:

```rust
// src/completion/path.rs
use std::fs;
use std::path::{Path, PathBuf};
use reedline::{Completer, Suggestion, Span};

pub struct PathCompleter {
    #[cfg(target_os = "macos")]
    case_sensitive: bool,
}

impl PathCompleter {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            case_sensitive: false,
            #[cfg(not(target_os = "macos"))]
            case_sensitive: true,
        }
    }

    fn list_directory(&self, dir: &Path) -> Vec<String> {
        // TODO: List directory entries
        // See data-model.md for algorithm
        vec![]
    }
}

impl Completer for PathCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // TODO: Implement path completion
        // See contracts/completer-trait.md for requirements
        vec![]
    }
}
```

2. **Upgrade REPL to CompletionRegistry**:

```rust
// src/completion/registry.rs
use reedline::{Completer, Suggestion};
use super::{CommandCompleter, PathCompleter};

pub struct CompletionRegistry {
    command_completer: CommandCompleter,
    path_completer: PathCompleter,
}

impl CompletionRegistry {
    pub fn new() -> Self {
        Self {
            command_completer: CommandCompleter::new(),
            path_completer: PathCompleter::new(),
        }
    }

    fn is_first_word(&self, line: &str, pos: usize) -> bool {
        // TODO: Parse line to determine if completing first word
        false
    }
}

impl Completer for CompletionRegistry {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        if self.is_first_word(line, pos) {
            self.command_completer.complete(line, pos)
        } else {
            self.path_completer.complete(line, pos)
        }
    }
}
```

### Phase 3: P3 - Flag Completion (Optional for v0.2)

**Goal**: Complete flags for common commands

**Files to Create**:
1. `src/completion/flag.rs`
2. `tests/unit/completion/flag_tests.rs`

**Implementation Steps**:

1. **Create Flag Registry**:

```rust
// src/completion/flag.rs
use std::collections::HashMap;
use lazy_static::lazy_static;
use reedline::{Completer, Suggestion, Span};

pub struct FlagDefinition {
    pub flag: &'static str,
    pub short: Option<&'static str>,
    pub description: &'static str,
}

lazy_static! {
    static ref FLAG_REGISTRY: HashMap<&'static str, Vec<FlagDefinition>> = {
        let mut registry = HashMap::new();

        // Git flags
        registry.insert("git", vec![
            FlagDefinition {
                flag: "--version",
                short: Some("-v"),
                description: "Show version",
            },
            FlagDefinition {
                flag: "--help",
                short: Some("-h"),
                description: "Show help",
            },
            // TODO: Add more git flags
        ]);

        // Cargo flags
        registry.insert("cargo", vec![
            FlagDefinition {
                flag: "--version",
                short: Some("-V"),
                description: "Show version",
            },
            // TODO: Add more cargo flags
        ]);

        // ls flags
        registry.insert("ls", vec![
            FlagDefinition {
                flag: "-l",
                short: None,
                description: "Long format",
            },
            FlagDefinition {
                flag: "-a",
                short: None,
                description: "Show hidden files",
            },
            // TODO: Add more ls flags
        ]);

        registry
    };
}

pub struct FlagCompleter;

impl FlagCompleter {
    pub fn new() -> Self {
        Self
    }
}

impl Completer for FlagCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // TODO: Implement flag completion
        // Extract command name and partial flag
        // Look up in FLAG_REGISTRY
        // Return matching suggestions with descriptions
        vec![]
    }
}
```

---

## Testing Guide

### Unit Testing

**Test Each Completer Independently**:

```rust
// Example: Testing CommandCompleter
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_completion_single_match() {
        let mut completer = CommandCompleter::new();
        let result = completer.complete("git", 3);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].value, "git");
        assert!(result[0].append_whitespace);
    }

    #[test]
    fn test_command_completion_case_insensitive_macos() {
        #[cfg(target_os = "macos")]
        {
            let mut completer = CommandCompleter::new();
            let result = completer.complete("GIT", 3);

            assert!(result.iter().any(|s| s.value == "git"));
        }
    }

    #[test]
    fn test_command_completion_too_many_matches() {
        let mut completer = CommandCompleter::new();
        let result = completer.complete("c", 1);

        // Should return single "too many" message
        assert_eq!(result.len(), 1);
        assert!(result[0].value.contains("matches"));
    }
}
```

### Integration Testing

**Test Complete REPL Flow**:

```rust
// tests/integration/completion_tests.rs
use rush::Repl;
use reedline::Signal;

#[test]
fn test_tab_completion_in_repl() {
    let config = Config::default();
    let mut repl = Repl::with_config(config).unwrap();

    // Simulate user typing "git" and pressing Tab
    // Note: This requires mocking or test harness
    // See reedline examples for testing approach
}
```

### Manual Testing

**Interactive Testing Checklist**:

```bash
# Build and run
cargo build -p rush --release
./target/release/rush

# Command completion
$ gi<TAB>           # Should complete to "git"
$ ca<TAB>           # Should show menu: cat, cal, cargo
$ cargo<TAB>        # Should complete with space after
$ zzz<TAB>          # Should do nothing (no matches)

# Path completion (P2)
$ ls src/re<TAB>    # Should complete to "src/repl/"
$ cat README<TAB>   # Should complete to "README.md"
$ cd ~/<TAB>        # Should show home directory contents
$ ls .<TAB>         # Should show hidden files

# Flag completion (P3)
$ git --<TAB>       # Should show git flags with descriptions
$ ls -<TAB>         # Should show ls flags
$ cargo b<TAB>      # Should complete to "cargo build"
```

---

## Debugging

### Enable Verbose Logging

```bash
# Run rush with trace logging
cargo run -p rush -- -vv

# Check logs
tail -f ~/Library/Application\ Support/rush/rush-v0.1.0.log
```

### Common Issues

**Issue: Completions not appearing**
- Check: Is completer registered with reedline?
- Check: Is `complete()` being called? (add tracing)
- Check: Are suggestions being returned? (log Vec length)

**Issue: Wrong completions**
- Check: CompletionRegistry routing logic
- Check: Is completion type detected correctly?
- Check: Test completer directly (unit tests)

**Issue: Slow completions**
- Check: PATH scan time (log duration)
- Check: Directory size (log entry count)
- Check: Cache is being used (log cache hits)

**Issue: Completion breaks REPL**
- Check: Completer doesn't panic
- Check: Suggestions have valid spans
- Check: No stdout/stderr output from completer

---

## Performance Benchmarking

### Measure Completion Latency

```rust
use std::time::Instant;

#[test]
fn bench_command_completion() {
    let mut completer = CommandCompleter::new();

    // First completion (includes PATH scan)
    let start = Instant::now();
    let _ = completer.complete("git", 3);
    let first_duration = start.elapsed();

    println!("First completion: {:?}", first_duration);
    assert!(first_duration.as_millis() < 100, "First completion too slow");

    // Subsequent completion (cached)
    let start = Instant::now();
    let _ = completer.complete("cargo", 5);
    let cached_duration = start.elapsed();

    println!("Cached completion: {:?}", cached_duration);
    assert!(cached_duration.as_millis() < 10, "Cached completion too slow");
}
```

---

## Extending Completers

### Adding New Flag Definitions

**Edit `src/completion/flag.rs`**:

```rust
lazy_static! {
    static ref FLAG_REGISTRY: HashMap<&'static str, Vec<FlagDefinition>> = {
        let mut registry = HashMap::new();

        // Add new command flags
        registry.insert("npm", vec![
            FlagDefinition {
                flag: "--version",
                short: Some("-v"),
                description: "Show version",
            },
            FlagDefinition {
                flag: "--help",
                short: Some("-h"),
                description: "Show help",
            },
            FlagDefinition {
                flag: "--global",
                short: Some("-g"),
                description: "Install globally",
            },
        ]);

        registry
    };
}
```

### Adding New Completer Type

**Example: Adding git branch completer**:

1. Create `src/completion/git_branch.rs`:

```rust
pub struct GitBranchCompleter;

impl Completer for GitBranchCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // Parse: "git checkout <partial-branch>"
        // Run: git branch --list
        // Return matching branches
        vec![]
    }
}
```

2. Update `CompletionRegistry` to route git checkout commands to GitBranchCompleter

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Test Completion

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
      - name: Run completion tests
        run: cargo test -p rush completion
      - name: Run integration tests
        run: cargo test -p rush --test completion_tests
```

---

## Resources

**Documentation**:
- [spec.md](./spec.md) - Feature requirements
- [plan.md](./plan.md) - Technical approach
- [research.md](./research.md) - Technical decisions
- [data-model.md](./data-model.md) - Data structures
- [contracts/completer-trait.md](./contracts/completer-trait.md) - Interface contracts

**reedline Documentation**:
- [reedline repo](https://github.com/nushell/reedline)
- [Completer trait](https://docs.rs/reedline/latest/reedline/trait.Completer.html)
- [reedline examples](https://github.com/nushell/reedline/tree/main/examples)

**Constitution Principles**:
- Performance-First: <100ms completion
- Zero-Config: Works immediately
- Progressive Complexity: P1/P2/P3 tiers
- Rust-Native: Pure Rust, reedline ecosystem

---

## Checklist for Implementation

### P1: Command Completion

- [ ] Create `src/completion/mod.rs`
- [ ] Create `src/completion/command.rs`
- [ ] Implement PATH scanning
- [ ] Implement lazy cache loading
- [ ] Implement prefix matching (case-aware)
- [ ] Handle >50 matches
- [ ] Write unit tests (single, multiple, none, too many)
- [ ] Integrate with REPL
- [ ] Manual testing
- [ ] Performance benchmarking

### P2: Path Completion

- [ ] Create `src/completion/path.rs`
- [ ] Implement directory listing
- [ ] Handle relative paths
- [ ] Handle absolute paths
- [ ] Handle tilde expansion
- [ ] Handle hidden files
- [ ] Handle paths with spaces (quoting)
- [ ] Create `CompletionRegistry`
- [ ] Implement routing logic
- [ ] Write unit tests
- [ ] Manual testing

### P3: Flag Completion

- [ ] Create `src/completion/flag.rs`
- [ ] Define flag registry (lazy_static)
- [ ] Add git flags
- [ ] Add cargo flags
- [ ] Add ls flags
- [ ] Implement flag matching
- [ ] Include descriptions
- [ ] Update `CompletionRegistry` routing
- [ ] Write unit tests
- [ ] Manual testing

---

**Ready to implement! Start with P1 (Command Completion) and iterate from there.**
