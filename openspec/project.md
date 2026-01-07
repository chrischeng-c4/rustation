# Project Context

## Purpose
rustation v3 is a desktop development workflow manager built with Electron. It provides a unified interface for managing multi-project development environments with integrated AI assistance, Docker orchestration, terminal emulation, and intelligent context management through Material Design 3 interface.

**Core Innovation**: CESDD Workflow (Constitution + Execution + Specification + Dev Delivery) integrating human review, automated suggestions, and change management.

## Tech Stack

### Desktop & Frontend
- **Shell**: Electron (window management, native integration)
- **Framework**: React 19 + Vite
- **UI Library**: MUI (Material UI v7) - Material Design 3
- **Styling**: Emotion (CSS-in-JS)
- **Terminal**: xterm.js (terminal emulator)

### Backend & Core
- **Language**: Rust
- **Binding**: napi-rs (Node.js native addon)
- **Async Runtime**: Tokio
- **HTTP Server**: Axum (for MCP server)
- **Docker**: Bollard (Docker API client)
- **PTY**: portable-pty (terminal emulation)

### State Management
- **Pattern**: Redux-like reducer pattern
- **Location**: Rust AppState (single source of truth)
- **Serialization**: JSON/YAML (all state must be serializable)
- **Communication**: IPC (state:dispatch, state:get, state:update)

### Testing
- **Rust**: cargo test
- **React**: Vitest + React Testing Library
- **E2E**: Playwright
- **Visual Regression**: Playwright screenshots

## Project Conventions

### Code Style

**Rust**:
- Follow rustfmt standard style
- Run `cargo clippy` before commits (MANDATORY)
- Naming: snake_case for functions/variables, PascalCase for types
- Error handling: Use `anyhow::Result` for errors
- Logging: Use `tracing` crate

**TypeScript/React**:
- ESLint + Prettier
- Material Design 3 patterns (MUI components)
- Naming: camelCase for functions/variables, PascalCase for components/types
- Hooks: Use custom hooks (useAppState, useActiveWorktree, etc.)

**File Naming**:
- Rust: snake_case.rs
- TypeScript: PascalCase for components (TasksPage.tsx), kebab-case for utils
- Feature modules: kebab-case directories (features/tasks/, features/dockers/)

### Architecture Patterns

**Three Core Pillars**:

#### 1. State-First Architecture
- **Rust owns ALL state** - React is pure display layer
- All state structs MUST derive `Serialize + Deserialize + Debug + Clone`
- **UI = render(State)** - React components are pure functions of state
- State must be JSON/YAML serializable at all times
- NO closures, thread-locals, or file handles in state
- Test state via serialization round-trips and transitions

#### 2. Frontend/Backend Separation
- **IPC Boundary**: React ↔ Rust via IPC channels
  - `state:dispatch` - Send actions from React
  - `state:get` - Request current state
  - `state:update` - Push state changes from Rust
- **NO business logic in React** - only display and user interaction
- **Backend-driven UI** - state machine in Rust drives UI flow

#### 3. Workflow-Driven UI
- **Fixed Sidebar Navigation** with feature tabs
- **Feature-based organization**: `src/renderer/src/features/`
- Each worktree maintains isolated state (Tasks, Docker, Terminal)

### Critical Architecture Rules

**State Management**:
- Redux-like reducer pattern: `Action → reduce(state, action) → NewState`
- Modular reducers in `packages/core/src/reducer/`
- Actions defined in `packages/core/src/actions.rs` (200+ actions)

**Layer Connectivity**:
```
React Component
    ↓ window.api.dispatch()
Preload Bridge (IPC)
    ↓ @rstn/core bindings
napi-rs Functions
    ↓ Rust FFI
Rust Backend
    ↓ reduce(state, action)
New State → JSON
    ↓ IPC emit
React re-renders
```

**File Size Limits**:
- **500 lines**: Consider splitting the file
- **1000 lines**: MUST split (no exceptions)
- Use submodules with `mod.rs` pattern
- Example: `reducer/` is a module directory, not a single file

### Testing Strategy

**Unit Tests (Rust)**:
- Business logic tests in `packages/core/src/`
- State transition tests (MANDATORY)
- Serialization round-trip tests for all state structs
- Mock external dependencies (Docker, Git)

