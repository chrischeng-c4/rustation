## 1. Implementation
- [x] 1.1 Update `desktop/src/renderer/src/features/tasks/TasksPage.tsx` to pass `<ListAlt />` and `<Refresh />` as `icon` props to `EmptyState`.
- [x] 1.2 Verify `desktop/src/renderer/src/features/dockers/DockersPage.tsx` and `desktop/src/renderer/src/features/explorer/ExplorerPage.tsx` for correctness (already checked, but good to confirm).
- [x] 1.3 Add missing TypeScript type definitions for File Explorer (FileExplorerState, FileEntry, GitFileStatus, etc.)
- [x] 1.4 Update WorktreeState interface to include explorer field
- [x] 1.5 Add Explorer actions to Action union type
- [x] 1.6 Fix react-window import issues in FileTable component
- [x] 1.7 Fix ExplorerPage render logic order (check !worktree before !explorer)

## 2. Testing
- [x] 2.1 Verify `TasksPage` renders the "No Project Selected" empty state without errors.
- [x] 2.2 Verify `TasksPage` renders the "No Commands" empty state without errors (requires an empty justfile project).
- [x] 2.3 Update File Explorer test mocks to match complete TypeScript type definitions
- [x] 2.4 All File Explorer unit tests passing (4/4 tests pass)
