# Data Model: Enhanced Worktree View

**Feature**: 049-enhanced-worktree-view
**Date**: 2025-12-14
**Purpose**: Define entities, relationships, and state management for logging and tab navigation

## Entity Definitions

### 1. LogEntry

**Description**: Represents a single logged event in the TUI with timestamp, category, and content.

**Fields**:
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `timestamp` | `SystemTime` | When the log entry was created | Set to `SystemTime::now()` on creation |
| `category` | `LogCategory` | Type of event (SlashCommand, ClaudeStream, etc.) | Must be valid enum variant |
| `content` | `String` | Log message text | Non-empty string |

**State Transitions**: Immutable once created (no state changes)

**Lifecycle**:
1. Created via `LogEntry::new(category, content)`
2. Pushed to LogBuffer
3. Rendered in Output panel
4. Evicted when buffer exceeds 1000 entries

**Validation Rules**:
- `content` must not be empty (enforced at construction)
- `timestamp` must be valid SystemTime (enforced by Rust type system)
- `category` must be valid LogCategory variant

**Example**:
```rust
let entry = LogEntry {
    timestamp: SystemTime::now(),
    category: LogCategory::SlashCommand,
    content: "/speckit.specify Add feature".to_string(),
};
```

---

### 2. LogCategory

**Description**: Enumeration of log entry types for categorization, coloring, and filtering.

**Variants**:
| Variant | Icon | Color | Usage |
|---------|------|-------|-------|
| `SlashCommand` | ⚡ | Cyan | Slash commands executed (e.g., /speckit.specify) |
| `ClaudeStream` | 🤖 | White | Claude Code real-time streaming output |
| `FileChange` | 📝 | Green | External file modifications detected |
| `ShellOutput` | 🔧 | Yellow | Bash script execution and completion |
| `System` | ℹ️ | DarkGray | TUI internal messages and separators |

**Methods**:
```rust
impl LogCategory {
    pub fn icon(&self) -> &'static str;  // Returns emoji icon
    pub fn color(&self) -> Color;         // Returns ratatui Color
}
```

**Relationships**: Referenced by LogEntry.category field

---

### 3. LogBuffer

**Description**: Circular buffer holding up to 1000 log entries with automatic old-entry eviction.

**Fields**:
| Field | Type | Description | Invariants |
|-------|------|-------------|------------|
| `entries` | `VecDeque<LogEntry>` | Ordered log entries (oldest to newest) | `entries.len() <= capacity` |
| `capacity` | `usize` | Maximum entries (constant: 1000) | Always 1000 |

**State Transitions**:
```
Empty (0 entries)
  ↓ push()
Partial (1-999 entries)
  ↓ push() when len < capacity
Partial (1-999 entries)
  ↓ push() when len == capacity
Full (1000 entries) → pop_front() then push_back()
  ↓ continuous push() operations
Full (1000 entries) - steady state
```

**Operations**:
```rust
impl LogBuffer {
    pub fn new() -> Self;                          // Create empty buffer
    pub fn push(&mut self, entry: LogEntry);       // Add entry (evict oldest if full)
    pub fn entries(&self) -> impl Iterator<Item = &LogEntry>;  // Iterate all entries
    pub fn len(&self) -> usize;                    // Current entry count
}
```

**Invariants**:
- `entries.len()` never exceeds `capacity`
- Entries maintain insertion order (FIFO)
- Eviction happens automatically before insertion when full

**Performance**:
- `push()`: O(1) amortized
- `entries()`: O(1) iterator creation, O(n) iteration
- Memory: ~200 KB for 1000 entries

---

### 4. FileChangeTracker

**Description**: Tracks file modification times to detect external changes via polling.

**Fields**:
| Field | Type | Description | Invariants |
|-------|------|-------------|------------|
| `file_mtimes` | `HashMap<PathBuf, SystemTime>` | Map of file paths to last-seen modification times | Keys are absolute paths |

**State Transitions**:
```
Empty (no files tracked)
  ↓ check_files([path1, path2, path3])
Tracking (3 files with mtimes stored)
  ↓ file modified externally
  ↓ check_files() detects mtime change
Modified detected → update stored mtime
```

**Operations**:
```rust
impl FileChangeTracker {
    pub fn new() -> Self;                           // Create empty tracker
    pub fn check_files(&mut self, paths: &[PathBuf]) -> Vec<PathBuf>;  // Check and return changed files
}
```

**Detection Logic**:
1. For each path in `paths`:
   - Call `fs::metadata(path).modified()`
   - Compare with stored `file_mtimes[path]`
   - If different → add to changed list, update stored mtime
   - If not in map → add to map (first-time tracking)

**Edge Cases**:
- File doesn't exist → Skip (no error)
- File deleted → Not detected as change (future enhancement)
- File created → Detected if path was being checked

---

### 5. ContentType (existing, extended)

**Description**: Enumeration of content types displayed in the Content panel.

**Variants**:
| Variant | File | Tab Label |
|---------|------|-----------|
| `Spec` | spec.md | "Spec" |
| `Plan` | plan.md | "Plan" |
| `Tasks` | tasks.md | "Tasks" |

**State Transitions**:
```
Spec
  ↓ Right arrow or 's' key
Plan
  ↓ Right arrow or 's' key
Tasks
  ↓ Right arrow (wraps) or 's' key
Spec (cycle complete)
```

**Usage**: Maps tab selection to file content display

---

## Relationships

