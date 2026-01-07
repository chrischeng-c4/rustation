# Context Engine

Intelligent context aggregation system for AI workflows.

## Purpose

Automatically gather and prioritize relevant project information (files, Git status, Docker logs, terminal output) to provide AI assistants with optimal context. Reduce token usage while maximizing relevance through intelligent filtering and prioritization.

## Requirements

### Requirement: Context Source Aggregation
The system SHALL gather relevant information from multiple sources to build AI context.

#### Scenario: Aggregate from file system
- **WHEN** building context
- **THEN** include currently open file path and cursor line, directory tree (depth=2)

#### Scenario: Aggregate from Git
- **WHEN** building context
- **THEN** include branch name, unstaged changes (git diff), and file status

#### Scenario: Aggregate from runtime state
- **WHEN** building context
- **THEN** include Docker error logs, last task output, and terminal history

### Requirement: Token Budget Management
The system SHALL optimize context to fit within token limits.

#### Scenario: Priority-based inclusion
- **WHEN** total context exceeds budget
- **THEN** include items by priority: High → Active Files → Git Diff → Low → Directory Tree

#### Scenario: Estimate token count
- **WHEN** adding context item
- **THEN** estimate tokens as ~4 characters per token

### Requirement: Context Generation
The system SHALL format context as Markdown for AI consumption.

#### Scenario: Generate system prompt
- **WHEN** AI client requests context
- **THEN** format as structured Markdown with sections:
  ```markdown
  ## Project Context
  **Path**: /path/to/project
  **Branch**: feature/auth

  ## Active Files
  - src/main.rs (line 42)

  ## Git Status
  Modified: 3 files

  ## Docker Status
  postgres: Running (port 5432)
  redis: Error (connection refused)

  ## Recent Output
  [Last 20 lines of terminal/task output]
  ```

### Requirement: Context Priority Levels
The system SHALL assign priority levels to context sources.

#### Scenario: High priority items
- **WHEN** categorizing context
- **THEN** mark as High: Active file with cursor, Docker errors, Task failures

#### Scenario: Normal priority items
- **WHEN** categorizing context
- **THEN** mark as Normal: Git diff, Terminal output, File tree

#### Scenario: Low priority items
- **WHEN** categorizing context
- **THEN** mark as Low: Directory structure, Metadata

### Requirement: Context Refresh
The system SHALL update context when state changes.

#### Scenario: File changed
- **WHEN** user switches to different file
- **THEN** refresh context with new active file

#### Scenario: Git status changed
- **WHEN** user commits or stages changes
- **THEN** refresh context with updated git status

#### Scenario: Docker state changed
- **WHEN** container status changes
- **THEN** refresh context with updated Docker info

### Requirement: MCP Integration
The system SHALL provide context to MCP `get_project_context` tool.

#### Scenario: MCP tool call
- **WHEN** AI client calls `get_project_context`
- **THEN** generate and return current context as formatted string

### Requirement: Context Caching
The system SHALL cache context to avoid redundant computation.

#### Scenario: Cache valid context
- **WHEN** context is generated
- **THEN** cache result for 5 seconds

#### Scenario: Invalidate cache
- **WHEN** relevant state changes (file switch, git action, Docker update)
- **THEN** clear cached context

## State Structure

```rust
pub struct ContextState {
    pub sources: Vec<ContextSource>,
    pub last_generated: Option<String>,
    pub last_generated_at: Option<DateTime<Utc>>,
    pub token_budget: usize,
    pub priority_config: PriorityConfig,
}

pub struct ContextSource {
    pub kind: ContextKind,  // File | Git | Docker | Terminal | Directory
    pub content: String,
    pub priority: Priority, // High | Normal | Low
    pub token_estimate: usize,
}

pub enum Priority {
    High = 3,
    Normal = 2,
    Low = 1,
}
```

## Implementation References

- Backend: `packages/core/src/context_engine.rs`
- State: `packages/core/src/reducer/context.rs`
- MCP Integration: `packages/core/src/mcp_server.rs` (get_project_context tool)
