# UI Testing Plan - GPUI Application

**Status**: 🟡 Planned (Blocked by Metal/Xcode requirement)
**Created**: 2026-01-12
**Purpose**: Comprehensive UI testing strategy for GPUI migration

## Overview

This document defines the UI testing strategy for rustation's GPUI application. While we cannot execute these tests without Xcode/Metal installed, we can **plan and write test code now** to be ready when Metal is available.

## GPUI Testing Architecture

### TestAppContext Features

According to [GPUI documentation](https://himasnhu-at.github.io/GPUI-docs-unofficial/), GPUI provides `TestAppContext` with:

- **Headless Mode**: `TestPlatform` provides stub implementations (no native windows)
- **Deterministic Executor**: `BackgroundExecutor::new(TestDispatcher)` for reproducible async tests
- **Simulated Input**: `TestWindow::simulate_input()` injects platform events
- **Direct Access**: `Entity::read(cx)` and `Entity::update(cx, ...)` for state inspection
- **Fake Clipboard**: `TestPlatform` stores clipboard data in memory

### Test Macro

```rust
#[gpui::test]
async fn test_name(cx: &mut TestAppContext) {
    // Test code here
}
```

## Testing Strategy

### Three-Layer Testing Approach

```
┌─────────────────────────────────────────────┐
│ Layer 1: State Tests (✅ 200+ passing)      │
│   - Unit tests for state transitions        │
│   - Serialization/deserialization           │
│   - Business logic                          │
│   - NO Xcode required                       │
├─────────────────────────────────────────────┤
│ Layer 2: View Integration Tests (📝 Planned)│
│   - Views render with state                 │
│   - State → UI data flow                    │
│   - Component composition                   │
│   - REQUIRES Xcode/Metal                    │
├─────────────────────────────────────────────┤
│ Layer 3: Interactive Tests (📝 Planned)     │
│   - Button clicks, form inputs              │
│   - Async operations (command execution)    │
│   - State updates from UI events            │
│   - REQUIRES Xcode/Metal                    │
└─────────────────────────────────────────────┘
```

**Current Status**:
- Layer 1: ✅ Complete (200+ tests passing without Xcode)
- Layer 2: 📝 Planned in this document
- Layer 3: 📝 Planned in this document

## Layer 2: View Integration Tests

### Test File Structure

```
crates/rstn/tests/
├── ui/
│   ├── mod.rs
│   ├── tasks_view_test.rs      # TasksView rendering
│   ├── dockers_view_test.rs    # DockersView rendering
│   ├── explorer_view_test.rs   # ExplorerView rendering
│   ├── terminal_view_test.rs   # TerminalView rendering
│   ├── chat_view_test.rs       # ChatView rendering
│   ├── workflows_view_test.rs  # WorkflowsView rendering
│   ├── mcp_view_test.rs        # McpView rendering
│   └── settings_view_test.rs   # SettingsView rendering
└── integration/
    ├── mod.rs
    ├── state_to_ui_test.rs      # Full state → UI pipeline
    └── app_view_test.rs         # Main AppView integration
```

### Example: TasksView Integration Test

```rust
// crates/rstn/tests/ui/tasks_view_test.rs

use gpui::*;
use rstn::state::AppState;
use rstn_views::TasksView;
use rstn_ui::MaterialTheme;

#[gpui::test]
async fn test_tasks_view_renders_with_empty_state(cx: &mut TestAppContext) {
    // Setup: Create empty state
    let state = AppState::new();

    cx.update(|cx| {
        let window = cx.open_window(WindowOptions::default(), |cx| {
            // Render TasksView with empty tasks
            let tasks = state.get_justfile_tasks();
            let theme = MaterialTheme::dark();

            TasksView::new(tasks, theme)
        }).unwrap();

        // Assertion: View should render without panic
        // In GPUI, if render() doesn't panic, the test passes
        assert!(window.is_some());
    });
}

#[gpui::test]
async fn test_tasks_view_renders_with_real_tasks(cx: &mut TestAppContext) {
    // Setup: Create state with real justfile
    let mut state = AppState::new();
    state.initialize(); // Loads from real justfile

    cx.update(|cx| {
        let tasks = state.get_justfile_tasks();

        // Verify tasks were loaded
        assert!(!tasks.is_empty(), "Should load tasks from justfile");

        let window = cx.open_window(WindowOptions::default(), |cx| {
            let theme = MaterialTheme::dark();
            TasksView::new(tasks.clone(), theme)
        }).unwrap();

        assert!(window.is_some());
    });
}

#[gpui::test]
async fn test_tasks_view_displays_correct_task_count(cx: &mut TestAppContext) {
    // Setup: State with known tasks
    let mut state = AppState::new();
    state.initialize();

    let tasks = state.get_justfile_tasks();
    let expected_count = tasks.len();

    cx.update(|cx| {
        let window = cx.open_window(WindowOptions::default(), |cx| {
            let theme = MaterialTheme::dark();
            TasksView::new(tasks, theme)
        }).unwrap();

        // TODO: Once we can query rendered elements, verify task count in UI
        // For now, we verify state → view data flow works
        assert!(expected_count > 0, "Should have at least 1 task");
    });
}
```

### Example: DockersView Integration Test

```rust
// crates/rstn/tests/ui/dockers_view_test.rs

use gpui::*;
use rstn::state::AppState;
use rstn_views::DockersView;
use rstn_ui::MaterialTheme;

#[gpui::test]
async fn test_dockers_view_renders_built_in_services(cx: &mut TestAppContext) {
    let mut state = AppState::new();
    state.initialize();

    cx.update(|cx| {
        let services = state.get_docker_services();

        // Verify built-in services loaded
        assert!(!services.is_empty(), "Should have built-in Docker services");

        let window = cx.open_window(WindowOptions::default(), |cx| {
            let theme = MaterialTheme::dark();
            DockersView::new(services, theme)
        }).unwrap();

        assert!(window.is_some());
    });
}

#[gpui::test]
async fn test_dockers_view_handles_empty_services(cx: &mut TestAppContext) {
    let state = AppState::new();
    // Don't initialize - should have empty services

    cx.update(|cx| {
        let services = state.get_docker_services();
        assert!(services.is_empty());

        let window = cx.open_window(WindowOptions::default(), |cx| {
            let theme = MaterialTheme::dark();
            DockersView::new(services, theme)
        }).unwrap();

        // Should render empty state without panic
        assert!(window.is_some());
    });
}
```

## Layer 3: Interactive Tests

### Example: Button Click Tests

```rust
// crates/rstn/tests/integration/button_click_test.rs

use gpui::*;
use rstn::state::AppState;

#[gpui::test]
async fn test_execute_task_button_click(cx: &mut TestAppContext) {
    let mut state = cx.new_model(|_| {
        let mut s = AppState::new();
        s.initialize();
        s
    });

    cx.update(|cx| {
        let window = cx.open_window(WindowOptions::default(), |cx| {
            // Create TasksView with button click handler
            // TODO: Implement once event handlers are added to views
        }).unwrap();

        // Simulate button click
        // window.simulate_input(...);

        // Verify state updated
        // let tasks = state.read(cx).get_justfile_tasks();
        // assert!(tasks[0].status == TaskStatus::Running);
    });
}
```

### Example: Async Command Execution Test

```rust
// crates/rstn/tests/integration/command_execution_test.rs

use gpui::*;
use rstn::state::AppState;

#[gpui::test]
async fn test_execute_justfile_command_updates_state(cx: &mut TestAppContext) {
    let mut state = cx.new_model(|_| {
        let mut s = AppState::new();
        s.initialize();
        s
    });

    // Get first task name
    let task_name = state.update(cx, |s, _cx| {
        s.get_justfile_tasks()[0].name.clone()
    });

    // Execute command (async)
    let result = state.update(cx, |s, _cx| {
        s.execute_just_command(&task_name)
    }).await;

    // Verify execution completed
    assert!(result.is_ok());

    // Verify state reflects completion
    state.update(cx, |s, _cx| {
        let tasks = s.get_justfile_tasks();
        let executed_task = tasks.iter().find(|t| t.name == task_name).unwrap();
        // TODO: Check task status once status tracking is implemented
    });
}
```

## Test Coverage Goals

### Must Have (MVP)

| View | Empty State | With Data | Component Rendering |
|------|-------------|-----------|-------------------|
| TasksView | ✅ Planned | ✅ Planned | ✅ Planned |
| DockersView | ✅ Planned | ✅ Planned | ✅ Planned |
| ExplorerView | ✅ Planned | ✅ Planned | ✅ Planned |
| TerminalView | ✅ Planned | ⏸️ Deferred (PTY) | ✅ Planned |
| ChatView | ✅ Planned | ✅ Planned | ✅ Planned |
| WorkflowsView | ✅ Planned | ✅ Planned | ✅ Planned |
| McpView | ✅ Planned | ✅ Planned | ✅ Planned |
| SettingsView | ✅ Planned | ✅ Planned | ✅ Planned |

### Should Have

- [ ] Button click handlers (all views)
- [ ] Form input validation (SettingsView, ChatView)
- [ ] Async operation handling (TasksView command execution)
- [ ] Background polling (DockersView status updates)
- [ ] File selection (ExplorerView)
- [ ] Tab switching (AppView navigation)

### Nice to Have

- [ ] Keyboard shortcuts
- [ ] Drag and drop (ExplorerView)
- [ ] Copy/paste (TerminalView, ChatView)
- [ ] Scroll behavior (long lists)
- [ ] Resize behavior (split panels)

## Execution Requirements

### To Run These Tests

**Requires**:
1. Full Xcode installation (15.4+)
2. Metal toolchain configured (`xcrun --find metal` works)
3. Active developer directory set to Xcode:
   ```bash
   sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
   ```

**Run tests**:
```bash
# All UI tests
cargo test --test '*' --features gpui/test-support

# Specific test file
cargo test --test tasks_view_test

# Single test
cargo test test_tasks_view_renders_with_empty_state
```

### CI/CD Strategy

**Option 1: GitHub Actions (Recommended)**
```yaml
# .github/workflows/ui-tests.yml
name: UI Tests
on: [push, pull_request]
jobs:
  test-ui:
    runs-on: macos-latest  # Has Xcode pre-installed
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --test '*' --features gpui/test-support
```

**Option 2: Local Testing**
```bash
# Only run when Xcode is available
if xcrun --find metal &>/dev/null; then
    cargo test --test '*' --features gpui/test-support
else
    echo "⚠️  Skipping UI tests (Xcode not installed)"
    cargo test --package rstn-core  # Run unit tests only
fi
```

## Implementation Timeline

### Phase 1: Write Test Structure (Can Do NOW)
- [x] Create UI_TESTING_PLAN.md (this document)
- [ ] Create `crates/rstn/tests/ui/` directory structure
- [ ] Write example tests for TasksView
- [ ] Write example tests for DockersView
- [ ] Add test dependencies to Cargo.toml

**Status**: Can do without Xcode ✅

### Phase 2: Execute Tests (After Xcode Install)
- [ ] Install Xcode 15.4+
- [ ] Run `cargo test --test '*'`
- [ ] Fix any test failures
- [ ] Achieve >80% UI test coverage

**Status**: Blocked by Metal/Xcode ❌

### Phase 3: Add Interactive Tests (Post-MVP)
- [ ] Implement button click handlers
- [ ] Write click simulation tests
- [ ] Test async command execution
- [ ] Test background polling

**Status**: Requires event handlers first ⏸️

## Current Limitations

### What We CAN Test Now (Without Xcode)

✅ **State Tests** (Layer 1):
```bash
cargo test --package rstn-core        # 182 tests ✅
cargo test --package rstn             # 18 state tests ✅
```

### What We CANNOT Test Now (Needs Xcode)

❌ **View Rendering** (Layer 2):
```bash
cargo test --test ui_tests  # ❌ Metal compilation error
```

❌ **Interactive Tests** (Layer 3):
```bash
cargo test --test integration_tests  # ❌ Metal compilation error
```

## Workarounds While Waiting for Xcode

### 1. Write Tests as Documentation

Even if we can't run tests, writing them serves as:
- **Specification**: Documents expected behavior
- **Design verification**: Ensures views are testable
- **Readiness**: Tests ready to run when Metal available

### 2. Use GitHub Actions

Push code to GitHub and let CI run tests on macOS runners:
- Free for public repos
- macOS runners have Xcode pre-installed
- Tests run automatically on every push

### 3. Focus on State Testing

Since state tests don't need Metal:
- Verify all state transitions
- Test all accessor methods
- Ensure state serialization works
- **If state is correct, UI will follow** (render is pure function)

## References

- **GPUI Documentation**: [GPUI Unofficial Docs](https://himasnhu-at.github.io/GPUI-docs-unofficial/)
- **GPUI Testing**: [TestAppContext](https://deepwiki.com/zed-industries/zed/2.2-gpui-framework)
- **Zed Source**: [GPUI README](https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md)
- **Metal Setup**: [METAL_TOOLCHAIN_SETUP.md](METAL_TOOLCHAIN_SETUP.md)
- **State Tests**: [crates/rstn-core/src/reducer/tests.rs](../crates/rstn-core/src/reducer/tests.rs)

---

**Status**: 📝 Plan Complete, Ready for Implementation
**Next Step**: Create test file structure and write example tests
**Blocker**: Tests cannot execute until Xcode/Metal available
