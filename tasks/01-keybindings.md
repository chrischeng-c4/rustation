---
title: "Implement Keybinding System & Global Actions"
status: "done"
priority: "high"
last_updated: 2025-12-23
---

# Task: Implement Keybinding System & Global Actions

## Source
- `kb/architecture/08-keybindings.md`
- `kb/architecture/02-state-first-mvi.md`

## Todo List
- [x] Add `CopyContentRequested` and `CopyStateRequested` to Messages.
- [x] Add `CopyToClipboard` to Effects.
- [x] Implement global key handlers in TUI for:
    - `q` / `Ctrl-c` (Quit)
    - `y` (Copy Visual Content)
    - `Y` (Copy Full State JSON)
- [x] Implement Worktree-specific navigation keys (`j`, `k`, `Tab`, `Enter`).
- [x] Implement Content-specific scrolling keys (`h`, `l`).
