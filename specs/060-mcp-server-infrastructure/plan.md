# Plan: MCP Server Infrastructure

**Feature**: 060-mcp-server-infrastructure
**Created**: 2024-12-17

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│ rstn TUI (app.rs)                                       │
│  └── main loop                                          │
│       └── tokio::spawn(mcp_server::start())             │
│            └── McpServer                                │
│                 ├── SSE endpoint (/sse)                 │
│                 ├── Tool registry                       │
│                 └── Event sender (mpsc)                 │
└─────────────────────────────────────────────────────────┘
              │ SSE
              ↓
┌─────────────────────────────────────────────────────────┐
│ Claude Code                                             │
│  └── MCP Client                                         │
│       ├── tools/list                                    │
│       └── tools/call                                    │
└─────────────────────────────────────────────────────────┘
```

## Implementation Approach

### Phase 1: Add Dependency
- Add `prism-mcp-rs` to `crates/rstn/Cargo.toml`
- Verify it compiles with existing dependencies

### Phase 2: Create MCP Server Module
- Create `crates/rstn/src/tui/mcp_server.rs`
- Implement `McpServer` struct with SSE transport
- Implement basic tool registry (empty initially)
- Export from `mod.rs`

### Phase 3: Integrate with App
- Start MCP server in `App::new()` or `run()`
- Pass event sender for tool → TUI communication
- Handle graceful shutdown

### Phase 4: Auto-Configuration
- Write MCP config to temp file on startup
- Set environment variable for Claude Code
- Clean up on shutdown

## Key Components

### McpServer Struct

```rust
pub struct McpServer {
    port: u16,
    tools: Vec<Tool>,
    event_sender: mpsc::Sender<Event>,
    shutdown_tx: oneshot::Sender<()>,
}

impl McpServer {
    pub async fn start(port: u16, event_sender: mpsc::Sender<Event>) -> Result<Self>;
    pub fn register_tool(&mut self, tool: Tool);
    pub async fn shutdown(self);
}
```

### Tool Registration

```rust
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub handler: Box<dyn ToolHandler>,
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn call(&self, args: serde_json::Value) -> Result<ToolResult>;
}
```

## Files to Create/Modify

| File | Changes |
|------|---------|
| `crates/rstn/Cargo.toml` | Add prism-mcp-rs dependency |
| `crates/rstn/src/tui/mcp_server.rs` | New - MCP server implementation |
| `crates/rstn/src/tui/mod.rs` | Export mcp_server module |
| `crates/rstn/src/tui/app.rs` | Start server, handle shutdown |

## Testing Strategy

1. Unit test: Server starts and stops cleanly
2. Unit test: Tool registration works
3. Integration test: SSE connection works
4. Manual test: Claude Code can connect

## Risk Mitigation

- **Port conflict**: Use dynamic port selection with fallback
- **Startup failure**: Log error, continue without MCP (degraded mode)
- **Dependency issues**: Pin prism-mcp-rs version

## Estimated Complexity

~400-600 lines of new code
