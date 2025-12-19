# Contribution Guide

**Last Updated**: 2025-12-19
**Version**: v2 (state-first architecture)

Welcome to rustation v2 development! This guide will help you set up your environment and understand our contribution workflow.

---

## Prerequisites

### Required Tools

- **Rust 1.75+** (edition 2021)
  ```bash
  rustc --version
  # Should be 1.75.0 or higher
  ```

- **cargo** (comes with Rust)
  ```bash
  cargo --version
  ```

- **Git** (version control)
  ```bash
  git --version
  ```

### Recommended Tools

- **rust-analyzer** (LSP for IDE support)
- **clippy** (linting)
  ```bash
  rustup component add clippy
  ```

- **rustfmt** (code formatting)
  ```bash
  rustup component add rustfmt
  ```

---

## Development Environment Setup

### 1. Clone the Repository

```bash
git clone https://github.com/chrischeng-c4/rustation.git
cd rustation
```

### 2. Build the Project

```bash
# Build all packages
cargo build

# Or build specific package
cargo build -p rstn
cargo build -p rush
```

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test -p rstn

# Run specific test
cargo test -p rstn test_state_serialization
```

### 4. Install Development Binaries

**Option 1: Install to cargo bin** (permanent):
```bash
cargo install --path crates/rstn
cargo install --path crates/rush
```

**Option 2: Run directly** (temporary):
```bash
cargo run -p rstn
cargo run -p rush
```

### 5. Verify Setup

```bash
# Check clippy
cargo clippy -p rstn -- -D warnings

# Check formatting
cargo fmt --check

# Run all tests
cargo test
```

If all commands succeed, you're ready to contribute!

---

## 🎯 **MANDATORY**: State-First Testing

**CRITICAL REQUIREMENT**: ALL features in v2 MUST include state serialization and transition tests.

### Why Mandatory?

State-first architecture is the **core principle** of v2:
- **Testability**: Observable, deterministic, stable
- **Reproducibility**: Save state → load state → exact bug reproduction
- **Refactoring safety**: Tests don't break on UI changes
- **Documentation**: Tests show intended behavior

**Without state tests, your PR WILL NOT be merged.**

### Required Tests for Every Feature

Every feature MUST include these three types of tests:

#### 1. Round-Trip Serialization Test

**What**: Verify state can be serialized to JSON and deserialized back without loss.

**Example**:
```rust
#[test]
fn test_state_serialization_round_trip() {
    let state = AppState::default();
    let json = serde_json::to_string(&state).unwrap();
    let loaded: AppState = serde_json::from_str(&json).unwrap();
    assert_eq!(state, loaded); // MUST pass
}
```

**Why**: Ensures state is truly serializable (no hidden data loss).

#### 2. State Transition Test

**What**: Verify actions mutate state correctly.

**Example**:
```rust
#[test]
fn test_state_transition() {
    let mut app = App::from_state(AppState::default()).unwrap();

    app.handle_action(ViewAction::YourFeature);

    let final_state = app.to_state();
    assert_eq!(final_state.your_field, expected_value);
}
```

**Why**: Tests behavior, not implementation details.

#### 3. State Invariant Test

**What**: Verify state invariants always hold.

**Example**:
```rust
#[test]
fn test_state_invariants() {
    let state = app.to_state();

    // Invariants that MUST always hold
    if state.feature_active {
        assert!(state.feature_data.is_some());
    }
}
```

**Why**: Catches invalid state combinations.

### Enforcement

**Code Review Checklist** (reviewers will verify):
- [ ] New state structs derive `Serialize + Deserialize + Debug + Clone`
- [ ] Round-trip serialization test included
- [ ] State transition tests included
- [ ] State invariant tests included (if applicable)

**CI Checks** (future):
- [ ] Test coverage >70%
- [ ] All state tests pass
- [ ] Clippy clean

**See Also**: [State-First Architecture](../02-architecture/state-first.md), [Testing Guide](testing-guide.md)

---

## Contribution Workflow

### 1. Choose a Task

**For rush features**:
- See `ROADMAP.md` for Phase 7-8 features
- All rush features use **Full SDD** workflow

**For rstn features**:
- Check GitHub Issues for open tasks
- Use **Full SDD** for complex features (>500 LOC, >5 files)
- Use **Lightweight SDD** for simple changes (<200 LOC, <3 files)

**Decision guide**: See [SDD Workflow Guide](sdd-workflow.md#decision-matrix)

### 2. Create Specification

**Full SDD**:
```bash
# Step 1: Create spec
/speckit.specify

