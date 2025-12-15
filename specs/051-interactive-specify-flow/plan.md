# Implementation Plan: Interactive Specify Flow

**Branch**: `051-interactive-specify-flow` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/051-interactive-specify-flow/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Transform the `/speckit.specify` workflow from a shell-out process to an integrated TUI experience using the drop dialog pattern established in feature 050. Users input feature descriptions directly in the Content area, review generated specs before saving, and can edit inline using keyboard-first interactions. This maintains the existing `create-new-feature.sh` script integration while providing a seamless, context-switching-free workflow.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**:
- ratatui 0.29+ (TUI framework - already in project)
- crossterm 0.28 (terminal I/O - already in project)
- tokio (async runtime - already in project)
- anyhow/thiserror (error handling - already in project)

**Storage**: In-memory state during specify workflow; final spec written to `specs/{NNN}-{name}/spec.md`
**Testing**: cargo test (unit tests for state management, integration tests for workflow)
**Target Platform**: macOS (MVP), Linux post-MVP
**Project Type**: Single project (monorepo with crates/rstn TUI application)
**Performance Goals**:
- Input dialog render: <50ms
- Mode transitions (input→review→edit): <50ms
- Key input responsiveness: <16ms (60 FPS)
**Constraints**:
- Must maintain existing shell script integration (no changes to create-new-feature.sh)
- Pattern must match feature 050 (Commit Review) for UX consistency
- Keyboard-first interaction (all operations via keyboard)
**Scale/Scope**: Single feature touching 4-5 files in crates/rstn/src/tui/

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Performance-First ✅ PASS

- **Fast startup**: ✅ No impact on shell startup (feature only active when triggered)
- **Instant responsiveness**: ✅ Target <50ms transitions, <16ms input response
- **Minimal overhead**: ✅ In-memory state only, no persistent background processes
- **Memory efficiency**: ✅ Temporary state during workflow, cleaned up on completion
- **No blocking operations**: ✅ Shell script execution via tokio::process (async)

**Assessment**: Feature maintains performance-first principle through async operations and instant UI transitions.

### Principle II: Zero-Config Philosophy ✅ PASS

- **Sensible defaults**: ✅ Works immediately, no configuration required
- **No mandatory setup**: ✅ Feature available as soon as installed
- **Configuration optional**: ✅ No configuration file needed

**Assessment**: Feature requires zero configuration and works out of the box.

### Principle III: Progressive Complexity ✅ PASS

- **Layered functionality**: ✅ Basic flow (input→generate→save) simple; edit mode is optional enhancement
- **No forced complexity**: ✅ Users can skip edit mode, go straight to save
- **Escape hatches**: ✅ Can cancel at any stage (Esc key)

**Assessment**: Feature provides simple default path with optional advanced editing capability.

### Principle IV: Modern UX ✅ PASS

- **Visual feedback**: ✅ Clear status updates, action hints, mode indicators
- **Smart interactions**: ✅ Keyboard shortcuts consistent with TUI patterns
- **Delightful experience**: ✅ Seamless flow without context switching

**Assessment**: Feature enhances UX by eliminating context switching and providing smooth workflow.

### Principle V: Rust-Native ✅ PASS

- **Pure Rust**: ✅ All new code in Rust, uses existing dependencies
- **Ecosystem integration**: ✅ Leverages ratatui, crossterm, tokio already in project
- **Idiomatic code**: ✅ Follows existing codebase patterns from feature 050

**Assessment**: Feature is pure Rust using existing ecosystem dependencies.

**Constitution Compliance**: ✅ **ALL GATES PASS** - No violations, no justifications needed.

## Project Structure

### Documentation (this feature)

```text
specs/051-interactive-specify-flow/
├── spec.md              # Feature specification (already exists)
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (not needed - no unknowns)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (not applicable - internal API)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created yet)
```

### Source Code (repository root)

