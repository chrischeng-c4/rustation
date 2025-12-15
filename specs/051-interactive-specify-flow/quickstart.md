# Developer Quickstart: Interactive Specify Flow

**Feature**: 051-interactive-specify-flow
**Date**: 2025-12-15

## Overview

This guide helps developers quickly understand and work on the Interactive Specify Flow feature. Read this before diving into implementation.

## 5-Minute Context

**What**: Transform `/speckit.specify` from a shell-out to an integrated TUI workflow
**Why**: Eliminate context switching, enable spec review/edit before saving
**Pattern**: Copy feature 050 (Commit Review) - same drop dialog UX
**Scope**: TUI-only changes, no modifications to shell script

## Prerequisites

Before working on this feature:

1. **Understand the baseline**:
   ```bash
   # Current behavior: /speckit.specify shells out, spec saved immediately
   # New behavior: Stay in TUI, input → generate → review → save
   ```

2. **Study feature 050** (Commit Review):
   ```bash
   # Read the spec
   cat specs/050-commit-review-content-area/spec.md

   # Examine the implementation
   rg "CommitReview" crates/rstn/src/tui/
   rg "commit_review" crates/rstn/src/tui/
   ```

   Feature 051 mirrors 050's architecture:
   - ContentType enum variants (CommitReview → SpecifyInput/SpecifyReview)
   - State structure (commit_review_state → specify_state)
   - Event flow (CommitGroupsReady → SpecifyGenerationCompleted)
   - Key bindings (Enter/Esc/edit mode)

3. **Run the existing TUI**:
   ```bash
   cargo build -p rstn
   ./target/debug/rstn --cli
   ```

## Key Files

### Files You'll Modify

| File | What Changes | Why |
|------|-------------|-----|
| `crates/rstn/src/tui/views/mod.rs` | Add `ContentType::SpecifyInput`, `ContentType::SpecifyReview` | Discriminate specify modes |
| `crates/rstn/src/tui/views/worktree.rs` | Add `SpecifyState`, methods for input/review/edit | Core workflow logic |
| `crates/rstn/src/tui/app.rs` | Handle specify events and actions | Orchestrate async operations |
| `crates/rstn/src/tui/event.rs` | Add specify events | Event-driven architecture |

### Files You'll Create

| File | Purpose |
|------|---------|
| `tests/unit/tui_specify_tests.rs` | Unit tests for state management |
| `tests/integration/specify_workflow_test.rs` | Integration tests for full workflow |

### Files You Won't Touch

| File | Why Not |
|------|---------|
| `.specify/scripts/bash/create-new-feature.sh` | Feature 051 uses as-is; feature 052 will internalize |
| `crates/rstn-core/src/git/*` | No git operations in this feature |
| `crates/rstn/src/tui/views/panes.rs` | Commands pane already extensible |

## Architecture At A Glance

### State Machine

```
Normal View
    ↓ (trigger "Specify")
SpecifyInput (user types description)
    ↓ (press Enter)
Generating (async shell script)
    ↓ (on success)
SpecifyReview (display generated spec)
    ↓ (press 'e')
SpecifyEdit (inline editing)
    ↓ (press Ctrl+S or Enter)
Save & Return to Normal View
```

### Data Flow

```
User Input
    ↓
WorktreeView.specify_state.input_buffer
    ↓
ViewAction::GenerateSpec { description }
    ↓
App spawns async task → shell script
    ↓
Event::SpecifyGenerationCompleted { spec, number, name }
    ↓
WorktreeView.specify_state.generated_spec = Some(spec)
    ↓
User reviews/edits
    ↓
ViewAction::SaveSpec { content, number, name }
    ↓
Write to specs/{number}-{name}/spec.md
    ↓
Event::SpecifySaved { path }
    ↓
WorktreeView.specify_state.clear()
```

## Development Workflow

### Step 1: Set Up Branch

```bash
# Branch already created during /speckit.specify
git checkout 051-interactive-specify-flow

# Verify you're on the right branch
git branch --show-current
```

### Step 2: Implement in Phases (PRs)

Follow the PR plan in `plan.md`:

**PR #1: Foundation + Input Mode**
```bash
# 1. Add ContentType variants
# 2. Add SpecifyState struct
# 3. Implement input handling
# 4. Add rendering for input mode
# 5. Add tests

# Build and test
cargo build -p rstn
cargo test --package rstn

# Manual test
./target/debug/rstn --cli
# Trigger "Specify" from Commands pane
# Verify input dialog appears
# Type description, press Enter
```

**PR #2: Review Mode**
```bash
# 1. Implement async generation
# 2. Add review rendering
# 3. Implement save logic
# 4. Add tests

# Test workflow
./target/debug/rstn --cli
# Complete input → generation → review → save
```

**PR #3: Edit Mode**
```bash
# 1. Add edit mode flag
# 2. Implement edit input handling
# 3. Add edit mode rendering
# 4. Add tests

# Test editing
# Press 'e' in review mode
# Make changes, press Ctrl+S
```

**PR #4: Polish**
```bash
# 1. Error messages
# 2. Status bar updates
# 3. Timeout handling
# 4. Integration tests
```

### Step 3: Testing Strategy

