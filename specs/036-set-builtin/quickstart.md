# Quickstart: Shell Options with set Builtin

**Feature**: 036-set-builtin

## What You'll Build

Extend the `set` builtin to support shell options that control error handling, debugging, and pipeline behavior.

## Prerequisites

- Rust 1.75+
- rush shell codebase
- Familiarity with shell scripting

## Quick Setup (5 minutes)

### Step 1: Add ShellOptions Struct

Create the option state tracker in `crates/rush/src/executor/execute.rs`:

```rust
#[derive(Debug, Clone)]
pub struct ShellOptions {
    pub errexit: bool,    // -e: exit on error
    pub xtrace: bool,     // -x: trace commands
    pub pipefail: bool,   // -o pipefail: detect pipeline failures
}

impl Default for ShellOptions {
    fn default() -> Self {
        Self {
            errexit: false,
            xtrace: false,
            pipefail: false,
        }
    }
}
```

### Step 2: Add to CommandExecutor

In `CommandExecutor` struct:

```rust
pub struct CommandExecutor {
    // ... existing fields
    shell_options: ShellOptions,
    conditional_depth: usize,  // Track if/while/&& contexts
}
```

Update `new()`:

```rust
pub fn new() -> Self {
    Self {
        // ... existing initialization
        shell_options: ShellOptions::default(),
        conditional_depth: 0,
    }
}
```

### Step 3: Extend set Builtin

In `crates/rush/src/executor/builtins/set.rs`, add option parsing:

```rust
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // Handle no args (existing code)
    if args.is_empty() {
        // ... list variables
    }

    // NEW: Handle shell options
    for arg in args {
        if arg.starts_with('-') || arg.starts_with('+') {
            parse_option(executor, arg)?;
        } else {
            // ... existing variable handling
        }
    }

    Ok(0)
}

fn parse_option(executor: &mut CommandExecutor, arg: &str) -> Result<()> {
    let enable = arg.starts_with('-');
    let opts = &arg[1..];

    if opts.is_empty() {
        return Ok(());  // "set -" or "set +" is no-op
    }

    // Handle -o/+o long form
    if opts.starts_with('o') {
        // ... parse long form
        return Ok(());
    }

    // Handle short form: -e, -x, etc.
    for ch in opts.chars() {
        match ch {
            'e' => executor.shell_options.errexit = enable,
            'x' => executor.shell_options.xtrace = enable,
            _ => eprintln!("rush: set: {}: invalid option", ch),
        }
    }

    Ok(())
}
```

### Step 4: Add set to Builtin Dispatch

In `crates/rush/src/executor/builtins/mod.rs`, add to the match:

```rust
pub fn execute_builtin(...) -> Option<Result<i32>> {
    match command {
        "set" => Some(set::execute(executor, args)),  // ADD THIS LINE
        "cd" => Some(cd::execute(executor, args)),
        // ... other builtins
    }
}
```

Also add at the top:

```rust
pub mod set;  // ADD THIS
```

### Step 5: Integrate into execute()

In `crates/rush/src/executor/execute.rs`, add option checks:

```rust
pub fn execute(&mut self, line: &str) -> Result<i32> {
    // ... existing expansion logic

    // NEW: Xtrace - print command before execution
    if self.shell_options.xtrace {
        eprintln!("+ {}", expanded_line);
    }

    // ... existing execution logic
    let exit_code = /* result from execution */;

    // NEW: Errexit - exit on error
    if self.shell_options.errexit
        && exit_code != 0
        && self.conditional_depth == 0
    {
        std::process::exit(exit_code);
    }

    Ok(exit_code)
}
```

## Testing

### Unit Test

```rust
#[test]
fn test_set_errexit() {
    let mut executor = CommandExecutor::new();
    set::execute(&mut executor, &vec!["-e".to_string()]).unwrap();
    assert!(executor.shell_options.errexit);
}
```

### Integration Test

```bash
# Create test script: test_errexit.sh
set -e
echo "start"
false
echo "not reached"  # Should not print

# Run with rush
$ rush test_errexit.sh
start
# (exits with code 1)
```

## Common Usage

```bash
# Exit on any error
set -e

# Trace commands (debugging)
set -x

# Detect pipeline failures
set -o pipefail

# Combine options
set -ex

# Disable option
set +e
```

## Next Steps

1. Implement pipefail in pipeline executor
2. Add conditional context tracking
3. Add `set -o` query output
4. Write comprehensive tests

## Resources

- Specification: [spec.md](./spec.md)
- Implementation Plan: [plan.md](./plan.md)
- Research: [research.md](./research.md)
