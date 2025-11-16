# Implementation Plan: History-Based Autosuggestions

**Branch**: `003-autosuggestions` | **Date**: 2025-11-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-autosuggestions/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement autosuggestions from command history that display inline (grayed-out text) as users type, enabling fast command reuse through Right Arrow acceptance. Feature follows fish shell patterns: show most recent match, update in real-time, accept full or partial (word-by-word) suggestions.

## Technical Context

**Language/Version**: Rust 1.75+ (Rust 2021 edition)
**Primary Dependencies**: reedline (line editing library with Hinter trait support)
**Storage**: File-based command history (existing FileBackedHistory from reedline)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: macOS (MVP), Linux (post-MVP)
**Project Type**: Single project (Rust monorepo crate)
**Performance Goals**: <50ms suggestion latency, <16ms UI response (60 FPS)
**Constraints**: <100ms startup overhead, <1MB memory overhead for 10k history
**Scale/Scope**: Support 10,000+ history entries without noticeable performance degradation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Performance-First ✅ PASS

- **Fast startup**: Suggestion system adds minimal initialization (~5ms to load history hints)
- **Instant responsiveness**: Real-time suggestions within 50ms, well under 16ms UI budget
- **Minimal overhead**: History search uses prefix matching (O(n) worst case, O(log n) with optimization)
- **Memory efficiency**: Suggestion cache <1MB for 10k entries
- **No blocking**: Async history search if needed for large histories

**Justification**: Feature designed for performance - simple prefix matching, no complex parsing.

### Principle II: Zero-Config Philosophy ✅ PASS

- **Sensible defaults**: Autosuggestions enabled by default (fish shell pattern)
- **Fish-like UX**: Exact fish behavior (grayed text, Right Arrow acceptance)
- **No mandatory setup**: Works immediately with existing command history
- **Configuration optional**: Future config could disable or customize, but not required

**Justification**: Zero setup required - feature uses existing history file and reedline infrastructure.

### Principle III: Progressive Complexity ✅ PASS

- **Layered functionality**:
  - Basic: Display suggestions (US1)
  - Intermediate: Accept full suggestions (US2)
  - Advanced: Accept partial/word-by-word (US3)
- **No forced complexity**: Users can ignore suggestions if desired
- **Learn as you go**: Suggestions appear naturally, no manual needed

**Justification**: Three-tier user story structure aligns with progressive complexity.

### Principle IV: Modern UX ✅ PASS

- **Autosuggestions**: Core feature delivering modern UX expectation
- **Visual feedback**: Grayed-out text clearly indicates suggestion vs input
- **Accessible design**: Color scheme respects terminal capabilities (fallback to dimmed text)

**Justification**: This feature IS modern UX - directly implements Principle IV.

### Principle V: Rust-Native ✅ PASS

- **Pure Rust**: Implementation uses reedline Hinter trait (pure Rust)
- **Ecosystem integration**: Leverages existing reedline, no new dependencies
- **Zero-cost abstractions**: Trait-based design (compile-time dispatch)
- **Memory safety**: No unsafe code required
- **Idiomatic code**: Follows Rust trait patterns

**Justification**: Builds on existing reedline infrastructure, fully idiomatic Rust.

**GATE STATUS**: ✅ ALL GATES PASS - No violations, proceed to Phase 0

## Project Structure

### Documentation (this feature)

```text
specs/003-autosuggestions/
├── spec.md              # Feature specification (completed)
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (reedline Hinter trait research)
├── data-model.md        # Phase 1 output (Suggestion/Match entities)
├── quickstart.md        # Phase 1 output (manual testing guide)
├── contracts/           # Phase 1 output (Hinter trait contract)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/rush/
├── src/
│   ├── repl/
│   │   ├── mod.rs         # REPL coordinator (integrate hinter)
│   │   ├── suggest.rs     # NEW: Autosuggestion implementation
│   │   ├── highlight.rs   # Existing syntax highlighting
│   │   └── input.rs       # Existing input handling
│   └── history/
│       └── mod.rs         # Existing history management (query support)
│
└── tests/
    ├── unit/
    │   └── suggest_tests.rs   # NEW: Unit tests for suggestion logic
    └── integration/
        └── autosuggestions_tests.rs  # NEW: End-to-end suggestion tests
```

**Structure Decision**: Single project structure (Option 1) applies. Feature adds new `src/repl/suggest.rs` module implementing reedline's `Hinter` trait. Integrates with existing REPL infrastructure in `src/repl/mod.rs` and leverages existing history from `src/history/mod.rs`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations detected - section not needed.

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**CRITICAL: Keep PRs small and reviewable (see CLAUDE.md for limits).**

**Strategy**: PR per User Story (RECOMMENDED for multi-story features)

```
PR #1: Foundation + US1 - Basic Inline Suggestion Display
  - Implement Hinter trait
  - Add basic prefix matching from history
  - Display grayed-out suggestions
  - Unit tests for matching logic
  - Target: ≤ 800 lines

PR #2: US2 - Accept Suggestion with Right Arrow
  - Add keybinding for Right Arrow acceptance
  - Implement full suggestion acceptance
  - Integration tests for acceptance flow
  - Target: ≤ 600 lines

PR #3: US3 - Accept Partial Suggestion
  - Add keybinding for Alt+Right Arrow
  - Implement word-by-word acceptance
  - Edge case handling (cursor position, truncation)
  - Target: ≤ 700 lines

PR #4: Polish & Documentation
  - Update README and KNOWN_ISSUES
  - Performance optimization if needed
  - Additional edge case tests
  - Target: ≤ 400 lines
```

### Selected Strategy

Using **Option 2: PR per User Story** - 4 separate PRs

**Rationale**: 3 user stories × ~700 lines each + polish = 4 PRs, all under 1,500 line limit

### Merge Sequence

1. PR #1: Foundation + US1 → Merge to main (MVP: suggestions display)
2. PR #2: US2 → Merge to main (Complete: suggestions can be accepted)
3. PR #3: US3 → Merge to main (Power user: partial acceptance)
4. PR #4: Polish → Merge to main (Production ready)

**Branch Strategy**: Work on `003-autosuggestions` branch, create sequential PRs as each user story completes

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

If any PR exceeds limits, split into smaller increments.
