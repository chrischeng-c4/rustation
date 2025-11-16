# Data Model: History-Based Autosuggestions

**Feature**: 003-autosuggestions
**Date**: 2025-11-17
**Phase**: 1 - Design

## Overview

This document defines the entities and data structures for the autosuggestions feature. The model is intentionally simple: suggestions are ephemeral (computed on-demand), not persisted.

---

## Entity: RushHinter

**Purpose**: Implements reedline's `Hinter` trait to provide history-based autosuggestions.

**Attributes**:
- None (stateless implementation in MVP)

**Behavior**:
- Computes suggestions on-demand from history
- Returns suggestion text or None
- No caching or state management in MVP

**Lifecycle**:
- Created once during REPL initialization
- Lives for entire shell session
- Stateless (no cleanup needed)

**Example**:
```rust
pub struct RushHinter;

impl RushHinter {
    pub fn new() -> Self {
        Self
    }

    /// Find the most recent history entry that starts with the given input
    fn find_suggestion(&self, input: &str, history: &dyn History) -> Option<String> {
        // Implementation in research.md (R2)
    }
}
```

---

## Entity: Suggestion (Conceptual)

**Purpose**: Represents a suggestion to display to the user. This is an ephemeral concept (not a concrete struct in MVP).

**Attributes**:
- `display_text`: The text shown in gray after cursor (e.g., "tatus" for input "git s")
- `full_command`: The complete historical command (e.g., "git status")
- `match_source`: Which history entry provided this suggestion

**State**:
- **Computed**: Suggestion calculated from current input + history
- **Displayed**: Suggestion rendered in terminal (grayed out)
- **Accepted**: User pressed Right Arrow, suggestion becomes input
- **Rejected**: User typed different character, suggestion cleared

**State Transitions**:
```
[User types "git s"]
    ↓
[Computed] → find_suggestion() returns Some("tatus")
    ↓
[Displayed] → Reedline renders "git s" + dimmed "tatus"
    ↓
[User action]:
    - Right Arrow → [Accepted] → buffer becomes "git status"
    - Type 't' → [Rejected] → recompute for "git st"
    - Backspace → [Rejected] → recompute for "git "
```

**Note**: In implementation, there's no `Suggestion` struct. The state is implicit in:
- Input buffer (holds current text)
- Hinter return value (holds suggestion text)
- Reedline display (handles rendering)

---

## Entity: HistoryMatch (Conceptual)

**Purpose**: Represents a history entry that matches current input. Also ephemeral (not a concrete struct).

**Attributes**:
- `entry`: The history entry from reedline's History trait
- `score`: Recency-based ranking (implicit: reverse chronological iteration)
- `prefix`: The portion of input that matched

**Selection Logic**:
```
Input: "git s"
History (reverse chronological):
  1. "git stash" ← MATCH (most recent)
  2. "git status" ← MATCH
  3. "git show HEAD" ← MATCH
  4. "cargo build" ← NO MATCH

Selected: "git stash" (first match = most recent)
```

**Note**: No explicit scoring needed. Reverse chronological iteration provides implicit recency ranking.

---

## Data Flow

### Suggestion Generation Flow

```
User Input
    ↓
┌─────────────────┐
│  Reedline REPL  │
└────────┬────────┘
         │ (on each keystroke)
         ↓
┌─────────────────────────┐
│  RushHinter::hint()     │
│  - line: "git s"        │
│  - pos: 5 (end of line) │
│  - history: &dyn History│
└────────┬────────────────┘
         │
         ↓
┌──────────────────────────┐
│  Check Preconditions     │
│  - pos == line.len()? ✓  │
│  - line.is_empty()? ✗    │
└────────┬─────────────────┘
         │
         ↓
┌───────────────────────────────┐
│  Iterate History (reverse)    │
│  for entry in history.rev() { │
│    if entry.starts_with("git s") │
│       && entry != "git s"      │
│  }                              │
└────────┬──────────────────────┘
         │
         ↓
┌────────────────────────┐
│  Return Suggestion     │
│  Some("tatus")         │
│  (from "git status")   │
└────────┬───────────────┘
         │
         ↓
┌────────────────────────┐
│  Reedline Renders      │
│  "git s" + dim("tatus")│
└────────────────────────┘
```

### Acceptance Flow

