---
title: "Contribution Guide (Tauri GUI)"
description: "Setup, workflow, and guidelines for rustation v3 (Tauri)"
category: contribute
status: implemented
last_updated: 2025-12-24
version: 3.0.0
tags: [tauri, react, guide]
weight: 1
---

# Contribution Guide (Tauri GUI)

Welcome to rustation v3! This version uses **Tauri v2** with a **React** frontend.

---

## Prerequisites

### Required Tools
- **Rust 1.77+**: Backend logic and Tauri host.
- **Node.js 20+** and **npm/pnpm**: Frontend development.
- **Git**: Version control.

### Recommended Tools
- **Tauri CLI**: `cargo install tauri-cli`.
- **VS Code Extensions**: Rust-analyzer, Tailwind CSS, ES7+ React/Redux/React-Native snippets.

---

## Development Environment Setup

### 1. Clone & Install
```bash
git clone https://github.com/chrischeng-c4/rustation.git
cd rustation
npm install
```

### 2. Run in Development Mode
This starts the Vite dev server and the Tauri window with HMR (Hot Module Replacement).
```bash
npm run tauri dev
```

### 3. Run Tests
- **Backend (Rust)**: `cargo test`
- **Frontend (React)**: `npm test` (Vitest)

---

## 🎯 MANDATORY: State-First Testing (Backend)

The **State-First** principle remains our core architecture. Every feature MUST include state serialization and transition tests in Rust.

```rust
// src-tauri/src/state/your_feature.rs

#[test]
fn test_round_trip() {
    // Ensure state serializes/deserializes correctly for Frontend sync
}

#[test]
fn test_transition() {
    // Ensure Tauri Commands mutate state correctly
}
```

---

## Contribution Workflow

### 1. Specification (SDD)
All non-trivial changes must have a spec in `specs/`.
- Use `/speckit.specify` to define requirements.

### 2. Implementation Pattern
1.  **Backend State**: Define the state in Rust (`src-tauri/src/state/`).
2.  **Backend Logic**: Implement Tauri Commands (`src-tauri/src/commands/`).
3.  **Frontend UI**: Create React components (`src/components/`) and sync state.
4.  **Wiring**: Use `invoke` to call backend commands and `listen` for state updates.

### 3. Code Style
- **Rust**: Follow `clippy` and `rustfmt`. No `unwrap()` in command handlers.
- **React**: Functional components, Tailwind CSS for styling, Shadcn UI for primitives.
- **TypeScript**: Strict typing mandatory.

---

## PR Requirements
- [ ] Backend State tests included.
- [ ] Frontend builds without warnings.
- [ ] No business logic in React (Logic belongs in Rust).
- [ ] `npm run lint` and `cargo clippy` pass.

---

## Quick Commands
| Command | Action |
| :--- | :--- |
| `npm run tauri dev` | Start development app |
| `npm run tauri build` | Build production installer |
| `cargo test` | Run Rust tests |
| `npm run lint` | Run ESLint/Prettier |
| `npm run type-check` | Run TypeScript compiler check |