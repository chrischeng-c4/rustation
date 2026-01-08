## 1. Backend Implementation
- [x] 1.1 Add `ClaudeCode` and `A2UI` variants to `ActiveView` enum in `packages/core/src/app_state.rs`.
- [x] 1.2 Add `ClaudeCode` and `A2UI` variants to `ActiveViewData` enum in `packages/core/src/actions.rs`.
- [x] 1.3 Update `From<ActiveViewData>` implementation for `ActiveView` in `packages/core/src/app_state.rs` to handle new variants.
- [x] 1.4 Run `cargo check` to ensure no exhaustiveness errors.

## 2. Frontend Implementation
- [x] 2.1 Fix `TerminalPage.tsx` render error:
    - Change `icon={Terminal}` to `icon={<Terminal fontSize="large" />}` in `EmptyState` props.
    - Change `action={{ ..., icon: Terminal }}` to `action={{ ..., icon: <Terminal /> }}`.
- [x] 2.2 Create `desktop/src/renderer/src/components/AppStateProvider.tsx`:
    - Implement `AppStateContext`.
    - Move `stateApi.onStateUpdate` logic here (single subscription).
    - Provide `state`, `dispatch`, `isLoading` via Context.
- [x] 2.3 Update `desktop/src/renderer/src/main.tsx` to wrap the application with `AppStateProvider`.
- [x] 2.4 Refactor `desktop/src/renderer/src/hooks/useAppState.ts`:
    - Remove direct `stateApi` subscription.
    - Use `useContext(AppStateContext)`.
    - Throw error if used outside provider.

## 3. Testing
- [x] 3.1 Verify Terminal page loads the Empty State correctly without crashing.
- [x] 3.2 Click "Claude" and "A2UI" in sidebar and verify backend logs/state accept the change (no panic/error).
- [x] 3.3 Check DevTools console to confirm `MaxListenersExceededWarning` is gone after navigating between pages multiple times.
