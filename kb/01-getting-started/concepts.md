# Core Concepts

**Last Updated**: 2025-12-19
**Estimated Time**: 10 minutes

This guide introduces the core concepts behind rustation v2 and explains what makes it different.

---

## What is rustation v2?

**rustation** is a Rust-based development workflow tool that helps you manage software projects through **Specification-Driven Development (SDD)**.

### Key Features

- **📋 Specification-Driven Development**: Define WHAT before implementing HOW
- **🎯 State-First Architecture**: All application state is observable and testable
- **🖥️ Dual Interface**: Use CLI for scripting, TUI for interactive workflows
- **🔌 Claude Code Integration**: Built-in MCP server for AI assistant collaboration
- **📊 Session Management**: Track and resume development sessions

### What's New in v2?

**v2 is a fresh start** with lessons learned from v1:

| v1 (Archived) | v2 (Current) |
|---------------|--------------|
| Complex state management | ✅ State-first architecture |
| God classes (3,000+ lines) | ✅ Small modules (<500 lines) |
| Fragile UI tests | ✅ State-based testing |
| Tight coupling | ✅ CLI/TUI separation |
| 40% test coverage | ✅ 70%+ target coverage |

**Bottom line**: v2 is simpler, more testable, and easier to maintain.

---

## State-First Architecture

### The Core Principle

**At any time, rstn's entire state MUST be JSON/YAML serializable.**

This means you can:
- **Save state**: `rstn --save-state snapshot.json`
- **Load state**: `rstn --load-state snapshot.json`
- **Reproduce bugs**: Exact same state → same behavior
- **Test easily**: State in → actions → state out

### Why This Matters

**Before (v1 approach)**:
```rust
// ❌ Hidden state, hard to test
struct App {
    callback: Box<dyn Fn()>,  // Not serializable
    tx: mpsc::Sender<Event>,  // Not serializable
    // ... you can't save this!
}
```

**After (v2 approach)**:
```rust
// ✅ Observable state, easy to test
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppState {
    current_view: ViewType,
    worktree_path: PathBuf,
    active_session_id: Option<String>,
    // ... you can save/load this!
}

// UI is just a pure function
fn render(state: &AppState) -> Frame { /* ... */ }
```

**Benefits**:
- **Debugging**: Save buggy state → load → reproduce instantly
- **Testing**: Write state → verify transitions → assert final state
- **Refactoring**: Change UI freely, tests still pass (state unchanged)

### Example: Testing a Feature

**Old way (UI tests)**:
```rust
// ❌ Fragile - breaks on layout changes
#[test]
fn test_click_tab() {
    let mut terminal = TestBackend::new(80, 24);
    app.render(&mut terminal);
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.get(10, 5).symbol, "│"); // What if we resize?
}
```

**New way (state tests)**:
```rust
// ✅ Robust - tests behavior, not UI
#[test]
fn test_switch_view() {
    let mut app = App::from_state(AppState::default()).unwrap();

    app.handle_action(ViewAction::SwitchToSettings);

    let final_state = app.to_state();
    assert_eq!(final_state.current_view, ViewType::Settings);
}
```

**See Also**: [State-First Architecture](../02-architecture/state-first.md) for deep dive

---

## CLI vs TUI: Two Interfaces, Same Logic

rustation provides **two ways** to interact with the same underlying system:

### CLI (Command-Line Interface)

**Use when**:
- Scripting and automation
- CI/CD pipelines
- Quick one-off commands
- Testing business logic

**Example**:
```bash
# Direct command execution
rstn prompt "Add authentication feature" --max-turns 5

# Scriptable output
rstn spec generate --feature auth > spec.md

# Session continuation
rstn prompt "Refine the spec" --continue-session --session-id abc123
```

**Characteristics**:
- Non-interactive (args → execute → output)
- Pipeable (stdout/stderr)
- Exit codes (0 = success, non-zero = error)
- Fast (no UI rendering)

### TUI (Text User Interface)

**Use when**:
- Interactive workflows
- Visual feedback needed
- Exploring/browsing data
- Multi-step processes

**Example**:
- Launch: `rstn` (no args)
- Navigate with keyboard/mouse
- Visual progress indicators
- Input dialogs for user prompts

