---
title: "Architecture Principles"
description: "Four pillars: State-First, Frontend/Backend Separation, Backend-Driven UI"
category: concept
status: evergreen
last_updated: 2025-12-24
version: 3.0.0
tags: []
weight: 0
---

# Core Principles - rustation v3 (Tauri)

**Last Updated**: 2025-12-24
**Status**: Active (v3 Rewrite)

Welcome to rustation v3! This document outlines the core architectural principles for the Tauri-based GUI rewrite.

---

## 🎯 The Three Pillars

### 1. State-First Architecture (Preserved)

**Principle**: The Rust Backend remains the **Single Source of Truth**.

**Why**: Logic reliability, testing, persistence.

**Key Rules**:
- ✅ `AppState` lives in Rust (Arc<RwLock<AppState>>).
- ✅ Frontend (React) contains *no business logic*, only display logic.
- ✅ State updates are pushed from Rust -> Frontend via Events.
- ❌ Frontend never mutates state directly; it invokes Commands.

### 2. Frontend/Backend Separation

**Principle**: Explicit boundary between Presentation (Webview) and Logic (Core).

**Architecture**:
```
┌─────────────────────────────────────────────┐
│              Frontend (Webview)             │
│   ┌──────────────┐     ┌──────────────┐     │
│   │ React        │     │ State Sync   │     │
│   │ Components   │ ◄── │ (Zustand)    │     │
│   └──────┬───────┘     └──────────────┘     │
│          │ Invoke             ▲ Event       │
└──────────┼────────────────────┼─────────────┘
           │                    │
┌──────────▼────────────────────┼─────────────┐
│              Backend (Rust)   │             │
│   ┌──────────────┐     ┌──────┴───────┐     │
│   │ Commands     │     │ AppState     │     │
│   │ (Interface)  │ ──► │ (Logic)      │     │
│   └──────────────┘     └──────────────┘     │
└─────────────────────────────────────────────┘
```

**Key Rules**:
- ✅ **Commands**: Typed API for actions (`run_workflow`, `save_settings`).
- ✅ **Events**: Real-time updates (`state:update`, `log:entry`).
- ✅ **Types**: TypeScript types generated from Rust structs (via `ts-rs` or similar).

### 3. Workflow-Driven UI

**Principle**: The GUI is organized around Tasks, not Files.

**Structure**:
1.  **Workflows Tab**: Prompt-to-Code, Git Operations.
2.  **Dockers Tab**: Container Management.
3.  **Settings Tab**: Configuration.

---

## Design Philosophy

### Native Feel, Web Speed
- Use `shadcn/ui` for a professional, consistent look.
- Use `Tauri` for OS integration (Notifications, Tray, FS).

### Observability
- Backend logs (`tracing`) are streamed to the Frontend console in Dev mode.
- Production logs are written to `~/.rstn/logs/`.

---

## Anti-Patterns

### ❌ Fat Frontend
**Avoid**: calculating business rules in TypeScript.
**Do**: Calculate in Rust, send result to TS.

### ❌ Split Brain State
**Avoid**: Maintaining independent state in React that isn't synced or transient.
**Do**: Use `react-query` or `useSyncExternalStore` patterns driven by Rust events.

---

## Reference

- [System Specification](01-system-specification.md)
- [State-First Principle](02-state-first-principle.md)