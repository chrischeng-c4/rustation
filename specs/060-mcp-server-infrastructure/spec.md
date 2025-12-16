# Feature 060: MCP Server Infrastructure

**Feature Branch**: `060-mcp-server-infrastructure`
**Created**: 2024-12-17
**Status**: Ready for Implementation

## Overview

Add an embedded MCP (Model Context Protocol) server to rstn TUI, enabling Claude Code to interact with rstn via structured tool calls instead of fragile text-based status blocks.

## Problem Statement

Current rstn ↔ Claude Code communication relies on:
1. Claude outputting `rscli-status` markdown blocks
2. rstn parsing these blocks via string matching (`parse_status()`)

This is fragile because:
- Claude may forget to output the block
- Format errors break parsing
- Mixing control logic with display text

## Solution: Dual-Channel Architecture

| Channel | Transport | Purpose |
|---------|-----------|---------|
| Display | stream-json (stdout) | Real-time text, cost, session_id |
| Control | MCP (SSE localhost) | Tool calls, state transitions |

## User Stories

### US1 - MCP Server Startup (P1)
When rstn starts, it launches an embedded MCP server on localhost.

**Acceptance**:
- Server starts on configurable port (default: 19560)
- Server logs startup to rstn log
- Server accepts SSE connections

### US2 - Claude Code Connection (P1)
Claude Code can connect to rstn's MCP server.

**Acceptance**:
- rstn auto-generates MCP config for Claude
- Claude connects via SSE transport
- Connection status visible in rstn

### US3 - Tool Registration (P2)
rstn exposes tools via MCP that Claude can discover.

**Acceptance**:
- `tools/list` returns registered tools
- Tool schemas are valid JSON Schema
- Tools execute when called

## Requirements

### Functional Requirements

- **FR-001**: Server MUST use SSE transport (TUI owns stdio)
- **FR-002**: Server MUST support MCP tool calls
- **FR-003**: Server MUST be configurable (port, enable/disable)
- **FR-004**: rstn MUST auto-configure Claude Code to connect

### Non-Functional Requirements

- **NFR-001**: Server startup < 100ms
- **NFR-002**: Tool call latency < 50ms
- **NFR-003**: No impact on TUI responsiveness

## Research Phase (Completed)

### Option A: Official rust-sdk
- **Repo**: https://github.com/modelcontextprotocol/rust-sdk
- **Version**: 0.11.0 (Dec 2025)
- **SSE Support**: No (needs rmcp-actix-web extension)
- **Maturity**: High (2.7k stars, 421 forks)
- **Verdict**: Best for stdio-based servers, not ideal for embedded TUI

### Option B: mcp-sdk (crates.io)
- **Version**: 0.0.3 (Jan 2025)
- **SSE Support**: Yes (built-in)
- **Maturity**: Low (early stage)
- **API**: Builder pattern, minimalistic
- **Verdict**: Simple but immature

### Option C: prism-mcp-rs (Recommended)
- **Repo**: https://github.com/prismworks-ai/prism-mcp-rs
- **Version**: 0.1.5
- **SSE Support**: Yes (SSE, HTTP/2, WebSocket)
- **Maturity**: Medium (enterprise features, CI/CD)
- **Features**: Hot-reload, circuit breaker, zero-copy
- **Verdict**: Best fit for embedded TUI server

### Decision: prism-mcp-rs
- SSE transport built-in (TUI owns stdio)
- Tokio-based (already using tokio)
- Enterprise-grade reliability
- Hot-reload for future tool updates

## Technical Design

### Server Architecture
```
┌────────────────────────────────────────┐
│ rstn TUI                               │
│  ├── App (main loop)                   │
│  │    └── spawns MCP server task       │
│  └── mcp_server.rs                     │
│       ├── axum router                  │
│       ├── SSE endpoint                 │
│       ├── Tool registry                │
│       └── State (Arc<Mutex<AppState>>) │
└────────────────────────────────────────┘
         │ SSE
         ↓
┌────────────────────────────────────────┐
│ Claude Code                            │
│  └── MCP Client                        │
│       ├── tools/list                   │
│       ├── tools/call                   │
│       └── resources/read (future)      │
└────────────────────────────────────────┘
```

### MCP Protocol Subset (Minimum)
```json
// Request: tools/list
{"jsonrpc":"2.0","method":"tools/list","id":1}

// Response
{"jsonrpc":"2.0","result":{"tools":[...]},"id":1}

// Request: tools/call
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"rstn_report_status","arguments":{...}},"id":2}

// Response
{"jsonrpc":"2.0","result":{"content":[{"type":"text","text":"OK"}]},"id":2}
```

### Auto-Configuration

On rstn startup:
1. Start MCP server on port
2. Write temp config: `~/.rstn/mcp-session.json`
3. Set env var: `RSTN_MCP_URL=http://localhost:19560/sse`
4. Claude Code reads config via `--mcp-config` or env

## Files to Create/Modify

| File | Action |
|------|--------|
| `crates/rstn/Cargo.toml` | Add prism-mcp-rs |
| `crates/rstn/src/tui/mcp_server.rs` | New - MCP server wrapper |
| `crates/rstn/src/tui/mod.rs` | Export mcp_server |
| `crates/rstn/src/tui/app.rs` | Start server on init |

## Success Criteria

- [ ] MCP server starts with rstn
- [ ] Claude Code can connect via SSE
- [ ] `tools/list` returns empty list (tools added in 061)
- [ ] Server doesn't block TUI
- [ ] Graceful shutdown on rstn exit

## Dependencies

- Feature 059 (complete) - current architecture baseline
- prism-mcp-rs (new dep) - MCP server with SSE
- tokio (existing) - async runtime

## Complexity

Medium (~400-600 lines)
