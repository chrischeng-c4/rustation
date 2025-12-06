# Rush Shell Specifications (001-026 Planned)

This directory contains specifications and implementation plans for rush shell features, organized by feature number.

**Current Focus**: Control Flow Foundation (Features 017-026)

## Feature Directory Structure

Each feature is organized in its own directory with:
- `spec.md` - Requirement specification with user stories and acceptance criteria
- `plan.md` - Implementation plan with architecture and technical approach
- `tasks.md` - (When applicable) Breakdown of implementation tasks
- `checklists/` - (Optional) Quality assurance and testing checklists
- `contracts/` - (Optional) API contracts and interfaces

## Feature Status Overview

### ✅ Complete (Specification + Implementation)

| # | Feature | Status | Implementation | Tests |
|---|---------|--------|-----------------|-------|
| 001 | Rush MVP | ✅ Complete | ✅ Merged | ✅ 107+ |
| 002 | Tab Completion | ✅ Complete | ✅ Merged | ✅ 20+ |
| 003 | Autosuggestions | ✅ Complete | ✅ Merged | ✅ 15+ |
| 004 | Pipes | ✅ Complete | ✅ Merged | ✅ 10+ |
| 005 | Output Redirection (>, >>, <) | ✅ Complete | ✅ Merged | ✅ 10+ |
| 006 | Job Control | ✅ Complete | ✅ Merged | ✅ 26+ |
| 009 | Globbing & Wildcards | ✅ Complete | ✅ Merged | ✅ 16+ |
| 013 | CD Builtin | ✅ Complete | ✅ Merged | ✅ 8+ |
| 014 | Environment Variables | ✅ Complete | ✅ Merged | ✅ 20+ |
| 017 | Conditionals (if/then/elif/else/fi) | ✅ Complete | ✅ Ready for PR | ✅ 22 (11 unit + 11 integration) |

### 📋 Ready for Implementation (Specification + Plan + Tasks Complete)

| # | Feature | Spec | Plan | Tasks | Tests | GitHub |
|---|---------|------|------|-------|-------|--------|
| 018 | for/in/do/done loops | ✅ | ✅ | ✅ (58) | 45+ | #33 |
| 019 | while/until loops | ✅ | ✅ | ✅ (48) | 35+ | #37 |

### 📋 Specification Complete (All Artifacts Ready)

| # | Feature | Spec | Plan | Tasks | Tests | GitHub |
|---|---------|------|------|-------|-------|--------|
| 020 | case/esac patterns | ✅ | ✅ | ✅ (42) | 30+ | #38 |
| 021 | shell functions | ✅ | ✅ | ✅ (40) | 25+ | #39 |
| 022 | break statement | ✅ | ✅ | ✅ (20) | 15+ | #40 |
| 023 | continue statement | ✅ | ✅ | ✅ (20) | 15+ | #41 |
| 024 | return statement | ✅ | ✅ | ✅ (15) | 10+ | #42 |
| 025 | subshells | ✅ | ✅ | ✅ (30) | 20+ | #43 |
| 026 | command groups | ✅ | ✅ | ✅ (25) | 15+ | #44 |

### 🔮 Reserved for Future Features

| # | Status | Note |
|---|--------|------|
| 007 | Reserved | Future feature TBD |
| 008 | Reserved | Future feature TBD |
| 010 | Reserved | Future feature TBD |
| 011 | Reserved | Future feature TBD |
| 012 | Reserved | Future feature TBD |

---

## Features by Category

### Core Shell Functionality
- **001** - Rush MVP (REPL, basic command execution)
- **004** - Pipes (command chaining)
- **005** - Output Redirection (file I/O)
- **006** - Job Control (background jobs, Ctrl+Z)
- **013** - CD Builtin (change directory)
- **014** - Environment Variables (variable management)

### User Experience
- **002** - Tab Completion (intelligent command completion)
- **003** - Autosuggestions (history-based suggestions)
- **009** - Globbing (wildcard expansion)

### Reserved/Future
- **007-008** - Reserved for future features
- **010-012** - Reserved for future features

