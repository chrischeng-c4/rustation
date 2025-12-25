# rustation - Electron Desktop App

# Run Electron dev server
dev:
    cd apps/desktop && pnpm dev

# Build Electron app
build:
    cd apps/desktop && pnpm build

# Run all tests (unit + e2e)
test: test-rust test-e2e
    @echo "All tests passed!"

# Run Rust unit tests
test-rust:
    cargo test

# Run e2e tests
test-e2e:
    cd apps/desktop && pnpm test:e2e

# Build napi-rs module
build-core:
    cd packages/core && pnpm build

# Build distributable app (.app bundle for macOS)
build-app: build-core build
    cd apps/desktop && pnpm build:mac

# Install rstn CLI to ~/.local/bin
install: build-app
    mkdir -p ~/.local/bin
    ln -sf {{justfile_directory()}}/apps/desktop/bin/rstn ~/.local/bin/rstn
    @echo "Installed rstn to ~/.local/bin/rstn"
    @echo "Usage: rstn .  # Open current directory"

# ============================================================================
# Legacy Python CLI (rstn v2)
# ============================================================================

# Install Python CLI to ~/.local/bin
install-py:
    uv pip install -e ".[dev]"
    mkdir -p ~/.local/bin
    ln -sf {{justfile_directory()}}/.venv/bin/rstn ~/.local/bin/rstn
    @echo "Installed rstn to ~/.local/bin/rstn"

# Run Python tests
test-py:
    uv run pytest tests/ -v

# Run Python tests with coverage
test-py-cov:
    uv run pytest tests/ --cov=rstn --cov-report=term-missing

# Check Python code quality (ruff + mypy)
check-py:
    uv run ruff check rstn/ tests/
    uv run mypy rstn/ --ignore-missing-imports

# Fix Python lint issues
fix-py:
    uv run ruff check --fix rstn/ tests/
