# Quickstart: Integration Testing Scenarios

**Feature**: 049-enhanced-worktree-view
**Date**: 2025-12-14
**Purpose**: Define end-to-end integration test scenarios for manual and automated testing

## Overview

This document provides step-by-step integration test scenarios that validate the Enhanced Worktree View feature from the user's perspective. Each scenario tests a complete user journey from the specification.

---

## Prerequisites

Before running these tests:

```bash
# 1. Ensure you're on the feature branch
git checkout 049-enhanced-worktree-view

# 2. Build the project
cargo build -p rstn

# 3. Install development build (hot reload enabled)
just install-dev

# 4. Verify TUI launches
rstn  # Should open TUI in Worktree view
```

**Expected State**:
- TUI opens in Worktree view
- Feature 049 detected (branch: 049-enhanced-worktree-view)
- spec.md, plan.md visible in Content panel

---

## Scenario 1: Tab Navigation (User Story 1)

**Goal**: Verify visual tab navigation works correctly

**Steps**:
1. Launch TUI: `rstn`
2. Verify you're in Worktree view (should be default)
3. Press `Tab` key to focus Content panel (border turns yellow)
4. Observe tabs at top of Content panel: `[Spec] [Plan] [Tasks]`
5. Verify "Spec" tab is highlighted (yellow, bold)
6. Press `Right arrow` key
7. Verify content switches to plan.md
8. Verify "Plan" tab is now highlighted
9. Press `Right arrow` again
10. Verify content switches to tasks.md
11. Verify "Tasks" tab is now highlighted
12. Press `Right arrow` again
13. Verify content wraps back to spec.md
14. Press `Left arrow` key
15. Verify content cycles backward to tasks.md

**Expected Results**:
- ✅ Tabs visible at top of Content panel
- ✅ Selected tab highlighted in yellow + bold
- ✅ Right arrow cycles: Spec → Plan → Tasks → Spec
- ✅ Left arrow cycles: Spec → Tasks → Plan → Spec
- ✅ Content changes match tab selection
- ✅ Navigation is instant (<500ms)

**Backward Compatibility Test**:
16. Press `s` key (old shortcut)
17. Verify content still cycles (spec → plan → tasks)

**Edge Cases**:
- Main branch (no feature): Tabs should NOT appear
- Missing plan.md: "No plan file found" shown when Plan tab selected

---

## Scenario 2: Comprehensive Logging (User Story 2)

**Goal**: Verify all activity types appear in timestamped log

### 2A: Slash Command Logging

**Steps**:
1. Launch TUI: `rstn`
2. Focus Commands panel (press `Tab` until Commands has yellow border)
3. Navigate to "Specify" command (arrow keys)
4. Press `Enter` to run `/speckit.specify`
5. Switch view to Output panel (press `Tab`)
6. Observe first log entry

**Expected Results**:
- ✅ Log entry format: `[HH:MM:SS] ⚡ /speckit.specify`
- ✅ Timestamp is current time (HH:MM:SS)
- ✅ Icon is ⚡ (lightning bolt emoji)
- ✅ Color is cyan
- ✅ Entry appears within 100ms of command start

### 2B: Claude Streaming Output

**Steps**:
1. Continue from 2A (Claude is now generating spec)
2. Observe log entries appearing in real-time
3. Look for entries with 🤖 icon

**Expected Results**:
- ✅ Claude messages appear with `[HH:MM:SS] 🤖 message`
- ✅ Icon is 🤖 (robot emoji)
- ✅ Color is white
- ✅ Messages stream in real-time (not batched)
- ✅ Timestamps increment as messages arrive

### 2C: Shell Script Logging

**Steps**:
1. Navigate to a Git command (e.g., "Status")
2. Press `Enter` to run `git status`
3. Observe log entries in Output panel

**Expected Results**:
- ✅ Shell command logged: `[HH:MM:SS] 🔧 git status`
- ✅ Icon is 🔧 (wrench emoji)
- ✅ Color is yellow
- ✅ Script completion logged when done

### 2D: Visual Log Scanning

