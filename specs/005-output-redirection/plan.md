# Implementation Plan: Output Redirection Operators

**Branch**: `005-output-redirection` | **Date**: 2025-11-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-output-redirection/spec.md`

## Summary

Implement Unix-style I/O redirection operators (`>`, `>>`, `<`) for rush shell to enable file-based input/output redirection. This extends the existing parser and executor to support redirecting command stdout to files (overwrite or append) and stdin from files, enabling essential automation and data processing workflows. The implementation builds on the existing pipe operator (feature 004) infrastructure and integrates with the quote-aware parser.

**Primary Requirements**:
- Parse three redirection operators: `>` (output overwrite), `>>` (output append), `<` (input)
- Create/open files with appropriate modes (create, truncate, append, read)
- Set up file descriptors before process execution using Rust's `std::process::Stdio`
- Preserve operators as literals when inside quotes
- Provide clear error messages for file system errors (permissions, not found, is directory)
- Maintain <1ms redirection setup overhead per constitution requirement

**Technical Approach**:
- Extend tokenizer to recognize `>`, `>>`, `<` as special tokens (similar to `|` pipe handling)
- Create `Redirection` data structure to represent type, file path, and target descriptor
- Modify `PipelineExecutor` to apply redirections before spawning processes
- Use `std::fs::File` and `std::process::Stdio::from()` for file descriptor setup
- Integrate with existing quote parser to handle operators inside strings

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**:
- `std::process` (Command, Stdio) - process spawning with custom file descriptors
- `std::fs` (File, OpenOptions) - file creation and opening
- `std::os::unix::io` (AsRawFd, FromRawFd) - file descriptor manipulation
**Storage**: File system operations (create, open, truncate, append files)
**Testing**: cargo test (unit, integration, contract tests) + cargo bench (performance)
**Target Platform**: macOS (MVP), Linux (future)
**Project Type**: Single project (rush shell binary)
**Performance Goals**: <1ms redirection setup overhead, <5ms total command execution overhead
**Constraints**:
- Must maintain backward compatibility (all existing 286 tests pass)
- File operations must not block the shell (setup before fork/exec)
- Error messages must be specific (distinguish file not found, permission denied, is directory)
**Scale/Scope**:
- 3 operators (`>`, `>>`, `<`)
- 5 user stories (2 P1, 2 P2, 1 P3)
- ~60-80 tests (parser, execution, errors, integration)
- Estimated ~1,500-2,000 lines total across implementation and tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Performance-First ✅ PASS

**Requirement**: Command execution overhead MUST be <5ms
**This Feature**: Redirection setup targets <1ms overhead
- File opening happens after fork, before exec (doesn't block parent)
- File descriptors set up once per command using `Stdio::from()`
- No buffering or copying - direct kernel-level file descriptor redirection
- Performance benchmarks required (FR per constitution)

**Validation**: Will benchmark redirection setup time and compare redirected vs non-redirected command execution.

### II. Zero-Config Philosophy ✅ PASS

**Requirement**: Features work with zero configuration
**This Feature**: Redirection works immediately, no configuration needed
- Standard POSIX semantics (matches bash/zsh behavior)
- No config files, environment variables, or setup required
- Default file permissions (0644) follow Unix standards
- User types `echo "hello" > file.txt` and it just works

**Validation**: First-time users can use redirections without reading documentation.

### III. Progressive Complexity ✅ PASS

**Requirement**: Simple by default, powerful when needed
**This Feature**: Basic redirections are simple, advanced combinations optional
- P1: Simple `>` and `>>` for most common use cases
- P2: Input `<` and combinations for advanced workflows
- P3: Error handling exposes complexity only when errors occur
- Users who never use redirections pay zero cost (feature detection in parser)

**Validation**: Basic workflows (`ls > files.txt`) are trivial; advanced workflows (`sort < in.txt > out.txt | grep x`) are possible but optional.

### IV. Modern UX ✅ PASS

**Requirement**: Contemporary, delightful user experience
**This Feature**: Clear error messages and predictable behavior
- Specific error messages: "permission denied", "file not found", "is a directory"
- Errors caught early (file creation before command execution)
- Quote-aware parsing preserves user intent (`echo "a > b"` outputs "a > b")
- Consistent with existing shell UX (syntax highlighting will show operators)

**Validation**: Error messages tested for clarity; quote handling preserves literals.

### V. Rust-Native ✅ PASS

**Requirement**: Pure Rust, ecosystem integration
**This Feature**: Uses only Rust standard library
- `std::process::Stdio` for file descriptor management
- `std::fs::File` and `OpenOptions` for file operations
- `std::os::unix::io` for platform-specific FD handling (Unix-only for MVP)
- No external crates needed (all stdlib)
- Idiomatic Rust: Result types, From/Into traits, owned strings

**Validation**: Zero external dependencies; pure Rust implementation.

**Overall Constitution Check**: ✅ **ALL GATES PASS** - No violations, no complexity justification needed.

## Project Structure

### Documentation (this feature)

```text
specs/005-output-redirection/
├── spec.md                    # Feature specification (complete)
├── plan.md                    # This file (in progress)
├── research.md                # Phase 0: Technical research and decisions
├── data-model.md              # Phase 1: Data structures and types
├── quickstart.md              # Phase 1: Usage examples and quick reference
├── contracts/                 # Phase 1: API/behavior contracts
│   └── redirection-api.md    # Redirection behavior contract
├── checklists/
│   └── requirements.md       # Specification quality checklist (complete)
└── tasks.md                   # Phase 2: Implementation tasks (not created yet)
```

### Source Code (repository root)

```text
crates/rush/src/
├── executor/
│   ├── mod.rs                # Pipeline, PipelineSegment, Redirection structs (extend)
│   ├── parser.rs             # Token enum (add RedirectOut/RedirectAppend/RedirectIn)
│   ├── pipeline.rs           # PipelineExecutor (add redirection setup)
│   └── execute.rs            # CommandExecutor (unchanged, uses pipeline)
└── error.rs                  # RushError (add redirection-specific errors)

