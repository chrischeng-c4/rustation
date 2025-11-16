# Contract: Completer Trait Interface

**Feature**: 002-tab-completion
**Date**: 2025-11-16
**Purpose**: Define interface contracts for completion system

---

## Overview

This document defines the contracts that all completion components must adhere to. These contracts ensure consistent behavior across command, path, and flag completion while integrating seamlessly with reedline's completion system.

---

## 1. reedline::Completer Trait Contract

### Interface

```rust
pub trait Completer: Send {
    /// Generate completions for the given line at the specified cursor position
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion>;
}
```

### Contract Requirements

**Inputs**:
- `line: &str` - Full input buffer (entire command line)
- `pos: usize` - Zero-indexed cursor position within `line`

**Outputs**:
- `Vec<Suggestion>` - Completion suggestions (may be empty)

**Behavioral Contracts**:

1. **Empty Input**: `complete("", 0)` → returns empty vec or all possibilities (command completer shows all commands)

2. **No Matches**: If no completions match, return empty `Vec<Suggestion>`

3. **Single Match**: Return vec with one `Suggestion`

4. **Multiple Matches**: Return vec with multiple `Suggestion` items (up to 51)

5. **Too Many Matches**: If >50 matches, return vec with single suggestion explaining count

6. **Thread Safety**: Must be `Send` (safe to move across threads)

7. **Mutability**: `&mut self` allows for internal caching/state updates

8. **Performance**: Should complete within 100ms for typical scenarios

**Error Handling**:
- Must NOT panic on invalid input
- Must handle cursor position beyond line length gracefully
- Must handle Unicode/multi-byte characters correctly
- Must handle permission errors (e.g., unreadable directories) silently

---

## 2. Suggestion Structure Contract

### Interface

```rust
pub struct Suggestion {
    pub value: String,
    pub description: Option<String>,
    pub extra: Option<Vec<String>>,
    pub span: Span,
    pub append_whitespace: bool,
}

pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

### Contract Requirements

**`value` Field**:
- MUST contain the replacement text (completed command/path/flag)
- MUST be properly escaped/quoted if contains special characters
- Examples:
  - Command: `"git"`
  - Path with spaces: `"\"My Documents/\""`
  - Flag: `"--verbose"`

**`description` Field**:
- MUST be `None` for commands and paths (no help text)
- SHOULD be `Some(...)` for flags (e.g., `"Show verbose output"`)
- MUST be concise (max 50 characters recommended)

**`extra` Field**:
- MAY contain additional metadata (e.g., short flag alternatives)
- Example: For `--verbose`, extra might be `Some(vec!["-v"])`
- MUST be `None` if no extra data

**`span` Field**:
- MUST define the range in `line` to replace
- `span.start` MUST be ≤ `pos`
- `span.end` MUST be ≥ `pos`
- Example: For `"git st"` with cursor at position 6, span might be `{start: 4, end: 6}` (replacing "st")

**`append_whitespace` Field**:
- MUST be `true` for completed commands and flags (ready for next argument)
- MUST be `false` for directories (user may want to continue path)
- MUST be `true` for files (typically end of argument)

**Examples**:

```rust
// Command completion
Suggestion {
    value: "git".to_string(),
    description: None,
    extra: None,
    span: Span { start: 0, end: 2 },  // Replacing "gi"
    append_whitespace: true,
}

// Path completion (directory)
Suggestion {
    value: "src/".to_string(),
    description: None,
    extra: None,
    span: Span { start: 3, end: 5 },  // Replacing "sr" in "ls sr"
    append_whitespace: false,  // Don't add space (partial path)
}

// Flag completion
Suggestion {
    value: "--verbose".to_string(),
    description: Some("Show detailed output".to_string()),
    extra: Some(vec!["-v".to_string()]),
    span: Span { start: 4, end: 6 },  // Replacing "--v" in "git --v"
    append_whitespace: true,
}

// Too many matches
Suggestion {
    value: format!("{} matches found, type more characters", count),
    description: None,
    extra: None,
    span: Span { start: pos, end: pos },  // Don't replace anything
    append_whitespace: false,
}
```

---

## 3. CommandCompleter Contract

### Responsibilities

MUST provide command name completions from executables in PATH

### Interface

```rust
impl Completer for CommandCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion>;
}
```

### Behavioral Contract

**When to Activate**:
- MUST activate when completing the first word in the command line
- MUST NOT activate for arguments after the first word

**Completion Logic**:
- MUST scan PATH environment variable for executables
- MUST cache results for session (lazy load on first use)
- MUST perform prefix matching (case-sensitive on Linux, case-insensitive on macOS)
- MUST return at most 51 suggestions
- MUST return "too many matches" suggestion if >50 matches

**Performance**:
- First completion: MUST complete within 100ms (including PATH scan)
- Subsequent completions: SHOULD complete within 10ms (cached lookup)

**Examples**:

```rust
// Single match
complete("gi", 2) → vec![Suggestion { value: "git", ... }]

