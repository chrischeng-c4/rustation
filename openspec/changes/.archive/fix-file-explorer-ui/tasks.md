## 1. Auto-load Files on Tab Entry
- [x] 1.1 In `ExplorerPage.tsx`, add `useEffect` to dispatch `ExploreDir` when component mounts
- [x] 1.2 Get worktree root path from `appState.worktrees[appState.activeWorktreeId]`
- [x] 1.3 Only dispatch if `explorer.entries` is empty (avoid re-fetch on tab switch)
- [x] 1.4 Keep the "🏠 Project" button in `PathBreadcrumbs.tsx` (provides visual anchor and quick navigation to root)

## 2. UI Layout Rebalance
- [x] 2.1 Modify `ExplorerPage.tsx` layout: set FileTable container to fixed/resizable width (~300px)
- [x] 2.2 Set DetailPanel container to `flex: 1` to occupy remaining space
- [x] 2.3 Ensure FileTable handles narrow width gracefully (ellipsis for long names)
- [x] 2.4 Fix breadcrumb `current_path` not updating in `reducer/explorer.rs` when navigating
- [x] 2.5 Responsive behavior: flexbox layout with fixed 300px sidebar + flex:1 main panel adapts to window size

## 3. Comment Functionality Fix
- [x] 3.1 Verify `AddFileComment` action exists in `packages/core/src/actions.rs`
- [x] 3.2 Implement/fix `AddFileComment` logic in `packages/core/src/reducer/explorer.rs`
- [x] 3.3 Ensure comment is persisted to SQLite via `db.rs`
- [x] 3.4 Update `ExplorerPage.tsx` to refresh comments list after submission
- [x] 3.5 Verify comment count badge updates in FileTable

## 4. Testing
- [x] 4.1 Write unit test for `ExploreDir` auto-dispatch on mount
- [x] 4.2 Write unit test for `AddFileComment` reducer (already implemented in lib.rs with full E2E flow)
- [x] 4.3 Manual verify: Open Explorer tab -> Files load automatically (no click needed)
- [x] 4.4 Manual verify: Layout is balanced (Preview area > File list width)
- [x] 4.5 E2E test written for comment persistence (e2e/explorer.spec.ts) - backend logic verified in lib.rs:3731
