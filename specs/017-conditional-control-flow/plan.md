# Implementation Plan: Conditional Control Flow

**Branch**: `017-conditional-control-flow-if-then-else-elif-fi` | **Date**: 2025-12-06 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/017-conditional-control-flow/spec.md`

## Summary

Implement POSIX-compatible conditional control flow (`if/then/elif/else/fi`) for the rush shell. The parser will be extended to recognize reserved keywords and build AST nodes for conditional blocks. A new conditional executor will evaluate condition exit codes and execute the appropriate branch. The REPL will buffer incomplete constructs and display continuation prompts until `fi` is entered.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: reedline 0.35 (line editing), tokio (async I/O), anyhow/thiserror (error handling)
**Storage**: N/A (in-memory AST during execution)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: macOS (MVP), Linux post-MVP
**Project Type**: Single (Rust monorepo workspace)
**Performance Goals**: Conditional parsing overhead <1ms, execution overhead <5ms vs direct spawn
**Constraints**: <100ms shell startup, <16ms prompt response, <10MB memory baseline
**Scale/Scope**: Support nesting to 10+ levels, support 1000+ line scripts with conditionals

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Justification |
|-----------|--------|---------------|
| I. Performance-First | PASS | Conditional parsing is O(n) single-pass. No dynamic memory allocation beyond AST nodes. Exit code check is trivial comparison. |
| II. Zero-Config | PASS | Conditionals work out-of-box with POSIX syntax. No configuration required. |
| III. Progressive Complexity | PASS | Basic if/then/fi is simple. Nesting and elif are opt-in complexity. |
| IV. Modern UX | PASS | Continuation prompts provide visual feedback. Syntax errors show expected token. |
| V. Rust-Native | PASS | Pure Rust implementation. Uses existing reedline. No new dependencies. |

**Gate Status**: PASSED - All principles satisfied.

## Project Structure

### Documentation (this feature)

```text
specs/017-conditional-control-flow/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (AST structures)
├── quickstart.md        # Phase 1 output
├── contracts/           # N/A - no external API
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/rush/src/
├── executor/
│   ├── mod.rs           # Add: CompoundCommand enum, IfBlock struct
│   ├── parser.rs        # Modify: Add tokenization for keywords, parse_compound()
│   ├── conditional.rs   # NEW: Conditional block evaluation and execution
│   ├── execute.rs       # Modify: Integrate conditional execution
│   └── builtins/
│       └── keywords.rs  # NEW: Reserved keyword handling
├── repl/
│   ├── mod.rs           # Modify: Add continuation prompt logic
│   └── validator.rs     # Modify: Detect incomplete conditionals
└── lib.rs               # Add: RushError::Syntax variant

tests/
├── integration/
│   ├── conditionals.rs  # NEW: Integration tests for if/then/else/elif/fi
│   └── multiline.rs     # NEW: Interactive multiline tests
└── unit/
    └── parser_conditional.rs  # NEW: Parser unit tests
```

**Structure Decision**: Single project structure. All changes in `crates/rush/src/executor/` for core logic, `crates/rush/src/repl/` for interactive behavior.

## Complexity Tracking

No constitution violations. No complexity justification needed.

## Deployment Strategy

### Pull Request Plan

**Selected Strategy**: Option 2 - PR per User Story (RECOMMENDED)

**Rationale**: 5 user stories, each independently testable. P1-P3 are core functionality (~1,000-1,200 lines each). P4 (nesting) and P5 (multiline) are enhancements (~800-1,000 lines each). Total estimated ~5,000 lines across 5 PRs.

### Merge Sequence

1. **PR #1: Foundation + Core AST** (~500 lines)
   - Add AST structures to `mod.rs` (CompoundCommand, IfBlock, ElifClause)
   - Add reserved keyword recognition in parser
   - Add RushError::Syntax variant
   - Tests: Unit tests for tokenization of keywords

2. **PR #2: US1 - Simple if/then/fi** (~1,200 lines)
   - Parse basic if/then/fi constructs (single-line)
   - Implement conditional executor in `conditional.rs`
   - Integrate with execute.rs
   - Tests: if true; if false; command exit codes

3. **PR #3: US2 - else clause** (~800 lines)
   - Extend parser for else keyword
   - Extend executor for else branch
   - Tests: if/else scenarios, branch selection

4. **PR #4: US3 - elif clause** (~1,000 lines)
   - Extend parser for multiple elif clauses
   - Implement short-circuit evaluation
   - Tests: Multi-branch selection, first-match-wins

5. **PR #5: US4 - Nested conditionals** (~800 lines)
   - Recursive parsing support
   - Stack-based execution for nesting
   - Tests: 3+ level nesting, combined with elif

6. **PR #6: US5 - Interactive multiline** (~1,000 lines)
   - REPL continuation prompt logic
   - Input buffering until fi
   - Validator for incomplete constructs
   - Tests: Interactive scenarios, error recovery

**Branch Strategy**: Create `017-conditional-control-flow` base branch, then individual PRs merge to main.

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- Ideal: ≤ 500 lines
- Maximum: ≤ 1,500 lines
- Too large: > 3,000 lines (must split)
