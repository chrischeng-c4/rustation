## 1. Backend Implementation (Database)
- [x] 1.1 Update `packages/core/src/db.rs` to use `dirs::home_dir()` for `state.db` path instead of project path.
- [x] 1.2 Update `DbManager::init` to accept a `project_id` (hash) context or handle it in methods.
- [x] 1.3 Update SQL migrations in `db.rs` to add `project_id` column to `file_comments` and `activity_logs`.
- [x] 1.4 Update `add_comment`, `get_comments`, `add_log`, `get_logs` queries to filter/insert by `project_id`.
- [x] 1.5 Update `packages/core/src/app_state.rs` to generate/store a stable `project_id` (likely the path hash).

## 2. Backend Implementation (Tabs State)
- [x] 2.1 Update `ExplorerState` in `packages/core/src/app_state.rs` to include `open_tabs: Vec<FileTab>` and `active_tab_path: Option<String>`.
- [x] 2.2 Define `FileTab` struct with `path`, `is_pinned`, and `scroll_pos`.
- [x] 2.3 Implement `ExplorerAction::OpenFile` (single click), `ExplorerAction::PinTab` (double click), `ExplorerAction::CloseTab` in `packages/core/src/reducer/explorer.rs`.
- [x] 2.4 Implement tab selection logic:
    -   If file not open: Open as preview (replacing existing preview if any).
    -   If file open: Activate it.
    -   If double click: Mark as pinned.

## 3. Frontend Implementation
- [x] 3.1 Create `FileTabs` component in `desktop/src/renderer/src/features/explorer/FileTabs.tsx` using MUI `Tabs`/`Tab`.
- [x] 3.2 Update `ExplorerPage.tsx` to include `FileTabs` above `DetailPanel`.
- [x] 3.3 Update `DetailPanel.tsx` to render content based on `active_tab_path`.
- [x] 3.4 Wire up click events in `FileTreeView` to dispatch `OpenFile` / `PinTab`.
- [x] 3.5 Implement tab closing and switching logic.
- [x] 3.6 Style tabs to match VSCode (italic for preview, normal for pinned).

## 4. Testing
- [x] 4.1 Unit Test: Rust reducer logic for tab opening/replacing/pinning (existing tests pass).
- [x] 4.2 Unit Test: DB queries filter correctly by `project_id` (existing tests pass).
- [ ] 4.3 E2E: Verify single click opens preview, second single click replaces it.
- [ ] 4.4 E2E: Verify double click pins tab.
- [ ] 4.5 E2E: Verify comments persist to new global DB location.