```text
crates/rstn/src/tui/
├── views/
│   ├── worktree.rs           # MODIFY: Add specify state and methods
│   └── mod.rs                # MODIFY: Add ContentType::SpecifyInput, SpecifyReview
├── app.rs                    # MODIFY: Handle specify events and actions
├── event.rs                  # MODIFY: Add specify events
└── actions.rs                # MODIFY: Add specify actions

tests/
├── unit/
│   └── tui_specify_tests.rs  # NEW: Unit tests for specify state management
└── integration/
    └── specify_workflow_test.rs  # NEW: Integration tests for full workflow
```

**Structure Decision**: Single project structure. This feature extends the existing TUI application in `crates/rstn` without requiring new crates or services. All changes are localized to the TUI layer with clear integration points.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

N/A - All constitution checks pass. No complexity violations to justify.

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**CRITICAL: Keep PRs small and reviewable (see CLAUDE.md for limits).**

**Strategy**: PR per User Story (Option 2)

### Selected Strategy

Using **Option 2: PR per User Story** - Feature has 3 well-defined user stories that can be implemented and tested independently.

**Rationale**:
- Feature spec defines 3 prioritized user stories (P1, P2, P3)
- Each story is independently testable per the spec
- Estimated ~400-600 lines per story, well within 1,500 line limit
- Allows incremental value delivery and easier code review

### Merge Sequence

1. **PR #1: Foundation + User Story 1 (Stay in TUI)** → Merge to main
   - Add ContentType::SpecifyInput enum variant
   - Add specify state fields to WorktreeView
   - Implement start_specify_input() and input handling
   - Add async shell script execution
   - Add basic rendering for input dialog
   - Tests: Input mode entry, text input, submit, cancel
   - **Target**: ~600 lines (foundation + P1 story)
   - **Deliverable**: Users can input descriptions and trigger generation without leaving TUI

2. **PR #2: User Story 2 (Review generated specs)** → Merge to main
   - Add ContentType::SpecifyReview enum variant
   - Implement load_generated_spec() and review display
   - Add save_specify_spec() method
   - Implement review mode rendering with action hints
   - Tests: Review mode display, save workflow, cancel workflow
   - **Target**: ~400 lines
   - **Deliverable**: Users can review and save generated specs

3. **PR #3: User Story 3 (Inline editing)** → Merge to main
   - Add specify_edit_mode flag and cursor management
   - Implement toggle_specify_edit_mode() method
   - Add edit mode input handling (arrows, Home, End, Ctrl+S)
   - Implement edit mode rendering with visual indicator
   - Tests: Edit mode toggle, editing operations, save from edit, cancel edits
   - **Target**: ~500 lines
   - **Deliverable**: Users can edit specs inline before saving

4. **PR #4: Polish & Error Handling** → Merge to main
   - Enhance error messages and validation
   - Add comprehensive status bar updates
   - Improve visual feedback and transitions
   - Add timeout handling for generation
   - Complete integration tests for full workflow
   - Update documentation
   - **Target**: ~300 lines
   - **Deliverable**: Production-ready with robust error handling

**Branch Strategy**:
- Base branch: `051-interactive-specify-flow` (already created)
- Feature PRs merge directly to base branch
- Final PR merges base branch to main after all stories complete

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

## Phase 0: Research (SKIPPED - No Unknowns)

**Assessment**: All technical decisions are clear from the spec and existing codebase:

- **UI Framework**: ratatui 0.29+ (already in use)
- **Async Runtime**: tokio (already in use)
- **Pattern Reference**: Feature 050 (Commit Review) provides complete pattern
- **Shell Integration**: Existing create-new-feature.sh script (no changes needed)
- **State Management**: WorktreeView pattern already established

**No research.md needed** - All dependencies and patterns are already in the project. Feature 050 provides the exact architectural pattern to follow.

## Phase 1: Design Artifacts

See separate files in this directory:
- `data-model.md` - State structures and types
- `quickstart.md` - Developer onboarding for this feature
- No contracts/ needed - Internal TUI feature, no external API

## Next Steps

After planning complete:
1. Run `/speckit.tasks` to generate task breakdown from this plan
2. Run `/speckit.implement` to execute tasks and build the feature
3. Create PRs following the merge sequence above
