# Data Model: Tab Completion

**Feature**: 002-tab-completion
**Date**: 2025-11-16
**Purpose**: Define data structures, entities, and state management for completion system

---

## Overview

The tab completion system consists of three independent completers (P1: CommandCompleter, P2: PathCompleter, P3: FlagCompleter) coordinated by a CompletionRegistry. Each completer implements reedline's `Completer` trait and manages its own caching strategy.

---

## Core Entities

### 1. Completer Trait (from reedline)

**Purpose**: Interface that all completers must implement

**Definition**:
```rust
pub trait Completer: Send {
    /// Generate completions for the given line at the specified position
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion>;
}
```

**Contract**:
- `line`: Full input line buffer
- `pos`: Cursor position within line
- Returns: Vector of suggestions (empty if no matches)
- Must be thread-safe (`Send`)

---

### 2. Suggestion (from reedline)

**Purpose**: Represents a single completion suggestion

**Definition**:
```rust
pub struct Suggestion {
    pub value: String,              // Replacement text
    pub description: Option<String>, // Help text (shown in menu)
    pub extra: Option<Vec<String>>,  // Additional metadata
    pub span: Span,                  // What part of line to replace
    pub append_whitespace: bool,     // Add space after completion
}
```

**Usage**:
- `value`: Completed command/path/flag
- `description`: Used for flag completions (e.g., "--help: Show help message")
- `span`: Defines replacement range in input buffer
- `append_whitespace`: true for commands/flags, false for incomplete paths

---

### 3. CompletionContext

**Purpose**: Parse input line to determine what kind of completion is needed

**Definition**:
```rust
pub struct CompletionContext {
    pub completion_type: CompletionType,
    pub partial_input: String,
    pub cursor_position: usize,
}

pub enum CompletionType {
    Command,        // Completing first word (command name)
    Path,          // Completing path argument
    Flag,          // Completing flag (starts with -)
    None,          // No completion applicable
}
```

**Parsing Logic**:
```rust
impl CompletionContext {
    pub fn from_line(line: &str, pos: usize) -> Self {
        let tokens = tokenize_for_completion(line, pos);

        if tokens.is_empty() || pos <= first_word_end(line) {
            CompletionType::Command
        } else if current_token_starts_with_dash(&tokens, pos) {
            CompletionType::Flag
        } else {
            CompletionType::Path
        }
    }
}
```

**State**:
- No persistent state
- Created fresh for each completion request
- Determines which completer to invoke

---

## P1: CommandCompleter

### Purpose
Complete command names from executables in PATH

### Data Structure

```rust
pub struct CommandCompleter {
    cache: Option<HashSet<String>>,  // Lazy-loaded executable names
    case_sensitive: bool,             // Platform-dependent
}
```

### State Management

**Initialization**:
```rust
impl CommandCompleter {
    pub fn new() -> Self {
        Self {
            cache: None,  // Not loaded yet
            #[cfg(target_os = "macos")]
            case_sensitive: false,
            #[cfg(not(target_os = "macos"))]
            case_sensitive: true,
        }
    }
}
```

**Cache Loading** (Lazy):
```rust
fn ensure_cache_loaded(&mut self) {
    if self.cache.is_none() {
        self.cache = Some(self.scan_path());
    }
}

fn scan_path(&self) -> HashSet<String> {
    let path = env::var("PATH").unwrap_or_default();
    let mut executables = HashSet::new();

    for dir in path.split(':') {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if is_executable(&entry) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    executables.insert(name);
                }
            }
        }
    }

    executables
}
```

### Completion Algorithm

```rust
impl Completer for CommandCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        self.ensure_cache_loaded();

        let input = extract_partial_command(line, pos);
        let matches = self.cache.as_ref()
            .unwrap()
            .iter()
            .filter(|cmd| matches_prefix(cmd, &input, self.case_sensitive))
            .take(51)  // Limit to 51 (check if >50)
            .collect::<Vec<_>>();

        if matches.len() > 50 {
            return vec![create_too_many_matches_suggestion(matches.len())];
        }

        matches.into_iter()
            .map(|cmd| Suggestion {
                value: cmd.clone(),
                description: None,
                extra: None,
                span: calculate_span(line, pos),
                append_whitespace: true,
            })
            .collect()
    }
}
```

