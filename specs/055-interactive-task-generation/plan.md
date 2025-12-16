# Implementation Plan: Interactive Task Generation

**Branch**: `055-interactive-task-generation` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)

## Summary

Enhance the TUI task generation workflow with structured task editing capabilities. Building on the foundation from Feature 059 (PromptManager integration), this adds the ability to view, reorder, and edit individual tasks before saving to `tasks.md`.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: ratatui, crossterm (already in workspace)
**Testing**: cargo test with TestBackend
**Target Platform**: macOS (MVP), Linux (future)
**Project Type**: Enhancement to `rstn` TUI crate

## What Already Exists (from 059)

- `execute_tasks_generation()` - Generates tasks via Claude CLI + PromptManager
- `SpecifyState` generalized for all phases including Tasks
- Input → Generate → Review → Edit → Save workflow
- Hotkey `t` triggers task generation
- Text-based editing of generated content

## What This Feature Adds

1. **Structured Task View**: Parse generated tasks into list format
2. **Task Reordering**: Move tasks up/down with keyboard shortcuts
3. **Individual Task Editing**: Edit single task without losing list structure
4. **Dependency Markers**: Visual indicators for task dependencies

## Constitution Check

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Performance-First | ✅ PASS | No new API calls, local parsing only |
| II. Zero-Config | ✅ PASS | Works with existing task format |
| III. Progressive Complexity | ✅ PASS | Simple list view, optional reordering |
| IV. Modern UX | ✅ PASS | Keyboard shortcuts, visual feedback |
| V. Rust-Native | ✅ PASS | Pure ratatui widgets |

## Project Structure

### Documentation (this feature)

```text
specs/055-interactive-task-generation/
├── spec.md              # Feature specification
├── plan.md              # This file
└── tasks.md             # Task breakdown
```

### Source Code Changes

```text
crates/rstn/src/tui/
├── views/
│   └── worktree.rs      # Add TaskListState, task parsing
└── widgets/
    └── task_list.rs     # NEW: TaskList widget for reordering
```

## Design

### Task Format (from tasks.md)

```markdown
- [ ] T001 [P] [US1] Create User model in src/models/user.rs
- [ ] T002 [US1] Implement UserService in src/services/user_service.rs
```

### Task Parsing

```rust
pub struct ParsedTask {
    pub id: String,           // "T001"
    pub is_parallel: bool,    // [P] present
    pub user_story: Option<String>, // "US1"
    pub description: String,
    pub file_path: Option<String>,
    pub completed: bool,      // [X] vs [ ]
}
```

### Key Bindings (in task review mode)

| Key | Action |
|-----|--------|
| `j`/`↓` | Move selection down |
| `k`/`↑` | Move selection up |
| `J` (shift) | Move task down in list |
| `K` (shift) | Move task up in list |
| `e` | Edit selected task |
| `Enter` | Save tasks |
| `Esc` | Cancel |

## Deployment Strategy

**Single PR** - Feature is focused (~200-300 lines)

1. Add `ParsedTask` struct and parsing logic
2. Add `TaskListState` for selection/reordering
3. Add keyboard handlers for reordering
4. Update rendering to show structured list
5. Tests

## Complexity Tracking

No constitution violations expected - simple UI enhancement.
