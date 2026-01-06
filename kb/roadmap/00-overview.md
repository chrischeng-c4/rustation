---
title: "Roadmap Overview"
description: "Planned features for future development"
category: roadmap
status: planned
last_updated: 2026-01-06
version: 3.0.0
---

# Roadmap Overview

## Current State (v3.0)

**Implemented**:
- Multi-project management with tabs
- Git worktree support
- Docker container management
- Justfile task runner
- State persistence

**Architecture**: Electron + React + napi-rs (Rust)

---

## Planned Features

### Phase 0: UI System Consolidation
**Status**: In progress

- Replace Tailwind/shadcn/Radix UI with MUI-only components
- Migrate icons to `@mui/icons-material`
- Remove legacy `components/ui` wrappers and related CSS utilities

### Phase 1: Settings UI
**Status**: Partial (state exists, no UI)

- Theme switcher (Light/Dark/System)
- Default project path configuration
- Editor preferences

### Phase 2: MCP Server Integration
**Status**: Not started

- Embedded MCP server per worktree
- Claude Code integration
- Tool specifications for project context

### Phase 3: Prompt-to-Claude Workflow
**Status**: Not started

- Prompt input panel
- Streaming response display
- Conversation history
- Context injection (files, state)

---

## Architecture Decisions

### Why Electron over Tauri?

| Factor | Electron | Tauri |
|--------|----------|-------|
| **Ecosystem** | Mature, stable | Newer, less tooling |
| **React Support** | Native | Via webview |
| **napi-rs** | First-class | Would need Rust backend anyway |
| **Build Size** | Larger (~150MB) | Smaller (~10MB) |
| **Decision** | **Chosen** | Considered |

### Why napi-rs over pure Node?

- **Performance**: Rust for CPU-bound work
- **Type Safety**: Rust types flow to TypeScript
- **Reusability**: Core logic shared if we ever port

---

## Timeline

No specific dates - features implemented as needed.

---

## Contributing

1. Check this roadmap for planned features
2. Read [Core Architecture](../architecture/) for architecture
3. Follow state-first principles
4. Add tests for all new code
