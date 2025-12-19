# Implementation Plan: Fix TUI Input Dialog Bug

**Branch**: `046-fix-tui-input` | **Date**: 2025-12-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/046-fix-tui-input/spec.md`

## Summary

Fix a critical bug in the rstn TUI where the Specify workflow input dialog appears but keyboard input doesn't work. Users cannot type in the input field, completely blocking the SDD workflow. Additionally, add comprehensive testing infrastructure including unit tests (12+), integration tests (6+), and E2E tests (3+) using ratatui's TestBackend for visual verification.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: ratatui 0.29+ (TUI framework), crossterm (terminal I/O), tokio (async runtime)
**Storage**: N/A (in-memory state only)
**Testing**: cargo test + ratatui::backend::TestBackend for E2E
**Target Platform**: macOS (primary), Linux (secondary)
**Project Type**: Single project (rstn crate within rustation monorepo)
**Performance Goals**: Input response <16ms (60 FPS), no blocking on key events
**Constraints**: Must not regress existing 670+ tests, must maintain <100ms startup
**Scale/Scope**: Bug fix + ~21 new tests across unit/integration/E2E

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Performance-First | PASS | Bug fix improves responsiveness; no performance overhead added |
| II. Zero-Config | PASS | No configuration changes; feature works out of the box |
| III. Progressive Complexity | PASS | Bug fix; no new complexity introduced |
| IV. Modern UX | PASS | Restores expected TUI interaction behavior |
| V. Rust-Native | PASS | Uses existing Rust dependencies (ratatui, crossterm) |

**Gate Status**: PASS - No violations. Proceed with implementation.

## Project Structure

### Documentation (this feature)

```text
specs/046-fix-tui-input/
├── spec.md              # Feature specification (completed)
├── plan.md              # This file
├── research.md          # Phase 0 output (investigation findings)
├── data-model.md        # N/A (no new data models)
├── quickstart.md        # N/A (bug fix, no quickstart needed)
├── contracts/           # N/A (no API contracts)
└── tasks.md             # Phase 2 output (task breakdown)
```

### Source Code (repository root)

```text
crates/rstn/
├── src/
│   ├── tui/
│   │   ├── app.rs           # MODIFY: Add debug logging, fix event routing
│   │   ├── event.rs         # INVESTIGATE: Key event handling
│   │   ├── widgets/
│   │   │   ├── input_dialog.rs  # ADD TESTS: Unit tests for dialog
│   │   │   └── text_input.rs    # ADD TESTS: Unit tests for input
│   │   └── views/
│   │       └── worktree.rs      # ADD TESTS: Specify workflow tests
│   └── lib.rs
└── tests/
    ├── sdd_workflow_test.rs     # NEW: Integration tests
    └── e2e/
        ├── mod.rs               # NEW: E2E test harness
        └── sdd_workflow_e2e.rs  # NEW: E2E tests with TestBackend
```

**Structure Decision**: Uses existing rstn crate structure. New test files added to `tests/` directory following Rust conventions.

## Investigation Strategy

### Phase 1: Add Debug Logging

Add tracing at these critical points to identify where input is lost:

1. **app.rs:105-110** - `handle_key_event()` entry point
   - Log: `input_mode` value, key code received

2. **app.rs:413-428** - `ViewAction::RequestInput` handling
   - Log: Dialog creation, `input_mode` set to true

3. **app.rs:562-620** - `handle_key_event_in_input_mode()`
   - Log: Key received, `input_dialog.is_some()`, character insertion

### Phase 2: Identify Root Cause

Likely bug locations (in order of probability):

1. **State synchronization**: `input_mode` not set before next key event arrives
2. **Event thread race**: Key events processed before dialog is ready
3. **Terminal mode**: Raw mode not properly capturing character keys
4. **Event filtering**: Some keys filtered out in event handler

### Phase 3: Fix Implementation

Based on investigation, implement fix in the appropriate location.

## Complexity Tracking

> No constitution violations - this section is empty.

## Deployment Strategy

### Selected Strategy

**Option 2: PR per User Story** - This feature has 4 user stories of varying complexity.

**Rationale**: The bug fix (P1) should be merged first for immediate user benefit, followed by tests in subsequent PRs. Total estimated ~600 lines for fix + ~800 lines for tests.

### Pull Request Plan

```
PR #1: Bug Investigation & Fix (P1 - User Story 1)
  - Add debug logging to trace input flow
  - Identify and fix the root cause
  - Basic smoke test to verify fix
  - Target: ≤ 400 lines

PR #2: Unit Tests (P2 - User Story 2)
  - 12+ unit tests for input dialog and app input mode
  - Tests in existing test modules
  - Target: ≤ 500 lines

PR #3: Integration Tests (P2 - User Story 3)
  - 6+ integration tests for SDD workflow
  - New test file: sdd_workflow_test.rs
  - Target: ≤ 400 lines

PR #4: E2E Tests with TestBackend (P3 - User Story 4)
  - 3+ E2E tests using ratatui TestBackend
  - New test files: e2e/mod.rs, e2e/sdd_workflow_e2e.rs
  - Target: ≤ 500 lines
```

### Merge Sequence

1. **PR #1**: Fix input dialog bug → Merge to main (unblocks users)
2. **PR #2**: Add unit tests → Merge to main (prevents regression)
3. **PR #3**: Add integration tests → Merge to main (workflow coverage)
4. **PR #4**: Add E2E tests → Merge to main (visual verification)

**Branch Strategy**: Work on `046-fix-tui-input` branch, merge each PR to main sequentially.

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits**:
- PR #1: ≤ 400 lines (bug fix + logging)
- PR #2: ≤ 500 lines (unit tests)
- PR #3: ≤ 400 lines (integration tests)
- PR #4: ≤ 500 lines (E2E tests)

All PRs within ideal limits.

## Test Infrastructure Design

### Unit Test Structure (input_dialog.rs)

```rust
#[cfg(test)]
mod tests {
    // Existing tests...

    #[test]
    fn test_new_creates_active_input() { ... }

    #[test]
    fn test_insert_char_forwards_to_input() { ... }

    #[test]
    fn test_multiline_insert_char() { ... }

    #[test]
    fn test_cursor_movement_methods() { ... }
}
```

### E2E Test Harness (e2e/mod.rs)

```rust
use ratatui::backend::TestBackend;
use ratatui::Terminal;

pub struct TuiTestHarness {
    pub app: App,
    pub terminal: Terminal<TestBackend>,
}

impl TuiTestHarness {
    pub fn new(width: u16, height: u16) -> Self { ... }
    pub fn send_key(&mut self, key: KeyCode) { ... }
    pub fn send_text(&mut self, text: &str) { ... }
    pub fn render(&mut self) { ... }
    pub fn buffer_contains(&self, text: &str) -> bool { ... }
}
```

## Success Metrics

| Metric | Target | Validation |
|--------|--------|------------|
| Bug fix verified | Input works in dialog | Manual test |
| Existing tests | 670+ pass | `cargo test` |
| New unit tests | 12+ added | Count in PR |
| New integration tests | 6+ added | Count in PR |
| New E2E tests | 3+ added | Count in PR |
| No performance regression | <16ms input response | Manual test |
