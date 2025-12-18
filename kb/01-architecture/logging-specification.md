# Logging & Observability Specification

**Created**: 2025-12-18
**Status**: Design Phase - Must Complete Before Implementation
**Purpose**: Define exactly what to log, how to log, and how to observe

---

## Principle

**Observability First**: We don't understand how rstn ↔ MCP ↔ Claude integration works. **Logging must be comprehensive enough to debug ANY issue by reading logs alone.**

---

## Two-Tier Logging System

### Tier 1: Log Panel (UI - Column 3)
**Purpose**: Human-readable, essential info for user to understand workflow

**Target**: User watching the TUI

**Format**:
```
[HH:MM:SS] Category: Message
  Detail line 1
  Detail line 2
```

### Tier 2: Log File (Disk)
**Purpose**: Comprehensive, machine-readable, for debugging

**Target**: Developer investigating issues

**Format**:
```
[YYYY-MM-DD HH:MM:SS.mmm] LEVEL [CATEGORY] Message
  key1: value1
  key2: value2
```

**Location**: `~/.rstn/logs/<session-id>.log`

---

## Log Levels

### Tier 1 (Panel) - Fixed Categories
```
User     - User actions
→        - Outgoing (rstn → Claude)
←        - Incoming (Claude → rstn)
MCP      - MCP tool calls
File     - File system changes
Error    - Errors
System   - System events
```

### Tier 2 (File) - Standard Levels
```
TRACE    - Very detailed (raw data, full payloads)
DEBUG    - Detailed (PIDs, full args, state snapshots)
INFO     - Important events (user actions, completions)
WARN     - Warnings (degraded but working)
ERROR    - Errors (failed operations)
```

---

## What to Log: Complete Checklist

### 1. Session Lifecycle

#### Startup
```rust
// Panel Log:
[14:23:45] System: rstn started
  Session: 2025-12-18-142345-a3f9

// File Log:
[2025-12-18 14:23:45.001] INFO [STARTUP] rstn started
  version: "0.2.0"
  session_id: "2025-12-18-142345-a3f9"
  working_dir: "/Users/user/project"
  feature_branch: "078-worktree-new-command"
  feature_num: "078"
  feature_name: "worktree-new-command"

[2025-12-18 14:23:45.100] INFO [MCP] Starting MCP server
  port: 0 (auto-assign)

[2025-12-18 14:23:45.200] INFO [MCP] MCP server started
  port: 54321
  url: "http://127.0.0.1:54321/mcp"

[2025-12-18 14:23:45.250] DEBUG [MCP] Writing config file
  path: "~/.rstn/mcp-session.json"

[2025-12-18 14:23:45.300] INFO [MCP] Config written
  servers: ["rstn"]
  url: "http://127.0.0.1:54321/mcp"

[2025-12-18 14:23:45.400] INFO [STARTUP] TUI ready
```

#### Shutdown
```rust
// Panel Log:
[14:30:12] System: Shutting down
  Duration: 6m 27s

// File Log:
[2025-12-18 14:30:12.001] INFO [SHUTDOWN] User quit
  session_duration_secs: 387
  commands_executed: 3
  errors: 0

[2025-12-18 14:30:12.100] INFO [MCP] Stopping MCP server
  port: 54321

[2025-12-18 14:30:12.200] INFO [SHUTDOWN] Clean exit
```

---

### 2. User Actions

#### Command Selection
```rust
// Panel Log:
[14:23:50] User: Selected "Prompt Claude"

// File Log:
[2025-12-18 14:23:50.001] INFO [USER] Command selected
  command: "Prompt Claude"
  focus: "Commands"
  display_index: 0
  command_type: "SddPhase(PromptClaude)"
```

#### Input Submission
```rust
// Panel Log:
[14:23:55] User: Submitted input (25 chars)

// File Log:
[2025-12-18 14:23:55.001] INFO [USER] Input submitted
  command: "Prompt Claude"
  input_length: 25
  input_preview: "Fix the bug in worktree..." (first 30 chars)

[2025-12-18 14:23:55.002] DEBUG [USER] Full input
  input: """
Fix the bug in worktree view
"""
```

#### Focus Change
```rust
// Panel Log:
(Not logged in panel - too noisy)

// File Log:
[2025-12-18 14:23:52.001] DEBUG [USER] Focus changed
  from: "Commands"
  to: "Content"
```