**Characteristics**:
- Interactive (continuous event loop)
- Visual (tabs, panels, dialogs)
- Stateful (maintains context between actions)
- User-friendly (discoverability)

### Shared Business Logic

**Key insight**: CLI and TUI are just **interfaces** over the same **core logic**.

```
┌─────────────────────────────────────────┐
│       Presentation Layer                │
│  ┌─────────┐       ┌─────────┐         │
│  │   CLI   │       │   TUI   │         │
│  │ (args)  │       │ (events)│         │
│  └────┬────┘       └────┬────┘         │
└───────┼─────────────────┼──────────────┘
        │                 │
        └────────┬────────┘
                 │
        ┌────────▼────────┐
        │ Core Business   │
        │ Logic (shared)  │
        │                 │
        │ - Spec gen      │
        │ - Task exec     │
        │ - Claude CLI    │
        │ - Sessions      │
        └─────────────────┘
```

**Benefits**:
- Test business logic via CLI (easier)
- Add UI wrapper with TUI (user-friendly)
- Single source of truth (no duplication)
- Bug fixes apply to both interfaces

**See Also**: [Core Principles](../02-architecture/core-principles.md#cli-tui-separation)

---

## Specification-Driven Development (SDD)

### The Big Picture

SDD is a workflow that separates **WHAT** (requirements) from **HOW** (implementation).

**Workflow**:
```
1. Specify (WHAT)
   ↓
   Create spec.md: Requirements, user stories, acceptance criteria

2. Plan (HOW)
   ↓
   Create plan.md: Architecture, component design, file structure

3. Tasks (TODO)
   ↓
   Create tasks.md: Phased breakdown, dependencies, checkpoints

4. Implement (DO)
   ↓
   Execute tasks: Write code, tests, verify against spec

5. Review (VERIFY)
   ↓
   Verify implementation matches spec, all tasks complete
```

### Two Variants

rstn supports **two SDD workflows** depending on complexity:

#### Full SDD (Complex Features)

**When**: >500 LOC, >5 files, architecture changes, new domain concepts

**Artifacts**:
```
specs/{NNN}-{name}/
├── spec.md       (WHAT - requirements)
├── plan.md       (HOW - architecture)
└── tasks.md      (TODO - task breakdown)
```

**Commands**:
```bash
/speckit.specify   # Create spec.md
/speckit.plan      # Create plan.md
/speckit.tasks     # Create tasks.md
/speckit.implement # Execute tasks
/speckit.review    # Verify against spec
```

**Use case**: New features, refactoring, multi-week projects

#### Lightweight SDD (Simple Changes)

**When**: <200 LOC, <3 files, straightforward implementation, UI changes

**Artifacts**:
```
specs/{NNN}-{name}/
└── spec.md       (WHAT only - simplified)
```

**Commands**:
```bash
/speckit-lite     # Create simplified spec.md
# Then implement directly (no plan/tasks phase)
```

**Use case**: Bug fixes, small UI improvements, config changes

### Decision Tree

```
New work?
│
├─► >500 LOC or >5 files?
│   └─ YES → Full SDD
│
├─► Architecture change?
│   └─ YES → Full SDD
│
├─► New state structs?
│   └─ YES → Full SDD
│
└─► Otherwise → Lightweight SDD
```

**See Also**: [SDD Workflow Guide](../04-development/sdd-workflow.md) for detailed decision matrix

---

## Sessions and Persistence

### What is a Session?

A **session** represents a coherent unit of work:
- Specification generation
- Task execution
- Claude Code interaction
- Code changes

Each session has:
- **Unique ID**: e.g., `abc123`
- **Feature number**: e.g., `079-state-first`
- **Metadata**: Timestamps, status, artifacts
- **State snapshots**: Saved at checkpoints

### Session Lifecycle

```
┌─────────────┐
│   Create    │ ← /speckit.specify (new feature)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Active    │ ← Executing tasks, prompting Claude
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Completed  │ ← All tasks done, PR merged
└─────────────┘
```

### Continue a Session

**Via CLI**:
```bash
# Resume previous session
rstn prompt "Continue feature" --continue-session --session-id abc123
```

**Via TUI**:
1. Navigate to Dashboard view
2. Select session from history
3. Press Enter to resume

**State persistence** ensures exact continuation:
- Session context restored
- Previous messages included
- Feature metadata available

---

## MCP (Model Context Protocol)

### What is MCP?

MCP is how **rstn** communicates with **Claude Code** (AI assistant).

**Before MCP** (fragile):
```
rstn writes status to stdout
  ↓
Claude Code parses text (error-prone)
  ↓
May miss/misparse status
```

**After MCP** (robust):
```
rstn starts HTTP server
  ↓
Claude Code calls MCP tools
  ↓
Structured JSON-RPC (reliable)
```

### Available MCP Tools

| Tool | Purpose | Example |
|------|---------|---------|
| `rstn_report_status` | Report status changes | `{"status": "needs_input"}` |
| `rstn_read_spec` | Read spec artifacts | `{"artifact": "spec"}` |
| `rstn_get_context` | Get feature context | `{}` (returns metadata) |
| `rstn_complete_task` | Mark task done | `{"task_id": "T001"}` |

### How It Works

```
┌─────────────────────────────────────────┐
│          rstn (Main Process)            │
│                                         │
│  ┌────────┐      ┌──────────────┐     │
│  │  TUI   │◄─────┤ Axum Server  │◄────┼─── HTTP POST /mcp
│  │ Loop   │      │ (port 19560) │     │
│  └────────┘      └──────────────┘     │
└─────────────────────────────────────────┘
                     ▲
                     │ MCP Tools
                     │
          ┌──────────┴──────────┐
          │  Claude Code        │
          │  (subprocess)       │
          └─────────────────────┘
```

**Configuration** (`~/.rstn/mcp-session.json`):
```json
{
  "mcpServers": {
    "rstn": {
      "type": "http",
      "url": "http://127.0.0.1:19560/mcp"
    }
  }
}
```

**See Also**: [MCP Tools Reference](../03-api-reference/mcp-tools.md)

---

## Key Terminology

| Term | Definition | Example |
|------|------------|---------|
| **State** | JSON/YAML serializable application data | `AppState { current_view: Worktree }` |
| **View** | Screen in the TUI (Worktree, Settings, Dashboard) | `ViewType::Settings` |
| **Session** | Unit of work with unique ID and metadata | `abc123-feature-079` |
| **Spec** | Specification document (WHAT to build) | `specs/079-state-first/spec.md` |
| **Plan** | Architecture document (HOW to build) | `specs/079-state-first/plan.md` |
| **Tasks** | Task breakdown (TODO list) | `specs/079-state-first/tasks.md` |
| **MCP** | Model Context Protocol (rstn ↔ Claude Code) | HTTP server on port 19560 |
| **CLI** | Command-line interface (args → output) | `rstn prompt "message"` |
| **TUI** | Text user interface (interactive, visual) | `rstn` (no args) |
| **SDD** | Specification-Driven Development | Specify → Plan → Tasks → Implement |

---

## Mental Model

Think of rstn as:

```
┌────────────────────────────────────────────────────────┐
│                    Your Project                        │
│                                                        │
│  ┌──────────────────────────────────────────────┐    │
│  │           Specification Layer                 │    │
│  │   (WHAT to build - specs/*.md)               │    │
│  └────────────────┬─────────────────────────────┘    │
│                   │                                   │
│                   ▼                                   │
│  ┌──────────────────────────────────────────────┐    │
│  │           Planning Layer                     │    │
│  │   (HOW to build - plan.md, tasks.md)        │    │
│  └────────────────┬─────────────────────────────┘    │
│                   │                                   │
│                   ▼                                   │
│  ┌──────────────────────────────────────────────┐    │
│  │           Implementation Layer               │    │
│  │   (actual code - src/*.rs)                  │    │
│  └──────────────────────────────────────────────┘    │
│                                                        │
│  rstn manages the top two layers,                    │
│  you (+ Claude Code) implement the bottom layer.     │
└────────────────────────────────────────────────────────┘
```

**rstn's job**: Keep specs, plans, and implementation aligned

**Your job**: Write code that matches the spec

**Claude Code's job**: Help generate specs, plans, and code

---

## Design Philosophy

### Simplicity Over Completeness
- Start minimal, add only when needed
- YAGNI (You Aren't Gonna Need It)
- Delete aggressively

### Observability Over Cleverness
- Explicit > Implicit
- Log state transitions
- Copy-friendly error messages

### Small Modules Over God Classes
- Target: <500 lines per file
- Target: <15 fields per struct
- Composition > Inheritance

### Evolution Over Perfection
- Ship working code, iterate
- Refactor when needed
- Don't over-engineer

**Lesson from v1**: Complexity creeps in. v2 fights back with strict size limits and state-first testing.

---

## What Makes v2 Different?

### v1 (2024, Archived)

**Problems**:
- God classes (WorktreeView: 4,118 lines, 54+ fields)
- Hidden state (closures, thread-locals)
- Fragile tests (UI coordinate assertions)
- Tight coupling (business logic in UI layer)

**Result**: Hard to test, hard to refactor, hard to maintain

### v2 (2025, Current)

**Solutions**:
- State-first architecture (all state serializable)
- Small modules (<500 lines)
- State-based testing (70%+ coverage target)
- CLI/TUI separation (shared business logic)

**Result**: Easy to test, easy to refactor, easy to maintain

**Archive**: See `kb/99-archive/` for v1 analysis and lessons learned

---

## Learning Path

### I'm a New User

1. ✅ [Install rstn](installation.md)
2. ✅ [Run your first session](quick-start.md)
3. ✅ **Understand concepts** (you're here!)
4. → [Try SDD workflow](../04-development/sdd-workflow.md)

### I'm a Contributor

1. ✅ Understand state-first architecture (above)
2. → [Read core principles](../02-architecture/core-principles.md)
3. → [Learn state-first details](../02-architecture/state-first.md)
4. → [Follow SDD workflow](../04-development/sdd-workflow.md)
5. → [Write state tests](../04-development/sdd-workflow.md#v2-state-first-testing-requirements)

### I'm Integrating with Claude Code

1. ✅ Understand MCP concepts (above)
2. → [MCP Tools Reference](../03-api-reference/mcp-tools.md)
3. → [Claude CLI Reference](../03-api-reference/claude-cli.md)
4. → [Headless Mode Patterns](../03-api-reference/claude-headless.md)

---

## Common Questions

### Q: Why JSON/YAML serialization?

**A**: Three reasons:
1. **Testability**: State in → actions → state out (deterministic)
2. **Reproducibility**: Save buggy state → load → instant repro
3. **Clarity**: If you can't serialize it, it's probably too complex

### Q: Why separate CLI and TUI?

**A**: Testing and reuse:
- Test business logic via CLI (simple)
- Add UI layer with TUI (user-friendly)
- Bug fixes apply to both

### Q: When do I use Full vs Lightweight SDD?

**A**: Simple rule:
- **Full SDD**: >500 LOC, >5 files, architecture changes
- **Lightweight SDD**: <200 LOC, <3 files, straightforward

### Q: What if I don't want to use MCP?

**A**: You can still use rstn:
- CLI mode works without MCP
- TUI works standalone
- MCP is optional (enhances Claude Code integration)

### Q: Is v1 still supported?

**A**: No, v1 is archived (2025-12-19):
- All v1 specs moved to `specs/archive/`
- v1 analysis in `kb/99-archive/`
- No bug fixes or features
- v2 is the current version

---

## Next Steps

You now understand the core concepts! Choose your path:

**As a User**:
1. [Try interactive workflows in the TUI](quick-start.md)
2. [Learn SDD workflow basics](../04-development/sdd-workflow.md)

**As a Contributor**:
1. [Read state-first architecture](../02-architecture/state-first.md)
2. [Understand core principles](../02-architecture/core-principles.md)
3. [Learn testing requirements](../04-development/sdd-workflow.md#v2-state-first-testing-requirements)

**For Claude Code Integration**:
1. [MCP Tools Reference](../03-api-reference/mcp-tools.md)
2. [Claude CLI Flags](../03-api-reference/claude-cli.md)

---

## Summary

**rustation v2 in 60 seconds**:

- **What**: SDD workflow tool (specify → plan → tasks → implement)
- **How**: State-first architecture (all state = JSON/YAML)
- **Why**: Testability, reproducibility, maintainability
- **Interfaces**: CLI (scripting) + TUI (interactive)
- **Integration**: MCP server for Claude Code
- **Philosophy**: Simple, observable, small modules

**Core principle**: If you can't serialize your state, you're doing it wrong.

---

## Changelog

- 2025-12-19: Initial concepts guide for v2