### Cache Lifecycle

- **Creation**: Lazy (first Tab press)
- **Updates**: Never (valid for entire session)
- **Invalidation**: Never in v0.1 (PATH changes rare)
- **Memory**: ~100-200KB (1000 commands × 100 bytes avg)

---

## P2: PathCompleter

### Purpose
Complete file and directory paths

### Data Structure

```rust
pub struct PathCompleter {
    case_sensitive: bool,  // Platform-dependent
    show_hidden: bool,     // Dynamic (based on input)
}
```

### State Management

**No Persistent Cache**:
- Directories change frequently
- Always read fresh from filesystem
- No memory overhead
- Slight performance cost (10-50ms per completion)

**Initialization**:
```rust
impl PathCompleter {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            case_sensitive: false,
            #[cfg(not(target_os = "macos"))]
            case_sensitive: true,
            show_hidden: false,  // Determined dynamically
        }
    }
}
```

### Completion Algorithm

```rust
impl Completer for PathCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let partial_path = extract_partial_path(line, pos);
        let (parent_dir, prefix) = split_path_and_prefix(&partial_path);

        // Show hidden files if user typed '.'
        let show_hidden = prefix.starts_with('.');

        let matches = list_directory_entries(&parent_dir)
            .into_iter()
            .filter(|entry| {
                matches_prefix(&entry.name, &prefix, self.case_sensitive)
                    && (show_hidden || !entry.name.starts_with('.'))
            })
            .take(51)
            .collect::<Vec<_>>();

        if matches.len() > 50 {
            return vec![create_too_many_matches_suggestion(matches.len())];
        }

        matches.into_iter()
            .map(|entry| {
                let mut value = entry.name;
                if entry.is_dir {
                    value.push('/');
                }

                // Quote if contains spaces
                if value.contains(' ') {
                    value = format!("\"{}\"", value);
                }

                Suggestion {
                    value,
                    description: None,
                    extra: None,
                    span: calculate_span(line, pos),
                    append_whitespace: !entry.is_dir,  // No space for dirs
                }
            })
            .collect()
    }
}
```

### Path Parsing

**Extract Partial Path**:
```rust
fn extract_partial_path(line: &str, pos: usize) -> String {
    // Find token at cursor position
    // Handle quoted paths: "My Doc" vs My Doc
    // Handle escaped spaces: My\ Documents
    // Return partial path including directory separator context
}
```

**Split into Parent and Prefix**:
```rust
fn split_path_and_prefix(path: &str) -> (PathBuf, String) {
    // "src/re" → (PathBuf("src"), "re")
    // "/etc/hos" → (PathBuf("/etc"), "hos")
    // "~/proj" → (PathBuf(home_dir()), "proj")
    // "README" → (PathBuf("."), "README")
}
```

---

## P3: FlagCompleter

### Purpose
Complete flags for known commands

### Data Structure

```rust
pub struct FlagCompleter {
    flag_registry: &'static FlagRegistry,
}

pub struct FlagRegistry {
    commands: HashMap<&'static str, Vec<FlagDefinition>>,
}

pub struct FlagDefinition {
    pub flag: &'static str,           // e.g., "--help"
    pub short: Option<&'static str>,  // e.g., "-h"
    pub description: &'static str,     // e.g., "Show help message"
}
```

### Static Flag Data

**Compile-Time Registry**:
```rust
use lazy_static::lazy_static;

lazy_static! {
    static ref FLAG_REGISTRY: FlagRegistry = {
        let mut registry = FlagRegistry::new();

        // Git flags
        registry.add("git", vec![
            FlagDefinition { flag: "--version", short: Some("-v"), description: "Show version" },
            FlagDefinition { flag: "--help", short: Some("-h"), description: "Show help" },
            // ... more git flags
        ]);

        // Cargo flags
        registry.add("cargo", vec![
            FlagDefinition { flag: "--version", short: Some("-V"), description: "Show version" },
            FlagDefinition { flag: "--verbose", short: Some("-v"), description: "Verbose output" },
            // ... more cargo flags
        ]);

        // ls flags
        registry.add("ls", vec![
            FlagDefinition { flag: "-l", short: None, description: "Long format" },
            FlagDefinition { flag: "-a", short: None, description: "Show hidden" },
            FlagDefinition { flag: "-h", short: None, description: "Human readable" },
            // ... more ls flags
        ]);

        registry
    };
}
```

