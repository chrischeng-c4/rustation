# Worktree View Redesign - Three Column Layout

**Created**: 2025-12-18
**Status**: Design Phase - Ready for Implementation
**Context**: Refactoring Worktree view (currently 4,118 lines) to be simpler, more observable

---

## Executive Summary

**Goal**: Simplify Worktree view to focus on **one core workflow**: Send prompts to Claude Code and observe what happens.

**Key Changes**:
1. **Three-column layout** (was two columns)
2. **Remove tab bar** (focus only on Worktree)
3. **Two-tier logging** (detailed file + human-readable panel)
4. **Session tracking** (timestamp-based session IDs)
5. **Copy functions** (easy to paste debug info to Claude Code)

**Why**:
- Current Worktree view is too complex (54+ fields, 4,118 lines)
- We don't understand how rstn ↔ Claude Code ↔ MCP integration works
- Need **observability** to debug and fix the system

---

## Layout Design

### Three-Column Layout (20% / 40% / 40%)

```
┌───────────────────────────────────────────────────────────────────────────┐
│ rstn                                    Session: 2025-12-18-142345-a3f9   │
├───────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────┬──────────────────────────┬─────────────────────────┐   │
│  │  Commands   │  Content (Interactive)   │  Logs (Human-Readable)  │   │
│  │  20%        │  40%                     │  40%                    │   │
│  │             │                          │                         │   │
│  │ > Prompt    │  ┌────────────────────┐  │ Session started         │   │
│  │   Claude    │  │ Enter prompt:      │  │                         │   │
│  │             │  │                    │  │ [14:23:45] User: Send   │   │
│  │ Specify     │  │ Fix the bug in...  │  │   prompt to Claude      │   │
│  │ Clarify     │  │ > _                │  │                         │   │
│  │ Plan        │  └────────────────────┘  │ [14:23:46] → Claude CLI │   │
│  │ Tasks       │                          │   --print --verbose     │   │
│  │ Implement   │                          │   --mcp-config ...      │   │
│  │             │                          │                         │   │
│  │ Commit      │  When running:           │ [14:23:48] ← Streaming  │   │
│  │ Push        │  ┌────────────────────┐  │   Response chunk 1/3    │   │
│  │             │  │ [████████░░] 80%   │  │                         │   │
│  │             │  │                    │  │ [14:23:50] ← Complete   │   │
│  │             │  │ Streaming output...│  │   Exit: 0, 5.2s         │   │
│  │             │  └────────────────────┘  │                         │   │
│  │             │                          │ MCP events:             │   │
│  │             │                          │ • rstn_report_status    │   │
│  │             │                          │ • needs_input: false    │   │
│  └─────────────┴──────────────────────────┴─────────────────────────┘   │
│                                                                           │
│  q:Quit  Enter:Run  y:Copy Pane  Y:Copy All Debug Info  ?:Help          │
└───────────────────────────────────────────────────────────────────────────┘
```

---

## Column 1: Commands (20%)

**Purpose**: Start workflows

**Content**:
```
Workflow:
  > Prompt Claude    <-- PRIMARY: Direct prompt to Claude Code

SDD: (secondary, don't need to work perfectly yet)
  Specify
  Clarify
  Plan
  Tasks
  Implement

Git: (secondary)
  Commit
  Push
```

**Interaction**:
- Arrow keys: Navigate
- Enter: Execute selected command
- All commands except "Prompt Claude" are pre-filled prompts

**Implementation Note**:
All SDD commands are just **different prompts** passed to Claude Code:
```rust
match command {
    "Prompt Claude" => claude_cli(user_input),
    "Specify"       => claude_cli("Generate spec for: <feature>"),
    "Clarify"       => claude_cli("Clarify spec: <spec.md>"),
    "Plan"          => claude_cli("Generate plan: <spec.md>"),
    "Tasks"         => claude_cli("Generate tasks: <plan.md>"),
}
```

---

## Column 2: Content / Interactive (40%)

**Purpose**: Context-aware interaction area

**Shows Based on Context**:

### 1. When "Prompt Claude" Selected
```
┌────────────────────────────┐
│ Enter prompt:              │
│                            │
│ Fix the bug in...          │
│ > _                        │
│                            │
│ (Multi-line input)         │
│ Esc to finish, Enter to    │
│ send                       │
└────────────────────────────┘
```

