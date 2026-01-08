# Change: Fix React Render Errors and Unsupported Views

## Why
The application is experiencing critical runtime errors that degrade stability and user experience:
1.  **React Render Error**: `TerminalPage` crashes because an icon component is passed incorrectly, causing a white screen in the terminal view.
2.  **Action Dispatch Error**: The frontend attempts to navigate to 'claude-code' and 'a2ui' views, but the Rust backend's `ActiveView` enum does not support these variants, causing serialization errors or panic.
3.  **EventEmitter Warning**: The `useAppState` hook creates a new IPC listener on every mount without using a shared context, leading to `MaxListenersExceededWarning` and potential memory leaks.

## What Changes
- **Backend**: Update `ActiveView` enum in `app_state.rs` and `actions.rs` to include `ClaudeCode` and `A2UI`.
- **Frontend**: Fix `TerminalPage.tsx` to pass React Elements (JSX) instead of Component types to `EmptyState`.
- **Frontend**: Refactor `useAppState` to use a React Context (`AppStateProvider`) that manages a single subscription to the Rust backend, resolving the event listener warning.

## Impact
- **Affected specs**: `shared-ui`
- **Affected code**:
    - `packages/core/src/app_state.rs`
    - `packages/core/src/actions.rs`
    - `packages/core/src/reducer/conversions.rs`
    - `desktop/src/renderer/src/features/terminal/TerminalPage.tsx`
    - `desktop/src/renderer/src/hooks/useAppState.ts`
    - `desktop/src/renderer/src/App.tsx` (to add Provider)
    - `desktop/src/renderer/src/components/AppStateProvider.tsx` (new)
