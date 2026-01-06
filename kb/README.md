# rustation v3 Engineering Handbook

**Target Audience**: Contributors, Developers, Maintainers

---

## Core Architecture

*Foundation and Principles*

| Document | Description |
|----------|-------------|
| [00. Overview](architecture/00-overview.md) | Electron + React + napi-rs architecture |
| [01. State-First](architecture/01-state-first.md) | Core principle: JSON-serializable state |
| [02. State Topology](architecture/02-state-topology.md) | AppState tree structure |
| [03. Persistence](architecture/03-persistence.md) | Save/load application state |
| [07. Testing](architecture/07-testing.md) | Testing patterns |

---

## Feature Specifications

*Domain Logic and Features*

| Document | Description |
|----------|-------------|
| [Project Management](features/project-management.md) | Multi-project tabs, worktrees |
| [Docker Management](features/docker-management.md) | Container dashboard |
| [Tasks (Justfile)](features/tasks-justfile.md) | Justfile command runner |

---

## Development Workflow

*Processes and Standards*

| Document | Description |
|----------|-------------|
| [Contribution Guide](workflow/contribution-guide.md) | Dev setup & PR workflow |
| [SDD Workflow](workflow/sdd-workflow.md) | Specification-Driven Development |
| [Testing Guide](workflow/testing-guide.md) | Test patterns |
| [Definition of Done](workflow/definition-of-done.md) | PR checklist |

---

## Roadmap (Future)

*Planned but not yet implemented*

| Document | Description | Status |
|----------|-------------|--------|
| [00. Overview](roadmap/00-overview.md) | Future vision | - |
| [01. MCP Integration](roadmap/01-mcp-integration.md) | Claude Code integration | Planned |
| [02. Prompt Claude](roadmap/02-prompt-claude.md) | Conversation interface | Planned |
| [03. Settings UI](roadmap/03-settings-ui.md) | Settings form | Partial |

---

## Experimental

*Features in early prototyping phase*

| Document | Description |
|----------|-------------|
| [A2UI Integration](experimental/a2ui.md) | Server-Driven UI via JSON |

---

## Quick Reference

### Tech Stack
- **Desktop**: Electron
- **Frontend**: React 19 + Vite + MUI (Material UI)
- **Backend**: napi-rs (Rust)
- **State**: Rust AppState (JSON-serializable)

### Commands
```bash
# Development
cd apps/desktop && pnpm dev

# Build
cd packages/core && pnpm build
cd apps/desktop && pnpm build

# Tests
cargo test                    # Rust tests
pnpm test                     # React tests
pnpm test:e2e                 # Playwright E2E
```

### Key Directories
```
packages/core/src/    # Rust napi-rs module
apps/desktop/src/     # Electron + React app
kb/                   # This documentation
docs/                 # User documentation
```
