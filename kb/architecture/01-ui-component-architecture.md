---
title: "UI Component Architecture"
description: "Architecture for the frontend component library and feature composition"
category: architecture
status: draft
last_updated: 2025-01-04
version: 1.0.0
tags: [architecture, frontend, react, components, shadcn]
weight: 2
---

# UI Component Architecture

## 1. Core Principle

The frontend follows a strict **Composition over Creation** principle.
- **Fixed Component Library**: We use a standardized set of UI primitives (shadcn/ui).
- **No Ad-Hoc Components**: Specialized components must be composed of existing primitives.
- **Feature Encapsulation**: Domain logic and feature-specific UI belong in `features/`, not `components/`.

## 2. Directory Structure

```
apps/desktop/src/renderer/src/
├── components/          # GLOBAL SHARED COMPONENTS
│   ├── ui/              # ✅ THE "FIXED" LIBRARY (shadcn/ui)
│   │   ├── button.tsx
│   │   ├── dialog.tsx
│   │   └── ...
│   ├── shared/          # ✅ SHARED COMPOSITE COMPONENTS
│   │   ├── LogPanel.tsx
│   │   ├── SourceCodeViewer.tsx
│   │   └── ...
│   └── (No other files here) ❌
│
├── features/            # FEATURE MODULES
│   ├── command-palette/ # Command palette feature
│   ├── dockers/
│   │   ├── components/  # Feature-specific components
│   │   │   └── ServiceCard.tsx
│   │   └── index.tsx    # Public API
│   ├── projects/        # Project and Worktree management
│   │   ├── components/
│   │   │   ├── ProjectTabs.tsx
│   │   │   └── AddWorktreeDialog.tsx
│   │   └── ...
│   └── ...
```

## 3. Rules

### 3.1 The "Fixed" Library (`components/ui`)
- Contains **only** generic, style-agnostic primitives.
- Based on `shadcn/ui`.
- Modifications here affect the **entire application**.
- **DO NOT** add business logic here.

### 3.2 Shared Components (`components/shared`)
- Components used by **multiple features** (e.g., `LogPanel` used by Docker and Tasks).
- Must be generic and reusable.
- Should accept props for data, not fetch data themselves (dumb components).

### 3.3 Feature Components (`features/*/components`)
- Components specific to a single feature.
- Can contain business logic or be tied to specific data structures.
- Example: `AddWorktreeDialog` belongs to `features/projects` because it knows about "worktrees" and calls specific APIs.

## 4. Workflow Composition

Workflows (views) are composed by assembling components from the Fixed Library and Feature Components.

**Example (Docker Page)**:
```tsx
// features/dockers/DockersPage.tsx
import { Button } from '@/components/ui/button' // Fixed Library
import { LogPanel } from '@/components/shared/LogPanel' // Shared Component
import { ServiceList } from './components/ServiceList' // Feature Component

export function DockersPage() {
  return (
    <div className="p-4">
      <ServiceList />
      <LogPanel />
      <Button>Refresh</Button>
    </div>
  )
}
```

## 5. Migration Strategy

1.  **Phase 1**: Create `src/components/shared` directory.
2.  **Phase 2**: Create `features/projects/components` directory.
3.  **Phase 3**: Move `LogPanel.tsx`, `DevLogPanel.tsx`, `SourceCodeViewer.tsx` to `shared/`.
4.  **Phase 4**: Move `ProjectTabs.tsx`, `AddWorktreeDialog.tsx` to `features/projects/components/`.
5.  **Phase 5**: Update all imports and verify build.