---

## Feature Details

### 001: Rush MVP ✅ COMPLETE
**User Stories**: 6 (Core REPL, command execution, exit codes, signals, logging, configuration)
**Key Features**:
- Interactive REPL with line editing
- Command execution with argument passing
- Syntax highlighting
- Persistent history with navigation
- Signal handling (Ctrl+C, Ctrl+D)
- CLI with verbose logging

**Files**: `001-rush-mvp/`

---

### 002: Tab Completion ✅ COMPLETE
**User Stories**: 3 (Command completion, path completion, flag completion)
**Key Features**:
- Command name completion from PATH
- File/directory path completion
- Flag completion for common commands
- Works with partial input

**Files**: `002-tab-completion/`

---

### 003: Autosuggestions ✅ COMPLETE
**User Stories**: 3 (Inline suggestions, acceptance with arrow keys, word-by-word acceptance)
**Key Features**:
- Fish-like inline suggestions from history
- Right arrow to accept full suggestion
- Alt+Right to accept word-by-word
- Most recent matches prioritized

**Files**: `003-autosuggestions/`

---

### 004: Pipes ✅ COMPLETE (RETROSPECTIVE SPECS)
**User Stories**: 4 (Basic 2-cmd, multi-cmd, error handling, exit codes)
**Key Features**:
- Pipe operator (`|`) for command chaining
- Support for arbitrary length pipelines
- Concurrent execution
- Exit code from last command
- Proper signal handling (no zombies)

**Status**: Fully implemented, specs created for documentation
**Files**: `004-pipes/`

---

### 005: Output Redirection ✅ COMPLETE
**User Stories**: 3 (Output redirect >, append >>, input redirect <)
**Key Features**:
- Redirect stdout to file: `command > file`
- Append to file: `command >> file`
- Redirect stdin: `command < file`
- Works with pipes

**Files**: `005-output-redirection/`

---

### 006: Job Control ✅ COMPLETE
**User Stories**: 6 (Background &, jobs listing, fg/bg commands, Ctrl+Z suspension, status updates)
**Implemented Features**:
- Background execution with `&` - Run commands in background
- `jobs` command - List running/stopped jobs with status
- `fg` command - Resume stopped job in foreground
- `bg` command - Resume stopped job in background
- Ctrl+Z (SIGTSTP) suspension - Stop foreground process, convert to background job
- Automatic job cleanup - Clean up finished jobs
- Process group management - Proper signal delivery to all processes
- Enhanced integration tests - 26+ tests covering all job control workflows

**Status**: Complete implementation with comprehensive test coverage
**Files**: `006-job-control/`
**Completion**: Phase 3 completed (Commit 232255d)

---

### 009: Globbing ✅ COMPLETE
**User Stories**: 5 (Wildcard *, single char ?, character sets, negation, escaping)
**Implemented Features**:
- `*` matches zero or more characters (excluding path separator /)
- `?` matches single character (excluding /)
- `[abc]` character sets
- `[a-z]` ranges
- `[!abc]` negation
- Escape sequences (backslash escaping of metacharacters)
- Quote handling (single/double quotes prevent expansion)
- Directory traversal and file matching
- Non-matching pattern preservation

**Status**: Complete implementation with comprehensive test coverage
**Files**: `009-globbing/` (spec.md, plan.md, tasks.md)
**Test Coverage**: 11 unit tests + 5 integration tests = 16 tests total
**Completion**: Implemented with Phase 1 (core matching), Phase 2 (integration), Phase 3 (testing)

---

### 013: CD Builtin ✅ COMPLETE
**User Stories**: 4 (Basic cd, no args, tilde expansion, cd -)
**Key Features**:
- Change directory: `cd <path>`
- Home directory: `cd` with no args
- Tilde expansion: `cd ~` or `cd ~/path`
- Previous directory: `cd -`
- Proper PWD/OLDPWD tracking

**Files**: `013-cd-builtin/`

---

