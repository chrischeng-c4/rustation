---
title: "v1→v2 Migration Guide"
description: "Migrating from v1 God classes to v2 state-first architecture"
category: concept
status: evergreen
last_updated: 2025-12-21
version: 0.2.0
tags: [migration, v1, v2, refactoring]
weight: 13
---

# v1→v2 Migration Guide

## v1 Problems (God Classes)

The v1 architecture suffered from:
- **God classes**: `WorktreeView` with 36+ fields
- **Scattered state**: 20+ state fields spread across subsystems
- **Implicit transitions**: No validation of state changes
- **Fragile tests**: UI-dependent tests that broke frequently

## v2 Solutions (State-First)

The v2 architecture addresses these issues with:
- **Serializable state**: All state structs derive `Serialize + Deserialize`
- **Explicit state machines**: Type-safe transitions with validation
- **State tests**: Test state directly, not UI
- **CLI/TUI separation**: Business logic independent of interface

## Migration Checklist

When migrating code from v1 to v2:
- [ ] Identify all state variables
- [ ] Ensure all state types are serializable
- [ ] Remove closures, thread-locals, and non-serializable types
- [ ] Write round-trip serialization tests
- [ ] Write state transition tests
- [ ] Write invariant validation tests

## Before/After Examples

This guide summarizes the key differences between v1 and v2. The full v1 analysis has been archived and removed from the active documentation.

