# TODOS - rstn Development Roadmap

> This file tracks feature development for rustation.

---

## Track A: File Operations (Complete)

### Phase A1: MVP - Single File Read API (Complete)

- [x] `file_reader.rs` - Rust core module with security validation
- [x] `lib.rs` - napi-rs binding export
- [x] `preload/index.ts` - Frontend bridge (`window.api.file.read`)
- [x] `index.d.ts` - TypeScript type definitions
- [x] `kb/internals/file-reader.md` - KB documentation
- [x] Unit tests (security validation, path traversal prevention)

### Phase A2: Extended Use Cases (Complete)

- [x] Read `constitution.md` and display in Constitution Panel UI
- [x] Read project source code for Claude analysis
- [x] `SourceCodeViewer` component for viewing files
- [x] `ContextFilesInput` component for selecting context files
- [x] Read workflow outputs (proposal/plan files)

---

## Track B: ReviewGate (In Progress)

### Phase B1-B5: Core Implementation & Workflow Integration (In Progress)

- [x] Core Data Model (`ReviewPolicy`, `ReviewSession`, `ReviewComment`)
- [x] Actions & Reducer (all review actions implemented)
- [x] MCP Integration (`submit_for_review`, `get_review_feedback`, `update_review_content`)
- [x] UI Components (`ReviewPanel`, `ContentView`, `CommentsSidebar`, `ActionBar`)
- [x] Change Management workflow integration
- [x] Auto-start review after proposal/plan generation
- [ ] `SubmitReviewFeedback` async pipeline to Claude (Phase B3 TODO in `packages/core/src/lib.rs`)

---

## Track C: Living Context (Complete)

### Phase C1-C2: Implementation (Complete)

- [x] `ContextState` with files array
- [x] `InitializeContext` - create template files
- [x] `GenerateContext` - AI analyzes codebase and generates context
- [x] `context_generate.rs` - codebase summarization module
- [x] Enhanced `SyncContext` - streaming output, multi-file updates
- [x] UI: Two initialization options (AI generation vs templates)

---

## Track D: Testing (In Progress)

### E2E Tests (Synchronized)

- [x] Constitution workflow tests
- [x] Agent Rules tests
- [x] Workflows page tests
- [x] Docker workflow E2E tests
- [x] Env management E2E tests
- [x] Claude Code / Chat integration E2E tests
- [x] Command Palette E2E tests

### Known Test Environment Gaps

- [ ] Investigate sandbox permission issue for TCP bind (MCP server start/stop test skips in restricted env)
- [ ] Electron SIGABRT on macOS during RegisterApplication in sandboxed test runs (E2E cannot launch)
- [ ] Electron runs fine via `pnpm -C apps/desktop dev` (GUI) but SIGABRT via Playwright launch; isolate Playwright/loader vs headless env cause

### Phase D2: Synchronize E2E tests with UI Refactoring (Complete)

- [x] Update `workflows-page.spec.ts`
- [x] Update `docker.spec.ts`
- [x] Update `env.spec.ts`
- [x] Update `claude-code.spec.ts`
- [x] Update `command-palette.spec.ts`

---

## Track E: UI Component Standardization (Complete)

### Phase E1: Architecture & Library Setup (Complete)

- [x] `kb/architecture/01-ui-component-architecture.md` - Formalize component hierarchy and rules
- [x] Audit `src/components/ui` for completeness (Added Select, Switch, Progress, Accordion, etc.)
- [x] Create `src/components/shared` for generic composite components

### Phase E2: Component Migration (Complete)

- [x] Move domain-specific components to features (Projects, Command Palette)
- [x] Move shared composite components to `src/components/shared/` (LogPanel, SourceCodeViewer, PageHeader, etc.)
- [x] Clean up `src/components/` root

### Phase E3: Enforcement & Polish (Complete)

