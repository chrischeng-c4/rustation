# Rush Shell - CLI Arguments Reference

## Quick Start

```bash
# Normal usage
rush

# Alpha testing with verbose logging
rush -v

# Extra verbose (trace level)
rush -vv

# Check configuration
rush --dump-config

# Health check
rush --doctor

# Get help
rush --help
```

## Command-Line Arguments

### Logging & Debugging

#### `-v, --verbose` (repeatable)
Increase logging verbosity for alpha testing and debugging.

- **No flag**: No logging (clean output)
- **`-v`**: Debug level - logs to file only (quiet console)
- **`-vv`**: Trace level - logs to both file AND console

**What gets logged:**
- REPL iterations, command execution, config loading
- Process details, exit codes, signals
- Debug/trace information based on level

**Log file location:** `~/.local/share/rush/rush-v{version}.log`

```bash
# Debug logging
rush -v

# Trace logging (very detailed)
rush -vv

# Check log file location
rush -v --dump-config
```

**What gets logged:**
- **Debug level (-v)**:
  - REPL loop iterations
  - User input received
  - Command parsing (program + args)
  - Process spawn and completion
  - Exit codes
  - Configuration loading
  - Ctrl+C / Ctrl+D signals

- **Trace level (-vv)**:
  - Everything from debug level
  - Waiting for user input
  - Empty line detection
  - Process IDs
  - Internal state transitions

#### `--log-file <path>`
Override default log file location (requires --verbose).

```bash
rush -v --log-file /tmp/rush-debug.log
```

#### `--log-format <format>`
Log format: `pretty` (default) or `json`.

Currently only `pretty` format is implemented. JSON format will be added in future releases.

```bash
rush -v --log-format pretty
```

#### `-q, --quiet`
Suppress all non-essential output. Conflicts with `--verbose`.

```bash
rush --quiet
```

### Configuration

#### `--config <path>`
Use a custom TOML configuration file.

```bash
rush --config ~/my-custom-rush.toml
```

**Note:** Custom config loading is not yet fully implemented in v0.1.0. Currently uses defaults.

#### `--no-config`
Ignore configuration file and use built-in defaults.

```bash
rush --no-config
```

#### `--history-size <n>`
Override the history size limit.

```bash
rush --history-size 50000
```

#### `--no-history`
Disable command history persistence (useful for privacy/temporary sessions).

```bash
rush --no-history
```

### Utilities

#### `--dump-config`
Print the resolved configuration and exit.

Shows:
- Version information
- Configuration values (history size, timeouts, etc.)
- CLI flags active
- File paths (config, log, history)

```bash
rush --dump-config

# With verbose to see log file path
rush -v --dump-config
```

**Example output:**
```
Rush Shell Configuration
========================

Version: 0.1.0

Configuration:
  history_size: 10000
  prompt: "$ "
  completion_timeout_ms: 100
  suggestion_delay_ms: 50

CLI Flags:
  verbose: 1
  quiet: false
  ...

Paths:
  config_file: ~/.config/rush/rush.toml
  log_file: ~/.local/share/rush/rush-v0.1.0.log
```

#### `--doctor`
Run health check and print diagnostics.

Verifies:
- Rush version
- Config file existence and validity
- History directory existence and writability
- File permissions

```bash
rush --doctor
```

**Example output:**
```
Rush Shell Health Check
=======================

✓ Version: 0.1.0
✓ Config file exists: ~/.config/rush/rush.toml
  └─ Loaded successfully
     history_size: 10000
✓ History directory exists: ~/.local/share/rush
✓ History directory is writable

✓ All checks passed!
```

### Single Command Execution

#### `-c, --command <cmd>`
Execute a single command and exit (like `bash -c`).

```bash
rush -c "echo hello world"
rush -c "git status"
rush -c 'ls -la | head -10'  # Note: pipes work within the command!
```

The command is executed and rush exits with the command's exit code.

### Feature Toggles

#### `--no-highlight`
Disable syntax highlighting.

```bash
rush --no-highlight
```

#### `--no-suggestions`
Disable autosuggestions.

```bash
rush --no-suggestions
```

#### `--no-completion`
Disable tab completion.

```bash
rush --no-completion
```

**Note:** These feature toggles are defined but not yet wired up in v0.1.0.