**Component Tests (React/Vitest)**:
- Display logic only (NO business logic tests)
- Test component rendering based on props
- User interaction tests (clicks, inputs)

**Integration Tests**:
- Test napi-rs binding connectivity (JS can call Rust)
- Test IPC communication
- Test state persistence (save/load)

**E2E Tests (Playwright)**:
- Full-stack integration
- MUST test REAL backend behavior (NO MOCK data)
- Test actual Docker operations, file system, etc.
- Skip gracefully if backend unavailable

**Anti-Pattern**:
- ❌ Using MOCK_* data in E2E tests → proves nothing
- ❌ Testing UI coordinates instead of state transitions
- ❌ Manual testing without automated verification

### Git Workflow

**Branches**:
- Main branch: `main`
- Feature branches: `feature/<description>` or numbered (`001-feature-name`)
- No direct commits to main

**Commit Format**:
```
feat(scope): description
fix(scope): description
refactor(scope): description
test(scope): description
```

**Pre-commit Checklist**:
- [ ] `cargo test` passes
- [ ] `cargo clippy` clean
- [ ] `pnpm test` passes (from root)
- [ ] All layers connected (see Definition of Done)

**Definition of Done (MANDATORY)**:
1. ✅ Backend (Rust) implemented
2. ✅ Binding (napi-rs) exported with `#[napi]`
3. ✅ Bridge (Preload) calls @rstn/core (NOT placeholder)
4. ✅ Frontend (React) uses window.api.* (NO MOCK data)
5. ✅ E2E tests verify REAL backend behavior

See: `kb/workflow/definition-of-done.md` for complete checklist

## Domain Context

### Development Workflow Management

**Core Concepts**:
- **Project**: A git repository with optional worktrees
- **Worktree**: Separate working directory for same repo (git worktree)
- **Service**: Docker container (nginx, postgres, redis, etc.)
- **Task**: Justfile command/target
- **Change**: Managed development proposal (Draft → Implementing → Completed)
- **Constitution**: Modular coding rules and conventions

**Key Features**:

#### 1. MCP Integration
- **Embedded MCP Server**: HTTP SSE transport per worktree
- **Auto-port allocation**: 3000, 3001, 3002, etc.
- **MCP Tools**: read_file, list_directory, get_project_context, run_just_task, submit_for_review, check_review_feedback
- **Compatible with**: Claude Desktop, Claude Code CLI

#### 2. Context Engine
- **Intelligent context aggregation** from multiple sources:
  - Open files (with cursor line)
  - Git status and diffs
  - Terminal output
  - Docker logs and errors
  - Directory tree structure
- **Token budget optimization**: Priority queue ensures key info fits
- **System prompt generation**: Markdown-formatted context for AI

#### 3. Constitution System
- **Modular coding rules**: `.rstn/constitutions/` directory
- **YAML front matter**: Priority, token estimates, apply paths
- **Templates**: Global rules, Rust-specific, TypeScript-specific
- **AI injection**: Dynamically included in system prompts

#### 4. Agent Rules
- **Temporary system prompt files**: `/tmp/rstn-agent-rules-{project_id}.txt`
- **Claude Code integration**: `--system-prompt-file` flag
- **Idempotent generation**: Auto-cleanup and regeneration

#### 5. Review Gate
- **Human approval workflow**: Plan, Proposal, Code, Artifact reviews
- **Strategies**: AlwaysReview, OnlyCode, NeverReview
- **Session tracking**: Feedback, iterations, comments
- **MCP tools**: submit_for_review(), check_review_feedback()

#### 6. Change Management (CESDD)
- **State flow**: Draft → Planning → Proposed → PlanApproved → Implementing → Completed
- **Content tracking**: Proposals, plans, code changes
- **Streaming support**: Append from Claude responses
- **Review integration**: Link review sessions to changes

## Important Constraints

### Technical Constraints
- **File Size Limit**: 500 lines (consider), 1000 lines (MUST split)
- **No Non-Serializable State**: No closures, thread-locals, file handles in AppState
- **IPC Boundary**: All React↔Rust via IPC, no direct function calls
- **Material Design 3**: MUST use MUI components, follow MD3 patterns
- **JSON/YAML Serializable**: All state structs must derive Serialize + Deserialize

