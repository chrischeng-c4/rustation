# Feature Specification: Enhanced Worktree View with Tabs and Comprehensive Logging

**Feature Branch**: `049-enhanced-worktree-view`
**Created**: 2025-12-14
**Status**: Draft
**Input**: User description: "Enhanced Worktree View with in-panel tabs for spec/plan/tasks navigation, comprehensive command logging (slash commands, Claude streaming, file changes, shell output) with timestamps and icons, real-time file change detection, and 1000-line rolling buffer"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Quick Navigation Between Spec Documents (Priority: P1)

As a developer using the SDD workflow, I want to quickly switch between viewing spec.md, plan.md, and tasks.md without leaving the current view, so I can reference different documents while working on a feature without disrupting my workflow.

**Why this priority**: This is the most fundamental improvement to the user experience. Currently, users must use keyboard commands to cycle through content types, which is cumbersome when frequently referencing multiple documents. Tab-based navigation is intuitive and provides visual feedback on available documents.

**Independent Test**: Can be fully tested by opening the Worktree view on a feature branch with spec/plan/tasks files, clicking or navigating tabs, and verifying that content switches immediately. Delivers immediate value even without the logging features.

**Acceptance Scenarios**:

1. **Given** I'm in the Worktree view on a feature branch, **When** I focus the Content panel, **Then** I see visual tabs labeled "Spec", "Plan", and "Tasks" at the top of the panel
2. **Given** I'm viewing the Spec tab, **When** I press the Right arrow key, **Then** the content switches to Plan and the Plan tab is highlighted
3. **Given** I'm viewing the Tasks tab, **When** I press the Left arrow key, **Then** the content switches to Plan
4. **Given** I'm viewing the Plan tab, **When** I press 's' (existing shortcut), **Then** the content cycles to the next document type (backward compatibility)
5. **Given** I'm on the main branch with no feature detected, **When** I view the Content panel, **Then** no tabs are shown and I see instructions for creating a feature

---

### User Story 2 - Comprehensive Activity Logging (Priority: P1)

As a developer troubleshooting the SDD workflow, I want to see a timestamped log of all commands executed (slash commands, shell scripts, Claude output) in a single unified view, so I can understand what happened during feature development and diagnose issues quickly.

**Why this priority**: Critical for debugging workflow issues and understanding the sequence of events. Without comprehensive logging, users have no visibility into what commands were executed, when Claude ran, or what scripts completed. This is essential for a developer tool.

**Independent Test**: Can be tested by running various SDD workflow commands (/speckit.specify, git status, etc.) and verifying all activities appear in the Output panel with timestamps and category icons. Delivers value independently of tabs.

**Acceptance Scenarios**:

1. **Given** I run a slash command like `/speckit.specify`, **When** the command starts, **Then** I see a log entry with timestamp, ⚡ icon, and the full command in the Output panel
2. **Given** Claude is generating a specification, **When** Claude streams output, **Then** I see Claude's messages in real-time with 🤖 icon in the Output panel
3. **Given** a bash script executes (e.g., create-new-feature.sh), **When** the script completes, **Then** I see a log entry with timestamp, 🔧 icon, script name, and exit code
4. **Given** I have run multiple commands generating 50 log lines, **When** I scroll up in the Output panel, **Then** I can review all previous log entries in chronological order
5. **Given** the Output panel shows various log entries, **When** I view them, **Then** each category (slash commands, Claude, shell, file changes, system) has a distinct color for easy visual scanning

---

### User Story 3 - Automatic File Change Detection (Priority: P2)

As a developer editing spec files in an external editor while the TUI is open, I want the TUI to automatically detect when I save changes and reload the file content, so I can see updates immediately without manually refreshing or reopening files.

**Why this priority**: Enhances workflow fluidity for users who prefer external editors (VS Code, Vim, etc.) while keeping the TUI open for monitoring. Important for multi-tool workflows but not blocking for core SDD functionality.

**Independent Test**: Can be tested by opening spec.md in VS Code, making a change, saving, and verifying within 1-2 seconds the TUI shows updated content and a log entry. Works independently of tabs and logging features.

**Acceptance Scenarios**:

1. **Given** I have spec.md open in the TUI, **When** I edit spec.md in VS Code and save, **Then** within 1-2 seconds the Content panel shows the updated content
2. **Given** file change detection is active, **When** spec.md is modified externally, **Then** I see a log entry with timestamp, 📝 icon, and "File updated: spec.md"
3. **Given** I modify plan.md externally, **When** the file is saved, **Then** the plan content is reloaded and a log entry appears
4. **Given** I delete tasks.md externally, **When** the deletion is detected, **Then** the Content panel shows "No tasks file found" instead of stale content
5. **Given** I'm on the main branch with no feature, **When** I switch to a feature branch, **Then** file watching activates for that feature's spec directory

---

### User Story 4 - Managed Log History (Priority: P3)

As a power user running many SDD workflow commands in a single session, I want the log history to be automatically limited to the most recent 1000 lines, so the TUI remains responsive and doesn't consume excessive memory during long development sessions.

**Why this priority**: Nice-to-have performance optimization. Most users won't hit 1000 log lines in a typical session, but this prevents pathological cases. Lower priority because current behavior (unlimited growth) won't cause immediate problems for most users.

**Independent Test**: Can be tested by generating >1000 log lines (via script or many commands) and verifying the buffer caps at 1000 lines with oldest entries removed. Tests memory management independently.

**Acceptance Scenarios**:

