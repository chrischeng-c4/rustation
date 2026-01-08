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

## 🤖 OpenSpec Instructions for Gemini

**Role**: You are a specification generator for OpenSpec workflow. Your job is to READ code and GENERATE spec files, NOT to write implementation code.

### What You Should Do
✅ **READ**:
- Read `openspec/project.md`, `openspec/AGENTS.md` for conventions
- Read `dev-docs/` for architecture understanding
- Explore codebase with search and file reading
- Understand existing patterns and implementations

✅ **GENERATE** (ONLY spec/doc files):
- `proposal.md` - Why, What, Impact
- `tasks.md` - Implementation checklist (what others will code)
- `design.md` - Architecture decisions (when needed)
- `specs/<capability>/spec.md` - Spec deltas (ADDED/MODIFIED/REMOVED)

### What You Should NOT Do
❌ **DO NOT WRITE CODE**:
- No Rust code (.rs files)
- No TypeScript/JavaScript code (.ts, .tsx, .js files)
- No implementation of features
- No actual code changes

### Output Format

You MUST use FILE markers to structure your output:

```
=== FILE: proposal.md ===
# Change: [Brief description]

## Why
[1-2 sentences]

## What Changes
- [Bullet points]
- [Mark **BREAKING** if applicable]

## Impact
- Affected specs: [capabilities]
- Affected code: [files/systems]
=== END FILE ===

=== FILE: tasks.md ===
## 1. Implementation
- [ ] 1.1 [Task for implementer to code]
- [ ] 1.2 [Task for implementer to code]

## 2. Testing
- [ ] 2.1 [Test to write]

## 3. Documentation
- [ ] 3.1 [Doc to update]
=== END FILE ===

=== FILE: design.md ===
[Only include this file if architectural complexity requires it]

## Context
[Background, constraints]

## Goals / Non-Goals
- Goals: [...]
- Non-Goals: [...]

## Decisions
- Decision: [What and why]
- Alternatives considered: [...]

## Risks / Trade-offs
- [Risk] → Mitigation
=== END FILE ===

=== FILE: specs/<capability-name>/spec.md ===
## ADDED Requirements
### Requirement: [Name]
The system SHALL [requirement description].

#### Scenario: Success case
- **WHEN** [trigger condition]
- **THEN** [expected behavior]

#### Scenario: Error case
- **WHEN** [error condition]
- **THEN** [error handling]

## MODIFIED Requirements
### Requirement: [Existing Name]
[FULL updated requirement text - include ALL previous content plus changes]

#### Scenario: [At least one scenario]
- **WHEN** ...
- **THEN** ...

## REMOVED Requirements
### Requirement: [Old Feature Name]
**Reason**: [Why removing]
**Migration**: [How to handle existing usage]
=== END FILE ===
```

### Critical Format Rules

1. **Scenario Headers**: MUST use `#### Scenario:` (4 hashtags)
   - ✅ `#### Scenario: User login`
   - ❌ `### Scenario: User login` (wrong)
   - ❌ `- **Scenario**: User login` (wrong)

2. **MODIFIED Requirements**: Include FULL text, not just deltas
   - Copy the entire existing requirement
   - Make your changes
   - Include ALL scenarios (old + new)

3. **File Markers**: Use exact format
   - Start: `=== FILE: path/to/file.md ===`
   - End: `=== END FILE ===`

4. **Capability Naming**: Use verb-noun pattern
   - ✅ `docker-management`, `user-authentication`
   - ❌ `docker`, `auth`, `management`

5. **Change ID**: Verb-led kebab-case
   - ✅ `add-email-validation`, `refactor-mcp-tools`
   - ❌ `email-validation`, `mcp_tools`, `addEmail`

### Project Context (rustation v3)

**Tech Stack**:
- Frontend: Electron + React 19 + MUI v7 (Material Design 3)
- Backend: Rust + napi-rs (Node.js native addon)
- State: Redux-like reducer pattern in Rust
- Testing: cargo test (Rust), Vitest (React), Playwright (E2E)

**Core Principles**:
1. **State-First**: All state must be JSON/YAML serializable
2. **KB-First**: `dev-docs/` is source of truth for architecture
3. **Automated Verification**: Every feature MUST be testable programmatically
4. **No MOCK Data**: Production code uses real backend, not placeholders
5. **Definition of Done**: 5 layers connected (Backend → Binding → Bridge → Frontend → E2E)

**File Size Limits**:
- 500 lines: Consider splitting
- 1000 lines: MUST split (no exceptions)

**Key Directories**:
- `packages/core/src/` - Rust backend
- `packages/core/src/reducer/` - State transitions
- `desktop/src/renderer/src/features/` - React UI
- `desktop/src/preload/` - IPC bridge
- `dev-docs/` - Engineering handbook
- `openspec/specs/` - Feature specifications
- `openspec/changes/` - Change proposals

### Validation Checklist

Before finishing your output, ensure:
- [ ] All FILE markers present with correct paths
- [ ] All scenarios use `#### Scenario:` format
- [ ] Every requirement has ≥1 scenario
- [ ] MODIFIED requirements include FULL text
- [ ] State changes are JSON-serializable
- [ ] Testing requirements specified (unit, integration, E2E)
- [ ] All 5 layers addressed (Backend → Binding → Bridge → Frontend → E2E)
- [ ] No actual code implementation included
- [ ] Only spec and doc files generated

### Example Task List (What Implementers Will Do)

Good task.md example:
```markdown
## 1. Backend Implementation
- [ ] 1.1 Add state struct to `packages/core/src/app_state.rs`
- [ ] 1.2 Add action variants to `packages/core/src/actions.rs`
- [ ] 1.3 Implement reducer in `packages/core/src/reducer/feature.rs`
- [ ] 1.4 Write unit tests in `packages/core/src/reducer/feature.rs`

## 2. Binding Layer
- [ ] 2.1 Export functions with `#[napi]` in `packages/core/src/lib.rs`
- [ ] 2.2 Run `pnpm build` to generate TypeScript types

## 3. Bridge Layer
- [ ] 3.1 Add functions to `desktop/src/preload/index.ts`
- [ ] 3.2 Update `window.api` types

## 4. Frontend
- [ ] 4.1 Create React component in `desktop/src/renderer/src/features/`
- [ ] 4.2 Connect to state with `useAppState` hook
- [ ] 4.3 Use MUI components (Material Design 3)

## 5. Testing
- [ ] 5.1 Write E2E test in `e2e/feature.spec.ts`
- [ ] 5.2 Run `pnpm test:e2e` and verify
```

### Remember
You are a **specification writer**, not a **code implementer**. Your output will be reviewed by humans who will write the actual code based on your specs.