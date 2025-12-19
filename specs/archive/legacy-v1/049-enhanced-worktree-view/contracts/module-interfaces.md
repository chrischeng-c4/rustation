# Module Interface Contracts

**Feature**: 049-enhanced-worktree-view
**Date**: 2025-12-14
**Purpose**: Define public APIs and interfaces between logging module and TUI components

## Contract Overview

This feature introduces a new `logging/` module that integrates with existing TUI components. These contracts define the public APIs, event handling protocols, and integration points.

---

## Contract 1: logging/ Module Public API

**Provider**: `crates/rstn/src/tui/logging/`
**Consumer**: `views/worktree.rs`, `app.rs`

### 1.1 LogEntry API

```rust
/// Public constructor
pub fn new(category: LogCategory, content: String) -> Self

/// Format timestamp as HH:MM:SS
pub fn format_timestamp(&self) -> String

/// Get emoji icon for this entry's category
pub fn category_icon(&self) -> &'static str

/// Public fields (read-only)
pub timestamp: SystemTime
pub category: LogCategory
pub content: String
```

**Guarantees**:
- `content` will never be empty
- `timestamp` will always be valid SystemTime
- `format_timestamp()` returns valid HH:MM:SS string

**Preconditions**:
- None (construction handles all validation)

---

### 1.2 LogCategory API

```rust
/// Get emoji icon for this category
pub fn icon(&self) -> &'static str

/// Get ratatui Color for this category
pub fn color(&self) -> Color

/// Enum variants (exhaustive)
SlashCommand,  // ⚡ Cyan
ClaudeStream,  // 🤖 White
FileChange,    // 📝 Green
ShellOutput,   // 🔧 Yellow
System,        // ℹ️ DarkGray
```

**Guarantees**:
- Each variant has exactly one icon
- Each variant has exactly one color
- Icons are UTF-8 emoji characters
- Colors are valid ratatui::style::Color variants

**Preconditions**:
- None (pure functions)

---

### 1.3 LogBuffer API

```rust
/// Create new empty buffer with 1000-entry capacity
pub fn new() -> Self

/// Add entry (evicts oldest if at capacity)
pub fn push(&mut self, entry: LogEntry)

/// Iterate all entries (oldest to newest)
pub fn entries(&self) -> impl Iterator<Item = &LogEntry>

/// Get current entry count
pub fn len(&self) -> usize

/// Check if buffer is empty
pub fn is_empty(&self) -> bool
```

**Guarantees**:
- `len()` will never exceed 1000
- `push()` always succeeds (never fails)
- `entries()` iterator preserves insertion order
- Oldest entries evicted first when buffer full

**Invariants**:
- `len() <= 1000` at all times
- `entries()` returns items in FIFO order

**Preconditions**:
- None (self-managing capacity)

---

### 1.4 FileChangeTracker API

```rust
/// Create new empty tracker
pub fn new() -> Self

/// Check files for changes, return paths of modified files
pub fn check_files(&mut self, paths: &[PathBuf]) -> Vec<PathBuf>
```

**Guarantees**:
- Returns only files that have changed since last check
- Updates internal state after each check
- Non-existent files are silently skipped

**Preconditions**:
- `paths` should contain absolute paths (relative paths may not work correctly)

**Side Effects**:
- Modifies internal `file_mtimes` map
- Calls `std::fs::metadata()` for each path

---

## Contract 2: Event System Extensions

**Provider**: `tui/event.rs`
**Consumer**: `app.rs`, `views/worktree.rs`

### 2.1 New Event Variants

```rust
/// File changed in spec directory
FileChanged {
    file_path: PathBuf,  // Absolute path to changed file
    file_type: String,   // "spec.md", "plan.md", or "tasks.md"
}

/// Slash command executed
SlashCommandExecuted {
    command: String,  // Full command string (e.g., "/speckit.specify Add feature")
}
```

**Guarantees**:
- `FileChanged.file_path` will always be absolute
- `FileChanged.file_type` will be one of the three spec file names
- `SlashCommandExecuted.command` will start with "/"

**When Emitted**:
- `FileChanged`: When `file_tracker.check_files()` detects modification
- `SlashCommandExecuted`: When user runs a slash command via Commands panel

**Handler Requirements**:
- Must handle these variants in `app.rs::handle_event()`
- Should log entry via `worktree_view.log()`

---

## Contract 3: WorktreeView Integration Points

**Provider**: `views/worktree.rs`
**Consumer**: `app.rs`

### 3.1 New Public Methods

```rust
/// Log an entry with category and content
pub fn log(&mut self, category: LogCategory, content: String)

/// Log a slash command execution
pub fn log_slash_command(&mut self, command: &str)

/// Log a file change event
pub fn log_file_change(&mut self, path: &Path)

/// Log a shell command result
pub fn log_shell_command(&mut self, script: &str, exit_code: i32)
```

