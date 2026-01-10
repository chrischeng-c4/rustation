# Change: Fix File Preview & Add Inline Comments

## Why
The current file preview experience is suboptimal:
1.  **No syntax highlighting**: Code is hard to read.
2.  **Scrolling issues**: Large files are difficult to navigate due to layout nesting issues.
3.  **No inline comments**: Code review is limited to file-level comments, lacking precision.

## What Changes
- **Backend**:
    - Update `Comment` struct to include optional `line_number`.
    - Update SQLite schema to store `line_number`.
    - Update `AddFileComment` action to accept `line_number`.
- **Frontend**:
    - integrate `prismjs` for syntax highlighting in `SourceCodeViewer`.
    - Refactor `SourceCodeViewer` to render line numbers and content with support for inline widgets.
    - Add UI for adding/viewing inline comments (click gutter/line number).
    - Fix scrolling by ensuring proper flex/overflow propagation in `DetailPanel` and `SourceCodeViewer`.

## Impact
- **Affected specs**: `file-explorer`, `shared-ui`
- **Affected code**:
    - `packages/core/src/app_state.rs`
    - `packages/core/src/actions.rs`
    - `packages/core/src/db.rs`
    - `packages/core/src/explorer/mod.rs`
    - `desktop/src/renderer/src/features/explorer/DetailPanel.tsx`
    - `desktop/src/renderer/src/components/shared/SourceCodeViewer.tsx`
