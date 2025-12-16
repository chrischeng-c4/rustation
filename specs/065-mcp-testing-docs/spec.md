# Feature 065: MCP Testing & Documentation

**Feature Branch**: `065-mcp-testing-docs`
**Created**: 2024-12-17
**Status**: Draft
**Depends On**: Feature 064 (MCP Migration & Cleanup)

## Overview

Comprehensive testing and documentation for the MCP dual-channel architecture. Ensures robustness and provides clear guidance for future development.

## User Stories

### US1 - Unit Tests (P1)
All MCP components have unit tests.

**Acceptance**:
- MCP server startup/shutdown tested
- Tool registration tested
- Tool execution tested
- Event dispatch tested

### US2 - Integration Tests (P1)
End-to-end workflow tests with MCP.

**Acceptance**:
- Tool call → state change verified
- Full SDD phase workflow tested
- Error handling verified

### US3 - Documentation (P2)
Architecture and usage documented.

**Acceptance**:
- CLAUDE.md updated with MCP section
- Tool schemas documented
- Troubleshooting guide

## Requirements

### Functional Requirements

- **FR-001**: Unit tests for `mcp_server.rs`
- **FR-002**: Integration test for tool → event → state flow
- **FR-003**: Documentation in CLAUDE.md
- **FR-004**: Tool schema reference

## Test Plan

### Unit Tests

#### MCP Server Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_startup() {
        let (server, _) = McpServer::start(0).await.unwrap();
        assert!(server.port() > 0);
        server.shutdown().await;
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let server = McpServer::new();
        server.register_tool("rstn_report_status", schema, handler);

        let tools = server.list_tools();
        assert!(tools.iter().any(|t| t.name == "rstn_report_status"));
    }

    #[tokio::test]
    async fn test_report_status_needs_input() {
        let (tx, mut rx) = mpsc::channel(10);
        let server = McpServer::with_event_sender(tx);

        server.call_tool("rstn_report_status", json!({
            "status": "needs_input",
            "prompt": "Enter description"
        })).await.unwrap();

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, Event::McpStatus { status, .. } if status == "needs_input"));
    }

    #[tokio::test]
    async fn test_read_spec_not_found() {
        let result = handle_read_spec(json!({"artifact": "spec"}), &context).await;
        assert!(result.is_error);
        assert!(result.text.contains("not found"));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_status_flow() {
    // 1. Start app with MCP server
    let mut app = App::new_test();

    // 2. Simulate tool call
    app.handle_mcp_tool_call("rstn_report_status", json!({
        "status": "needs_input",
        "prompt": "Enter feature description"
    })).await;

    // 3. Verify state change
    assert!(app.input_dialog.is_some());
    assert_eq!(app.input_dialog.unwrap().prompt(), "Enter feature description");
}

#[tokio::test]
async fn test_task_completion_flow() {
    let mut app = App::new_test();
    app.load_test_tasks();

    // Complete task via MCP
    app.handle_mcp_tool_call("rstn_complete_task", json!({
        "task_id": "T001"
    })).await;

    // Verify task marked complete
    let tasks = app.worktree_view.specify_state.task_list_state.as_ref().unwrap();
    assert!(tasks.tasks[0].completed);
}
```

## Documentation

### CLAUDE.md Addition

```markdown
## MCP Architecture (Features 060-065)

rstn uses a dual-channel architecture for Claude Code communication:

### Display Channel (stream-json)
- Real-time text rendering
- Cost tracking (`total_cost_usd`)
- Session management (`session_id`)

### Control Channel (MCP)
- State transitions via tool calls
- Structured JSON-RPC
- SSE transport on localhost

### Available Tools

| Tool | Purpose |
|------|---------|
| `rstn_report_status` | Report status changes (needs_input, completed, error) |
| `rstn_read_spec` | Read spec artifacts (spec, plan, tasks, checklist) |
| `rstn_get_context` | Get current feature context |
| `rstn_complete_task` | Mark task complete with validation |

### Tool Usage Example

Instead of outputting status blocks:
```
❌ Old: Output ```rscli-status {"status":"needs_input"} ```
✅ New: Call rstn_report_status({"status":"needs_input","prompt":"..."})
```

### Troubleshooting

- **MCP server not starting**: Check port availability (default: 19560)
- **Tool not found**: Ensure rstn version >= 0.2.0
- **Connection refused**: Verify `RSTN_MCP_URL` environment variable
```

### Tool Schema Reference

Create `docs/mcp-tools.md`:

```markdown
# rstn MCP Tool Reference

## rstn_report_status

Report current task status to rstn control plane.

**Input Schema:**
- `status` (required): "needs_input" | "completed" | "error"
- `prompt` (optional): Prompt string for needs_input
- `message` (optional): Error message for error status

**Example:**
\`\`\`json
{"status": "needs_input", "prompt": "Describe the feature"}
\`\`\`

## rstn_read_spec

Read a spec artifact for the current feature.

**Input Schema:**
- `artifact` (required): "spec" | "plan" | "tasks" | "checklist" | "analysis"

**Example:**
\`\`\`json
{"artifact": "spec"}
\`\`\`

## rstn_get_context

Get current feature context and metadata.

**Input Schema:** (none required)

**Response:**
\`\`\`json
{
  "feature_num": "060",
  "feature_name": "mcp-server-infrastructure",
  "branch": "060-mcp-server-infrastructure",
  "phase": "implement",
  "spec_dir": "specs/060-mcp-server-infrastructure"
}
\`\`\`

## rstn_complete_task

Mark a task as complete with optional validation.

**Input Schema:**
- `task_id` (required): Task ID (e.g., "T001")
- `skip_validation` (optional): Skip validation checks

**Example:**
\`\`\`json
{"task_id": "T001"}
\`\`\`
```

## Files to Create/Modify

| File | Action |
|------|--------|
| `crates/rstn/src/tui/mcp_server.rs` | Add #[cfg(test)] module |
| `crates/rstn/tests/mcp_integration_test.rs` | New - integration tests |
| `CLAUDE.md` | Add MCP architecture section |
| `docs/mcp-tools.md` | New - tool reference |

## Success Criteria

- [ ] Unit tests for all MCP components
- [ ] Integration test for full workflow
- [ ] CLAUDE.md updated
- [ ] Tool schema documentation created
- [ ] All tests pass in CI

## Complexity

Low (~300 lines tests, ~200 lines docs)
