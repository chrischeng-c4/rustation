---
title: "System Requirements & High-Level Design"
description: "User requirements mapping to architectural components"
category: architecture
status: draft
last_updated: 2025-12-22
version: 0.1.0
tags: [architecture, requirements, layout, persistence]
weight: 1
---

# System Requirements & High-Level Design

This document maps the user's specific requirements to the system architecture.

## 1. Interaction Layer (Bottom)

**Requirement**: Global status and shortcuts.
- **`y` (Copy View)**: Captures the current visible rendered text (UI snapshot).
- **`Y` (Copy State)**: Captures the current `AppState` (Serialization snapshot).

**Architecture Implication**:
- Global KeyHandler in the main loop needs to intercept `y`/`Y` before dispatching to Views.
- `AppState` must implement `Serialize`.
- UI rendering must support "export to string".

## 2. Navigation Layer (Top)

**Requirement**: Tab Bar focused on **[Workflows]**.
- Primary View: **Workflows** (Command Launcher).
- Other views (Dashboard, Settings) are secondary or accessible via commands.

**Architecture Implication**:
- `CurrentView` enum defaults to `Workflows`.
- UI Layout puts a Tab Bar at `Rect { y: 0, height: 1 }`.

## 3. Content Layer (Middle - Split Layout)

**Requirement**: Two-column layout.
- **Left (15-20%)**: Workflow List / Command List.
  - Initial Command: "Prompt to CC" (Claude Code).
- **Right (80-85%)**: Dynamic Content Area.
  - Driven by: Active Workflow Node State.

**Architecture Implication**:
- **Layout Template**: `Standard` (Sidebar + Content).
- **Layout Constraints**:
  - `sidebar_width`: Percentage-based (15-20%) or fixed width (~25-30 chars).
- **Dynamic Content**:
  - `WorktreeView` (or `WorkflowView`) renders the Right Panel.
  - It switches rendering logic based on `state.active_workflow`.

## 4. Background / Invisible Layer

### A. Communication (rstn-mcp)
**Requirement**: MCP server for communication with Claude Code.
- **Role**: Bridge between `rstn` TUI and `claude` subprocess.

### B. Observability (Logs)
**Requirement**: Structured logs in `~/.rstn/logs`.
- **Format**: JSON lines or structured text.

### C. Persistence (SQLite)
**Requirement**: State persistence via SQLite.
- **Role**:
  - Persistent Store for History (Sessions, Chat Logs).
  - Snapshot Store for `AppState` (Resume session).
- **Change from v2.0**:
  - Previous: `session.yaml` (Plain text/YAML).
  - New: `rstn.db` (SQLite).

## Gap Analysis & Action Plan

1.  **SQLite Integration**:
    - Need to define schema for State persistence.
    - Need to decide: Is `AppState` stored as a JSON blob in SQLite, or relational tables?
    - *Recommendation*: `AppState` as a JSON blob for full restore, specific tables for queryable History.

2.  **Workflow-Driven UI**:
    - Need to refactor `WorktreeView` to be explicitly "Workflow-Driven".
    - Define "Prompt to CC" as the first `WorkflowDef`.

3.  **Global Shortcuts**:
    - Implement the `y`/`Y` logic in the main event loop.