**Guarantees**:
- All log methods create timestamped entries
- Entries are immediately visible in next render
- Auto-scroll to bottom happens automatically

**Usage Pattern**:
```rust
// In app.rs
self.worktree_view.log(LogCategory::SlashCommand, "/speckit.plan".to_string());
// or
self.worktree_view.log_slash_command("/speckit.plan");
```

---

### 3.2 Internal State Changes

**New Fields**:
```rust
pub log_buffer: LogBuffer,
pub file_tracker: FileChangeTracker,
pub last_file_check_tick: u64,
```

**Modified Behavior**:
- `tick()`: Now calls `check_file_changes()` every 10 ticks
- `render_output()`: Renders from `log_buffer` instead of `Vec<String>`
- `render_content()`: Includes Tabs widget above content

**Backward Compatibility**:
- `output_scroll` still works for log scrolling
- `clear_output()` still clears log buffer
- 's' key still cycles content types

---

## Contract 4: Rendering Contracts

**Provider**: `views/worktree.rs`
**Consumer**: ratatui rendering system

### 4.1 Tab Rendering Contract

```rust
/// Render tabs at top of Content panel (3 lines)
fn render_content(&self, frame: &mut Frame, area: Rect)
```

**Layout Contract**:
```
┌─────────────────────────────────┐
│ [Spec] [Plan] [Tasks]  ← 3 lines│
├─────────────────────────────────┤
│ Content display area            │
│                                 │
│ (remaining height)              │
└─────────────────────────────────┘
```

**Rendering Rules**:
- Selected tab highlighted with yellow + bold
- Unselected tabs in default white
- Tab bar has border (part of Content panel block)
- Minimum width per tab: 6 characters ("[Tab] ")

---

### 4.2 Log Rendering Contract

```rust
/// Render log entries in Output panel
fn render_output(&self, frame: &mut Frame, area: Rect)
```

**Entry Format**:
```
[HH:MM:SS] 🤖 Claude message here
[HH:MM:SS] ⚡ /speckit.specify Add feature
[HH:MM:SS] 📝 File updated: spec.md
```

**Rendering Rules**:
- Timestamp: 10 characters "[HH:MM:SS] " in dark gray
- Icon: 2 characters "🤖 "
- Content: Remaining width, color-coded by category
- Visible lines: `area.height - 2` (accounting for borders)
- Skip: `output_scroll` lines before displaying
- Take: Up to `visible_height` lines

**Performance Contract**:
- Only format visible lines (lazy evaluation)
- Use iterator `.skip().take()` pattern
- No full buffer formatting on each render

---

## Contract 5: Integration Testing Contracts

**Test Scenarios Must Verify**:

### 5.1 Logging Workflow
```rust
// Given: Empty log buffer
// When: Run slash command
// Then: Log entry appears with correct category, timestamp, icon
```

### 5.2 File Change Detection
```rust
// Given: spec.md open in TUI
// When: Modify spec.md externally and save
// Then: Within 2 seconds, content updates and log entry appears
```

### 5.3 Tab Navigation
```rust
// Given: Viewing Spec tab
// When: Press Right arrow
// Then: Content switches to Plan, Plan tab highlighted
```

### 5.4 Buffer Limit
```rust
// Given: 1000 log entries
// When: Add 100 more entries
// Then: Buffer size stays at 1000, oldest 100 evicted
```

---

## Contract 6: Error Handling Contracts

### 6.1 File Watching Errors

```rust
// When: File doesn't exist
// Then: Skip silently, continue checking other files

// When: fs::metadata() fails
// Then: Skip that file, log error entry (optional)

// When: File read fails after change detected
// Then: Keep old content, log error entry
```

**Error Recovery**:
- File watching continues even if one file fails
- Next tick will retry failed files

---

### 6.2 Logging Errors

```rust
// When: LogBuffer is full
// Then: Automatically evict oldest entry (not an error)

// When: Empty content passed to log()
// Then: Debug assertion in dev, skip in release (defensive)
```

**No Panics**:
- All logging operations are infallible
- Buffer management is automatic

---

## Contract Validation

### Pre-Implementation Checklist
- [ ] All public APIs documented with examples
- [ ] Preconditions and guarantees stated
- [ ] Error handling strategies defined
- [ ] Performance characteristics specified
- [ ] Test contracts identified

### Post-Implementation Verification
- [ ] Unit tests cover all public methods
- [ ] Integration tests verify event flows
- [ ] Property tests validate invariants
- [ ] Performance benchmarks meet targets

---

**Contract Status**: ✅ **COMPLETE** - All module interfaces and integration points defined
