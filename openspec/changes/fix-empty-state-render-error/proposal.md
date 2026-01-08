# Change: Fix EmptyState Render Error

## Why
The `TasksPage` component is currently crashing with a React rendering error: "Objects are not valid as a React child". This is caused by passing React component functions (e.g., `ListAlt`) directly to the `EmptyState` component's `icon` prop, which expects a rendered `ReactNode` (e.g., `<ListAlt />`).

## What Changes
- **Refactor `TasksPage.tsx`**: Update `EmptyState` usages to pass instantiated icon elements (JSX) instead of component references.
- **Formalize `EmptyState` API**: Create a new `shared-ui` spec to clearly define that `EmptyState` accepts `ReactNode` for icons, ensuring future usages are correct.

## Impact
- Affected specs: `shared-ui` (new)
- Affected code: `desktop/src/renderer/src/features/tasks/TasksPage.tsx`
