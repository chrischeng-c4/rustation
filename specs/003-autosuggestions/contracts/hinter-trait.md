# Contract: Hinter Trait Implementation

**Feature**: 003-autosuggestions
**Component**: RushHinter
**Contract Type**: Interface Implementation (reedline::Hinter)
**Date**: 2025-11-17

## Interface Contract

### Trait Signature (from reedline)

```rust
pub trait Hinter: Send {
    /// Provide a suggestion for the current line and cursor position
    ///
    /// # Arguments
    /// * `line` - The current input buffer content
    /// * `pos` - The current cursor position (0-indexed)
    /// * `history` - Access to command history via History trait
    ///
    /// # Returns
    /// * `Some(String)` - The suggestion text to display (without the input prefix)
    /// * `None` - No suggestion available
    fn hint(&mut self, line: &str, pos: usize, history: &dyn History) -> Option<String>;
}
```

### Implementation Contract

```rust
pub struct RushHinter;

impl RushHinter {
    /// Creates a new RushHinter instance
    ///
    /// # Returns
    /// A new, stateless RushHinter
    pub fn new() -> Self {
        Self
    }
}

impl Hinter for RushHinter {
    fn hint(&mut self, line: &str, pos: usize, history: &dyn History) -> Option<String> {
        // Contract implementation defined below
    }
}
```

---

## Behavioral Contract

### Preconditions

| Condition | Check | Behavior if Violated |
|-----------|-------|---------------------|
| Cursor at end of line | `pos == line.len()` | Return `None` (no suggestion) |
| Non-empty input | `!line.is_empty()` | Return `None` (no suggestion for empty input) |
| Valid cursor position | `pos <= line.len()` | Panic in debug, graceful return in release |

### Postconditions

| Outcome | Guarantee |
|---------|-----------|
| `None` returned | No suggestion should be displayed |
| `Some(text)` returned | `text` is the suffix to append to `line` for display |
| No panic | Function never panics (defensive programming) |
| No blocking | Function completes within 50ms |
| No mutation | History is not modified (read-only access) |

### Invariants

| Invariant | Description |
|-----------|-------------|
| Suggestion completeness | If `Some(text)`, then `line + text` is a valid complete command from history |
| No exact match | Never suggest text that results in exact match with input |
| Most recent first | If multiple matches exist, return most recent |
| No side effects | Function is pure (besides internal caching, future) |

---

## Input/Output Contract

### Input Space

| Parameter | Type | Domain | Example |
|-----------|------|--------|---------|
| `line` | `&str` | Any valid UTF-8 string | `"git s"`, `"cargo build"`, `""` |
| `pos` | `usize` | `0..=line.len()` | `5` (for "git s") |
| `history` | `&dyn History` | Any history implementation | `FileBackedHistory` |

### Output Space

| Return Value | Meaning | Example |
|--------------|---------|---------|
| `None` | No suggestion available | Input: `"xyz"` (no match) |
| `Some("")` | Invalid, should not occur | N/A |
| `Some("tatus")` | Suggest "tatus" after current input | Input: `"git s"`, Match: `"git status"` |
| `Some("uild --release")` | Suggest rest of command | Input: `"cargo b"`, Match: `"cargo build --release"` |

---

## Functional Requirements Mapping

### FR-001: Display autosuggestions as grayed-out text

**Contract**:
- Hinter returns `Some(text)` when suggestion available
- Reedline handles rendering (out of contract scope)
- Hinter guarantees: text is displayable UTF-8

**Test**:
```rust
#[test]
fn test_returns_displayable_suggestion() {
    let mut hinter = RushHinter::new();
    let mut history = MockHistory::new();
    history.add("git status");

    let result = hinter.hint("git s", 5, &history);

    assert_eq!(result, Some("tatus".to_string()));
    assert!(result.unwrap().is_ascii()); // Displayable
}
```

### FR-002: Search command history for entries starting with input

**Contract**:
- Iterate `history.iter_chronologic()` in reverse order
- Check `entry.starts_with(line)` for each entry
- Return first match

**Test**:
```rust
#[test]
fn test_prefix_matching() {
    let mut hinter = RushHinter::new();
    let mut history = MockHistory::new();
    history.add("git commit");
    history.add("git status");
    history.add("cargo build");

    // Matches "git" prefix
    let result = hinter.hint("git", 3, &history);
    assert!(result.is_some());

    // No match for "xyz"
    let result = hinter.hint("xyz", 3, &history);
    assert_eq!(result, None);
}
```

### FR-003: Suggest most recent matching entry

