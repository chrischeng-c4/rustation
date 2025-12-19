# Debugging Guide

**Last Updated**: 2025-12-19
**Version**: v2 (state-first debugging)

This guide covers debugging techniques for rustation v2, with emphasis on **state inspection** and **observability**.

---

## Core Debugging Principle

**In v2, debugging = state inspection.**

Because all state is JSON/YAML serializable, you can:
- **Save buggy state** → reproduce exact bug
- **Load state** → start from specific point
- **Inspect state** → see exact application state at any time

**No more guessing. Just observe the state.**

---

## Quick Debugging Checklist

```
□ 1. Check logs first (~/.rstn/logs/ or ~/.rustation/logs/)
□ 2. Save current state (--save-state buggy.json)
□ 3. Reproduce with saved state (--load-state buggy.json)
□ 4. Inspect state as JSON (cat buggy.json | jq)
□ 5. Add targeted logging if needed
□ 6. Verify fix with state tests
```

---

## Log Locations

### rstn Logs

**Primary location**:
```bash
~/.rstn/logs/rstn.log
```

**Alternative location** (if using older config):
```bash
~/.rustation/logs/rstn.log
```

**Check which location is active**:
```bash
ls -la ~/.rstn/logs/
ls -la ~/.rustation/logs/
```

### rush Logs

**Location**:
```bash
~/.rustation/logs/rush.log
```

### View Logs in Real-Time

```bash
# Follow logs (auto-scroll)
tail -f ~/.rstn/logs/rstn.log

# Last 100 lines
tail -100 ~/.rstn/logs/rstn.log

# Search for specific keyword
grep -i "error" ~/.rstn/logs/rstn.log

# Filter by timestamp
grep "2025-12-19 10:" ~/.rstn/logs/rstn.log
```

---

## State Inspection

### Save Current State

```bash
# From CLI
rstn --save-state snapshot.json

# From TUI (future feature)
# Press 'S' in TUI to save state
```

**Output**: `snapshot.json` with complete application state

### Load Saved State

```bash
# Start with saved state
rstn --load-state snapshot.json

# Verify state loaded correctly
rstn --load-state snapshot.json --save-state verify.json
diff snapshot.json verify.json  # Should be identical
```

**Use case**: Reproduce bugs exactly

### Inspect State as JSON

```bash
# Pretty-print JSON
cat snapshot.json | jq '.'

# Extract specific field
cat snapshot.json | jq '.current_view'

# Filter by condition
cat snapshot.json | jq '.worktree_view | select(.active_session_id != null)'

# Count items
cat snapshot.json | jq '.worktree_view.sessions | length'
```

**Tool**: Install `jq` for JSON manipulation
```bash
# macOS
brew install jq

# Linux
sudo apt install jq
```

---

## Common Issues and Solutions

### Issue 1: rstn crashes on startup

**Symptoms**:
```
thread 'main' panicked at 'Failed to initialize terminal'
```

**Diagnosis**:
```bash
# Check logs
tail -50 ~/.rstn/logs/rstn.log

# Look for initialization errors
grep -i "init" ~/.rstn/logs/rstn.log
```

**Common causes**:
1. Terminal not compatible
2. TERM environment variable incorrect
3. Crossterm initialization failed

**Solutions**:
```bash
# Solution 1: Set TERM explicitly
TERM=xterm-256color rstn

# Solution 2: Try different terminal emulator
# Recommended: iTerm2 (macOS), Alacritty (cross-platform)

# Solution 3: Check terminal capabilities
echo $TERM
infocmp $TERM
```

### Issue 2: State deserialization fails

**Symptoms**:
```
Error: Failed to load state from snapshot.json
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value'
```

**Diagnosis**:
```bash
# Validate JSON syntax
cat snapshot.json | jq '.' > /dev/null
# If error: JSON is malformed

# Check for missing fields
cat snapshot.json | jq 'keys'
```

**Common causes**:
1. JSON file corrupted
2. State struct changed (breaking change)
3. Missing required fields