```
User Presses Right Arrow (at end of line with suggestion)
    ↓
┌─────────────────────────────┐
│  Reedline EditCommand       │
│  EditCommand::AcceptHint    │
└────────┬────────────────────┘
         │
         ↓
┌─────────────────────────────┐
│  Reedline Updates Buffer    │
│  buffer = "git s" + "tatus" │
│  cursor = end of new buffer │
└────────┬────────────────────┘
         │
         ↓
┌──────────────────────────────┐
│  Recompute Suggestion        │
│  hint("git status", 10, ...)│
│  → None (exact match)        │
└──────────────────────────────┘
         │
         ↓
┌────────────────────────┐
│  Display: "git status" │
│  (no suggestion shown) │
└────────────────────────┘
```

---

## Relationships

```
┌──────────────┐
│ Reedline     │
│ (framework)  │
└──────┬───────┘
       │ uses
       ↓
┌──────────────┐       queries        ┌─────────────────┐
│ RushHinter   │────────────────────→ │ History         │
│ (implements  │                       │ (reedline trait)│
│  Hinter)     │                       └─────────────────┘
└──────────────┘
       │ returns
       ↓
┌──────────────┐
│ Option<String>
│ (suggestion) │
└──────────────┘
```

**Key Points**:
- No custom types needed (leverage reedline's types)
- History managed by reedline's FileBackedHistory
- Hinter is stateless (no state to manage)

---

## Validation Rules

### Input Validation

| Rule | Check | Action |
|------|-------|--------|
| Cursor position | pos != line.len() | Return None (no suggestion) |
| Empty input | line.is_empty() | Return None (no suggestion) |
| Whitespace-only | line.trim().is_empty() | Return None (no suggestion) |

### Suggestion Validation

| Rule | Check | Action |
|------|-------|--------|
| Exact match | entry == input | Skip (don't suggest identical text) |
| Prefix match | entry.starts_with(input) | Include in candidates |
| Empty history | history.is_empty() | Return None gracefully |

### Acceptance Validation

| Rule | Check | Action |
|------|-------|--------|
| Cursor at end | Required by keybinding | EditCommand::AcceptHint only fires at end |
| Suggestion exists | Hinter returned Some(_) | Acceptance proceeds |
| No suggestion | Hinter returned None | Right Arrow moves cursor right instead |

---

## Performance Characteristics

### Memory Footprint

| Component | Size | Notes |
|-----------|------|-------|
| RushHinter | ~0 bytes | Zero-sized type (no fields) |
| Suggestion (ephemeral) | ~24 bytes | String (stack-allocated, short-lived) |
| History cache | 0 bytes (MVP) | No caching in MVP |

**Total**: <1KB overhead (within <1MB requirement)

### Time Complexity

| Operation | Complexity | Typical Case |
|-----------|------------|--------------|
| hint() call | O(n) worst case | O(1) to O(log n) typical |
| Prefix match | O(m) where m = entry length | O(1) average (first few chars) |
| String allocation | O(k) where k = suggestion length | <1μs |

**Total**: ~1-2ms for 10,000 entries (well under 50ms requirement)

---

## Future Optimizations (Out of MVP Scope)

1. **LRU Cache**: Cache last 100 (input → suggestion) pairs
   - Reduces repeated searches
   - Adds ~8KB memory overhead
   - Speeds up "backspace then retype" pattern

2. **Prefix Trie**: Build in-memory index of history
   - O(log n) → O(1) suggestion lookup
   - Adds ~100KB memory for 10k entries
   - Worth it only if >100k history entries

3. **Async Search**: Offload search to background thread
   - Prevents UI blocking on huge histories
   - Adds complexity (thread synchronization)
   - Needed only if measured latency >16ms

**MVP Decision**: No optimizations. Validate performance first, optimize only if measurements show need.

---

## Summary

**Entities**:
- `RushHinter`: Stateless struct implementing `Hinter` trait
- `Suggestion`: Ephemeral concept (no concrete type needed)
- `HistoryMatch`: Ephemeral concept (implicit in iteration)

**Data Flow**:
- Keystroke → hint() → history iteration → suggestion → display
- Right Arrow → AcceptHint → buffer update → rehint → display

**Validation**:
- Cursor position, empty input, exact matches
- All edge cases covered

**Performance**:
- <1KB memory overhead
- ~1-2ms latency (30x under requirement)
- No optimization needed for MVP

**Ready for**: Contract definition and implementation
