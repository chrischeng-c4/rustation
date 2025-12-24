# rustation v2 Engineering Handbook

**Target Audience**: Contributors, Developers, Maintainers.
**For Users**: See [`docs/`](../docs/README.md) for installation and usage guides.

---

## Architecture 🏗️
*From Principles (High) to Features (Low)*

### Level 1: Foundation
| Document | Description | Status |
|----------|-------------|--------|
| [00. Overview](architecture/00-overview.md) | The Three Pillars: State-First, CLI/TUI, Testing | 🟢 Active |
| [01. Requirements](architecture/01-system-requirements.md) | User requirements & High-level design | 🟢 Active |

### Level 2: Core Architecture
| Document | Description | Status |
|----------|-------------|--------|
| [02. State-First Principle](architecture/02-state-first-principle.md) | Core principle: all state serializable | 🟢 Active |
| [03. State Topology](architecture/03-state-topology.md) | Structure of the AppState tree | 🟢 Active |
| [04. State-First MVI](architecture/04-state-first-mvi.md) | **Runtime Model**: Msg → Reduce → State → Render | 🟢 Active |
| [05. Serialization](architecture/05-serialization.md) | Rules, patterns, anti-patterns | 🟢 Active |

### Level 3: UI Architecture
| Document | Description | Status |
|----------|-------------|--------|
| [06. Layout Engine](architecture/06-layout-management.md) | Layout as State & Templates | 🟢 Active |
| [07. Tab Management](architecture/07-tab-management.md) | Top-Level Navigation (Workflows/Dockers/Settings) | 🟢 Active |
| [08. Keybindings](architecture/08-keybindings.md) | Input handling & Shortcuts | 🟢 Active |

### Level 4: Features
| Document | Description | Status |
|----------|-------------|--------|
| [09. Workflows Tab](architecture/09-workflow-prompt-claude.md) | Prompting & Agent integration | 🟢 Active |
| [10. Dockers Tab](architecture/10-docker-management.md) | Container Management | 🟡 Draft |
| [11. MCP Server](architecture/11-mcp-server.md) | Backend Integration | 🟢 Active |

### Level 5: Meta & Legacy
| Document | Description | Status |
|----------|-------------|--------|
| [12. Testing State](architecture/12-testing-state.md) | Verification Principles | 🟢 Active |
| [13. Migration](architecture/13-migration-from-v1.md) | v1 problems → v2 solutions | 🟢 Active |

---

## Workflow & Standards 🛠️
*Development processes and guidelines*

| Document | Description | Status |
|----------|-------------|--------|
| [Contribution Guide](workflow/contribution-guide.md) | PR requirements, code style | 🟢 Implemented |
| [SDD Workflow](workflow/sdd-workflow.md) | Specification-Driven Development guide | 🟢 Implemented |
| [Testing Guide](workflow/testing-guide.md) | How to write state & MVI tests | 🟢 Implemented |
| [Debugging](workflow/debugging.md) | State inspection, logs, troubleshooting | 🟢 Implemented |

---

## Internals ⚙️
*Deep dives into subsystems*

| Document | Description | Status |
|----------|-------------|--------|
| [MCP Tools](internals/mcp/tools.md) | Internal MCP tool schemas and protocol | 🟢 Implemented |

---

## Legend

- 🟢 **Active/Implemented** - Current source of truth
- 🟡 **Draft** - Work in progress
- 🔴 **Deprecated** - Kept for reference only

---

## Documentation Principles

This `kb/` directory is the **Source of Truth** for the codebase.
- **Code follows KB**: If code contradicts KB, code is wrong (or KB needs update).
- **KB-First**: Design changes must be documented here *before* implementation.