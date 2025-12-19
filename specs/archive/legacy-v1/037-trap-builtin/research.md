# Research: Trap Builtin Signal Handling

**Date**: 2025-12-10
**Feature**: Trap Builtin (037)
**Purpose**: Research signal handling patterns, nix API usage, and best practices for trap implementation

## Technical Decisions

### Decision 1: Signal Handling API (nix crate)

**Decision**: Use `nix::sys::signal` module from nix 0.29 (already in dependencies)

**Rationale**:
- Already integrated into rush for job control (see `executor/job.rs`)
- Provides safe Rust bindings to POSIX signal APIs
- Supports all required signals (SIGHUP through SIGUSR2, SIGRTMIN-SIGRTMAX)
- Handles platform differences (macOS vs Linux) transparently
- Zero additional dependencies

**Alternatives considered**:
- `signal-hook` crate: More features but adds dependency, overkill for our use case
- Raw `libc` bindings: Unsafe, platform-specific, reinvents what nix provides
- Custom FFI: Unnecessary complexity, nix already provides safe abstraction

**API Examples**:
```rust
use nix::sys::signal::{Signal, sigaction, SaFlags, SigAction, SigHandler, SigSet};

// Parse signal from string
let sig = Signal::from_str("INT")?;  // Or Signal::SIGINT

// Check if signal is catchable
fn is_catchable(sig: Signal) -> bool {
    !matches!(sig, Signal::SIGKILL | Signal::SIGSTOP)
}

// Signal number to name
let name = format!("{:?}", sig);  // "SIGINT"
```

### Decision 2: Trap Registry Storage

**Decision**: HashMap<Signal, String> in CommandExecutor

**Rationale**:
- CommandExecutor already manages shell state (variables, jobs, aliases)
- O(1) lookup for signal → handler command mapping
- Simple serialization (Signal enum is Copy + Eq + Hash)
- Lifetime matches shell session (cleared on exit)
- Memory efficient: ~16 bytes per entry + string length

**Alternatives considered**:
- Global static: Not safe for multiple shell instances, harder to test
- Separate TrapManager struct: Adds complexity, no clear benefit over HashMap
- Vec<(Signal, String)>: O(n) lookup, slower for typical 5-10 traps

**Implementation**:
```rust
use std::collections::HashMap;
use nix::sys::signal::Signal;

pub struct CommandExecutor {
    // ... existing fields ...
    trap_handlers: HashMap<Signal, String>,
}
```

### Decision 3: Handler Execution Strategy

**Decision**: Execute trap commands synchronously in signal handler context (with safety caveats)

**Rationale**:
- POSIX trap semantics expect synchronous execution before signal action
- Rust's signal-safe limitations require careful design
- Command execution already async-safe (uses fork+exec)
- Performance target: <100ms execution time

**Safety constraints**:
- Only async-signal-safe functions in signal handler
- No memory allocation in handler itself
- Use atomic flag + deferred execution pattern:
  1. Signal handler sets atomic flag
  2. Main REPL loop checks flag
  3. Execute trap command when safe

**Pattern**:
```rust
use std::sync::atomic::{AtomicBool, Ordering};

static TRAP_PENDING: AtomicBool = AtomicBool::new(false);

extern "C" fn signal_handler(_: libc::c_int) {
    TRAP_PENDING.store(true, Ordering::SeqCst);
}

// In REPL loop:
if TRAP_PENDING.swap(false, Ordering::SeqCst) {
    execute_trap_handler()?;
}
```

**Alternatives considered**:
- Async channels: Adds complexity, overkill for single-threaded shell
- Blocking execution in handler: Violates async-signal-safety
- Ignore handlers: Defeats purpose of trap command

### Decision 4: Signal Name Parsing

**Decision**: Case-insensitive matching, support both SIG prefix and bare names, support signal numbers

**Rationale**:
- User ergonomics: `trap 'cmd' INT` and `trap 'cmd' SIGINT` both work
- Shell compatibility: bash/zsh accept both forms
- Signal numbers: Support `trap 'cmd' 2` for backward compatibility

**Implementation**:
```rust
fn parse_signal(spec: &str) -> Result<Signal> {
    // Try numeric parse first
    if let Ok(num) = spec.parse::<i32>() {
        return Signal::try_from(num).ok_or(TrapError::InvalidSignal);
    }

    // Try with SIG prefix
    let with_prefix = if spec.to_uppercase().starts_with("SIG") {
        spec.to_uppercase()
    } else {
        format!("SIG{}", spec.to_uppercase())
    };

    Signal::from_str(&with_prefix)
        .map_err(|_| TrapError::InvalidSignal(spec.to_string()))
}
```

