# Data Model: Set Builtin Shell Options

**Feature**: 036-set-builtin
**Date**: 2025-12-09

## Overview

This feature adds shell option state tracking to the CommandExecutor. No persistent storage required - all state is in-memory for the duration of the shell session.

## Entities

### ShellOptions

**Purpose**: Stores the current state of all shell options

**Fields**:
- `errexit: bool` - If true, shell exits immediately when any command returns non-zero exit code (except in conditional contexts)
- `xtrace: bool` - If true, shell prints each command to stderr with `+` prefix before execution
- `pipefail: bool` - If true, pipelines return the exit code of the first failing command, not just the last command

**Relationships**: Owned by CommandExecutor (1:1 relationship)

**Validation Rules**:
- All fields are boolean (no validation needed)
- Default values: all false (off)

**State Transitions**:
```
Initial State: { errexit: false, xtrace: false, pipefail: false }

set -e  → errexit: true
set +e  → errexit: false
set -x  → xtrace: true
set +x  → xtrace: false
set -o pipefail  → pipefail: true
set +o pipefail  → pipefail: false
```

### CommandExecutor (Modified)

**Purpose**: Executes shell commands with option state awareness

**New Fields**:
- `shell_options: ShellOptions` - Current shell option state
- `conditional_depth: usize` - Nesting depth of conditional contexts (for errexit exception handling)

**Modified Behavior**:
- Before executing: Check xtrace, print command if enabled
- After executing: Check errexit, exit if enabled and conditions met
- During pipeline: Check pipefail, adjust exit code collection

## Data Flow

### Setting an Option

```
User Input: "set -e"
  ↓
Parse args: ["-e"]
  ↓
Identify option: "errexit"
  ↓
Update state: shell_options.errexit = true
  ↓
Store in CommandExecutor
```

### Executing with Xtrace

```
User Input: "echo $USER"
  ↓
Expand variables: "echo john"
  ↓
Check xtrace: shell_options.xtrace == true
  ↓
Print to stderr: "+ echo john"
  ↓
Execute command
  ↓
Return exit code
```

### Executing with Errexit

```
User Input: "false"
  ↓
Execute command: exit_code = 1
  ↓
Check errexit: shell_options.errexit == true
  ↓
Check conditional depth: conditional_depth == 0
  ↓
Exit shell: std::process::exit(1)
```

### Pipeline with Pipefail

```
User Input: "false | cat | true"
  ↓
Execute pipeline:
  - false → exit_code = 1
  - cat → exit_code = 0
  - true → exit_code = 0
  ↓
Collect codes: [1, 0, 0]
  ↓
Check pipefail: shell_options.pipefail == true
  ↓
Return first non-zero: 1
  (default would return last: 0)
```

## Implementation Notes

**Thread Safety**: Each CommandExecutor instance has its own ShellOptions. No shared state, no locks needed.

**Memory**: ShellOptions is 3 bytes (3 bools). Conditional depth is 8 bytes (usize). Total overhead: 11 bytes per CommandExecutor.

**Performance**: All accesses are direct field reads (O(1), zero allocation). Option checks add <0.1ms overhead.

**Persistence**: Options are not persisted across shell sessions. Users must set options in startup scripts if desired.