### Development Constraints
- **KB-First Principle**: Document architecture in `kb/` before implementation
- **Automated Verification**: Every feature MUST be programmatically testable (no manual testing)
- **YAGNI**: Start minimal, avoid premature abstraction
- **Simplicity**: Default to <100 LOC, single-file implementations until proven insufficient

### Workflow Constraints
- **OpenSpec (Optional)**: Use for features >500 LOC, >5 files, or architectural changes
- **Definition of Done**: All 5 layers verified (Backend → Binding → Bridge → Frontend → E2E)
- **No Fake Complete**: Feature incomplete if MOCK data used or layers disconnected
- **Test-First**: Write tests before/during implementation, not after

## External Dependencies

### Required at Runtime
- **Docker**: Container management requires Docker daemon running
- **Git**: Project management requires git CLI
- **just**: Task runner (Justfile execution)

### Planned/Optional
- **Claude Code CLI**: For MCP integration and AI-assisted workflows
- **Claude Desktop**: Alternative MCP client

### Critical Build Dependencies
- **napi-rs**: Rust↔Node.js bindings (packages/core)
- **Electron**: Desktop app framework
- **Node.js**: v18+ (for napi-rs compatibility)
- **Rust**: 1.75+ (2021 edition)

## Directory Structure Reference

```
rustation/
├── Cargo.toml                      # Rust workspace
├── package.json                    # Root pnpm workspace
├── pnpm-workspace.yaml             # Workspace config: desktop, packages/*, e2e
│
├── packages/
│   └── core/                       # Rust napi-rs module
│       ├── Cargo.toml              # rstn-core package
│       ├── package.json            # @rstn/core npm package
│       ├── build.rs                # napi-build
│       └── src/
│           ├── lib.rs              # #[napi] exports
│           ├── app_state.rs        # Complete state tree (AppState, ProjectState, WorktreeState)
│           ├── actions.rs          # Action enum (200+ variants)
│           ├── mcp_server.rs       # MCP HTTP SSE server
│           ├── mcp_config.rs       # MCP config management
│           ├── context_engine.rs   # AI context aggregation
│           ├── context.rs          # Context state
│           ├── constitution.rs     # Coding rules system
│           ├── agent_rules.rs      # System prompt generation
│           ├── docker.rs           # Docker API client (Bollard)
│           ├── justfile.rs         # Justfile parser
│           ├── terminal.rs         # PTY support (portable-pty)
│           ├── worktree.rs         # Git worktree management
│           ├── file_reader.rs      # Safe file reading
│           ├── persistence.rs      # State save/load (JSON/YAML)
│           ├── db.rs               # SQLite database
│           ├── explorer/           # File browser
│           │   └── mod.rs
│           └── reducer/            # Modular state transitions
│               ├── mod.rs          # Coordinator
│               ├── chat.rs
│               ├── docker.rs
│               ├── mcp.rs
│               ├── tasks.rs
│               ├── worktree.rs
│               ├── terminal.rs
│               ├── explorer.rs
│               ├── changes.rs      # Change management
│               ├── review_gate.rs  # Review workflow
│               ├── constitution.rs
│               ├── context.rs
│               ├── settings.rs
│               ├── notifications.rs
│               ├── dev_log.rs
│               ├── file_viewer.rs
│               ├── a2ui.rs
│               ├── env.rs
│               └── conversions.rs
│
├── desktop/                        # Electron app (root level, NOT apps/desktop)
│   ├── package.json                # rstn-desktop
│   ├── electron.vite.config.ts     # Electron + Vite config
│   ├── playwright.config.ts        # Desktop E2E tests
│   └── src/
│       ├── main/                   # Electron main process
│       │   └── index.ts            # IPC handler, window management
│       ├── preload/                # IPC bridge layer
│       │   ├── index.ts            # Exposes @rstn/core to window.api
│       │   └── index.d.ts          # TypeScript type definitions
│       └── renderer/               # React frontend
│           └── src/
│               ├── App.tsx         # Root component (Sidebar + routing)
│               ├── main.tsx        # Entry point
│               ├── index.css       # Global styles
│               ├── features/       # Feature modules (pages)
│               │   ├── tasks/
│               │   │   ├── TasksPage.tsx
│               │   │   └── TaskCard.tsx
│               │   ├── dockers/
│               │   │   ├── DockersPage.tsx
│               │   │   ├── DockerServiceCard.tsx
│               │   │   ├── AddDbDialog.tsx
│               │   │   └── AddVhostDialog.tsx
│               │   ├── chat/
│               │   │   ├── ChatPage.tsx
│               │   │   └── ChatMessage.tsx
│               │   ├── terminal/
│               │   │   ├── TerminalPage.tsx
│               │   │   └── XTermComponent.tsx
│               │   ├── workflows/
│               │   │   ├── WorkflowsPage.tsx
│               │   │   ├── ConstitutionPanel.tsx
│               │   │   ├── ChangeManagementPanel.tsx
│               │   │   ├── ReviewPanel.tsx
│               │   │   └── ContextPanel.tsx
│               │   ├── explorer/
│               │   │   ├── ExplorerPage.tsx
│               │   │   ├── FileTable.tsx
│               │   │   └── DetailPanel.tsx
│               │   ├── mcp/
│               │   │   └── McpPage.tsx
│               │   ├── a2ui/
│               │   │   ├── A2UIPage.tsx
│               │   │   ├── A2UIRenderer.tsx
│               │   │   └── registry.tsx
│               │   ├── agent-rules/
│               │   │   └── AgentRulesPage.tsx
│               │   ├── env/
│               │   │   └── EnvPage.tsx
│               │   ├── settings/
│               │   │   └── SettingsPage.tsx
│               │   ├── projects/
│               │   │   ├── ProjectTabs.tsx
│               │   │   └── AddWorktreeDialog.tsx
│               │   └── command-palette/
│               │       └── CommandPalette.tsx
│               ├── components/       # Shared components
│               │   ├── layout/
│               │   │   ├── Sidebar.tsx
│               │   │   └── RightIconBar.tsx
│               │   └── shared/
│               │       ├── LogPanel.tsx
│               │       ├── SourceCodeViewer.tsx
│               │       ├── ErrorBoundary.tsx
│               │       └── MarkdownDisplay.tsx
│               ├── hooks/            # React hooks
│               │   └── useAppState.ts
│               ├── theme/            # MUI MD3 theme
│               │   └── index.ts
│               └── types/            # TypeScript types
│                   └── state.ts      # Mirror Rust state types
│
├── e2e/                            # Root-level E2E tests (Playwright)
│   ├── playwright.config.ts
│   ├── test-helpers.ts
│   ├── docker.spec.ts
│   ├── change-management.spec.ts
│   ├── md3-visual-regression.spec.ts
│   └── electron.fixture.ts
│
├── kb/                             # Engineering Handbook (source of truth)
│   ├── README.md
│   ├── architecture/
│   │   ├── 00-overview.md          # Three pillars
│   │   ├── 01-state-first.md       # State-first principle
│   │   ├── 02-state-topology.md    # AppState tree
│   │   ├── 03-persistence.md
│   │   └── 07-testing.md
│   ├── features/
│   │   ├── docker-management.md
│   │   ├── project-management.md
│   │   ├── tasks-justfile.md
│   │   └── file-explorer.md
│   ├── roadmap/
│   │   ├── 01-mcp-integration.md
│   │   ├── 14-feature-context-engine.md
│   │   └── 15-feature-terminal.md
│   └── workflow/
│       ├── contribution-guide.md
│       ├── definition-of-done.md   # 🚨 MANDATORY checklist
│       ├── testing-guide.md
│       └── sdd-workflow.md
│
├── openspec/                       # OpenSpec specs and changes
│   ├── AGENTS.md                   # This workflow doc
│   ├── project.md                  # This file
│   ├── specs/                      # Current truth (what IS built)
│   └── changes/                    # Proposals (what SHOULD change)
│
└── docs/                           # User documentation
    └── README.md
```

