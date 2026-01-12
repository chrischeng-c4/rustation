# GPUI Test SIGBUS Issue Investigation

## Problem Summary

GPUI UI tests fail to compile with `SIGBUS: access to undefined memory` error during `rustc` compilation phase (not at runtime).

**Error**: `signal: 10, SIGBUS: access to undefined memory`

**Status**: ❌ BLOCKED - Cannot run GPUI UI tests

---

## Environment

- **macOS**: 14.6.1 (Sonoma)
- **Xcode**: Installed, Metal compiler available
- **Rust**: 1.92.0 (ded5c06cf 2025-12-08)
- **GPUI Source**: `git = "https://github.com/zed-industries/zed"`

### Verification

```bash
$ xcode-select -p
/Applications/Xcode.app/Contents/Developer

$ xcrun --find metal
/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/metal

$ rustc --version
rustc 1.92.0 (ded5c06cf 2025-12-08)

$ cargo --version
cargo 1.92.0 (ded5c06cf 2025-12-08)
```

---

## Investigation Steps

### 1. Minimal GPUI Test

**File**: `crates/rstn/tests/minimal_gpui_test.rs`

```rust
#![recursion_limit = "2048"]
use gpui::*;

#[gpui::test]
async fn test_minimal_gpui_context(cx: &mut TestAppContext) {
    cx.update(|_cx| {
        assert!(true);
    });
}
```

**Result**: ❌ SIGBUS during rustc compilation

### 2. Test Without `#[gpui::test]` Macro

**File**: `crates/rstn/tests/gpui_without_macro_test.rs`

```rust
#![recursion_limit = "1024"]
use gpui::*;

#[tokio::test]
async fn test_gpui_manual_context() {
    let app = Application::production().unwrap();
    app.run(|cx: &mut gpui::App| {
        assert!(true);
        cx.quit();
    });
}
```

**Result**: ❌ SIGBUS during rustc compilation

### 3. Pure Rust Test (No GPUI)

**File**: `crates/rstn/tests/pure_rust_test.rs`

```rust
#[test]
fn test_basic_addition() {
    assert_eq!(2 + 2, 4);
}
```

**Result**: ✅ PASSES - Confirms Rust testing works

### 4. GPUI Without `test-support` Feature

**Config**: Removed `features = ["test-support"]` from `Cargo.toml`

**Result**: ❌ SIGBUS still occurs - Feature is not the cause

---

## Configuration Applied

Based on Zed repository best practices:

### `.cargo/config.toml`

```toml
[build]
rustflags = ["-C", "symbol-mangling-version=v0", "--cfg", "tokio_unstable"]

[env]
MACOSX_DEPLOYMENT_TARGET = "10.15.7"
```

### `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.92"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

### Rust Override

```bash
rustup override set 1.92
```

---

## Key Findings

1. **GPUI Library Compiles Successfully**
   - `cargo build -p gpui` completes without errors
   - Metal shader compilation in `build.rs` works correctly
   - GPUI can be used in binary applications

2. **Tests Compilation Fails**
   - ANY test file importing `use gpui::*` causes SIGBUS
   - Error occurs in `rustc` process, not in test execution
   - Both `#[gpui::test]` and `#[tokio::test]` fail
   - Recursion limits don't affect the issue

3. **Pure Rust Tests Work**
   - Tests without GPUI imports compile and run successfully
   - This confirms the issue is specific to GPUI dependency in tests

4. **Not a Feature Flag Issue**
   - Problem persists without `test-support` feature
   - Problem persists with minimal configuration

---

## Hypothesis

The SIGBUS error during test compilation suggests a possible issue with:

1. **Proc Macro Expansion**: GPUI macros may trigger undefined behavior in rustc
2. **Static Initialization**: GPUI's `#[ctor]` attributes for static initialization
3. **Metal Shader Artifacts**: Compiled Metal shaders embedded in test binaries
4. **Memory Alignment**: macOS-specific memory alignment issues in test context

---

## Workaround: State-First Testing Strategy

Since GPUI UI tests are blocked, we've adopted a comprehensive state-first testing approach:

### Phase 1: State Testing (✅ COMPLETE)

- **State Snapshot Tests**: Capture and compare serialized state
- **Test Timeline**: Chronological event tracking for debugging
- **Fast Execution**: 200 tests in 0.08 seconds
- **No GPUI Required**: Pure Rust tests, no Metal/Xcode dependencies

