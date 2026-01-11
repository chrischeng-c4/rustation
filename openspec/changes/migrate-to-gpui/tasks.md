# Migration Tasks - GPUI

## Status Overview

**Overall Progress**: 83% (5/6 phases complete)
**Current Phase**: Phase 6 - Backend Integration & Polish (6%)
**Last Updated**: 2026-01-12

---

## Phase 1: Foundation & Cleanup ✅ COMPLETE

- [x] 1.1 Create new Rust crate `crates/rstn/` in workspace
- [x] 1.2 Remove Electron+React stack (`desktop/`, `packages/`)
- [x] 1.3 Configure `crates/rstn-core/` for pure Rust (remove napi-rs)
- [x] 1.4 Initialize GPUI application entry point (`main.rs`)
- [x] 1.5 Create basic window and app structure

**Completion**: 2026-01-11 (Commit: 69c5134)

---

## Phase 2: OpenSpec Updates ✅ COMPLETE

- [x] 2.1 Update `openspec/specs/shared-ui/spec.md` for GPUI
- [x] 2.2 Update `openspec/specs/terminal-pty/spec.md` for native rendering
- [x] 2.3 Remove framework-specific implementation details

**Completion**: 2026-01-11 (Commit: f43d09c)

---

## Phase 3: UI Foundation ✅ COMPLETE

- [x] 3.1 Create `crates/rstn-ui/` component library
- [x] 3.2 Implement Material Design 3 theme system
- [x] 3.3 Implement core components:
  - [x] Sidebar (navigation with pill indicators)
  - [x] ShellLayout (header + sidebar + content + status bar)
  - [x] PageHeader (titles, descriptions, action buttons)
  - [x] EmptyState (placeholder for empty data)
- [x] 3.4 Integrate components into main app
- [x] 3.5 Create navigation matching old Electron UI (8 tabs)

**Completion**: 2026-01-11 (Commit: be0a3d5)

---

## Phase 4: Core Feature Views ✅ COMPLETE

- [x] 4.0 Create `crates/rstn-views/` feature views library
- [x] 4.1 **TasksView**: Justfile command runner
  - [x] TaskCard component (state indicators)
  - [x] LogPanel (command output)
  - [x] 50/50 split layout
- [x] 4.2 **DockersView**: Container management
  - [x] ServiceCard component (status badges)
  - [x] Service grouping by project
  - [x] Status color coding
- [x] 4.3 **ExplorerView**: File browser
  - [x] TreeNode component (file tree)
  - [x] FileEntry component (file list)
  - [x] Git status integration
  - [x] 3-panel layout (tree, files, details)
- [x] 4.4 **TerminalView**: PTY terminal
  - [x] TerminalSession data structure
  - [x] Session tabs
  - [x] ANSI rendering plan (not implemented yet)
- [x] 4.5 Resolve Metal Toolchain blocker (Xcode 26 beta issue)
- [x] 4.6 Verify application compiles and runs

**Completion**: 2026-01-11 (Commit: 32470d0, 61e1e62)

---

## Phase 5: Advanced Feature Views ✅ COMPLETE

- [x] 5.1 **ChatView**: AI conversation interface
  - [x] ChatMessage component
  - [x] Message role rendering (User, Assistant, System)
  - [x] Input area
  - [x] Message history display
- [x] 5.2 **WorkflowsView**: Workflow management
  - [x] 4 panels: Constitution, Change Management, Review Gate, Context Engine
  - [x] ConstitutionRule component
  - [x] Panel switching UI
- [x] 5.3 **McpView**: MCP server inspector
  - [x] Server status display
  - [x] Tools list rendering
  - [x] Tool parameter inspection UI
- [x] 5.4 **SettingsView**: Configuration interface
  - [x] 4 categories: General, Project, MCP, Claude Code
  - [x] Settings form UI
  - [x] Category navigation
- [x] 5.5 Integrate all 8 views into main application
- [x] 5.6 Fix recursion limit compilation issues
- [x] 5.7 Clean up compiler warnings
- [x] 5.8 Verify all views render correctly

**Completion**: 2026-01-11 (Commits: b8f00d6, a7e1b6a, a1065f3, 56275dc)

---

## Phase 6: Backend Integration & Polish 🟡 IN PROGRESS (6%)

### Stage 1: Backend Data Integration ✅ COMPLETE (25%)

- [x] 6.1.1 **TasksView Backend Integration**
  - [x] Load justfile from current directory
  - [x] Parse commands using `rstn-core::justfile`
  - [x] Display all commands with descriptions
  - [x] Empty state handling
- [x] 6.1.2 **DockersView Backend Integration**
  - [x] Load built-in Docker services
  - [x] Display service metadata
  - [x] Status indicators (static, no polling yet)
- [x] 6.1.3 **Justfile Modernization**
  - [x] Remove Electron/Node.js commands
  - [x] Add GPUI/Rust commands (build, dev, run, test, lint, fmt, etc.)
  - [x] Update from 11 to 13 commands
- [x] 6.1.4 **Project Cleanup**
  - [x] Delete `pnpm-workspace.yaml`
  - [x] Update `package.json` (remove E2E scripts)
  - [x] Delete obsolete GitHub workflow (check-mock.yml)
  - [x] Mark E2E tests as deprecated
  - [x] Create cleanup checklist (CLEANUP_TODO.md)

**Completion**: 2026-01-12 (Commits: 2cacbc5, 92bdf49, 6ec68ab)

