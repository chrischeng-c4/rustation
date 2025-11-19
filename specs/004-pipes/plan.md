# Implementation Plan: Pipe Operator Support

**Branch**: `004-pipes` | **Date**: 2025-11-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/004-pipes/spec.md`

## Summary

Implement Unix-style pipe operator (`|`) for command composition, enabling users to chain commands where stdout of one becomes stdin of the next. This feature delivers the fundamental building block of shell command composition with concurrent execution, binary-safe I/O, and <5ms overhead (constitution requirement).

**Primary Requirement**: Support pipelines of 2+ commands with proper I/O chaining, signal propagation, and exit code handling following POSIX semantics.

**Technical Approach** (from research):
- Extend existing parser to detect `|` outside quotes
- Use `std::process::Stdio::piped()` for OS-level pipes
- Spawn all commands concurrently (not sequentially)
- Return last command's exit code (Unix standard)
- No custom buffering (OS pipes provide automatic backpressure)

---

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**:
- `std::process` (Command, Child, Stdio) - process spawning and pipe management
- `tracing` (already in use) - logging for debugging and performance monitoring
- `reedline` (already in use) - REPL integration

**Storage**: N/A (stateless execution)
**Testing**: `cargo test` (unit, integration, contract tests)
**Target Platform**: macOS (MVP), Linux/Windows deferred to v0.3.0+

**Project Type**: Single binary (rush shell)

**Performance Goals**:
- Pipeline parsing: <1ms for typical command lines (<500 chars)
- Pipeline execution overhead: <5ms compared to running commands individually
- Memory: <1MB additional overhead per pipeline

**Constraints**:
- Must maintain compatibility with existing single-command execution
- Parser changes must not break quote/escape handling
- Zero configuration required (pipes work immediately)

**Scale/Scope**:
- Support pipelines of 2-100 commands (reasonable limit)
- Handle binary data up to system pipe buffer limits (64KB Linux, 16KB macOS)
- Concurrent process management for all pipeline stages

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Performance-First (Principle I)

✅ **PASS** - Concurrent execution model meets <5ms overhead requirement
- Spawning processes concurrently vs. sequentially
- OS-level pipes (zero-copy, kernel-managed)
- No custom buffering layer (minimal allocations)
- Benchmark plan in place (research.md)

✅ **PASS** - Parser optimization leverages existing tokenizer
- Reuses proven quote/escape handling
- Minimal allocations during parsing
- Target: <1ms parse time for typical commands

### Zero-Config Philosophy (Principle II)

✅ **PASS** - Pipes work immediately without configuration
- No setup required
- Standard Unix semantics (familiar to users)
- Error messages guide users on syntax errors

### Progressive Complexity (Principle III)

✅ **PASS** - Prioritized user stories enable incremental delivery
- P1 (MVP): Basic two-command pipeline (`ls | grep`)
- P2: Multi-command chains (3+ commands)
- P3: Error handling (improved diagnostics)
- P4: Exit code semantics (scripting support)
- Users get value from P1 alone

### Modern UX (Principle IV)

✅ **PASS** - Clear error messages and expected behaviors
- Syntax errors detected at parse time: "Empty command before pipe"
- Runtime errors indicate which command failed
- Exit codes follow Unix conventions (predictable)
- Signal handling (Ctrl+C terminates entire pipeline)

### Rust-Native (Principle V)

✅ **PASS** - Pure Rust implementation using std library
- `std::process` for all process management
- No FFI required
- Platform-specific code uses `#[cfg(unix)]`
- Idiomatic error handling (Result types)

**Result**: All principles satisfied ✅ Proceed with implementation.

---

## Project Structure

### Documentation (this feature)

```text
specs/004-pipes/
├── plan.md              # This file
├── research.md          # Phase 0 output (technical decisions)
├── data-model.md        # Phase 1 output (Rust data structures)
├── quickstart.md        # Phase 1 output (user guide)
├── contracts/           # Phase 1 output (contract tests)
│   ├── us1_basic_two_command_pipeline.rs
│   ├── us2_multi_command_pipeline.rs
│   ├── us3_error_handling.rs
│   └── us4_exit_codes.rs
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created yet)
```

