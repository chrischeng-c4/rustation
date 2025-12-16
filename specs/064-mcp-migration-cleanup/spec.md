# Feature 064: MCP Migration & Cleanup (Hard Cutover)

**Feature Branch**: `064-mcp-migration-cleanup`
**Created**: 2024-12-17
**Status**: Draft
**Depends On**: Feature 063 (MCP Task Completion)

## Overview

Remove all text-based status parsing from rstn. MCP becomes the sole control channel. This is a hard cutover - no backward compatibility with `rscli-status` blocks.

## Problem Statement

After MCP tools are implemented (060-063), the old text parsing code becomes:
- Dead code (never executed)
- Maintenance burden
- Confusing for future developers

## Solution

Remove:
- `RscliStatus` struct
- `parse_status()` method
- Status block constants
- Text-based fallback in event handling

Keep:
- `ClaudeStreamMessage` (display channel)
- `get_text()` method
- `total_cost_usd`, `session_id` handling

## User Stories

### US1 - Remove Status Parsing (P1)
All `rscli-status` parsing code is removed.

**Acceptance**:
- `claude_stream.rs` simplified
- No references to `RscliStatus`
- `cargo build` succeeds

### US2 - Update System Prompt (P1)
Remove status block instructions from prompts.

**Acceptance**:
- `RSCLI_SYSTEM_PROMPT` no longer mentions status blocks
- Claude uses MCP tools instead

### US3 - Clean Event Handling (P2)
Remove text-based fallback in `handle_claude_completed()`.

**Acceptance**:
- Only MCP events trigger state changes
- Simpler, more maintainable code

## Requirements

### Functional Requirements

- **FR-001**: MUST remove `RscliStatus` struct
- **FR-002**: MUST remove `parse_status()` and related methods
- **FR-003**: MUST remove status block constants
- **FR-004**: MUST update `handle_claude_completed()` to MCP-only
- **FR-005**: MUST update system prompts

### Code to Remove

#### From `claude_stream.rs`

```rust
// REMOVE these:
const STATUS_BLOCK_START: &str = "```rscli-status";
const STATUS_BLOCK_END: &str = "```";

pub struct RscliStatus {
    pub status: String,
    pub prompt: Option<String>,
    pub message: Option<String>,
}

impl ClaudeStreamMessage {
    pub fn parse_status(&self) -> Option<RscliStatus> { ... }
    pub fn needs_input(&self) -> bool { ... }
    pub fn is_completed(&self) -> bool { ... }
    pub fn has_error(&self) -> bool { ... }
    pub fn get_input_prompt(&self) -> Option<String> { ... }
    pub fn get_error_message(&self) -> Option<String> { ... }
    pub fn get_display_text(&self) -> Option<String> { ... } // Simplify, remove strip logic
}
```

#### From `cargo.rs`

```rust
// UPDATE RSCLI_SYSTEM_PROMPT - remove:
// "Output status blocks in format: ```rscli-status {...} ```"
// Replace with:
// "Use rstn_report_status tool to report status changes"
```

#### From `app.rs`

```rust
// In handle_claude_completed():
// REMOVE text-based status detection
// REMOVE fallback heuristics ("ends with ?", etc.)
// KEEP session_id handling
```

### Code to Keep

```rust
// In claude_stream.rs - KEEP these:
pub struct ClaudeStreamMessage {
    pub msg_type: String,
    pub message: Option<ClaudeMessage>,
    pub session_id: Option<String>,
    pub result: Option<String>,
    pub total_cost_usd: Option<f64>,
    pub is_error: Option<bool>,
}

impl ClaudeStreamMessage {
    pub fn get_text(&self) -> Option<String> { ... } // KEEP for display
}
```

## Technical Design

### Simplified `get_display_text()`

Before (complex):
```rust
pub fn get_display_text(&self) -> Option<String> {
    let text = self.get_text()?;
    // Complex logic to strip status blocks
    if let Some(start) = text.find(STATUS_BLOCK_START) {
        // ... 20 lines of stripping ...
    }
    Some(text)
}
```

After (simple):
```rust
pub fn get_display_text(&self) -> Option<String> {
    self.get_text() // No stripping needed - no status blocks in output
}
```

### Updated Event Handling

```rust
// In app.rs handle_claude_completed()
fn handle_claude_completed(&mut self, phase: String, success: bool, session_id: Option<String>) {
    // Save session for resume
    if let Some(sid) = session_id {
        session::save_session_id(&self.feature_number, &sid);
    }

    // Status changes now come via Event::McpStatus
    // No text-based detection here

    if success {
        self.status_message = Some(format!("{} phase completed", phase));
    }
}
```

## Files to Modify

| File | Action |
|------|--------|
| `crates/rstn/src/tui/claude_stream.rs` | Remove ~100 lines |
| `crates/rstn/src/runners/cargo.rs` | Update RSCLI_SYSTEM_PROMPT |
| `crates/rstn/src/tui/app.rs` | Simplify handle_claude_completed |

## Success Criteria

- [ ] `RscliStatus` struct removed
- [ ] `parse_status()` and related methods removed
- [ ] Status block constants removed
- [ ] System prompt updated for MCP
- [ ] All tests pass
- [ ] No regressions in SDD workflow

## Complexity

Low (~200 lines removed, ~50 lines modified)

## Risk Mitigation

- Feature flag to re-enable text parsing if needed (temporary)
- Comprehensive testing before merge
- Quick rollback plan (revert commit)
