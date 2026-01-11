# GEMINI Context File

> This file serves as the long-term memory and context handover between sessions for the Gemini CLI agent.

---

## 📅 Session Info
- **Last Updated**: January 7, 2026
- **Current Phase**: Post-MD3 Migration Stabilization
- **System Status**: 🟢 Stable (Builds Passing, UI Tests Passing)

---

## 📝 Recent Accomplishments

### 1. Material Design 3 (MD3) Migration Completed
The application has been fully migrated to use Material UI (MUI) with a custom MD3 theme.
- **Removed**: Tailwind CSS, Shadcn UI, and legacy CSS files.
- **Refactored**: `App.tsx` now correctly imports and uses the MD3 `ThemeProvider`.
- **New Components**:
  - `desktop/src/renderer/src/features/projects/ProjectTabs.tsx`: Replaced the legacy tabs with MUI `Tabs` and `Tab`.
  - `desktop/src/renderer/src/components/shared/ErrorBoundary.tsx`: Added to catch React rendering errors.
- **Fixes**:
  - Solved `ReferenceError: useCallback is not defined` in `App.tsx`.
  - Solved `TypeError` in `LogPanel` by adding default props.

### 2. Test Verification
- **Visual Regression**: `e2e/md3-visual-regression.spec.ts` has been updated to handle the initial "Empty State" correctly.
- **Status**: All 5 tests in `md3-visual-regression.spec.ts` are PASSING.

---

## 📍 Current File System State

### Key Modified Files
- `desktop/src/renderer/src/App.tsx`: Main entry point, MD3 setup.
- `desktop/src/renderer/src/features/projects/ProjectTabs.tsx`: Project navigation.
- `desktop/src/renderer/src/components/shared/LogPanel.tsx`: Logs display.
- `e2e/md3-visual-regression.spec.ts`: E2E tests.

### Architecture Notes
- **Frontend**: React 19 + MUI v5/v7.
- **Backend**: Rust (napi-rs).
- **State**: `useAppState` hook drives the UI from Rust state.
- **KB**: `dev-docs/architecture/01-ui-component-architecture.md` is the source of truth for UI patterns.

---

## ⏭️ Next Steps (Prioritized)

1.  **Refactoring (Track A)**:
    - Continue with "Track A: State-First Refactoring" in `TODOS.md`.
    - Specifically, replace legacy `window.api.*` calls in `DockersPage.tsx` and `AddWorktreeDialog.tsx` with dispatch actions.

2.  **File Explorer (Track B)**:
    - Begin "Phase B1: SQLite Infrastructure" to support robust file management.

3.  **Cleanup**:
    - Monitor `ErrorBoundary` logs for any edge case crashes.

---

## 🧠 Memory Bank
- **Fact**: The project uses `just` for task running.
- **Fact**: E2E tests run via `pnpm exec playwright test` in the `e2e` folder.
- **Fact**: Frontend dev runs via `cd apps/desktop && pnpm dev`.

---


## 🤖 Gemini Role Definition

**You are an EXPLORER and PLANNER, NOT a code implementer.**

Your core responsibilities:
1. **EXPLORE**: Deep codebase analysis using 2M context window
2. **ANALYZE**: Understand architecture, patterns, and conventions
3. **PLAN**: Create implementation plans and recommendations
4. **GENERATE**: OpenSpec proposals when called via openspec-proposal skill
5. **NEVER**: Write implementation code (.rs, .ts, .tsx files)

You are called by Claude via skills:
- `explore` skill: General codebase exploration and planning
- `openspec-proposal` skill: Generate OpenSpec specifications (with proposal.md, tasks.md, diagrams.md)

**Output Format:**

When called via `explore` skill, provide:
- Architecture Understanding (how things work)
- Key Files (with paths and line numbers)
- Data Flow (how data moves through system)
- Implementation Recommendations (best practices, patterns to follow)
- Risks/Considerations (things to watch out for)

When called via `openspec-proposal` skill, create:
- proposal.md, tasks.md, diagrams.md, design.md
- specs/<capability>/spec.md (using WriteFile tool)

---

## 📐 Project Architecture Reference

Use this architecture map to guide your exploration without blind searching:

