# AGENTS.md

<language>
Respond in English (U.S.) by default. Use Traditional Chinese only when user writes in Traditional Chinese.
</language>

---

<kb-first-principle>
## KB-First = Spec-First

**Core Development Philosophy**: The entire project architecture and logic can be derived from the Knowledge Base. KB-First equals Spec-First.

### Principle

- **Knowledge Base (`kb/`) as Source of Truth**: Contains authoritative engineering documentation:
  - Architecture decisions and patterns
  - Workflows and processes
  - Internal API references
  - Design specifications

- **User Documentation (`docs/`)**: Contains user-facing guides:
  - Installation and Quick Start
  - Manuals and Command References

- **Code Implements KB**: Implementation follows what is specified in the Knowledge Base.

### Simplicity & Minimalism

- **YAGNI (You Aren't Gonna Need It)**: Start with minimal viable solution
- **Delete Aggressively**: Remove unused code and UI elements
- **Minimal Complexity**: Only add features that are immediately needed

### Code File Size Limits

**Critical Rule**: Prevent monolithic code files

- **500 lines**: Consider splitting the file into smaller modules
- **1000 lines**: MUST split the file - no exceptions
- **Benefits**:
  - Easier code review and navigation
  - Better module boundaries and separation of concerns
  - Reduced cognitive load
  - Prevents god classes/modules

**When to split**:
- Extract related functions into a submodule
- Move tests to separate `tests.rs` or `mod_test.rs` files
- Create feature-specific modules (e.g., `state/worktree.rs`, `state/dashboard.rs`)
- Use `mod.rs` as a thin coordination layer that re-exports from submodules

**Example**:
```
Before (1200 lines):
  src/state.rs

After:
  src/state/
    ├── mod.rs          (50 lines - re-exports only)
    ├── app.rs          (200 lines)
    ├── worktree.rs     (300 lines)
    ├── dashboard.rs    (150 lines)
    ├── settings.rs     (150 lines)
    └── tests.rs        (350 lines)
```

### Examples

**Tauri Command Design** (KB-First approach):
1. Define command interface in `kb/architecture/01-system-specification.md`
2. Implement in `src-tauri/src/commands/`
3. Frontend invokes via `invoke('command_name', params)`

**State Machine Workflows** (KB-First approach):
1. Document workflow architecture in `kb/architecture/` (e.g., 09-workflow-prompt-claude.md)
2. Define state transitions and validation rules in KB
3. Implement state machine in Rust based on KB specification

### Benefits

1. **Single Source of Truth**: No confusion about intended behavior or architecture
2. **Onboarding Efficiency**: New contributors can understand the system from KB alone
3. **Consistency**: All implementations follow documented patterns

### Workflow Integration

**Policy**: KB-First is the default workflow.
- Write design/architecture/workflow docs in `kb/`.
- Write user guides in `docs/`.

**Before implementing ANY feature**:
1. Check `kb/` for existing patterns.
2. Update `kb/` if architectural changes are needed.
3. Update `docs/` if user-facing behavior changes.

See: `kb/README.md` for Engineering Handbook.
See: `docs/README.md` for User Documentation.
</kb-first-principle>

---

<automated-verification>
## Automated Verification Principle

**Critical Rule**: Everything MUST be checkable/testable without human intervention. If a feature cannot be verified programmatically, the design is fundamentally flawed and MUST be rejected.

### Core Principles

1. **No Manual Testing Required**
   - Features MUST be testable through automated tests
   - Debug workflows MUST be programmatically verifiable
   - NEVER ask humans to "run the app and check" - you MUST write a test instead
   - Proactively write tests BEFORE implementing features (TDD)

2. **Self-Debugging Systems**
   - Systems MUST provide introspection capabilities
   - Logs MUST be machine-readable and parseable
   - State MUST be queryable programmatically
   - Health checks MUST be automatable
   - Proactively add instrumentation when implementing features

3. **Proactive Test-First Development**
   - MUST write integration tests that verify end-to-end functionality
   - MUST use property-based testing for complex logic
   - MUST mock external dependencies to enable isolated testing
   - Tests are the primary documentation of expected behavior

### Anti-Patterns to Avoid

❌ **BAD**: "Add debug logs and ask user to check console"
✓ **GOOD**: Write a test that captures the logs and asserts on them

❌ **BAD**: "Start the app manually to see if feature works"
✓ **GOOD**: Write an E2E test that starts the app programmatically and verifies behavior

❌ **BAD**: "Check if the API returns the right data"
✓ **GOOD**: Write an integration test that calls the API and validates the response structure

### Implementation Guidelines

1. **For New Features**:
   ```
   1. Write test that exercises the feature
   2. Implement the feature
   3. Test passes → feature is verified
   4. Test fails → fix implementation
   ```

2. **For Debugging**:
   ```
   1. Reproduce issue in automated test
   2. Add instrumentation (structured logs, metrics)
   3. Test queries instrumentation to verify behavior
   4. Fix root cause
   5. Test validates fix
   ```

3. **For Integration Points**:
   - HTTP APIs: Use curl/httpie in test scripts
   - Databases: Use SQL queries in test assertions
   - File systems: Use find/grep in test validation
   - Processes: Use ps/lsof in health checks

### Benefits

- **Reliability**: Tests catch regressions before humans see them
- **Speed**: Automated tests run in seconds, manual testing takes minutes
- **Documentation**: Tests document expected behavior better than comments
- **Confidence**: Every change is verified before deployment

### Examples

**Good Example - HTTP API Test**:
```bash
# Test MCP server tools endpoint
response=$(curl -s -X POST http://localhost:5000/mcp \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}')

tools_count=$(echo "$response" | jq '.result.tools | length')
[[ $tools_count -eq 4 ]] || exit 1  # Assert 4 tools exist
```

**Good Example - Integration Test**:
```rust
#[tokio::test]
async fn test_fetch_mcp_tools_returns_valid_response() {
    // Start MCP server
    let server = start_mcp_server().await;

    // Call fetch_mcp_tools
    let result = fetch_mcp_tools().await.unwrap();
    let data: Value = serde_json::from_str(&result).unwrap();

    // Validate response structure
    assert!(data["result"]["tools"].is_array());
    assert_eq!(data["result"]["tools"].as_array().unwrap().len(), 4);
}
```

</automated-verification>

---

<state-first-architecture>
## State-First Architecture

**State is King**: At any time, rstn's entire state MUST be JSON/YAML serializable.

### Core Principles

- **UI = render(State)**: UI is a pure function of state
- **Testing**: Test state transitions, not UI coordinates
- **No Hidden State**: All state must be serializable (no closures, thread-locals, non-serializable types)
- **State Structs**: All state structs MUST derive `Serialize + Deserialize + Debug + Clone`

### Critical Requirements

1. **State tests MANDATORY**: Round-trip serialization + transitions + invariants
2. See `kb/workflow/testing-guide.md` for examples
3. See `kb/architecture/01-state-first-principle.md` for principles

### References

- `kb/architecture/02-state-first-principle.md` - **🎯 CORE PRINCIPLE**: All state MUST be JSON/YAML serializable
- `kb/architecture/00-overview.md` - Three pillars (state-first, frontend/backend separation, backend-driven UI)
- `kb/workflow/testing-guide.md` - How to write state tests
</state-first-architecture>

---

<workflow-driven-ui>
## Workflow-Driven UI (Electron + napi-rs)

The GUI is an **Electron** desktop application with **React** frontend and **Rust** backend via **napi-rs**.

### Navigation (Fixed Sidebar)

1. **Tasks Tab**: Justfile command runner
2. **Dockers Tab**: Container management dashboard
3. **Settings Tab**: Configuration

### Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│ React Frontend (renderer)                               │
│   └─ Uses window.api.* (NO MOCK data!)                  │
├─────────────────────────────────────────────────────────┤
│ Preload Bridge (preload/index.ts)                       │
│   └─ Exposes @rstn/core to window.api                   │
│   └─ MUST connect to real napi-rs, NOT placeholder      │
├─────────────────────────────────────────────────────────┤
│ napi-rs Bindings (packages/core)                        │
│   └─ #[napi] decorated functions                        │
├─────────────────────────────────────────────────────────┤
│ Rust Backend (packages/core/src/)                       │
│   └─ docker.rs, justfile.rs                             │
└─────────────────────────────────────────────────────────┘
```

### Critical Rule

**Frontend → Preload → napi-rs → Rust**

Every layer MUST be connected. If ANY layer is missing or placeholder, feature is NOT complete.

### Reference

See `kb/workflow/definition-of-done.md` for feature completion checklist.
</workflow-driven-ui>

---

<chain-of-thought>
Before starting ANY non-trivial work, work through these steps IN ORDER:

<step number="1" name="WHAT">
  - Topic/Change: ___
  - User-facing outcome: ___
</step>

<step number="2" name="KB CHECK">
  - Which `kb/` doc(s) are the source of truth for this change? ___
  - If missing: which new KB doc will be added/updated first? ___
</step>

<step number="3" name="NEXT ACTION">
  - Update KB first (spec-first, project-wide)
  - Then implement code changes (if requested)
  - Then add/adjust tests (state-first)
</step>

<step number="4" name="TESTS NEEDED">
  - Unit tests (Rust): ___
  - Integration tests (Rust): ___
  - Component tests (React/Vitest): ___
</step>

<step number="5" name="COMPLETE?">
  - All tests pass? YES/NO
  - cargo clippy clean? YES/NO
</step>

Write out these 5 steps when the change spans multiple files or introduces new behavior.
</chain-of-thought>

---

<decision-trees>

NOTE: SDD (speckit + `specs/`) is optional in this repo. Prefer KB-first updates in `kb/` unless the user explicitly requests speckit artifacts.

<tree name="Which SDD Workflow">
START: New work?
│
├─► Estimated LOC > 500?
│   └─ YES → Full SDD (spec + plan + tasks)
│
├─► Touches > 5 files?
│   └─ YES → Full SDD
│
├─► Architecture change?
│   └─ YES → Full SDD
│
├─► rush feature (Phase 7-8)?
│   └─ YES → Full SDD
│
├─► Complex algorithm or new domain concept?
│   └─ YES → Full SDD
│
└─► Otherwise → Lightweight SDD (spec only)
    │
    ├─ Full SDD path:
    │  /speckit.specify → /speckit.plan → /speckit.tasks → implement
    │
    └─ Lightweight SDD path:
       /speckit-lite → implement directly (no plan/tasks)

See: kb/workflow/sdd-workflow.md for detailed guide
</tree>

<tree name="When to use Design-First Planning">
START: Planning rstn GUI feature?
│
├─► Does feature involve interactive flow?
│   ├─ YES → Continue checking
│   └─ NO → Use standard SDD workflow
│
├─► Does it involve ANY of these?
│   ├─ rstn ↔ Claude Code communication → Design-First Planning REQUIRED
│   ├─ rstn ↔ MCP server interaction → Design-First Planning REQUIRED
│   ├─ Multi-step user workflows → Design-First Planning REQUIRED
│   ├─ State machine (>3 states) → Design-First Planning REQUIRED
│   ├─ Async operations / streaming → Design-First Planning REQUIRED
│   └─ Simple UI-only change → Standard SDD
│
└─► Use Design-First Planning:
    Plan phase MUST include:
    1. Flow chart (Mermaid)
    2. Sequence chart (Mermaid)
    3. State machine (Mermaid)
    4. Logging specification
    5. Verification method
</tree>

<tree name="Feature Completion Verification">
START: Is feature "done"?
│
├─► Backend (Rust) implemented?
│   └─ NO → Implement backend first, run cargo test
│
├─► napi-rs binding exported?
│   └─ NO → Add #[napi] decorator, run pnpm build in packages/core
│
├─► Integration test passes? (JS can call Rust)
│   └─ NO → Fix binding, DO NOT proceed to UI
│
├─► Preload bridge connected?
│   └─ NO → Add functions to window.api in preload/index.ts
│   └─ Check: Is it using @rstn/core or placeholder?
│            └─ Placeholder → NOT connected, fix it
│
├─► Frontend uses window.api.*?
│   └─ NO → Remove MOCK_* data, use real API
│   └─ Check: grep -rE "MOCK_" apps/desktop/src/renderer/
│            └─ Matches found → NOT done, remove MOCK
│
├─► E2E tests real backend?
│   └─ NO → Update E2E to test real functionality
│   └─ Check: Does E2E skip gracefully when backend unavailable?
│            └─ NO → Add availability check
│
└─► ALL checks pass?
    ├─ YES → Feature is DONE ✓
    └─ NO → Feature is NOT done, fix failing checks
</tree>

<tree name="Claude CLI Integration">
START: rstn needs to call Claude CLI?
│
├─► What mode?
│   ├─ Headless/programmatic → Use `-p` (print mode)
│   └─ Interactive → Use default (no -p)
│
├─► Need streaming output?
│   ├─ YES → `--output-format stream-json`
│   │        └─► MUST add `--verbose` flag (required with -p + stream-json)
│   └─ NO → `--output-format json` or `text`
│
├─► Need partial messages?
│   ├─ YES → `--include-partial-messages` (requires stream-json)
│   └─ NO → Skip flag
│
├─► Using MCP?
│   ├─ YES → `--mcp-config ~/.rstn/mcp-session.json`
│   │        Config format: `{"mcpServers":{"rstn":{"type":"http","url":"..."}}}`
│   └─ NO → Skip flag
│
├─► Custom system prompt?
│   ├─ Replace all → `--system-prompt-file /path/to/file`
│   └─ Append → `--append-system-prompt "extra instructions"`
│
└─► END: Build command with all required flags
    See: docs/manual/claude-code/cli-reference.md
</tree>

</decision-trees>

---

<agent-orchestration>
The main conversation thread acts as an **Orchestrator/PM/Planner**. It should:

1. **NEVER read code directly** - Delegate to `reader` agent
2. **NEVER write code directly** - Delegate to `writer` agent
3. **Plan and coordinate** - Break down tasks, sequence agent calls
4. **Review and approve** - Validate agent outputs before proceeding

## Agent Definitions

| Agent | Model | Location | Purpose |
|-------|-------|----------|---------|
| reader | haiku | .claude/agents/reader.md | All reading and summarization |
| writer | sonnet | .claude/agents/writer.md | All writing and updating |

## Delegation Rules

| Task | Delegate To | Model |
|------|-------------|-------|
| Read files | reader | haiku |
| Search codebase | reader | haiku |
| Summarize code | reader | haiku |
| Understand patterns | reader | haiku |
| Find file locations | reader | haiku |
| Write code | writer | sonnet |
| Edit files | writer | sonnet |
| Create files | writer | sonnet |
| Run commands | writer | sonnet |
| Fix bugs | writer | sonnet |

## Workflow Pattern

```
User Request
    ↓
Orchestrator (main thread)
    ├── Spawns reader agent(s) for understanding
    ├── Reviews reader findings
    ├── Creates implementation plan
    ├── Spawns writer agent(s) for execution
    └── Reviews writer results
```

## Example Orchestration

```
User: "Add email validation to the User document"

Orchestrator:
1. Spawn reader → "Find existing validation patterns in validation.rs"
2. Review findings → Understands pattern
3. Create plan → Task breakdown with file targets
4. Spawn writer → "Add email regex validation following the existing pattern"
5. Review result → Verify implementation matches plan
```

</agent-orchestration>

---

<grounding>

<repository-structure>
rustation/
├── Cargo.toml                      # Workspace root
├── CLAUDE.md                       # This file
├── docs/                           # User Documentation
├── kb/                             # Engineering Handbook
│   ├── architecture/
│   ├── workflow/
│   │   └── definition-of-done.md   # 🚨 MANDATORY checklist
│   └── internals/
├── packages/
│   └── core/                       # Rust → napi-rs bindings
│       ├── src/
│       │   ├── lib.rs              # #[napi] exports
│       │   ├── docker.rs           # Docker management
│       │   └── justfile.rs         # Justfile parser
│       └── package.json
├── apps/
│   └── desktop/                    # Electron app
│       ├── src/
│       │   ├── main/               # Electron main process
│       │   ├── preload/            # 🔗 BRIDGE LAYER (window.api)
│       │   │   ├── index.ts        # Must call @rstn/core, NOT placeholder
│       │   │   └── index.d.ts      # TypeScript types
│       │   └── renderer/           # React frontend
│       │       └── src/
│       │           ├── features/   # Feature modules
│       │           └── components/ # shadcn/ui
│       └── package.json
├── e2e/                            # Electron E2E tests
│   ├── docker.spec.ts
│   └── electron.fixture.ts
└── .github/
    └── workflows/
        └── check-mock.yml          # CI: blocks MOCK in renderer
</repository-structure>

<knowledge-base>
**rustation v3 Documentation** (Electron + napi-rs):

**Engineering Handbook (`kb/`)**:
- `kb/README.md` - Start here for development
- `kb/architecture/00-overview.md` - Three pillars
- `kb/architecture/01-system-specification.md` - **Tech Stack & Layout**
- `kb/architecture/02-state-first-principle.md` - **🎯 CORE PRINCIPLE**
- `kb/workflow/sdd-workflow.md` - SDD Guide
- `kb/workflow/definition-of-done.md` - **🚨 MANDATORY**: Feature completion checklist
- `kb/workflow/contribution-guide.md` - Dev setup

**User Documentation (`docs/`)**:
- `docs/README.md` - Start here for usage
- `docs/get-started/quick-start.md` - Quick Start

**CRITICAL REQUIREMENTS for ALL features**:
1. **Definition of Done MANDATORY**: All layers connected (see `kb/workflow/definition-of-done.md`)
2. **NO MOCK data** in renderer production code
3. **Preload must connect to @rstn/core**, NOT be placeholder
4. NO business logic in React (Logic belongs in Rust)
5. E2E tests must test REAL backend behavior

**Development Workflow**:
- Build core: `cd packages/core && pnpm build`
- Build desktop: `cd apps/desktop && pnpm build`
- Run dev: `cd apps/desktop && pnpm dev`
- Rust tests: `cargo test`
- E2E tests: `cd e2e && pnpm exec playwright test --config playwright.config.ts`
</knowledge-base>

</grounding>

---

<negative-constraints>

<rule severity="NEVER">Change architecture without updating KB → Loss of source of truth → Update `kb/` first</rule>
<rule severity="NEVER">Block work on missing speckit artifacts → speckit is optional → Use KB-first instead</rule>
<rule severity="NEVER">Implement interactive flow without design diagrams → Leads to complexity → Use Design-First Planning</rule>
<rule severity="NEVER">Skip flow diagrams for rstn GUI features → Can't debug interaction → Create Mermaid diagrams in plan phase</rule>
<rule severity="NEVER">Implement without logging spec → No observability → Define what to log BEFORE coding</rule>
<rule severity="NEVER">Put business logic in React → Fat frontend anti-pattern → Logic belongs in Rust Backend</rule>
<rule severity="NEVER">Mutate state directly from Frontend → Split brain state → Use Tauri Commands to mutate Backend</rule>
<rule severity="NEVER">Commit without running tests → Broken code enters repo → Run `cargo test` and `npm test` first</rule>
<rule severity="NEVER">Skip clippy → Lints accumulate → Run cargo clippy before commit</rule>
<rule severity="NEVER">Use -p + stream-json without --verbose → CLI error → Always add --verbose flag</rule>
<rule severity="NEVER">Use "transport" in MCP config → Invalid schema → Use "type" field instead</rule>
<rule severity="NEVER">Implement features without state tests → Untestable code → All features MUST have state serialization and transition tests</rule>
<rule severity="NEVER">Use concrete language code blocks (rust, python, shell) in `kb/` files → KB is for architecture, not implementation → Use `mermaid` or `pseudo-code` instead</rule>
<rule severity="NEVER">Create files >500 lines without considering split → Monolithic code, hard to maintain → Split at 500 lines, MUST split at 1000 lines</rule>
<rule severity="NEVER">Put all code in single file → Creates god modules → Use submodules (mod.rs pattern) for organization</rule>
<rule severity="NEVER">Use MOCK_* data in renderer production code → Fake complete anti-pattern → Use window.api.* from real backend</rule>
<rule severity="NEVER">Leave preload as placeholder → Bridge layer missing → Connect preload to @rstn/core before building UI</rule>
<rule severity="NEVER">Claim feature complete without verifying all layers → Fake complete → Run DoD checklist in kb/workflow/definition-of-done.md</rule>
<rule severity="NEVER">Write E2E tests that only test MOCK UI → Tests prove nothing → E2E must test real backend behavior</rule>
<rule severity="NEVER">Skip integration test after binding → Can't verify JS→Rust connection → Test binding works before building UI</rule>

</negative-constraints>

---

<delimiters>
Use these markers in workflow updates:

<marker name="STATUS">
Topic: streaming chat UI
Phase: KB | IMPLEMENT | TEST
</marker>

<marker name="IMPLEMENTING">
Task: Add send_prompt Tauri Command
File: src-tauri/src/commands/workflow.rs
</marker>

<marker name="BUILD CHECK">
cargo build: PASS
cargo test: PASS
cargo clippy: PASS
npm run lint: PASS
npm test: PASS
</marker>

<marker name="READY FOR PR">
All tasks complete, tests pass
</marker>
</delimiters>

---

<output-structure>
After each work session, report in this format:

<report>
  <topic>{short-description}</topic>

  <kb-updates>
    <doc status="updated">kb/.../something.md</doc>
    <doc status="added">kb/.../new-doc.md</doc>
  </kb-updates>

  <implementation>
    <item status="done">Code change summary</item>
    <item status="next">Next code change</item>
  </implementation>

  <tests>
    <test name="unit_test_name" status="PASS"/>
    <test name="integration_test_name" status="PENDING"/>
  </tests>

  <build-status>
    <check name="cargo build" status="PASS"/>
    <check name="cargo test" status="PASS" note="7 new tests"/>
    <check name="cargo clippy" status="PASS"/>
  </build-status>

  <next-steps>
    <step>Update KB doc for X</step>
    <step>Implement Y</step>
  </next-steps>
</report>
</output-structure>

---

<self-correction>
Before committing or creating PR, verify ALL items:

<checklist name="KB Compliance">
  <item>KB updated for new/changed behavior?</item>
  <item>KB remains the single source of truth?</item>
</checklist>

<checklist name="Code Quality">
  <item>cargo build passes?</item>
  <item>cargo test passes?</item>
  <item>cargo clippy clean?</item>
  <item>No unwrap() in production code?</item>
</checklist>

<checklist name="Testing">
  <item>Rust unit tests written?</item>
  <item>React component tests written (if UI changed)?</item>
  <item>All tests pass (`cargo test` + `npm test`)?</item>
  <item>Edge cases covered?</item>
</checklist>

<checklist name="Commit">
  <item>Commit message format: feat(NNN): description?</item>
  <item>Changes are focused (not mixed features)?</item>
  <item>PR size reasonable (&lt;500 lines ideal)?</item>
</checklist>

If ANY item is NO, fix it before proceeding.
</self-correction>

---

<definition-of-done>
## Definition of Done (DoD) - MANDATORY

**A feature is NOT complete until ALL layers are connected and tested with REAL data.**

See: `kb/workflow/definition-of-done.md` for full checklist.

### Anti-Pattern: "Fake Complete"

```
❌ UI works but uses MOCK_* data
❌ E2E tests pass but test MOCK, not real backend
❌ Backend implemented but bridge layer missing
❌ Tests pass = Feature complete (WRONG!)
```

### Layer Verification Checklist

Before claiming ANY feature is "done", verify ALL layers:

| Layer | Verification |
|-------|--------------|
| 1. Backend (Rust) | `cargo test` passes, functions work |
| 2. Binding (napi-rs) | Exported with `#[napi]`, types generated |
| 3. Bridge (Preload) | Functions in `window.api.*`, NOT placeholder |
| 4. Frontend (React) | Uses `window.api.*`, NO `MOCK_*` constants |
| 5. E2E Tests | Tests REAL backend, skips gracefully if unavailable |

### Mandatory Verification Steps

**BEFORE saying "feature complete":**

1. **Check for MOCK data**:
   ```
   grep -rE "MOCK_SERVICES|MOCK_COMMANDS|MOCK_" apps/desktop/src/renderer/
   ```
   If ANY matches → Feature is NOT complete

2. **Verify preload bridge**:
   - Open `apps/desktop/src/preload/index.ts`
   - Confirm functions call `@rstn/core`, not placeholders

3. **Run E2E with real backend**:
   - E2E must test actual functionality
   - If E2E passes with MOCK data, it's testing nothing

### Development Order (MANDATORY)

```
1. Backend (Rust)     → cargo test
2. Binding (napi-rs)  → pnpm build (in packages/core)
3. Integration Test   → Verify JS can call Rust
4. Bridge (Preload)   → Add to window.api
5. Frontend (React)   → Use window.api.*, NO MOCK
6. E2E Test           → Test real behavior
```

**NEVER skip step 3-4. This is where "fake complete" happens.**

### CI Enforcement

CI automatically blocks MOCK data in production:
- `.github/workflows/check-mock.yml` - Fails PR if MOCK found in renderer

</definition-of-done>