crates/rush/tests/
├── unit/
│   ├── redirection_parser_tests.rs    # Parser tests for >, >>, < tokenization
│   └── redirection_model_tests.rs     # Redirection struct validation tests
├── integration/
│   ├── redirection_tests.rs           # End-to-end redirection execution tests
│   └── combined_tests.rs              # Redirections with pipes
└── contract/
    └── redirection_spec_validation.rs  # Validate against spec requirements

crates/rush/benches/
└── redirection_bench.rs               # Performance benchmarks
```

**Structure Decision**: Single project structure (Option 1). rush is a monolithic shell binary with modular internal organization. The executor module already exists from feature 004 (pipes) and will be extended to handle redirections. No new top-level modules needed - this is an enhancement to existing executor infrastructure.

## Complexity Tracking

> **No violations - this section intentionally left empty.**

All constitution checks pass without requiring justification.

## Deployment Strategy

**How this feature will be delivered incrementally via pull requests.**

### Pull Request Plan

**Strategy**: PR per User Story (Option 2) - RECOMMENDED for multi-story features

Given the feature scope:
- 5 user stories (2 P1, 2 P2, 1 P3)
- Estimated ~1,500-2,000 lines total
- User stories are independently testable
- Each story builds on foundation

**Breakdown**:

```
PR #1: Foundation + Setup (Infrastructure)
  - Redirection data structures (Redirection enum/struct)
  - Token enum extensions (RedirectOut, RedirectAppend, RedirectIn)
  - Basic parser infrastructure (no execution yet)
  - Unit tests for data structures
  - Target: ~400-500 lines

PR #2: User Story 1 - Basic Output Redirection (>) [P1]
  - Implement > operator parsing
  - PipelineExecutor integration for output redirection
  - File creation with truncate mode
  - Integration tests for overwrite behavior
  - Error handling (permissions, is directory)
  - Target: ~600-800 lines

