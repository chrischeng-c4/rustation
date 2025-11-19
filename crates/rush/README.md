# Rush Shell

A modern, fast, fish-like shell written in Rust with real-time syntax highlighting, intelligent autosuggestions, and persistent command history.

## ⚠️ Alpha Status (v0.1.0)

Rush is currently in **alpha testing**. Core features work, but some functionality is still in development. See [KNOWN_ISSUES.md](KNOWN_ISSUES.md) for current limitations and planned features.

**Alpha testers welcome!** Please report issues and feedback.

## Features

### ✅ Implemented (v0.1.0)
- **Interactive REPL** - Read-eval-print loop with line editing
- **Command execution** - Run external commands with argument passing
- **Syntax highlighting** - Real-time color coding as you type
- **Command history** - Persistent history with navigation (↑/↓ arrows)
- **Autosuggestions** - Fish-like suggestions from command history
  - Inline grayed-out suggestions as you type
  - Accept full suggestion with Right Arrow (→)
  - Accept word-by-word with Alt+Right Arrow (⌥→)
- **Tab completion** - Intelligent completion for commands, paths, and flags
  - Command names from PATH
  - File and directory paths
  - Flags for common commands (git, cargo, ls, grep, cat, find, etc.)
- **Pipe operator** - Unix-style command composition with `|`
  - Chain commands together: `ls | grep txt`
  - Multi-command pipelines: `cat file | grep error | wc -l`
  - Binary-safe data flow with concurrent execution
  - ~0.5μs parsing, ~2-4ms overhead
- **Exit code tracking** - Track and display last command status
- **Signal handling** - Ctrl+C (cancel), Ctrl+D (exit)
- **Comprehensive CLI** - Verbose logging, config inspection, health checks

### 🚧 Planned (v0.2.0+)
- Output redirections (`>`, `>>`, `2>`)
- Job control & background execution (`&`, `fg`, `bg`)
- Configuration file support
- Custom prompts

See [KNOWN_ISSUES.md](KNOWN_ISSUES.md) for details.

## Quick Start

### Prerequisites

- macOS or Linux
- Rust toolchain (for building from source)

### Installation

#### Option 1: Install from Source (Recommended for Alpha)

```bash
# Clone the repository
git clone <repository-url>
cd rust-station

# Build release binary
cargo build --release -p rush

# Install to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/rush ~/.local/bin/

# Add to PATH (add this to your ~/.zshrc or ~/.bashrc)
export PATH="$HOME/.local/bin:$PATH"

# Reload shell configuration
source ~/.zshrc  # or source ~/.bashrc
```

#### Option 2: Build and Run Directly

```bash
# From workspace root
cargo run -p rush --release

# Or from crates/rush directory
cd crates/rush
cargo run --release
```

### First Steps

1. **Launch rush:**
   ```bash
   rush
   ```

2. **Try basic commands:**
   ```bash
   $ ls -la
   $ pwd
   $ echo "Hello from rush!"
   $ date
   ```

3. **Test history navigation:**
   - Press ↑ to see previous commands
   - Press ↓ to navigate forward
   - Type and see syntax highlighting

4. **Try autosuggestions:**
   ```bash
   $ git s          # See 'tatus' in gray (if 'git status' is in history)
   $ git s→         # Press Right Arrow to accept → 'git status'
   $ git commit -m "⌥→  # Press Alt+Right to accept word-by-word
   ```

   - Suggestions appear in dimmed text as you type
   - Based on your command history (most recent matches first)
   - Right Arrow (→) accepts the full suggestion
   - Alt+Right Arrow (⌥→) accepts one word at a time

5. **Try tab completion:**
   ```bash
   $ gi<TAB>        # Completes to git, gist, etc.
   $ ls ./s<TAB>    # Completes to ./src/, ./specs/, etc.
   $ git --ver<TAB> # Completes to --version, --verbose
   $ ls -<TAB>      # Shows available ls flags
   ```

6. **Exit:**
   ```bash
   $ exit
   # or press Ctrl+D
   ```

### Alpha Testing with Verbose Logging

For alpha testing, enable detailed logging to help diagnose issues:

```bash
# Start with debug logging
rush -v

# Or maximum verbosity (trace level)
rush -vv
```

**Log location (macOS):**
```
~/Library/Application Support/rush/rush-v0.1.0.log
```

**Log location (Linux):**
```
~/.local/share/rush/rush-v0.1.0.log
```

Review logs:
```bash
# macOS
tail -f ~/Library/Application\ Support/rush/rush-v0.1.0.log

# Linux
tail -f ~/.local/share/rush/rush-v0.1.0.log
```

## Usage

### Command-Line Options

```bash
rush --help          # Show all options
rush --version       # Show version
rush --doctor        # Run health check
rush --dump-config   # Show configuration
rush -v              # Debug logging
rush -vv             # Trace logging (very detailed)
```

