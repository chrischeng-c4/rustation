# Implementation Tasks

## 1. Backend (Rust)
- [x] 1.1 Verify `EnvConfig` in `ProjectState` supports "View/Edit" requirements (or add `EnvState` if variables need to be loaded).
- [x] 1.2 Ensure `ActiveView` transitions correctly handle Project vs. Worktree context switching.

## 2. Frontend (React/MUI)
- [x] 2.1 Refactor `ProjectTabs` to be the top-level navigation bar (Level 1).
- [x] 2.2 Implement `WorktreeTabs` as a secondary bar below Project Tabs (Level 2).
- [x] 2.3 Create/Update `EnvManagement` view at the Project level (accessible when a project is active, applying to all worktrees).
- [x] 2.4 Ensure `Docker` view is accessible globally (e.g., via a global sidebar or a dedicated "System" project tab).
- [x] 2.5 Add 7 Global Icon Buttons (Copy, Screenshot, Download, Notifications, Logs, Docker, Settings).

## 3. Migration
- [x] 3.1 Migrate existing single-level navigation to the new two-tier system.
- [x] 3.2 Update E2E tests to navigate the new hierarchy.

## 4. Documentation
- [x] 4.1 Update User Guide to explain the Project -> Worktree hierarchy.
