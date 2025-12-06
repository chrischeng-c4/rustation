# Data Model: Conditional Control Flow

**Feature**: 017-conditional-control-flow
**Date**: 2025-12-06
**Based On**: [research.md](./research.md)

## Overview

This document defines the AST structures and data types for implementing if/then/else/elif/fi conditional control flow in the rush shell.

---

## Core Entities

### 1. IfBlock

Represents a complete `if/then/[elif/then]*/[else]/fi` construct.

```rust
/// A complete if/elif/else/fi conditional block
///
/// Grammar: If compound_list Then compound_list [else_part] Fi
#[derive(Debug, Clone, PartialEq)]
pub struct IfBlock {
    /// The condition command(s) - evaluated for exit code
    /// Exit code 0 = true, non-zero = false
    pub condition: CompoundList,

    /// Commands to execute when condition succeeds (exit 0)
    pub then_block: CompoundList,

    /// Zero or more elif clauses, evaluated in order
    pub elif_clauses: Vec<ElifClause>,

    /// Optional else block, executed when all conditions fail
    pub else_block: Option<CompoundList>,

    /// Source location for error reporting
    pub span: Span,
}
```

**Relationships**:
- Contains 1 `CompoundList` for condition
- Contains 1 `CompoundList` for then block
- Contains 0..N `ElifClause` for elif branches
- Contains 0..1 `CompoundList` for else block

**Validation Rules**:
- condition MUST NOT be empty
- then_block MAY be empty (no-op)
- else_block MAY be empty if present

---

### 2. ElifClause

Represents an `elif condition then block` within an if construct.

```rust
/// An elif clause within an if block
///
/// Grammar: Elif compound_list Then compound_list
#[derive(Debug, Clone, PartialEq)]
pub struct ElifClause {
    /// The elif condition command(s)
    pub condition: CompoundList,

    /// Commands to execute when this elif condition succeeds
    pub then_block: CompoundList,

    /// Source location for error reporting
    pub span: Span,
}
```

**Relationships**:
- Owned by `IfBlock.elif_clauses`
- Contains 2 `CompoundList` instances

**Validation Rules**:
- condition MUST NOT be empty
- then_block MAY be empty

---

### 3. CompoundList

A sequence of commands that can include simple commands, pipelines, or nested compound commands.

```rust
/// A sequence of commands separated by ; or newlines
///
/// Grammar: linebreak term [separator]
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundList {
    /// Commands in execution order
    pub commands: Vec<Command>,

    /// Source location spanning all commands
    pub span: Span,
}
```

**Relationships**:
- Contains 0..N `Command` instances
- Used by `IfBlock`, `ElifClause` for condition and body blocks

**Validation Rules**:
- MAY be empty (represents no-op)
- Commands execute sequentially

---

### 4. Command (Extended Enum)

Union type representing all executable command forms.

```rust
/// A single executable command unit
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Simple command or pipeline (existing functionality)
    Simple(Pipeline),

    /// Conditional block (this feature)
    If(Box<IfBlock>),

    /// Future: loop constructs
    // While(Box<WhileBlock>),
    // For(Box<ForBlock>),
    // Case(Box<CaseBlock>),
}
```

**Note**: `Box<IfBlock>` required because `Command` is recursive (if blocks can contain Commands).

---

### 5. Keyword (Token Type)

Reserved words recognized during tokenization.

```rust
/// Reserved keywords for control flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    If,
    Then,
    Elif,
    Else,
    Fi,
    // Future:
    // While, Do, Done,
    // For, In,
    // Case, Esac,
}

impl Keyword {
    pub fn as_str(&self) -> &'static str {
        match self {
            Keyword::If => "if",
            Keyword::Then => "then",
            Keyword::Elif => "elif",
            Keyword::Else => "else",
            Keyword::Fi => "fi",
        }
    }

    pub fn from_str(s: &str) -> Option<Keyword> {
        match s {
            "if" => Some(Keyword::If),
            "then" => Some(Keyword::Then),
            "elif" => Some(Keyword::Elif),
            "else" => Some(Keyword::Else),
            "fi" => Some(Keyword::Fi),
            _ => None,
        }
    }
}
```

---

### 6. Token (Extended)

Extended token type for lexer output.

```rust
/// Token types produced by the lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Existing tokens
    Word(String),
    Pipe,
    RedirectOut,
    RedirectAppend,
    RedirectIn,
    StderrOut,
    StderrAppend,
    Heredoc,
    HeredocStrip,
    Background,
    Semicolon,

    // New: Keywords
    Keyword(Keyword),

    // New: Operators (for compound lists)
    And,        // &&
    Or,         // ||

    // New: Delimiters
    Newline,
    Eof,
}
```

---

### 7. Span (Source Location)

For error reporting with location information.

```rust
/// Source location for error messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Starting byte offset in input
    pub start: usize,
    /// Ending byte offset in input (exclusive)
    pub end: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}
```

---

