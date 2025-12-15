# Build release binary
build:
    cargo build --release

# Build debug binary
build-debug:
    cargo build

# Install release builds to ~/.local/bin
install: build
    mkdir -p ~/.local/bin
    cp target/release/rstn ~/.local/bin/
    cp target/release/rush ~/.local/bin/
    @echo "Installed RELEASE builds to ~/.local/bin"

# Link debug builds to ~/.local/bin (hot reload: just rebuild to update)
install-dev: build-debug
    mkdir -p ~/.local/bin
    ln -sf {{justfile_directory()}}/target/debug/rstn ~/.local/bin/rstn
    ln -sf {{justfile_directory()}}/target/debug/rush ~/.local/bin/rush
    @echo "Linked DEBUG builds to ~/.local/bin (hot reload enabled)"

# Quick link just rstn debug for fast iteration
install-rstn-dev:
    cargo build -p rstn
    mkdir -p ~/.local/bin
    ln -sf {{justfile_directory()}}/target/debug/rstn ~/.local/bin/rstn
    @echo "Linked DEBUG rstn to ~/.local/bin (hot reload enabled)"

# Check which build type is currently installed
which-build:
    #!/usr/bin/env bash
    echo "Checking installed binaries in ~/.local/bin..."
    rstn_type=$([ -L ~/.local/bin/rstn ] && echo "symlink" || echo "binary")
    rstn_ver=$(~/.local/bin/rstn --version 2>/dev/null | grep -o '\[.*\]' || echo "not installed")
    echo "rstn: [$rstn_type] $rstn_ver"
    rush_type=$([ -L ~/.local/bin/rush ] && echo "symlink" || echo "binary")
    rush_ver=$(~/.local/bin/rush --version 2>/dev/null | grep -o '\[.*\]' || echo "not installed")
    echo "rush: [$rush_type] $rush_ver"

# Build and install (alias for backward compatibility)
all: install-dev
