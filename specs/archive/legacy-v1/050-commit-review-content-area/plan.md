# Implementation Plan: Commit Review in Content Area

**Branch**: `050-commit-review-content-area` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/050-commit-review-content-area/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Replace the modal dialog for intelligent commit workflow with a dynamic Content area view in the Worktree TUI. This eliminates truncated file lists, enables easy copying, provides inline message editing, and delivers better UX without modal interruptions. Users can review commit groups, edit messages, navigate between groups, and submit commits all within the Content pane using keyboard shortcuts.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: ratatui 0.29+ (TUI framework), crossterm 0.28 (terminal I/O), arboard 3.4 (clipboard), rstn-core (git operations), tokio (async runtime)
**Storage**: In-memory state only (commit review session data lives in WorktreeView struct)
**Testing**: cargo test (unit tests for state management, integration tests for workflow)
**Target Platform**: macOS (MVP), Linux/WSL (future)
**Project Type**: Single Rust workspace monorepo (crates/rstn TUI application)
**Performance Goals**: <50ms rendering for commit review, instant navigation between groups, <16ms input response (60 FPS)
**Constraints**: Support up to 50 files per commit group, preserve completed commits on error, non-blocking async git operations
**Scale/Scope**: 1-20 commit groups per workflow, 1-50 files per group, session-scoped state (no persistence)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Performance-First ✅ PASS

- **Rendering performance**: <50ms target aligns with "instant responsiveness" requirement
- **Input response**: <16ms (60 FPS) meets "instant responsiveness within 16ms" requirement
- **No blocking**: Async git operations via tokio prevent blocking the UI
- **Memory efficiency**: In-memory session state is minimal (50 files × ~100 bytes paths = ~5KB per group)
- **Benchmarks**: Will include rendering benchmarks in tests

**Verdict**: Compliant. No performance-first violations.

### Principle II: Zero-Config Philosophy ✅ PASS

- **Sensible defaults**: Commit review mode activates automatically when triggered
- **No setup required**: Works immediately with existing intelligent commit infrastructure
- **Configuration optional**: No RC files or setup needed
- **Progressive disclosure**: Advanced features (n/p navigation) shown in UI hints

**Verdict**: Compliant. Feature works with zero configuration.

### Principle III: Progressive Complexity ✅ PASS

- **Simple by default**: Basic workflow is Enter to submit, Esc to cancel
- **Advanced features opt-in**: Navigation (n/p) and copying (y) are discoverable but optional
- **No forced complexity**: Users can use simple Enter/Esc workflow without learning shortcuts
- **Discoverability**: Keyboard hints shown in UI

**Verdict**: Compliant. Complexity is layered appropriately.

### Principle IV: Modern UX ✅ PASS

- **Visual feedback**: Group number (1/6), navigation controls, validation errors shown inline
- **Clear indicators**: Distinct commit review mode with dedicated tab
- **Accessible**: Terminal-based, works in any terminal with ratatui support
- **Polished experience**: Inline editing, instant navigation, no modal interruptions

**Verdict**: Compliant. Delivers modern, delightful UX.

### Principle V: Rust-Native ✅ PASS

- **Pure Rust**: ratatui (Rust TUI), crossterm (Rust terminal I/O), arboard (Rust clipboard)
- **Ecosystem integration**: Using mature crates (ratatui, tokio, crossterm)
- **Zero-cost abstractions**: State management via Rust enums and structs
- **Idiomatic**: Follows Rust patterns (Result, Option, async/await)

**Verdict**: Compliant. No FFI, all Rust ecosystem.

### Overall Gate Status: ✅ ALL GATES PASS

No principle violations. No complexity justification required.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/
├── rstn/                          # TUI application (PRIMARY)
│   └── src/
│       ├── tui/
│       │   ├── views/
│       │   │   ├── worktree.rs   # PRIMARY FILE - Add commit review logic
│       │   │   └── mod.rs        # Add SubmitCommitGroup action
│       │   ├── event.rs          # Add commit review events
│       │   ├── app.rs            # Add event handlers
│       │   └── widgets/          # (no changes needed)
│       └── lib.rs
│
├── rstn-core/                     # Core git operations
│   └── src/
│       └── git/
│           └── commit.rs         # Add commit_group() function
│
└── rstn-tui/                      # (no changes needed)

