# Implementation Plan: Enhanced Worktree View with Tabs and Comprehensive Logging

**Branch**: `049-enhanced-worktree-view` | **Date**: 2025-12-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/049-enhanced-worktree-view/spec.md`

## Summary

This feature enhances the rush shell's TUI Worktree view with four key improvements: (1) visual tab navigation for switching between spec.md, plan.md, and tasks.md, (2) comprehensive timestamped activity logging for all commands, Claude output, file changes, and shell scripts with color-coded categories, (3) automatic file change detection with 1-2 second responsiveness when spec files are edited externally, and (4) a 1000-line rolling log buffer for memory-efficient long sessions.

**Technical Approach**: Implement a new `logging/` module with type-safe log entries and a circular buffer using `VecDeque`. Add ratatui's `Tabs` widget to the Content panel. Use polling-based file watching (checking modification times every 1 second) to detect external file changes without adding new dependencies. All features integrate cleanly into the existing TUI architecture with minimal disruption.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**:
- ratatui 0.29 (TUI framework - already in project)
- crossterm 0.28 (terminal I/O - already in project)
- tokio (async runtime - already in project)
- chrono (timestamp formatting - already in project)

**Storage**: In-memory only
- `LogBuffer`: VecDeque<LogEntry> with 1000-entry circular buffer
- `FileChangeTracker`: HashMap<PathBuf, SystemTime> for modification time tracking
- No persistent storage required (log entries ephemeral per TUI session)

**Testing**:
- cargo test (unit tests for LogBuffer, FileChangeTracker, tab navigation logic)
- Manual TUI testing (visual verification of tabs, colors, timestamps, file detection)
- Integration tests (verify event flow from command execution to log display)

**Target Platform**: macOS (MVP), Linux (post-MVP), Windows via WSL (future)

**Project Type**: Single project (enhancement to existing `crates/rstn/src/tui/` TUI module)

**Performance Goals**:
- Tab switching: <0.5 seconds (<500ms)
- Log entry creation: <100ms from event to display
- File change detection: <2 seconds from file save to content reload
- TUI responsiveness: >30 FPS even with full 1000-line log buffer
- Memory overhead: <2MB for full log buffer (1000 entries × ~2KB each)

**Constraints**:
- CPU: File polling overhead <1% (3 × fs::metadata() per second)
- Memory: Baseline <10MB (per constitution), log buffer adds <2MB max
- No new dependencies: Use only existing crates in Cargo.toml
- No blocking I/O: All file operations remain non-blocking
- Backward compatibility: Existing 's' key for content switching must continue working

**Scale/Scope**:
- Monitor 3 files (spec.md, plan.md, tasks.md)
- Support 1000 log entries in circular buffer
- Handle >2000 total log entries per session (with automatic eviction)
- Display 4 user stories across ~5-6 source files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status | Justification |
|-----------|-------------|--------|---------------|
| **I. Performance-First** | <100ms startup, <16ms responsiveness, <5ms overhead, <10MB memory | ✅ PASS | • File polling adds <1% CPU (3 stat calls/sec is negligible)<br>• VecDeque circular buffer is O(1) for push/pop<br>• Tab rendering adds ~5ms (well under 16ms frame budget)<br>• Log buffer adds <2MB memory (within 10MB baseline)<br>• All operations non-blocking (file checks in tick(), not main thread) |
| **II. Zero-Config** | Sensible defaults, works immediately | ✅ PASS | • Tabs appear automatically when feature detected<br>• Logging starts immediately, no setup required<br>• File watching activates automatically on feature branches<br>• 1000-line buffer limit is invisible, automatic<br>• No configuration files, settings, or user actions needed |
| **III. Progressive Complexity** | Simple by default, powerful when needed | ✅ PASS | • Basic tab navigation works with arrow keys (simple)<br>• Existing 's' key still works (no forced change)<br>• Log categories auto-color-coded (no manual setup)<br>• File watching is automatic (user doesn't configure polling)<br>• No advanced modes, options, or toggles—complexity hidden |
| **IV. Modern UX** | Visual feedback, syntax highlighting, accessibility | ✅ PASS | • Tabs provide clear visual navigation (yellow highlight)<br>• Color-coded log categories (cyan/green/yellow/white)<br>• Emoji icons (⚡🤖📝🔧) for quick visual scanning<br>• Timestamps for all entries (HH:MM:SS format)<br>• Auto-scroll to bottom, responsive scrolling<br>• Colors respect terminal capabilities (ratatui handles fallbacks) |
| **V. Rust-Native** | Pure Rust, ecosystem integration, idiomatic | ✅ PASS | • Zero new dependencies (uses existing ratatui, std::collections)<br>• VecDeque from std::collections (idiomatic Rust)<br>• fs::metadata() for file checking (std::fs, pure Rust)<br>• Follows existing TUI patterns (Event enum, view rendering)<br>• No unsafe code, no FFI, no external binaries |

**Overall Assessment**: ✅ **ALL GATES PASS** - Feature fully aligns with constitution principles

**Key Alignment Points**:
- **Performance-First**: Efficient data structures, minimal overhead, non-blocking operations
- **Zero-Config**: Works immediately without any user configuration
- **Progressive Complexity**: Simple tab navigation with hidden implementation complexity
- **Modern UX**: Visual tabs, color coding, icons, timestamps enhance developer experience
- **Rust-Native**: Pure Rust implementation using standard library and existing dependencies

**Potential Concerns & Mitigations**:
- ❓ Polling vs real-time file watching → Polling chosen to avoid new dependencies; 1Hz rate is responsive enough for typical editing workflows
- ❓ Memory growth with logs → 1000-line circular buffer prevents unbounded growth
- ❓ Log performance with many entries → VecDeque is O(1) for all operations; rendering only visible lines

**Post-Phase-1 Re-check**: Will verify that data models and contracts maintain these guarantees.

## Project Structure

### Documentation (this feature)

```text
specs/049-enhanced-worktree-view/
├── spec.md              # Feature requirements (already created)
├── plan.md              # This file (/speckit.plan output)
├── research.md          # Phase 0: Research findings
├── data-model.md        # Phase 1: Entity definitions
├── quickstart.md        # Phase 1: Integration testing scenarios
└── checklists/
    └── requirements.md  # Spec quality validation (already created)
