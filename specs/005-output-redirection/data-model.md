# Data Model: Output Redirection

**Feature**: 005-output-redirection
**Date**: 2025-11-20
**Purpose**: Define core data structures, types, and relationships for I/O redirection

## Core Types

### RedirectionType Enum

Represents the three types of I/O redirection operators.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectionType {
    /// Output redirection (>) - truncate and write to file
    Output,

    /// Append redirection (>>) - append to file
    Append,

    /// Input redirection (<) - read from file
    Input,
}
```

**Properties**:
- **Copy**: Lightweight, cheap to copy
- **Eq**: Can be compared for equality
- **Debug**: Can be printed for debugging

**Invariants**:
- Exhaustive: Covers all three redirection operators
- Distinct: Each operator has unique enum variant
- Immutable: Enum variants are fixed at compile time

**Usage**:
```rust
let redir_type = RedirectionType::Output;
match redir_type {
    RedirectionType::Output => { /* handle > */ },
    RedirectionType::Append => { /* handle >> */ },
    RedirectionType::Input => { /* handle < */ },
}
```

---

### Redirection Struct

Represents a single redirection operation with type and target file path.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Redirection {
    /// Type of redirection (>, >>, or <)
    pub redir_type: RedirectionType,

    /// File path for redirection target/source
    pub file_path: String,
}
```

**Properties**:
- **Clone**: Can be duplicated (needed for validation and execution)
- **PartialEq**: Can be compared (useful for testing)
- **Debug**: Can be printed for debugging

**Invariants**:
- `file_path` MUST NOT be empty string
- `file_path` MAY be relative or absolute path
- `file_path` MAY contain spaces (handled by quotes in command line)

**Validation Rules**:
```rust
impl Redirection {
    /// Validates that the redirection is well-formed
    pub fn validate(&self) -> Result<()> {
        if self.file_path.is_empty() {
            return Err(RushError::Parse("Empty file path for redirection".to_string()));
        }
        Ok(())
    }
}
```

**Constructor**:
```rust
impl Redirection {
    /// Creates a new redirection
    pub fn new(redir_type: RedirectionType, file_path: String) -> Self {
        Self { redir_type, file_path }
    }
}
```

**Usage**:
```rust
let redir = Redirection::new(
    RedirectionType::Output,
    "output.txt".to_string()
);
redir.validate()?;
```

---

### Token Enum (Extended)

Extended from feature 004 (pipes) to include redirection operators.

```rust
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// Regular word (command, argument, file path)
    Word(String),

    /// Pipe operator (|)
    Pipe,

    /// Output redirection (>)
    RedirectOut,

    /// Append redirection (>>)
    RedirectAppend,

    /// Input redirection (<)
    RedirectIn,
}
```

**Properties**:
- **Clone**: Tokens can be copied during parsing
- **PartialEq**: Tokens can be compared (testing)
- **Debug**: Tokens can be printed for debugging

**Invariants**:
- `Word` tokens contain the actual text (command name, argument value, or file path)
- Operator tokens (`Pipe`, `RedirectOut`, etc.) have no associated data
- Tokens preserve order as they appear in command line

**Usage**:
```rust
// Tokenizing "ls > files.txt"
let tokens = vec![
    Token::Word("ls".to_string()),
    Token::RedirectOut,
    Token::Word("files.txt".to_string()),
];
```

**Parsing Rules**:
- `>>` MUST be recognized as single `RedirectAppend` token, not two `RedirectOut` tokens
- Operators inside quotes become `Word` tokens: `echo "a > b"` → `[Word("echo"), Word("a > b")]`
- Whitespace around operators is ignored: `>file.txt` == `> file.txt` == `>  file.txt`

---

### PipelineSegment Struct (Extended)

Extended from feature 004 to include redirections.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineSegment {
    /// Program name to execute
    pub program: String,

    /// Arguments to pass to program
    pub args: Vec<String>,

    /// Index in pipeline (0-based)
    pub index: usize,

    /// Redirections to apply before execution (NEW)
    pub redirections: Vec<Redirection>,
}
```

**New Field**:
- `redirections`: List of redirections for this segment
- Order preserved (last wins for each stream)
- Can be empty (no redirections)

**Validation Rules**:
```rust
impl PipelineSegment {
    /// Validates segment including redirections
    pub fn validate(&self) -> Result<()> {
        if self.program.is_empty() {
            return Err(RushError::Parse("Empty program name".to_string()));
        }

        // Validate each redirection
        for redir in &self.redirections {
            redir.validate()?;
        }

        Ok(())
    }
}
```

**Constructor** (updated):
```rust
impl PipelineSegment {
    /// Creates a new pipeline segment with redirections
    pub fn new(program: String, args: Vec<String>, index: usize, redirections: Vec<Redirection>) -> Self {
        Self { program, args, index, redirections }
    }

    /// Helper: Create segment without redirections (backward compatible)
    pub fn without_redirections(program: String, args: Vec<String>, index: usize) -> Self {
        Self::new(program, args, index, Vec::new())
    }
}
```

**Usage**:
```rust
// "ls > files.txt"
let segment = PipelineSegment::new(
    "ls".to_string(),
    vec![],
    0,
    vec![Redirection::new(RedirectionType::Output, "files.txt".to_string())]
);