#### Scroll
```rust
// Panel Log:
(Not logged)

// File Log:
[2025-12-18 14:23:53.001] TRACE [USER] Scroll
  pane: "Content"
  offset: 10
```

---

### 3. Command Execution

#### Building Command
```rust
// Panel Log:
[14:23:56] → Claude CLI
  Building command...

// File Log:
[2025-12-18 14:23:56.001] INFO [CMD] Building Claude CLI command
  command: "Prompt Claude"
  user_input: "Fix the bug in worktree view"

[2025-12-18 14:23:56.002] DEBUG [CMD] Claude CLI options
  max_turns: 10
  skip_permissions: true
  output_format: "stream-json"
  verbose: true
  include_partial_messages: true
  mcp_config: "~/.rstn/mcp-session.json"
  system_prompt_file: "/path/to/prompt.md"

[2025-12-18 14:23:56.003] DEBUG [CMD] Full command line
  binary: "/usr/local/bin/claude"
  args: ["-p", "Fix the bug...", "--output-format", "stream-json", ...]

[2025-12-18 14:23:56.004] TRACE [CMD] Full args (one per line)
  arg[0]: "-p"
  arg[1]: "Fix the bug in worktree view"
  arg[2]: "--output-format"
  arg[3]: "stream-json"
  arg[4]: "--verbose"
  arg[5]: "--include-partial-messages"
  arg[6]: "--mcp-config"
  arg[7]: "/Users/user/.rstn/mcp-session.json"
  arg[8]: "--system-prompt-file"
  arg[9]: "/path/to/prompt.md"
```

#### Spawning Process
```rust
// Panel Log:
[14:23:56] → Spawning process...

// File Log:
[2025-12-18 14:23:56.100] INFO [CMD] Spawning Claude CLI
  binary: "/usr/local/bin/claude"
  working_dir: "/Users/user/project"

[2025-12-18 14:23:56.200] INFO [CMD] Process spawned
  pid: 12345
  elapsed_ms: 100
```

#### Process Failed to Spawn
```rust
// Panel Log:
[14:23:56] Error: Failed to spawn Claude CLI
  Error: No such file or directory

// File Log:
[2025-12-18 14:23:56.100] ERROR [CMD] Failed to spawn process
  binary: "/usr/local/bin/claude"
  error: "No such file or directory"
  search_paths: ["/usr/local/bin", "/usr/bin", ...]
```

---

### 4. Stream Processing

#### Stream Start
```rust
// Panel Log:
[14:23:57] ← Stream started

// File Log:
[2025-12-18 14:23:57.001] INFO [STREAM] Stream started
  pid: 12345
```

#### Chunk Received
```rust
// Panel Log:
[14:23:58] ← Chunk 1/? (235 bytes)

// File Log:
[2025-12-18 14:23:58.001] DEBUG [STREAM] Chunk received
  chunk_num: 1
  size_bytes: 235
  elapsed_since_start_ms: 1000

[2025-12-18 14:23:58.002] TRACE [STREAM] Raw chunk
  raw: """
{"type":"init","session_id":"abc-123","apiKeySource":"ANTHROPIC_API_KEY"}
"""
```

#### Parse Chunk
```rust
// Panel Log:
[14:23:58] ← Init (session: abc-123)

// File Log:
[2025-12-18 14:23:58.010] INFO [STREAM] Parsed chunk
  type: "init"
  session_id: "abc-123"
  api_key_source: "ANTHROPIC_API_KEY"

[2025-12-18 14:23:58.011] TRACE [STREAM] Full parsed JSON
  json: {
    "type": "init",
    "session_id": "abc-123",
    "apiKeySource": "ANTHROPIC_API_KEY",
    "model": "claude-sonnet-4-20250514"
  }
```

#### Parse Error
```rust
// Panel Log:
[14:23:58] Error: Failed to parse chunk
  Invalid JSON

// File Log:
[2025-12-18 14:23:58.010] ERROR [STREAM] Parse error
  error: "expected value at line 1 column 1"
  chunk_num: 3
  size_bytes: 150

[2025-12-18 14:23:58.011] TRACE [STREAM] Failed chunk content
  raw: """
{invalid json here
"""
```

#### Content Chunk (Assistant Message)
```rust
// Panel Log:
[14:24:00] ← Response (512 bytes)

// File Log:
[2025-12-18 14:24:00.001] INFO [STREAM] Assistant message
  chunk_num: 5
  role: "assistant"
  content_blocks: 1
  size_bytes: 512

[2025-12-18 14:24:00.002] DEBUG [STREAM] Content preview
  preview: "Sure, I can help with that bug. Let me investigate..."

[2025-12-18 14:24:00.003] TRACE [STREAM] Full content
  content: """
Sure, I can help with that bug. Let me investigate the worktree view code...
"""
```