```

### Source Code (repository root)

```text
crates/rstn/src/tui/
├── logging/                    # NEW: Logging infrastructure module
│   ├── mod.rs                  # Module exports
│   ├── entry.rs                # LogEntry, LogCategory types
│   ├── buffer.rs               # LogBuffer (VecDeque circular buffer)
│   └── file_tracker.rs         # FileChangeTracker (polling-based)
│
├── views/
│   └── worktree.rs             # MODIFIED: Add tabs, integrate logging
│
├── widgets/
│   └── mod.rs                  # MODIFIED: Existing widgets (no new widgets needed)
│
├── event.rs                    # MODIFIED: Add FileChanged event
├── app.rs                      # MODIFIED: Handle logging in event handlers
└── mod.rs                      # MODIFIED: Export logging module

tests/
└── tui/                        # NEW: TUI-specific tests
    ├── logging_tests.rs        # LogBuffer, FileChangeTracker tests
    └── worktree_tests.rs       # Tab navigation, integration tests
```

**Structure Decision**: **Single project** (Option 1)

This is an enhancement to the existing TUI within the `crates/rstn` crate. The feature adds a new `logging/` module and modifies existing `views/worktree.rs`, `event.rs`, and `app.rs` files. No new crates, no backend/frontend split, no separate mobile components—purely a TUI enhancement within the existing monorepo structure.

**File Modification Summary**:
- **New directory**: `crates/rstn/src/tui/logging/` (4 files)
- **Modified files**: `worktree.rs`, `event.rs`, `app.rs`, `tui/mod.rs` (4 files)
- **New test files**: `tests/tui/logging_tests.rs`, `tests/tui/worktree_tests.rs` (2 files)
- **Total**: ~10 files touched, ~1,200-1,500 lines of new code

## Complexity Tracking

> **No violations - table left empty per template instructions**

This feature introduces zero violations of constitution principles:
- No new dependencies → Rust-Native ✅
- No required configuration → Zero-Config ✅
- No performance regressions → Performance-First ✅
- No forced complexity → Progressive Complexity ✅
- Enhances UX with visual feedback → Modern UX ✅

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**CRITICAL: Keep PRs small and reviewable (see CLAUDE.md for limits).**

**Strategy**: **Option 2: PR per User Story** (RECOMMENDED for multi-story features)

This feature has 4 user stories prioritized P1-P3 with estimated ~1,200-1,500 total lines. Breaking by user story ensures each PR delivers standalone value and stays within the 1,500-line maximum.

```
PR #1: Foundation - Logging Infrastructure
  - Create logging/ module (entry.rs, buffer.rs, file_tracker.rs, mod.rs)
  - Unit tests for LogBuffer and FileChangeTracker
  - Integration into tui/mod.rs
  - Target: ~400 lines (module structure + tests)
  - Value: Logging types available for integration