### 2. When Command Running
```
┌────────────────────────────┐
│ Running: Specify           │
│                            │
│ [████████░░] 80%           │
│                            │
│ Streaming output:          │
│ Sure, let me help with...  │
│                            │
└────────────────────────────┘
```

### 3. When Command Complete
```
┌────────────────────────────┐
│ Complete: Specify          │
│                            │
│ ✓ Spec generated           │
│ ✓ Written to spec.md       │
│                            │
│ Duration: 5.2s             │
│ Exit: 0                    │
└────────────────────────────┘
```

### 4. When Idle / Showing Content
```
┌────────────────────────────┐
│ spec.md                    │
│                            │
│ # Spec: Feature Name       │
│                            │
│ ## What                    │
│ This feature adds...       │
│                            │
│ (Scrollable content)       │
└────────────────────────────┘
```

**Interaction**:
- Arrow keys / PageUp/PageDown: Scroll
- i: Enter input mode (for Prompt Claude)
- Esc: Exit input mode

---

## Column 3: Logs / Journal (40%)

**Purpose**: Detailed, observable, copy-friendly logs

**Content Format**:
```
Session started

[14:23:45] User: Send prompt to Claude
[14:23:46] → Claude CLI
  Command: claude --print --verbose
  Args: --output-format stream-json
        --mcp-config ~/.rstn/mcp-session.json

[14:23:48] ← Streaming response
  Chunk 1/3 received (512 bytes)

[14:23:49] MCP Event
  Tool: rstn_report_status
  Status: needs_input=false

[14:23:50] ← Complete
  Exit code: 0
  Duration: 5.2s

Files changed:
+ spec.md (modified)

Full logs: ~/.rstn/logs/2025-12-18-142345-a3f9.log
```

**What to Log** (Human-Readable):
1. User actions ("User: <action>")
2. Commands executed ("→ Claude CLI")
3. Command details (full command line)
4. Stream events ("← Streaming", "← Complete")
5. MCP tool calls ("MCP Event: rstn_report_status")
6. Exit codes and duration
7. Files changed
8. Link to detailed log file

**Interaction**:
- Arrow keys / PageUp/PageDown: Scroll
- Home: Jump to top
- End: Jump to bottom (latest)
- Auto-scroll to bottom when new log appears

---

## Session ID System

**Format**: `YYYY-MM-DD-HHMMSS-random`

**Example**: `2025-12-18-142345-a3f9`

**Location**:
- Displayed: Top-right of screen
- Log file: `~/.rstn/logs/2025-12-18-142345-a3f9.log`

**Behavior**:
- Auto-generated on rstn start
- Timestamp prefix for chronological sorting
- 4-char random suffix for uniqueness

---

## Two-Tier Logging System

### Tier 1: Log Panel (Column 3) - Human-Readable

**Purpose**: Show essential info for user to understand workflow

**Format**:
```
[HH:MM:SS] Category: Message
  Detail line 1
  Detail line 2
```

**Categories**:
- User: User actions
- →: Outgoing (rstn → Claude Code)
- ←: Incoming (Claude Code → rstn)
- MCP: MCP tool calls
- File: File changes
- Error: Errors

**Example**:
```
[14:23:45] User: Execute "Prompt Claude"
[14:23:46] → Claude CLI
  claude --print "Fix the bug"
[14:23:48] ← Streaming (chunk 1/3)
[14:23:50] ← Complete (exit: 0, 5.2s)
[14:23:50] File: + spec.md
```

### Tier 2: Detailed Log File - Machine-Readable

**Purpose**: Comprehensive debugging info

**Location**: `~/.rstn/logs/<session-id>.log`

**Format**:
```
[YYYY-MM-DD HH:MM:SS.mmm] LEVEL Category: Message
```

**Levels**:
- TRACE: Very detailed (raw JSON, all bytes)
- DEBUG: Detailed (process IDs, full args, raw data)
- INFO: Important events
- WARN: Warnings
- ERROR: Errors