# Step 2 (optional): Clarify spec
/speckit.clarify

# Step 3: Create plan
/speckit.plan

# Step 4: Generate tasks
/speckit.tasks
```

**Lightweight SDD**:
```bash
# Create simplified spec only
/speckit-lite
```

**Output**: `specs/{NNN}-{name}/spec.md` (+ plan.md, tasks.md for Full SDD)

### 3. Implement Feature

**Order of operations** (state-first testing):

1. **Define state structs FIRST**:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   pub struct YourFeatureState {
       pub field1: String,
       pub field2: Option<u32>,
   }
   ```

2. **Write state tests BEFORE implementation**:
   ```rust
   #[test]
   fn test_your_feature_state_round_trip() {
       // Round-trip test
   }

   #[test]
   fn test_your_feature_transition() {
       // Transition test
   }
   ```

3. **Implement business logic**:
   - Keep modules <500 lines
   - Keep structs <15 fields
   - No business logic in view layer

4. **Implement UI layer** (if applicable):
   - CLI: `crates/rstn/src/commands/`
   - TUI: `crates/rstn/src/tui/views/`

5. **Verify all tests pass**:
   ```bash
   cargo test -p rstn
   ```

### 4. Code Quality Checks

**Before committing**:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -p rstn -- -D warnings

# Run all tests
cargo test

# Check test coverage (future)
# cargo tarpaulin
```

**All checks must pass.**

### 5. Commit and Push

**Commit message format**:
```
<type>(<feature-number>): <description>

Examples:
feat(079): add state serialization support
fix(079): correct state transition logic
test(079): add state invariant tests
docs(079): update state-first architecture guide
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `test`: Add/update tests
- `docs`: Documentation only
- `refactor`: Code refactoring (no behavior change)
- `perf`: Performance improvement
- `chore`: Maintenance (deps, CI, etc.)

**Push to branch**:
```bash
git checkout -b feature-079-state-first
git add .
git commit -m "feat(079): add state serialization"
git push origin feature-079-state-first
```

### 6. Create Pull Request

**PR Title**: Same as commit message
```
feat(079): add state serialization support
```

**PR Description Template**:
```markdown
## Summary
[Brief description of changes]

## Feature
- Feature number: 079
- Feature name: state-first

## Artifacts
- [ ] spec.md exists and is current
- [ ] plan.md exists (Full SDD only)
- [ ] tasks.md exists (Full SDD only)

## Testing
- [ ] Round-trip serialization test included
- [ ] State transition tests included
- [ ] State invariant tests included (if applicable)
- [ ] All tests pass: `cargo test`
- [ ] Clippy clean: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt --check`

## Implementation
- [ ] State structs derive Serialize + Deserialize + Debug + Clone
- [ ] No business logic in view layer
- [ ] Modules <500 lines, structs <15 fields
- [ ] Implementation matches spec