PR #2: User Story 1 (P1) - Tab Navigation
  - Modify worktree.rs: add Tabs widget to render_content()
  - Add tab switching keybindings (Left/Right arrows)
  - Maintain backward compatibility ('s' key still works)
  - Unit tests for tab switching logic
  - Target: ~300 lines (UI changes + tests)
  - Value: Users can navigate spec/plan/tasks with tabs

PR #3: User Story 2 (P1) - Comprehensive Logging
  - Modify worktree.rs: integrate LogBuffer, replace Vec<String>
  - Modify app.rs: log slash commands, Claude streams, shell commands
  - Modify event.rs: add FileChanged, SlashCommandExecuted events
  - Update render_output() to display formatted log entries
  - Integration tests for logging workflow
  - Target: ~500 lines (event wiring + rendering + tests)
  - Value: All activity appears in timestamped, color-coded log

PR #4: User Story 3 (P2) - File Change Detection
  - Modify worktree.rs: add file_tracker field, implement check_file_changes()
  - Add file checking to tick() method
  - Log file change events
  - Manual testing checklist (edit files externally, verify detection)
  - Target: ~200 lines (polling logic + tests)
  - Value: TUI auto-reloads when spec files change externally

PR #5: User Story 4 (P3) + Polish
  - Verify 1000-line buffer limit (already in LogBuffer from PR #1)
  - Performance testing (generate >1000 lines, verify no lag)
  - Documentation updates (CLAUDE.md if needed)
  - Final integration testing across all user stories
  - Target: ~100 lines (tests + docs)
  - Value: Production-ready feature with validated performance
```

### Selected Strategy

**Option 2: PR per User Story**

**Rationale**:
- 4 user stories × ~200-500 lines each ≈ 1,200-1,500 lines total
- Each PR delivers independently testable value
- PR #1 (foundation) enables all subsequent PRs
- PR #2-#4 implement P1-P2 user stories in priority order
- PR #5 validates P3 and adds polish
- All PRs stay well under 1,500-line maximum

### Merge Sequence

1. **PR #1: Foundation** → Merge to main
   - Creates `logging/` module infrastructure
   - No user-visible changes (internal types only)
   - Enables all subsequent PRs

2. **PR #2: Tab Navigation (US1)** → Merge to main
   - User-visible: Tabs appear in Content panel
   - Independent: Works without logging enhancements
   - Validates ratatui Tabs widget integration

3. **PR #3: Comprehensive Logging (US2)** → Merge to main
   - User-visible: Timestamped, color-coded log entries
   - Depends on: PR #1 (logging module)
   - Independent: Works without tabs or file watching

4. **PR #4: File Change Detection (US3)** → Merge to main
   - User-visible: Auto-reload on external file edits
   - Depends on: PR #1 (logging module for file change events)
   - Independent: Works without tabs or comprehensive logging

5. **PR #5: Polish & Validation (US4)** → Merge to main
   - User-visible: Validated 1000-line buffer, performance benchmarks
   - Depends on: All prior PRs
   - Verifies: End-to-end integration

**Branch Strategy**:
- Base branch: `049-enhanced-worktree-view` (already created)
- PR branches: `049-foundation`, `049-tabs`, `049-logging`, `049-file-watch`, `049-polish`
- Each PR branch created from `049-enhanced-worktree-view`
- After merge to main, sync `049-enhanced-worktree-view` with main before next PR

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

**Estimated PR Sizes**:
- PR #1: ~400 lines (✅ Ideal range)
- PR #2: ~300 lines (✅ Ideal range)
- PR #3: ~500 lines (✅ Ideal range)
- PR #4: ~200 lines (✅ Ideal range)
- PR #5: ~100 lines (✅ Ideal range)

**Total**: ~1,500 lines across 5 PRs, all within acceptable limits.

If any PR exceeds limits during implementation, split into smaller increments (e.g., separate UI changes from event handling).
