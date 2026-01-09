## 1. Auto-load Files on Tab Entry
- [ ] 1.1 In `ExplorerPage.tsx`, add `useEffect` to dispatch `ExploreDir` when component mounts
- [ ] 1.2 Get worktree root path from `appState.worktrees[appState.activeWorktreeId]`
- [ ] 1.3 Only dispatch if `explorer.entries` is empty (avoid re-fetch on tab switch)
- [ ] 1.4 Remove or simplify the "Project" button in `PathBreadcrumbs.tsx` (keep as home navigation)

## 2. UI Layout Rebalance
- [ ] 2.1 Modify `ExplorerPage.tsx` layout: set FileTable container to fixed/resizable width (~300px)
- [ ] 2.2 Set DetailPanel container to `flex: 1` to occupy remaining space
- [ ] 2.3 Ensure FileTable handles narrow width gracefully (ellipsis for long names)
- [ ] 2.4 Test responsive behavior on different window sizes

## 3. Comment Functionality Fix
- [ ] 3.1 Verify `AddFileComment` action exists in `packages/core/src/actions.rs`
- [ ] 3.2 Implement/fix `AddFileComment` logic in `packages/core/src/reducer/explorer.rs`
- [ ] 3.3 Ensure comment is persisted to SQLite via `db.rs`
- [ ] 3.4 Update `ExplorerPage.tsx` to refresh comments list after submission
- [ ] 3.5 Verify comment count badge updates in FileTable

## 4. Testing
- [ ] 4.1 Write unit test for `ExploreDir` auto-dispatch on mount
- [ ] 4.2 Write unit test for `AddFileComment` reducer
- [ ] 4.3 Manual verify: Open Explorer tab -> Files load automatically (no click needed)
- [ ] 4.4 Manual verify: Layout is balanced (Preview area > File list width)
- [ ] 4.5 Manual verify: Add comment -> Reload -> Comment persists
