# Metal Toolchain Setup Guide

**Issue**: GPUI requires the Metal shader compiler to build on macOS
**Error**: `xcrun: error: unable to find utility "metal", not a developer tool or in PATH`
**Impact**: Cannot run `cargo run --bin rstn` or execute integration tests
**Status**: 🔴 **Blocker for running the application**

## Problem Analysis

### What is Metal?

Metal is Apple's GPU programming framework for macOS, iOS, and other Apple platforms. GPUI (Zed's UI framework) uses Metal for hardware-accelerated rendering.

### Why It's Failing

The `metal` shader compiler is **only available in full Xcode**, not in the standalone Command Line Tools package.

**Current system status**:
- ✅ Command Line Tools installed: `/Library/Developer/CommandLineTools`
- ✅ `xcrun` available: `/usr/bin/xcrun`
- ❌ `metal` compiler: **Not found**
- ❌ Full Xcode: **Not installed**

## Solution Options

### Option A: Install Full Xcode (Recommended)

**Best for**: Long-term GPUI development

1. **Download Xcode**:
   - Visit https://developer.apple.com/xcode/
   - Download Xcode 15.4 or later (compatible with macOS version)
   - **Note**: Xcode is ~14GB download, ~40GB installed

2. **Install Xcode**:
   ```bash
   # Open the downloaded .xip file
   # Drag Xcode.app to /Applications/

   # Set Xcode as the active developer directory
   sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer

   # Accept license
   sudo xcodebuild -license accept

   # Install additional components
   xcodebuild -runFirstLaunch
   ```

3. **Verify Metal is available**:
   ```bash
   xcrun --find metal
   # Should output: /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/metal
   ```

4. **Run rustation**:
   ```bash
   cd /Users/chris.cheng/chris-project/rustation
   RUST_LOG=info cargo run --bin rstn
   ```

**Time to resolve**: 1-2 hours (download + install + setup)

### Option B: Use Xcode Beta (If Available)

According to migration notes, the issue was previously resolved by switching from Xcode 26 beta to Xcode 15.4.

**If you have Xcode installed but it's the wrong version**:

1. Check current Xcode version:
   ```bash
   xcodebuild -version
   ```

2. Switch to stable Xcode (not beta):
   ```bash
   # If you have multiple Xcode versions
   sudo xcode-select --switch /Applications/Xcode-15.4.app/Contents/Developer
   ```

3. Verify:
   ```bash
   xcrun --find metal
   ```

### Option C: Continue Development Without Running App

**Best for**: Testing and code review without running the binary

**What works without Metal**:
- ✅ `cargo check` - Code compilation check
- ✅ `cargo test --package rstn-core` - Unit tests (182 tests)
- ✅ `cargo test --package rstn-ui` - UI component tests
- ✅ `cargo test --package rstn-views` - View tests
- ✅ Code editing, refactoring, type checking
- ✅ Git operations, PR creation

**What doesn't work**:
- ❌ `cargo run --bin rstn` - Running the application
- ❌ `cargo test --bin rstn` - Binary integration tests
- ❌ Visual testing of UI components
- ❌ Interactive feature development

**Workflow**:
```bash
# Develop features
cargo check --package rstn

# Run tests
cargo test --package rstn-core
cargo test --package rstn-views

# Commit changes
git add .
git commit -m "feat: ..."
git push

# Wait for CI/CD or teammate with Xcode to test
```

### Option D: Use GitHub Actions / CI

**Best for**: Automated testing without local Xcode

Create a GitHub Actions workflow that runs on macOS runners (which have Xcode pre-installed):

```yaml
# .github/workflows/test-gpui.yml
name: Test GPUI App

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Build rstn
        run: cargo build --bin rstn

      - name: Run tests
        run: cargo test
```

GitHub-hosted macOS runners include Xcode by default.

## Verification Steps

After installing Xcode, verify everything works:

1. **Check Metal compiler**:
   ```bash
   xcrun --find metal
   # Should output path to metal compiler
   ```

2. **Check active developer directory**:
   ```bash
   xcode-select -p
   # Should output: /Applications/Xcode.app/Contents/Developer
   ```

3. **Build GPUI**:
   ```bash
   cargo clean
   cargo build --bin rstn
   # Should compile without Metal errors
   ```

4. **Run application**:
   ```bash
   RUST_LOG=info cargo run --bin rstn
   # Application window should open
   ```

5. **Test all 8 views**:
   - Click through all tabs: Tasks, Dockers, Explorer, Terminal, Chat, Workflows, MCP, Settings
   - Verify each view renders correctly
   - Check that data loads from backend

## Troubleshooting

### Error: "metal shader compilation failed" persists

**Possible causes**:
1. Xcode not set as active developer directory
   ```bash
   sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
   ```

2. Cached build artifacts
   ```bash
   cargo clean
   rm -rf target/
   cargo build --bin rstn
   ```

3. Wrong Xcode version (need 15.4+, not beta)
   ```bash
   xcodebuild -version
   ```

### Error: "tool 'metal' requires Xcode"

You're still using Command Line Tools instead of full Xcode:
```bash
xcode-select -p
# If this shows /Library/Developer/CommandLineTools
# Run: sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

### Application builds but crashes on launch

Check Metal runtime errors in logs:
```bash
RUST_LOG=debug cargo run --bin rstn 2>&1 | grep -i metal
```

## Migration Status Impact

**Current Status**: 91% complete (Phase 6: 59%)

**Blocked by Metal**:
- Interactive feature testing
- Visual UI verification
- Integration tests
- Performance profiling

**Not blocked**:
- State management (complete)
- View integrations (complete)
- Unit tests (200+ passing)
- Code architecture (solid)

**Recommendation**: Install Xcode to unlock the remaining 9% of work

## References

- **Migration tasks**: [openspec/changes/migrate-to-gpui/tasks.md](changes/migrate-to-gpui/tasks.md)
- **Phase 6 summary**: [openspec/PHASE_6_STAGE_3_4_SUMMARY.md](PHASE_6_STAGE_3_4_SUMMARY.md)
- **Apple Metal**: https://developer.apple.com/metal/
- **GPUI**: https://github.com/zed-industries/zed

---

**Last Updated**: 2026-01-12
**Status**: Documented workaround options
**Next Step**: Choose option A, B, C, or D based on requirements
