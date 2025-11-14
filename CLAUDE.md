# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

This is a Rust monorepo workspace called `rust-station` that contains multiple Rust projects. The primary project is **rush**, a shell implementation written in Rust designed to replace traditional shells like zsh, bash, and fish.

## Workspace Structure

```
rust-station/
├── Cargo.toml          # Workspace root configuration
├── crates/             # All projects live here
│   └── rush/          # Shell implementation
└── target/            # Shared build output (gitignored)
```

The workspace uses Cargo's workspace feature with `resolver = "2"`. All projects are organized under `crates/` and share common workspace-level configurations.

## Common Commands

### Building

```bash
# Build all workspace members
cargo build

# Build in release mode
cargo build --release

# Build a specific project
cargo build -p rush

# Build and run rush
cargo run -p rush
```

### Testing

```bash
# Run all tests in the workspace
cargo test

# Run tests for a specific project
cargo test -p rush

# Run a specific test
cargo test -p rush test_name
```

### Linting and Formatting

```bash
# Check code with clippy
cargo clippy --all-targets --all-features

# Format all code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

### Working with Dependencies

```bash
# Add a workspace-level dependency (edit Cargo.toml [workspace.dependencies])
# Then reference it in a crate's Cargo.toml with:
# dependency-name.workspace = true

# Add a project-specific dependency
cd crates/rush
cargo add <dependency-name>
```

### Cleaning

```bash
# Clean all build artifacts
cargo clean
```

## Workspace Configuration

The root `Cargo.toml` defines workspace-level settings that all member crates inherit:
- **version**: 0.1.0
- **edition**: 2021
- **resolver**: Version 2 (newer dependency resolver)

Common dependencies available to all workspace members are defined in `[workspace.dependencies]` including:
- tokio (async runtime)
- serde/serde_json (serialization)
- anyhow/thiserror (error handling)
- tracing/tracing-subscriber (logging)

## Adding New Projects to the Workspace

New projects are automatically included via the `members = ["crates/*"]` glob pattern:

```bash
cd crates
cargo new --bin project-name    # For a binary
cargo new --lib project-name    # For a library
```

The new project will automatically become part of the workspace.

## Rush Shell Project

Located in `crates/rush/`, this is a shell implementation being developed as an alternative to traditional Unix shells. It's a binary project with its entry point at `crates/rush/src/main.rs`.
