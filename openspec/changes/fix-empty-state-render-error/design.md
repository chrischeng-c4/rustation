## Context
The `EmptyState` component is a shared UI utility used across multiple features (Tasks, Dockers, Explorer). It is designed to accept a `ReactNode` for its `icon` prop to allow for pre-configured icons (color, size, props) to be passed in.

## Decisions
- **Decision**: Enforce `ReactNode` (Element) passing for `icon` prop.
  - **Reason**: Passing an Element (`<Icon sx={{...}} />`) is more flexible than passing a Component type (`Icon`) because it allows the caller to style the icon contextually without `EmptyState` needing to expose complex styling props for the icon.
  - **Alternative Considered**: Modifying `EmptyState` to accept `React.ElementType` and instantiating it internally.
    - **Rejection**: This would complicate the `EmptyState` implementation and limit the caller's ability to customize the icon instance (e.g., adding specific ARIA labels or `sx` props directly to the icon).

## Risks
- **Risk**: Developers might continue to pass Component types by mistake.
  - **Mitigation**: TypeScript types define `icon` as `React.ReactNode`, which technically includes `ReactElement` but *not* `() => JSX.Element`. TypeScript should ideally warn about this, but strictness settings might be allowing it or `MUI` icon types are broad. Explicit spec documentation helps clarify this pattern.
