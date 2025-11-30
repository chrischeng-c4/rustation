# Technical Plan: Command Substitution - $(…) (Feature 010)

**Branch**: `010-command-substitution` | **Date**: 2025-11-30
**Status**: Ready for Implementation

## Summary

Implement `$()` command substitution syntax enabling users to capture command output and use it as input to other commands. Examples: `echo $(date)`, `files=$(ls *.txt)`, `cmd $(nested $(inner))`.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: rush executor, parser, lexer (existing)
**Testing**: cargo test, integration tests
**Performance Goals**: <1ms per substitution, handle large output (>1MB)
**Constraints**: POSIX-compatible semantics, proper error propagation

## Architecture

### High-Level Flow

```
Input: echo $(date)
   ↓
Lexer: tokenize, identify $(...) regions
   ↓
Parser: extract inner command string
   ↓
CommandExecutor: execute inner command, capture stdout
   ↓
OutputCapture: collect stderr-free output, trim newlines
   ↓
WordSplit: split output into tokens by whitespace
   ↓
SubstitutionExpansion: replace $(...) with tokens
   ↓
ContinueExecution: use expanded tokens in command
```

### Components

- **SubstitutionLexer**: Identifies $(...) regions in input, respecting quotes/escapes
- **SubstitutionParser**: Extracts nested command string, handles nesting
- **SubstitutionExecutor**: Spawns subprocess, captures stdout, handles errors
- **SubstitutionExpander**: Replaces $(...) with captured output, handles word splitting
- **NestedExpander**: Recursively expands nested substitutions (innermost first)

## Critical Design Decisions

### Decision 1: Expansion Timing

**Decision**: After tokenization, before command parsing
**Rationale**: Token stream is mutable, clear separation of concerns
**Trade-offs**: One-pass expansion only

### Decision 2: Output Handling

**Decision**: Word split using POSIX semantics (spaces/tabs/newlines)
**Rationale**: Matches bash behavior, enables multiple args from single substitution
**Trade-offs**: Users must quote if single arg needed

### Decision 3: Error Handling

**Decision**: Abort on any substitution failure—don't execute outer command
**Rationale**: Constitution "Correctness", prevents silent failures
**Trade-offs**: No way to ignore errors

### Decision 4: Nested Substitution

**Decision**: Full recursion support, innermost-first evaluation
**Rationale**: POSIX feature, relatively simple recursive descent
**Trade-offs**: Stack depth limits (practically unlimited)

### Decision 5: Substitution Contexts

**Decision**: Support everywhere—args, variables, redirections
**Rationale**: Full feature completeness, matches bash
**Trade-offs**: Complex implementation across multiple code paths

### Decision 6: Stderr Handling

**Decision**: Capture stdout only, stderr to terminal (unless redirected in inner cmd)
**Rationale**: Standard shell behavior, separates output from diagnostics
**Trade-offs**: Can't capture stderr directly into substitution

### Decision 7: Output Size Limits

**Decision**: 10MB limit (configurable)
**Rationale**: Prevents memory exhaustion, covers 99.9% of use cases
**Trade-offs**: Large file processing may fail

## Implementation Phases

### Phase 1: Lexer & Identification (2-3 days)

Identify and extract $(...) regions from input

**Files**:
- `crates/rush/src/executor/substitution/mod.rs` (new)
- `crates/rush/src/executor/substitution/lexer.rs` (new)

**Tasks**: Create Substitution types, implement pattern scanner, handle quotes, identify nesting

### Phase 2: Command Execution (2-3 days)

Execute inner commands and capture output

**Files**:
- `crates/rush/src/executor/substitution/executor.rs` (new)

**Tasks**: Subprocess spawning, stdout capture, error handling, output size limits

### Phase 3: Expansion & Integration (3-4 days)

Integrate captured output back into command

**Files**:
- `crates/rush/src/executor/substitution/expander.rs` (new)
- `crates/rush/src/executor/parser.rs` (modify)
- `crates/rush/src/executor/mod.rs` (integrate)

**Tasks**: Word splitting, token replacement, nested expansion, parser integration

### Phase 4: Advanced Features (2-3 days)

Support all user stories—variables, redirections, errors

### Phase 5: Polish (1-2 days)

Testing, validation, documentation

## Testing Strategy

- **Unit Tests**: Lexer patterns, executor subprocess, expander word-split
- **Integration Tests**: End-to-end execution, error cases, nesting
- **Coverage Goal**: >85%

## Acceptance Criteria

- US1: `echo $(date)` works
- US2: `echo $(echo $(date))` nests correctly
- US3: `var=$(cmd)` assigns output
- US4: `ls $(find . -name "*.txt")` passes multiple args
- US5: Failed substitution prevents outer execution

---

**Estimated Duration**: 10-14 hours
**Complexity**: High
**Status**: Ready for Phase 1
