# MCP Server

Embedded Model Context Protocol (MCP) server for exposing project context to AI clients.

## Purpose

Enable AI assistants (Claude Desktop, Claude Code) to safely access project context, read files, list directories, and execute tasks through a standardized Model Context Protocol interface. Each worktree runs its own MCP server instance.

## Requirements

### Requirement: Per-Worktree MCP Server
The system SHALL create one MCP server instance per worktree.

#### Scenario: Start MCP server
- **WHEN** worktree is opened
- **THEN** spawn HTTP SSE server on auto-assigned port (3000, 3001, etc.)

#### Scenario: Stop MCP server
- **WHEN** worktree is closed
- **THEN** shutdown MCP server and release port

### Requirement: HTTP SSE Transport
The system SHALL use Server-Sent Events (SSE) over HTTP for MCP communication.

#### Scenario: Client connection
- **WHEN** AI client connects to `http://localhost:<port>/mcp`
- **THEN** establish SSE connection and handle JSON-RPC requests

#### Scenario: Multiple clients
- **WHEN** multiple clients connect to same server
- **THEN** handle requests concurrently

### Requirement: JSON-RPC 2.0 Protocol
The system SHALL implement JSON-RPC 2.0 specification for MCP communication.

#### Scenario: Handle tools/list request
- **WHEN** client sends `{"method":"tools/list"}`
- **THEN** return list of available tools with schemas

#### Scenario: Handle tools/call request
- **WHEN** client sends tool call with parameters
- **THEN** execute tool and return result or error

### Requirement: Core MCP Tools
The system SHALL provide the following tools to AI clients.

#### Tool: read_file
- **WHEN** tool is called with `{ path: string }`
- **THEN** return file contents if path is within project root, reject if outside

#### Tool: list_directory
- **WHEN** tool is called with `{ path: string }`
- **THEN** return directory entries respecting `.gitignore`, reject paths outside project

#### Tool: get_project_context
- **WHEN** tool is called
- **THEN** return aggregated context from Context Engine (files, git status, docker logs, etc.)

#### Tool: run_just_task
- **WHEN** tool is called with `{ task_name: string }`
- **THEN** execute just command and return output

### Requirement: Security Sandboxing
The system SHALL prevent path traversal and unauthorized access.

#### Scenario: Validate file paths
- **WHEN** tool receives file path parameter
- **THEN** validate path is within project root, reject if outside

#### Scenario: Reject absolute paths outside project
- **WHEN** tool receives `/etc/passwd` or similar
- **THEN** return error "Path outside project root"

### Requirement: MCP Configuration
The system SHALL support MCP configuration for AI clients.

#### Scenario: Generate config for Claude Desktop
- **WHEN** user requests MCP config
- **THEN** generate JSON config with format:
  ```json
  {
    "mcpServers": {
      "rstn": {
        "type": "http",
        "url": "http://localhost:3000/mcp"
      }
    }
  }
  ```

#### Scenario: Auto-port allocation
- **WHEN** multiple worktrees are open
- **THEN** assign sequential ports (3000, 3001, 3002, etc.)

### Requirement: MCP State Management
The system SHALL track MCP server state in WorktreeState.

#### Scenario: Server running
- **WHEN** MCP server is active
- **THEN** set `mcp.server_running = true` and `mcp.port = <assigned_port>`

#### Scenario: Server stopped
- **WHEN** MCP server is inactive
- **THEN** set `mcp.server_running = false`

## State Structure

```rust
pub struct McpState {
    pub server_running: bool,
    pub port: Option<u16>,
    pub connected_clients: usize,
    pub tools: Vec<McpToolInfo>,
    pub last_error: Option<String>,
}

pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
```

## Implementation References

- Backend: `packages/core/src/mcp_server.rs` (Axum HTTP server)
- Config: `packages/core/src/mcp_config.rs`
- State: `packages/core/src/reducer/mcp.rs`
- MCP Spec: https://modelcontextprotocol.io/
