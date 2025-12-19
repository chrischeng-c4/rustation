# Research: Fix TUI Input Dialog Bug

**Feature**: 046-fix-tui-input
**Date**: 2025-12-14

## Code Flow Analysis

### Event Flow (Expected)

```
1. worktree.rs:994-999
   → User presses Enter on Specify
   → Returns ViewAction::RequestInput {prompt, placeholder}

2. app.rs:413-428
   → App.handle_view_action() receives RequestInput
   → Creates InputDialog::new_multiline() (for feature description)
   → Sets self.input_dialog = Some(dialog)
   → Sets self.input_mode = true

3. event.rs:113-116
   → EventHandler thread captures Key events via crossterm
   → Sends Event::Key(KeyEvent) to main loop channel

4. app.rs:1375-1377
   → Main loop receives Event::Key(key)
   → Calls self.handle_key_event(key)

5. app.rs:107-110
   → handle_key_event() checks if self.input_mode
   → If true, calls handle_key_event_in_input_mode(key)

6. app.rs:564-620
   → handle_key_event_in_input_mode() processes key
   → For KeyCode::Char(c), calls dialog.insert_char(c)
```

### Key Files

| File | Lines | Purpose |
|------|-------|---------|
| app.rs | 105-110 | Key event routing entry point |
| app.rs | 413-428 | RequestInput handling, dialog creation |
| app.rs | 562-620 | Input mode key processing |
| event.rs | 113-116 | Crossterm key event capture |
| input_dialog.rs | 77-84 | Character insertion forwarding |
| text_input.rs | 82-94 | Actual character insertion |

## Potential Bug Causes

### Hypothesis 1: State Synchronization (Most Likely)

**Theory**: `input_mode` is checked before `input_dialog` is set, causing a race condition.

**Evidence**:
- Lines 413-428 set `input_dialog` first, then `input_mode`
- This is the correct order, but there may be a frame where key events arrive between

**Investigation**: Add logging to verify state at key event time

### Hypothesis 2: Event Thread Race

**Theory**: Key events from crossterm are processed before the main loop processes ViewAction::RequestInput.

**Evidence**:
- Event handler runs in separate thread (event.rs:104)
- ViewAction is processed in main loop

**Investigation**: Check event processing order

### Hypothesis 3: Terminal Mode Issue

**Theory**: Crossterm raw mode may not be capturing character keys properly.

**Evidence**:
- The dialog appears (Enter key works)
- Typing doesn't work (character keys may be filtered)

**Investigation**: Check crossterm event polling configuration

### Hypothesis 4: Key Event Filtering

**Theory**: Some keys are being filtered in the event handler or main loop.

**Evidence**:
- event.rs:112-116 only sends Key events to channel
- No filtering visible, but worth investigating

**Investigation**: Log all key events received

## Testing Strategy Research

### ratatui TestBackend

**Documentation**: https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html

**Key Features**:
- In-memory terminal rendering
- `buffer()` - Access internal buffer for verification
- `assert_buffer_lines()` - Compare expected vs actual
- `assert_cursor_position()` - Verify cursor location

**Best Practice**: Use TestBackend for E2E tests, direct buffer testing for unit tests.

### Test Organization

**Rust Conventions**:
- Unit tests: `#[cfg(test)]` modules in source files
- Integration tests: `tests/` directory at crate root
- E2E tests: `tests/e2e/` with mod.rs for shared harness

## Decisions

### Decision 1: Debug Logging Approach

**Choice**: Use `tracing::debug!` macros at critical points

**Rationale**:
- Already using tracing crate in the project
- Can enable via RUST_LOG environment variable
- Non-invasive, can be left in production code

**Alternatives Rejected**:
- println! - Not controllable, clutters output
- Custom debug flag - More invasive, requires code changes

### Decision 2: Test Harness Design

**Choice**: Create TuiTestHarness struct wrapping App + TestBackend

**Rationale**:
- Encapsulates test setup complexity
- Reusable across multiple test files
- Clear API for sending keys and verifying output

**Alternatives Rejected**:
- Direct TestBackend usage - Too verbose, duplicated setup
- Mock-based testing - Over-engineered for this use case

### Decision 3: Test File Organization

**Choice**:
- Unit tests: In existing source files
- Integration: `tests/sdd_workflow_test.rs`
- E2E: `tests/e2e/*.rs`

**Rationale**:
- Follows Rust conventions
- Clear separation of test types
- Easy to run specific test sets

## Root Cause (Identified)

### Bug Location

**File**: `crates/rstn/src/tui/widgets/input_dialog.rs`
**Lines**: 172-187 (constraint definitions) and 246 (render check)

### Root Cause

The layout constraints in `InputDialog::render()` always allocated exactly 1 line height for the input area via `Constraint::Length(1)`, regardless of whether the dialog was in multiline mode.

However, in `render_input()` for multiline mode:
```rust
if area.height > 1 {  // Line 246
    let input_area = Rect::new(...);
    Widget::render(&self.input, input_area, buf);
}
```

Since `area.height` was always 1, this check always failed for multiline dialogs, causing the TextInput widget to never be rendered. Users could see the dialog frame and prompt, but could not see or interact with the input field.

### Fix Applied

Changed the constraint calculation to be dynamic based on multiline mode:
```rust
let input_height = if self.input.multiline {
    1 + (self.input.max_lines as u16).min(5)  // prompt line + input lines
} else {
    1
};
```

This ensures multiline dialogs have enough height to render both the prompt and the TextInput widget.

### Why Other Hypotheses Were Not the Cause

1. **State synchronization** - Not the cause: `input_mode` and `input_dialog` were correctly synchronized
2. **Event thread race** - Not the cause: Key events were being received correctly
3. **Terminal mode** - Not the cause: Crossterm was properly capturing all key events
4. **Event filtering** - Not the cause: Debug logs showed characters reaching `insert_char()`

The characters were being inserted into the TextInput, but the widget was never rendered due to the height constraint issue.

## Next Steps

1. ~~Add debug logging to app.rs~~ DONE
2. ~~Run TUI and trigger bug to capture logs~~ SKIPPED (found bug via code analysis)
3. ~~Analyze logs to identify root cause~~ DONE (found via code analysis)
4. ~~Implement fix based on findings~~ DONE
5. Add comprehensive tests (in progress)
