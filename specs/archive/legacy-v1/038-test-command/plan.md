# Implementation Plan: Extended Test Command

**Branch**: `038-test-command` | **Date**: 2025-12-10 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/038-test-command/spec.md`

## Summary

Implement the `[[ ]]` extended test command for robust conditional testing in shell scripts. This provides bash-compatible string/numeric comparisons, glob pattern matching, regex support via `=~`, and complex conditional logic with `&&`/`||` operators. Unlike the traditional `[` command, `[[` handles unquoted variables safely without word splitting and provides more powerful pattern matching capabilities.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: regex crate (for `=~` operator), nix crate (for file test operators)
**Storage**: N/A (stateless command execution)
**Testing**: cargo test (unit and integration tests)
**Target Platform**: macOS (MVP), Linux (post-MVP)
**Project Type**: Single (monorepo workspace)
**Performance Goals**: < 1ms for simple expressions, < 10ms for complex expressions with regex
**Constraints**: < 200ms p95 for worst-case regex evaluation, no blocking I/O
**Scale/Scope**: ~20 operators, 3 user stories, estimated 1,500-2,000 lines total

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Performance-First ✅ PASS

**Evaluation**: Feature aligns with performance principle
- Test command execution overhead: < 1ms for simple conditionals
- Regex evaluation: < 10ms with pattern length limits (10KB) to prevent ReDoS
- No blocking operations: File test operators use sync calls but complete in microseconds
- Memory efficient: Stateless execution, temporary allocations only during test evaluation
- Short-circuit evaluation: `&&`/`||` stop evaluating when result determined

**Justification**: Test commands are fundamental to shell performance. Fast evaluation critical for script responsiveness.

### II. Zero-Config Philosophy ✅ PASS

**Evaluation**: Feature requires zero configuration
- Works immediately with sensible defaults
- No setup required
- All operators available out-of-box
- BASH_REMATCH array automatically populated for regex matches
- No configuration files needed

**Justification**: Test commands must work immediately for basic scripting. Zero friction for users.

### III. Progressive Complexity ✅ PASS

**Evaluation**: Feature follows progressive complexity model
- **Basic (P1)**: Simple string/numeric comparisons - easy to learn and use
- **Intermediate (P2)**: Pattern matching with globs and regex - discovered when needed
- **Advanced (P3)**: Complex logic with `&&`/`||`/`()` - for sophisticated scripts
- Users pay zero cost for unused operators (dead code elimination)
- Help system and error messages guide discovery

**Justification**: Beginners start with simple comparisons. Advanced users access full power naturally.

### IV. Modern UX ✅ PASS

**Evaluation**: Feature enhances UX
- Clear error messages for syntax errors and invalid operators
- Helpful feedback for unset variables and type mismatches
- Exit codes follow conventions (0=true, 1=false, 2=error)
- BASH_REMATCH provides discoverable regex captures
- Pattern matching syntax familiar to bash/zsh users

**Justification**: Good error messages and familiar syntax reduce friction.

### V. Rust-Native ✅ PASS

**Evaluation**: Feature leverages Rust ecosystem
- **regex crate**: Battle-tested, maintained by Rust team, zero-overhead abstractions
- **nix crate**: Safe syscall wrappers for file tests (already in project)
- Pure Rust implementation, no FFI needed
- Type-safe parsing and evaluation
- Zero-cost abstractions via enums and match expressions

**Justification**: Regex crate provides POSIX ERE support efficiently. Nix crate already used for signals/process management.

**Constitution Status**: ✅ **ALL PRINCIPLES SATISFIED** - No violations to justify

## Project Structure

### Documentation (this feature)

```text
specs/038-test-command/
├── spec.md              # Feature specification (complete)
├── plan.md              # This file
├── research.md          # Technical decisions (to be created in Phase 0)
├── data-model.md        # Core entities (to be created in Phase 1)
├── quickstart.md        # Usage examples (to be created in Phase 1)
├── contracts/           # API specification (to be created in Phase 1)
│   └── test-api.md
├── checklists/          # Quality validation
│   └── requirements.md  # Spec quality checklist (complete)
└── tasks.md             # Implementation tasks (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/rush/
├── src/
│   ├── parser/
│   │   ├── mod.rs
│   │   └── test_expr.rs          # NEW: Parse [[ ]] syntax
│   ├── executor/
│   │   ├── builtins/
│   │   │   ├── mod.rs             # MODIFIED: Register [[ builtin
│   │   │   └── test_extended.rs   # NEW: Extended test command implementation
│   │   └── execute.rs
│   ├── expansion/
│   │   └── mod.rs                 # Use existing expansion for variable substitution
│   └── lib.rs                     # MODIFIED: Add TestError variants
└── tests/
    ├── integration/
    │   └── test_extended_test.rs  # NEW: Integration tests
    └── unit/
        └── test_expr_parser.rs    # NEW: Parser unit tests
```

**Structure Decision**: Single project structure (Option 1). Extended test command is a core builtin that fits naturally into existing `crates/rush/src/executor/builtins/` directory. Parser additions go into `src/parser/` for expression parsing. All tests in `tests/` directory.

## Complexity Tracking

> No complexity violations - Constitution Check passed all principles.

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**Strategy**: PR per User Story (Option 2)

The feature has 3 user stories with estimated sizes:
- **Foundation + Setup**: ~200 lines (parser infrastructure, error types)
- **US1 (Basic conditionals)**: ~800 lines (operators, parsing, evaluation)
- **US2 (Pattern matching)**: ~500 lines (glob matching, regex support)
- **US3 (Complex logic)**: ~400 lines (&&/||/() logic, short-circuit)
- **Polish**: ~100 lines (docs, edge case hardening)

**Total**: ~2,000 lines across 4 PRs

### Selected Strategy

**Option 2: PR per User Story** (RECOMMENDED for multi-story features)

```
PR #1: Foundation + Setup
  - Project structure, error types, parser scaffolding
  - Target: ~200 lines

PR #2: User Story 1 (P1 - Basic Conditional Testing)
  - String operators (==, !=, <, >)
  - Numeric operators (-eq, -ne, -lt, -le, -gt, -ge)
  - String tests (-z, -n)
  - File operators (-f, -d, -e, -r, -w, -x, -s)
  - Variable handling without word splitting
  - Target: ~800 lines

PR #3: User Story 2 (P2 - Pattern Matching and Regex)
  - Glob pattern matching with == and !=
  - Regex matching with =~
  - BASH_REMATCH array population
  - Target: ~500 lines

PR #4: User Story 3 (P3 - Complex Conditional Logic)
  - Logical operators (&&, ||, !)
  - Parentheses for grouping
  - Short-circuit evaluation
  - Target: ~400 lines

PR #5: Polish & Integration
  - Documentation updates
  - Edge case hardening
  - Performance optimization
  - Target: ~100 lines
```

**Rationale**: Each PR delivers independently testable functionality. US1 provides MVP (basic comparisons). US2 adds pattern matching value. US3 completes feature with complex expressions.

### Merge Sequence

1. **PR #1**: Foundation + Setup → Merge to main
2. **PR #2**: User Story 1 (Basic conditionals) → Merge to main (MVP complete)
3. **PR #3**: User Story 2 (Pattern matching) → Merge to main
4. **PR #4**: User Story 3 (Complex logic) → Merge to main
5. **PR #5**: Polish → Merge to main (Feature complete)

**Branch Strategy**: Create `038-test-command` base branch, then create feature branches from updated main after each merge: `038-test-command-us1`, `038-test-command-us2`, etc.

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

All planned PRs are within limits (largest is US1 at ~800 lines).
