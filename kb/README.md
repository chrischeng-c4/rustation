# rustation v3 Engineering Handbook

**Target Audience**: Contributors, Developers, Maintainers.

---

## Architecture 🏗️
*Tauri-based GUI Architecture*

### Level 1: Foundation
| Document | Description | Status |
|----------|-------------|--------|
| [00. Overview](architecture/00-overview.md) | The Pillars: State-First, Frontend/Backend Separation | 🟢 Active |
| [01. System Specification](architecture/01-system-specification.md) | **Master Spec**: Tech stack, Layout, Data Flow | 🟢 Active |

### Level 2: Core Architecture
| Document | Description | Status |
|----------|-------------|--------|
| [02. State-First Principle](architecture/02-state-first-principle.md) | Core principle: Rust as Source of Truth | 🟢 Active |
| [03. State Topology](architecture/03-state-topology.md) | Structure of the AppState tree | 🟢 Active |
| [05. Serialization](architecture/05-serialization.md) | Persistence rules | 🟢 Active |

### Level 3: Features (To Be Updated for GUI)
| Document | Description | Status |
|----------|-------------|--------|
| [09. Workflows Tab](architecture/09-workflow-prompt-claude.md) | Prompting & Agent integration | 🟡 Needs Update |
| [10. Dockers Tab](architecture/10-docker-management.md) | Container Management | 🟡 Needs Update |
| [11. MCP Server](architecture/11-mcp-server.md) | Backend Integration | 🟢 Reusable |

### Level 4: Meta
| Document | Description | Status |
|----------|-------------|--------|
| [13. Migration](architecture/13-migration-from-v1.md) | v1 legacy context | 🟢 Reference |

---

## Workflow & Standards 🛠️
*Development processes*

| Document | Description | Status |
|----------|-------------|--------|
| [Contribution Guide](workflow/contribution-guide.md) | PR requirements | 🟡 Needs Update |
| [SDD Workflow](workflow/sdd-workflow.md) | Specification-Driven Development | 🟢 Active |

---

## Legend

- 🟢 **Active** - Current source of truth
- 🟡 **Needs Update** - Contains TUI specific info, needs migration to GUI concepts
- 🔴 **Deprecated** - Kept for reference only

---

## Documentation Principles

This `kb/` directory is the **Source of Truth** for the codebase.
- **KB-First**: Design changes must be documented here *before* implementation.