// "cat < input.txt | grep x > output.txt" (segment 0)
let segment0 = PipelineSegment::new(
    "cat".to_string(),
    vec![],
    0,
    vec![Redirection::new(RedirectionType::Input, "input.txt".to_string())]
);
```

---

## Data Relationships

### Parsing Flow

```
Command Line String
        ↓
   Tokenization
        ↓
   Token Vector [Word, RedirectOut, Word, Pipe, Word, RedirectIn, Word]
        ↓
   Segmentation (split at Pipe tokens)
        ↓
   Pipeline { segments: [Segment1, Segment2] }
        ↓
   PipelineSegment { program, args, redirections }
```

### Execution Flow

```
Pipeline
   ↓
PipelineSegment (for each)
   ↓
Command + Redirections
   ↓
Apply Redirections (open files, set stdio)
   ↓
Spawn Process with configured stdio
   ↓
Child Process with redirected I/O
```

### Redirection Resolution

When segment has multiple redirections for same stream, **last wins**:

```rust
// "echo test > a.txt > b.txt"
// Redirections: [Output("a.txt"), Output("b.txt")]
// Resolution: Only b.txt gets output (last wins)

let mut stdout_redir: Option<&Redirection> = None;

for redir in &segment.redirections {
    match redir.redir_type {
        RedirectionType::Output | RedirectionType::Append => {
            stdout_redir = Some(redir); // Overwrites previous
        }
        _ => {}
    }
}

// stdout_redir now points to last output redirection
```

---

## Error Types

### RushError Extension

Add new error variant for redirection-specific errors:

```rust
#[derive(Debug)]
pub enum RushError {
    // ... existing variants ...

    /// Redirection-specific errors
    Redirection(String),
}
```

**Usage**:
```rust
// File not found
RushError::Redirection(format!("{}: file not found", path))

// Permission denied
RushError::Redirection(format!("{}: permission denied", path))

// Is a directory
RushError::Redirection(format!("{}: is a directory", path))
```

**Error Message Format**:
- Include file path for context
- Use lowercase for consistency with bash/zsh
- Be specific about error cause

---

## Validation Rules Summary

### At Parse Time

1. **Token validation**: Ensure `>>` recognized as single token
2. **Quote validation**: Operators inside quotes become Word tokens
3. **Syntax validation**: Redirection token must be followed by Word token (file path)
4. **Empty path validation**: File path Word cannot be empty string

### Before Execution

1. **Segment validation**: Program name not empty
2. **Redirection validation**: Each redirection has non-empty file path
3. **Pipeline validation**: At least one segment exists

### At Execution Time

1. **File existence** (input only): File must exist for `<` operator
2. **Directory check**: File path must not be a directory
3. **Permission check**: User must have read/write permissions
4. **Disk space check**: Sufficient space for write operations (OS-level)

---

## State Transitions

### Redirection Lifecycle

```
1. TOKENIZED: Operator recognized as Token::RedirectOut/Append/In
              File path recognized as Token::Word

2. PARSED:    Tokens combined into Redirection struct
              Added to PipelineSegment.redirections

3. VALIDATED: Redirection.validate() checks file_path non-empty
              PipelineSegment.validate() checks all redirections valid

4. APPLIED:   File opened with appropriate mode (read/write/append)
              Stdio created from file handle
              Command configured with Stdio

5. EXECUTED:  Process spawned with redirected I/O
              File descriptors inherited by child process

6. CLOSED:    File handles automatically closed when process exits (RAII)
```

---

## Type Safety Guarantees

### Compile-Time Safety

1. **Enum exhaustiveness**: Rust ensures all RedirectionType variants handled
2. **Result propagation**: Errors must be explicitly handled via `?` or `match`
3. **Ownership**: File handles moved into Stdio, preventing double-close
4. **Lifetime**: Redirections live as long as PipelineSegment (owned data)

### Runtime Safety

1. **Validation before execution**: validate() called before spawn()
2. **Error context**: All errors include file path for debugging
3. **Resource cleanup**: RAII ensures files closed even on panic
4. **No unsafe code**: Pure safe Rust implementation

---

## Memory Layout

### Size Estimates

```rust
// RedirectionType: 1 byte (enum discriminant)
std::mem::size_of::<RedirectionType>() == 1

// Redirection: 1 byte (type) + 24 bytes (String) ≈ 32 bytes (with padding)
std::mem::size_of::<Redirection>() ≈ 32

// Vec<Redirection>: 24 bytes (ptr + len + capacity)
// Typical case: 0-2 redirections per segment
// Memory overhead: 24 + (0-2 × 32) = 24-88 bytes per segment
```

### Allocation Strategy

- Strings allocated on heap (file paths)
- Vec allocated on heap (redirections list)
- Most commands have 0 redirections (empty Vec, 24 bytes overhead)
- Maximum practical redirections per segment: ~5 (rare case)

---

## Testing Considerations

### Unit Test Cases

1. **RedirectionType**: Enum variant creation and matching
2. **Redirection**: Validation (empty path, valid path)
3. **PipelineSegment**: With redirections, without redirections
4. **Token**: Parsing `>`, `>>`, `<` in various contexts

### Integration Test Cases

1. **Single redirection**: `echo test > file.txt`
2. **Multiple redirections**: `echo test > a.txt > b.txt` (last wins)
3. **Combined I/O**: `sort < input.txt > output.txt`
4. **With pipes**: `cat < file.txt | grep x > results.txt`
5. **Quote handling**: `echo "a > b"` outputs literal "a > b"

### Property Tests

1. **Idempotency**: Parsing same command yields same redirections
2. **Order preservation**: Redirections maintain left-to-right order
3. **Last wins**: Multiple output redirections use last one only
4. **Quote isolation**: Operators in quotes never parsed as redirections

---

**Data Model Complete**: All types, relationships, and validation rules defined. Ready for implementation.
