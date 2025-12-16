# Feature 061: MCP Status Tool

**Feature Branch**: `061-mcp-status-tool`
**Created**: 2024-12-17
**Status**: Draft
**Depends On**: Feature 060 (MCP Server Infrastructure)

## Overview

Replace fragile `rscli-status` text block parsing with a robust MCP tool call. Claude calls `rstn_report_status()` instead of outputting markdown blocks.

## Problem Statement

Current status reporting:
```markdown
```rscli-status
{"status":"needs_input","prompt":"Enter description"}
```
```

Issues:
- Claude may forget to output the block
- Format errors break `parse_status()`
- Mixing control with display text

## Solution

Claude calls MCP tool instead:
```json
{
  "method": "tools/call",
  "params": {
    "name": "rstn_report_status",
    "arguments": {
      "status": "needs_input",
      "prompt": "Enter description"
    }
  }
}
```

## User Stories

### US1 - Status Tool Registration (P1)
rstn registers `rstn_report_status` tool with MCP server.

**Acceptance**:
- Tool appears in `tools/list` response
- Schema validates status enum

### US2 - Status Tool Execution (P1)
When Claude calls `rstn_report_status`, rstn updates state.

**Acceptance**:
- `needs_input` → Shows InputDialog
- `completed` → Marks phase done
- `error` → Shows error message

### US3 - State Machine Integration (P2)
Tool call triggers same state transitions as current text parsing.

**Acceptance**:
- All existing workflows continue to work
- No behavioral regression

## Requirements

### Functional Requirements

- **FR-001**: Tool MUST accept `status` (required): "needs_input" | "completed" | "error"
- **FR-002**: Tool MUST accept `prompt` (optional): string for input prompt
- **FR-003**: Tool MUST accept `message` (optional): string for error message
- **FR-004**: Tool call MUST trigger state machine via event channel

### Tool Schema

```json
{
  "name": "rstn_report_status",
  "description": "Report current task status to rstn control plane",
  "inputSchema": {
    "type": "object",
    "properties": {
      "status": {
        "type": "string",
        "enum": ["needs_input", "completed", "error"],
        "description": "Current status"
      },
      "prompt": {
        "type": "string",
        "description": "Prompt to show user (for needs_input)"
      },
      "message": {
        "type": "string",
        "description": "Error message (for error status)"
      }
    },
    "required": ["status"]
  }
}
```

## Technical Design

### Event Flow

```
Claude Code                    rstn MCP Server              rstn TUI
    │                               │                          │
    │ tools/call rstn_report_status │                          │
    │ ─────────────────────────────>│                          │
    │                               │ Event::McpStatus         │
    │                               │ ────────────────────────>│
    │                               │                          │ Update state
    │                               │                          │ Show dialog
    │         {"content":[...]}     │                          │
    │ <─────────────────────────────│                          │
```

### New Event Type

```rust
// In event.rs
pub enum Event {
    // ... existing ...

    /// MCP tool call received (Feature 061)
    McpStatus {
        status: String,      // "needs_input", "completed", "error"
        prompt: Option<String>,
        message: Option<String>,
    },
}
```

### Tool Handler

```rust
// In mcp_server.rs
async fn handle_report_status(
    args: ReportStatusArgs,
    event_sender: mpsc::Sender<Event>,
) -> Result<ToolResult, McpError> {
    event_sender.send(Event::McpStatus {
        status: args.status,
        prompt: args.prompt,
        message: args.message,
    }).await?;

    Ok(ToolResult::text("Status reported"))
}
```

## Files to Modify

| File | Action |
|------|--------|
| `crates/rstn/src/tui/mcp_server.rs` | Add tool registration and handler |
| `crates/rstn/src/tui/event.rs` | Add McpStatus event |
| `crates/rstn/src/tui/app.rs` | Handle McpStatus event |

## Success Criteria

- [ ] Tool registered and discoverable
- [ ] `needs_input` shows InputDialog
- [ ] `completed` marks phase done
- [ ] `error` displays error message
- [ ] Existing phase workflows work via MCP

## Complexity

Medium (~200-300 lines)
