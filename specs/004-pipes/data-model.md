# Phase 1: Data Model - Pipe Operator Support

**Feature**: 004-pipes
**Date**: 2025-11-19
**Status**: Complete

## Overview

This document defines the Rust data structures for pipeline execution. The model separates **parsing** (building the pipeline structure) from **execution** (spawning processes and managing I/O).

---

## Core Data Structures

### 1. Pipeline

Represents a complete parsed pipeline ready for execution.

```rust
/// A complete pipeline parsed from user input
///
/// Example: "ls -la | grep txt | wc -l" becomes:
/// Pipeline {
///     segments: [
///         PipelineSegment { program: "ls", args: ["-la"], index: 0 },
///         PipelineSegment { program: "grep", args: ["txt"], index: 1 },
///         PipelineSegment { program: "wc", args: ["-l"], index: 2 },
///     ],
///     raw_input: "ls -la | grep txt | wc -l",
/// }
#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    /// Individual commands in the pipeline
    pub segments: Vec<PipelineSegment>,

    /// Original user input for error messages and logging
    pub raw_input: String,
}

impl Pipeline {
    /// Create a new pipeline from segments
    pub fn new(segments: Vec<PipelineSegment>, raw_input: String) -> Self {
        Self { segments, raw_input }
    }

    /// Number of commands in the pipeline
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Check if pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Validate pipeline structure
    ///
    /// Returns Ok(()) if valid, Err with reason if invalid.
    pub fn validate(&self) -> Result<()> {
        if self.is_empty() {
            return Err(RushError::Execution("Empty pipeline".to_string()));
        }

        for segment in &self.segments {
            segment.validate()?;
        }

        Ok(())
    }
}
```

**Design Notes**:
- `Clone` enables retrying failed pipelines
- `PartialEq` enables testing
- `raw_input` preserved for error reporting
- Validation separate from construction (fail early pattern)

---

### 2. PipelineSegment

Represents one command within a pipeline.

```rust
/// One command in a pipeline
///
/// Example: In "ls -la | grep txt", the first segment is:
/// PipelineSegment {
///     program: "ls",
///     args: ["-la"],
///     index: 0,
/// }
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineSegment {
    /// Command name (e.g., "ls", "grep")
    pub program: String,

    /// Command arguments (e.g., ["-la"], ["txt"])
    pub args: Vec<String>,

    /// Position in pipeline (0-indexed)
    /// First command is 0, second is 1, etc.
    pub index: usize,
}

impl PipelineSegment {
    /// Create a new pipeline segment
    pub fn new(program: String, args: Vec<String>, index: usize) -> Self {
        Self { program, args, index }
    }

    /// Validate segment
    pub fn validate(&self) -> Result<()> {
        if self.program.is_empty() {
            return Err(RushError::Execution(
                format!("Empty program at position {}", self.index)
            ));
        }
        Ok(())
    }

    /// Check if this is the first segment in a pipeline
    pub fn is_first(&self) -> bool {
        self.index == 0
    }

    /// Check if this is the last segment in a pipeline
    pub fn is_last(&self, pipeline_len: usize) -> bool {
        self.index == pipeline_len - 1
    }
}
```

**Design Notes**:
- `index` enables position-aware logic (first/last handling)
- Immutable after construction (functional style)
- Validation checks for empty program

---

### 3. PipelineExecutor

Manages the execution of a pipeline (spawning processes, managing pipes).

```rust
/// Executes pipelines by spawning processes and connecting pipes
pub struct PipelineExecutor {
    // No state needed - executor is stateless
}

impl PipelineExecutor {
    /// Create a new pipeline executor
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a pipeline and return the last command's exit code
    ///
    /// # Arguments
    /// * `pipeline` - The pipeline to execute
    ///
    /// # Returns
    /// * `Ok(exit_code)` - Exit code from last command (0 for success)
    /// * `Err(_)` - If pipeline execution failed
    ///
    /// # Example
    /// ```ignore
    /// let executor = PipelineExecutor::new();
    /// let pipeline = parse_pipeline("ls | grep txt")?;
    /// let exit_code = executor.execute(&pipeline)?;
    /// ```
    pub fn execute(&self, pipeline: &Pipeline) -> Result<i32> {
        // Validate pipeline structure
        pipeline.validate()?;

        // Special case: Single command (no pipes)
        if pipeline.len() == 1 {
            return self.execute_single(&pipeline.segments[0]);
        }

        // Execute multi-command pipeline
        self.execute_pipeline(pipeline)
    }

    /// Execute a single command (no pipes)
    fn execute_single(&self, segment: &PipelineSegment) -> Result<i32> {
        // Delegate to existing CommandExecutor logic
        // (This is the current execute.rs implementation)
        todo!("Reuse existing single-command execution")
    }

    /// Execute a multi-command pipeline
    fn execute_pipeline(&self, pipeline: &Pipeline) -> Result<i32> {
        // Implementation in next section
        todo!("Pipeline execution logic")
    }
}
```

**Design Notes**:
- Stateless executor (can be reused for multiple pipelines)
- Single-command optimization (avoid pipe overhead when not needed)
- Clear separation between single and multi-command execution

---

### 4. PipelineExecution (Internal)

Runtime state during pipeline execution. Not exposed in public API.

```rust
/// Internal state during pipeline execution
struct PipelineExecution {
    /// Spawned child processes (one per segment)
    children: Vec<std::process::Child>,

