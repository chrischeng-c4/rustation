# Research: Output Redirection Implementation

**Feature**: 005-output-redirection
**Date**: 2025-11-20
**Purpose**: Technical research and design decisions for implementing I/O redirection operators

## Research Questions

### Q1: How to integrate redirections with existing parser tokenization?

**Decision**: Extend the existing `Token` enum to include redirection operators

**Rationale**:
- Feature 004 (pipes) already uses token-based parsing with `Token::Word` and `Token::Pipe`
- Adding `Token::RedirectOut`, `Token::RedirectAppend`, `Token::RedirectIn` maintains consistency
- Tokenization happens first, then tokens are grouped into segments
- Quote-aware tokenization already exists - just needs to recognize additional special characters

**Alternatives considered**:
1. **Parse redirections after segmentation** - Rejected because it complicates parsing logic and makes quote handling harder
2. **Use regex-based parsing** - Rejected because token-based approach is already proven and performant
3. **Separate redirection parser** - Rejected because it duplicates quote-handling logic

**Implementation approach**:
```rust
// In parser.rs Token enum
enum Token {
    Word(String),
    Pipe,
    RedirectOut,      // >
    RedirectAppend,   // >>
    RedirectIn,       // <
}
```

**Key insight**: `>>` must be recognized as single token (not two `>` tokens). Check for `>>` first, then fall back to single `>`.

---

### Q2: How to represent redirections in the data model?

**Decision**: Create a `Redirection` struct with type, file path, and target descriptor

**Rationale**:
- Need to store both the operation type (`>`, `>>`, `<`) and the target file path
- Each command/segment can have multiple redirections (e.g., `< in.txt cmd > out.txt`)
- Redirections should be validated independently before execution
- Type-safe representation prevents invalid combinations

**Alternatives considered**:
1. **Store as strings and parse at execution time** - Rejected because it defers validation and makes testing harder
2. **Separate Input/Output redirection types** - Rejected because they share common structure (type + path)
3. **Embed redirections in PipelineSegment directly** - Accepted as extension to existing model

**Implementation approach**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum RedirectionType {
    Output,        // >
    Append,        // >>
    Input,         // <
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirection {
    pub redir_type: RedirectionType,
    pub file_path: String,
}

// Extend PipelineSegment
pub struct PipelineSegment {
    pub program: String,
    pub args: Vec<String>,
    pub index: usize,
    pub redirections: Vec<Redirection>,  // NEW
}
```

**Key insight**: Multiple redirections are allowed (spec requirement FR-019: last one wins for output). Store as Vec to preserve order.

---

### Q3: How to set up file descriptors before process execution?

**Decision**: Use `std::process::Stdio::from(File)` to convert opened files into stdio handles

**Rationale**:
- Rust's `Command` builder accepts `Stdio` objects for stdin/stdout/stderr
- `std::fs::File` implements `Into<Stdio>` on Unix platforms
- File opening can fail early (permissions, not found) before spawning process
- Direct file descriptor handoff - no buffering or copying needed

**Alternatives considered**:
1. **Use dup2 directly via libc** - Rejected because `Stdio::from()` provides safe abstraction
2. **Pipe data through parent process** - Rejected because it adds overhead and complexity
3. **Use shell redirection (sh -c)** - Rejected because it defeats purpose of native shell

**Implementation approach**:
```rust
use std::fs::{File, OpenOptions};
use std::process::Stdio;

// For output redirection (>)
let file = File::create(path)?;
command.stdout(Stdio::from(file));

// For append redirection (>>)
let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)?;
command.stdout(Stdio::from(file));

