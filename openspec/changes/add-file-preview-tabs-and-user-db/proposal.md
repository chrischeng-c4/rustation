# Change: Add File Preview Tabs and User-Scope Database

## Why
Currently, the File Explorer only supports viewing a single file at a time, which makes multi-file workflows (like comparing code or referencing definitions) difficult. Users expect standard tab behaviors found in IDEs like VSCode.

Additionally, the SQLite database is currently stored per-project inside `.rstn/rstn.db`. This scatters data across the filesystem and makes it harder to manage global application state or perform cross-project queries in the future. Since `rustation` is a single-instance application managing multiple worktrees, a centralized user-scoped database is more appropriate.

## What Changes
1.  **File Preview Tabs**:
    -   Implement VSCode-style tabs in the Detail Panel.
    -   **Preview Tab**: Opened on single-click, italicized title, reused for subsequent single-clicks.
    -   **Pinned Tab**: Created on double-click (or double-clicking a preview tab), normal title, persists until closed.
    -   No limit on the number of pinned tabs.
    -   Close button (x) on hover/active.

2.  **User-Scope Database**:
    -   **BREAKING**: Move SQLite database from `<project>/.rstn/rstn.db` to `~/.rstn/state.db`.
    -   **Fresh Start**: No migration of existing data from project-local databases. Old data will be ignored.
    -   Add `project_id` column to all tables (`file_comments`, `activity_logs`) to segregate data.
    -   Update `DbManager` to connect to the global database instance.

## Impact
-   **Affected specs**:
    -   `file-explorer` (UI behavior)
    -   `project-management` (Persistence layer)
-   **Affected code**:
    -   Frontend: `ExplorerPage.tsx`, `DetailPanel.tsx`, `explorer` reducer state.
    -   Backend: `db.rs`, `persistence.rs`, `DbManager`.
