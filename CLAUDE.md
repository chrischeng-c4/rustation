# CLAUDE.md

## Language Preference

Respond in English (U.S.) by default. Use Traditional Chinese only when user writes in Traditional Chinese.

## Repository Overview

Rust monorepo workspace containing **rush** - a shell implementation replacing zsh/bash/fish.

```
rust-station/
├── Cargo.toml      # Workspace root
├── crates/rush/    # Shell implementation
└── target/         # Build output (gitignored)
```

## Development Workflow (Mandatory)

**All features follow this spec-kit + GitHub cycle. No exceptions.**

### Phase 1: Specify & Plan

1. `/speckit.specify` → create spec.md
2. `/speckit.clarify` → refine spec
3. `/speckit.plan` → create plan.md
4. **Sync to GitHub**: Create/update issue with spec & plan content
   - Issue body = specification
   - Issue comments = thinking, decisions, clarifications

```bash
gh issue create --title "Feature: {name}" --body-file .specify/features/{feature}/spec.md
gh issue comment {number} --body "## Plan\n$(cat .specify/features/{feature}/plan.md)"
```

### Phase 2: Tasks → Sub-issues

1. `/speckit.tasks` → create tasks.md
2. **Sync to GitHub**: Create sub-issue for each US/task

```bash
gh issue create --title "US1: {description}" --body "Parent: #{main-issue}"
```

### Phase 3: Validate

1. `/speckit.analyze` → cross-artifact consistency & coverage check
2. `/speckit.checklist` → generate quality validation checklist
3. Fix any gaps before implementation

### Phase 4: Implement → PR + Docs

For each sub-issue:

1. **Branch**: `{issue-number}-{feature-name}`
2. `/speckit.implement` → code + tests
3. **PR**: linked to sub-issue
4. **Docs**: add/update `docs/` folder (GitHub Pages)

### Phase 5: Review

1. `/speckit.review` → verify PR matches issue requirements
2. Check code implements spec correctly
3. Validate tests cover acceptance criteria
4. Merge PR & close sub-issue

### Phase 6: Complete

- All sub-issues closed → close main ticket

### Quick Reference

| Phase | Spec-Kit | GitHub |
|-------|----------|--------|
| 1. Specify/Plan | spec.md, plan.md | Issue + comments |
| 2. Tasks | tasks.md | Sub-issues |
| 3. Validate | analyze, checklist | - |
| 4. Implement | code + tests + docs | PR |
| 5. Review | review | PR ↔ Issue check |
| 6. Complete | - | Merge & close |

## Feature Rules

- **One feature = one issue = one branch = one PR** (per sub-issue)
- Keep features small (<1,500 lines per PR)
- Each iteration: **code + tests + docs**
- Check size: `git diff main --stat`

## Commit Format

Use `-m` or `-F`. **No heredocs.**

```bash
git commit -m "feat(045): add heredoc support"
```

## Common Commands

```bash
# Build & Test
cargo build && cargo test
cargo clippy --all-targets --all-features

# GitHub CLI
gh issue create --title "title" --body "body"
gh issue comment {number} --body "comment"
gh pr create --title "title" --body "closes #{issue}"
```

## Technologies

- Rust 1.75+ (edition 2021)
- reedline (line editing)
- tokio, serde, anyhow/thiserror, tracing

## Active Technologies
- Rust 1.75+ (edition 2021) (001-rush-mvp)
- reedline 0.26+ (002-tab-completion, 003-autosuggestions)
- File-based history (~/.config/rush/history)
- Conditional control flow (017-conditionals): if/then/elif/else/fi with nested support
  - Recursive descent parser with keyword detection
  - Multiline REPL support with continuation prompts
  - Short-circuit evaluation
  - 22 passing tests (11 integration, 11 unit)