## Related
Closes #[issue-number]
```

**Use /speckit.review** to auto-generate PR description (recommended):
```bash
/speckit.review
```

---

## PR Review Process

### Reviewer Checklist

**State-First Requirements** (MANDATORY):
- [ ] New state structs have required derives
- [ ] Round-trip serialization test included
- [ ] State transition tests included
- [ ] Tests actually test state, not UI coordinates

**Code Quality**:
- [ ] No unwrap() in production code (use ? or proper error handling)
- [ ] No panic!() in production code
- [ ] Modules <500 lines
- [ ] Structs <15 fields
- [ ] No business logic in view layer

**Testing**:
- [ ] All tests pass
- [ ] Clippy clean
- [ ] Code formatted

**SDD Compliance**:
- [ ] spec.md exists and matches implementation
- [ ] plan.md exists (if Full SDD)
- [ ] tasks.md exists and all tasks complete (if Full SDD)

**Documentation**:
- [ ] Public APIs documented
- [ ] Complex logic has inline comments
- [ ] CLAUDE.md updated (if workflow changes)

### Approval Requirements

**Minimum**: 1 approval from maintainer

**Merge**: Squash and merge (keep history clean)

---

## Common Pitfalls

### ❌ Don't: Skip State Tests

```rust
// ❌ BAD: No state tests
#[test]
fn test_ui_rendering() {
    let buffer = render_to_buffer(&app);
    assert_eq!(buffer.get(10, 5).symbol, "│");
}
```

**Fix**: Write state tests first:
```rust
// ✅ GOOD: State test
#[test]
fn test_state_transition() {
    let mut app = App::default();
    app.handle_action(ViewAction::Switch);
    assert_eq!(app.to_state().view, ViewType::Settings);
}
```

### ❌ Don't: Non-Serializable State

```rust
// ❌ BAD: Can't serialize function pointers
#[derive(Serialize, Deserialize)]
struct BadState {
    callback: Box<dyn Fn()>, // Error!
}
```

**Fix**: Keep state data-only:
```rust
// ✅ GOOD: Data only
#[derive(Serialize, Deserialize)]
struct GoodState {
    action_type: ActionType, // Enum instead
}
```

### ❌ Don't: Business Logic in View Layer

```rust
// ❌ BAD: Logic in TUI view
impl WorktreeView {
    fn render(&mut self) {
        // ... render code ...
        self.calculate_metrics(); // Logic!
        self.update_database();   // Logic!
    }
}
```

**Fix**: Move logic to separate module:
```rust
// ✅ GOOD: Logic in domain layer
impl WorktreeView {
    fn render(&self) {
        // Only rendering here
    }
}

// In domain/worktree.rs:
pub fn calculate_metrics(state: &WorktreeState) -> Metrics {
    // Logic here
}
```

### ❌ Don't: Large Modules/Structs

```rust
// ❌ BAD: 60+ fields
struct GodStruct {
    field1: String,
    field2: u32,
    // ... 58 more fields ...
}
```

**Fix**: Break into smaller structs:
```rust
// ✅ GOOD: Composition
struct AppState {
    worktree: WorktreeState,  // 10 fields
    dashboard: DashboardState, // 8 fields
    settings: SettingsState,   // 12 fields
}
```

---

## Getting Help

### Documentation

- **Architecture**: [State-First Architecture](../02-architecture/state-first.md)
- **Principles**: [Core Principles](../02-architecture/core-principles.md)
- **Testing**: [Testing Guide](testing-guide.md)
- **Debugging**: [Debugging Guide](debugging.md)
- **Workflow**: [SDD Workflow](sdd-workflow.md)

### Community

- **GitHub Issues**: [Report bugs or ask questions](https://github.com/chrischeng-c4/rustation/issues)
- **GitHub Discussions**: [Feature requests, ideas](https://github.com/chrischeng-c4/rustation/discussions)

### Logs

Check logs when debugging:
```bash
# rstn logs
tail -f ~/.rstn/logs/rstn.log

# Or alternative location
tail -f ~/.rustation/logs/rstn.log
```

---

## Quick Reference

### Build Commands
```bash
cargo build -p rstn           # Build TUI
cargo test -p rstn            # Run tests
cargo clippy -p rstn          # Lint check
cargo fmt                     # Format code
```

### SDD Commands
```bash
/speckit.specify              # Create spec.md
/speckit.plan                 # Create plan.md
/speckit.tasks                # Create tasks.md
/speckit.implement            # Implement tasks
/speckit.review               # Verify against spec
```

### State Testing Template
```rust
#[test]
fn test_feature_state_round_trip() {
    let state = FeatureState::default();
    let json = serde_json::to_string(&state).unwrap();
    let loaded: FeatureState = serde_json::from_str(&json).unwrap();
    assert_eq!(state, loaded);
}

#[test]
fn test_feature_state_transition() {
    let mut app = App::from_state(AppState::default()).unwrap();
    app.handle_action(ViewAction::YourAction);
    let final_state = app.to_state();
    assert_eq!(final_state.your_field, expected);
}
```

---

## Changelog

- 2025-12-19: Initial contribution guide for v2