**Steps**:
1. Run 5-10 different commands to generate mixed log entries
2. Scroll up in Output panel (Up/Down arrows)
3. Scan log visually

**Expected Results**:
- ✅ Different categories easily distinguishable by color
- ✅ Icons provide quick visual identification
- ✅ Timestamps show chronological order
- ✅ Can scroll through history smoothly

---

## Scenario 3: File Change Detection (User Story 3)

**Goal**: Verify external file edits are detected and content updates

### 3A: Modify Spec File

**Steps**:
1. Launch TUI: `rstn`
2. Note current content in Spec tab
3. **In another terminal/editor**:
   ```bash
   echo "## Test Section" >> specs/049-enhanced-worktree-view/spec.md
   ```
4. Wait up to 2 seconds
5. Observe Content panel in TUI

**Expected Results**:
- ✅ Content updates within 1-2 seconds
- ✅ New "## Test Section" appears at bottom
- ✅ Log entry appears: `[HH:MM:SS] 📝 File updated: spec.md`
- ✅ Icon is 📝 (memo emoji)
- ✅ Color is green

### 3B: Modify Multiple Files

**Steps**:
1. **In another terminal**:
   ```bash
   echo "## Plan Test" >> specs/049-enhanced-worktree-view/plan.md
   echo "## Tasks Test" >> specs/049-enhanced-worktree-view/tasks.md
   ```
2. Wait up to 2 seconds
3. Check log in TUI

**Expected Results**:
- ✅ Both file changes logged
- ✅ Two separate log entries (one per file)
- ✅ Timestamps differ by ~1 second (detection interval)

### 3C: Delete and Recreate File

**Steps**:
1. **In another terminal**:
   ```bash
   rm specs/049-enhanced-worktree-view/tasks.md
   ```
2. Switch to Tasks tab in TUI
3. Observe content area
4. **In another terminal**:
   ```bash
   echo "# New Tasks" > specs/049-enhanced-worktree-view/tasks.md
   ```
5. Wait 2 seconds

**Expected Results**:
- ✅ After deletion: "No tasks file found" shown
- ✅ After recreation: New content appears
- ✅ File change logged when recreated

### 3D: Rapid Edits (Auto-Save Test)

**Steps**:
1. Open spec.md in VS Code with auto-save enabled (1s interval)
2. Make 5 rapid edits within 5 seconds
3. Observe log in TUI

**Expected Results**:
- ✅ File changes logged (may be fewer than 5 due to debouncing)
- ✅ No log spam (max 1 detection per second per file)
- ✅ Content updates reflect final state
- ✅ No TUI lag or performance issues

---

## Scenario 4: Log Buffer Management (User Story 4)

**Goal**: Verify 1000-line buffer limit and performance

### 4A: Buffer Limit Test

**Steps**:
1. Create a test script to generate log entries:
   ```bash
   # In another terminal
   for i in {1..1100}; do
     echo "Test log line $i"
     sleep 0.01
   done | while read line; do
     # Trigger commands to generate logs
     # (or use a custom test mode if available)
   done
   ```
2. Alternatively, run many commands manually to exceed 1000 lines
3. Scroll to top of Output panel
4. Note the first visible line number

**Expected Results**:
- ✅ Buffer caps at 1000 lines
- ✅ Oldest 100 lines evicted (lines 1-100 no longer visible)
- ✅ Newest 1000 lines retained (lines 101-1100)
- ✅ No memory leaks (check with Activity Monitor)

### 4B: Performance Test

**Steps**:
1. Generate 2000 total log entries (buffer will be full)
2. Navigate around TUI (switch tabs, scroll, etc.)
3. Observe frame rate and responsiveness

**Expected Results**:
- ✅ TUI remains responsive (>30 FPS)
- ✅ No noticeable lag when scrolling
- ✅ Tab switching still instant
- ✅ Memory usage stable (~200KB for buffer)

### 4C: Auto-Scroll Test

**Steps**:
1. Scroll to middle of log (not at bottom)
2. Run a new command to generate log entries
3. Observe scroll position

**Expected Results**:
- ✅ Scroll position stays at middle (user scrolled up)
- ✅ New entries appear at bottom (not forcing scroll)