#### Result Message (Final)
```rust
// Panel Log:
[14:24:05] ← Complete
  Session: abc-123
  Cost: $0.003
  Duration: 8.2s

// File Log:
[2025-12-18 14:24:05.001] INFO [STREAM] Result message
  type: "result"
  session_id: "abc-123"
  is_error: false
  total_cost_usd: 0.003
  duration_ms: 8200
  num_turns: 6

[2025-12-18 14:24:05.002] DEBUG [STREAM] Result details
  result: "Task completed successfully"
  total_chunks: 12
  total_bytes: 4096
```

#### Stream End (Process Exit)
```rust
// Panel Log:
[14:24:05] ← Process exited
  Exit code: 0
  Duration: 8.5s

// File Log:
[2025-12-18 14:24:05.100] INFO [CMD] Process exited
  pid: 12345
  exit_code: 0
  duration_ms: 8500

[2025-12-18 14:24:05.101] DEBUG [CMD] Process stats
  stdout_bytes: 4096
  stderr_bytes: 0
  chunks_received: 12
```

#### Non-Zero Exit
```rust
// Panel Log:
[14:24:05] Error: Process failed
  Exit code: 1

// File Log:
[2025-12-18 14:24:05.100] ERROR [CMD] Process failed
  pid: 12345
  exit_code: 1
  duration_ms: 2500

[2025-12-18 14:24:05.101] ERROR [CMD] Stderr output
  stderr: """
Error: API key not found
"""

[2025-12-18 14:24:05.102] TRACE [CMD] Full stdout (even if failed)
  stdout: """
{"type":"init"...}
"""
```

---

### 5. MCP Events

#### MCP Connection (from Claude)
```rust
// Panel Log:
[14:23:57] MCP: Client connected

// File Log:
[2025-12-18 14:23:57.001] INFO [MCP] Client connected
  remote_addr: "127.0.0.1:54322"
  user_agent: "Claude-CLI/1.0"
```

#### MCP Tool Call
```rust
// Panel Log:
[14:24:02] MCP: rstn_report_status
  Status: needs_input

// File Log:
[2025-12-18 14:24:02.001] INFO [MCP] Tool call received
  method: "tools/call"
  tool: "rstn_report_status"
  request_id: "req-123"

[2025-12-18 14:24:02.002] DEBUG [MCP] Tool arguments
  args: {
    "status": "needs_input",
    "prompt": "Enter branch name"
  }

[2025-12-18 14:24:02.003] TRACE [MCP] Full JSON-RPC request
  jsonrpc: "2.0"
  id: "req-123"
  method: "tools/call"
  params: {
    "name": "rstn_report_status",
    "arguments": {...}
  }
```

#### MCP Tool Response
```rust
// Panel Log:
[14:24:05] MCP: Response sent

// File Log:
[2025-12-18 14:24:05.001] INFO [MCP] Tool response sent
  request_id: "req-123"
  success: true
  duration_ms: 3000

[2025-12-18 14:24:05.002] DEBUG [MCP] Response content
  content: [{"type": "text", "text": "User input: main"}]

[2025-12-18 14:24:05.003] TRACE [MCP] Full JSON-RPC response
  jsonrpc: "2.0"
  id: "req-123"
  result: {
    "content": [...]
  }
```

#### MCP Tool Error
```rust
// Panel Log:
[14:24:03] Error: MCP tool failed
  Tool: rstn_read_spec
  Error: File not found

// File Log:
[2025-12-18 14:24:03.001] ERROR [MCP] Tool execution failed
  request_id: "req-124"
  tool: "rstn_read_spec"
  error: "File not found: spec.md"

[2025-12-18 14:24:03.002] DEBUG [MCP] Tool args that failed
  args: {
    "artifact": "spec"
  }

[2025-12-18 14:24:03.003] INFO [MCP] Error response sent
  error_code: -32603
  error_message: "Internal error"
```

---

### 6. Hooks

