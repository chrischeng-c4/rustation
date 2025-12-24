---
title: "MCP Server Architecture"
description: "Embedded MCP HTTP server for Claude Code integration"
category: architecture
status: implemented
last_updated: 2025-12-24
version: 1.0.0
tags: [mcp, http, fastapi, claude-code]
weight: 11
---

# MCP Server Architecture

## 1. Overview

rstn embeds an MCP (Model Context Protocol) HTTP server within the TUI process. This enables bidirectional communication between Claude Code and the TUI without external processes.

### Key Design Decisions

1. **Embedded Server**: MCP server runs in the same process as TUI (no IPC overhead)
2. **Dynamic Port**: Uses port 0 for auto-assignment (prevents conflicts)
3. **FastAPI + uvicorn**: Modern async HTTP stack
4. **Session-Scoped**: Each TUI session has unique MCP endpoint

## 2. Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    rstn TUI Process                      │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────────┐              │
│  │ Claude Code  │───▶│  MCP HTTP Server │              │
│  │   (client)   │    │   (dynamic port) │              │
│  └──────────────┘    └────────┬─────────┘              │
│                               │                         │
│              ┌────────────────┴────────────────┐       │
│              ▼                                 ▼       │
│     ┌─────────────────┐            ┌──────────────┐   │
│     │ Read-only Tools │            │ Action Tools │   │
│     │ state_getter()  │            │ msg_sender() │   │
│     └─────────────────┘            └──────┬───────┘   │
│                                           ▼           │
│                              ┌─────────────────────┐  │
│                              │ asyncio.Queue[Msg]  │  │
│                              │   → reduce() → UI   │  │
│                              └─────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## 3. Module Structure

```
rstn/
  mcp/
    __init__.py           # Public exports
    server.py             # FastAPI server lifecycle (~80 lines)
    routes.py             # MCP HTTP endpoints (~100 lines)
    tools.py              # MCP tool implementations (~320 lines)
    types.py              # MCP-specific types (~130 lines)
    hooks.py              # Hook config & execution (~140 lines)
```

## 4. Tool Categories

### Data Plane (Read-Only)

Tools that read state without side effects:

| Tool | Purpose |
|------|---------|
| `rstn_get_app_state` | Get full TUI state as JSON |
| `rstn_read_spec` | Read spec artifacts |
| `rstn_get_context` | Get feature context |

Implementation pattern:
```python
async def _tool_rstn_get_app_state(self, args: dict) -> McpToolResponse:
    state = self.state_getter()  # Synchronous, direct access
    return McpToolResponse.text(state.model_dump_json())
```

### Control Plane (Actions)

Tools that trigger state changes via message queue:

| Tool | Purpose |
|------|---------|
| `rstn_report_status` | Report needs_input/completed/error |
| `rstn_complete_task` | Mark task as complete |
| `rstn_run_hook` | Execute project hook |

Implementation pattern:
```python
async def _tool_rstn_report_status(self, args: dict) -> McpToolResponse:
    msg = McpReportStatusReceived(status=args["status"], ...)
    await self.msg_sender(msg)  # Route through reducer
    return McpToolResponse.text("Status reported")
```

## 5. Message Flow

### MCP → TUI State Updates

```
1. Claude Code calls /mcp/tools/rstn_report_status
2. McpToolRegistry creates McpReportStatusReceived message
3. Message sent via msg_sender → asyncio.Queue
4. TUI event loop picks up message
5. reduce(state, msg) → new_state, effects
6. UI re-renders with new state
```

### TUI → MCP State Access

```
1. Claude Code calls /mcp/tools/rstn_get_app_state
2. McpToolRegistry calls state_getter()
3. Direct synchronous access to AppState
4. Serialized to JSON and returned
```

## 6. Server Lifecycle

### Startup (in on_mount)

```python
async def _start_mcp_server(self) -> None:
    config = McpServerConfig(host="127.0.0.1", port=0, session_id=self._mcp_session_id)
    self._mcp_server = McpServer(config, state_getter, msg_sender, project_root)
    port = await self._mcp_server.start()
    await self._msg_queue.put(McpServerStarted(port=port, session_id=...))
```

### Shutdown (in cleanup)

```python
async def _cleanup(self) -> None:
    if self._mcp_server:
        await self._mcp_server.stop()
```

## 7. Hook System

Project-specific commands are configured in `.rstn/hooks.yaml`:

```yaml
hooks:
  lint:
    command: "uv run ruff check ."
    timeout_secs: 60
  test:
    command: "uv run pytest"
    timeout_secs: 300
  format:
    command: "uv run ruff format ."
    timeout_secs: 60
```

Hook execution:
1. Load config from `.rstn/hooks.yaml` or `.rstn/hooks.json`
2. Spawn subprocess with timeout
3. Capture stdout/stderr
4. Return HookResult with exit_code, output, duration

## 8. HTTP Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/mcp/` | GET | Server info |
| `/mcp/tools` | GET | List available tools |
| `/mcp/tools/{name}` | POST | Call a tool |
| `/mcp/health` | GET | Health check |

## 9. MCP Configuration for Claude CLI

When starting Claude CLI, pass the MCP config:

```bash
claude -p "{prompt}" \
  --mcp-config /tmp/rstn/{session_id}/mcp-config.json \
  ...
```

Config format:
```json
{
  "mcpServers": {
    "rstn": {
      "type": "http",
      "url": "http://127.0.0.1:{port}/mcp"
    }
  }
}
```

## 10. Testing

Tests are in `tests/test_mcp/`:

- `test_types.py` - Type serialization
- `test_tools.py` - Tool invocation
- `test_hooks.py` - Hook execution

## 11. Implementation Reference

| File | Purpose |
|------|---------|
| `rstn/mcp/server.py` | McpServer class |
| `rstn/mcp/routes.py` | FastAPI router |
| `rstn/mcp/tools.py` | McpToolRegistry |
| `rstn/mcp/types.py` | Type definitions |
| `rstn/mcp/hooks.py` | Hook system |
| `rstn/msg/__init__.py` | MCP message types |
| `rstn/reduce/__init__.py` | MCP reducers |
| `rstn/tui/app.py` | Server integration |