## Change Proposal Guidance

When creating OpenSpec changes for this project:

### 1. Check KB First
- Review relevant `kb/` docs before creating specs
- Understand existing architecture patterns
- Check for conflicting or overlapping changes in `kb/`

### 2. Capability Naming
- Use verb-noun pattern: `docker-management`, `worktree-creation`, `mcp-integration`
- Single purpose per capability
- Keep names short and descriptive

### 3. State-First Specs
If changing state:
- Include complete state struct definitions
- Document state transitions (Draft → Planning → Implementing)
- Specify serialization requirements (all fields JSON-serializable)
- Define Action variants needed

### 4. Testing Requirements
Every requirement MUST specify:
- **Unit tests**: Rust state transition tests
- **Integration tests**: napi-rs binding connectivity
- **E2E tests**: Playwright tests for full-stack behavior
- **Anti-pattern**: "Manually verify by running the app" → REJECTED

### 5. Layer Verification
Include checklist for all 5 layers:
- [ ] Backend (Rust) implemented - functions work
- [ ] Binding (napi-rs) exported - `#[napi]` decorators added
- [ ] Bridge (Preload) connected - calls @rstn/core (NOT placeholder)
- [ ] Frontend (React) integrated - uses window.api.* (NO MOCK data)
- [ ] E2E tested - Playwright tests verify REAL backend behavior

