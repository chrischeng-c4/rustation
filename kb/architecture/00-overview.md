---
title: "Architecture Principles"
description: "Four pillars: state-first, CLI/TUI separation, testing-first, workflow-driven UI"
category: concept
status: evergreen
last_updated: 2025-12-21
version: 0.2.0
tags: []
weight: 0
aliases: ["/02-architecture/core-principles.md"]
---

# Core Principles - rustation v2

**Last Updated**: 2025-12-19
**Status**: Active (v2)

Welcome to rustation v2! This document outlines the core architectural principles that guide all development.

---

## 🎯 The Three Pillars

### 1. State-First Architecture

**Principle**: At any time, rstn's entire state MUST be JSON/YAML serializable.

**Why**: Testability, reproducibility, clarity
**Details**: See [State-First Architecture](01-state-first-principle.md)
**Implementation Pattern (Target)**: See [State-First MVI](02-state-first-mvi.md)

**Key Rules**:
- ✅ All state structs derive `Serialize + Deserialize + Debug + Clone`
- ✅ State is the single source of truth
- ✅ UI = render(state) - pure function
- ❌ No hidden state (closures, globals, thread-locals)
- ❌ No non-serializable types in state

**Testing**:
- Every feature MUST include state round-trip test
- Every feature MUST include state transition tests
- CI enforces: All state structs have required derives

---

### 2. CLI/TUI Separation

**Principle**: CLI and TUI are different **interfaces** over the same **business logic**.

**Why**: Easier testing, code reuse, clear responsibilities

**Architecture**:
```
┌─────────────────────────────────────────────┐
│           Presentation Layer                 │
│   ┌──────────────┐     ┌──────────────┐    │
│   │ CLI          │     │ TUI          │    │
│   │ (commands/)  │     │ (tui/views/) │    │
│   │              │     │              │    │
│   │ - Parse args │     │ - Events     │    │
│   │ - Print stdout│    │ - Rendering  │    │
│   └──────┬───────┘     └──────┬───────┘    │
└──────────┼──────────────────────┼───────────┘
           │                      │
           └──────────┬───────────┘
                      │
         ┌────────────▼───────────┐
         │   Business Logic       │
         │   (runners/, domain/)  │
         │                        │
         │ - Spec generation      │
         │ - Task execution       │
         │ - Git operations       │
         │ - Session management   │
         └────────────────────────┘
```

**Key Rules**:
- ✅ Business logic in `runners/` and `domain/` (NO UI dependencies)
- ✅ CLI and TUI call the same functions
- ✅ Test business logic via CLI first (easier than TUI)
- ❌ Don't duplicate logic between CLI and TUI
- ❌ Don't put business logic in view layer

**Testing Strategy**:
1. Build CLI first → test business logic
2. Build TUI wrapper → test UI behavior only
3. If TUI breaks but CLI works → UI/UX issue only, not logic bug

**Example**:
```rust
// ✅ GOOD: Shared business logic
pub fn run_claude_command_streaming<F>(
    message: &str,
    options: &ClaudeCliOptions,
    output_handler: F, // Generic callback
) -> Result<ClaudeResult>
where F: Fn(ClaudeStreamMessage) -> Result<()>
{
    // ... spawn Claude CLI, parse JSONL ...
    for line in lines {
        let msg = parse_jsonl(&line)?;
        output_handler(msg)?; // ← CLI and TUI use different handlers
    }
}

// CLI: Print to stdout
run_claude_command_streaming(msg, opts, |msg| {
    print!("{}", msg.get_text().unwrap_or_default());
    Ok(())
})

// TUI: Send event
run_claude_command_streaming(msg, opts, |msg| {
    sender.send(Event::ClaudeStream(msg))?;
    Ok(())
})
```

**See Also**: `CLAUDE.md` - "CLI/TUI Architecture Pattern" section

---

### 3. Testing-First Development

**Principle**: Tests define correctness. Write state tests BEFORE implementation.

**Why**: Confidence in changes, regression prevention, design validation

**Testing Hierarchy**:

1. **State Tests** (Primary, 70%):
   - Round-trip serialization
   - State transitions
   - State invariants
   - Observable, stable, fast

2. **Integration Tests** (Secondary, 20%):
   - CLI command tests
   - Business logic flows
   - Error handling

3. **UI Tests** (Minimal, 10%):
   - Layout regressions
   - Widget rendering
   - Mouse/keyboard events (via tui-tester)

**Key Rules**:
- ✅ State test for every feature (MANDATORY)
- ✅ Test state transitions, not UI coordinates
- ✅ Use builders for test state setup
- ❌ Don't skip tests ("will add later")
- ❌ Don't test implementation details

