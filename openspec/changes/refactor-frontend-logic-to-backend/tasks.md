## 1. Explorer Refactoring
- [x] 1.1 Update `ExplorerState` in `packages/core/src/app_state.rs` to include `expanded_paths: HashSet<String>` and `directory_cache: HashMap<String, Vec<FileEntry>>`.
- [x] 1.2 Add `ExpandDirectory`, `CollapseDirectory` actions to `packages/core/src/actions.rs`.
- [x] 1.3 Implement `reducer/explorer.rs` handling for expansion (reading directory if not cached).
- [x] 1.4 Update `FileTreeView.tsx` to use `useAppState` for expanded paths and dispatch actions instead of local state.
- [x] 1.5 Remove `listDirectory` from `window.explorerApi` and `desktop/src/preload/index.ts`.

## 2. Chat Refactoring
- [x] 2.1 Update `reducer/chat.rs` to handle `SubmitChatMessage`: add user message to state, then trigger AI effect.
- [x] 2.2 Remove `AddChatMessage` dispatch from `ChatPage.tsx` (frontend only dispatches `SubmitChatMessage`).
- [x] 2.3 Ensure `SubmitChatMessage` is handled as an async action that can trigger subsequent updates.

## 3. Workflows Context Validation
- [x] 3.1 Add `validation_result: Option<ValidationResult>` to `ChangesState`.
- [x] 3.2 Add `ValidateContextFile` action.
- [x] 3.3 Implement validation logic in `reducer/changes.rs` (using `file_reader` internally).
- [x] 3.4 Update `ContextFilesInput.tsx` to dispatch `ValidateContextFile` and observe `changes.validation_result`.

## 4. Dockers Cleanup
- [x] 4.1 Remove mock logic in `AddDbDialog.tsx`.
- [x] 4.2 Ensure `onCreateDb` prop is always provided or default to dispatching `CreateDatabase`.

## 5. Testing
- [x] 5.1 Write Rust unit tests for `ExplorerState` transitions (expand/collapse).
- [x] 5.2 Write Rust unit tests for `SubmitChatMessage` flow.
- [x] 5.3 Write E2E test for Explorer directory expansion (verifying state persistence).