For complete CLI reference, see [CLI.md](CLI.md).

### Health Check

Verify your rush installation:

```bash
rush --doctor
```

This checks:
- Version information
- Config file status
- History directory permissions
- File paths

### Configuration Inspection

See the resolved configuration rush is using:

```bash
rush --dump-config
```

Shows:
- Active configuration values
- CLI flags in effect
- File paths (config, logs, history)

## Alpha Testing

Rush v0.1.0 is in alpha testing. Help improve rush by using it daily and reporting issues!

### Quick Start Testing

**Helper script (easiest):**
```bash
rush-test       # Start with debug logging
rush-test logs  # View recent logs
rush-test -vv   # Start with trace logging
```

**Manual:**
```bash
# Start rush with file logging (recommended)
rush -v

# Or with console logging too
rush -vv

# Check logs after
# macOS:
tail ~/Library/Application\ Support/rush/rush-v0.1.0.log

# Linux:
tail ~/.local/share/rush/rush-v0.1.0.log
```

### Testing Workflow

See [DAILY_TESTING.md](../../DAILY_TESTING.md) in the project root for:
- Daily testing template
- Bug report template
- Feature request template
- Comprehensive testing checklist

**Daily testing (15-20 minutes):**
1. Launch `rush -v` in a separate terminal
2. Use rush for your normal commands
3. Note what works, what doesn't, what's missing
4. Log findings in DAILY_TESTING.md

**What to test:**
- Basic commands (`ls`, `cd`, `pwd`, `echo`)
- Git workflows (`status`, `log`, `diff`, `commit`)
- History navigation (↑/↓ arrows)
- Quote handling (`echo "hello world"`)
- Syntax highlighting
- Signal handling (Ctrl+C, Ctrl+D)

**What won't work yet:**
- Pipes (`ls | grep`)
- Redirections (`echo x > file`)
- Autosuggestions
- Globbing (`ls *.txt`)

### Reporting Issues

Found a bug or missing feature? Check [DAILY_TESTING.md](../../DAILY_TESTING.md) for templates, then:

1. Run diagnostics:
   ```bash
   rush --doctor
   rush --dump-config
   ```

2. Capture logs:
   ```bash
   rush -vv  # Enable trace logging
   # Reproduce issue
   # Exit and attach log file
   ```

3. Create GitHub issue with:
   - Steps to reproduce
   - Expected vs actual behavior
   - Logs and diagnostic output
   - OS and rush version

## Project Structure

This project is part of the `rust-station` monorepo workspace:

```
rust-station/
├── Cargo.toml           # Workspace configuration
├── crates/
│   └── rush/           # Rush shell implementation
│       ├── src/
│       │   ├── main.rs
│       │   ├── repl/   # Interactive loop
│       │   ├── executor/ # Command execution
│       │   ├── config/ # Configuration
│       │   └── ...
│       ├── tests/
│       └── Cargo.toml
└── target/             # Build output
```

## Development

### Building

```bash
# Debug build
cargo build -p rush

# Release build (optimized)
cargo build -p rush --release

# From crates/rush directory
cargo build
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test -p rush

# Run with output
cargo test -p rush -- --nocapture

# Run specific test
cargo test -p rush test_name
```

**Test coverage:** See [TEST_COVERAGE.md](TEST_COVERAGE.md) for detailed test statistics.

Current: **107 tests passing**, 73.37% code coverage.

### Linting

```bash
# Check with clippy
cargo clippy -p rush --all-targets --all-features

# Format code
cargo fmt -p rush

# Check formatting
cargo fmt -p rush -- --check
```

## Reporting Issues

As an alpha tester, your feedback is valuable! When reporting issues:

1. **Run health check:**
   ```bash
   rush --doctor
   rush --dump-config
   ```

2. **Capture logs:**
   ```bash
   rush -vv  # Enable trace logging
   # Reproduce the issue
   # Exit rush
   # Attach log file to report
   ```

3. **Include:**
   - Rush version (`rush --version`)
   - OS and version
   - Steps to reproduce
   - Expected vs. actual behavior
   - Log file (if applicable)

## Documentation

- **[CLI.md](CLI.md)** - Complete command-line arguments reference
- **[KNOWN_ISSUES.md](KNOWN_ISSUES.md)** - Current limitations and roadmap
- **[TEST_COVERAGE.md](TEST_COVERAGE.md)** - Test statistics and coverage

## License

MIT OR Apache-2.0

## Acknowledgments

Built with:
- [reedline](https://github.com/nushell/reedline) - Line editing library
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing
- [tracing](https://github.com/tokio-rs/tracing) - Structured logging

---

**Version:** 0.1.0 (Alpha)
**Last Updated:** 2025-11-15
