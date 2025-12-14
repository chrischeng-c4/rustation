# Research: Release Channels

**Feature**: 047-release-channels
**Date**: 2025-12-14

## Research Tasks

### 1. Debug vs Release Build Detection in Rust

**Question**: How to detect build type at compile time and runtime?

**Decision**: Use `#[cfg(debug_assertions)]` and `env!("BUILD_PROFILE")`

**Rationale**:
- `#[cfg(debug_assertions)]` is the standard Rust idiom for debug-only code
- `debug_assertions` is true for `cargo build` and false for `cargo build --release`
- `BUILD_PROFILE` env var can be set in `build.rs` using `PROFILE` env var from Cargo

**Alternatives Considered**:
| Alternative | Why Rejected |
|-------------|--------------|
| Feature flags | Overkill for debug/release detection; adds complexity |
| Runtime env vars | Determined at runtime, not compile time; less reliable |
| Custom build script detection | More complex than necessary |

**Implementation**:
```rust
// In build.rs
println!("cargo:rustc-env=BUILD_PROFILE={}",
    std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()));

// In code
#[cfg(debug_assertions)]
fn default_log_level() -> String { "trace".to_string() }

#[cfg(not(debug_assertions))]
fn default_log_level() -> String { "info".to_string() }
```

---

### 2. Homebrew Formula Best Practices

**Question**: How to create a proper Homebrew formula for Rust projects?

**Decision**: Use `depends_on "rust" => :build` and build from source

**Rationale**:
- Standard practice for Rust projects in Homebrew
- Builds on user's machine, ensuring architecture compatibility
- No need to maintain pre-compiled binaries
- Formula tests verify installation works

**Alternatives Considered**:
| Alternative | Why Rejected |
|-------------|--------------|
| Pre-compiled bottles | Requires CI infrastructure for cross-compilation; more maintenance |
| Cargo binstall | Less familiar to users; not standard Homebrew workflow |
| GitHub releases only | Missing Homebrew's update/uninstall benefits |

**Implementation**:
```ruby
class Rustation < Formula
  desc "Modern shell and development toolkit written in Rust"
  homepage "https://github.com/chrischeng-c4/rustation"
  url "https://github.com/chrischeng-c4/rustation.git", tag: "v0.35.0"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--locked"
    bin.install "target/release/rush"
    bin.install "target/release/rstn"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rush --version")
  end
end
```

---

### 3. Just Task Runner Recipes

**Question**: What's the best pattern for install-dev vs install recipes?

**Decision**: Separate recipes with clear naming and output messages

**Rationale**:
- `just install` = release (existing behavior, unchanged)
- `just install-dev` = debug (new, explicit opt-in)
- `just which-build` = diagnostic (helpful for verification)
- Clear echo messages confirm what was installed

**Alternatives Considered**:
| Alternative | Why Rejected |
|-------------|--------------|
| Single recipe with flag | Less discoverable; users might not know flags exist |
| Environment variable | Less explicit; easy to install wrong build accidentally |
| Separate script files | Unnecessary complexity for simple commands |

**Implementation**:
```just
# Build and install debug builds for development
install-dev:
    cargo build
    mkdir -p ~/.local/bin
    cp target/debug/rstn ~/.local/bin/
    cp target/debug/rush ~/.local/bin/
    @echo "Installed DEBUG builds to ~/.local/bin"

# Check which build type is installed
which-build:
    @file ~/.local/bin/rstn 2>/dev/null | grep -q "not stripped" && echo "rstn: DEBUG" || echo "rstn: RELEASE"
    @file ~/.local/bin/rush 2>/dev/null | grep -q "not stripped" && echo "rush: DEBUG" || echo "rush: RELEASE"
```

---

### 4. Version Display Format

**Question**: How should version output differentiate debug vs release?

**Decision**: Append build type in parentheses: `0.1.0 (debug)` or `0.1.0 (release)`

**Rationale**:
- Clear, unambiguous identification
- Follows common patterns (git hash in parentheses)
- Doesn't break version parsing for tools expecting semver

**Alternatives Considered**:
| Alternative | Why Rejected |
|-------------|--------------|
| Separate flag | Requires user to remember extra flag |
| Debug suffix in version | Breaks semver parsing |
| Emoji indicators | Not professional; terminal compatibility issues |

**Implementation**:
```rust
pub fn build_info() -> String {
    format!("{} ({})", env!("CARGO_PKG_VERSION"), env!("BUILD_PROFILE"))
}
// Output: "0.1.0 (debug)" or "0.1.0 (release)"
```

---

## Summary

All research tasks resolved. No clarifications needed from user.

| Topic | Decision | Confidence |
|-------|----------|------------|
| Build detection | `#[cfg(debug_assertions)]` + `BUILD_PROFILE` env | High |
| Homebrew formula | Build from source with `rust` dependency | High |
| Just recipes | Separate `install-dev`, `install`, `which-build` | High |
| Version format | `version (profile)` pattern | High |

**Ready for Phase 1: Design & Contracts**
