# Phase 0: Research - Pipe Operator Support

**Feature**: 004-pipes
**Date**: 2025-11-19
**Status**: Complete

## Research Questions

### 1. How should we connect stdout of one process to stdin of another?

**Answer**: Use `std::process::Stdio::piped()` to create pipes between processes.

**Rationale**:
- Rust's standard library provides `Stdio::piped()` which creates an OS-level pipe
- For command `A | B`:
  - Command A: `stdout(Stdio::piped())`
  - Command B: `stdin(child_a.stdout.take().unwrap())`
- This approach is idiomatic Rust and leverages OS-level pipes for efficiency
- Binary-safe by design (OS pipes transfer bytes without interpretation)

**Example Pattern**:
```rust
let mut child_a = Command::new("ls")
    .stdout(Stdio::piped())
    .spawn()?;

let stdout_a = child_a.stdout.take().unwrap();

let mut child_b = Command::new("grep")
    .arg("txt")
    .stdin(stdout_a)
    .spawn()?;
```

**References**:
- `std::process::Stdio` documentation
- Current implementation: [execute.rs:58-63](../../../crates/rush/src/executor/execute.rs#L58-L63)

---

### 2. Should pipelines execute commands sequentially or concurrently?

**Answer**: Commands MUST execute **concurrently** (not sequentially).

**Rationale**:
- Constitution Principle I (Performance-First) requires <5ms overhead
- Waiting for each command sequentially would violate performance requirements
- Real Unix shells execute pipeline stages concurrently with backpressure
- Concurrent execution enables streaming data (e.g., `tail -f log.txt | grep error`)
- OS pipe buffers provide natural backpressure (typically 64KB on Linux, 16KB on macOS)

**Implementation Strategy**:
1. Spawn all pipeline commands without waiting
2. Connect stdout→stdin between adjacent processes
3. Wait for all processes to complete in pipeline order
4. Return exit code from last command

**Performance Impact**:
- Sequential: ~100ms for 5-command pipeline (20ms × 5)
- Concurrent: ~25ms for 5-command pipeline (max of all commands + coordination)
- Meets constitution requirement of <5ms overhead

---

### 3. How do we handle pipe buffer overflow for large data?

**Answer**: OS-level pipes provide automatic backpressure; no additional handling needed for MVP.

**Rationale**:
- OS pipe buffers are finite (64KB Linux, 16KB macOS)
- When buffer fills, writing process blocks automatically
- Reading process draining buffer unblocks writer
- This is Unix pipe semantics - no custom logic required
- For extremely large data (>1GB), OS handles gracefully

**Edge Cases Covered**:
- Large output (>1GB): Writer blocks when buffer full, resumes when reader drains
- Fast producer, slow consumer: Producer blocks automatically
- Slow producer, fast consumer: Consumer blocks on read, resumes when data available

**Future Enhancement** (post-MVP):
- Monitor pipe buffer utilization for diagnostics
- Warn users if pipeline stalls (no I/O progress for >5s)

---

### 4. How should we handle signal propagation (Ctrl+C)?

**Answer**: Propagate SIGINT to all processes in pipeline group using process groups.

**Rationale**:
- Unix convention: Signals sent to foreground process group
- All pipeline commands should receive SIGINT simultaneously
- Prevents zombie processes or incomplete cleanup
- Users expect `Ctrl+C` to terminate entire pipeline, not just one stage

**Implementation Strategy**:
1. Set process group for all pipeline commands:
   ```rust
   use std::os::unix::process::CommandExt;
   command.process_group(0)  // New process group
   ```
2. When rush receives SIGINT, send to process group
3. All children receive signal and can clean up

**Platform Notes**:
- macOS: Process groups via `setpgid(0, 0)`
- Linux: Same mechanism
- Windows (future): Job objects for equivalent functionality

**References**:
- Constitution: Signal Handling (FR-009)
- Existing: [Current SIGINT handling in REPL](../../../crates/rush/src/repl/mod.rs)

---

### 5. Which command's exit code should the pipeline return?

**Answer**: Last command's exit code (standard Unix semantics).

**Rationale**:
- POSIX standard: Pipeline exit code is last command's exit code
- Users expect `echo test | grep nomatch; echo $?` → returns 1 (grep's exit code)
- Aligns with bash, zsh, fish behavior
- Simple and predictable

**Special Cases**:
- All commands succeed: Return 0
- Middle command fails: Still return last command's exit code
- Last command fails: Return its non-zero exit code

**Alternative Considered** (rejected for MVP):
- `PIPESTATUS` array (bash feature) - deferred to v0.3.0+
- Return first failure exit code - conflicts with POSIX

**Example Behaviors**:
```bash
true | false        # Returns 1 (false's exit code)
false | true        # Returns 0 (true's exit code)
false | false       # Returns 1 (last false's exit code)
```

---

### 6. How do we parse pipe operators while respecting quotes?

**Answer**: Extend existing tokenizer to recognize `|` as special token outside quotes.

**Rationale**:
- Current parser: [parser.rs](../../../crates/rush/src/executor/parser.rs) handles quotes and escapes
- Already distinguishes special characters inside vs. outside quotes
- Pipe parsing follows same pattern as quote handling
- Minimal changes to existing, well-tested parser

**Implementation Approach**:
1. Add `|` detection to tokenizer (like quote detection)
2. When `|` found outside quotes, mark as operator boundary
3. Split tokens into command segments at pipe operators
4. Each segment becomes a `Command` in the `Pipeline`

**Example**:
```rust
Input:  echo "hello | world" | grep hello
Tokens: ["echo", "hello | world", "|", "grep", "hello"]
        ^^^^^^^^^^^^^^^^^^^^^  SEGMENT 1 (pipe inside quotes)
                                   ^^^^^^^^^^^^  SEGMENT 2
Pipeline: [Command("echo", ["hello | world"]), Command("grep", ["hello"])]
```

**Parser Changes**:
- Tokenizer: Add pipe detection (similar to quote detection at line 67-82)
- New function: `split_into_segments(tokens: Vec<String>) -> Vec<Vec<String>>`
- Validation: Reject malformed pipes (`| grep`, `ls |`, `ls | | grep`)

---

### 7. What data structures are needed?

**Answer**: Three primary structures - `Pipeline`, `PipelineSegment`, `PipeConnection`.

**Design**:

```rust
/// Represents a complete pipeline (e.g., "ls | grep txt | wc -l")
pub struct Pipeline {
    /// Individual commands in the pipeline
    pub segments: Vec<PipelineSegment>,
    /// Original command line
    pub raw_input: String,
}

/// One command in a pipeline
pub struct PipelineSegment {
    /// Command to execute
    pub program: String,
    /// Arguments
    pub args: Vec<String>,
    /// Position in pipeline (0-indexed)
    pub index: usize,
}

/// Runtime state for executing a pipeline
pub struct PipelineExecution {
    /// Child processes (one per segment)
    pub children: Vec<Child>,
    /// Pipe connections between segments
    pub pipes: Vec<PipeConnection>,
}

/// I/O connection between two commands
pub struct PipeConnection {
    /// Source command index
    pub from: usize,
    /// Destination command index
    pub to: usize,
    // Actual pipe handles managed by std::process::Child
}
```

**Rationale**:
- `Pipeline`: High-level representation, parsed from command line
- `PipelineSegment`: Individual command within pipeline (maps to one `std::process::Command`)
- `PipelineExecution`: Runtime state during execution (child processes, pipes)
- Separation of parsing (Pipeline) from execution (PipelineExecution) enables testing

**Existing Foundation**:
- Current `Command` struct: [executor/mod.rs:18-51](../../../crates/rush/src/executor/mod.rs#L18-L51)
- Already has `Operator::Pipe` enum variant (line 63)
- Need to wire up parsing and execution logic

---

### 8. How do we maintain <5ms execution overhead?

**Answer**: Minimize allocations, avoid unnecessary copies, leverage OS pipes.

**Performance Strategy**:
1. **Reuse existing parser**: Current tokenizer is fast (<1ms for typical commands)
2. **Zero-copy where possible**: Pass `&str` references until command spawn
3. **Batch spawns**: Spawn all processes in tight loop (minimize syscall overhead)
4. **No buffering layer**: Use OS pipes directly (no intermediate buffers)
5. **Async waits** (future): Use tokio for concurrent waits (deferred to v0.3.0+)

**Benchmarking Plan**:
1. Measure baseline: `cargo bench -- execute_simple`
2. Add pipe support
3. Measure regression: `cargo bench -- execute_pipeline`
4. Target: Pipeline overhead <5ms vs. running commands individually

**Constitution Compliance**:
- SC-003: "Pipeline execution completes within 5% overhead"
- SC-006: "Pipeline parsing completes in <1ms for typical command lines"

**Measurement Points**:
```rust
let start = Instant::now();
// Parse pipeline
let parse_time = start.elapsed();  // Target: <1ms

// Execute pipeline
let exec_start = Instant::now();
// ... spawn, wait ...
let exec_time = exec_start.elapsed();  // Target: <5ms overhead

tracing::info!(
    parse_ms = parse_time.as_millis(),
    exec_ms = exec_time.as_millis(),
    "Pipeline performance"
);
```

---

### 9. What testing strategy should we use?

**Answer**: Three-tier testing - unit, integration, and contract tests.

**Testing Tiers**:

**1. Unit Tests** (Fast, isolated)
- Parser: Pipe detection, quote handling, malformed syntax
- Data structures: Pipeline construction, segment validation
- Location: `crates/rush/tests/unit/pipe_parser_tests.rs`

**2. Integration Tests** (Real commands, real pipes)
- Basic two-command pipeline: `echo hello | grep hello`
- Multi-command pipeline: `cat file | grep pattern | wc -l`
- Large data: `dd if=/dev/zero bs=1M count=100 | wc -c`
- Binary data: `tar czf - . | tar tzf -`
- Error handling: `ls /nonexistent | grep foo`
- Signal handling: Spawn pipeline, send SIGINT, verify cleanup
- Location: `crates/rush/tests/integration/pipe_tests.rs`

**3. Contract Tests** (User story validation)
- US1-AS1: `ls | grep txt` filters correctly
- US1-AS2: `echo "test" | wc -l` returns 1
- US2-AS1: `cat file | grep error | wc -l` counts filtered lines
- US3-AS1: `ls /nonexistent | grep foo` shows error
- US4-AS1: `true | false` returns exit code 1
- Location: `specs/004-pipes/contracts/`

**Existing Test Patterns**:
- Parser tests: [parser.rs:118-248](../../../crates/rush/src/executor/parser.rs#L118-L248)
- Integration tests: [autosuggestions_tests.rs](../../../crates/rush/tests/integration/autosuggestions_tests.rs)
- Follow same structure for pipe tests

---

### 10. What are the implementation risks?

**Answer**: Three primary risks identified with mitigation strategies.

**Risk 1: Zombie Processes**
- **Probability**: Medium
- **Impact**: High (leaked resources, user confusion)
- **Mitigation**:
  - Always call `child.wait()` for all spawned processes
  - Use RAII pattern: Wait in Drop implementation if not explicitly waited
  - Test: Integration test spawns pipeline, kills rush, checks for zombies

**Risk 2: Pipe Deadlock**
- **Probability**: Low (OS handles most cases)
- **Impact**: High (pipeline hangs indefinitely)
- **Scenario**: Circular pipe (impossible with linear pipeline)
- **Mitigation**:
  - Validate pipeline structure (no cycles)
  - Set timeouts for testing (detect hangs early)
  - Test: Long-running pipeline with fast Ctrl+C

**Risk 3: Performance Regression**
- **Probability**: Medium
- **Impact**: Medium (violates constitution)
- **Mitigation**:
  - Benchmark before/after pipeline implementation
  - Profile with `cargo flamegraph` if overhead exceeds 5ms
  - Use `criterion` for statistical benchmarking
  - Test: Benchmark suite in `benches/pipeline_bench.rs`

**Risk 4: Platform-Specific Behavior**
- **Probability**: Medium (macOS-only for MVP)
- **Impact**: Low (future work for Linux/Windows)
- **Mitigation**:
  - Document macOS-specific assumptions
  - Use `#[cfg(unix)]` for Unix-specific code
  - Defer cross-platform to v0.2.0+
  - Test: CI runs on macOS only for MVP

---

## Technical Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Pipe Mechanism** | `Stdio::piped()` | Idiomatic Rust, OS-native, binary-safe |
| **Execution Model** | Concurrent | Performance-First principle, streaming support |
| **Backpressure** | OS pipes (automatic) | Zero custom logic, Unix semantics |
| **Signal Handling** | Process groups | Standard Unix, clean termination |
| **Exit Code** | Last command | POSIX standard, user expectation |
| **Parser Approach** | Extend tokenizer | Reuse existing, proven code |
| **Data Model** | Pipeline/Segment/Execution | Clean separation, testable |
| **Performance** | <5ms overhead | Constitution requirement, benchmarked |
| **Testing** | Unit/Integration/Contract | Comprehensive coverage, spec-aligned |

---

## Open Questions (Deferred)

These questions are out of scope for MVP but documented for future work:

1. **PIPESTATUS support** (bash feature) - Deferred to v0.3.0+
   - Requires: Array of all exit codes
   - Use case: Debugging which stage failed
   - Complexity: Medium

2. **Pipe to/from files** (`<`, `>`) - Deferred to v0.2.0 (separate feature)
   - Specification exists for redirections
   - This feature only handles `|` operator

3. **Named pipes (FIFOs)** - Deferred to v1.0+
   - Advanced feature, niche use case
   - Requires: `mkfifo` support, bidirectional pipes

4. **Pipeline introspection** (debugging) - Deferred to v0.3.0+
   - Show pipeline structure: `rush --explain "ls | grep foo"`
   - Performance monitoring: Per-stage timing

5. **Async execution with tokio** - Deferred to v0.3.0+
   - Current: Synchronous spawns and waits
   - Future: Async for better concurrency (job control integration)

---

## Constitution Compliance Check

| Principle | Compliance | Evidence |
|-----------|-----------|----------|
| **I. Performance-First** | ✅ PASS | Concurrent execution, <5ms overhead, OS pipes |
| **II. Zero-Config** | ✅ PASS | Pipes work immediately, no configuration required |
| **III. Progressive Complexity** | ✅ PASS | Basic pipes (P1) before advanced features (P2-P4) |
| **IV. Modern UX** | ✅ PASS | Error messages, signal handling, familiar semantics |
| **V. Rust-Native** | ✅ PASS | `std::process`, no FFI, idiomatic patterns |

**Result**: All principles satisfied. Proceed to Phase 1.

---

## Next Steps

Phase 1 will deliver:
1. **data-model.md** - Complete Rust data structures with examples
2. **contracts/** - Test files for each user story acceptance scenario
3. **quickstart.md** - 5-minute guide for using pipes in rush

Proceed to Phase 1 design.