1. **Given** I have 950 log entries, **When** I execute commands generating 100 more entries, **Then** the total entries remain at 1000 and the oldest 50 are removed
2. **Given** I have a full log buffer (1000 lines), **When** I scroll to the top of the Output panel, **Then** I see entries starting from line 501 (oldest 500 removed)
3. **Given** I run commands generating 2000 log lines total, **When** I view the Output panel, **Then** the TUI remains responsive with no noticeable lag
4. **Given** the log buffer is at capacity, **When** new entries arrive, **Then** auto-scrolling to the bottom still works smoothly

---

### Edge Cases

- What happens when spec.md exists but plan.md and tasks.md don't? → Tabs still show but clicking Plan/Tasks shows "No file found" message
- What happens when I rapidly save a file multiple times (e.g., auto-save every second)? → File change detection debounces to prevent log spam, max one detection per second per file
- What happens when I'm viewing Plan tab and delete plan.md externally? → Content area shows "No plan file found" but tab remains visible
- What happens when the log buffer is full and Claude streams a very long response? → Older entries are removed as new lines arrive, maintaining 1000-line limit
- What happens when I switch from a feature branch to main? → File watching stops, tabs disappear, log entries remain in buffer
- What happens when spec.md is corrupted or has encoding issues? → File change is detected but reload fails gracefully, showing previous content with an error log entry
- What happens when I navigate tabs while a command is running? → Tab switching works immediately; running command output continues in the Output panel

## Requirements *(mandatory)*

### Functional Requirements

#### Tab Navigation
- **FR-001**: System MUST display visual tabs labeled "Spec", "Plan", and "Tasks" at the top of the Content panel when a feature branch is detected
- **FR-002**: System MUST highlight the currently selected tab with distinct styling (yellow color, bold text)
- **FR-003**: Users MUST be able to switch tabs using Left/Right arrow keys when the Content panel is focused
- **FR-004**: System MUST maintain backward compatibility with the existing 's' key for cycling through content types
- **FR-005**: System MUST show the appropriate file content (spec.md, plan.md, or tasks.md) when a tab is selected
- **FR-006**: System MUST hide tabs when no feature is detected (e.g., on main branch) and show feature creation instructions instead

#### Comprehensive Logging
- **FR-007**: System MUST log all slash command executions (e.g., /speckit.specify) with format: `[HH:MM:SS] ⚡ /command args`
- **FR-008**: System MUST log all Claude Code streaming output with format: `[HH:MM:SS] 🤖 message`
- **FR-009**: System MUST log all file change detections with format: `[HH:MM:SS] 📝 File updated: filename`
- **FR-010**: System MUST log all shell script executions with format: `[HH:MM:SS] 🔧 script-name completed (exit: code)`
- **FR-011**: System MUST display all log entries in chronological order (oldest to newest)
- **FR-012**: System MUST apply distinct colors to log entries by category: cyan (slash commands), white (Claude), green (file changes), yellow (shell), dark gray (system)
- **FR-013**: System MUST support scrolling through log history in the Output panel
- **FR-014**: System MUST auto-scroll to the bottom of the log when new entries arrive (unless user has scrolled up manually)

#### File Change Detection
- **FR-015**: System MUST monitor spec.md, plan.md, and tasks.md for external modifications when a feature is detected
- **FR-016**: System MUST detect file changes within 1-2 seconds of the file being saved
- **FR-017**: System MUST reload file content automatically when a change is detected
- **FR-018**: System MUST log each file change detection event
- **FR-019**: System MUST handle file deletions gracefully by showing "No file found" message without crashing
- **FR-020**: System MUST stop file watching when switching away from a feature branch

#### Log Buffer Management
- **FR-021**: System MUST maintain a rolling buffer of the most recent 1000 log entries
- **FR-022**: System MUST automatically remove the oldest entry when the buffer is full and a new entry arrives
- **FR-023**: System MUST maintain the 1000-line limit regardless of log entry length or content
- **FR-024**: System MUST preserve log entries across tab switches and view changes within the same TUI session
- **FR-025**: Log buffer MUST clear when explicitly requested by the user (existing clear functionality)

### Key Entities *(include if feature involves data)*

- **LogEntry**: Represents a single logged event with timestamp (SystemTime), category (SlashCommand, ClaudeStream, FileChange, ShellOutput, System), and content (string message)
- **LogBuffer**: Circular buffer data structure holding up to 1000 LogEntry items, automatically evicting oldest when full
- **FileChangeTracker**: Tracks file modification times (SystemTime) for spec.md, plan.md, tasks.md to detect external changes
- **TabState**: Current tab selection state (Spec, Plan, or Tasks) within the Content panel
- **ContentType**: Enumeration of available content types (Spec, Plan, Tasks) mapped to tab positions

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can switch between spec/plan/tasks views in under 0.5 seconds using arrow keys or tab clicks
- **SC-002**: All command executions (slash commands, shell scripts, Claude runs) appear in the log within 100ms of starting
- **SC-003**: External file changes are detected and content is reloaded within 2 seconds of file save
- **SC-004**: Log buffer maintains 1000-line limit with no memory leaks during sessions exceeding 2000 total log entries
- **SC-005**: Tab navigation works correctly 100% of the time when Content panel is focused
- **SC-006**: Users can identify log entry type at a glance using color coding and emoji icons with 95% accuracy
- **SC-007**: TUI remains responsive (frame rate >30 FPS) even with a full 1000-line log buffer
- **SC-008**: File change detection works for all three file types (spec, plan, tasks) with 100% reliability
- **SC-009**: Log entries are displayed in strict chronological order with accurate timestamps (HH:MM:SS format)
- **SC-010**: Backward compatibility: existing 's' key command continues to work for users who prefer it

### Qualitative Outcomes

- Users report improved workflow efficiency when referencing multiple spec documents
- Developers can troubleshoot SDD workflow issues faster using the comprehensive log
- Users working with external editors experience seamless integration with the TUI
- Long development sessions remain smooth without performance degradation