**Solutions**:
```bash
# Solution 1: Validate JSON
jq '.' snapshot.json

# Solution 2: Check state version
jq '.version' snapshot.json

# Solution 3: Regenerate state from default
rstn --save-state clean_state.json
diff snapshot.json clean_state.json
```

### Issue 3: MCP server not responding

**Symptoms**:
```
Error: Connection refused (os error 61)
Claude Code can't connect to MCP server
```

**Diagnosis**:
```bash
# Check if server is running
lsof -i :19560  # Or check dynamic port

# Check MCP config
cat ~/.rstn/mcp-session.json

# Check logs for server errors
grep -i "mcp" ~/.rstn/logs/rstn.log
```

**Common causes**:
1. MCP server not started
2. Port binding failed
3. Invalid MCP config

**Solutions**:
```bash
# Solution 1: Verify rstn TUI is running
# MCP server only runs when TUI is active

# Solution 2: Check MCP config format
cat ~/.rstn/mcp-session.json
# Should have:
# {"mcpServers":{"rstn":{"type":"http","url":"http://127.0.0.1:{port}/mcp"}}}

# Solution 3: Regenerate config
rm ~/.rstn/mcp-session.json
rstn  # Will regenerate on startup
```

### Issue 4: Session not continuing

**Symptoms**:
```
Error: Session 'abc123' not found
Claude Code doesn't remember previous context
```

**Diagnosis**:
```bash
# Check session state
cat snapshot.json | jq '.worktree_view.active_session_id'

# Check logs for session errors
grep -i "session.*abc123" ~/.rstn/logs/rstn.log
```

**Common causes**:
1. Session ID incorrect
2. Session state not persisted
3. Session directory missing

**Solutions**:
```bash
# Solution 1: List available sessions
rstn sessions list  # (future feature)

# Solution 2: Check session storage
ls ~/.rstn/sessions/

# Solution 3: Save state before exit
rstn --save-state before_exit.json
# Then check: jq '.worktree_view.active_session_id' before_exit.json
```

### Issue 5: Logs show panics/unwrap errors

**Symptoms**:
```
thread 'main' panicked at 'called `unwrap()` on a `None` value'
```

**Diagnosis**:
```bash
# Get full backtrace
RUST_BACKTRACE=full rstn 2>&1 | tee panic.log

# Find the panic location
grep -A 10 "panicked at" panic.log
```

**Common causes**:
1. Unwrap on None/Err (code smell!)
2. Missing error handling
3. Invalid assumptions

**Solutions**:
```bash
# Solution 1: Save state before panic
rstn --save-state before_panic.json

# Solution 2: Report bug with state + backtrace
# Include both files in GitHub issue

# Solution 3: Fix in code (replace unwrap with ?)
# See Contribution Guide for proper error handling
```

---

## Advanced Debugging Techniques

### Technique 1: State Diffing

**Problem**: What changed between two states?

**Solution**: Diff JSON files
```bash
# Save state before action
rstn --save-state before.json

# Perform action (e.g., switch view)

# Save state after action
rstn --save-state after.json

# Diff to see what changed
diff <(jq --sort-keys '.' before.json) <(jq --sort-keys '.' after.json)

# Or use specialized JSON diff tool
jd before.json after.json  # Requires 'jd' tool
```

### Technique 2: Logging State Transitions

**Problem**: Don't know when state changed

**Solution**: Add tracing to state mutations
```rust
// In your code
impl App {
    pub fn handle_action(&mut self, action: ViewAction) {
        let before = self.to_state();

        // ... perform action ...

        let after = self.to_state();

        // Log state diff
        tracing::debug!(
            "State transition: action={:?}, before={:?}, after={:?}",
            action,
            before,
            after
        );
    }
}
```

**View logs**:
```bash
# Enable debug logging
RUST_LOG=debug rstn

# Filter for state transitions
RUST_LOG=debug rstn 2>&1 | grep "State transition"
```

### Technique 3: Time-Travel Debugging

**Problem**: Bug appears after multiple actions

