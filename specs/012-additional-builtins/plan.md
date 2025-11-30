# Technical Plan: Additional Builtins (Feature 012)

**Branch**: `012-additional-builtins` | **Status**: Ready for Implementation

## Summary

Implement standard shell builtins: `echo`, `printf`, `test`, `true`, `false`, `[`, `pwd`, `type`

## Architecture

**Components**:
- **Echo**: Print args with spaces, `-n` flag
- **Printf**: Formatted output (basic subset)
- **Test**: Conditional testing `test -f file`, `[ -f file ]`
- **True/False**: Exit codes 0/1
- **Pwd**: Print working directory
- **Type**: Show command type (builtin/function/command)

**Integration Points**:
- `executor/builtins/mod.rs`: Register all builtins
- `executor/builtins/echo.rs`: Echo implementation
- `executor/builtins/printf.rs`: Printf implementation
- `executor/builtins/test.rs`: Test operator implementation

## Critical Decisions

1. **Test operators**: Support subset (file tests: -f, -d, -e; string tests: -z, -n, =)
2. **Echo behavior**: Default with newline, `-n` to suppress
3. **Printf**: Basic format strings (%s, %d, %c), no floating point initially
4. **Exit codes**: Propagate correctly from all builtins

## Implementation Phases

**Phase 1** (1-2 days): Echo + true/false
**Phase 2** (2-3 days): Test builtin
**Phase 3** (1-2 days): Printf
**Phase 4** (1 day): pwd, type

## User Stories

- US1: `echo` with newline handling
- US2: `test` for file/string conditions
- US3: `printf` formatted output
- US4: `true`/`false` for scripting
- US5: `pwd`/`type` utility commands

---

**Estimated Duration**: 5-8 hours
**Complexity**: Low-Medium
