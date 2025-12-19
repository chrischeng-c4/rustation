# Testing Guide

**Last Updated**: 2025-12-19
**Version**: v2 (state-first testing)

This guide explains how to write tests for rustation v2, with emphasis on **state-first testing** principles.

---

## Testing Philosophy

### v2 Testing Hierarchy

```
┌──────────────────────────────────────┐
│   State Tests (Primary, 70%)         │  ← Observable, stable, fast
│   - Round-trip serialization          │
│   - State transitions                 │
│   - State invariants                  │
└──────────────────────────────────────┘
           ▲
           │
┌──────────────────────────────────────┐
│   Integration Tests (Secondary, 20%) │  ← Business logic flows
│   - CLI command tests                 │
│   - Multi-step workflows              │
│   - Error handling                    │
└──────────────────────────────────────┘
           ▲
           │
┌──────────────────────────────────────┐
│   UI Tests (Minimal, 10%)            │  ← Layout, rendering only
│   - Widget rendering                  │
│   - Mouse/keyboard events             │
│   - Layout regressions                │
└──────────────────────────────────────┘
```

**Key Insight**: Test **state**, not **UI**. State tests are:
- **Observable**: Serialize state → inspect JSON
- **Stable**: Don't break on UI changes
- **Fast**: No rendering, no event loops
- **Deterministic**: Same input → same output

---

## State-First Testing Principles

### Principle 1: All State is Serializable

**Rule**: If you can't serialize it, you can't test it properly.

**Example**:
```rust
// ✅ GOOD: Serializable state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    pub current_view: ViewType,
    pub worktree_path: PathBuf,
    pub active_session_id: Option<String>,
}

// ❌ BAD: Non-serializable state
pub struct AppState {
    callback: Box<dyn Fn()>,      // Can't serialize!
    sender: mpsc::Sender<Event>,  // Can't serialize!
}
```

### Principle 2: Test State Transitions, Not Implementation

**Rule**: Test **what changed**, not **how it changed**.

**Example**:
```rust
// ✅ GOOD: Test state change
#[test]
fn test_switch_view() {
    let mut app = App::from_state(AppState::default()).unwrap();

    app.handle_action(ViewAction::SwitchToSettings);

    let state = app.to_state();
    assert_eq!(state.current_view, ViewType::Settings);
}

// ❌ BAD: Test implementation detail
#[test]
fn test_switch_view_calls_internal() {
    let mut app = App::default();
    // Mocking internal private method (fragile!)
    assert!(app.internal_switch_was_called);
}
```

### Principle 3: State Invariants Must Always Hold

**Rule**: Define and test invariants that MUST be true for any valid state.

**Example**:
```rust
#[test]
fn test_state_invariants() {
    let state = app.to_state();

    // Invariant: If session active, must have session_id
    if state.session_active {
        assert!(state.session_id.is_some());
    }

    // Invariant: Path must exist if worktree is loaded
    if state.worktree_loaded {
        assert!(state.worktree_path.exists());
    }

    // Invariant: Active view must be valid
    assert!(matches!(
        state.current_view,
        ViewType::Worktree | ViewType::Settings | ViewType::Dashboard
    ));
}
```

---

## Required Tests (MANDATORY)

Every feature MUST include these three types of tests:

### 1. Round-Trip Serialization Test

**Purpose**: Verify state can be serialized and deserialized without data loss.

**Template**:
```rust
#[test]
fn test_{feature}_state_serialization_round_trip() {
    // Create state
    let state = YourFeatureState {
        field1: "value".to_string(),
        field2: Some(42),
        field3: vec![1, 2, 3],
    };

    // Serialize to JSON
    let json = serde_json::to_string(&state).unwrap();

    // Deserialize back
    let loaded: YourFeatureState = serde_json::from_str(&json).unwrap();

    // Verify equality
    assert_eq!(state, loaded);
}
```

**Real Example** (from codebase):
```rust
#[test]
fn test_app_state_serialization() {
    let state = AppState {
        version: "0.1.0".to_string(),
        current_view: ViewType::Worktree,
        worktree_view: WorktreeViewState::default(),
        dashboard_view: DashboardState::default(),
        settings_view: SettingsState::default(),
    };

    let json = serde_json::to_string(&state).unwrap();
    let loaded: AppState = serde_json::from_str(&json).unwrap();

    assert_eq!(state, loaded);
}
```

