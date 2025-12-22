# rstn v2 - Python CLI

# Install to ~/.local/bin
install:
    uv pip install -e ".[dev]"
    mkdir -p ~/.local/bin
    ln -sf {{justfile_directory()}}/.venv/bin/rstn ~/.local/bin/rstn
    @echo "Installed rstn to ~/.local/bin/rstn"

# Run tests
test:
    uv run pytest tests/ -v

# Run tests with coverage
test-cov:
    uv run pytest tests/ --cov=rstn --cov-report=term-missing

# Check code quality (ruff + mypy)
check:
    uv run ruff check rstn/ tests/
    uv run mypy rstn/ --ignore-missing-imports

# Fix lint issues
fix:
    uv run ruff check --fix rstn/ tests/
