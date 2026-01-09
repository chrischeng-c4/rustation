# Change: Fix File Explorer UI and UX

## Why
The current File Explorer has significant usability issues:
1.  **Navigation Friction**: Users must click the "🏠 Project" button (in `PathBreadcrumbs.tsx`) before any files are displayed. The file list should auto-load when entering Explorer tab.
2.  **Layout Balance**: The file list takes up too much space compared to the content/preview area, making it hard to read code or view details.
3.  **Broken Features**: The comment functionality is present in the UI but fails to persist or display comments correctly.

## What Changes
-   **Auto-load on Tab Entry**: When user navigates to Explorer tab, automatically dispatch `ExploreDir` with the current worktree root path. The "🏠 Project" button remains as a quick way to return to project root.
-   **UI Layout Rebalance**: The File List should be narrower (sidebar-like), and the Detail/Preview panel should occupy the majority of the width.
-   **Comment Fixes**: The `AddFileComment` action flow will be properly connected to the SQLite backend and state reducer.

## Impact
-   **Affected specs**: `file-explorer`
-   **Affected code**:
    -   `desktop/src/renderer/src/features/explorer/ExplorerPage.tsx` (Auto-load + Layout)
    -   `desktop/src/renderer/src/features/explorer/PathBreadcrumbs.tsx` (Project button behavior)
    -   `packages/core/src/reducer/explorer.rs` (Comments)
    -   `packages/core/src/app_state.rs` (Explorer state initialization)
