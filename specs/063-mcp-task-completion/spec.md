# Feature 063: MCP Task Completion

**Feature Branch**: `063-mcp-task-completion`
**Created**: 2024-12-17
**Status**: Draft
**Depends On**: Feature 062 (MCP Resource Tools)

## Overview

Add MCP tool for Claude to mark tasks as complete with validation. When Claude finishes a task, it calls `rstn_complete_task()` which validates completion criteria before updating status.

## Problem Statement

Current task completion:
1. Claude outputs "task complete" text
2. User manually marks task done
3. No automated validation

Better approach:
- Claude calls `rstn_complete_task(task_id)`
- rstn validates (tests pass, file exists, etc.)
- Auto-updates task status in TUI
- Returns error if validation fails

## User Stories

### US1 - Complete Task (P1)
Claude marks a task complete via tool call.

**Acceptance**:
- Task status updates in tasks.md
- TUI refreshes to show completion
- Progress indicator updates

### US2 - Validation Failure (P1)
If validation fails, task remains incomplete.

**Acceptance**:
- Returns specific error message
- Suggests remediation
- Task stays incomplete

### US3 - Auto-Advance (P2)
After completion, optionally advance to next task.

**Acceptance**:
- Returns next incomplete task info
- Context ready for next task

## Requirements

### Functional Requirements

- **FR-001**: Tool MUST accept task_id (e.g., "T001", "T002")
- **FR-002**: Tool MUST validate task exists in tasks.md
- **FR-003**: Tool SHOULD run validation checks (configurable)
- **FR-004**: Tool MUST update task status in memory and file
- **FR-005**: Tool MUST return next incomplete task (if any)

### Tool Schema

```json
{
  "name": "rstn_complete_task",
  "description": "Mark a task as complete with optional validation",
  "inputSchema": {
    "type": "object",
    "properties": {
      "task_id": {
        "type": "string",
        "description": "Task ID (e.g., T001, T002)"
      },
      "skip_validation": {
        "type": "boolean",
        "default": false,
        "description": "Skip validation checks"
      }
    },
    "required": ["task_id"]
  }
}
```

### Response Format

#### Success

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"completed\":true,\"task_id\":\"T001\",\"next_task\":{\"id\":\"T002\",\"description\":\"Implement feature X\"},\"progress\":\"3/10 tasks complete\"}"
    }
  ]
}
```

#### Validation Failure

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"completed\":false,\"task_id\":\"T001\",\"error\":\"Tests failing: 2 failures in test_foo\",\"suggestion\":\"Fix failing tests before marking complete\"}"
    }
  ],
  "isError": true
}
```

## Technical Design

### Validation Pipeline (Configurable)

```rust
struct TaskValidator {
    checks: Vec<Box<dyn ValidationCheck>>,
}

trait ValidationCheck {
    fn name(&self) -> &str;
    fn validate(&self, task: &Task, context: &Context) -> Result<(), String>;
}

// Built-in checks:
// - TestsPassCheck: Run `cargo test` for relevant tests
// - FileExistsCheck: Verify expected files created
// - CompileCheck: Ensure `cargo build` succeeds
```

### Integration with TaskListState

```rust
// In worktree.rs - reuse existing
impl SpecifyState {
    pub fn complete_task_by_id(&mut self, task_id: &str) -> Result<(), String> {
        if let Some(ref mut task_list) = self.task_list_state {
            task_list.complete_by_id(task_id)?;
            self.save_tasks_to_file()?;
        }
        Ok(())
    }
}
```

### Event for TUI Update

```rust
pub enum Event {
    // ... existing ...

    /// Task completed via MCP (Feature 063)
    McpTaskCompleted {
        task_id: String,
        next_task: Option<String>,
    },
}
```

## Files to Modify

| File | Action |
|------|--------|
| `crates/rstn/src/tui/mcp_server.rs` | Add tool handler |
| `crates/rstn/src/tui/event.rs` | Add McpTaskCompleted event |
| `crates/rstn/src/tui/app.rs` | Handle event, refresh TUI |
| `crates/rstn/src/tui/views/worktree.rs` | Add `complete_task_by_id()` |

## Success Criteria

- [ ] Tool marks task complete in tasks.md
- [ ] TUI updates to show completion
- [ ] Validation failure blocks completion
- [ ] Returns next task info
- [ ] Progress indicator updates

## Complexity

Low-Medium (~200-250 lines)