**When to Use**: ALWAYS. Every state struct MUST have this test.

### 2. State Transition Test

**Purpose**: Verify actions mutate state correctly.

**Template**:
```rust
#[test]
fn test_{feature}_state_transition() {
    // Setup initial state
    let initial_state = AppState::default();
    let mut app = App::from_state(initial_state).unwrap();

    // Perform action
    app.handle_action(ViewAction::YourAction {
        param: "value".to_string()
    });

    // Extract final state
    let final_state = app.to_state();

    // Verify state changed correctly
    assert_eq!(final_state.your_field, expected_value);
    assert!(final_state.your_flag);
}
```

**Real Example** (from codebase):
```rust
#[test]
fn test_prompt_claude_workflow() {
    let mut app = App::from_state(AppState::default()).unwrap();

    // User action: Run prompt
    app.handle_action(ViewAction::RunPromptClaude {
        prompt: "test prompt".to_string()
    });

    let state = app.to_state();

    // Verify state transition
    assert!(state.worktree_view.active_session_id.is_some());
    assert_eq!(state.worktree_view.pending_follow_up, false);
}
```

**When to Use**: For every user action that changes state.

### 3. State Invariant Test

**Purpose**: Verify state constraints always hold.

**Template**:
```rust
#[test]
fn test_{feature}_state_invariants() {
    // Create various states (normal, edge cases)
    let states = vec![
        create_normal_state(),
        create_edge_case_1(),
        create_edge_case_2(),
    ];

    for state in states {
        // Invariant 1: Field X implies field Y exists
        if state.field_x {
            assert!(state.field_y.is_some());
        }

        // Invariant 2: Value must be within range
        assert!(state.count >= 0 && state.count <= 100);

        // Invariant 3: Required relationships
        if state.parent_id.is_some() {
            assert!(state.children.is_empty() == false);
        }
    }
}
```

**Real Example** (from codebase):
```rust
#[test]
fn test_worktree_view_state_invariants() {
    let state = WorktreeViewState {
        active_session_id: Some("abc123".to_string()),
        pending_follow_up: true,
        current_input: "test".to_string(),
    };

    // Invariant: If pending_follow_up, must have active_session
    if state.pending_follow_up {
        assert!(state.active_session_id.is_some());
    }

    // Invariant: Input length must be reasonable
    assert!(state.current_input.len() < 10_000);
}
```

**When to Use**: When your state has logical constraints.

---

## Testing Patterns

### Pattern 1: Builder for Test State

**Problem**: Creating complex state for tests is verbose.

**Solution**: Use builder pattern.

```rust
// Test helper
struct AppStateBuilder {
    state: AppState,
}

impl AppStateBuilder {
    fn new() -> Self {
        Self {
            state: AppState::default(),
        }
    }

    fn with_view(mut self, view: ViewType) -> Self {
        self.state.current_view = view;
        self
    }

    fn with_session(mut self, session_id: &str) -> Self {
        self.state.worktree_view.active_session_id = Some(session_id.to_string());
        self
    }

    fn build(self) -> AppState {
        self.state
    }
}

// Usage in tests
#[test]
fn test_with_builder() {
    let state = AppStateBuilder::new()
        .with_view(ViewType::Settings)
        .with_session("abc123")
        .build();

    assert_eq!(state.current_view, ViewType::Settings);
    assert_eq!(state.worktree_view.active_session_id, Some("abc123".to_string()));
}
```

### Pattern 2: Snapshot Testing

**Problem**: Complex state is hard to verify field-by-field.

**Solution**: Serialize to JSON and compare snapshots.

```rust
#[test]
fn test_state_snapshot() {
    let state = create_complex_state();

    let json = serde_json::to_string_pretty(&state).unwrap();

    // First run: Manually verify and save as snapshot
    // Subsequent runs: Compare against snapshot
    let expected = include_str!("../snapshots/complex_state.json");
    assert_eq!(json, expected);
}
```

**Tool**: Use `insta` crate for automatic snapshot management (future enhancement).

### Pattern 3: Property-Based Testing

**Problem**: Hard to test all edge cases manually.