**Solution**: Save state at each step
```bash
# In test or development:
rstn --save-state step_0.json  # Initial
# ... action 1 ...
rstn --save-state step_1.json
# ... action 2 ...
rstn --save-state step_2.json
# ... action 3 (bug appears)
rstn --save-state step_3_buggy.json

# Now replay step by step
rstn --load-state step_0.json  # Start from beginning
rstn --load-state step_1.json  # After action 1
rstn --load-state step_2.json  # After action 2
rstn --load-state step_3_buggy.json  # Reproduce bug
```

### Technique 4: Minimal Reproduction

**Problem**: Bug only appears in complex state

**Solution**: Reduce state to minimum that reproduces bug
```bash
# Start with buggy state
cat buggy.json | jq '.' > minimal.json

# Manually remove fields one by one
jq 'del(.dashboard_view)' minimal.json > minimal_1.json

# Test if bug still reproduces
rstn --load-state minimal_1.json

# If bug reproduced: Keep removing fields
# If bug gone: That field was necessary, restore it

# Repeat until state is minimal
```

### Technique 5: State Fuzzing (Advanced)

**Problem**: Don't know which state causes bug

**Solution**: Generate random valid states, test for panics
```rust
// In tests
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_no_panic_on_random_state(
        view in prop::sample::select(vec![
            ViewType::Worktree,
            ViewType::Settings,
            ViewType::Dashboard
        ]),
        session_id in prop::option::of(prop::string::string_regex("[a-z0-9]{6}").unwrap()),
    ) {
        let state = AppState {
            current_view: view,
            worktree_view: WorktreeViewState {
                active_session_id: session_id,
                ..Default::default()
            },
            ..Default::default()
        };

        // Should not panic on any valid state
        let mut app = App::from_state(state).unwrap();
        app.handle_action(ViewAction::Noop);
    }
}
```

---

## Debugging Workflow

### Step-by-Step Process

**1. Observe the symptom**
```bash
# Note exact error message
rstn 2>&1 | tee error.log

# Or capture from logs
tail -50 ~/.rstn/logs/rstn.log > symptom.log
```

**2. Save current state**
```bash
rstn --save-state buggy_state.json
```

**3. Inspect state**
```bash
# Pretty-print to see structure
cat buggy_state.json | jq '.'

# Look for unexpected values
cat buggy_state.json | jq '.worktree_view.active_session_id'
```

**4. Reproduce**
```bash
# Load saved state
rstn --load-state buggy_state.json

# Verify bug reproduces
# If yes: State captured correctly
# If no: State not the cause, check logs
```

**5. Minimize**
```bash
# Create minimal reproduction state
cat buggy_state.json | jq '{
  version: .version,
  current_view: .current_view,
  worktree_view: {
    active_session_id: .worktree_view.active_session_id
  }
}' > minimal_state.json

# Test minimal state
rstn --load-state minimal_state.json
```

**6. Write state test**
```rust
#[test]
fn test_bug_reproduction() {
    let buggy_state = serde_json::from_str(include_str!("minimal_state.json")).unwrap();
    let mut app = App::from_state(buggy_state).unwrap();

    // Should not panic
    app.handle_action(ViewAction::ProblemAction);

    // Assert expected behavior
    let final_state = app.to_state();
    assert_eq!(final_state.expected_field, expected_value);
}
```

**7. Fix and verify**
```bash
# Fix the code

# Run test
cargo test test_bug_reproduction

# Verify with saved state
rstn --load-state buggy_state.json

# Should not reproduce bug anymore
```

---

## Environment Variables

### Logging Control

```bash
# Set log level
RUST_LOG=debug rstn       # Debug (verbose)
RUST_LOG=info rstn        # Info (default)
RUST_LOG=warn rstn        # Warnings only
RUST_LOG=error rstn       # Errors only

# Module-specific logging
RUST_LOG=rstn::tui::app=debug rstn   # Debug app.rs only

# Multiple modules
RUST_LOG=rstn::tui=debug,rstn::domain=info rstn
```

### Backtrace Control