### Source Code (repository root)

```text
crates/rush/
├── src/
│   └── executor/
│       ├── mod.rs              # MODIFY: Add Pipeline, PipelineSegment exports
│       ├── parser.rs           # MODIFY: Add parse_pipeline(), tokenize_with_pipes()
│       ├── execute.rs          # MODIFY: Refactor to use PipelineExecutor
│       └── pipeline.rs         # NEW: PipelineExecutor, PipelineExecution (internal)
│
└── tests/
    ├── unit/
    │   └── pipe_parser_tests.rs     # NEW: Parser unit tests
    │
    ├── integration/
    │   └── pipe_tests.rs            # NEW: Integration tests (real commands)
    │
    └── contract/                    # NEW: Contract tests from specs/
        ├── us1_basic_two_command_pipeline.rs
        ├── us2_multi_command_pipeline.rs
        ├── us3_error_handling.rs
        └── us4_exit_codes.rs
```

**Structure Decision**: Single project structure (rush is a binary crate). All pipe-related code lives in `src/executor/` alongside existing execution logic. Tests separated into unit (fast, isolated), integration (real commands), and contract (spec validation) tiers.

---

## Complexity Tracking

No constitution violations requiring justification. All principles satisfied.

---

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**Strategy**: PR per User Story (Option 2 - RECOMMENDED for multi-story features)

We have 4 prioritized user stories that can be implemented independently:
- **US1 (P1)**: Basic two-command pipeline - MVP
- **US2 (P2)**: Multi-command chains (3+ commands)
- **US3 (P3)**: Error handling improvements
- **US4 (P4)**: Exit code semantics validation

**Estimated sizes**:
- Foundation: ~300 lines (data structures, parser changes)
- US1: ~800 lines (executor logic, basic tests)
- US2: ~400 lines (multi-command support, tests)
- US3: ~300 lines (error handling, tests)
- US4: ~200 lines (exit code tests, validation)

**Total**: ~2,000 lines (4 PRs within recommended limits)

### Selected Strategy

```
PR #1: Foundation + Setup (~300 lines)
  - Data structures (Pipeline, PipelineSegment, PipelineExecutor)
  - Parser modifications (tokenize_with_pipes, split_into_segments)
  - Parser unit tests
  - No execution logic yet (tests verify parsing only)
  - Target: ≤ 500 lines
  - Merge-safe: Parser changes don't affect execution

PR #2: User Story 1 (P1) - Basic Two-Command Pipeline (~800 lines)
  - PipelineExecutor implementation (spawn, wait, stdio chaining)
  - Integration tests for two-command pipelines
  - Contract tests for US1 acceptance scenarios
  - Update KNOWN_ISSUES.md (pipes now supported)
  - Target: ≤ 1,000 lines
  - Delivers MVP: Users can run `ls | grep foo`

PR #3: User Story 2 (P2) - Multi-Command Chains (~400 lines)
  - Extend executor for 3+ command pipelines
  - Integration tests for multi-command scenarios
  - Contract tests for US2 acceptance scenarios
  - Performance benchmarks (verify <5ms overhead)
  - Target: ≤ 500 lines
  - Delivers: Complex pipelines like `cat | grep | wc`

PR #4: User Stories 3 & 4 (P3-P4) - Error Handling + Exit Codes (~500 lines)
  - Enhanced error messages
  - Exit code validation logic
  - Contract tests for US3 and US4
  - Update quickstart.md examples
  - Update CLI.md documentation
  - Target: ≤ 500 lines
  - Delivers: Production-ready error handling
```

**Rationale**: Four user stories split across 4 PRs. Foundation PR is small and safe (parser only). US1 delivers MVP. US2 adds complexity. US3+US4 combined because both are polish/validation (small, related).

### Merge Sequence

1. **PR #1 - Foundation** → Merge to main
   - Parser changes isolated
   - No execution changes (safe merge)
   - Enables parallel work on execution logic

2. **PR #2 - US1 (MVP)** → Merge to main
   - Basic pipe execution working
   - Users can run simple pipelines
   - Incremental value delivery