// Multiple matches
complete("ca", 2) → vec![
    Suggestion { value: "cat", ... },
    Suggestion { value: "cal", ... },
    Suggestion { value: "cargo", ... },
]

// No matches
complete("zzz", 3) → vec![]

// Too many matches
complete("c", 1) → vec![
    Suggestion { value: "50+ matches, type more characters", ... }
]
```

---

## 4. PathCompleter Contract

### Responsibilities

MUST provide file and directory path completions

### Interface

```rust
impl Completer for PathCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion>;
}
```

### Behavioral Contract

**When to Activate**:
- MUST activate when completing arguments after the first word
- MUST detect partial paths (relative, absolute, tilde-expanded)

**Completion Logic**:
- MUST list directory entries matching prefix
- MUST handle relative paths (e.g., `src/re`)
- MUST handle absolute paths (e.g., `/etc/hos`)
- MUST handle tilde expansion (e.g., `~/pro`)
- MUST show hidden files (`.foo`) ONLY if input starts with `.`
- MUST append `/` to completed directories
- MUST quote paths containing spaces
- MUST respect platform case sensitivity (macOS: insensitive, Linux: sensitive)
- MUST return at most 51 suggestions
- MUST NOT cache (always read fresh from filesystem)

**Performance**:
- MUST complete within 50ms for typical directories (<1000 entries)
- MAY be slower for large directories (acceptable trade-off for freshness)

**Examples**:

```rust
// Directory completion
complete("ls src/re", 9) → vec![
    Suggestion { value: "src/repl/", append_whitespace: false, ... }
]

// File completion
complete("cat READ", 8) → vec![
    Suggestion { value: "README.md", append_whitespace: true, ... }
]

// Hidden file (user typed '.')
complete("ls .git", 7) → vec![
    Suggestion { value: ".github/", append_whitespace: false, ... },
    Suggestion { value: ".gitignore", append_whitespace: true, ... },
]

// Path with spaces
complete("cd My Do", 7) → vec![
    Suggestion { value: "\"My Documents/\"", append_whitespace: false, ... }
]

// No matches
complete("ls nonexistent", 14) → vec![]
```

---

## 5. FlagCompleter Contract

### Responsibilities

MUST provide flag/option completions for known commands

### Interface

```rust
impl Completer for FlagCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion>;
}
```

### Behavioral Contract

**When to Activate**:
- MUST activate when completing tokens starting with `-` or `--`
- MUST extract the command name to determine which flags to suggest

**Completion Logic**:
- MUST maintain static registry of known flags for common commands
- MUST support git, cargo, ls, cd, cat, echo, grep, find (minimum set)
- MUST match both long flags (`--verbose`) and short flags (`-v`)
- MUST include descriptions in `Suggestion.description`
- MUST include short alternative in `Suggestion.extra` (if applicable)
- MUST NOT cache (static data, no cache needed)
- MUST return at most 50 suggestions

**Performance**:
- MUST complete within 5ms (static data lookup)

**Flag Registry Requirements**:
- MUST use compile-time static data (`lazy_static!` or `once_cell`)
- MUST organize by command name
- MUST include at least 20 common flags across supported commands

**Examples**:

```rust
// Long flag completion
complete("git --ver", 9) → vec![
    Suggestion {
        value: "--version",
        description: Some("Show version"),
        extra: None,
        ...
    }
]

// Short flag completion
complete("ls -", 4) → vec![
    Suggestion { value: "-l", description: Some("Long format"), ... },
    Suggestion { value: "-a", description: Some("Show hidden"), ... },
    Suggestion { value: "-h", description: Some("Human readable"), ... },
    ...
]

// Unknown command
complete("unknowncmd --foo", 15) → vec![]  // No flags for unknown commands