**Unit Tests**:
```rust
#[test]
fn test_specify_state_validation() {
    let mut state = SpecifyState::new();

    // Empty input should fail
    assert!(state.validate_input().is_err());

    // Valid input should pass
    state.input_buffer = "Add user authentication".to_string();
    assert!(state.validate_input().is_ok());
}

#[test]
fn test_specify_state_transitions() {
    let mut view = WorktreeView::new();

    // Start specify
    view.start_specify_input();
    assert_eq!(view.content_type, ContentType::SpecifyInput);

    // Cancel
    view.cancel_specify();
    assert_eq!(view.content_type, ContentType::Spec);
    assert!(!view.specify_state.is_active());
}
```

**Integration Test**:
```rust
#[tokio::test]
async fn test_full_specify_workflow() {
    // 1. Start specify
    // 2. Input description
    // 3. Trigger generation
    // 4. Verify review mode
    // 5. Save spec
    // 6. Verify file exists
}
```

**Manual Testing**:
```bash
# Test script
./target/debug/rstn --cli

# In TUI:
# 1. Navigate to Commands pane
# 2. Select "Specify"
# 3. Enter: "Add dark mode toggle"
# 4. Verify generation starts
# 5. Wait for spec to appear
# 6. Press 'e' to edit
# 7. Make a change
# 8. Press Ctrl+S
# 9. Verify spec saved
# 10. Check file exists: ls specs/*/spec.md
```

## Common Patterns

### Adding a New Mode

1. Add enum variant:
   ```rust
   pub enum ContentType {
       // ...
       YourNewMode,
   }
   ```

2. Add state:
   ```rust
   pub struct YourModeState {
       // fields
   }
   ```

3. Handle in render:
   ```rust
   fn render_content(&self, area: Rect) -> Vec<Line> {
       match self.content_type {
           ContentType::YourNewMode => self.render_your_mode(area),
           // ...
       }
   }
   ```

4. Handle input:
   ```rust
   fn handle_key_event(&mut self, key: KeyEvent) -> ViewAction {
       match self.content_type {
           ContentType::YourNewMode => self.handle_your_mode_input(key),
           // ...
       }
   }
   ```

### Async Operations

```rust
// In app.rs
match action {
    ViewAction::GenerateSpec { description } => {
        let event_sender = self.event_sender.clone();
        tokio::spawn(async move {
            match execute_spec_generation(description).await {
                Ok(result) => {
                    event_sender.send(Event::SpecifyGenerationCompleted {
                        spec: result.spec,
                        number: result.number,
                        name: result.name,
                    }).await.ok();
                }
                Err(e) => {
                    event_sender.send(Event::SpecifyGenerationFailed {
                        error: e,
                    }).await.ok();
                }
            }
        });
    }
}
```

### Error Handling

```rust
// Show error in Output area
self.output_lines.push(format!("Error: {}", error));

// Show inline validation error
self.specify_state.validation_error = Some(error);

// Clear errors when user retries
self.specify_state.validation_error = None;
```

## Debugging Tips

### Enable Debug Logging

```rust
// Add tracing
use tracing::{debug, info, warn, error};

info!("Starting specify input");
debug!("Input buffer: {:?}", self.specify_state.input_buffer);
warn!("Validation failed: {}", error);
error!("Generation failed: {}", error);
```

### Check Logs

```bash
# Logs are written to ~/.rustation/logs/
tail -f ~/.rustation/logs/rstn.log

# Search for specify events
grep -i "specify" ~/.rustation/logs/rstn.log
```

### Visual Debugging

```rust
// Add debug info to UI (temporarily)
let debug_info = format!(
    "State: input={}, generating={}, review={}",
    !self.specify_state.input_buffer.is_empty(),
    self.specify_state.is_generating,
    self.specify_state.generated_spec.is_some()
);
lines.push(Line::from(debug_info));
```

## Performance Considerations

### Target Metrics

- Input dialog render: <50ms
- Key press response: <16ms (60 FPS)
- Mode transitions: <50ms
- Generation time: Variable (depends on shell script)

### Profiling

```bash
# Build with release profile
cargo build --release -p rstn

# Run with time measurement
time ./target/release/rstn --cli

# Profile specific operations
cargo flamegraph --bin rstn
```

### Optimization Checklist

- [ ] Minimize allocations in hot paths (input handling)
- [ ] Reuse buffers where possible
- [ ] Avoid cloning large strings unnecessarily
- [ ] Batch UI updates to avoid flickering

## Getting Help

### Resources

1. **Feature 050 implementation**: The reference pattern
   ```bash
   rg "CommitReview" crates/rstn/src/tui/
   ```

2. **Ratatui docs**: https://ratatui.rs/
3. **Crossterm docs**: https://docs.rs/crossterm/

### Common Issues

| Issue | Solution |
|-------|----------|
| Input not captured | Check `ActivePane` is set to `Content` |
| Async operation blocks | Ensure using `tokio::spawn`, not blocking |
| State persists after cancel | Call `specify_state.clear()` |
| UI doesn't update | Verify event loop processes events |

### Review Checklist

Before submitting PR:
- [ ] Builds without warnings: `cargo build -p rstn`
- [ ] Tests pass: `cargo test --package rstn`
- [ ] Clippy clean: `cargo clippy --all-targets --all-features`
- [ ] Formatted: `cargo fmt`
- [ ] Manual testing complete (see test script above)
- [ ] PR size < 1,500 lines
- [ ] Follows feature 050 patterns
- [ ] Documentation updated

## Next Steps

1. Read `spec.md` for full requirements
2. Read `plan.md` for implementation strategy
3. Study feature 050 implementation
4. Start with PR #1: Foundation + Input Mode
5. Follow the PR sequence in `plan.md`

**Questions?** Check feature 050 code first - it's your best reference.