**Contract**:
- Reverse chronological iteration guarantees first match is most recent
- Return immediately on first match (don't continue searching)

**Test**:
```rust
#[test]
fn test_most_recent_match() {
    let mut hinter = RushHinter::new();
    let mut history = MockHistory::new();
    history.add("git status");   // Older
    history.add("git stash");    // Newer

    let result = hinter.hint("git s", 5, &history);

    // Should suggest "tash" from "git stash" (most recent)
    assert_eq!(result, Some("tash".to_string()));
}
```

### FR-004: Update suggestions in real-time

**Contract**:
- Hinter called on every keystroke by reedline
- Stateless design ensures fresh computation each call
- No caching means no stale suggestions

**Test**:
```rust
#[test]
fn test_realtime_update() {
    let mut hinter = RushHinter::new();
    let mut history = MockHistory::new();
    history.add("git status");

    // First call: "git "
    let result1 = hinter.hint("git ", 4, &history);
    assert_eq!(result1, Some("status".to_string()));

    // Second call: "git s" (user typed 's')
    let result2 = hinter.hint("git s", 5, &history);
    assert_eq!(result2, Some("tatus".to_string()));

    // Third call: "git st" (user typed 't')
    let result3 = hinter.hint("git st", 6, &history);
    assert_eq!(result3, Some("atus".to_string()));
}
```

### FR-005: Only display when cursor at end of line

**Contract**:
- Check `pos == line.len()` before searching
- Return `None` if cursor not at end

**Test**:
```rust
#[test]
fn test_cursor_position_check() {
    let mut hinter = RushHinter::new();
    let mut history = MockHistory::new();
    history.add("git status");

    // Cursor at end: suggest
    let result = hinter.hint("git s", 5, &history);
    assert!(result.is_some());

    // Cursor in middle: no suggestion
    let result = hinter.hint("git s", 3, &history);
    assert_eq!(result, None);

    // Cursor at start: no suggestion
    let result = hinter.hint("git s", 0, &history);
    assert_eq!(result, None);
}
```

### FR-008: Clear suggestion when no match

**Contract**:
- If no history entry starts with `line`, return `None`
- Reedline clears suggestion display on `None`

**Test**:
```rust
#[test]
fn test_no_match_returns_none() {
    let mut hinter = RushHinter::new();
    let mut history = MockHistory::new();
    history.add("git status");

    // No match for "cargo"
    let result = hinter.hint("cargo", 5, &history);
    assert_eq!(result, None);
}
```

### FR-009: Handle empty history gracefully

**Contract**:
- If `history.is_empty()`, iteration returns no entries
- Return `None` without error

**Test**:
```rust
#[test]
fn test_empty_history() {
    let mut hinter = RushHinter::new();
    let history = MockHistory::new(); // Empty

    let result = hinter.hint("git s", 5, &history);
    assert_eq!(result, None); // Graceful, no panic
}
```

---

## Performance Contract

### Latency Requirements

| Operation | Maximum Time | Measurement |
|-----------|-------------|-------------|
| `hint()` call | 50ms | 99th percentile with 10k history |
| History iteration | 2ms | Worst case for 10k entries |
| Prefix check | 50ns | Per-entry average |
| String allocation | 1μs | For suggestion text |

**Violation Handling**: If measured latency >50ms, optimization required (see data-model.md future optimizations).

### Memory Requirements

| Component | Maximum Size | Notes |
|-----------|-------------|-------|
| RushHinter instance | 0 bytes | Zero-sized type |
| Temporary allocation | <1KB | Per-call suggestion string |
| Total overhead | <10KB | Including call stack |

**Violation Handling**: If memory exceeds 1MB, caching strategy must be reevaluated.

---

## Error Handling Contract

### Panic Conditions

| Condition | Behavior | Mitigation |
|-----------|----------|------------|
| `pos > line.len()` | Debug panic, release graceful | Check precondition, return None |
| History iteration error | Propagate to caller | Reedline handles gracefully |

### Graceful Degradation

| Scenario | Behavior |
|----------|----------|
| Empty history | Return `None`, no error |
| No matching entry | Return `None`, no error |
| Cursor in middle | Return `None`, no error |
| Extremely long input | Truncate search, return best match or None |

---

## Integration Contract

### Reedline Integration

```rust
// In src/repl/mod.rs
use crate::repl::suggest::RushHinter;

let hinter = Box::new(RushHinter::new());
let editor = Reedline::create()
    .with_hinter(hinter)
    .with_history(history);
```

**Contract**:
- Hinter instance owned by Reedline
- `hint()` called on every keystroke
- Returned suggestion rendered by Reedline (dimmed style)

### History Integration

```rust
// History trait from reedline
pub trait History: Send {
    fn iter_chronologic(&self) -> impl Iterator<Item = &HistoryEntry>;
    // ... other methods
}
```

**Contract**:
- Read-only access to history
- Iteration in chronological order (oldest to newest)
- No modification of history state

---

## Acceptance Criteria

### AC-1: Suggestions Display Correctly

**Given**: History contains "git status"
**When**: User types "git s"
**Then**: Hint returns `Some("tatus")`
**And**: Reedline displays "git s" + dimmed "tatus"

### AC-2: Most Recent Match Preferred

**Given**: History contains ["git status" (old), "git stash" (new)]
**When**: User types "git s"
**Then**: Hint returns `Some("tash")` (from "git stash")

### AC-3: No Suggestion When Cursor Not at End

**Given**: History contains "git status"
**When**: User types "git status" but cursor is at position 3
**Then**: Hint returns `None`

### AC-4: Empty History Handled Gracefully

**Given**: History is empty
**When**: User types "git s"
**Then**: Hint returns `None` without panic

### AC-5: Performance Under Load

**Given**: History contains 10,000 entries
**When**: User types "git s"
**Then**: Hint completes within 50ms (99th percentile)

---

## Summary

**Contract Guarantees**:
- ✅ Implements reedline::Hinter trait correctly
- ✅ Returns most recent matching history entry
- ✅ Only suggests when cursor at end of line
- ✅ Handles empty history and no-match gracefully
- ✅ Completes within 50ms for 10k history entries
- ✅ No panics in release builds
- ✅ No memory leaks or unbounded growth

**Dependencies**:
- reedline::Hinter (trait to implement)
- reedline::History (data source)

**Testing Strategy**:
- Unit tests: All FR mappings covered
- Integration tests: Acceptance criteria validated
- Performance benchmarks: Latency under load

**Ready for**: Implementation (Phase 2)
