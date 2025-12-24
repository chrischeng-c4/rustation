---
title: "State Testing Patterns"
description: "Round-trip tests, transition tests, and invariant validation"
category: concept
status: evergreen
last_updated: 2025-12-21
version: 0.2.0
tags: [testing, state, round-trip, invariants]
weight: 12
---

## Implementation Requirements

### 1. State Structs Must Be Serializable

All state structs **MUST** derive:
- `Serialize` (serde)
- `Deserialize` (serde)
- `Debug` (for logging)
- `Clone` (for snapshotting)

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub version: String,
    pub current_view: ViewType,
    pub input_mode: bool,
    pub worktree_view: WorktreeViewState,
    // ... all app state
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeViewState {
    pub focus: WorktreeFocus,
    pub current_branch: Option<String>,
    pub git_status: Vec<GitStatusEntry>,
    pub session_id: Option<String>,
    pub claude_output: Vec<OutputLine>,
    // ... all view state
}
```

### 2. Bidirectional Conversion

Apps/Views must implement `to_state()` and `from_state()`:

```rust
impl App {
    /// Convert app to serializable state
    pub fn to_state(&self) -> AppState {
        AppState {
            version: env!("CARGO_PKG_VERSION").to_string(),
            current_view: self.current_view,
            input_mode: self.input_mode,
            worktree_view: self.worktree_view.to_state(),
            settings_view: self.settings_view.to_state(),
            mcp_server: self.mcp_state.lock().unwrap().to_state(),
        }
    }

    /// Reconstruct app from state
    pub fn from_state(state: AppState) -> Result<Self> {
        Ok(App {
            current_view: state.current_view,
            input_mode: state.input_mode,
            worktree_view: WorktreeView::from_state(state.worktree_view)?,
            settings_view: SettingsView::from_state(state.settings_view)?,
            mcp_state: Arc::new(Mutex::new(McpState::from_state(state.mcp_server)?)),
            // ...
        })
    }
}
```

### 3. No Hidden State

**Forbidden**:
- State in `Rc<RefCell<T>>` that's not serialized
- State in closures
- State in thread-local storage
- Implicit state in external systems (without serializing references)

**Required**:
- All state in the state tree
- External references stored as IDs (e.g., `session_id: String`)
- Transient state marked clearly (e.g., `#[serde(skip)]` for caches)

---

## Testing Requirements

### Mandatory Tests for All Features

Every feature **MUST** include:

#### 1. State Serialization Round-Trip Test
```rust
#[test]
fn test_state_serialization_round_trip() {
    let app = App::new_for_test();

    // Serialize
    let state = app.to_state();
    let json = serde_json::to_string(&state).unwrap();

    // Deserialize
    let loaded_state: AppState = serde_json::from_str(&json).unwrap();
    let loaded_app = App::from_state(loaded_state).unwrap();

    // Must be identical
    assert_eq!(app.to_state(), loaded_app.to_state());
}
```

#### 2. State Transition Tests
```rust
#[test]
fn test_feature_state_transitions() {
    // Load initial state
    let initial = AppState {
        input_mode: false,
        session_id: None,
        // ...
    };
    let mut app = App::from_state(initial).unwrap();

    // Trigger action
    app.handle_view_action(ViewAction::RunPromptClaude {
        prompt: "test".into()
    });

    // Assert state changed correctly
    let final_state = app.to_state();
    assert!(final_state.session_id.is_some());
    assert_eq!(final_state.input_mode, false);
    assert!(final_state.worktree_view.pending_response);
}
```

#### 3. State Invariant Tests
```rust
#[test]
fn test_state_invariants() {
    let state = app.to_state();

    // Invariants that must always hold:

    // Example: input_dialog is Some iff input_mode is true
    if state.input_mode {
        assert!(state.input_dialog.is_some());
    } else {
        assert!(state.input_dialog.is_none());
    }

    // Example: session_id exists iff claude_output is non-empty
    if !state.worktree_view.claude_output.is_empty() {
        assert!(state.worktree_view.session_id.is_some());
    }
}
```

---

## Comparison: State Testing vs UI Testing

| Aspect | State Testing | UI Testing |
|--------|---------------|------------|
| **Observability** | ✅ Direct field access | ❌ Parse TestBackend buffer |
| **Stability** | ✅ Stable APIs | ❌ Breaks on layout changes |
| **Speed** | ✅ Fast (no rendering) | ❌ Slow (full render cycle) |
| **Clarity** | ✅ `assert_eq!(app.input_mode, true)` | ❌ `buffer.get(10, 5).symbol == "│"` |
| **Maintainability** | ✅ Refactor-safe | ❌ Brittle coordinate checks |
| **Determinism** | ✅ Pure state transitions | ❌ Depends on terminal size |