**Alternatives considered**:
- Strict POSIX (require SIG prefix): Poor UX, inconsistent with bash
- Case-sensitive: Error-prone, inconsistent with shell conventions
- Names only (no numbers): Breaks backward compatibility

### Decision 5: EXIT Pseudo-Signal Handling

**Decision**: Implement EXIT as special case outside normal signal registry

**Rationale**:
- EXIT is not a real signal, cannot use sigaction()
- Needs to trigger on shell termination, not signal delivery
- Execute after all real signal handlers but before shell cleanup

**Implementation**:
```rust
pub struct CommandExecutor {
    trap_handlers: HashMap<Signal, String>,
    exit_trap: Option<String>,  // Separate field for EXIT
}

impl Drop for CommandExecutor {
    fn drop(&mut self) {
        if let Some(cmd) = &self.exit_trap {
            let _ = self.execute_command(cmd);  // Ignore errors on shutdown
        }
    }
}
```

**Alternatives considered**:
- Treat EXIT as Signal enum variant: Requires modifying nix::Signal (not possible)
- Register atexit() handler: Global state, harder to test
- Ignore EXIT: Breaks POSIX compatibility, common use case

### Decision 6: Duplicate Trap Behavior

**Decision**: Error on duplicate registration (per FR-006 from spec clarification)

**Rationale**:
- Explicit user choice: prevents accidental overwrites
- Forces intentional clearing: `trap "" INT` then `trap 'new' INT`
- Better error detection: catches script bugs where multiple traps conflict

**Implementation**:
```rust
pub fn register_trap(&mut self, signal: Signal, command: String) -> Result<()> {
    if self.trap_handlers.contains_key(&signal) {
        return Err(TrapError::DuplicateTrap(signal));
    }
    self.trap_handlers.insert(signal, command);
    Ok(())
}
```

**Alternatives considered**:
- Allow overwrite (bash default): Chosen explicitly against in spec clarification
- Stack handlers: Too complex, not standard shell behavior
- Warn but allow: Inconsistent with error-on-duplicate requirement

## Best Practices

### Signal-Safe Code
- Use atomic operations for cross-signal communication
- Defer complex work to main loop
- No heap allocation in signal handlers
- Test with `cargo test --release` for timing-sensitive code

### Error Messages
- Include signal name in errors: "invalid signal specification: FOO" not "error: 1"
- Suggest fixes: "cannot trap SIGKILL (use EXIT for cleanup on all terminations)"
- Context-aware: "trap already exists for INT (use 'trap \"\" INT' to clear first)"

### Testing Strategy
- Unit tests: Signal parsing, registration, listing format
- Integration tests: Send actual signals, verify handler execution
- Use `kill -s SIGNAL $$` in tests to trigger handlers
- Test real-time signals on supported platforms

### Performance Optimization
- Lazy initialization: Don't allocate HashMap until first trap set
- Avoid string clones: Store &str where possible
- Fast path: Check trap_handlers.is_empty() before lookup

## Implementation Checklist

Phase 0 (Research) - Complete:
- [x] nix API patterns researched
- [x] Storage strategy decided
- [x] Handler execution approach defined
- [x] Signal parsing logic designed
- [x] EXIT pseudo-signal approach determined
- [x] Duplicate behavior clarified

Phase 1 (Design) - Next:
- [ ] Define TrapHandler and TrapRegistry types in data-model.md
- [ ] Document public API in contracts/trap-api.md
- [ ] Create usage examples in quickstart.md

Phase 2 (Implementation) - Future:
- [ ] Create trap.rs module
- [ ] Add trap_handlers field to CommandExecutor
- [ ] Implement signal parsing function
- [ ] Implement register/list/clear operations
- [ ] Write unit tests
- [ ] Write integration tests

## References

- nix documentation: https://docs.rs/nix/0.29.0/nix/sys/signal/
- POSIX signal specification: https://pubs.opengroup.org/onlinepubs/9699919799/functions/trap.html
- Bash trap documentation: https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html#index-trap
- Async-signal-safety: https://man7.org/linux/man-pages/man7/signal-safety.7.html