    /// Pipeline being executed (for logging and errors)
    pipeline: Pipeline,
}

impl PipelineExecution {
    /// Spawn all commands in the pipeline with pipes connected
    fn spawn(pipeline: &Pipeline) -> Result<Self> {
        let mut children = Vec::with_capacity(pipeline.len());
        let mut prev_stdout: Option<std::process::ChildStdout> = None;

        for (i, segment) in pipeline.segments.iter().enumerate() {
            let mut command = std::process::Command::new(&segment.program);
            command.args(&segment.args);

            // First command: stdin from terminal
            // Middle/last commands: stdin from previous command's stdout
            if let Some(stdout) = prev_stdout.take() {
                command.stdin(stdout);
            } else {
                command.stdin(std::process::Stdio::inherit());
            }

            // Last command: stdout to terminal
            // Other commands: stdout to pipe
            if i == pipeline.len() - 1 {
                command.stdout(std::process::Stdio::inherit());
            } else {
                command.stdout(std::process::Stdio::piped());
            }

            // All commands: stderr to terminal
            command.stderr(std::process::Stdio::inherit());

            // Spawn process
            let mut child = command.spawn().map_err(|e| {
                RushError::Execution(format!(
                    "Failed to spawn {}: {}",
                    segment.program, e
                ))
            })?;

            // Save stdout for next command
            if i < pipeline.len() - 1 {
                prev_stdout = child.stdout.take();
            }

            children.push(child);
        }

        Ok(Self {
            children,
            pipeline: pipeline.clone(),
        })
    }

    /// Wait for all processes to complete and return last exit code
    fn wait_all(mut self) -> Result<i32> {
        let mut last_exit_code = 0;

        for (i, mut child) in self.children.into_iter().enumerate() {
            match child.wait() {
                Ok(status) => {
                    let exit_code = status.code().unwrap_or(1);

                    tracing::debug!(
                        command = %self.pipeline.segments[i].program,
                        exit_code,
                        position = i,
                        "Pipeline segment completed"
                    );

                    // Save exit code from last command
                    if i == self.pipeline.len() - 1 {
                        last_exit_code = exit_code;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        command = %self.pipeline.segments[i].program,
                        error = %e,
                        "Failed to wait for pipeline segment"
                    );
                    return Err(RushError::Execution(format!(
                        "Failed to wait for {}: {}",
                        self.pipeline.segments[i].program, e
                    )));
                }
            }
        }

        Ok(last_exit_code)
    }
}
```

**Design Notes**:
- Private struct (not exposed in public API)
- RAII pattern: Processes spawned and waited in structured way
- Clear stdin/stdout/stderr handling based on position in pipeline
- Comprehensive logging for debugging

---

## Parser Integration

### Extended Tokenizer

Modify existing [parser.rs](../../../crates/rush/src/executor/parser.rs) to detect pipes.

```rust
/// Token types after parsing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// Regular word (command, argument)
    Word(String),

    /// Pipe operator
    Pipe,
}

/// Parse command line into pipeline
///
/// Examples:
/// - "ls" -> single-segment pipeline
/// - "ls | grep txt" -> two-segment pipeline
/// - "echo \"a | b\"" -> single-segment (pipe inside quotes)
pub fn parse_pipeline(line: &str) -> Result<Pipeline> {
    // Tokenize with pipe detection
    let tokens = tokenize_with_pipes(line)?;

    // Split tokens at pipe boundaries
    let segments = split_into_segments(tokens)?;

    // Build pipeline
    let pipeline = Pipeline::new(segments, line.to_string());

    // Validate
    pipeline.validate()?;

    Ok(pipeline)
}

