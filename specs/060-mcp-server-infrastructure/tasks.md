# Tasks: MCP Server Infrastructure

**Feature**: 060-mcp-server-infrastructure
**Created**: 2024-12-17
**Status**: Ready for implementation

## Phase 1: Dependency Setup

- [x] T001 Add prism-mcp-rs to Cargo.toml
- [x] T002 Verify cargo build succeeds

## Phase 2: MCP Server Module

- [x] T003 Create mcp_server.rs with McpServer struct
- [x] T004 Implement SSE transport setup
- [x] T005 Implement Tool struct and ToolHandler trait
- [x] T006 Implement tool registry (register_tool, list_tools)
- [x] T007 Export module from tui/mod.rs

## Phase 3: App Integration

- [x] T008 Add MCP server startup in App::new() or run()
- [x] T009 Pass event sender to MCP server
- [x] T010 Handle graceful shutdown on app exit

## Phase 4: Auto-Configuration

- [x] T011 Write MCP config to ~/.rstn/mcp-session.json on startup
- [x] T012 Set RSTN_MCP_URL environment variable
- [x] T013 Clean up config on shutdown

## Phase 5: Testing

- [x] T014 Unit test: server start/stop
- [x] T015 Unit test: tool registration
- [x] T016 Integration test: SSE connection
- [x] T017 Manual test: Claude Code connection

## Dependencies

```
T001 → T002 → T003
T003 → T004 → T005 → T006 → T007
T007 → T008 → T009 → T010
T010 → T011 → T012 → T013
T013 → T014 → T015 → T016 → T017
```

## Notes

- Use port 19560 as default (configurable)
- prism-mcp-rs handles MCP protocol
- Event sender allows tool calls to trigger TUI updates