- [x] Update `kb/workflow/definition-of-done.md` with component placement rules
- [x] Add Tech Stack presets to Constitution workflow
- [x] Refactor ALL feature components to use standardized shared components:
    - [x] `ConstitutionPanel.tsx`
    - [x] `TasksPage.tsx`
    - [x] `DockersPage.tsx`
    - [x] `McpPage.tsx`
    - [x] `TerminalPage.tsx`
    - [x] `WorkflowsPage.tsx`
    - [x] `ChangeManagementPanel.tsx`
    - [x] `ContextPanel.tsx`
    - [x] `ReviewPanel.tsx`
    - [x] `ChatPage.tsx`

---

## Track F: Experimental Features (In Progress)

### Phase F1: A2UI Integration (In Progress)

- [x] Implement recursive `A2UIRenderer`
- [x] Define `A2UI_REGISTRY` mapping to standardized components
- [x] Integrate `A2UI` view into main navigation
- [x] Document architecture in `kb/experimental/a2ui.md`
- [ ] MCP tool + backend bridge to accept A2UI JSON and push to renderer

---

## Track G: Spec / State / Test Gaps (New)

### Phase G1: KB State Machine Coverage (Missing)

- [x] ReviewGate session state machine (pending -> reviewing -> iterating -> approved/rejected)
- [x] Constitution workflow state machine (collecting -> generating -> complete + template path)
- [x] Living Context state machine (init/load/refresh/sync/generate/fail)
- [x] Tasks/Justfile execution state machine (idle -> running -> success/error)
- [x] Env management state machine (idle -> copying -> success/error)
- [x] Notifications lifecycle state machine (new -> read -> dismissed/cleared)
- [x] Terminal session state machine (spawn -> active -> resize -> killed/error)
- [x] Command palette state machine (closed -> open -> filtering -> execute)
- [x] A2UI payload state machine (idle -> rendering -> error)
- [x] MCP inspector UI state machine (status/log stream)

### Phase G2: KB <-> Implementation Mismatches

- [x] Constitution KB deprecates Q&A, but implementation and E2E use Q&A (align KB or remove)
- [x] ChangeStatus includes `planning` in code but not in CESDD state machine doc

### Phase G3: State Transition + Integration Tests (Rust)

- [ ] Reducer tests for MCP actions (start/stop/status/port/error/log/tools)
- [ ] Reducer tests for Notifications actions (add/dismiss/mark read/clear)
- [ ] Reducer tests for Terminal actions (spawn/resize/set session/kill)
- [ ] Reducer tests for Context actions (load/set/init/generate/sync/complete/fail)
- [ ] Reducer tests for Change edge transitions (cancel/fail/archive)
- [ ] Serialization round-trip tests for ContextState/McpState/Notification/TerminalState
- [ ] Automate napi integration test (move `packages/core/test-mcp-tools.mjs` into test runner)

### Phase G4: Property-Based Tests (Complex Logic)

- [ ] Add proptest for worktree parsing + path validation invariants

### Phase G5: E2E Full-Flow Coverage (Must Run End-to-End)

- [ ] Change Management full flow (proposal -> review -> plan -> approve -> execute -> done)
- [ ] Living Context generate + sync full flow
- [ ] ReviewGate full flow (comment -> submit feedback -> approve/reject)
- [ ] Tasks/Justfile run command flow
- [ ] Project/worktree management flow (open/switch/add/remove)
- [ ] MCP inspector flow (start/stop/log/tools)
- [ ] Terminal session flow (spawn/resize/kill)
- [ ] Notifications flow (toast -> drawer -> mark read)
- [ ] A2UI flow via backend push
- [ ] Remove/replace skipped legacy constitution E2E or justify skip

---

## Technical Notes

### UI Architecture
Follow the **Composition over Creation** principle. Feature components should be composed of `ui/` primitives and `shared/` composites.

### ReviewGate Flow
Workflow Node -> CC CLI -> rstn-mcp submit_for_review -> ReviewSession -> Approve/Reject.