/// Tokenize command line, recognizing pipes as special tokens
fn tokenize_with_pipes(line: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in line.chars() {
        if escape_next {
            current_word.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => escape_next = true,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '|' if !in_single_quote && !in_double_quote => {
                // Pipe outside quotes - emit current word and Pipe token
                if !current_word.is_empty() {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                }
                tokens.push(Token::Pipe);
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                // Whitespace outside quotes - end word
                if !current_word.is_empty() {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                }
            }
            _ => {
                current_word.push(ch);
            }
        }
    }

    // Validation
    if in_single_quote {
        return Err(RushError::Execution("Unclosed single quote".to_string()));
    }
    if in_double_quote {
        return Err(RushError::Execution("Unclosed double quote".to_string()));
    }

    // Emit final word
    if !current_word.is_empty() {
        tokens.push(Token::Word(current_word));
    }

    Ok(tokens)
}

/// Split tokens into pipeline segments at pipe boundaries
fn split_into_segments(tokens: Vec<Token>) -> Result<Vec<PipelineSegment>> {
    let mut segments = Vec::new();
    let mut current_segment: Vec<String> = Vec::new();

    for token in tokens {
        match token {
            Token::Word(word) => {
                current_segment.push(word);
            }
            Token::Pipe => {
                // Validate segment before pipe
                if current_segment.is_empty() {
                    return Err(RushError::Execution(
                        "Empty command before pipe".to_string()
                    ));
                }

                // Create segment
                let program = current_segment[0].clone();
                let args = current_segment[1..].to_vec();
                segments.push(PipelineSegment::new(
                    program,
                    args,
                    segments.len(),
                ));

                current_segment.clear();
            }
        }
    }

    // Validate final segment
    if current_segment.is_empty() {
        return Err(RushError::Execution(
            "Empty command after pipe".to_string()
        ));
    }

    // Add final segment
    let program = current_segment[0].clone();
    let args = current_segment[1..].to_vec();
    segments.push(PipelineSegment::new(
        program,
        args,
        segments.len(),
    ));

    Ok(segments)
}
```

**Integration Points**:
- Reuses existing quote/escape handling from parser.rs
- Adds pipe detection without breaking existing functionality
- Maintains error messages from current parser

---

## Execution Flow

### Complete Pipeline Execution

```rust
impl PipelineExecutor {
    fn execute_pipeline(&self, pipeline: &Pipeline) -> Result<i32> {
        tracing::info!(
            segments = pipeline.len(),
            raw_input = %pipeline.raw_input,
            "Executing pipeline"
        );

        // Spawn all processes with pipes connected
        let execution = PipelineExecution::spawn(pipeline)?;

        // Wait for all to complete and get last exit code
        let exit_code = execution.wait_all()?;

        tracing::info!(
            exit_code,
            "Pipeline completed"
        );

        Ok(exit_code)
    }
}
```

---

## Example Usage

### Basic Two-Command Pipeline

```rust
// User input: "ls | grep txt"
let pipeline = parse_pipeline("ls | grep txt")?;

// Pipeline structure:
// Pipeline {
//     segments: [
//         PipelineSegment { program: "ls", args: [], index: 0 },
//         PipelineSegment { program: "grep", args: ["txt"], index: 1 },
//     ],
//     raw_input: "ls | grep txt",
// }

let executor = PipelineExecutor::new();
let exit_code = executor.execute(&pipeline)?;
// exit_code = grep's exit code (0 if matches found, 1 if no matches)
```

### Multi-Command Pipeline

```rust
// User input: "cat file.txt | grep error | wc -l"
let pipeline = parse_pipeline("cat file.txt | grep error | wc -l")?;

// Pipeline structure:
// Pipeline {
//     segments: [
//         PipelineSegment { program: "cat", args: ["file.txt"], index: 0 },
//         PipelineSegment { program: "grep", args: ["error"], index: 1 },
//         PipelineSegment { program: "wc", args: ["-l"], index: 2 },
//     ],
//     raw_input: "cat file.txt | grep error | wc -l",
// }

let executor = PipelineExecutor::new();
let exit_code = executor.execute(&pipeline)?;
// exit_code = wc's exit code (0 for success)
```

### Single Command (No Pipes)

```rust
// User input: "ls -la"
let pipeline = parse_pipeline("ls -la")?;

// Pipeline structure:
// Pipeline {
//     segments: [
//         PipelineSegment { program: "ls", args: ["-la"], index: 0 },
//     ],
//     raw_input: "ls -la",
// }