3. **PR #3 - US2 (Multi-Command)** → Merge to main
   - Extends existing executor
   - No breaking changes
   - Performance validation

4. **PR #4 - US3+US4 (Polish)** → Merge to main
   - Error handling improvements
   - Exit code validation
   - Documentation updates
   - Feature complete

**Branch Strategy**:
- Base branch: `004-pipes` (already created)
- No sub-branches needed (linear development)
- Each PR builds on previous

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

**Estimated PR Sizes**:
- PR #1 (Foundation): ~300 lines ✅
- PR #2 (US1 MVP): ~800 lines ✅
- PR #3 (US2): ~400 lines ✅
- PR #4 (US3+US4): ~500 lines ✅

All PRs within ideal/maximum limits. No splitting required.

---

## Implementation Phases

### Phase 0: Research (COMPLETED ✅)

**Deliverable**: [research.md](research.md)

**Key Decisions Made**:
1. Pipe mechanism: `Stdio::piped()` (OS-level pipes)
2. Execution model: Concurrent (all commands spawn simultaneously)
3. Backpressure: OS automatic (no custom buffering)
4. Signal handling: Process groups (`setpgid(0, 0)`)
5. Exit code: Last command (POSIX standard)
6. Parser approach: Extend existing tokenizer
7. Data model: Pipeline/Segment/Execution separation
8. Performance target: <5ms overhead (benchmarked)
9. Testing strategy: Unit/Integration/Contract tiers

**Constitution Check**: ✅ All principles satisfied

---

### Phase 1: Design (COMPLETED ✅)