**When to use each**:
- **State testing**: Business logic, workflows, state transitions (95% of tests)
- **UI testing**: Layout bugs, visual regressions, widget rendering (5% of tests)

---

## CLI vs TUI: Same Core, Different Interfaces

Both CLI and TUI are **interfaces** over the same **state**:

```
┌────────────────────────────────────────────┐
│         Presentation Layer                 │
│  ┌──────────────┐    ┌──────────────┐     │
│  │ CLI Interface│    │ TUI Interface│     │
│  │              │    │              │     │
│  │ - Parse args │    │ - Events     │     │
│  │ - Print stdout│   │ - Rendering  │     │
│  └──────┬───────┘    └──────┬───────┘     │
└─────────┼────────────────────┼─────────────┘
          │                    │
          └─────────┬──────────┘
                    │
        ┌───────────▼────────────┐
        │    Core State Logic    │
        │  (runners/cargo.rs)    │
        │                        │
        │  - Business logic      │
        │  - State transitions   │
        │  - Side effects        │
        └────────────────────────┘
```

**Testing Strategy**:
1. **Test state via CLI first** (easy: input → output)
2. **Then test TUI** (only UI behavior, core already validated)
3. **Result**: If CLI passes, TUI issues are UI/UX only

---

## File Structure

```
crates/rstn/src/
├── tui/
│   ├── state.rs           # ← NEW: AppState, WorktreeViewState, etc.
│   ├── app.rs             # App::to_state(), App::from_state()
│   └── views/
│       ├── worktree/
│       │   ├── view.rs    # WorktreeView::to_state()
│       │   └── state.rs   # ← NEW: WorktreeViewState
│       └── ...
└── commands/
    └── state.rs           # ← NEW: CLI for --save-state, --load-state
```

---

## Enforcement

### CI Checks
- All state structs derive `Serialize + Deserialize`
- Round-trip tests pass for all views
- State coverage > 80% (all fields tested)

### Code Review Checklist
- [ ] New features include state tests
- [ ] State structs are serializable
- [ ] `to_state()` and `from_state()` implemented
- [ ] No hidden state introduced
- [ ] State invariants documented and tested

### SDD Workflow
When creating plan.md for new features:
- **Task N**: Define state structs
- **Task N+1**: Implement state tests
- **Task N+2**: Implement feature logic

State tests come **before** implementation.

---

## Future Enhancements

### Phase 1 (Feature 079)
- [x] Document principle in KB
- [ ] Create `tui/state.rs` with AppState
- [ ] Implement `App::to_state()` / `from_state()`
- [ ] Add state tests for existing features
- [ ] Update CLAUDE.md with testing requirements

### Phase 2 (Feature 080)
- [ ] Session persistence (`~/.rstn/session.yaml`)
- [ ] `--save-state` / `--load-state` CLI flags
- [ ] State changelog for version migrations

### Phase 3 (Future)
- [ ] Time-travel debugging (record all transitions)
- [ ] State diff viewer (before/after comparisons)
- [ ] State replay for bug reproduction

---

## References

- **Similar Architectures**:
  - Elm Architecture (Model is serializable)
  - Redux (State tree is plain JSON)
  - Jupyter Notebooks (Serialize execution state)
  - Emacs (Save/restore sessions)

- **Related KB Docs**:
  - `kb/01-architecture/rstn-tui-architecture.md` - TUI design
  - `kb/04-sdd-workflow/when-to-use-which.md` - Testing strategy
  - `CLAUDE.md` - CLI/TUI Architecture Pattern

---

**Key Takeaway**: State is the single source of truth. UI is derived. Tests assert state, not UI.

---

## Changelog

- **2025-12-19**: Added comprehensive "State Persistence" section with 7 subsections answering practical implementation questions:
  1. State file locations (automatic session.yaml + CLI flags)
  2. File format and schema (YAML/JSON auto-detection, 52-field AppState structure)
  3. Default state initialization (Default trait implementations, defensive deserialization)
  4. Error handling (corrupted files, version checking, post-load validation)
  5. State validation (StateInvariants trait with assert_invariants())
  6. State-to-UI flow (bidirectional conversion, pure rendering)
  7. Logging management (two-tier architecture: file logging + TUI event buffer, State + Logs = Observability principle)
