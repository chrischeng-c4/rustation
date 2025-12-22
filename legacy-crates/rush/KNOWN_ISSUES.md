# Known Issues and Limitations

**Version:** 0.1.0 (Alpha)
**Last Updated:** 2025-11-18

This document tracks what's implemented, what's not, and what's planned for future releases.

## ✅ What Works (v0.1.0)

### Core Shell Functionality
- ✅ **Interactive REPL** - Read-eval-print loop with line editing
- ✅ **Command Execution** - Run external commands (e.g., `ls`, `grep`, `echo`)
- ✅ **Argument Passing** - Commands with flags and arguments work correctly
- ✅ **Exit Codes** - Properly captures and tracks command exit codes
- ✅ **Signal Handling**
  - Ctrl+C: Cancel current line
  - Ctrl+D: Exit shell

### User Experience
- ✅ **Syntax Highlighting** - Real-time color coding as you type
- ✅ **Command History** - Persistent history across sessions
  - ↑/↓ arrow key navigation
  - Survives shell restart
- ✅ **Autosuggestions** - Fish-like suggestions from command history
  - Inline grayed-out text as you type
  - Right Arrow (→) to accept full suggestion
  - Alt+Right Arrow (⌥→) to accept word-by-word
  - Most recent matches prioritized
- ✅ **Tab Completion** - Intelligent completion for commands, paths, and flags
  - Command names from PATH
  - File and directory paths (with tilde expansion)
  - Flags for common commands (git, cargo, ls, grep, cat, find, echo, cd)
- ✅ **Line Editing** - Cursor movement, backspace, delete
- ✅ **Empty Line Handling** - Gracefully skips empty input

### CLI & Diagnostics
- ✅ **Verbose Logging** - Debug (`-v`) and trace (`-vv`) modes
- ✅ **Configuration Inspection** - `--dump-config` shows settings
- ✅ **Health Check** - `--doctor` validates installation
- ✅ **Version Info** - `--version` displays build version

## ❌ What Doesn't Work Yet

### Shell Features (Planned for v0.2.0+)
- ❌ **Pipes** - `ls | grep foo` not supported
- ❌ **Redirections** - `echo foo > file.txt` not supported
- ❌ **Job Control** - No background jobs or `fg`/`bg`
- ❌ **Custom Prompts** - Prompt is fixed to `"$ "`
- ❌ **Environment Variables** - Cannot set/export variables
- ❌ **Aliases** - No alias support
- ❌ **Shell Scripts** - Cannot execute `.sh` files

### Configuration (Planned for v0.2.0)
- ❌ **Config File Loading** - `--config <file>` flag exists but always uses defaults
  - Workaround: Use command-line flags to override settings
  - Planned: Full TOML config loading in v0.2.0
- ❌ **Custom Settings** - Cannot customize prompt, colors, etc.
- ❌ **History Size Override** - `--history-size` flag defined but not wired

### Advanced Features (Future)
- ❌ **JSON Logging** - `--log-format json` not implemented
- ❌ **Feature Toggles** - `--no-highlight`, `--no-suggestions` defined but not wired

## 🐛 Known Bugs

None reported yet. Please report issues during alpha testing!

## ⚠️ Current Limitations

### Command Parsing
- ✅ **Quote Handling** - Double and single quotes now supported (`echo "hello world"` works!)
- ✅ **Escape Characters** - Backslash escaping supported (`echo hello\ world` works)
- ❌ **No Globbing** - `ls *.txt` won't expand wildcards (planned for v0.2.0)

### Process Management
- **No Background Jobs** - All commands run in foreground
- **No Process Control** - Cannot suspend, resume, or kill jobs
- **Inherit Stdio** - All output goes directly to terminal

### Cross-Platform
- **macOS/Linux Only** - Windows support not tested
- **Path Differences** - Config/log locations differ by OS

### Concurrency
- **No File Locking** - Multiple rush instances may corrupt history file
  - Workaround: Avoid running multiple instances simultaneously
  - Planned: File locking in v0.2.0 using fs2 crate

## 📋 Roadmap

### v0.2.0 (Next Release)
**Focus: Enhanced Parsing & Configuration**
- [x] Implement tab completion (✅ Completed in v0.1.0)
- [x] Add autosuggestions from history (✅ Completed in v0.1.0)
- [ ] Support basic pipes (`|`)
- [ ] Support basic redirections (`>`, `>>`, `<`)
- [ ] Load configuration from TOML file
- [ ] Custom prompt configuration
- [ ] Add file locking for history (prevent concurrent write corruption)

### v0.3.0 (Future)
**Focus: Advanced Shell Features**
- [ ] Job control (background jobs, `&`)
- [ ] Environment variable support
- [ ] Alias support
- [ ] Shell script execution
- [ ] Globbing/wildcard expansion

### v1.0.0 (Future)
**Focus: Production Ready**
- [ ] Full POSIX compatibility
- [ ] Plugin system
- [ ] Scripting language
- [ ] Windows support
- [ ] Performance optimization

## 🧪 Testing Notes

### Test Coverage
- **159 tests passing** (all unit, integration, and doc tests)
  - **Tab completion tests**: Command, path, and flag completion
  - **Autosuggestions tests**: Prefix matching, cursor validation, real-time updates
  - **Parser tests**: Quote and escape character handling
- See [TEST_COVERAGE.md](TEST_COVERAGE.md) for details

### Alpha Testing Focus

Please test and report issues for:
1. **Command execution** - Try various commands with different arguments
2. **History persistence** - Restart rush, verify history survives
3. **Syntax highlighting** - Type commands, check colors appear correctly
4. **Autosuggestions** - Type partial commands, verify suggestions appear and accept correctly
5. **Tab completion** - Test command, path, and flag completion
6. **Signal handling** - Test Ctrl+C and Ctrl+D
7. **Logging** - Run with `-v` and check log files

### Expected Behavior

**This should work:**
```bash
$ ls -la
$ pwd
$ echo "hello world"        # ✅ Quotes now work!
$ echo 'single quotes'      # ✅ Single quotes too!
$ git commit -m "message"   # ✅ Complex quoted args
$ date
$ whoami
$ git status
$ git s→                    # ✅ Autosuggestions work! (→ = Right Arrow)
$ gi<TAB>                   # ✅ Tab completion works!
$ git --ver<TAB>            # ✅ Flag completion too!
```

**This won't work (yet):**
```bash
$ ls *.txt           # No globbing
$ ls | grep foo      # No pipes
$ echo x > file      # No redirections
$ sleep 10 &         # No background jobs
```

## 🚀 Contributing

Found a bug or limitation not listed here? Please report it!

Include:
- Rush version (`rush --version`)
- OS and version
- Steps to reproduce
- Expected vs. actual behavior
- Logs (run with `rush -vv` to capture)

---

**Questions?** Check [README.md](README.md) for installation and usage, or [CLI.md](CLI.md) for command-line options.