**Solution**: Generate random states, verify invariants hold.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_state_invariants_always_hold(
        count in 0u32..100,
        name in "[a-z]{1,20}",
    ) {
        let state = YourState {
            count,
            name,
        };

        // Invariants must hold for ALL generated states
        prop_assert!(state.count < 100);
        prop_assert!(!state.name.is_empty());
    }
}
```

**Tool**: Use `proptest` crate (future enhancement).

### Pattern 4: Error State Testing

**Problem**: Error states are often untested.

**Solution**: Create states representing errors, verify handling.

```rust
#[test]
fn test_error_state_handling() {
    let error_state = AppState {
        current_view: ViewType::Worktree,
        error_message: Some("Failed to load".to_string()),
        ..Default::default()
    };

    let mut app = App::from_state(error_state).unwrap();

    // Verify error is shown
    assert!(app.to_state().error_message.is_some());

    // User dismisses error
    app.handle_action(ViewAction::DismissError);

    // Verify error is cleared
    assert!(app.to_state().error_message.is_none());
}
```

---

## Integration Testing

### CLI Command Tests

**Purpose**: Test business logic via CLI interface.

**Example**:
```rust
#[test]
fn test_prompt_command() {
    let result = run_prompt_command(PromptOptions {
        message: "test".to_string(),
        max_turns: 5,
        skip_permissions: true,
        ..Default::default()
    });

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.session_id.is_some());
    assert_eq!(output.success, true);
}
```

**Why CLI?**: Easier to test than TUI (no event loop, no rendering).

### Multi-Step Workflow Tests

**Purpose**: Test sequences of actions.

**Example**:
```rust
#[test]
fn test_complete_workflow() {
    let mut app = App::from_state(AppState::default()).unwrap();

    // Step 1: Switch to Settings
    app.handle_action(ViewAction::SwitchToSettings);
    assert_eq!(app.to_state().current_view, ViewType::Settings);

    // Step 2: Change setting
    app.handle_action(ViewAction::UpdateSetting {
        key: "theme".to_string(),
        value: "dark".to_string(),
    });
    assert_eq!(app.to_state().settings_view.theme, "dark");

    // Step 3: Save settings
    app.handle_action(ViewAction::SaveSettings);
    assert_eq!(app.to_state().settings_view.unsaved_changes, false);
}
```

---

## UI Testing (Minimal)

**Principle**: Only test UI when absolutely necessary. Prefer state tests.

### When to Use UI Tests

- Widget rendering correctness
- Layout regressions
- Mouse/keyboard event handling

### ratatui TestBackend Pattern

```rust
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_widget_rendering() {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let app = App::default();

    terminal.draw(|frame| {
        app.render(frame, frame.area());
    }).unwrap();

    let buffer = terminal.backend().buffer();

    // Verify specific symbols (fragile!)
    assert_eq!(buffer.get(0, 0).symbol(), "┌");
    // Better: Verify layout structure, not coordinates
}
```

**Caution**: UI tests are fragile. Prefer state tests.

---

## Common Pitfalls

### ❌ Pitfall 1: Testing UI Coordinates

```rust
// ❌ BAD: Fragile, breaks on resize
#[test]
fn test_tab_position() {
    let buffer = render_app(&app);
    assert_eq!(buffer.get(10, 5).symbol, "│");
    // What if we change tab width? Test breaks!
}
```

**Fix**: Test state instead:
```rust
// ✅ GOOD: Tests behavior, not position
#[test]
fn test_tab_state() {
    let mut app = App::default();
    app.handle_action(ViewAction::SwitchToSettings);
    assert_eq!(app.to_state().current_view, ViewType::Settings);
}
```

### ❌ Pitfall 2: Hidden Test State

```rust
// ❌ BAD: Test state is hidden in closure
#[test]
fn test_with_hidden_state() {
    let mut secret_value = 0;
    let callback = || secret_value += 1;
    // Can't inspect secret_value from outside!
}
```

**Fix**: Make state explicit:
```rust
// ✅ GOOD: State is observable
#[test]
fn test_with_explicit_state() {
    let mut state = TestState { value: 0 };
    state.increment();
    assert_eq!(state.value, 1); // Observable!
}
```

### ❌ Pitfall 3: Testing Implementation Details

```rust
// ❌ BAD: Test internal private method
#[test]
fn test_internal_method() {
    let app = App::default();
    assert!(app.internal_private_helper()); // Breaks on refactor!
}
```

**Fix**: Test public behavior:
```rust
// ✅ GOOD: Test observable state change
#[test]
fn test_public_behavior() {
    let mut app = App::default();
    app.handle_action(ViewAction::DoSomething);
    assert_eq!(app.to_state().result, expected);
}
```

### ❌ Pitfall 4: Non-Deterministic Tests

```rust
// ❌ BAD: Depends on current time
#[test]
fn test_timestamp() {
    let state = create_state_with_current_time();
    assert_eq!(state.timestamp, chrono::Utc::now()); // Flaky!
}
```

**Fix**: Inject time dependency:
```rust
// ✅ GOOD: Deterministic
#[test]
fn test_timestamp() {
    let fixed_time = Utc.ymd(2025, 12, 19).and_hms(10, 0, 0);
    let state = create_state_with_time(fixed_time);
    assert_eq!(state.timestamp, fixed_time);
}
```

---

## Test Organization

### File Structure

```
crates/rstn/
├── src/
│   ├── tui/
│   │   ├── state/
│   │   │   ├── app_state.rs
│   │   │   └── worktree_state.rs
│   │   └── app.rs
│   └── domain/
│       └── worktree.rs
└── tests/
    ├── state/                      # State tests
    │   ├── app_state_test.rs       # AppState tests
    │   └── worktree_state_test.rs  # WorktreeState tests
    ├── integration/                # Integration tests
    │   ├── cli_commands_test.rs    # CLI command tests
    │   └── workflows_test.rs       # Multi-step workflows
    └── ui/                         # UI tests (minimal)
        └── rendering_test.rs       # Widget rendering