// For input redirection (<)
let file = File::open(path)?;
command.stdin(Stdio::from(file));
```

**Key insight**: File operations must happen in PipelineExecution::spawn() before command.spawn(), enabling early error detection.

---

### Q4: How to handle multiple redirections (e.g., `cmd > a.txt > b.txt`)?

**Decision**: Last redirection wins for each stream (stdout/stdin)

**Rationale**:
- Matches POSIX shell behavior (bash, zsh)
- Spec requirement FR-019 explicitly states "last one wins"
- Simplifies implementation - process redirections left-to-right, overwriting previous

**Alternatives considered**:
1. **Error on multiple redirections** - Rejected because it's more restrictive than POSIX
2. **Redirect to all files (tee behavior)** - Rejected because it changes semantics
3. **First wins** - Rejected because it contradicts POSIX and user expectations

**Implementation approach**:
```rust
let mut stdin_redirect: Option<Redirection> = None;
let mut stdout_redirect: Option<Redirection> = None;

for redir in &segment.redirections {
    match redir.redir_type {
        RedirectionType::Input => stdin_redirect = Some(redir.clone()),
        RedirectionType::Output | RedirectionType::Append => stdout_redirect = Some(redir.clone()),
    }
}

// Apply only the last redirect for each stream
```

**Key insight**: Process redirections sequentially, keeping only the last one for each file descriptor.

---

### Q5: How to preserve operators inside quotes?

**Decision**: Extend existing quote-aware tokenizer to treat `>`, `>>`, `<` as special characters only outside quotes

**Rationale**:
- Feature 004 (pipes) already handles `|` this way
- Quote state tracking already exists in tokenizer
- Consistent with user expectations: `echo "a > b"` should output "a > b"

**Alternatives considered**:
1. **Always parse as operators** - Rejected because breaks quoted strings
2. **Escape sequences (`\>`)** - Rejected because quotes are more natural
3. **Separate quote stripping pass** - Rejected because tokenizer already tracks quote state

**Implementation approach**:
```rust
// In tokenize_with_redirections()
for ch in line.chars() {
    match ch {
        '\'' if !in_double_quote && !escaped => {
            in_single_quote = !in_single_quote;
        }
        '"' if !in_single_quote && !escaped => {
            in_double_quote = !in_double_quote;
        }
        '>' if !in_single_quote && !in_double_quote && !escaped => {
            // Check for >>
            if next_char == '>' {
                emit(Token::RedirectAppend);
                skip_next = true;
            } else {
                emit(Token::RedirectOut);
            }
        }
        '<' if !in_single_quote && !in_double_quote && !escaped => {
            emit(Token::RedirectIn);
        }
        // ... other cases
    }
}
```

**Key insight**: Reuse existing quote state tracking, add redirection character checks.

---

### Q6: How to provide clear error messages for file operations?

**Decision**: Match on `io::ErrorKind` and provide context-specific messages

**Rationale**:
- Rust's `io::Error` includes `ErrorKind` enum with detailed error types
- Spec requirement FR-021/FR-022: errors must be clear (permissions, not found, is directory)
- Early error detection (before spawn) enables better error messages

**Alternatives considered**:
1. **Generic error message** - Rejected because spec requires specificity
2. **Raw OS error codes** - Rejected because not user-friendly
3. **Try/catch at execution** - Rejected because early validation is better

**Implementation approach**:
```rust
use std::io::ErrorKind;

match file_result {
    Err(e) => {
        let msg = match e.kind() {
            ErrorKind::NotFound => format!("{}: file not found", path),
            ErrorKind::PermissionDenied => format!("{}: permission denied", path),
            ErrorKind::IsADirectory => format!("{}: is a directory", path),
            ErrorKind::InvalidInput => format!("{}: invalid file path", path),
            _ => format!("{}: {}", path, e),
        };
        return Err(RushError::Redirection(msg));
    }
    Ok(file) => file,
}
```

**Key insight**: Map ErrorKind to user-friendly messages matching bash/zsh conventions.

---

### Q7: How to handle redirection order (`< in.txt cmd` vs `cmd < in.txt`)?

**Decision**: Allow redirection tokens anywhere in command line, associate with adjacent command

**Rationale**:
- POSIX shells allow flexible ordering
- Spec requirement FR-016: `<` can appear before or after command
- Simplifies parsing - collect all redirections for a segment regardless of position

**Alternatives considered**:
1. **Require redirections after command** - Rejected because too restrictive
2. **Require redirections before command** - Rejected because unnatural for output
3. **Different syntax for position** - Rejected because complicates parser

**Implementation approach**:
```rust
// In split_into_segments()
// Collect all tokens between pipe boundaries
// Separate Word tokens (command + args) from Redirection tokens
// First Word is program, rest are args, all Redirections added to segment