**Example**:
```rust
// ✅ GOOD: State-based test
#[test]
fn test_prompt_workflow() {
    let mut app = App::from_state(AppState::default()).unwrap();

    app.handle_action(ViewAction::RunPromptClaude {
        prompt: "test".into()
    });

    let state = app.to_state();
    assert!(state.worktree_view.active_session_id.is_some());
    assert_eq!(state.worktree_view.pending_follow_up, false);
}

// ❌ BAD: UI coordinate test
#[test]
fn test_prompt_ui() {
    let mut terminal = TestBackend::new(80, 24);
    app.render(&mut terminal);
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.get(10, 5).symbol, "│"); // Fragile!
}
```

### 4. Workflow-Driven UI

**Principle**: The TUI is a **Workflow Launcher**, not just a static viewer.

**Why**: Focuses user attention on task execution, reduces visual clutter.

**Architecture**:
```
┌─────────────────────────────────────────────┐
│              TUI Layout                     │
│   ┌──────────────┐   ┌──────────────────┐   │
│   │ Commands     │   │ Dynamic Content  │   │
│   │ (Triggers)   │   │ (Active Node)    │   │
│   │              │   │                  │   │
│   │ ▶ Prompt     │   │  [Input Dialog]  │   │
│   │   Commit     │   │       OR         │   │
│   │   ...        │   │  [Stream View]   │   │
│   └──────────────┘   └──────────────────┘   │
└─────────────────────────────────────────────┘
```

**Key Rules**:
- ✅ **Command as Trigger**: Left panel lists workflows (e.g., "Prompt Claude"), not navigation menus.
- ✅ **Dynamic Content**: Right panel visualizes the *current node* of the active workflow.
- ✅ **Minimalism**: No permanent "Log Panel" (logs go to file). Top "View Tabs" allowed for high-level navigation (Workflows/Dockers/Settings).
- ✅ **Agent Integration**: AI agents are invoked only when the workflow reaches a specific node (e.g., `Node::CallAgent`).

---

## Design Philosophy

### Simplicity Over Completeness
- Start with minimal viable solution
- Add complexity only when needed
- YAGNI: You Aren't Gonna Need It
- Delete code aggressively

### Observability Over Cleverness
- Prefer explicit over implicit
- Log state transitions
- Make debugging easy
- Copy-friendly error messages

### Small Modules Over Large Classes
- Target: <500 lines per file
- Target: <15 fields per struct
- Extract when growing
- Composition over inheritance

### Evolution Over Perfection
- Ship working code, iterate
- Refactor when needed
- Don't over-engineer
- v1 taught us what NOT to do

---

## Anti-Principles (v1 Mistakes)

### ❌ God Classes
**v1 Problem**: App (3,404 lines), WorktreeView (4,118 lines)
**v2 Solution**: Small modules, clear responsibilities

### ❌ State Explosion
**v1 Problem**: 54+ mutable fields in WorktreeView
**v2 Solution**: Serializable state, <15 fields per struct

### ❌ Tight Coupling
**v1 Problem**: TUI layer contains business logic
**v2 Solution**: CLI/TUI separation, shared business logic

### ❌ Testing Gaps
**v1 Problem**: 40% coverage, fragile UI tests
**v2 Solution**: State-first testing, 70%+ target

---

## Enforcement

### Code Review Checklist
- [ ] New state structs derive `Serialize + Deserialize + Debug + Clone`
- [ ] State round-trip test included
- [ ] State transition tests included
- [ ] No business logic in view layer
- [ ] CLI and TUI share business logic (if applicable)
- [ ] Module <500 lines, struct <15 fields

### CI Checks (Future)
- [ ] Enforce trait derives on state structs
- [ ] Test coverage >70%
- [ ] Clippy clean
- [ ] All state tests pass

### PR Approval Requirements
- State tests MUST be included
- No direct merge without review
- Breaking changes require discussion

---

## Quick Reference

| Principle | Key Rule | Example |
|-----------|----------|---------|
| **State-First** | All state serializable | `#[derive(Serialize, Deserialize)]` |
| **CLI/TUI Separation** | Shared business logic | `runners/cargo.rs` used by both |
| **Testing-First** | State tests mandatory | `#[test] fn test_state_transition()` |

---

## Related Documents

- [System Requirements & High-Level Design](09-system-requirements.md) - User requirements mapping
- [State-First Architecture](01-state-first-principle.md) - Deep dive into principle #1
- [SDD Workflow](../02-how-to-guides/sdd-workflow.md) - Development process
- [CLAUDE.md](../../CLAUDE.md) - CLI/TUI architecture pattern

---

## Changelog

- 2025-12-19: Initial v2 core principles document created