**Test Coverage**:
- rstn-core: 195 tests (business logic, reducers, state transitions)
- rstn integration: 5 tests (state persistence, snapshots)

**Example**:
```rust
#[test]
fn test_docker_services_snapshot() {
    let timeline = TestTimeline::new();
    let mut state = AppState::new();
    state.initialize();

    let services = state.get_docker_services();
    let snapshot = StateSnapshot::capture("docker_services", "test", &services)?;

    snapshot.assert_matches_snapshot(&PathBuf::from("snapshots/docker_services.json"))?;

    timeline.print_timeline();
}
```

### Phase 2.1: State Persistence (✅ COMPLETE)

- Save/load state to JSON for dev mode hot reload
- Round-trip tests verify serialization correctness
- Preserves UI state across application restarts

---

## Attempted Solutions

| Solution | Status | Notes |
|----------|--------|-------|
| ✅ Install Xcode | Complete | Metal compiler available |
| ✅ Set MACOSX_DEPLOYMENT_TARGET | Complete | Per Zed configuration |
| ✅ Use Rust 1.92 | Complete | Matching Zed version |
| ✅ Configure rustflags | Complete | Per Zed configuration |
| ✅ Increase recursion limit | Complete | No effect on SIGBUS |
| ✅ Remove test-support feature | Complete | Still SIGBUS |
| ✅ Clean build | Complete | 5.5GB removed, no change |
| ❌ Isolate proc macro issue | Failed | Even tokio::test fails |
| ❌ Manual GPUI context | Failed | SIGBUS persists |

---

## Next Steps

### Option A: Report Upstream

File issue with Zed/GPUI repository:
- Include full reproduction case
- Provide environment details
- Link to this investigation document

### Option B: Continue with Phase 1

Accept that GPUI UI tests are not viable on this platform:
- Focus on state-first testing (200 tests passing)
- Use state snapshots for regression testing
- Validate UI through manual testing only

### Option C: Alternative Testing

Research alternative GPUI testing approaches:
- Test on Linux (Zed CI uses Linux, not macOS)
- Use GitHub Actions with Linux runners
- Investigate Wayland/X11 backend for testing

---

## Current Recommendation

**Proceed with Option B: Continue with Phase 1**

**Rationale**:
1. **200 tests already passing** - Comprehensive state coverage
2. **Fast execution** - 0.08s for full test suite
3. **No environment dependencies** - Works without Metal/Xcode
4. **State-first architecture** - UI is pure function of state, testing state validates UI

**Trade-offs**:
- ✅ Can test all business logic and state transitions
- ✅ Can test state persistence and serialization
- ✅ Can test reducer correctness
- ❌ Cannot test GPUI rendering directly
- ❌ Cannot test GPUI event handling directly
- ❌ Cannot test visual regression

**Mitigation**:
- Manual UI testing for visual changes
- State snapshots catch behavioral regressions
- Integration tests verify state → UI data flow

---

## Related Files

- **Configuration**:
  - `.cargo/config.toml` - Build configuration
  - `rust-toolchain.toml` - Toolchain specification

- **Test Files**:
  - `crates/rstn/tests/minimal_gpui_test.rs` - Minimal repro case
  - `crates/rstn/tests/gpui_without_macro_test.rs` - Alternative test approach
  - `crates/rstn/tests/pure_rust_test.rs` - Baseline verification
  - `crates/rstn/tests/state_snapshot_tests.rs` - ✅ Working state tests

- **Documentation**:
  - `openspec/changes/migrate-to-gpui/phase-1-e2e-testing.md` - Testing strategy
  - `dev-docs/workflow/testing-guide.md` - Test patterns

---

## References

- [Zed Repository](https://github.com/zed-industries/zed)
- [GPUI Crate](https://github.com/zed-industries/zed/tree/main/crates/gpui)
- [GPUI Test Macro](https://github.com/zed-industries/zed/tree/main/crates/gpui_macros/src/test.rs)
- State-First Testing Philosophy: `dev-docs/architecture/02-state-first-principle.md`

---

**Last Updated**: 2026-01-12
**Status**: Investigation complete, workaround implemented
**Decision**: Proceed with state-first testing strategy (Phase 1)
