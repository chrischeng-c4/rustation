---
title: "State-First MVI (Unidirectional Data Flow)"
description: "A concrete, testable runtime model for state-first: Msg → Reduce → State → Render (+ serializable Effects)"
category: concept
status: active
last_updated: 2025-12-22
version: 0.2.0
tags: [architecture, state, testing, mvi, reducer]
weight: 4
---

# State-First MVI (Unidirectional Data Flow)

This document defines the **target** app runtime model for rstn that makes the state-first principle **executable**:

> **All state mutations happen through a pure reducer.**  
> Side effects are modeled explicitly as **serializable Effects**.

This is an MVI/Redux-style pattern adapted to rstn’s hard requirement:

> **At any moment, the entire app state must be JSON/YAML serializable.**

---

## Why this exists

State-first currently specifies the *property* (“state is serializable”) but needs a concrete *mechanism* for how state changes happen so we can:
- test state transitions deterministically
- make async/streaming workflows robust
- avoid hidden state leaking into the TUI event loop
- support multi-instance (multiple rstn processes) without fragile globals

---

## Core loop

The app runs a single unidirectional loop:

```mermaid
flowchart LR
  Event[External Event\n(key/mouse/tick/mcp/agent)] --> Msg[AppMsg]
  Msg --> Reduce[reduce(state, msg)]
  Reduce --> State[New AppState]
  Reduce --> Effects[Vec<AppEffect>]
  Effects --> Exec[EffectExecutor\n(IO/async/spawn)]
  Exec --> Msg2[AppMsg (results)]
  Msg2 --> Reduce
  State --> Render[render(state)]
```

Key rule: **`reduce` is pure** (no IO, no async, no channels, no time reads).

---

## Terminology

### `AppState`
The single source of truth (must be serializable).
- includes UI state, workflow state, selection, scroll, etc.
- includes *references* to runtime tasks (as IDs), not task handles

### `AppMsg`
The only way to request a state change.
Examples:
- `KeyPressed`, `MouseClicked`, `Tick`
- `WorkflowStartRequested { workflow_def_id }`
- `AgentStreamDelta { workflow_run_id, text }`
- `McpNeedsInput { prompt_id, prompt }`
- `EffectCompleted { effect_id, outcome }`
- `CopyContentRequested`
- `CopyStateRequested`

### `AppEffect`
A **serializable** description of work to do outside the reducer.
Examples:
- `SpawnAgent { workflow_run_id, agent_kind, prompt, mcp_config_path }`
- `WriteFile { path, contents }`
- `McpReply { request_id, result }`
- `StartTimer { timer_id, delay_ms }`
- `Cancel { cancel_token_id }`
- `CopyToClipboard { content }`

### `EffectExecutor`
Non-serializable runtime context (process handles, channels, tokio tasks, file IO).
- owns maps like `cancel_token_id -> JoinHandle/AbortHandle`
- produces `AppMsg` results back into the loop

---

## State-first constraints (hard rules)

1. **`AppState` is JSON/YAML serializable**
   - no closures, no channels, no JoinHandles, no Mutex guards, no file descriptors
2. **`reduce()` is pure**
   - no filesystem, no network, no time calls, no randomness
3. **Effects are explicit and serializable**
   - if something “happens”, it must be either a state change or an effect
4. **Single active workflow is a state invariant**
   - if policy is “only one active workflow at a time”, enforce it in the reducer

---

## Minimal API shape (conceptual)

```rust
pub fn reduce(state: AppState, msg: AppMsg) -> (AppState, Vec<AppEffect>);

pub trait EffectExecutor {
    fn spawn(&mut self, effect: AppEffect);
}
```

Implementation detail: executor runs effects and feeds their results back as `AppMsg`.

---

## Testing strategy (what becomes easy)

MVI makes testing match the state-first philosophy:

1. **Round-trip serialization**
   - `serde_yaml` / `serde_json` for `AppState`
2. **State transition tests**
   - `reduce(s0, msg) -> (s1, effects)` assertions
3. **State invariant tests**
   - e.g. “at most one active workflow”, “no orphaned prompt_id”
4. **Effect planning tests**
   - verify the reducer emits expected effects (without executing them)

This avoids brittle tests that depend on UI coordinates or async scheduling.

---

## How workflows fit

Workflows are **substates** inside `AppState`, not separate control flows.

- `workflow_def_id` = definition (“prompt-claude”, “specify”, “plan”, …)
- `workflow_run_id` = one execution instance of a workflow (a “wf session”)

