# CLAUDE.md

## Language Preference

Respond in English (U.S.) by default. Use Traditional Chinese only when user writes in Traditional Chinese.

## Repository Overview

Rust monorepo workspace containing **rush** - a shell implementation replacing zsh/bash/fish.

```
rust-station/
├── Cargo.toml          # Workspace root
├── crates/rush/        # Shell implementation
├── specs/              # Feature specifications
│   └── features.json   # Master feature catalog (001-044)
└── target/             # Build output (gitignored)
```

## Spec-Driven Development Workflow

Use spec-kit commands for all feature development:

```
/speckit.specify  → spec.md      # Define requirements
/speckit.clarify  → refine spec  # Ask clarifying questions
/speckit.plan     → plan.md      # Design architecture
/speckit.tasks    → tasks.md     # Generate task breakdown
/speckit.analyze  → validation   # Check consistency
/speckit.checklist → checklist   # QA checklist
/speckit.implement → code+tests  # Implement feature
/speckit.review   → PR review    # Verify against spec
```

### Quick Status

```bash
/spec-status      # Full status
/spec-check       # Quick check
```

## Common Commands

```bash
# Build & Test
cargo build && cargo test
cargo clippy --all-targets --all-features

# GitHub CLI
gh issue create --title "Feature: {name}" --body-file spec.md
gh pr create --title "{description}" --body "Closes #{issue}"
```

## Commit Format

```bash
git commit -m "feat(NNN): description"
```

## Technologies

- Rust 1.75+ (edition 2021)
- reedline 0.26+ (line editing)
- tokio, serde, anyhow/thiserror, tracing

## Test Coverage

- 670+ passing tests
- All tests complete in <1 second

## Active Technologies
- Rust 1.75+ (edition 2021) + No new dependencies (pure Rust implementation) (029-arithmetic-expansion)
- N/A (uses existing VariableManager) (029-arithmetic-expansion)
- Rust 1.75+ (edition 2021) + reedline (already in project), std::io for terminal I/O (030-read-builtin)
- N/A (variables stored in existing VariableManager) (030-read-builtin)
- Rust 1.75+ (edition 2021) + None (pure Rust implementation) (034-brace-expansion)
- Rust 1.75+ (edition 2021) + None (pure Rust std library) (036-set-builtin)
- In-memory (ShellOptions struct in CommandExecutor) (036-set-builtin)
- Rust 1.75+ (edition 2021) + nix 0.29 (signal handling), existing in Cargo.toml (037-trap-builtin)
- In-memory HashMap in CommandExecutor (trap registry persists for shell session lifetime) (037-trap-builtin)
- Rust 1.75+ (edition 2021) + regex 1.10 (for `=~` operator and pattern matching) (038-test-command)
- N/A (stateless command execution, uses existing VariableManager for BASH_REMATCH) (038-test-command)

## Recent Changes
- 029-arithmetic-expansion: Added Rust 1.75+ (edition 2021) + No new dependencies (pure Rust implementation)
