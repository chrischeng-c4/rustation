# Implementation Plan: Stderr Redirection

**Feature**: 010-stderr-redirection
**Date**: 2025-11-27

## Technical Approach

### Extend Existing Infrastructure

The current codebase already has:
- `Redirect` struct with `fd` field (supports fd 1 and 2)
- `RedirectionType` enum (Output, Append, Input)
- Pipeline execution with redirection support

Changes needed:
1. **Parser**: Recognize new operator tokens (2>, 2>>, 2>&1, &>, &>>)
2. **Executor**: Handle fd 2 redirections and dup2 for 2>&1

### Implementation Steps

1. Add new Token variants for stderr operators
2. Update tokenizer to recognize 2>, 2>>, 2>&1, &>, &>>
3. Extend Redirection/RedirectionType to handle stderr
4. Update pipeline execution to apply fd 2 redirects
5. Handle 2>&1 with dup2 semantics

## File Changes

| File | Changes |
|------|---------|
| parser.rs | Add tokens, update tokenizer |
| mod.rs | Extend RedirectionType if needed |
| pipeline.rs | Handle stderr fd in execution |

## Testing

- Unit tests for parser tokens
- Integration tests for actual redirection
- Order-dependent tests (> file 2>&1 vs 2>&1 > file)