### 8. SyntaxError (New Error Type)

Specific error type for parser errors.

```rust
/// Syntax errors during parsing
#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxError {
    /// Found unexpected token
    UnexpectedToken {
        found: Token,
        expected: &'static str,
        span: Span,
    },

    /// Missing required keyword
    MissingKeyword {
        expected: Keyword,
        after: &'static str,
        span: Span,
    },

    /// Keyword without matching structure
    UnmatchedKeyword {
        keyword: Keyword,
        span: Span,
    },

    /// Unexpected end of input
    UnexpectedEof {
        expected: &'static str,
        span: Span,
    },
}
```

---

## Entity Relationships

```text
┌─────────────────────────────────────────────────────────────┐
│                         Command                              │
│  ┌─────────────┐    ┌─────────────────────────────────────┐ │
│  │   Simple    │    │              If                     │ │
│  │  (Pipeline) │    │   ┌─────────────────────────────┐  │ │
│  └─────────────┘    │   │         IfBlock             │  │ │
│                      │   │  ┌───────────────────────┐  │  │ │
│                      │   │  │ condition:            │  │  │ │
│                      │   │  │   CompoundList ───────┼──┼──┼─┤
│                      │   │  │     commands: Vec<Command>    │
│                      │   │  ├───────────────────────┤  │  │ │
│                      │   │  │ then_block:           │  │  │ │
│                      │   │  │   CompoundList        │  │  │ │
│                      │   │  ├───────────────────────┤  │  │ │
│                      │   │  │ elif_clauses:         │  │  │ │
│                      │   │  │   Vec<ElifClause>     │  │  │ │
│                      │   │  │   ┌─────────────────┐ │  │  │ │
│                      │   │  │   │ condition       │ │  │  │ │
│                      │   │  │   │ then_block      │ │  │  │ │
│                      │   │  │   └─────────────────┘ │  │  │ │
│                      │   │  ├───────────────────────┤  │  │ │
│                      │   │  │ else_block:           │  │  │ │
│                      │   │  │   Option<CompoundList>│  │  │ │
│                      │   │  └───────────────────────┘  │  │ │
│                      │   └─────────────────────────────┘  │ │
│                      └─────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

Note: CompoundList.commands can contain Command::If, enabling nesting.
      Box<IfBlock> breaks the infinite size cycle.
```

---

## State Transitions

### Conditional Execution Flow

```text
┌─────────────┐
│ Parse Input │
└──────┬──────┘
       ▼
┌─────────────────┐
│ Evaluate        │
│ if condition    │
└──────┬──────────┘
       │
       ▼
   exit == 0?
  /          \
 yes          no
  │            │
  ▼            ▼
┌────────┐  ┌─────────────────┐
│Execute │  │ Any elif left?  │
│ then   │  └──────┬──────────┘
│ block  │         │
└───┬────┘    yes /  \ no
    │            /    \
    │           ▼      ▼
    │    ┌──────────┐  ┌─────────────┐
    │    │ Evaluate │  │ else block? │
    │    │ elif     │  └──────┬──────┘
    │    │ condition│      yes│  │no
    │    └────┬─────┘         │  │
    │    exit == 0?           ▼  ▼
    │   /          \    ┌────────┐  ┌────────────┐
    │  yes          no  │Execute │  │ Return 0   │
    │   │            │  │ else   │  │ (no branch)│
    │   ▼            │  │ block  │  └────────────┘
    │ ┌────────┐     │  └───┬────┘
    │ │Execute │     │      │
    │ │ elif   │     │      │
    │ │ then   │◄────┘      │
    │ └───┬────┘            │
    │     │                 │
    └─────┴─────────────────┘
          │
          ▼
    ┌──────────────┐
    │ Return exit  │
    │ code of last │
    │ command      │
    └──────────────┘
```

---

## Validation Summary

| Entity | Field | Constraint |
|--------|-------|------------|
| IfBlock | condition | MUST NOT be empty |
| IfBlock | then_block | MAY be empty |
| IfBlock | elif_clauses | MAY be empty |
| IfBlock | else_block | MAY be None or empty |
| ElifClause | condition | MUST NOT be empty |
| ElifClause | then_block | MAY be empty |
| CompoundList | commands | MAY be empty |
| Keyword | - | Must be in command position |
| Span | line, column | 1-indexed |

---

## Memory Layout Estimates

| Structure | Size (bytes) | Notes |
|-----------|--------------|-------|
| Keyword | 1 | Single byte enum discriminant |
| Token | 32 | Enum with String variant |
| Span | 32 | 4 × usize |
| ElifClause | ~100 | 2 CompoundLists + Span |
| IfBlock | ~200 | 4 fields + heap for Vecs |
| CompoundList | ~48 | Vec + Span |
| Command | ~16 | Enum with Box pointer |

Typical if/then/fi uses ~300 bytes. Deeply nested (10 levels) uses ~3KB. Well under 10MB budget.