let mut words = Vec::new();
let mut redirections = Vec::new();

for token in segment_tokens {
    match token {
        Token::Word(w) => words.push(w),
        Token::RedirectOut | Token::RedirectAppend | Token::RedirectIn => {
            // Next word is file path
            redirections.push(Redirection { type, path });
        }
        _ => {}
    }
}
```

**Key insight**: Position doesn't matter - just collect all redirections and apply them before execution.

---

### Q8: Performance - where to open files to minimize overhead?

**Decision**: Open files in PipelineExecution::spawn() after command creation, before command.spawn()

**Rationale**:
- Opening files is I/O operation - should not block parent shell
- spawn() already creates child processes - natural place for file operations
- Early opening enables fail-fast before spawning process
- Constitution requires <1ms overhead - file operations are fast enough

**Alternatives considered**:
1. **Open in parent before fork** - Rejected because blocks parent shell
2. **Open in child after fork** - Accepted (this is what spawn() does)
3. **Lazy open on first write** - Rejected because complicates error handling

**Implementation approach**:
```rust
// In PipelineExecution::spawn()
for segment in pipeline.segments {
    let mut command = Command::new(&segment.program);
    command.args(&segment.args);

    // Apply redirections BEFORE command.spawn()
    for redir in &segment.redirections {
        match redir.redir_type {
            RedirectionType::Output => {
                let file = File::create(&redir.file_path)?;  // Fail fast if error
                command.stdout(Stdio::from(file));
            }
            // ... other types
        }
    }

    let child = command.spawn()?;  // Already has redirections configured
    children.push(child);
}
```

**Key insight**: File operations between Command creation and spawn() provide early validation without blocking parent.

---

## Technology Stack Summary

### Core Dependencies (Rust Standard Library)

1. **std::process** (Command, Stdio, Child)
   - Purpose: Process spawning with custom file descriptors
   - Why: Already used in feature 004 (pipes), proven approach
   - Version: Rust 1.75+ stdlib

2. **std::fs** (File, OpenOptions)
   - Purpose: File creation, opening, truncation, appending
   - Why: Safe Rust abstractions for file operations
   - Version: Rust 1.75+ stdlib

3. **std::os::unix::io** (AsRawFd, FromRawFd, IntoRawFd)
   - Purpose: File descriptor manipulation for Unix platforms
   - Why: Required for Stdio::from(File) conversion
   - Version: Rust 1.75+ stdlib (Unix-only)

4. **std::io** (ErrorKind, Result, Error)
   - Purpose: Error handling for file operations
   - Why: Standard Rust error handling patterns
   - Version: Rust 1.75+ stdlib

### Testing Stack

1. **cargo test** (built-in)
   - Unit tests: Parser, data structures, validation
   - Integration tests: End-to-end command execution with redirections
   - Contract tests: Validate spec requirements

2. **cargo bench** (built-in with criterion)
   - Performance benchmarks: Redirection setup overhead
   - Comparison: Redirected vs non-redirected execution

### Build/Development Tools

1. **cargo fmt** - Code formatting
2. **cargo clippy** - Linting
3. **rustfmt** - Formatting rules

## Best Practices

### File Operation Safety

1. **Early Validation**: Open files before spawning processes to fail fast
2. **Error Context**: Provide file path in all error messages
3. **Resource Cleanup**: Files automatically closed when dropped (RAII)
4. **Permission Check**: Let OS handle permissions, catch and report errors

### Parser Design

1. **Token-Based**: Maintain existing token-based parsing approach
2. **Quote Awareness**: Respect quote boundaries for operator detection
3. **Lookahead**: Check for `>>` before treating as single `>`
4. **Validation**: Validate redirection syntax during parsing, not execution

### Performance Optimization

1. **No Buffering**: Direct file descriptor handoff via Stdio::from()
2. **Minimal Allocation**: Reuse existing String allocations where possible
3. **Early Termination**: Stop parsing on syntax errors
4. **Benchmarking**: Measure redirection setup overhead separately

### Error Handling

1. **Specific Messages**: Map ErrorKind to user-friendly messages
2. **Context Preservation**: Include file path and operation type in errors
3. **Early Detection**: Validate before spawning to provide better errors
4. **Graceful Degradation**: Clean up partial state on errors

## Integration Points

### With Existing Features

1. **Feature 001 (MVP)**: Extends CommandExecutor through PipelineExecutor
2. **Feature 004 (Pipes)**: Redirections work with pipes (`cat < file | grep x > out`)
3. **Quote Parser**: Reuses existing quote-aware tokenization logic
4. **Error System**: Uses existing RushError enum (add Redirection variant)

### Future Features

1. **Job Control**: Redirections should work with background jobs (`cmd > log.txt &`)
2. **Stderr Redirection**: Foundation enables future `2>` operator
3. **Here Documents**: Parser infrastructure supports future `<<` operator

## Performance Considerations

### Benchmarking Strategy

1. **Baseline**: Measure command execution without redirections
2. **With Redirection**: Measure same command with `> output.txt`
3. **Overhead**: Calculate difference (target: <1ms)
4. **Large Files**: Test append performance with existing multi-GB files

### Expected Performance

- File open/create: ~0.1-0.5ms (OS-dependent)
- Stdio setup: ~0.01ms (in-memory operation)
- Total overhead: ~0.1-0.6ms (well under 1ms target)

### Optimization Opportunities

1. **Path Caching**: Not needed (file operations infrequent)
2. **File Pooling**: Not needed (files are process-specific)
3. **Async I/O**: Not needed (setup is synchronous, execution is async)

## Risk Assessment

### Low Risk

✅ **Technology maturity**: Rust stdlib is stable and well-tested
✅ **Integration**: Extends existing proven architecture (pipes)
✅ **Performance**: File operations are inherently fast
✅ **Testing**: Testable at multiple levels (unit, integration, contract)

### Medium Risk

⚠️ **Platform-specific behavior**: macOS vs Linux file semantics may differ slightly
  - Mitigation: Follow POSIX standards, test on target platform
  - Note: MVP is macOS-only, Linux support deferred

⚠️ **Quote handling complexity**: Three operators to recognize, multiple quote types
  - Mitigation: Reuse existing quote parser, add comprehensive tests
  - Note: Parser already handles pipes this way successfully

### Negligible Risk

✅ **Backward compatibility**: New feature doesn't affect existing commands
✅ **Error handling**: Rust's Result type forces explicit error handling
✅ **Resource leaks**: RAII guarantees cleanup

## Decision Summary

| Question | Decision | Confidence |
|----------|----------|------------|
| Q1: Parser integration | Extend Token enum | HIGH (proven approach) |
| Q2: Data model | Redirection struct in PipelineSegment | HIGH (type-safe, testable) |
| Q3: File descriptors | Stdio::from(File) | HIGH (idiomatic Rust) |
| Q4: Multiple redirections | Last wins | HIGH (POSIX standard) |
| Q5: Quote handling | Extend existing tokenizer | HIGH (reuses proven code) |
| Q6: Error messages | Match ErrorKind | HIGH (user-friendly) |
| Q7: Redirection order | Position-independent | HIGH (POSIX compatible) |
| Q8: Performance | Open in spawn() | HIGH (<1ms overhead) |

**Overall Confidence**: **HIGH** - All decisions based on proven approaches, existing infrastructure, and Rust stdlib capabilities.

---

**Research Complete**: All technical questions resolved. Ready for Phase 1 (Design).