#### Hook Fired
```rust
// Panel Log:
[14:24:05] Hook: tool-result
  Running...

// File Log:
[2025-12-18 14:24:05.200] INFO [HOOK] Hook executing
  hook: "tool-result"
  path: ".claude/hooks/tool-result"

[2025-12-18 14:24:05.300] DEBUG [HOOK] Hook stdout
  stdout: """
🔨 Detected rstn changes, rebuilding...
"""

[2025-12-18 14:24:08.001] INFO [HOOK] Hook completed
  hook: "tool-result"
  exit_code: 0
  duration_ms: 2800

[2025-12-18 14:24:08.002] DEBUG [HOOK] Hook stderr
  stderr: """
✅ rstn rebuilt successfully
"""
```

#### Hook Failed
```rust
// Panel Log:
[14:24:05] Error: Hook failed
  Hook: tool-result
  Exit code: 1

// File Log:
[2025-12-18 14:24:05.200] ERROR [HOOK] Hook failed
  hook: "tool-result"
  exit_code: 1
  duration_ms: 500

[2025-12-18 14:24:05.201] ERROR [HOOK] Hook stderr
  stderr: """
⚠️ rstn build failed
error: could not compile `rstn`
"""
```

---

### 7. File System Operations

#### File Read
```rust
// Panel Log:
(Not logged unless error)

// File Log:
[2025-12-18 14:24:01.001] DEBUG [FILE] Reading file
  path: "specs/078-worktree-new-command/spec.md"
  reason: "MCP tool: rstn_read_spec"

[2025-12-18 14:24:01.002] INFO [FILE] File read
  path: "specs/078-worktree-new-command/spec.md"
  size_bytes: 2048
  duration_ms: 1
```

#### File Write
```rust
// Panel Log:
[14:24:10] File: + spec.md

// File Log:
[2025-12-18 14:24:10.001] INFO [FILE] Writing file
  path: "specs/078-worktree-new-command/spec.md"
  size_bytes: 3072
  reason: "SaveSpec action"

[2025-12-18 14:24:10.002] DEBUG [FILE] File content preview
  preview: "# Spec: Worktree New Command..."

[2025-12-18 14:24:10.003] TRACE [FILE] Full content
  content: """
# Spec: Worktree New Command
...
"""
```

#### File Error
```rust
// Panel Log:
[14:24:01] Error: File not found
  Path: spec.md

// File Log:
[2025-12-18 14:24:01.001] ERROR [FILE] File operation failed
  operation: "read"
  path: "specs/078-worktree-new-command/spec.md"
  error: "No such file or directory"
```

---

### 8. State Transitions

#### State Change
```rust
// Panel Log:
(Not logged - too noisy)

// File Log:
[2025-12-18 14:23:56.001] DEBUG [STATE] State transition
  from: "Idle{focus: Commands, content: Spec}"
  to: "PromptInput{buffer: \"\", cursor: 0}"
  trigger: "Command selected: Prompt Claude"

[2025-12-18 14:23:58.001] DEBUG [STATE] State transition
  from: "PromptInput{...}"
  to: "CommandRunning{command: \"Prompt Claude\", progress: 0.0}"
  trigger: "Input submitted"

[2025-12-18 14:24:05.001] DEBUG [STATE] State transition
  from: "CommandRunning{...}"
  to: "ShowingResult{content: \"...\"}"
  trigger: "Command completed successfully"
```

---

### 9. Errors

#### User-Facing Error
```rust
// Panel Log:
[14:24:00] Error: Input too short
  Minimum 10 characters

// File Log:
[2025-12-18 14:24:00.001] WARN [ERROR] Validation error
  type: "InputValidation"
  message: "Input too short (5 chars, minimum 10)"
  user_input_length: 5
  recoverable: true
```

#### System Error
```rust
// Panel Log:
[14:24:05] Error: Command failed
  See logs for details

// File Log:
[2025-12-18 14:24:05.001] ERROR [ERROR] Command execution error
  type: "CommandFailed"
  command: "Prompt Claude"
  exit_code: 1
  duration_ms: 2000
  recoverable: false

[2025-12-18 14:24:05.002] ERROR [ERROR] Error context
  session_id: "2025-12-18-142345-a3f9"
  feature_branch: "078-worktree-new-command"
  last_successful_command: "git status"
  time_since_last_success_secs: 120
```

---

## Log Format Standards

### Panel Log Format
```
[HH:MM:SS] Category: First line message
  Detail line 1 (2 spaces indent)
  Detail line 2
```