```

### Naming Conventions

**State tests**: `test_{struct_name}_state_{test_type}`
```rust
test_app_state_serialization_round_trip()
test_app_state_transition()
test_app_state_invariants()
```

**Integration tests**: `test_{feature}_{scenario}`
```rust
test_prompt_command_success()
test_prompt_command_handles_error()
test_session_continuation_workflow()
```

**UI tests**: `test_{widget}_{aspect}`
```rust
test_tab_bar_rendering()
test_input_dialog_keyboard_events()
```

---

## Running Tests

### Basic Commands

```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test -p rstn

# Run specific test
cargo test -p rstn test_app_state_serialization

# Run tests matching pattern
cargo test -p rstn state_transition
```

### With Coverage (Future)

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

**Target**: 70%+ coverage (state + integration tests)

### Continuous Integration

**CI Pipeline** (GitHub Actions):
```yaml
- name: Run tests
  run: cargo test --all

- name: Check coverage
  run: cargo tarpaulin --fail-under 70

- name: Clippy
  run: cargo clippy -- -D warnings
```

---

## Best Practices Summary

### ✅ DO

- Write state tests FIRST (before implementation)
- Test state transitions, not implementation details
- Use builders for complex test state
- Keep tests deterministic (no randomness, no current time)
- Test error states
- Verify invariants hold for all states
- Use descriptive test names
- Group related tests in modules

### ❌ DON'T

- Test UI coordinates (fragile)
- Test private methods directly
- Use hidden state (closures, globals)
- Write non-deterministic tests
- Skip state serialization tests
- Test implementation details
- Use unwrap() in production code (ok in tests)
- Mix state tests with UI tests

---

## Debugging Test Failures

### 1. Inspect State

```rust
#[test]
fn test_debug_state() {
    let state = app.to_state();

    // Pretty-print state as JSON
    let json = serde_json::to_string_pretty(&state).unwrap();
    eprintln!("State: {}", json);

    // Or use Debug formatting
    eprintln!("State: {:#?}", state);
}
```

### 2. Use RUST_BACKTRACE

```bash
RUST_BACKTRACE=1 cargo test test_name
```

### 3. Run Single Test with Output

```bash
cargo test test_name -- --nocapture
```

### 4. Save Failing State

```rust
#[test]
fn test_save_failing_state() {
    let state = create_failing_state();

    // Save to file for inspection
    let json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write("failing_state.json", json).unwrap();

    // ... rest of test ...
}
```

---

## Resources

- [State-First Architecture](../02-architecture/state-first.md) - Core principle
- [Core Principles](../02-architecture/core-principles.md) - Testing philosophy
- [Contribution Guide](contribution-guide.md) - PR requirements
- [Debugging Guide](debugging.md) - Troubleshooting

---

## Changelog

- 2025-12-19: Initial testing guide for v2
