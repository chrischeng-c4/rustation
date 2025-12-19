# Implementation Plan: TUI Input Keybindings

## Overview

Change keyboard shortcuts for TUI input dialogs to avoid WezTerm conflicts:
- **Enter** → Submit (was: newline in multiline mode)
- **Shift+Enter** → Newline in multiline mode (was: Alt+Enter)

## Files to Modify

| File | Change |
|------|--------|
| `crates/rstn/src/tui/app.rs:654-672` | Swap Enter/Shift+Enter logic in multiline input handling |
| `crates/rstn/src/tui/widgets/input_dialog.rs:225-240` | Update help text from "Alt+Enter" to "Shift+Enter: new line" |
| `crates/rstn/src/tui/app.rs` (tests) | Update test `test_alt_enter_submits_multiline_input` |
| `crates/rstn/tests/sdd_workflow_test.rs` | Update test `test_input_dialog_submits_on_alt_enter` |
| `crates/rstn/tests/e2e_tests/sdd_workflow_e2e.rs` | Update test `test_alt_enter_submits_and_clears_dialog` |

## Implementation Details

### 1. app.rs - Key Handling Logic (lines 654-672)

**Current**:
```rust
KeyCode::Enter => {
    if dialog.is_multiline() {
        // Multiline mode: Alt+Enter submits, Enter creates newline
        if key.modifiers.contains(KeyModifiers::ALT) {
            // Submit
        } else {
            dialog.insert_newline();
        }
    } else {
        // Single-line: Enter submits
    }
}
```

**New**:
```rust
KeyCode::Enter => {
    if dialog.is_multiline() {
        // Multiline mode: Shift+Enter creates newline, Enter submits
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            dialog.insert_newline();
        } else {
            // Submit
        }
    } else {
        // Single-line: Enter submits
    }
}
```

### 2. input_dialog.rs - Help Text (lines 225-240)

**Current**: `"Alt+Enter"` Submit
**New**: `"Enter"` Submit, `"Shift+Enter"` New line

### 3. Tests to Update

All tests using `KeyModifiers::ALT` with Enter need to be updated:
- Change `KeyModifiers::ALT` → `KeyModifiers::SHIFT` for newline tests
- Add tests for Enter submission in multiline mode

## Tech Stack

- Rust 1.75+ (edition 2021)
- crossterm (KeyModifiers::SHIFT already available)
- ratatui (TUI framework)

## No New Dependencies

All required functionality exists in crossterm's `KeyModifiers::SHIFT`.