```
┌─────────────────┐
│   WorktreeView  │ (owns)
└────────┬────────┘
         │
         ├─────────► LogBuffer (1:1)
         │              │
         │              └─► LogEntry (1:many)
         │                     │
         │                     └─► LogCategory (1:1)
         │
         ├─────────► FileChangeTracker (1:1)
         │              │
         │              └─► HashMap<PathBuf, SystemTime>
         │
         └─────────► ContentType (1:1)
                        │
                        └─► Tab selection state
```

**Cardinality**:
- WorktreeView **has one** LogBuffer
- LogBuffer **contains many** LogEntry instances (0-1000)
- LogEntry **has one** LogCategory
- WorktreeView **has one** FileChangeTracker
- FileChangeTracker **tracks many** file paths (typically 3)
- WorktreeView **has one** ContentType (current selection)

---

## State Management

### WorktreeView Extensions

**New Fields**:
```rust
pub struct WorktreeView {
    // ... existing fields ...

    /// Circular log buffer (replaces Vec<String>)
    pub log_buffer: LogBuffer,

    /// File modification time tracker
    pub file_tracker: FileChangeTracker,

    /// Last tick when files were checked
    pub last_file_check_tick: u64,

    // ... rest of fields ...
}
```

**State Consistency Rules**:
1. `log_buffer.len()` never exceeds 1000
2. `last_file_check_tick` updated every 10 ticks (1 second)
3. `file_tracker` only active when `feature_info.is_some()`
4. Tab state (`content_type`) independent of logging state

---

## Data Flow

### Log Entry Creation Flow

```
User Action (e.g., /speckit.specify)
  ↓
SlashCommandExecuted event
  ↓
app.rs: handle_event()
  ↓
worktree_view.log(SlashCommand, "/speckit.specify")
  ↓
LogEntry::new(category, content)
  ↓
log_buffer.push(entry)
  ↓
(if full) entries.pop_front() → evict oldest
  ↓
entries.push_back(new_entry)
  ↓
render_output() displays updated buffer
```

### File Change Detection Flow

```
tick() method called (every 100ms)
  ↓
Check: tick_count - last_file_check_tick >= 10?
  ↓ (yes, every 1 second)
check_file_changes()
  ↓
file_tracker.check_files([spec.md, plan.md, tasks.md])
  ↓
For each file: fs::metadata().modified()
  ↓
Compare with stored mtime
  ↓ (if different)
Changed files returned
  ↓
For each changed file:
  - Read file content
  - Update spec_content/plan_content/tasks_content
  - Log file change event
```

---

## Storage & Persistence

**In-Memory Only**:
- LogBuffer: Cleared on TUI exit
- FileChangeTracker: Cleared on TUI exit
- ContentType: Ephemeral session state

**No Persistent Storage**:
- Log entries not saved to disk
- File modification times not persisted
- Tab selection not remembered across sessions

**Rationale**: Logging is for current session debugging. Historical logs not required per spec.

**Future Enhancement**: Could add optional log export to file (e.g., Ctrl+E to save)

---

## Performance Characteristics

| Operation | Complexity | Memory |
|-----------|------------|--------|
| LogBuffer.push() | O(1) amortized | O(1) per entry |
| LogBuffer.entries() iteration | O(n) | O(1) |
| FileChangeTracker.check_files() | O(k) where k = file count | O(k) |
| Tab switching | O(1) | O(1) |
| Render log entries | O(v) where v = visible lines | O(v) |

**Memory Bounds**:
- LogBuffer: ≤ 200 KB (1000 entries × 200 bytes)
- FileChangeTracker: ≤ 1 KB (3 paths × ~300 bytes)
- Total overhead: ≤ 1 MB

---

## Validation & Constraints

### LogEntry Constraints
- Content: 1-10,000 characters (typical log line)
- Timestamp: Must be valid SystemTime (enforced by type)
- Category: Must be valid enum variant (enforced by type)

### LogBuffer Constraints
- Capacity: Hard limit of 1000 entries
- No gaps: Entries are contiguous (VecDeque guarantees)
- Order: FIFO (oldest evicted first)

### FileChangeTracker Constraints
- File paths: Must be absolute paths
- Check frequency: Max 1Hz (enforced by tick logic)
- File count: Typically 3 (spec/plan/tasks)

---

## Error Handling

| Error Scenario | Handling Strategy |
|----------------|-------------------|
| File doesn't exist during check | Skip silently (not an error) |
| File read fails after change detected | Log error entry, keep old content |
| fs::metadata() fails | Skip that file, continue with others |
| Log entry content is empty | Prevent at construction (debug_assert) |
| Timestamp in future | Accept (clock skew possible) |

**Graceful Degradation**:
- File watching continues even if one file fails
- Log buffer works even if file watching fails
- Tab navigation works even if no files exist

---

## Testing Strategy

### Unit Tests
- LogBuffer: push/pop/eviction with 1000+ entries
- FileChangeTracker: detect changes, ignore unchanged
- LogCategory: icon/color mapping correctness

### Integration Tests
- End-to-end: Run command → log appears
- File change: Modify file → content updates → log entry appears
- Tab switching: Navigate tabs → content switches

### Property Tests
- LogBuffer never exceeds 1000 entries (property: len <= capacity)
- FileChangeTracker detects all mtimechanges (property: no false negatives)

---

**Data Model Status**: ✅ **COMPLETE** - All entities defined with fields, relationships, and constraints