**Example**:
```
[2025-12-18 14:23:45.123] INFO User: Execute "Prompt Claude"
[2025-12-18 14:23:45.124] DEBUG Session: 2025-12-18-142345-a3f9
[2025-12-18 14:23:46.001] DEBUG Command: claude --print "Fix the bug"
[2025-12-18 14:23:46.002] DEBUG Args: ["--print", "--verbose", "--output-format", "stream-json", ...]
[2025-12-18 14:23:46.100] DEBUG Process spawned: PID 12345
[2025-12-18 14:23:46.200] DEBUG MCP server: port 54321, ready
[2025-12-18 14:23:48.001] DEBUG Stream chunk: 512 bytes
[2025-12-18 14:23:48.002] TRACE Raw JSON: {"type":"content_block_start","index":0,...}
[2025-12-18 14:23:50.001] INFO Complete: exit_code=0, duration=5.2s
[2025-12-18 14:23:50.002] DEBUG Stdout: <3.2 KB>
[2025-12-18 14:23:50.003] DEBUG Stderr: <empty>
```

**Why Two Tiers**:
- Panel: User-friendly, quick understanding
- File: Comprehensive, for deep debugging
- Panel links to file for "more details"

---

## Copy Functions

### "y" - Copy Current Pane

**Behavior**: Copies content of currently focused column

**Examples**:
- Focus on Column 1 (Commands): Copies command list
- Focus on Column 2 (Content): Copies displayed content (spec.md, input, output)
- Focus on Column 3 (Logs): Copies all visible logs

**Use Case**: Quick copy of specific content

### "Y" - Copy All Debug Info

**Behavior**: Copies comprehensive debug information in structured format

**Format**:
```
=== rstn Debug Info ===
Session: 2025-12-18-142345-a3f9
Time: 2025-12-18 14:23:50
Feature: 078-worktree-new-command (if on feature branch)
Branch: 078-worktree-new-command

=== Command Executed ===
claude --print --verbose --output-format stream-json \
  --mcp-config ~/.rstn/mcp-session.json \
  "Fix the bug in..."

=== Result ===
Exit code: 0
Duration: 5.2s
Output size: 3.2 KB

=== MCP Events ===
• rstn_report_status (needs_input: false)
• rstn_complete_task (task: T001)

=== Log Excerpt (last 20 lines) ===
[14:23:46] → Claude CLI started
[14:23:48] ← Streaming response
[14:23:50] ← Complete

=== Files Changed ===
+ spec.md (modified)
+ plan.md (created)

=== Environment ===
rstn version: 0.2.0
Claude CLI: available
MCP server: port 54321

Full logs: ~/.rstn/logs/2025-12-18-142345-a3f9.log
```

**Use Case**: Paste entire debug context to Claude Code for help fixing bugs

---

## Observability Requirements

**Why Observability is Critical**:
We don't fully understand how rstn ↔ Claude Code ↔ MCP integration works. We need to **observe** the system to debug and fix it.

### What to Observe:

#### 1. rstn → Claude Code Communication
- Full command line executed
- All arguments passed
- Working directory
- Environment variables (if relevant)
- Process ID

#### 2. Stream Output
- When chunks arrive
- Chunk size
- Total chunks received
- Stream format (JSON structure)
- Any parsing errors

#### 3. MCP Tool Calls
- Which MCP tools are called
- Tool arguments
- Tool responses
- Timing (when called, duration)

#### 4. Claude Code Hooks
- Which hooks fire
- Hook timing
- Hook output

#### 5. File System Changes
- Files created
- Files modified
- Files deleted
- Timestamps

#### 6. Errors
- Where errors occur (rstn, Claude CLI, MCP)
- Error messages (full text)
- Exit codes
- Stack traces (if available)

### Observable Logs Should Answer:
1. "What command was executed?"
2. "What happened during execution?"
3. "What MCP events occurred?"
4. "What files changed?"
5. "Why did it fail?" (if failed)
6. "How long did it take?"

---

## Keyboard Shortcuts

**Global**:
- `q`: Quit
- `?`: Help (show all shortcuts)
- `Tab`: Cycle focus between columns
- `y`: Copy current pane
- `Y`: Copy all debug info

**Navigation**:
- `↑/↓`: Navigate items in focused column
- `←/→`: Switch focus between columns
- `PageUp/PageDown`: Scroll content
- `Home`: Jump to top
- `End`: Jump to bottom

**Commands Column (Column 1)**:
- `Enter`: Execute selected command
- `↑/↓`: Select command

**Content Column (Column 2)**:
- `i`: Enter input mode (for "Prompt Claude")
- `Esc`: Exit input mode
- `Enter`: Submit input (when in input mode)
- `↑/↓`: Scroll content (when not in input mode)

