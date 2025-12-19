# Quickstart: Release Channels

**Feature**: 047-release-channels
**Date**: 2025-12-14

## For Developers (Local Debug Builds)

### Install Debug Builds

```bash
# Clone and enter repository
cd ~/projects/rustation

# Build and install debug versions with trace logging
just install-dev

# Verify installation
~/.local/bin/rstn --version
# Output: rstn 0.1.0 (debug)

# Check which build type is installed
just which-build
# Output: rstn: DEBUG
#         rush: DEBUG
```

### Quick Iteration

```bash
# Rebuild just rstn for faster iteration
just install-rstn-dev

# Logs automatically go to ~/.rustation/logs/rstn.log at trace level
```

### Switch Back to Release

```bash
# Install optimized release builds
just install

# Verify
just which-build
# Output: rstn: RELEASE
#         rush: RELEASE
```

---

## For End Users (Homebrew)

### First-Time Installation

```bash
# Add the tap
brew tap chrischeng-c4/rustation

# Install both rush and rstn
brew install rustation

# Verify installation
rush --version
# Output: rush 0.35.0 (release)

rstn --version
# Output: rstn 0.1.0 (release)
```

### Upgrading

```bash
brew update
brew upgrade rustation
```

### Uninstalling

```bash
brew uninstall rustation
brew untap chrischeng-c4/rustation
```

---

## Troubleshooting

### Check Build Type

```bash
# Using file command (checks if debug symbols present)
file ~/.local/bin/rstn | grep "not stripped"
# If matches: DEBUG build
# If no match: RELEASE build

# Or use just recipe
just which-build
```

### PATH Issues

If you have both `~/.local/bin` and Homebrew in PATH:

```bash
# Check which binary is being used
which rstn

# For development, ensure ~/.local/bin comes first
export PATH="$HOME/.local/bin:$PATH"
```

### Log Level Override

```bash
# Force trace logging on release build
RSTN_LOG=trace rstn --cli doctor

# Force quiet logging on debug build
RSTN_LOG=warn rstn --cli doctor
```

---

## Quick Reference

| Action | Command |
|--------|---------|
| Install debug builds | `just install-dev` |
| Install release builds | `just install` |
| Check build type | `just which-build` |
| Install via Homebrew | `brew tap chrischeng-c4/rustation && brew install rustation` |
| Show version + build | `rstn --version` |
