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

### Examples

**Keybinding Management** (KB-First approach):
1. Define keybindings in `docs/manual/cli/keybindings.md` (user contract)
2. Implement mapping logic in `crates/rstn/src/tui/keybindings.rs`
3. Code reads from specification.

**State Machine Workflows** (KB-First approach):
1. Document workflow architecture in `kb/architecture/02-state-first-mvi.md`
2. Define state transitions and validation rules in KB
3. Implement state machine based on KB specification

### Benefits

1. **Single Source of Truth**: No confusion about intended behavior or architecture
2. **Onboarding Efficiency**: New contributors can understand the system from KB alone
3. **Consistency**: All implementations follow documented patterns

### Workflow Integration

**Policy**: KB-First is the default workflow.
- Write design/architecture/workflow docs in `kb/`.
- Write user guides in `docs/`.
- `specs/` are optional and feature-scoped.

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

- `kb/architecture/01-state-first-principle.md` - **🎯 CORE PRINCIPLE**: All state MUST be JSON/YAML serializable
- `kb/architecture/00-overview.md` - Three pillars (state-first, CLI/TUI separation, testing-first)
- `kb/workflow/testing-guide.md` - How to write state tests
</state-first-architecture>

---

<workflow-driven-ui>
## Workflow-Driven UI (The "n8n" Model)

The TUI is shifting from a static document viewer to a **Workflow Launcher**.

### 1. Command as Workflow Trigger

- **Left Panel (Commands)**: List of available Workflows.
- **Action**: Selecting a command triggers a Workflow.
- **Constraint**: **Single Active Workflow**.

### 2. Dynamic Content Area

- **Middle Panel (Content)**: Visualizes the current state of the active Workflow Node.

### 3. Log Obsolescence

- **No Log Panel**: Detailed logs are persisted to `~/.rstn/logs/`.

### 4. No Tab Bar

- **Focus**: The interface should be focused on the current task (Worktree).
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
  - Unit tests: ___
  - Integration tests: ___
  - TUI e2e tests: ___ (dispatch to tui-tester)
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
START: Planning rstn TUI feature?
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

<tree name="Dispatch to tui-tester">
START: Need TUI testing?
│
├─► Does feature touch TUI code?
│   ├─ NO → Skip tui-tester, use regular unit tests
│   └─ YES → Continue
│
├─► What TUI component?
│   ├─ Mouse handling → Dispatch with mouse context
│   ├─ Keyboard handling → Dispatch with keyboard context
│   ├─ View/Focus → Dispatch with state context
│   └─ Widget rendering → Dispatch with render context
│
└─► Prepare context, then dispatch:
    Task(subagent_type="tui-tester", prompt="<context>...</context>")
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
├── Cargo.toml              # Workspace root
├── AGENTS.md               # This file
├── docs/                   # User Documentation
│   ├── get-started/
│   └── manual/
├── kb/                     # Engineering Handbook
│   ├── architecture/
│   ├── workflow/
│   └── internals/
├── crates/
│   ├── rush/
│   └── rstn/
├── specs/{NNN}-{name}/
└── target/
</repository-structure>

<knowledge-base>
**rustation v2 Documentation** (reorganized 2025-12-22):

**Engineering Handbook (`kb/`)**:
- `kb/README.md` - Start here for development
- `kb/architecture/01-state-first-principle.md` - **🎯 CORE PRINCIPLE**
- `kb/architecture/02-state-first-mvi.md` - **Runtime Model**
- `kb/workflow/sdd-workflow.md` - SDD Guide
- `kb/workflow/testing-guide.md` - Testing Guide

**User Documentation (`docs/`)**:
- `docs/README.md` - Start here for usage
- `docs/get-started/quick-start.md` - Quick Start
- `docs/manual/cli/commands.md` - Command Reference

**CRITICAL REQUIREMENTS for ALL features**:
1. **State tests MANDATORY**: Round-trip serialization + transitions + invariants
2. All state structs derive `Serialize + Deserialize + Debug + Clone`
3. NO hidden state
4. See `kb/architecture/01-state-first-principle.md`

**Development Workflow**:
- New feature? → See `kb/workflow/sdd-workflow.md`
- Writing tests? → See `kb/workflow/testing-guide.md`
- Contributing? → See `kb/workflow/contribution-guide.md`
</knowledge-base>

</grounding>

---

<negative-constraints>

<rule severity="NEVER">Change architecture without updating KB → Loss of source of truth → Update `kb/` first</rule>
<rule severity="NEVER">Block work on missing speckit artifacts → speckit is optional → Use KB-first instead</rule>
<rule severity="NEVER">Implement interactive flow without design diagrams → Leads to complexity → Use Design-First Planning</rule>
<rule severity="NEVER">Skip flow diagrams for rstn TUI features → Can't debug interaction → Create Mermaid diagrams in plan phase</rule>
<rule severity="NEVER">Implement without logging spec → No observability → Define what to log BEFORE coding</rule>
<rule severity="NEVER">Dispatch to tui-tester without context → Agent lacks info → Use context template</rule>
<rule severity="NEVER">Hardcode test coordinates → Breaks on resize → Calculate from layout rects</rule>
<rule severity="NEVER">Forget EnableMouseCapture → Mouse events won't work → Add to terminal setup</rule>
<rule severity="NEVER">Commit without running tests → Broken code enters repo → Run cargo test first</rule>
<rule severity="NEVER">Skip clippy → Lints accumulate → Run cargo clippy before commit</rule>
<rule severity="NEVER">Use -p + stream-json without --verbose → CLI error → Always add --verbose flag</rule>
<rule severity="NEVER">Use "transport" in MCP config → Invalid schema → Use "type" field instead</rule>
<rule severity="NEVER">Implement features without state tests → Untestable code → All features MUST have state serialization and transition tests</rule>
<rule severity="NEVER">Use concrete language code blocks (rust, python, shell) in `kb/` files → KB is for architecture, not implementation → Use `mermaid` or `pseudo-code` instead</rule>

</negative-constraints>

---

<delimiters>
Use these markers in workflow updates:

<marker name="STATUS">
Topic: multi-instance sessions
Phase: KB | IMPLEMENT | TEST
</marker>

<marker name="IMPLEMENTING">
Task: Add mouse click handler
File: crates/rstn/src/tui/app.rs
</marker>

<marker name="DISPATCHING TEST">
Agent: tui-tester
Focus: Mouse click on tab bar
</marker>

<marker name="BUILD CHECK">
cargo build: PASS
cargo test: PASS
cargo clippy: PASS
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
  <item>Unit tests written?</item>
  <item>TUI e2e tests dispatched to tui-tester?</item>
  <item>All tests pass?</item>
  <item>Edge cases covered?</item>
</checklist>

<checklist name="Commit">
  <item>Commit message format: feat(NNN): description?</item>
  <item>Changes are focused (not mixed features)?</item>
  <item>PR size reasonable (&lt;500 lines ideal)?</item>
</checklist>

If ANY item is NO, fix it before proceeding.
</self-correction>