**Example**:
```
[14:23:56] → Claude CLI
  Building command...

[14:23:57] ← Stream started

[14:24:00] ← Response (512 bytes)

[14:24:05] ← Complete
  Session: abc-123
  Cost: $0.003
  Duration: 8.2s
```

### File Log Format
```
[YYYY-MM-DD HH:MM:SS.mmm] LEVEL [CATEGORY] Message
  key1: value1
  key2: value2
```

**Example**:
```
[2025-12-18 14:23:56.001] INFO [CMD] Building Claude CLI command
  command: "Prompt Claude"
  user_input: "Fix the bug in worktree view"

[2025-12-18 14:23:56.100] INFO [CMD] Process spawned
  pid: 12345
  elapsed_ms: 100
```

---

## Log Rotation

### File Naming
```
~/.rstn/logs/
├── 2025-12-18-142345-a3f9.log  (current session)
├── 2025-12-18-131022-b2c1.log  (previous session)
├── 2025-12-17-095533-d4e2.log  (old session)
```

### Cleanup Policy
- Keep all logs from last 7 days
- Keep 1 log per day for last 30 days
- Delete logs older than 30 days
- Max total size: 100 MB (delete oldest if exceeded)

---

## Observability Tools

### 1. Log Viewer (built-in)
```bash
# View current session
rstn-cli logs

# View specific session
rstn-cli logs 2025-12-18-142345-a3f9

# Follow logs (tail -f)
rstn-cli logs --follow

# Filter by level
rstn-cli logs --level ERROR

# Filter by category
rstn-cli logs --category MCP
```

### 2. Log Analysis
```bash
# Find all errors
grep "ERROR" ~/.rstn/logs/2025-12-18-*.log

# Find all MCP tool calls
grep "MCP.*Tool call" ~/.rstn/logs/2025-12-18-*.log

# Session duration
grep "session_duration_secs" ~/.rstn/logs/2025-12-18-*.log

# Exit codes
grep "exit_code" ~/.rstn/logs/2025-12-18-*.log
```

### 3. Copy Debug Info (Y key)
**Format**:
```
=== rstn Debug Info ===
Session: 2025-12-18-142345-a3f9
Time: 2025-12-18 14:24:05
Feature: 078-worktree-new-command

=== Last Command ===
Command: Prompt Claude
Input: "Fix the bug in worktree view"

Full command line:
claude -p "Fix the bug..." --output-format stream-json --verbose ...

=== Result ===
Exit code: 0
Duration: 8.5s
Session ID: abc-123
Cost: $0.003

=== MCP Events (last 5) ===
[14:23:57] MCP: Client connected
[14:24:02] MCP: rstn_report_status (needs_input)
[14:24:05] MCP: rstn_read_spec (spec)

=== Errors (last 5) ===
(none)

=== Recent Log Excerpt (last 20 lines) ===
[14:23:56] → Claude CLI
[14:23:57] ← Stream started
[14:24:00] ← Response (512 bytes)
[14:24:05] ← Complete

=== Files Changed ===
+ spec.md (3072 bytes)

Full logs: ~/.rstn/logs/2025-12-18-142345-a3f9.log
```

---

## Implementation Checklist

Before writing ANY code:
- [ ] Logging infrastructure is designed (this doc)
- [ ] Flow diagrams are complete (rstn-integration-flow.md)
- [ ] State machine is documented (worktree-state-machine.md)
- [ ] UI design is finalized (worktree-view-redesign.md)

Then implement in order:
1. [ ] Session ID generation
2. [ ] Log file creation (~/.rstn/logs/)
3. [ ] Panel log buffer (in-memory)
4. [ ] Log formatting (Panel + File)
5. [ ] Log each checkpoint (see "What to Log" section)
6. [ ] Copy functions (y/Y keys)
7. [ ] Test: Can we debug issues from logs alone?

---

## Success Criteria

**We have good observability when:**
- ✅ Every user action is logged (with context)
- ✅ Every external call is logged (command, args, result)
- ✅ Every state transition is logged
- ✅ Every error is logged (with full context)
- ✅ Logs are copy-friendly (Y key → paste to Claude Code)
- ✅ Can answer "what happened?" by reading logs alone
- ✅ Can reproduce any issue from log file

---

## Related Documents

- [Worktree View Redesign](worktree-view-redesign.md) - UI design with log panel
- [Integration Flow](rstn-integration-flow.md) - What events to observe
- [State Machine](worktree-state-machine.md) - What state transitions to log

---

## Changelog

- 2025-12-18: Initial logging specification created
