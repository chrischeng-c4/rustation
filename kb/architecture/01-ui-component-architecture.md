---
title: "UI Component Architecture"
description: "Material Design (MUI) based component architecture for rustation"
category: architecture
status: active
last_updated: 2026-01-06
version: 2.0.0
---

# UI Component Architecture

## 1. Overview

Rustation uses **Material UI (MUI) v7** as its primary design system. This ensures a consistent, professional, and accessible user interface that follows Google's Material Design 3 guidelines.

### Tech Stack
- **Framework**: React 19
- **Design System**: Material UI (MUI) v7 (`@mui/material`)
- **Styling**: Emotion (`@emotion/react`, `@emotion/styled`)
- **Icons**: Material Icons (`@mui/icons-material`)

---

## 2. Design Principles

### 2.1 Material 3 Foundation
We adopt Material 3's key characteristics:
- **Color**: Dynamic color system with clear Primary/Secondary/Surface roles.
- **Typography**: Roboto/Inter based type scale.
- **Shape**: Generous rounding (16px+ for cards, 20px+ for buttons).
- **Elevation**: Usage of tonal surfaces instead of heavy drop shadows.

### 2.2 Component Hierarchy

```
src/renderer/src/
├── components/
│   └── shared/          # Reusable composites (e.g., PageHeader, LogPanel)
├── features/
│   ├── explorer/        # File Explorer feature components
│   ├── docker/          # Docker management components
│   └── ...
├── theme/               # MUI Theme Definitions
│   └── index.ts         # createTheme() configuration
└── App.tsx              # Root with ThemeProvider
```

### 2.3 Composition Rules

1.  **Prefer MUI Primitives**: Use `Box`, `Stack`, `Paper` for layout instead of raw `div`.
2.  **Theming over Custom CSS**: Define colors and shapes in `theme/index.ts`. Avoid hardcoded hex values in components.
3.  **Sx Prop**: Use the `sx` prop for one-off styles that need access to theme tokens.

---

## 3. Core Components

### 3.1 Layout
- **`Box`**: Replacement for `div`. Supports `sx` prop.
- **`Stack`**: Flexbox layout with `spacing`.
- **`Grid`**: 2D grid layout.
- **`Paper`**: Surface container with elevation/outlined variants.

### 3.2 Navigation
- **`Tabs` / `Tab`**: Secondary navigation within features.
- **`Breadcrumbs`**: Path navigation.
- **`IconButton`**: Action triggers (e.g., Back, Refresh).

### 3.3 Feedback
- **`CircularProgress`**: Loading states.
- **`Alert`**: Error/Warning messages.
- **`Snackbar`**: Toast notifications.

---

## 4. Theme Configuration

Defined in `src/renderer/src/theme/index.ts`:

```typescript
export const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: { main: '#D0BCFF' }, // M3 Purple 80
    background: {
      default: '#1C1B1F',
      paper: '#2B2930',
    },
  },
  shape: {
    borderRadius: 16,
  },
  // Component overrides...
})
```

## 5. Migration Strategy (Legacy -> MUI)

When refactoring existing components:
1.  Replace Tailwind class lists with MUI layout components (`Stack`, `Box`, `Grid`) and `sx`.
2.  Replace `lucide-react` icons with `@mui/icons-material`.
3.  Replace custom buttons with `<Button variant="contained">`.
4.  Remove legacy `components/ui` wrappers and all `shadcn/ui` imports.

## 6. References
- [MUI Documentation](https://mui.com/material-ui/getting-started/)
- [Material Design 3](https://m3.material.io/)