// Cargo subcommands (treated as flags)
complete("cargo b", 7) → vec![
    Suggestion { value: "build", description: Some("Compile package"), ... }
]
```

---

## 6. CompletionRegistry Contract

### Responsibilities

MUST route completion requests to appropriate completer based on context

### Interface

```rust
impl Completer for CompletionRegistry {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion>;
}
```

### Behavioral Contract

**Routing Logic**:
- MUST parse `line` to determine completion context
- MUST route to `CommandCompleter` if completing first word
- MUST route to `FlagCompleter` if current token starts with `-`
- MUST route to `PathCompleter` otherwise (default for arguments)
- MUST handle edge cases gracefully (empty line, cursor at start)

**Context Determination**:

```rust
// First word → Command
"git" (pos=3) → CommandCompleter

// After first word, no dash → Path
"ls src/re" (pos=9) → PathCompleter

// Starts with dash → Flag
"git --ver" (pos=9) → FlagCompleter

// Empty → Command
"" (pos=0) → CommandCompleter

// After complete command → Path
"cat " (pos=4) → PathCompleter
```

**Performance**:
- Context parsing MUST take <1ms
- Total completion MUST inherit completer's performance characteristics

---

## 7. Integration Contract with REPL

### REPL Responsibilities

MUST integrate `CompletionRegistry` with reedline

**Integration Point**:

```rust
// In src/repl/mod.rs
let completer = Box::new(CompletionRegistry::new());
let line_editor = Reedline::create()
    .with_completer(completer);
```

**Behavioral Requirements**:
- MUST pass completer to reedline during initialization
- MUST NOT call completer directly (reedline handles Tab key)
- MUST allow reedline to manage menu display and navigation
- MUST NOT interfere with completion state

**User Interaction Flow**:

```
1. User types: "git st"
2. User presses: Tab
3. Reedline calls: completer.complete("git st", 6)
4. CompletionRegistry routes to CommandCompleter
5. CommandCompleter returns: vec![Suggestion { value: "status", ... }]
6. Reedline displays: "git status"
7. User presses: Enter
8. REPL executes: "git status"
```

---

## 8. Error Handling Contracts

### Required Error Handling

**All Completers MUST**:

1. **Never Panic**:
   - MUST handle invalid cursor positions gracefully
   - MUST handle filesystem errors (permission denied, not found)
   - MUST handle malformed input (weird Unicode, control characters)

2. **Return Empty Vec on Errors**:
   - Permission denied reading directory → `vec![]`
   - PATH not set → `vec![]` (command completer)
   - Invalid UTF-8 in path → `vec![]`

3. **Log Errors** (if verbose mode):
   - MAY log to tracing::debug! for diagnostics
   - MUST NOT print to stdout/stderr (breaks REPL)

**Examples**:

```rust
// Permission denied
complete("ls /root/secret", 15) → vec![]  // No panic, just empty

// Broken symlink
complete("ls broken_link", 14) → vec![]  // Skip broken entries

// Invalid Unicode
complete("ls \u{FFFF}", 3) → vec![]  // Handle gracefully
```

---

## 9. Testing Contracts

### Required Test Coverage

**Unit Tests** (per completer):
- MUST test single match scenario
- MUST test multiple matches scenario
- MUST test no matches scenario
- MUST test too many matches scenario (>50)
- MUST test edge cases (empty input, cursor at end, Unicode)

**Integration Tests** (REPL):
- MUST test Tab completion in actual REPL
- MUST verify menu display
- MUST verify completion insertion
- MUST verify navigation (Tab, arrows)

**Performance Tests**:
- SHOULD measure completion latency
- SHOULD verify <100ms target

---

## 10. Future Extension Contracts

### Extensibility Points

**Adding New Completers**:
- New completers MUST implement `Completer` trait
- New completers MUST be added to `CompletionRegistry` routing logic
- New completers MUST document their behavioral contracts

**Adding New Flags**:
- MUST add to `FlagRegistry` static data
- MUST include `flag`, `description`, and optional `short` alternative
- MUST follow existing format

**Example**:

```rust
// Adding npm flags
registry.add("npm", vec![
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
]);
```

---

## Summary

**Key Contracts**:
1. All completers implement `Completer` trait
2. All completers return `Vec<Suggestion>` within 100ms
3. All completers handle errors gracefully (never panic)
4. CommandCompleter: lazy caching, case-aware matching
5. PathCompleter: fresh reads, proper quoting, hidden file rules
6. FlagCompleter: static data, include descriptions
7. CompletionRegistry: context-aware routing
8. REPL: integrate with reedline, no manual calls

These contracts ensure consistent, predictable completion behavior aligned with spec requirements and constitution principles.