### 6. MCP Tool Considerations
If adding MCP tools:
- Document JSON-RPC method signature
- Specify tool capabilities and parameters
- Include example request/response
- Test with MCP inspector page

### 7. Context Engine Impact
If feature affects AI context:
- Specify priority level (High/Normal/Low)
- Estimate token cost
- Document context format
- Test context generation

### 8. Breaking Changes
If change affects existing behavior:
- Mark with **BREAKING** in proposal
- Document migration path
- Update affected KB docs
- Plan deprecation timeline

## Key Success Metrics

### Code Quality
- All tests pass (Rust + React + E2E)
- `cargo clippy` clean (no warnings)
- No MOCK data in production renderer code
- All layers connected (Definition of Done checklist)

### Architecture Compliance
- State-first: All state serializable
- KB-first: Architecture documented before implementation
- Test-first: Automated verification for all features
- Layer isolation: No business logic in React

### Feature Completeness
- Backend implemented and tested
- napi-rs binding exported and integration-tested
- Preload bridge connects to real @rstn/core
- Frontend uses window.api.* (no MOCK)
- E2E tests verify real behavior

## Common Pitfalls to Avoid

1. ❌ **Fake Complete**: UI works with MOCK data but backend missing
2. ❌ **Placeholder Bridge**: Preload doesn't connect to real @rstn/core
3. ❌ **Manual Testing**: "Run the app and check" instead of automated tests
4. ❌ **Monolithic Files**: Single file >1000 lines (MUST split)
5. ❌ **Business Logic in React**: State calculations in components
6. ❌ **Non-Serializable State**: Closures or file handles in AppState
7. ❌ **apps/desktop Path**: Wrong - correct path is `desktop/` at root level
8. ❌ **Single reducer.rs**: Wrong - use modular `reducer/` directory

## Quick Reference

### Build Commands
```bash
# Build Rust core
cd packages/core && pnpm build

# Build desktop
cd desktop && pnpm build

# Development mode
cd desktop && pnpm dev

# Run tests
cargo test                 # Rust tests
pnpm test                  # React tests (from root)
pnpm test:e2e             # E2E tests (from root)
```

### Key Files to Check Before Changes
- `packages/core/src/app_state.rs` - State structure
- `packages/core/src/actions.rs` - Action enum
- `packages/core/src/reducer/mod.rs` - Reducer coordinator
- `desktop/src/preload/index.ts` - IPC bridge
- `kb/workflow/definition-of-done.md` - Feature checklist

### Documentation System

rustation uses three complementary documentation systems:

| System | Purpose | Audience | Content Type |
|--------|---------|----------|--------------|
| **openspec/** | Feature specifications | AI, PM, QA, Developers | Requirements, Scenarios, Acceptance Criteria |
| **kb/** | Engineering handbook | Contributors, Maintainers | Architecture decisions, Development guides |
| **docs/** | User manual | End Users | Installation, Tutorials, How-to guides |

**Relationship**:
```
openspec/specs/          → What features do (Requirements)
    ↓
kb/architecture/         → Why we built it this way (ADR)
    ↓
docs/features/           → How users operate features (Tutorials)
```

**When to Update**:
- **Adding feature**: Create `openspec/specs/<capability>/spec.md` → Update `docs/features/<feature>.md`
- **Changing architecture**: Update `kb/architecture/*.md` → Reference in `openspec/project.md`
- **User-facing change**: Update `docs/` regardless of spec/KB changes

### Documentation References
- **Engineering**: `kb/README.md`
- **User Guide**: `docs/` (VitePress site)
- **OpenSpec**: `openspec/AGENTS.md`
- **Architecture**: `kb/architecture/00-overview.md`
