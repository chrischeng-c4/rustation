# Tasks: MCP Server Infrastructure

**Feature**: 060-mcp-server-infrastructure
**Created**: 2024-12-17
**Status**: Ready for implementation

## Phase 1: Dependency Setup

- [ ] T001 Add prism-mcp-rs to Cargo.toml
- [ ] T002 Verify cargo build succeeds

## Phase 2: MCP Server Module

- [ ] T003 Create mcp_server.rs with McpServer struct
- [ ] T004 Implement SSE transport setup
- [ ] T005 Implement Tool struct and ToolHandler trait
- [ ] T006 Implement tool registry (register_tool, list_tools)
- [ ] T007 Export module from tui/mod.rs

## Phase 3: App Integration

- [ ] T008 Add MCP server startup in App::new() or run()
- [ ] T009 Pass event sender to MCP server
- [ ] T010 Handle graceful shutdown on app exit

## Phase 4: Auto-Configuration

- [ ] T011 Write MCP config to ~/.rstn/mcp-session.json on startup
- [ ] T012 Set RSTN_MCP_URL environment variable
- [ ] T013 Clean up config on shutdown

## Phase 5: Testing

- [ ] T014 Unit test: server start/stop
- [ ] T015 Unit test: tool registration
- [ ] T016 Integration test: SSE connection
- [ ] T017 Manual test: Claude Code connection

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
