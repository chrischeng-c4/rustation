# Feature 062: MCP Resource Tools

**Feature Branch**: `062-mcp-resource-tools`
**Created**: 2024-12-17
**Status**: Draft
**Depends On**: Feature 061 (MCP Status Tool)

## Overview

Add MCP tools for Claude to read spec artifacts and get current context. This enables dynamic spec injection - Claude reads authoritative spec via tool call instead of relying on prompt-embedded content.

## Problem Statement

Current approach:
1. rstn reads spec.md, plan.md, tasks.md
2. Pastes content into prompt
3. Claude may ignore or misinterpret

Better approach:
- Claude calls `rstn_read_spec("spec")` when needed
- Gets authoritative, current content
- Tool results have higher weight than system prompts

## User Stories

### US1 - Read Spec Artifact (P1)
Claude can read any spec artifact via tool call.

**Acceptance**:
- `rstn_read_spec("spec")` returns spec.md content
- `rstn_read_spec("plan")` returns plan.md content
- `rstn_read_spec("tasks")` returns tasks.md content
- `rstn_read_spec("checklist")` returns checklist.md content

### US2 - Get Current Context (P1)
Claude can get current feature context.

**Acceptance**:
- Returns feature number and name
- Returns current branch
- Returns current SDD phase
- Returns spec directory path

### US3 - Resource Not Found (P2)
Graceful handling when artifact doesn't exist.

**Acceptance**:
- Returns clear error message
- Suggests next action (e.g., "Run specify phase first")

## Requirements

### Functional Requirements

- **FR-001**: `rstn_read_spec` MUST read from current feature's spec directory
- **FR-002**: `rstn_get_context` MUST return current feature metadata
- **FR-003**: Tools MUST handle missing files gracefully
- **FR-004**: Content MUST be current (not cached)

### Tool Schemas

#### rstn_read_spec

```json
{
  "name": "rstn_read_spec",
  "description": "Read a spec artifact for the current feature",
  "inputSchema": {
    "type": "object",
    "properties": {
      "artifact": {
        "type": "string",
        "enum": ["spec", "plan", "tasks", "checklist", "analysis"],
        "description": "Which artifact to read"
      }
    },
    "required": ["artifact"]
  }
}
```

#### rstn_get_context

```json
{
  "name": "rstn_get_context",
  "description": "Get current feature context and metadata",
  "inputSchema": {
    "type": "object",
    "properties": {},
    "required": []
  }
}
```

### Response Formats

#### rstn_read_spec Response

```json
{
  "content": [
    {
      "type": "text",
      "text": "# Feature 060: MCP Server Infrastructure\n\n..."
    }
  ]
}
```

#### rstn_get_context Response

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"feature_num\":\"060\",\"feature_name\":\"mcp-server-infrastructure\",\"branch\":\"060-mcp-server-infrastructure\",\"phase\":\"implement\",\"spec_dir\":\"specs/060-mcp-server-infrastructure\"}"
    }
  ]
}
```

## Technical Design

### Artifact Mapping

```rust
fn artifact_to_filename(artifact: &str) -> &'static str {
    match artifact {
        "spec" => "spec.md",
        "plan" => "plan.md",
        "tasks" => "tasks.md",
        "checklist" => "checklist.md",
        "analysis" => "analysis.md",
        _ => "spec.md",
    }
}
```

### Context Detection

Reuse existing detection logic:
- `detect_current_feature()` from app.rs
- `find_spec_dir()` from app.rs
- Git branch parsing

## Files to Modify

| File | Action |
|------|--------|
| `crates/rstn/src/tui/mcp_server.rs` | Add tool handlers |

## Success Criteria

- [ ] `rstn_read_spec` returns correct artifact content
- [ ] `rstn_get_context` returns current feature metadata
- [ ] Missing artifact returns helpful error
- [ ] Works across all SDD phases

## Complexity

Low (~150-200 lines)
