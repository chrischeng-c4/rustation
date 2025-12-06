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

## Development Workflow (Mandatory Sequential Execution)

> **Execution Rules:**
> - Execute ALL steps in order. Never skip, never reorder, never parallelize.
> - After each step, output: `✅ Step {n}: {description} — {output/result}`
> - If a step fails, STOP and report the failure. Do not continue.
> - Phase is complete ONLY when all outputs are verified.

---

## Master Checklist

Print this at workflow start. Update after each step:
```
PHASE 1: Specify & Plan
⬜ 1.1 /speckit.specify → spec.md
⬜ 1.2 /speckit.clarify → spec.md refined
⬜ 1.3 /speckit.plan → plan.md
⬜ 1.4 gh issue create → issue #___
⬜ 1.5 gh issue comment → plan attached
⬜ 1.6 Phase 1 verification passed

PHASE 2: Tasks → Sub-issues
⬜ 2.1 /speckit.tasks → tasks.md
⬜ 2.2 gh issue create (per task) → sub-issues #___, #___, ...
⬜ 2.3 Phase 2 verification passed

PHASE 3: Validate
⬜ 3.1 /speckit.analyze → analysis output
⬜ 3.2 /speckit.checklist → checklist.md
⬜ 3.3 Fix gaps (if any)
⬜ 3.4 Phase 3 verification passed

PHASE 4: Implement (per sub-issue)
⬜ 4.x.1 git checkout -b {branch}
⬜ 4.x.2 /speckit.implement → code + tests
⬜ 4.x.3 tests pass
⬜ 4.x.4 docs updated
⬜ 4.x.5 PR created → PR #___
⬜ 4.x.6 Sub-issue #{n} verification passed

PHASE 5: Review (per PR)
⬜ 5.x.1 /speckit.review
⬜ 5.x.2 PR merged
⬜ 5.x.3 Sub-issue closed

PHASE 6: Complete
⬜ 6.1 All sub-issues closed
⬜ 6.2 Main issue closed
⬜ 6.3 Final verification passed
```

---

## Phase 1: Specify & Plan

| Step | Command | Required Output | Verification |
|------|---------|-----------------|--------------|
| 1.1 | `/speckit.specify` | `.specify/features/{feature}/spec.md` | File exists, non-empty |
| 1.2 | `/speckit.clarify` | spec.md updated | Diff shown if changes made |
| 1.3 | `/speckit.plan` | `.specify/features/{feature}/plan.md` | File exists, non-empty |
| 1.4 | `gh issue create --title "Feature: {name}" --body-file .specify/features/{feature}/spec.md` | GitHub issue created | Issue number captured |
| 1.5 | `gh issue comment {number} --body "## Plan$(echo '\n\n')$(cat .specify/features/{feature}/plan.md)"` | Plan attached to issue | Comment visible |

**Phase 1 Verification (run before proceeding):**
```bash
# All must succeed:
test -s .specify/features/{feature}/spec.md && echo "✅ spec.md exists"
test -s .specify/features/{feature}/plan.md && echo "✅ plan.md exists"
gh issue view {number} --json title,body,comments --jq '.title' && echo "✅ issue exists"
```

**Output after Phase 1:**
```
✅ Phase 1 Complete
- spec.md: .specify/features/{feature}/spec.md
- plan.md: .specify/features/{feature}/plan.md
- Main Issue: #{number}
Proceeding to Phase 2...
```

---

## Phase 2: Tasks → Sub-issues

| Step | Command | Required Output | Verification |
|------|---------|-----------------|--------------|
| 2.1 | `/speckit.tasks` | `.specify/features/{feature}/tasks.md` | File exists with task list |
| 2.2 | For EACH task in tasks.md: `gh issue create --title "US{n}: {description}" --body "Parent: #{main-issue}"` | Sub-issue per task | All sub-issue numbers captured |

**Phase 2 Verification:**
```bash
test -s .specify/features/{feature}/tasks.md && echo "✅ tasks.md exists"
# List all sub-issues:
gh issue list --search "Parent: #{main-issue}" --json number,title
```

**Output after Phase 2:**
```
✅ Phase 2 Complete
- tasks.md: .specify/features/{feature}/tasks.md
- Sub-issues created: #101, #102, #103, ...
Proceeding to Phase 3...
```

---

## Phase 3: Validate

| Step | Command | Required Output | Verification |
|------|---------|-----------------|--------------|
| 3.1 | `/speckit.analyze` | Analysis report (stdout or file) | No critical gaps found |
| 3.2 | `/speckit.checklist` | `.specify/features/{feature}/checklist.md` | File exists |
| 3.3 | Review checklist, fix any gaps | Updated artifacts | All checklist items passing |

**Phase 3 Verification:**
```bash
test -s .specify/features/{feature}/checklist.md && echo "✅ checklist.md exists"
# Manual check: Are there blocking issues in analyze output?
```

**If gaps found:** Fix them, re-run `/speckit.analyze`, verify clean.

**Output after Phase 3:**
```
✅ Phase 3 Complete
- Analysis: clean (no blocking gaps)
- Checklist: .specify/features/{feature}/checklist.md
- Ready for implementation
Proceeding to Phase 4...
```

---

## Phase 4: Implement (Repeat for EACH Sub-issue)

