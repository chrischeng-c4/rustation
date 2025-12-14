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

# Install debug builds to ~/.local/bin (with trace logging)
install-dev: build-debug
    mkdir -p ~/.local/bin
    cp target/debug/rstn ~/.local/bin/
    cp target/debug/rush ~/.local/bin/
    @echo "Installed DEBUG builds to ~/.local/bin"

# Quick install just rstn debug for fast iteration
install-rstn-dev:
    cargo build -p rstn
    mkdir -p ~/.local/bin
    cp target/debug/rstn ~/.local/bin/
    @echo "Installed DEBUG rstn to ~/.local/bin"

# Check which build type is currently installed
which-build:
    @echo "Checking installed binaries in ~/.local/bin..."
    @printf "rstn: " && (~/.local/bin/rstn --version 2>/dev/null | grep -o '\[.*\]' || echo "not installed")
    @printf "rush: " && (~/.local/bin/rush --version 2>/dev/null | grep -o '\[.*\]' || echo "version info not available")

# Build and install (alias for backward compatibility)
all: install