### Completion Algorithm

```rust
impl Completer for FlagCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let (command, partial_flag) = extract_command_and_flag(line, pos);

        let flags = match self.flag_registry.get_flags(&command) {
            Some(flags) => flags,
            None => return vec![],  // Unknown command
        };

        flags.iter()
            .filter(|flag| {
                flag.flag.starts_with(&partial_flag) ||
                flag.short.map_or(false, |s| s.starts_with(&partial_flag))
            })
            .take(50)
            .map(|flag| Suggestion {
                value: flag.flag.to_string(),
                description: Some(flag.description.to_string()),
                extra: flag.short.map(|s| vec![s.to_string()]),
                span: calculate_span(line, pos),
                append_whitespace: true,
            })
            .collect()
    }
}
```

### Memory

- Static data (~10-20KB)
- Loaded once at compile time
- Zero runtime overhead
- No cache invalidation needed

---

## CompletionRegistry (Coordinator)

### Purpose
Route completion requests to appropriate completer

### Data Structure

```rust
pub struct CompletionRegistry {
    command_completer: CommandCompleter,
    path_completer: PathCompleter,
    flag_completer: FlagCompleter,
}
```

### Routing Logic

```rust
impl Completer for CompletionRegistry {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let context = CompletionContext::from_line(line, pos);

        match context.completion_type {
            CompletionType::Command => {
                self.command_completer.complete(line, pos)
            }
            CompletionType::Path => {
                self.path_completer.complete(line, pos)
            }
            CompletionType::Flag => {
                self.flag_completer.complete(line, pos)
            }
            CompletionType::None => {
                vec![]
            }
        }
    }
}
```

---

## Integration with REPL

### REPL Modification

```rust
// In src/repl/mod.rs

use crate::completion::CompletionRegistry;

impl Repl {
    pub fn with_config(config: Config) -> Result<Self> {
        let completer = Box::new(CompletionRegistry::new());

        let line_editor = Reedline::create()
            .with_history(history)
            .with_highlighter(highlighter)
            .with_completer(completer);  // <-- Add completer

        Ok(Self { line_editor, config })
    }
}
```

---

## Memory Budget

| Component | Memory Usage | Justification |
|-----------|-------------|---------------|
| CommandCompleter cache | 100-200KB | 1000 commands × ~150 bytes avg |
| PathCompleter | 0KB | No persistent cache |
| FlagCompleter static data | 10-20KB | ~200 flags × ~100 bytes |
| CompletionRegistry | <1KB | Just struct overhead |
| **Total** | **~200KB** | Well within <10MB limit |

---

## Performance Characteristics

| Operation | First Time | Subsequent | Target | Status |
|-----------|-----------|------------|--------|--------|
| Command completion | 50-100ms | <10ms | <100ms | ✅ Meets |
| Path completion | 10-50ms | 10-50ms | <100ms | ✅ Meets |
| Flag completion | <5ms | <5ms | <100ms | ✅ Meets |

---

## State Lifecycle

```
Session Start
    ↓
Tab Pressed → CompletionRegistry.complete()
    ↓
CompletionContext parsed
    ↓
Route to appropriate completer
    ↓
CommandCompleter:
    - cache = None → scan PATH (100ms)
    - cache loaded → lookup (10ms)
PathCompleter:
    - Always scan directory (10-50ms)
FlagCompleter:
    - Lookup static data (<5ms)
    ↓
Return Vec<Suggestion>
    ↓
Reedline displays menu
    ↓
User selects → line updated
    ↓
Session End (cache discarded)
```

---

## Summary

**Key Entities**:
1. `CompletionRegistry` - Routes to appropriate completer
2. `CommandCompleter` - Lazy-loaded PATH cache
3. `PathCompleter` - Stateless, always fresh
4. `FlagCompleter` - Static compile-time data
5. `CompletionContext` - Parsing/routing logic

**Design Principles**:
- Lazy loading (Performance-First)
- Minimal memory footprint (<200KB)
- No persistent storage (Zero-Config)
- Progressive: P1/P2/P3 independent (Progressive Complexity)
- Pure Rust (Rust-Native)

All entities documented. Ready for contracts definition.
