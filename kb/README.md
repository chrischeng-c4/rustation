# rustation v3 Engineering Handbook

**Target Audience**: Contributors, Developers, Maintainers

---

## Implemented Features

*Currently working in production*

| Document | Description |
|----------|-------------|
| [00. Architecture](implemented/00-architecture.md) | Electron + React + napi-rs architecture |
| [01. UI Components](architecture/01-ui-component-architecture.md) | Frontend component hierarchy & patterns |
| [01. State-First](implemented/01-state-first.md) | Core principle: JSON-serializable state |
| [02. State Topology](implemented/02-state-topology.md) | AppState tree structure |
| [03. Persistence](implemented/03-persistence.md) | Save/load application state |
| [04. Project Management](implemented/04-project-management.md) | Multi-project tabs, worktrees |
| [05. Docker Management](implemented/05-docker-management.md) | Container dashboard |
| [06. Tasks (Justfile)](implemented/06-tasks-justfile.md) | Justfile command runner |
| [07. Testing](implemented/07-testing.md) | Testing patterns |

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

## Development Workflow

| Document | Description |
|----------|-------------|
| [Contribution Guide](workflow/contribution-guide.md) | Dev setup & PR workflow |
| [SDD Workflow](workflow/sdd-workflow.md) | Specification-Driven Development |
| [Testing Guide](workflow/testing-guide.md) | Test patterns |
| [Definition of Done](workflow/definition-of-done.md) | PR checklist |

---

## Quick Reference

### Tech Stack
- **Desktop**: Electron
- **Frontend**: React 19 + Vite + Tailwind + shadcn/ui
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

---

## Documentation Principles

1. **KB-First**: Design changes documented here *before* implementation
2. **State-First**: All state must be JSON-serializable
3. **No Business Logic in React**: Frontend is display-only