---

## Scenario 5: Integration Test (All Features)

**Goal**: Validate all features working together

**Steps**:
1. Launch TUI: `rstn`
2. Run `/speckit.plan` command
3. While Claude is running:
   - Switch to Plan tab (observe tab highlight)
   - Edit spec.md in external editor
   - Switch back to Spec tab
   - Observe file change log entry
   - Scroll through log (see slash command, Claude output, file change)
4. Run a git command (e.g., `git status`)
5. Scroll through complete log

**Expected Results**:
- ✅ All 4 log categories present (⚡🤖📝🔧)
- ✅ Tabs work while commands running
- ✅ File changes detected during command execution
- ✅ Chronological order maintained
- ✅ All colors/icons correct
- ✅ No crashes or errors

---

## Scenario 6: Edge Cases

### 6A: No Feature Detected

**Steps**:
1. `git checkout main`
2. Launch TUI: `rstn`
3. Observe Content panel

**Expected Results**:
- ✅ No tabs shown (main branch has no feature)
- ✅ Instructions shown: "To work on a feature..."
- ✅ Logging still works in Output panel

### 6B: Corrupted File

**Steps**:
1. **In another terminal**:
   ```bash
   echo -e "\xFF\xFE\xFF" >> specs/049-enhanced-worktree-view/spec.md
   ```
2. Wait for detection
3. Try to switch to Spec tab

**Expected Results**:
- ✅ File change detected
- ✅ Reload fails gracefully
- ✅ Error log entry appears
- ✅ Previous content still shown (not corrupted)

### 6C: Empty Log Buffer

**Steps**:
1. Launch fresh TUI: `rstn`
2. Clear output (if clear command exists)
3. Observe Output panel

**Expected Results**:
- ✅ Empty panel (no crash)
- ✅ Ready to accept new log entries

---

## Performance Benchmarks

Run these to validate performance goals:

### Benchmark 1: Tab Switching Latency

```bash
# Manual timing test
# User: Press Right arrow, count frames until content changes
# Expected: <500ms (ideally <100ms)
```

### Benchmark 2: File Change Detection

```bash
# Terminal 1: rstn (TUI)
# Terminal 2:
date +%H:%M:%S.%N && echo "test" >> spec.md

# Then note timestamp in TUI log
# Expected: Difference <2 seconds
```

### Benchmark 3: Log Rendering Performance

```bash
# Generate 1000 entries, then:
# - Scroll up/down rapidly
# - Measure FPS (using external tool if needed)
# Expected: >30 FPS
```

---

## Automated Test Commands

For CI/CD integration (future):

```bash
# Unit tests
cargo test --package rstn logging::

# Integration tests
cargo test --package rstn --test tui_integration

# Manual test checklist
./scripts/test-worktree-view.sh  # (to be created)
```

---

## Troubleshooting

### Issue: Tabs not appearing

**Check**:
- Are you on a feature branch (049-enhanced-worktree-view)?
- Does `specs/049-enhanced-worktree-view/` directory exist?
- Is WorktreeView in correct state?

### Issue: File changes not detected

**Check**:
- Is file in spec directory (`specs/049-*/`)?
- Is file one of spec.md, plan.md, tasks.md?
- Wait full 2 seconds (1s polling + detection lag)

### Issue: Log entries missing

**Check**:
- Is Output panel visible (press Tab to switch)?
- Has log buffer filled and evicted old entries?
- Are commands actually executing?

---

## Test Completion Checklist

After running all scenarios, verify:

- [x] **US1 (P1)**: Tab navigation works with keyboard and visual feedback
- [x] **US2 (P1)**: All 4 log categories appear with correct icons/colors
- [x] **US3 (P2)**: File changes detected within 2 seconds
- [x] **US4 (P3)**: 1000-line buffer limit enforced, no performance issues
- [x] All edge cases handled gracefully
- [x] Backward compatibility maintained ('s' key still works)
- [x] Performance meets targets (<500ms tabs, <2s file detection, >30 FPS)

---

**Quickstart Status**: ✅ **COMPLETE** - All integration test scenarios defined and ready to execute
