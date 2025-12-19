# Installation Guide

**Last Updated**: 2025-12-19
**Estimated Time**: 5 minutes

This guide will help you install rustation v2 on your system.

---

## Prerequisites

### Required

- **Rust 1.75+** (edition 2021)
  ```bash
  # Check your Rust version
  rustc --version

  # Should output: rustc 1.75.0 or higher
  ```

- **cargo** (comes with Rust)
  ```bash
  # Check cargo version
  cargo --version
  ```

### Install Rust (if needed)

If you don't have Rust installed:

```bash
# Install via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then restart your terminal

# Verify installation
rustc --version
cargo --version
```

---

## Installation Methods

### Method 1: Install from Source (Recommended)

**Clone the repository:**
```bash
git clone https://github.com/chrischeng-c4/rustation.git
cd rustation
```

**Install rstn (TUI):**
```bash
cargo install --path crates/rstn
```

This installs the `rstn` binary to `~/.cargo/bin/rstn`.

**Install rush (Shell) - Optional:**
```bash
cargo install --path crates/rush
```

This installs the `rush` binary to `~/.cargo/bin/rush`.

**Verify installation:**
```bash
# Check rstn
rstn --version
# Should output: rstn 0.x.x

# Check rush (if installed)
rush --version
# Should output: rush 0.x.x
```

---

### Method 2: Build without Installing

If you just want to try rstn without installing:

```bash
# Clone repository
git clone https://github.com/chrischeng-c4/rustation.git
cd rustation

# Run directly
cargo run -p rstn

# Or for rush
cargo run -p rush
```

---

## Post-Installation Setup

### 1. Verify Binary Location

Make sure `~/.cargo/bin` is in your PATH:

```bash
echo $PATH | grep -q ".cargo/bin" && echo "✅ cargo bin in PATH" || echo "❌ Need to add to PATH"
```

If not in PATH, add to your shell config:

```bash
# For bash (~/.bashrc or ~/.bash_profile)
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# For zsh (~/.zshrc)
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc

# For fish (~/.config/fish/config.fish)
echo 'set -gx PATH $HOME/.cargo/bin $PATH' >> ~/.config/fish/config.fish

# Restart terminal or source the file
source ~/.bashrc  # or ~/.zshrc, etc.
```

### 2. Test Installation

```bash
# Run rstn TUI
rstn

# You should see the TUI interface
# Press 'q' to quit
```

### 3. Optional: Claude Code Integration

If you use Claude Code and want MCP integration:

```bash
# rstn will automatically create ~/.rstn/mcp-session.json
# when the MCP server starts

# To use with Claude CLI:
claude --mcp-config ~/.rstn/mcp-session.json
```

See [MCP Tools Reference](../03-api-reference/mcp-tools.md) for details.

---

## Troubleshooting

### Issue: `rstn: command not found`

**Cause**: `~/.cargo/bin` not in PATH

**Solution**: Add to PATH (see Post-Installation Setup above)

### Issue: `cargo install` fails

**Cause**: Missing dependencies or outdated Rust

**Solution**:
```bash
# Update Rust
rustup update

# Try again
cargo install --path crates/rstn
```

### Issue: Compilation errors

**Cause**: Incompatible Rust version

**Solution**:
```bash
# Check Rust version
rustc --version

# Must be 1.75.0 or higher
# Update if needed
rustup update
```

### Issue: Permission denied

**Cause**: Cargo bin directory not writable

**Solution**:
```bash
# Check permissions
ls -la ~/.cargo/bin

# Fix if needed (be careful!)
chmod u+w ~/.cargo/bin
```

---

## Uninstallation

To remove rustation:

```bash
# Uninstall rstn
cargo uninstall rstn

# Uninstall rush (if installed)
cargo uninstall rush

# Remove configuration (optional)
rm -rf ~/.rstn
rm -rf ~/.rustation
```

---

## Next Steps

1. **Quick Start**: [Run your first rstn session](quick-start.md)
2. **Concepts**: [Understand rustation v2 architecture](concepts.md)
3. **Reference**: [MCP Tools](../03-api-reference/mcp-tools.md)

---

## Need Help?

- Check [Quick Start Guide](quick-start.md)
- Read [Concepts](concepts.md)
- Open an issue: [GitHub Issues](https://github.com/chrischeng-c4/rustation/issues)

---

## System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| Rust | 1.75.0 | Latest stable |
| Memory | 100 MB | 500 MB |
| Disk Space | 50 MB | 200 MB |
| OS | macOS, Linux | macOS, Linux |
| Terminal | Any | Modern with mouse support |

**Note**: Windows support is experimental and not actively tested.

---

## Changelog

- 2025-12-19: Initial installation guide for v2
