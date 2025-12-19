# Quick Start Guide

**Last Updated**: 2025-12-19
**Estimated Time**: 5 minutes

This guide will help you run your first rstn session and understand the basic interface.

---

## Prerequisites

Before starting, ensure you've completed the [Installation Guide](installation.md).

Verify installation:
```bash
rstn --version
# Should output: rstn 0.x.x
```

---

## Your First Session

### 1. Launch rstn

Open your terminal and run:

```bash
rstn
```

You should see the rstn TUI (Text User Interface) appear:

```
┌─ rstn v0.x.x ──────────────────────────────────────────────────┐
│  Worktree  │  Settings  │  Dashboard                           │
│═══════════════════════════════════════════════════════════════│
│                                                                 │
│  Welcome to rustation v2!                                      │
│                                                                 │
│  [Your worktree content appears here]                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Tip**: If you see errors, check the [Troubleshooting](#troubleshooting) section.

---

## Basic Navigation

### Tab Switching

Use these keys to switch between views:

| Key | Action |
|-----|--------|
| `Tab` | Next view (Worktree → Settings → Dashboard) |
| `Shift+Tab` | Previous view |
| Mouse click | Click on tab name to switch |

**Try it**: Press `Tab` to switch to Settings, then `Tab` again to reach Dashboard.

### Vertical Navigation

Within each view, use:

| Key | Action |
|-----|--------|
| `j` or `↓` | Move down |
| `k` or `↑` | Move up |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |

**Try it**: Press `j` and `k` to move through items in the current view.

### Common Actions

| Key | Action |
|-----|--------|
| `Enter` | Execute selected item / Confirm |
| `Esc` | Cancel / Close dialog |
| `q` | Quit rstn |
| `?` | Show help (future feature) |

---

## Understanding the Interface

### Three Main Views

**1. Worktree View** (Default)
- Shows your current git worktree
- Lists recent sessions
- Main workspace for development tasks

**2. Settings View**
- Configuration options
- Claude CLI integration settings
- MCP server status

**3. Dashboard View**
- Project overview
- Statistics and metrics
- Quick actions

### Status Bar

The bottom status bar shows:
- Current view name
- Active session (if any)
- MCP server status (if running)
- Keyboard hints

```
┌─────────────────────────────────────────────────────────────────┐
│  [View: Worktree] [Session: abc123] [MCP: ●] [q:quit] [?:help]│
└─────────────────────────────────────────────────────────────────┘
```

---

## Your First Action

Let's try a simple workflow:

### 1. Navigate to Settings

```
Press Tab to switch to Settings view
```

### 2. Explore Options

```
Use j/k to browse settings
Press Enter on an option to toggle it
```

### 3. Return to Worktree

```
Press Tab until you're back at Worktree view
```

### 4. Exit

```
Press q to quit rstn
```

You should see a clean exit with no errors.

---

## Troubleshooting

### Issue: TUI doesn't appear / garbled output

**Cause**: Terminal compatibility issue

**Solution**:
```bash
# Try with explicit terminal type
TERM=xterm-256color rstn

# Or update your terminal emulator
# Recommended: iTerm2 (macOS), Alacritty (cross-platform)
```

### Issue: Mouse clicks don't work

**Cause**: Mouse support not enabled

**Solution**: Use keyboard navigation (`Tab`, `j`, `k`, `Enter`) instead. Mouse support may vary by terminal.

### Issue: Colors look wrong

**Cause**: Terminal doesn't support 256 colors

**Solution**:
```bash
# Check color support
echo $TERM

# Should be one of: xterm-256color, screen-256color, tmux-256color
# If not, add to your shell config:
export TERM=xterm-256color
```

### Issue: rstn freezes or hangs

**Cause**: Event loop blocked or resource contention

**Solution**:
```bash
# Force quit with Ctrl+C
# Check logs for errors
tail -f ~/.rustation/logs/rstn.log

# Or
tail -f ~/.rstn/logs/rstn.log
```

---

## What You've Learned

After completing this guide, you should be able to:

- ✅ Launch rstn TUI
- ✅ Switch between views (Tab)
- ✅ Navigate with keyboard (j/k, Enter, Esc)
- ✅ Understand the three main views
- ✅ Exit cleanly (q)

---

## Next Steps

Now that you're comfortable with basic navigation:

1. **Learn Core Concepts**: [Understand rustation v2 architecture](concepts.md)
2. **Advanced Usage**: [SDD Workflow Guide](../04-development/sdd-workflow.md)
3. **MCP Integration**: [MCP Tools Reference](../03-api-reference/mcp-tools.md)

---

## Tips for Productive Use

### Keyboard-First Workflow

rstn is designed for keyboard efficiency:
- Learn `j/k` navigation (faster than arrow keys)
- Use `Tab` for quick view switching
- `Esc` gets you out of any dialog

### Log Monitoring

Keep logs open in another terminal:

```bash
# Terminal 1: Run rstn
rstn

# Terminal 2: Monitor logs
tail -f ~/.rstn/logs/rstn.log
```

### Multiple Sessions

rstn supports multiple concurrent sessions:
- Each session has a unique ID
- Sessions persist across rstn restarts
- Use `--continue-session` to resume

---

## Common Workflows

### Start a New Feature

```
1. Launch rstn
2. Navigate to Worktree view (Tab if needed)
3. Press 'p' to prompt Claude (future feature)
4. Follow the interactive workflow
```

### Review Previous Sessions

```
1. Navigate to Dashboard view (Tab twice)
2. Use j/k to browse session history
3. Press Enter to view session details
```

### Configure MCP Server

```
1. Navigate to Settings view (Tab once)
2. Find "MCP Server" section
3. Toggle settings as needed
4. rstn automatically restarts MCP server
```

---

## Keyboard Shortcuts Reference

| Key | Action | Context |
|-----|--------|---------|
| `Tab` | Next view | Global |
| `Shift+Tab` | Previous view | Global |
| `j` / `↓` | Move down | Navigation |
| `k` / `↑` | Move up | Navigation |
| `g` | Jump to top | Navigation |
| `G` | Jump to bottom | Navigation |
| `Ctrl+d` | Page down | Navigation |
| `Ctrl+u` | Page up | Navigation |
| `Enter` | Execute / Confirm | Global |
| `Esc` | Cancel / Close | Global |
| `q` | Quit | Global |
| `?` | Help (future) | Global |
| `p` | Prompt Claude (future) | Worktree |

---

## Need Help?

- **Logs**: Check `~/.rstn/logs/rstn.log` or `~/.rustation/logs/rstn.log`
- **Installation Issues**: See [Installation Guide](installation.md#troubleshooting)
- **Concepts**: Read [Core Concepts](concepts.md)
- **GitHub Issues**: [Report bugs or request features](https://github.com/chrischeng-c4/rustation/issues)

---

## Changelog

- 2025-12-19: Initial quick start guide for v2