**⚠️ Execute sequentially. Complete one sub-issue before starting next.**

For sub-issue `#{sub}`:

| Step | Command | Required Output | Verification |
|------|---------|-----------------|--------------|
| 4.x.1 | `git checkout -b {sub}-{feature-name}` | Branch created | `git branch --show-current` confirms |
| 4.x.2 | `/speckit.implement` | Code + test files | New/modified files exist |
| 4.x.3 | Run test suite | Tests pass | Exit code 0 |
| 4.x.4 | Update `docs/` if needed | Docs updated | `git status` shows docs changes (if applicable) |
| 4.x.5 | `git add . && git commit -m "feat(#{sub}): {description}"` | Commit created | Commit hash captured |
| 4.x.6 | `git push -u origin {branch}` | Branch pushed | Remote branch exists |
| 4.x.7 | `gh pr create --title "{description}" --body "Closes #{sub}"` | PR created | PR number captured |

**Phase 4 Per-Issue Verification:**
```bash
git log -1 --oneline  # Commit exists
gh pr view --json number,title,state  # PR exists and open
```

**Output after each sub-issue:**
```
✅ Sub-issue #{sub} Implementation Complete
- Branch: {sub}-{feature-name}
- Commit: {hash}
- PR: #{pr-number}
Moving to next sub-issue... (or Phase 5 if last)
```

---

## Phase 5: Review (Repeat for EACH PR)

| Step | Command | Required Output | Verification |
|------|---------|-----------------|--------------|
| 5.x.1 | `/speckit.review` | Review report | Requirements matched |
| 5.x.2 | Verify tests cover acceptance criteria | Coverage check | All criteria have tests |
| 5.x.3 | `gh pr merge {pr-number} --squash` | PR merged | PR state = merged |
| 5.x.4 | `gh issue close {sub-issue}` | Sub-issue closed | Issue state = closed |

**Phase 5 Per-PR Verification:**
```bash
gh pr view {pr-number} --json state --jq '.state'  # Must be "MERGED"
gh issue view {sub-issue} --json state --jq '.state'  # Must be "CLOSED"
```

**Output after each PR:**
```
✅ PR #{pr-number} Review Complete
- Review: passed
- PR: merged
- Sub-issue #{sub}: closed
Moving to next PR... (or Phase 6 if last)
```

---

## Phase 6: Complete

| Step | Command | Required Output | Verification |
|------|---------|-----------------|--------------|
| 6.1 | Verify all sub-issues closed | All closed | Query returns 0 open |
| 6.2 | `gh issue close {main-issue}` | Main issue closed | Issue state = closed |
| 6.3 | `git checkout main && git pull` | Local main updated | Up to date with remote |

**Phase 6 Verification:**
```bash
# All sub-issues must be closed:
gh issue list --search "Parent: #{main-issue}" --state open --json number | jq 'length'  # Must be 0

# Main issue closed:
gh issue view {main-issue} --json state --jq '.state'  # Must be "CLOSED"
```

**Final Output:**
```
✅ WORKFLOW COMPLETE

Feature: {feature-name}
Main Issue: #{main-issue} (CLOSED)
Sub-issues: #{sub1}, #{sub2}, #{sub3} (ALL CLOSED)
PRs Merged: #{pr1}, #{pr2}, #{pr3}

All artifacts:
- .specify/features/{feature}/spec.md
- .specify/features/{feature}/plan.md
- .specify/features/{feature}/tasks.md
- .specify/features/{feature}/checklist.md
```

---

## Anti-Skip Enforcement
```markdown
### 🚫 Forbidden Actions

- Starting Phase N+1 before Phase N verification passes
- Creating PR before tests pass
- Closing issue before PR is merged
- Working on multiple sub-issues simultaneously
- Skipping `/speckit.*` commands
- Proceeding if any verification step fails

### Recovery Protocol

If a step fails:
1. STOP immediately
2. Report: "❌ Step {n} failed: {reason}"
3. Do NOT attempt to continue or skip
4. Wait for user guidance
```

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

### Core Infrastructure
- Rust 1.75+ (edition 2021) (001-rush-mvp)
- reedline 0.26+ (002-tab-completion, 003-autosuggestions)
- File-based history (~/.config/rush/history)

### Control Flow Features (017-026)
**Feature 017 - Conditionals (if/then/elif/else/fi)**
  - Recursive descent parser with keyword detection
  - Multiline REPL support with continuation prompts
  - Short-circuit evaluation
  - 22 integration tests

**Features 018-019 - Loops (for/while/until)**
  - Phase 2: Variable expansion ($VAR, ${VAR}), command substitution $(cmd), globbing (*, ?, [...])
  - Phase 3: Pipe support in loop bodies (for | while | until)
  - 148 integration tests (expansions, command substitution, globbing, pipes)
  - Architectural pattern: raw_body field for pipe/redirection support

**Feature 020+ - Case statements, functions, break/continue/return**
  - Phase 2: Variable expansion, command substitution, globbing
  - 20 integration tests

### Test Coverage
- Total: 752 passing tests (532 lib + 158 integration + 59 other + 3 doc)
- Phase 2 (complete): 93 expansion/substitution/globbing tests
- Phase 3 (partial): 10 pipe tests (5 for loops, 5 while/until loops)