### Stage 2: State Management + Event Handling ⏸️ PENDING (0%)

- [ ] 6.2.1 Design AppState structure
  - [ ] Create `crates/rstn/src/state.rs`
  - [ ] Define TasksState, DockersState, etc.
  - [ ] Implement state loading methods
- [ ] 6.2.2 Refactor main.rs to use Model<AppState>
  - [ ] Replace inline data loading with state reads
  - [ ] Update render methods
- [ ] 6.2.3 Add event handling system
  - [ ] Button click handlers
  - [ ] Keyboard shortcuts
  - [ ] Action dispatch pattern
- [ ] 6.2.4 Implement command execution
  - [ ] Execute `just` commands via rstn-core
  - [ ] Stream output to LogPanel
  - [ ] Update task status (Running, Success, Failed)
- [ ] 6.2.5 Add background Docker polling
  - [ ] Spawn async task to poll Docker daemon
  - [ ] Update service status every 2-3 seconds
  - [ ] Use channels for state updates

**Estimated Effort**: 2-3 hours (major refactoring)

### Stage 3: Remaining Views Integration ⏸️ PENDING (0%)

- [ ] 6.3.1 **ExplorerView Integration**
  - [ ] Load file tree from `rstn-core::worktree`
  - [ ] Implement Git status checking
  - [ ] Add directory expansion/collapse
  - [ ] File selection handling
- [ ] 6.3.2 **TerminalView Integration**
  - [ ] Integrate `alacritty_terminal` crate
  - [ ] Implement PTY session management
  - [ ] Add terminal rendering with GPUI
  - [ ] Keyboard input handling
  - [ ] ANSI escape sequence parsing
- [ ] 6.3.3 **ChatView Integration**
  - [ ] Claude API client implementation
  - [ ] Message streaming support
  - [ ] Chat history persistence
  - [ ] Message formatting (code blocks, markdown)
- [ ] 6.3.4 **McpView Integration**
  - [ ] MCP server health check
  - [ ] Tools list fetching from HTTP endpoint
  - [ ] Real-time status monitoring
- [ ] 6.3.5 **WorkflowsView Integration**
  - [ ] Constitution rules loading
  - [ ] Change management state
  - [ ] Review gate workflow
  - [ ] Context engine data
- [ ] 6.3.6 **SettingsView Integration**
  - [ ] Config file management (TOML/JSON)
  - [ ] Settings read/write functionality
  - [ ] Form input handling
  - [ ] Config validation

### Stage 4: Polish & Testing ⏸️ PENDING (0%)

- [ ] 6.4.1 Performance optimization
  - [ ] Cache justfile parsing results
  - [ ] Optimize Docker polling frequency
  - [ ] Add virtualization for large lists
  - [ ] Profile GPU rendering
- [ ] 6.4.2 Testing infrastructure
  - [ ] Fix SIGBUS test execution error
  - [ ] Add integration tests
  - [ ] Achieve >80% code coverage
- [ ] 6.4.3 Documentation
  - [ ] Update README.md for GPUI architecture
  - [ ] Write user guide
  - [ ] Document keyboard shortcuts
  - [ ] API reference
- [ ] 6.4.4 Feature parity verification
  - [ ] Compare with old Electron version
  - [ ] Test all interactive features
  - [ ] Performance benchmarks

---

## Completion Criteria

### Must Have (MVP)
- [x] All 8 views display UI correctly
- [x] Application compiles without errors
- [x] Application runs and displays content
- [x] Justfile commands load from project
- [x] Docker services display correctly
- [ ] Basic interactivity (button clicks work)
- [ ] TasksView can execute commands
- [ ] DockersView shows real-time status

### Should Have
- [ ] State management system working
- [ ] All views load real backend data
- [ ] Terminal renders correctly
- [ ] Chat integrates with Claude API
- [ ] Unit tests pass

### Nice to Have
- [ ] Performance benchmarks met
- [ ] Comprehensive documentation
- [ ] Advanced features (MCP inspector, A2UI)
- [ ] Keyboard shortcuts for all actions

---

## Blockers & Risks

### ✅ Resolved Blockers
- ~~GPUI build requires Metal Toolchain (Xcode 26 beta issue)~~ → Fixed by switching to Xcode 15.4

### 🟡 Current Issues
- Test execution fails with SIGBUS error (deferred, not blocking)
- State management requires major refactoring (planned for Stage 2)

### 🔴 Active Risks
- Async state management complexity (High likelihood, High impact)
  - Mitigation: Study Zed's patterns, start simple
- Performance issues with Docker polling (Medium likelihood, Medium impact)
  - Mitigation: Poll every 2-3 seconds, not every frame

---

## References

- [GPUI_MIGRATION_PROGRESS.md](../../GPUI_MIGRATION_PROGRESS.md) - Overall progress tracking
- [PHASE_6_PLAN.md](../../PHASE_6_PLAN.md) - Phase 6 implementation plan
- [PHASE_6_PROGRESS.md](../../PHASE_6_PROGRESS.md) - Phase 6 detailed progress
- [PHASE_6_COMPLETE_SUMMARY.md](../../PHASE_6_COMPLETE_SUMMARY.md) - Stage 1 completion report
- [CLEANUP_TODO.md](../../CLEANUP_TODO.md) - Post-migration cleanup checklist

---

**Last Updated**: 2026-01-12
**Next Milestone**: Phase 6 Stage 2 - State Management System
