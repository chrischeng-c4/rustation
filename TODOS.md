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

### Phase A2: Extended Use Cases (In Progress)

Apply file reading to specific workflow scenarios.

- [x] Read `constitution.md` and display in Constitution Panel UI
- [ ] Read project source code for Claude analysis
- [ ] Read workflow outputs (proposal/plan files)
- [ ] General file browser UI component

### Phase A3: Additional File Operations (Future)

Expand file operation capabilities.

- [ ] File listing API (`window.api.file.list`)
- [ ] Directory tree API
- [ ] File metadata API (size, mtime, permissions)
- [ ] File write API (with security scope validation)

---

## Track B: ReviewGate

Human-in-the-loop 審核機制，讓 workflow 產出在落地前經過審核。

**KB 文件**: `kb/architecture/14-review-gate.md`

### Phase B1: Core Data Model (Complete)

建立 ReviewGate 的核心資料結構。

- [x] `ReviewPolicy` enum (AutoApprove, AgentDecides, AlwaysReview)
- [x] `ReviewContent` struct (content_type, content, file_changes)
- [x] `ReviewSession` struct (id, status, comments, iteration)
- [x] `ReviewComment` struct (target, content, resolved)
- [x] Add to `app_state.rs` - `review_gate: ReviewGateState`
- [x] Unit tests for serialization roundtrips
- [x] TypeScript types in `state.ts`

### Phase B2: Actions & Reducer (Complete)

實作 ReviewGate 的 Actions。

- [x] `StartReview` - 開始審核會話
- [x] `AddReviewComment` - 新增留言
- [x] `ResolveReviewComment` - 標記留言已解決
- [x] `SubmitReviewFeedback` - 批次送出 feedback 給 CC
- [x] `ApproveReview` - 批准
- [x] `RejectReview` - 拒絕
- [x] `UpdateReviewContent` - 更新審核內容
- [x] `SetReviewStatus` - 設定狀態
- [x] `SetReviewGateLoading/Error` - Loading/Error 狀態
- [x] `SetActiveReviewSession` - 設定 active session
- [x] `ClearReviewSession` - 清除 session
- [x] Reducer handlers for all actions (reducer.rs)
- [x] Async handlers stubs (lib.rs)
- [x] Unit tests (11 tests passing)
- [x] TypeScript action types

### Phase B3: MCP Integration (Complete)

rstn-mcp tools 供 CC 呼叫。

- [x] `submit_for_review` - CC 送審內容
  - Creates ReviewSession with content, file_changes, policy
  - Returns session_id for subsequent calls
- [x] `get_review_feedback` - CC 取得 feedback
  - Returns session status (pending/reviewing/iterating/approved/rejected)
  - Returns unresolved comments for iteration
- [x] `update_review_content` - CC 更新內容進入下一輪
  - Updates content after addressing feedback
  - Increments iteration count
  - Returns status back to 'reviewing'
- [x] Tool schema definitions in `mcp_server.rs`
- [x] Tool execution handlers using app state

### Phase B4: UI Components (Complete)

ReviewPanel UI 元件。

- [x] `ReviewPanel` - 主要審核介面
- [x] `ContentView` - Markdown 渲染 + Section 標記
- [x] `FileChangesView` - 檔案變更清單 (integrated in ContentView)
- [x] `CommentsSidebar` - 留言側邊欄
- [x] `ActionBar` - Approve / Request Changes / Reject 按鈕
- [x] WorkflowsPage integration

### Phase B5: Workflow Integration (Complete)

整合到現有 workflow 系統。

- [x] Add `proposal_review_session_id` and `plan_review_session_id` to Change struct
- [x] `StartProposalReview` action - creates review session after proposal generation
- [x] `StartPlanReview` action - creates review session after plan generation
- [x] Modify `CompleteProposal` handler to auto-start proposal review
- [x] Modify `CompletePlan` handler to auto-start plan review
- [x] Change Management workflow 整合 ReviewGate
- [x] Update ChangeDetailView UI to show review status badges
- [x] TypeScript types for new actions
- [ ] Constitution workflow 整合 ReviewGate (future)
- [ ] Context workflow 整合 ReviewGate (future)

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
