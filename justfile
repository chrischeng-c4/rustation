# rustation - GPUI Native Desktop App

# Build the GPUI application
build:
    cargo build --workspace

# Run the GPUI application
dev:
    cargo run -p rstn

# Run the GPUI application (release mode)
run:
    cargo run -p rstn --release

# Run all Rust tests
test:
    cargo test --workspace

# Run Rust unit tests only
test-unit:
    cargo test --workspace --lib

# Run clippy linter
lint:
    cargo clippy --workspace -- -D warnings

# Format all Rust code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Build release binary
build-release:
    cargo build --workspace --release

# Clean build artifacts
clean:
    cargo clean

# Install rstn to ~/.local/bin
install: build-release
    mkdir -p ~/.local/bin
    cp target/release/rstn ~/.local/bin/rstn
    @echo "Installed rstn to ~/.local/bin/rstn"
    @echo "Usage: rstn  # Open current directory"

# Build and run in one command
dev-build:
    cargo build -p rstn && cargo run -p rstn

# Watch for changes and rebuild (requires cargo-watch)
watch:
    cargo watch -x 'build -p rstn' -x 'run -p rstn'