### Information

#### `-h, --help`
Display help message with all available options.

```bash
rush --help

# Short help
rush -h
```

#### `-V, --version`
Display version and exit.

```bash
rush --version
```

## Common Usage Patterns

### Alpha Testing Workflow

```bash
# Start rush with debug logging
rush -v

# Use rush normally, all actions are logged
> ls -la
> echo "testing"
> pwd

# Exit rush
> exit

# Review logs
tail -f ~/.local/share/rush/rush-v0.1.0.log
```

### Debugging Specific Issues

```bash
# Maximum verbosity for deep debugging
rush -vv --log-file /tmp/rush-trace.log

# Clean testing (no history/config)
rush --no-config --no-history -v

# Test configuration
rush --dump-config
rush --doctor
```

### Development Testing

```bash
# Test with custom settings
rush --history-size 1000 -v

# Quick command test
rush -c "git status"

# Minimal mode
rush --no-highlight --no-suggestions
```

## Log File Locations

### Default Locations

**macOS:**
- Config: `~/Library/Application Support/rush/rush.toml`
- History: `~/Library/Application Support/rush/history.txt`
- Logs: `~/Library/Application Support/rush/rush-v{version}.log`

**Linux:**
- Config: `~/.config/rush/rush.toml`
- History: `~/.local/share/rush/history.txt`
- Logs: `~/.local/share/rush/rush-v{version}.log`

### Log File Format

Logs are written in pretty format with:
- Timestamp
- Log level (INFO, DEBUG, TRACE, WARN, ERROR)
- Module path
- File and line number
- Structured fields
- Message

**Example log entries:**
```
[2025-11-14T15:30:31.750918Z]  INFO rush: crates/rush/src/main.rs:25: rush shell starting version="0.1.0"
[2025-11-14T15:30:31.751125Z] DEBUG rush::config::defaults: crates/rush/src/config/defaults.rs:33: Config file not found, using defaults
[2025-11-14T15:30:45.123456Z] DEBUG rush::repl: crates/rush/src/repl/mod.rs:100: User input received cmd="ls -la" iteration=1
[2025-11-14T15:30:45.124567Z] DEBUG rush::executor::execute: crates/rush/src/executor/execute.rs:45: Executing command program="ls" args=["-la"]
[2025-11-14T15:30:45.234567Z]  INFO rush::executor::execute: crates/rush/src/executor/execute.rs:67: Process completed program="ls" exit_code=0 pid=12345
```

## Environment Variables

Rush respects the `RUST_LOG` environment variable for advanced log filtering.

```bash
# Override log levels
RUST_LOG=rush=trace rush

# Filter specific modules
RUST_LOG=rush::executor=trace,rush::repl=debug,info rush

# Quiet everything except errors
RUST_LOG=error rush
```

**Note:** CLI flags (`-v`, `-vv`) take precedence over `RUST_LOG`.

## Exit Codes

- `0`: Success
- `1`: General error
- `127`: Command not found (from executed commands)

## Future Features (Not Yet Implemented)

The following flags are defined but functionality will be added in future releases:

- `--benchmark`: Run performance benchmarks
- `--trace-file <path>`: Performance tracing (Chrome trace format)
- `--config <path>`: Loading custom config files (currently ignored, uses defaults)
- Feature toggles (`--no-*`): Wiring to actual features
- `--history-size`: Override history size (flag exists but not wired)

## Troubleshooting

### Logs not appearing

1. Check you're using `-v` or `-vv` flag
2. Verify log file path with `rush -v --dump-config`
3. Check file permissions with `rush --doctor`

### Can't write to log file

```bash
# Check permissions
rush --doctor

# Use custom log location
rush -v --log-file ~/rush.log
```

### Too much logging

```bash
# Use -v instead of -vv
rush -v

# Or disable verbose mode
rush  # (no flags)
```

### Config file issues

```bash
# Check current config
rush --dump-config

# Bypass config
rush --no-config

# Health check
rush --doctor
```

## Contributing

If you find issues with CLI arguments or logging:

1. Capture logs: `rush -vv --log-file bug-report.log`
2. Include output of `rush --doctor`
3. Include output of `rush --dump-config`
4. Report issue with log file attached

---

**Version:** 0.1.0
**Last Updated:** 2025-11-14