let executor = PipelineExecutor::new();
let exit_code = executor.execute(&pipeline)?;
// Optimized: Uses single-command path (no pipe overhead)
```

### Pipe Inside Quotes

```rust
// User input: echo "hello | world"
let pipeline = parse_pipeline("echo \"hello | world\"")?;

// Pipeline structure:
// Pipeline {
//     segments: [
//         PipelineSegment { program: "echo", args: ["hello | world"], index: 0 },
//     ],
//     raw_input: "echo \"hello | world\"",
// }

// Result: Single command, pipe treated as literal text
```

### Error Cases

```rust
// Empty command before pipe
let result = parse_pipeline("| grep foo");
assert!(result.is_err());
// Error: "Empty command before pipe"

// Empty command after pipe
let result = parse_pipeline("ls |");
assert!(result.is_err());
// Error: "Empty command after pipe"

// Double pipe
let result = parse_pipeline("ls | | grep");
assert!(result.is_err());
// Error: "Empty command before pipe"
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_command() {
        let pipeline = parse_pipeline("ls").unwrap();
        assert_eq!(pipeline.len(), 1);
        assert_eq!(pipeline.segments[0].program, "ls");
    }

    #[test]
    fn test_parse_two_command_pipeline() {
        let pipeline = parse_pipeline("ls | grep txt").unwrap();
        assert_eq!(pipeline.len(), 2);
        assert_eq!(pipeline.segments[0].program, "ls");
        assert_eq!(pipeline.segments[1].program, "grep");
        assert_eq!(pipeline.segments[1].args, vec!["txt"]);
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

    #[test]
    fn test_pipeline_segment_is_first() {
        let seg = PipelineSegment::new("ls".to_string(), vec![], 0);
        assert!(seg.is_first());
    }

    #[test]
    fn test_pipeline_segment_is_last() {
        let seg = PipelineSegment::new("wc".to_string(), vec![], 2);
        assert!(seg.is_last(3));
        assert!(!seg.is_last(4));
    }
}
```

---

## Migration Path

### Current Code → Pipeline Code

**Before** (current single-command execution):
```rust
// crates/rush/src/executor/execute.rs
let (program, args) = parse_command_line(line)?;
let child = Command::new(&program)
    .args(&args)
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()?;
let status = child.wait()?;
```

**After** (pipeline-aware execution):
```rust
// Parse as pipeline (may have 1 or more segments)
let pipeline = parse_pipeline(line)?;

// Execute (handles both single-command and multi-command)
let executor = PipelineExecutor::new();
let exit_code = executor.execute(&pipeline)?;
```

**Compatibility**:
- Single commands work unchanged (optimized path)
- Multi-command pipelines use new logic
- Error messages remain similar
- Exit codes behave identically

---

## File Locations

```
crates/rush/src/executor/
├── mod.rs                # Add Pipeline, PipelineSegment exports
├── parser.rs             # Extend with parse_pipeline(), tokenize_with_pipes()
├── execute.rs            # Refactor to use PipelineExecutor
└── pipeline.rs           # NEW: PipelineExecutor, PipelineExecution

crates/rush/tests/unit/
└── pipe_parser_tests.rs  # NEW: Parser unit tests

crates/rush/tests/integration/
└── pipe_tests.rs         # NEW: Integration tests
```

---

## Performance Considerations

### Memory Allocations

- **Parsing**: `O(n)` allocations where n = number of tokens
  - Reuses existing tokenizer (already optimized)
  - One `Pipeline` allocation
  - One `Vec<PipelineSegment>` allocation

- **Execution**: `O(k)` where k = number of commands
  - One `Vec<Child>` allocation
  - OS pipe allocations (unavoidable, handled by kernel)

**Total overhead**: <5ms for typical pipelines (constitution requirement)

### Zero-Copy Optimization

```rust
// Instead of cloning strings during parsing:
pub struct PipelineSegment<'a> {
    pub program: &'a str,       // Reference to input
    pub args: Vec<&'a str>,     // References to input
    pub index: usize,
}
```

**Trade-off**: Lifetime complexity vs. performance
**Decision**: Use owned Strings for MVP (simpler, still meets <5ms requirement)
**Future**: Add zero-copy variant if profiling shows bottleneck

---

## Next Steps

1. **Contracts** (Phase 1 continued)
   - Create test files for all acceptance scenarios
   - Location: `specs/004-pipes/contracts/`

2. **Quickstart** (Phase 1 continued)
   - User-facing guide to using pipes
   - Examples, common patterns

3. **Implementation Plan** (Phase 2)
   - Step-by-step task breakdown
   - PR strategy (likely 3-4 PRs based on user stories)