```bash
# Short backtrace
RUST_BACKTRACE=1 rstn

# Full backtrace
RUST_BACKTRACE=full rstn

# No backtrace
RUST_BACKTRACE=0 rstn
```

### Terminal Control

```bash
# Force terminal type
TERM=xterm-256color rstn

# Disable color
NO_COLOR=1 rstn
```

---

## Performance Debugging

### Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile rstn
cargo flamegraph -p rstn

# Open flamegraph.svg in browser
open flamegraph.svg
```

### Memory Usage

```bash
# Install heaptrack (Linux) or Instruments (macOS)

# Linux: Track memory allocations
heaptrack rstn

# macOS: Use Instruments
instruments -t "Allocations" rstn
```

### CPU Usage

```bash
# Check CPU usage
top -pid $(pgrep rstn)

# Or use htop
htop -p $(pgrep rstn)
```

---

## Debugging Tools Reference

### Essential Tools

| Tool | Purpose | Install |
|------|---------|---------|
| `jq` | JSON manipulation | `brew install jq` |
| `tail` | View logs | Built-in |
| `grep` | Search logs | Built-in |
| `diff` | Compare states | Built-in |
| `lsof` | Check ports | Built-in (macOS/Linux) |

### Optional Tools

| Tool | Purpose | Install |
|------|---------|---------|
| `jd` | JSON diff | `brew install jd` |
| `flamegraph` | CPU profiling | `cargo install flamegraph` |
| `heaptrack` | Memory profiling | Linux package manager |
| `rust-lldb` / `rust-gdb` | Interactive debugger | `rustup component add lldb-preview` |

---

## Remote Debugging (Future)

**Planned features**:

- **Remote state inspection**: View state from another machine
- **State streaming**: Stream state changes in real-time
- **Remote control**: Send actions to running rstn instance

**Use case**: Debug issues on user machines without reproducing locally

---

## Best Practices

### ✅ DO

- Save state BEFORE reporting bugs
- Include logs with bug reports
- Minimize reproduction states
- Write state tests for bugs
- Use `?` for error propagation (not `unwrap`)
- Log state transitions at DEBUG level
- Test with --load-state before committing

### ❌ DON'T

- Use `unwrap()` in production code
- Ignore log messages
- Skip state inspection
- Debug by adding `println!()` (use `tracing::debug!()`)
- Commit code without reproducing bug
- Report bugs without state/logs

---

## Getting Help

### Documentation

- [State-First Architecture](../02-architecture/state-first.md)
- [Testing Guide](testing-guide.md)
- [Contribution Guide](contribution-guide.md)

### Community

- **GitHub Issues**: [Report bugs](https://github.com/chrischeng-c4/rustation/issues)
- **GitHub Discussions**: [Ask questions](https://github.com/chrischeng-c4/rustation/discussions)

### Bug Report Template

```markdown
## Environment
- rstn version: 0.x.x
- OS: macOS 14.0 / Linux Ubuntu 22.04
- Terminal: iTerm2 / Alacritty
- TERM: xterm-256color

## Symptom
[Describe what happened]

## Reproduction
[Steps to reproduce]

## State
[Attach buggy_state.json]

## Logs
[Attach relevant log snippet]

## Expected Behavior
[Describe what should happen]
```

---

## Quick Reference

### State Commands
```bash
rstn --save-state <file>      # Save current state
rstn --load-state <file>      # Load saved state
```

### Log Commands
```bash
tail -f ~/.rstn/logs/rstn.log          # Follow logs
grep -i "error" ~/.rstn/logs/rstn.log  # Search logs
```

### JSON Commands
```bash
cat state.json | jq '.'                # Pretty-print
jq '.field' state.json                 # Extract field
diff <(jq -S . a.json) <(jq -S . b.json)  # Diff states
```

### Debug Commands
```bash
RUST_LOG=debug rstn                    # Verbose logging
RUST_BACKTRACE=full rstn              # Full backtraces
```

---

## Changelog

- 2025-12-19: Initial debugging guide for v2