### 014: Environment Variables ✅ COMPLETE
**User Stories**: 4 (Set variables, export, unset, variable expansion)
**Key Features**:
- Variable assignment: `set NAME=value`
- Export variables: `export NAME=value`
- Unset variables: `unset NAME`
- Variable expansion: `$VAR`, `${VAR}`, `$$`, `$?`, `$0`
- Quote handling and escaping

**Files**: `014-environment-variables/`

---

## Reserved Features (007, 008, 010-012)

These feature numbers are reserved for future development. Possible candidates:
- **007**: Stderr redirection (2>, 2>>)
- **008**: Aliases
- **010**: Command substitution (`$()`)
- **011**: Array variables or advanced features
- **012**: Additional builtins or features

These will be formally defined and specified when development begins.

---

## Quick Links

### For Implementation
- **Rush MVP**: [`001-rush-mvp/spec.md`](001-rush-mvp/spec.md)
- **Tab Completion**: [`002-tab-completion/plan.md`](002-tab-completion/plan.md)
- **Autosuggestions**: [`003-autosuggestions/plan.md`](003-autosuggestions/plan.md)
- **Pipes (Reference)**: [`004-pipes/plan.md`](004-pipes/plan.md)
- **Output Redirection**: [`005-output-redirection/plan.md`](005-output-redirection/plan.md)
- **Job Control (To Complete)**: [`006-job-control/plan.md`](006-job-control/plan.md)
- **Globbing (To Implement)**: [`009-globbing/plan.md`](009-globbing/plan.md)
- **CD Builtin**: [`013-cd-builtin/plan.md`](013-cd-builtin/plan.md)
- **Environment Variables**: [`014-environment-variables/plan.md`](014-environment-variables/plan.md)

### For Specifications
- All features: See individual `spec.md` files in each feature directory
- Format: User stories with acceptance criteria, technical requirements, success metrics

---

## Development Workflow

The rush project uses **specification-driven development (SDD)**:

1. **Specification** (`spec.md`) - Define WHAT needs to be built
2. **Planning** (`plan.md`) - Design HOW to build it
3. **Tasks** (`tasks.md`) - Break into actionable steps
4. **Implementation** - Write code following the plan
5. **Testing** - Validate against acceptance criteria

Each feature directory should contain spec.md and plan.md at minimum.

---

## Statistics

**Completion Status**:
- ✅ **10 features complete** (001-006, 009, 013-014, 017)
- 📋 **2 features spec + plan + tasks complete** (018-019)
- 📋 **7 features spec + plan + tasks complete** (020-026)
- 🔮 **5 features reserved** (007-008, 010-012)

**Planning Phase Statistics**:
- **Total Specifications**: 18 feature specs (001-006, 009, 013-014, 017-026)
- **Total Task Breakdowns**: 9 features (018-026) with **308 total tasks**
- **Estimated Test Count**: 215+ tests for Features 018-026
- **Total Project Test Target**: 469+ tests (254 existing + 215 new)

**Documentation**:
- **Specs**: 5000+ lines (comprehensive specifications for all features)
- **Plans**: 3000+ lines (detailed implementation plans)
- **Tasks**: 2000+ lines (granular task breakdowns)
- **Implementation**: 2000+ lines (rush crate code to date)
- **Tests**: 1500+ lines (test code to date)

---

## Contributing

To add a new feature:

1. **Create feature directory**: `/specs/NNN-feature-name/`
2. **Write specification**: Create `spec.md` with user stories
3. **Create plan**: Write `plan.md` with implementation approach
4. **Generate tasks**: Break into implementation tasks
5. **Implement**: Follow the plan and keep code aligned with specs
6. **Test**: Write tests for all acceptance criteria
7. **Document**: Update this README with feature details

---

## Contact & Support

For questions about:
- **Specifications**: See individual feature's `spec.md`
- **Implementation**: See individual feature's `plan.md`
- **Status**: Check this README or individual feature status
- **Next steps**: See "Phase 3+" sections in plan files

---

**Last Updated**: 2025-12-06
**Total Features Documented**: 10 (001-006, 009, 013-014, 017)
**Total Features Complete**: 10 (001-006, 009, 013-014, 017)
