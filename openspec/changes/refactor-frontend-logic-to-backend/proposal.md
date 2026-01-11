# Change: Refactor Frontend Logic to Backend

## Why
Currently, several frontend features violate the "State-First" architecture by managing business logic, data fetching, and state mutations directly in React. This leads to:
- **Inconsistent State**: Local React state is lost on refresh/navigation.
- **Logic Duplication**: Validation logic exists in both frontend and backend.
- **Testing Difficulty**: Business logic in React components is harder to test than Rust pure functions.
- **Tight Coupling**: Components are coupled to specific data sources (e.g., file viewer state used for context validation).

## What Changes
- **Explorer**: Move `expandedPaths` and `directoryCache` to Rust `ExplorerState`. Replace `window.explorerApi.listDirectory` with `ExpandDirectory` action.
- **Chat**: Remove optimistic "Add User Message" logic. Introduce `SubmitChatMessage` action where backend handles message addition and AI trigger.
- **Workflows**: Remove dependency on `file_viewer` for context file validation. Add `ValidateContextFile` action and dedicated state.
- **Dockers**: Remove mock logic for database creation in `AddDbDialog`.
- **Breaking**: `ExplorerState` structure will change to include expansion state.

## Impact
- **Affected specs**: `file-explorer`, `shared-ui`, `docker-management`
- **Affected code**:
    - `desktop/src/renderer/src/features/explorer/FileTreeView.tsx`
    - `packages/core/src/app_state.rs` (ExplorerState)
    - `packages/core/src/reducer/explorer.rs`
    - `desktop/src/renderer/src/features/chat/ChatPage.tsx`
    - `packages/core/src/reducer/chat.rs`
    - `desktop/src/renderer/src/features/workflows/ContextFilesInput.tsx`
    - `desktop/src/renderer/src/features/dockers/AddDbDialog.tsx`