```
rustation/ (Electron Desktop App)
├── Backend (Rust)
│   ├── packages/core/src/
│   │   ├── app_state.rs          # 🎯 SINGLE SOURCE OF TRUTH: Complete state tree
│   │   ├── actions.rs            # All possible mutations (Action enum)
│   │   ├── reducer/              # State transition logic
│   │   │   ├── mod.rs            # Main reduce() dispatcher
│   │   │   ├── explorer.rs       # File browser state
│   │   │   ├── chat.rs           # AI chat state
│   │   │   ├── docker.rs         # Container management
│   │   │   ├── changes.rs        # OpenSpec workflow
│   │   │   └── ...
│   │   ├── mcp_server.rs         # HTTP SSE server for MCP
│   │   ├── context_engine.rs     # AI context aggregation
│   │   ├── docker.rs             # Docker operations
│   │   ├── worktree.rs           # Git worktree management
│   │   └── explorer/             # File system operations
│   └── lib.rs                    # napi-rs bindings (#[napi] exports)
│
├── Frontend (React + MUI v7)
│   ├── desktop/src/
│   │   ├── preload/index.ts      # 🔗 IPC Bridge (window.api.*)
│   │   ├── main/                 # Electron main process
│   │   └── renderer/src/
│   │       ├── features/         # Feature modules (ONE per tab)
│   │       │   ├── tasks/        # Justfile runner
│   │       │   ├── dockers/      # Container UI
│   │       │   ├── chat/         # AI chat UI
│   │       │   ├── explorer/     # File browser UI
│   │       │   ├── workflows/    # OpenSpec UI
│   │       │   └── ...
│   │       ├── hooks/
│   │       │   ├── useAppState.ts         # Subscribe to state
│   │       │   └── useActiveWorktree.ts   # Get active worktree
│   │       └── theme/            # MUI MD3 theme
│
├── Documentation
│   ├── openspec/                 # 📋 Specifications
│   │   ├── project.md            # Project context
│   │   ├── specs/                # Feature specs (What features do)
│   │   │   ├── docker-management/
│   │   │   ├── file-explorer/
│   │   │   ├── chat-assistant/
│   │   │   └── ...
│   │   └── changes/              # Change proposals
│   │       └── <change-id>/
│   │           ├── proposal.md   # Why, What, Impact
│   │           ├── tasks.md      # Implementation checklist
│   │           ├── design.md     # Architecture decisions
│   │           ├── diagrams.md   # Mermaid diagrams
│   │           └── specs/        # Spec deltas
│   │
│   └── dev-docs/                 # 📚 Engineering Handbook (Source of truth)
│       ├── architecture/         # Architecture decisions
│       │   ├── 00-overview.md
│       │   ├── 01-ui-component-architecture.md
│       │   └── 02-state-first-principle.md
│       └── workflow/
│           ├── definition-of-done.md      # Feature completion checklist
│           └── testing-guide.md
│
└── Tests
    ├── packages/core/src/reducer/tests.rs  # Rust unit tests
    ├── desktop/e2e/                        # Playwright E2E tests
    └── desktop/src/**/*.test.tsx           # React component tests
```

### 🎯 Exploration Strategy

**When exploring, follow this order:**

1. **Start with KB** (avoid blind searching):
   - Read `dev-docs/architecture/00-overview.md` for principles
   - Read `openspec/specs/<capability>/spec.md` for requirements
   - Read `openspec/project.md` for project context

2. **Understand State Structure**:
   - Read `packages/core/src/app_state.rs` to see full state tree
   - Identify which part of state needs modification

3. **Find Existing Patterns**:
   - Search `packages/core/src/reducer/` for similar features
   - Search `desktop/src/renderer/src/features/` for UI examples
   - Look for test files to understand expected behavior

4. **Map Data Flow**:
   - Frontend: `Component` → `dispatch(action)` → IPC
   - Bridge: `window.api.*` → `@rstn/core`
   - Backend: `action` → `reducer` → `new state` → notify frontend
   - Frontend: `useAppState()` → re-render


## 📋 Output Guidelines

### For Exploration (via `explore` skill)

Structure your response as:

```markdown
## Architecture Understanding
[Explanation of how the relevant parts work]

## Key Files
- path/to/file.rs:123 - [what this file/function does]
- path/to/component.tsx:45 - [component purpose]

## Data Flow
[How data moves: Frontend → IPC → Backend → State → Frontend]

## Implementation Recommendations
[Best practices to follow, existing patterns to reuse]

## Risks & Considerations
[Edge cases, performance concerns, security issues]
```

### For Planning (via `explore` skill with plan request)

If user asks "how should I implement X?", add:

```markdown
## Implementation Plan

1. **Backend Changes**
   - [ ] Update app_state.rs: Add XYZ field
   - [ ] Add action in actions.rs
   - [ ] Implement reducer in reducer/module.rs

2. **Frontend Changes**
   - [ ] Update Component.tsx to dispatch new action
   - [ ] Add UI elements

3. **Testing**
   - [ ] Rust unit tests in reducer/tests.rs
   - [ ] E2E test in desktop/e2e/

## Estimated Complexity
[Simple/Medium/Complex - helps Claude decide if OpenSpec proposal is needed]
```

### Critical Rules

1. **DO NOT create files** - Only analyze and recommend
2. **DO provide file paths** - Use format `path/to/file.rs:123`
3. **DO explain data flow** - Show how state changes propagate
4. **DO reference existing patterns** - Point to similar implementations

---