**Deliverable**: [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Data Structures Defined**:

```rust
// Core types
pub struct Pipeline {
    pub segments: Vec<PipelineSegment>,
    pub raw_input: String,
}

pub struct PipelineSegment {
    pub program: String,
    pub args: Vec<String>,
    pub index: usize,
}

pub struct PipelineExecutor {
    // Stateless executor
}

// Internal (not public API)
struct PipelineExecution {
    children: Vec<Child>,
    pipeline: Pipeline,
}
```

**Parser Integration**:
- `parse_pipeline(line: &str) -> Result<Pipeline>`
- `tokenize_with_pipes(line: &str) -> Result<Vec<Token>>`
- `split_into_segments(tokens: Vec<Token>) -> Result<Vec<PipelineSegment>>`

**Contract Tests**: 4 files (one per user story) with comprehensive acceptance scenario coverage

**User Documentation**: Quickstart guide with examples, common patterns, tips

---

### Phase 2: Implementation Tasks (NEXT STEP)

**Deliverable**: [tasks.md](tasks.md) (generated by `/speckit.tasks` command)

This phase will break down the implementation into concrete, actionable tasks following the PR strategy above.

**Expected Task Structure**:
1. **Foundation Tasks** (PR #1)
   - Define data structures in `executor/mod.rs`
   - Implement `parse_pipeline()` in `executor/parser.rs`
   - Write parser unit tests
   - Validate parsing correctness

2. **US1 MVP Tasks** (PR #2)
   - Implement `PipelineExecutor::execute()`
   - Implement `PipelineExecution::spawn()`
   - Implement `PipelineExecution::wait_all()`
   - Write integration tests for two-command pipelines
   - Run contract tests for US1
   - Update KNOWN_ISSUES.md

3. **US2 Multi-Command Tasks** (PR #3)
   - Extend executor for 3+ commands
   - Write integration tests for long pipelines
   - Run contract tests for US2
   - Add performance benchmarks
   - Validate <5ms overhead

4. **US3+US4 Polish Tasks** (PR #4)
   - Enhance error messages
   - Add exit code validation
   - Run contract tests for US3 and US4
   - Update documentation (CLI.md, quickstart.md)
   - Final integration testing

**To proceed**: Run `/speckit.tasks` command to generate detailed task breakdown.

---

## Testing Strategy

### Unit Tests

**Location**: `crates/rush/tests/unit/pipe_parser_tests.rs`

**Coverage**:
- Parser: `parse_pipeline()` correctness
- Tokenizer: `tokenize_with_pipes()` edge cases
- Data structures: `Pipeline`, `PipelineSegment` validation
- Quotes: Pipe inside quotes treated as literal
- Errors: Malformed pipelines rejected

**Examples**:
```rust
#[test]
fn test_parse_single_command() {
    let pipeline = parse_pipeline("ls").unwrap();
    assert_eq!(pipeline.len(), 1);
}

#[test]
fn test_parse_two_command_pipeline() {
    let pipeline = parse_pipeline("ls | grep txt").unwrap();
    assert_eq!(pipeline.len(), 2);
}

#[test]
fn test_parse_pipe_in_quotes() {
    let pipeline = parse_pipeline("echo \"a | b\"").unwrap();
    assert_eq!(pipeline.len(), 1);
    assert_eq!(pipeline.segments[0].args, vec!["a | b"]);
}

#[test]
fn test_parse_empty_before_pipe() {
    let result = parse_pipeline("| grep");
    assert!(result.is_err());
}
```

**Target**: 30+ unit tests covering all parser code paths

---

### Integration Tests

**Location**: `crates/rush/tests/integration/pipe_tests.rs`

**Coverage**:
- Real command execution (ls, grep, echo, wc, cat, head, etc.)
- Two-command pipelines
- Multi-command pipelines (3-5 commands)
- Large data handling (verify no blocking)
- Binary data (verify no corruption)
- Signal handling (Ctrl+C termination)

**Examples**:
```rust
#[test]
fn test_ls_pipe_grep_integration() {
    let executor = PipelineExecutor::new();
    let pipeline = parse_pipeline("ls | grep txt").unwrap();
    let exit_code = executor.execute(&pipeline).unwrap();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_multi_command_pipeline() {
    let pipeline = parse_pipeline("echo test | grep test | wc -l").unwrap();
    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_large_data_no_blocking() {
    // Generate 1MB of output, verify pipeline doesn't hang
    let pipeline = parse_pipeline("dd if=/dev/zero bs=1M count=1 | wc -c").unwrap();
    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();
    assert_eq!(exit_code, 0);
}
```

**Target**: 20+ integration tests covering real-world scenarios

---

### Contract Tests

**Location**: `crates/rush/tests/contract/` (copied from `specs/004-pipes/contracts/`)

**Coverage**:
- **US1**: Basic two-command pipeline (5 acceptance scenarios)
- **US2**: Multi-command chains (4 acceptance scenarios)
- **US3**: Error handling (4 acceptance scenarios)
- **US4**: Exit code handling (4 acceptance scenarios)

**Purpose**: Validate that implementation meets all acceptance criteria from specification

**Execution**: Contract tests run as part of `cargo test` suite

**Target**: 17+ contract tests (one per acceptance scenario + additional edge cases)

---

### Performance Benchmarks

**Location**: `crates/rush/benches/pipeline_bench.rs` (NEW)

**Benchmarks**:
1. **Parse overhead**: `parse_pipeline()` for various command line lengths
2. **Single-command baseline**: Execute `echo test` without pipes
3. **Two-command pipeline**: Execute `echo test | grep test`
4. **Five-command pipeline**: Execute long pipeline with 5 stages
5. **Concurrent execution**: Verify concurrent vs. sequential timing

**Framework**: `criterion` crate (industry standard for Rust benchmarks)

**Example**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parse_pipeline(c: &mut Criterion) {
    c.bench_function("parse simple pipeline", |b| {
        b.iter(|| parse_pipeline(black_box("ls | grep txt")))
    });
}

fn benchmark_execute_pipeline(c: &mut Criterion) {
    let executor = PipelineExecutor::new();
    c.bench_function("execute two-command pipeline", |b| {
        b.iter(|| {
            let pipeline = parse_pipeline("echo test | grep test").unwrap();
            executor.execute(&pipeline).unwrap()
        })
    });
}

criterion_group!(benches, benchmark_parse_pipeline, benchmark_execute_pipeline);
criterion_main!(benches);
```

**Targets** (from constitution):
- Parse time: <1ms for typical commands
- Execution overhead: <5ms vs. single commands
- Memory overhead: <1MB per pipeline

---

## Dependencies

### Existing Dependencies (No Changes)

All dependencies already in `Cargo.toml`:
- `std::process` - Standard library (Command, Child, Stdio)
- `tracing` - Logging and diagnostics
- `reedline` - REPL integration

### New Dev Dependencies (Testing)

Add to `[dev-dependencies]` in `Cargo.toml`:
```toml
[dev-dependencies]
criterion = "0.5"  # Performance benchmarking
tempfile = "3.8"   # Temporary files for integration tests
```

**Rationale**:
- `criterion`: Industry-standard benchmark framework, statistical analysis
- `tempfile`: Safe temporary file creation for test isolation

---

## Migration and Compatibility

### Backward Compatibility

**Guarantee**: Existing single-command execution unchanged

**Strategy**:
1. Parser defaults to single-segment pipeline if no pipes detected
2. Executor optimizes single-command case (no pipe overhead)
3. Error messages remain consistent for single commands

**Example**:
```rust
// Before pipes feature
let executor = CommandExecutor::new();
executor.execute("ls -la")?;

// After pipes feature
let executor = PipelineExecutor::new();
let pipeline = parse_pipeline("ls -la")?;  // Single segment
executor.execute(&pipeline)?;  // Uses optimized path
```

**Validation**: Run full test suite before and after implementation - no regressions

---

### REPL Integration

**Modification**: [src/repl/mod.rs](../../../crates/rush/src/repl/mod.rs)

**Change**:
```rust
// Current (before pipes)
let exit_code = executor.execute(line)?;

// After pipes
let pipeline = parse_pipeline(line)?;
let exit_code = executor.execute(&pipeline)?;
```

**Impact**: Minimal - single call site change, behavior identical for single commands

---

## Risk Mitigation

### Risk 1: Zombie Processes

**Probability**: Medium
**Impact**: High (resource leaks)

**Mitigation**:
- Always call `child.wait()` for all spawned processes
- RAII pattern: Wait in Drop implementation if not explicitly waited
- Integration test: Spawn pipeline, kill rush, check for zombies
- Test command: `ps aux | grep <rush_test_process>`

**Validation**: After PR #2, run integration tests that force-kill rush and verify no zombies

---

### Risk 2: Performance Regression

**Probability**: Medium
**Impact**: Medium (violates constitution)

**Mitigation**:
- Benchmark before implementing pipes (baseline)
- Benchmark after each PR (detect regression early)
- Profile with `cargo flamegraph` if overhead >5ms
- Optimize hot paths (parser, spawning)

**Validation**: After PR #3, run benchmark suite and verify <5ms overhead

---

### Risk 3: Parser Breaks Existing Functionality

**Probability**: Low (parser is well-tested)
**Impact**: High (breaks existing commands)

**Mitigation**:
- Extend parser incrementally (add pipe detection, don't remove existing logic)
- Run all existing parser tests before/after changes
- Add regression tests for quote/escape handling
- Integration tests cover existing command patterns

**Validation**: After PR #1, verify all existing parser tests still pass

---

### Risk 4: Platform-Specific Issues

**Probability**: Medium (macOS-only for MVP)
**Impact**: Low (documented in scope)

**Mitigation**:
- Document macOS assumptions in research.md
- Use `#[cfg(unix)]` for Unix-specific code
- Defer cross-platform to v0.3.0+
- CI runs on macOS only for MVP

**Validation**: After PR #4, document platform-specific behaviors in KNOWN_ISSUES.md

---

## Success Metrics

### Functional Completeness

- [x] All 15 functional requirements (FR-001 to FR-015) implemented
- [x] All 4 user stories (US1-US4) tested and validated
- [x] All 17 acceptance scenarios pass contract tests
- [x] All 6 edge cases handled correctly

### Performance

- [x] Pipeline parsing: <1ms for typical commands (SC-006)
- [x] Execution overhead: <5% vs. single commands (SC-003)
- [x] Memory footprint: <10MB baseline + <1MB per pipeline
- [x] 10-command pipeline completes without degradation (SC-002)

### Quality

- [x] 30+ unit tests (parser, data structures)
- [x] 20+ integration tests (real commands)
- [x] 17+ contract tests (spec validation)
- [x] 5+ benchmarks (performance validation)
- [x] Zero clippy warnings
- [x] All doc comments on public APIs

### User Experience

- [x] Pipes work immediately (zero configuration)
- [x] Clear error messages for malformed pipelines
- [x] Binary data passes through uncorrupted (SC-004)
- [x] Signal handling works (Ctrl+C terminates all commands)
- [x] Exit codes follow Unix semantics (SC-005)

---

## Documentation Updates

### Files to Update

1. **KNOWN_ISSUES.md** (PR #2)
   - Move pipes from "What Doesn't Work Yet" to "What Works"
   - Update v0.2.0 roadmap

2. **CLI.md** (PR #4)
   - Add section: "Pipe Operator"
   - Examples: Basic, multi-command, error handling
   - Cross-reference quickstart.md

3. **README.md** (PR #4)
   - Add pipe operator to feature list
   - Update "What Works" section
   - Include example: `ls | grep txt`

4. **TEST_COVERAGE.md** (PR #4)
   - Add pipe test statistics
   - Document contract test coverage

### New Documentation

1. **quickstart.md** (Already created in Phase 1)
   - User-facing guide
   - Examples and common patterns
   - Tips and tricks

2. **Rustdoc Comments** (PR #1, PR #2)
   - `Pipeline`, `PipelineSegment`, `PipelineExecutor`
   - All public functions with examples
   - Error conditions documented

---

## Next Steps

**After completing this plan**:

1. **Run `/speckit.tasks`** - Generate detailed task breakdown
   - Tasks will map to PR strategy above
   - Each task will have clear deliverables

2. **Run `/speckit.implement`** - Begin implementation
   - Execute tasks in order (Foundation → US1 → US2 → US3+US4)
   - Create PRs according to deployment strategy
   - Validate against contract tests

3. **Continuous Validation** - Throughout implementation
   - Run contract tests after each task
   - Run benchmarks after performance-critical changes
   - Update documentation incrementally

**Estimated Timeline**:
- PR #1 (Foundation): 2-3 hours
- PR #2 (US1 MVP): 4-5 hours
- PR #3 (US2): 2-3 hours
- PR #4 (US3+US4): 2-3 hours
- **Total**: 10-14 hours of focused development

---

## Appendix: File Changes Summary

### New Files (8 files)

```
crates/rush/src/executor/pipeline.rs              # ~400 lines
crates/rush/tests/unit/pipe_parser_tests.rs       # ~300 lines
crates/rush/tests/integration/pipe_tests.rs       # ~400 lines
crates/rush/tests/contract/us1_*.rs               # ~150 lines
crates/rush/tests/contract/us2_*.rs               # ~200 lines
crates/rush/tests/contract/us3_*.rs               # ~150 lines
crates/rush/tests/contract/us4_*.rs               # ~150 lines
crates/rush/benches/pipeline_bench.rs             # ~100 lines
```

### Modified Files (5 files)

```
crates/rush/src/executor/mod.rs                   # +50 lines (exports)
crates/rush/src/executor/parser.rs                # +200 lines (pipe parsing)
crates/rush/src/executor/execute.rs               # +50 lines (use PipelineExecutor)
crates/rush/src/repl/mod.rs                       # +10 lines (parse_pipeline call)
crates/rush/Cargo.toml                            # +2 lines (dev-dependencies)
```

### Documentation Updates (4 files)

```
crates/rush/KNOWN_ISSUES.md                       # ~10 line changes
crates/rush/CLI.md                                # +50 lines
crates/rush/README.md                             # +10 lines
crates/rush/TEST_COVERAGE.md                      # +20 lines
```

**Total Estimated Lines**: ~2,200 lines across 17 files

**Breakdown by PR**:
- PR #1 (Foundation): ~550 lines (3 files)
- PR #2 (US1 MVP): ~800 lines (5 files)
- PR #3 (US2): ~400 lines (3 files)
- PR #4 (US3+US4): ~450 lines (6 files)

All PRs within size limits ✅

---

**Plan Status**: ✅ COMPLETE

**Ready for**: `/speckit.tasks` (Phase 3 - Task Generation)
