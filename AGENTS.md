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
## Workflow-Driven UI (Tauri GUI)

The GUI is a **Tauri v2** desktop application with a **3-Tab Structure**.

### Navigation (Fixed Sidebar)

1. **Workflows Tab** (Home): Prompt-to-Code, Git operations
2. **Dockers Tab**: Container management dashboard
3. **Settings Tab**: Configuration

### Backend-Driven UI Model

- **Source of Truth**: Rust `AppState` (Backend)
- **Sync**: Backend pushes state updates to Frontend via Tauri Events
- **Action**: Frontend invokes Tauri Commands to mutate Backend state
- **No Fat Frontend**: Business logic lives in Rust, not TypeScript

### Reference

See `kb/architecture/01-system-specification.md` for full tech stack.
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

<grounding>

<repository-structure>
rustation/
├── Cargo.toml              # Workspace root (if any shared Rust libs)
├── CLAUDE.md               # This file
├── docs/                   # User Documentation
│   ├── get-started/
│   └── manual/
├── kb/                     # Engineering Handbook
│   ├── architecture/
│   ├── workflow/
│   └── internals/
├── src-tauri/              # Rust Backend (Tauri)
│   ├── src/
│   │   ├── commands/       # Tauri Commands
│   │   ├── state/          # AppState
│   │   └── main.rs
│   └── tauri.conf.json
├── src/                    # React Frontend
│   ├── components/         # shadcn/ui components
│   ├── features/           # Feature modules
│   ├── hooks/              # Custom React hooks
│   └── main.tsx
├── specs/{NNN}-{name}/
└── package.json
</repository-structure>

<knowledge-base>
**rustation v3 Documentation** (Tauri GUI):

**Engineering Handbook (`kb/`)**:
- `kb/README.md` - Start here for development
- `kb/architecture/00-overview.md` - Three pillars
- `kb/architecture/01-system-specification.md` - **Tech Stack & Layout**
- `kb/architecture/02-state-first-principle.md` - **🎯 CORE PRINCIPLE**
- `kb/workflow/sdd-workflow.md` - SDD Guide
- `kb/workflow/contribution-guide.md` - Tauri dev setup

**User Documentation (`docs/`)**:
- `docs/README.md` - Start here for usage
- `docs/get-started/quick-start.md` - Quick Start

**CRITICAL REQUIREMENTS for ALL features**:
1. **State tests MANDATORY**: Round-trip serialization + transitions + invariants
2. All state structs derive `Serialize + Deserialize + Debug + Clone`
3. NO hidden state
4. NO business logic in React (Logic belongs in Rust)
5. See `kb/architecture/02-state-first-principle.md`

**Development Workflow**:
- Run dev: `npm run tauri dev`
- Rust tests: `cargo test`
- React tests: `npm test`
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