tests/
└── (integration tests to be added in tasks phase)
```

**Structure Decision**: Single Rust workspace monorepo (Option 1). All changes are isolated to existing `crates/rstn` TUI application and `rstn-core` git module. No new crates or major structural changes required.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

**No violations**. All constitution gates passed. No complexity justification required.

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**CRITICAL: Keep PRs small and reviewable (see CLAUDE.md for limits).**

### Selected Strategy: **Single PR** (Option 1)

**Rationale**:
- Feature is focused and self-contained (commit review UI)
- Estimated total: ~800-1,000 lines
  - worktree.rs: ~400 lines (state + methods + rendering + input handling)
  - app.rs: ~100 lines (event handlers)
  - event.rs: ~50 lines (new events)
  - mod.rs: ~20 lines (new action)
  - commit.rs: ~100 lines (git integration)
  - Tests: ~200 lines
- All changes are tightly coupled (splitting would create non-functional intermediate states)
- Single PR allows atomic review of complete workflow

**Size Estimate**: ~1,000 lines (within ≤ 1,500 line maximum)

### Merge Sequence

**PR #1: Complete Commit Review Feature**
- **Branch**: `050-commit-review-content-area` (already created)
- **Target**: Merge to `main`
- **Estimated Size**: ~1,000 lines
- **Contents**:
  - Core Structure: Add `CommitReview` to `ContentType`, extend `WorktreeView` state
  - State Management: Implement 8 public methods (start, next, prev, cancel, validate, etc.)
  - Rendering: Implement `render_commit_review()` with validation errors, warnings
  - Input Handling: Handle all keyboard shortcuts (char input, arrows, n/p, Enter, Esc)
  - Events & Actions: Add 4 events, 1 action, wire up handlers in app.rs
  - Git Integration: Implement `commit_group()` in rstn-core
  - Logging: Add structured logging for key workflow events
  - Tests: Unit tests for state management, integration tests for workflow
  - Documentation: Update CLAUDE.md, add inline code comments

**PR Description Template**:
```markdown
## Feature 050: Commit Review in Content Area

### Summary
Replaces modal dialog for intelligent commit workflow with inline Content area view.

### Changes
- **worktree.rs** (~400 lines): Added commit review state, methods, rendering, input handling
- **app.rs** (~100 lines): Added event handlers for commit workflow
- **event.rs** (~50 lines): Added CommitGroupsReady, CommitGroupCompleted, CommitGroupFailed events
- **mod.rs** (~20 lines): Added SubmitCommitGroup action
- **commit.rs** (~100 lines): Implemented commit_group() function
- **Tests** (~200 lines): Unit + integration tests

### Testing
- [x] All files visible without truncation
- [x] Inline message editing works
- [x] Navigation (n/p) between groups
- [x] Validation blocks empty messages
- [x] Submit and auto-advance
- [x] Error handling preserves completed commits
- [x] Copying works (Tab + y)
- [x] Status bar updates correctly
- [x] Logging to ~/.rustation/logs/rstn.log

### Screenshots
(Add screenshot of commit review UI)

Closes #[issue-number]
```

**Branch Strategy**: Work directly on `050-commit-review-content-area` branch (already created). No sub-branches needed.

### PR Size Validation

**Before creating PR**:
```bash
git diff --stat main
```

**Expected Output**:
```
crates/rstn/src/tui/views/worktree.rs  | 400 ++++++++++++++++++++++++
crates/rstn/src/tui/app.rs            | 100 +++++++
crates/rstn/src/tui/event.rs          |  50 ++++
crates/rstn/src/tui/views/mod.rs      |  20 ++
crates/rstn-core/src/git/commit.rs    | 100 +++++++
tests/commit_review_test.rs           | 200 +++++++++++++
6 files changed, 870 insertions(+), 0 deletions(-)
```

**Size Check**:
- ✅ ~870-1,000 lines (within ≤ 1,500 line maximum)
- ✅ No split required

If actual size exceeds 1,500 lines, split into:
1. PR #1: Core structure + state management (~500 lines)
2. PR #2: Rendering + input handling + events (~500 lines)
3. PR #3: Git integration + tests (~300 lines)

### Pre-Merge Checklist

Before merging PR #1:
- [ ] All unit tests pass (`cargo test --package rstn`)
- [ ] Integration tests pass
- [ ] Manual testing completed (see Testing Checklist in quickstart.md)
- [ ] Performance benchmarks meet targets (<50ms render, <16ms input)
- [ ] Clippy warnings addressed (`cargo clippy --all-targets`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Logs verified in `~/.rustation/logs/rstn.log`
- [ ] CLAUDE.md updated with new tech stack
- [ ] PR description includes screenshots
- [ ] PR linked to issue/spec

### Rollout Plan

1. **Merge PR to main** (once approved)
2. **Tag release**: `git tag -a v0.1.51 -m "feat(050): commit review in content area"`
3. **Update CHANGELOG.md**:
   ```markdown
   ## [0.1.51] - 2025-12-15

   ### Added
   - Commit review in Content area for intelligent commit workflow
   - Inline message editing with validation
   - Navigation between commit groups (n/p keys)
   - Copy commit details (Tab + y)
   - Error recovery preserving completed commits
   ```
4. **Announce in team chat**: "Feature 050 merged: Commit review now in Content area! No more modal dialogs. 🎉"
5. **Monitor logs** for first week: Watch for errors in production use

### Risk Mitigation

**Risk**: Feature breaks existing intelligent commit workflow

**Mitigation**:
- Keep existing `intelligent_commit()` function unchanged
- Only add new commit review UI layer
- Add feature flag if needed (can toggle between modal and content area)

**Risk**: Performance regression on large file lists

**Mitigation**:
- 50-file limit with warnings
- Benchmarks verify <50ms rendering
- Integration tests with 50+ files

**Risk**: Cursor handling bugs with UTF-8

**Mitigation**:
- Extensive unit tests for multibyte characters
- Manual testing with emoji and international text
- Use `is_char_boundary()` for all cursor operations