**Logs Column (Column 3)**:
- `↑/↓`: Scroll logs
- `Home`: Jump to top
- `End`: Jump to bottom (latest)
- `f`: Toggle auto-scroll (follow mode)

---

## Removed Features (Simplification)

### 1. Tab Bar - REMOVED
**Old**: 4 tabs (Worktree / MCP Server / Settings / Dashboard)
**New**: Just Worktree view
**Reason**: Other tabs don't work, just noise. Focus on core functionality.

### 2. Feature Info Bar - REMOVED (for now)
**Old**: "Current Feature: 078-worktree-new-command"
**New**: Show in session header if needed, or in logs
**Reason**: Simplify UI, not critical for core workflow

### 3. Multiple Input Dialogs - SIMPLIFIED
**Old**: Different input dialogs for different workflows
**New**: Single inline input in Column 2
**Reason**: Reduce UI complexity, one interaction pattern

---

## Design Principles

### 1. **Observability First**
Every action should be logged in detail. We need to see what's happening to debug the system.

### 2. **Small Steps Forward**
Don't try to handle complicated context. Keep each workflow step simple and observable.

### 3. **Copy-Friendly**
Make it easy to copy information and paste to Claude Code for help. The system should be self-debugging.

### 4. **Context-Aware**
Column 2 (Content) should show the right UI based on what the user is doing. No fixed layout.

### 5. **Human-Readable Logs**
Logs should be understandable without deep technical knowledge. Timestamps, clear categories, structured format.

---

## Implementation Phases

### Phase 1: Layout & Structure (Week 1)
- [ ] Remove tab bar UI
- [ ] Implement three-column layout (20/40/40)
- [ ] Basic keyboard navigation (Tab, arrows, Enter)
- [ ] Session ID generation and display
- [ ] Basic log panel (Column 3)

### Phase 2: Commands & Content (Week 2)
- [ ] Command list in Column 1
- [ ] "Prompt Claude" command (primary)
- [ ] Input UI in Column 2
- [ ] Execute Claude CLI command
- [ ] Show output in Column 2

### Phase 3: Logging & Observability (Week 3)
- [ ] Two-tier logging system
- [ ] Detailed log file (~/.rstn/logs/)
- [ ] Human-readable log panel
- [ ] Log command execution
- [ ] Log MCP events
- [ ] Log file changes

### Phase 4: Copy Functions (Week 4)
- [ ] "y" - Copy current pane
- [ ] "Y" - Copy all debug info
- [ ] Format debug info structure
- [ ] Test copy/paste workflow

### Phase 5: Polish & Testing (Week 5)
- [ ] Auto-scroll logs
- [ ] Progress indicators
- [ ] Error handling
- [ ] Help screen (?)
- [ ] User testing and feedback

---

## Open Questions

### 1. Log Panel Scrolling
**Question**: Should log panel auto-scroll to bottom or stay at user position?

**Options**:
- A: Always auto-scroll (show latest)
- B: Stay at user scroll position
- C: Auto-scroll by default, toggle with `f` key

**Decision**: TBD

### 2. MCP Event Display
**Question**: Show all MCP tool calls or just important ones?

**Options**:
- A: Show all (very detailed)
- B: Show only important (needs_input, errors)
- C: Show all but collapsible

**Decision**: TBD

### 3. Input Mode
**Question**: Should "Prompt Claude" input be multi-line or single-line?

**Options**:
- A: Multi-line (i to enter, Esc to exit, like vim)
- B: Single line (inline editing)
- C: Multi-line with dedicated editor

**Decision**: TBD

---

## Success Metrics

**Goal**: Worktree view is simplified and observable

**Metrics**:
1. **Lines of Code**: Reduce from 4,118 → target <1,000
2. **Fields in State**: Reduce from 54+ → target <20
3. **User Feedback**: Can user execute "Prompt Claude" and understand what happened?
4. **Debugging Time**: Can developer copy logs and debug issues quickly?
5. **Implementation Time**: Can basic workflow be implemented in 2-3 weeks?

---

## Related Documents

- [Technical Debt Analysis](../03-complexity-analysis/technical-debt.md) - Current complexity issues
- [rstn TUI Architecture](rstn-tui-architecture.md) - Current architecture (before redesign)
- [System Overview](overview.md) - High-level system architecture

---

## Changelog

- 2025-12-18: Initial design created based on user discussion