PR #3: User Story 2 - Append Output Redirection (>>) [P1]
  - Implement >> operator parsing
  - Append mode file opening
  - Integration tests for append behavior
  - Verify no data loss scenarios
  - Target: ~300-400 lines (builds on PR #2)

PR #4: User Story 3 + 4 - Input and Combined Redirections [P2]
  - Implement < operator parsing
  - Stdin redirection setup
  - Combined redirection support (< input.txt command > output.txt)
  - Integration with pipes (cat < file | grep x > out)
  - Multiple redirections (last wins)
  - Integration tests for input and combinations
  - Target: ~700-900 lines

PR #5: User Story 5 + Polish - Error Handling and Edge Cases [P3]
  - Comprehensive error messages
  - Edge case handling (same file read/write, missing directories, etc.)
  - Contract tests validating spec requirements
  - Performance benchmarks
  - Documentation updates (CLI.md, README.md)
  - Target: ~400-500 lines
```

### Selected Strategy

**Option 2: PR per User Story** - Best fit for this feature

**Rationale**:
- 5 user stories with clear boundaries
- Each PR is independently mergeable (400-900 lines, well under 1,500 limit)
- Foundation PR enables subsequent PRs
- P1 stories (output redirection) deliver core value early
- P2 and P3 stories add progressively more advanced functionality
- Total ~2,400-3,100 lines split across 5 PRs = all under limits

### Merge Sequence

1. **PR #1: Foundation** → Merge to main
   - Data structures and parser infrastructure ready
   - No user-visible functionality yet but enables all other PRs

2. **PR #2: US1 - Output >** → Merge to main
   - First user-visible feature: `ls > files.txt` works
   - Core value delivered early

3. **PR #3: US2 - Append >>** → Merge to main
   - Second essential operator: `echo "line" >> log.txt` works
   - Logging and accumulation workflows enabled

4. **PR #4: US3+4 - Input < and Combined** → Merge to main
   - Advanced workflows: `sort < in.txt > out.txt`
   - Combinations with pipes work

5. **PR #5: US5 + Polish** → Merge to main
   - Error handling robust
   - Performance validated
   - Documentation complete

**Branch Strategy**:
- Base branch: `005-output-redirection` (current)
- All PRs merge directly to `main` sequentially
- Each PR builds on previous merged work
- No sub-branches needed (linear progression)

### PR Size Validation

**Before creating each PR, verify size**:
```bash
git diff --stat main  # Check line count
```

**Size Limits** (from CLAUDE.md):
- ✅ Ideal: ≤ 500 lines
- ⚠️ Maximum: ≤ 1,500 lines
- ❌ Too large: > 3,000 lines (must split)

**Estimated Sizes**:
- PR #1: ~450 lines ✅ (ideal)
- PR #2: ~700 lines ✅ (acceptable)
- PR #3: ~350 lines ✅ (ideal)
- PR #4: ~800 lines ✅ (acceptable)
- PR #5: ~450 lines ✅ (ideal)

All PRs comfortably under 1,500-line limit. If any PR exceeds during implementation, split further by component (e.g., separate tests into follow-up PR).

## Phase 0: Research & Decisions

*Generated during planning phase (`/speckit.plan` command)*

See [research.md](./research.md) for detailed technical research and decisions.

**Key Decisions**:
1. **File Descriptor Strategy**: Use `std::process::Stdio::from(File)` to convert opened files to Stdio handles
2. **Error Handling**: Early validation (open files before spawning) for fast failure
3. **Quote Integration**: Extend existing quote parser to recognize `>`, `>>`, `<` as special characters
4. **Performance**: File operations in child process (after fork) to avoid blocking parent shell

## Phase 1: Design Artifacts

*Generated during planning phase (`/speckit.plan` command)*

- [data-model.md](./data-model.md) - Core data structures (Redirection, RedirectionType, extended Token enum)
- [contracts/redirection-api.md](./contracts/redirection-api.md) - Behavior contracts and invariants
- [quickstart.md](./quickstart.md) - Usage examples and quick reference

## Phase 2: Task Breakdown

*Generated by `/speckit.tasks` command (not yet created)*

Task generation deferred to `/speckit.tasks` command after planning complete.

---

**Planning Status**: Phase 0 (Research) and Phase 1 (Design) in progress below.
