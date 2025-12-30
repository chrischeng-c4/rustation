# TODOS - rstn Development Roadmap

> This file tracks feature development for rustation.

---

## Track A: File Operations

### Phase A1: MVP - Single File Read API (Complete)

Basic file reading capability for all workflows.

- [x] `file_reader.rs` - Rust core module with security validation
- [x] `lib.rs` - napi-rs binding export
- [x] `preload/index.ts` - Frontend bridge (`window.api.file.read`)
- [x] `index.d.ts` - TypeScript type definitions
- [x] `kb/internals/file-reader.md` - KB documentation
- [x] Unit tests (security validation, path traversal prevention)

**API**: `window.api.file.read(path: string, projectRoot: string): Promise<string>`

**Security Scope**: Project directory + `~/.rstn/`

### Phase A2: Extended Use Cases (Complete)

Apply file reading to specific workflow scenarios.

- [x] Read `constitution.md` and display in Constitution Panel UI
- [x] Read project source code for Claude analysis
  - `SourceCodeViewer` component for viewing files
  - `ContextFilesInput` component for selecting context files
  - Claude context injection in proposal/plan generation
- [x] Read workflow outputs (proposal/plan files) - Already cached in Change state

### Phase A3: Additional File Operations (Future)

Expand file operation capabilities.

- [ ] File listing API (`window.api.file.list`)
- [ ] Directory tree API
- [ ] File metadata API (size, mtime, permissions)
- [ ] General file browser UI component (deferred from A2)
- [ ] File write API (with security scope validation)

---

## Track B: ReviewGate (Complete)

Human-in-the-loop 審核機制，讓 workflow 產出在落地前經過審核。

**KB 文件**: `kb/architecture/14-review-gate.md`

### Phase B1-B4: Core Implementation (Complete)

- [x] Core Data Model (`ReviewPolicy`, `ReviewSession`, `ReviewComment`)
- [x] Actions & Reducer (all review actions implemented)
- [x] MCP Integration (`submit_for_review`, `get_review_feedback`, `update_review_content`)
- [x] UI Components (`ReviewPanel`, `ContentView`, `CommentsSidebar`, `ActionBar`)

### Phase B5: Workflow Integration (Complete)

- [x] Change Management workflow 整合 ReviewGate (inline middleware)
- [x] Auto-start review after proposal/plan generation
- [x] ChangeDetailView UI shows review status badges
- [ ] Constitution workflow 整合 ReviewGate (future)
- [ ] Context workflow 整合 ReviewGate (future)

---

## Track C: Living Context (Complete)

CESDD Layer 2 - Auto-curated project context.

### Phase C1: Context Files (Complete)

- [x] `ContextState` with files array
- [x] `InitializeContext` - create template files
- [x] `RefreshContext` - reload from disk
- [x] `ContextPanel` UI - display context files

### Phase C2: AI-Powered Context (Complete)

- [x] `GenerateContext` - AI analyzes codebase and generates context
- [x] `context_generate.rs` - codebase summarization module
- [x] Enhanced `SyncContext` - streaming output, multi-file updates
- [x] UI: Two initialization options (AI generation vs templates)
- [x] UI: Regenerate button (wand icon) in header
- [x] Streaming progress display during generation/sync

---

## Track D: Testing

### E2E Tests (In Progress)

- [x] Constitution workflow tests (7/8 passing, 1 skipped for Claude CLI)
- [x] Agent Rules tests (10/10 passing)
- [x] Workflows page tests
- [ ] Add Claude CLI mock for testing generation without real Claude
- [ ] Docker workflow E2E tests
- [ ] MCP workflow E2E tests
- [ ] Living Context E2E tests

### Known Limitations

- napi-rs `stateInit()` requires Electron context (cannot test state in standalone Node.js)
- Tests requiring Claude CLI are skipped in CI

---

## Technical Notes

### File Operations Architecture

```
React Frontend → Preload Bridge → napi-rs → Rust Backend → File System
```

### ReviewGate Flow

```
Workflow Node → CC CLI (plan mode) → rstn-mcp submit_for_review
                                          ↓
                                    ReviewSession
                                          ↓
                              ┌───────────┼───────────┐
                              ↓           ↓           ↓
                         Approve    SubmitFeedback  Reject
                              ↓           ↓           ↓
                         NextNode   CC iterate    End/Back
                                          ↓
                                    ReviewSession (iteration++)
```

### Security Model (File Operations)

1. Path canonicalization (resolve symlinks and `..`)
2. Scope validation (path must start with allowed root)
3. Allowed roots: `projectRoot` parameter, `~/.rstn/`
