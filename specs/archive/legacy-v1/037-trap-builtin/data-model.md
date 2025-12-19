# Data Model: Trap Builtin

**Date**: 2025-12-10
**Feature**: Trap Builtin (037)
**Purpose**: Define data structures and relationships for trap signal handling

## Core Entities

### TrapHandler

Represents a single registered signal handler.

**Fields**:
```rust
pub struct TrapHandler {
    /// Signal being trapped (SIGINT, SIGTERM, etc.)
    pub signal: Signal,  // From nix::sys::signal::Signal

    /// Command string to execute when signal received
    pub command: String,

    /// When this handler was registered (for debugging/logging)
    pub registered_at: std::time::Instant,
}
```

**Invariants**:
- `signal` MUST NOT be SIGKILL or SIGSTOP (enforced at registration)
- `command` MUST NOT be empty (empty string reserved for clearing)
- Each signal can have at most ONE handler (uniqueness enforced by HashMap key)

**Lifecycle**:
1. **Creation**: User runs `trap 'command' SIGNAL`
2. **Registration**: Validated and inserted into TrapRegistry
3. **Execution**: Signal delivered → command executed via shell
4. **Removal**: User runs `trap "" SIGNAL` → handler removed from registry
5. **Termination**: Shell exits → all handlers cleared

### TrapRegistry

Collection of all active trap handlers for a shell session.

**Structure**:
```rust
use std::collections::HashMap;
use nix::sys::signal::Signal;

pub struct TrapRegistry {
    /// Maps signals to their handler commands
    handlers: HashMap<Signal, String>,

    /// Special EXIT pseudo-signal handler (executed on shell termination)
    exit_handler: Option<String>,
}
```

**Operations**:
```rust
impl TrapRegistry {
    /// Create empty registry
    pub fn new() -> Self;

    /// Register a trap handler for a signal
    /// Returns error if:
    /// - Signal is SIGKILL or SIGSTOP
    /// - Handler already exists for this signal (FR-006)
    pub fn register(&mut self, signal: Signal, command: String) -> Result<()>;

    /// Register EXIT pseudo-signal handler
    pub fn register_exit(&mut self, command: String) -> Result<()>;

    /// Clear trap handler for a signal
    /// Succeeds silently if no handler exists
    pub fn clear(&mut self, signal: Signal);

    /// Get handler command for a signal
    pub fn get(&self, signal: Signal) -> Option<&String>;

    /// List all registered handlers (for `trap` command with no args)
    pub fn list(&self) -> Vec<(Signal, &String)>;

    /// Check if signal has registered handler
    pub fn has_handler(&self, signal: Signal) -> bool;
}
```

**Invariants**:
- At most one handler per signal
- No handlers for SIGKILL or SIGSTOP
- exit_handler independent of regular signal handlers

**Memory footprint**:
- Empty registry: ~48 bytes (HashMap overhead)
- Per handler: ~16 bytes (Signal + String pointer) + string length
- Typical 5-10 traps: ~200-400 bytes total
- Meets <10MB performance target

### SignalSpec

User input for specifying signals (parsing layer).

**Structure**:
```rust
pub enum SignalSpec {
    /// Signal name (INT, SIGINT, TERM, etc.)
    Name(String),

    /// Signal number (2, 15, etc.)
    Number(i32),

    /// Pseudo-signal (EXIT)
    Pseudo(String),
}
```

**Parsing**:
```rust
impl SignalSpec {
    /// Parse signal specification from user input
    /// Accepts:
    /// - Names: INT, SIGINT, int, sigint (case-insensitive)
    /// - Numbers: 2, 15, 34 (POSIX signal numbers)
    /// - Pseudo: EXIT (case-insensitive)
    pub fn parse(input: &str) -> Result<Self>;

    /// Convert to nix::Signal
    /// Returns error for:
    /// - Invalid names (SIGFOO, XYZ)
    /// - Invalid numbers (negative, > max signal)
    /// - Uncatchable signals (SIGKILL=9, SIGSTOP=19)
    pub fn to_signal(&self) -> Result<Signal>;
}
```

**Examples**:
```rust
// Valid inputs
SignalSpec::parse("INT")?;       // -> Name("INT")
SignalSpec::parse("SIGINT")?;    // -> Name("SIGINT")
SignalSpec::parse("2")?;         // -> Number(2)
SignalSpec::parse("EXIT")?;      // -> Pseudo("EXIT")

// Invalid inputs
SignalSpec::parse("INVALID")?;   // Error: InvalidSignal
SignalSpec::parse("9")?;         // Error: UncatchableSignal (SIGKILL)
SignalSpec::parse("-1")?;        // Error: InvalidSignal
```

## Relationships

```text
┌─────────────────┐
│ CommandExecutor │
│                 │
│ Contains:       │
│  - Variables    │
│  - Job Manager  │
│  - TrapRegistry │◄─────┐
└─────────────────┘      │
                         │
                    ┌────┴──────────┐
                    │ TrapRegistry  │
                    │               │
                    │ handlers: Map │
                    └───┬───────────┘
                        │
                        │ Contains 0..N
                        │
                    ┌───▼─────────┐
                    │ TrapHandler │
                    │             │
                    │ signal: Sig │
                    │ command: Str│
                    └─────────────┘
```

**Ownership**:
- CommandExecutor OWNS TrapRegistry
- TrapRegistry OWNS all TrapHandler data
- Signal handlers REFERENCE registry via executor

**Concurrency**:
- Single-threaded access (rush is single-threaded REPL)
- Signal handlers set atomic flags, don't directly mutate registry
- Main loop processes pending traps when safe

## Validation Rules

### Signal Validation

**Valid signals** (✅):
- POSIX standard: SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGUSR1, SIGUSR2, etc.
- Real-time: SIGRTMIN through SIGRTMAX (typically 34-64 on Linux, 37-63 on macOS)
- Pseudo: EXIT

**Invalid signals** (❌):
- SIGKILL (9) - Cannot be caught per OS restriction
- SIGSTOP (19) - Cannot be caught per OS restriction
- Invalid names: SIGFOO, XYZ, INVALID
- Out-of-range numbers: -1, 999, 65

### Command Validation

**Valid commands** (✅):
- Any non-empty string: `rm /tmp/lockfile`
- Shell functions: `cleanup_function`
- Complex commands: `echo "Exiting" >> log.txt`
- Empty string ONLY for clearing: `""`

**Invalid commands** (❌):
- Empty string when registering (reserved for clearing)

### Duplicate Validation

**FR-006 Enforcement**:
```rust
// ❌ Error case
trap 'first' INT
trap 'second' INT  // Error: "trap already exists for signal INT"

// ✅ Correct approach
trap 'first' INT
trap "" INT       // Clear existing
trap 'second' INT // Now succeeds
```

## State Transitions

### Trap Handler Lifecycle

```text
   [Not Registered]
         │
         │ trap 'cmd' SIGNAL
         │
         ▼
    [Registered]
         │
         ├──► Signal delivered → [Executing] → [Registered]
         │
         │ trap "" SIGNAL
         │
         ▼
   [Cleared/Not Registered]
```

### EXIT Handler Lifecycle

```text
   [Not Registered]
         │
         │ trap 'cmd' EXIT
         │
         ▼
    [Registered]
         │
         │ Shell exits (normal or signal)
         │
         ▼
    [Executing]
         │
         │ Command completes
         │
         ▼
    [Shell Terminated]
```

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Target |
|-----------|------------|--------|
| Register  | O(1) | <1ms |
| Clear     | O(1) | <1ms |
| Get       | O(1) | <1μs |
| List      | O(n) | <5s for n=63 |
| Execute   | O(cmd) | <100ms per FR-002 |

### Space Complexity

| Structure | Size | Notes |
|-----------|------|-------|
| Empty TrapRegistry | ~48 bytes | HashMap overhead |
| Per handler | ~16 bytes + len(command) | Signal + String pointer |
| Max capacity (63 signals) | ~1.5KB + commands | All POSIX + RT signals |
| Typical (5-10 traps) | ~300 bytes | Real-world usage |

## Integration Points

### CommandExecutor

**Modifications required**:
```rust
pub struct CommandExecutor {
    // ... existing fields ...

    /// Trap signal handlers
    trap_registry: TrapRegistry,  // [ADD]
}

impl CommandExecutor {
    /// Access trap registry
    pub fn trap_registry(&self) -> &TrapRegistry;
    pub fn trap_registry_mut(&mut self) -> &mut TrapRegistry;

    /// Execute trap handler for signal (called by signal delivery system)
    pub fn execute_trap(&mut self, signal: Signal) -> Result<()>;
}
```

### Builtin Dispatcher

**Registration**:
```rust
// In executor/builtins/mod.rs
pub fn execute_builtin(executor: &mut CommandExecutor, command: &str, args: &[String]) -> Option<Result<i32>> {
    match command {
        // ... existing builtins ...
        "trap" => Some(trap::execute(executor, args)),  // [ADD]
        _ => None,
    }
}
```

## Error Handling

### Error Types

```rust
pub enum TrapError {
    /// Invalid signal specification (name or number)
    InvalidSignal(String),

    /// Attempt to trap uncatchable signal (SIGKILL, SIGSTOP)
    UncatchableSignal(Signal),

    /// Duplicate trap registration (FR-006)
    DuplicateTrap(Signal),

    /// Empty command when registering (not clearing)
    EmptyCommand,
}

impl Display for TrapError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidSignal(spec) =>
                write!(f, "invalid signal specification: {}", spec),
            Self::UncatchableSignal(sig) =>
                write!(f, "cannot trap {}: signal cannot be caught", sig),
            Self::DuplicateTrap(sig) =>
                write!(f, "trap already exists for signal {} (use 'trap \"\" {}' to clear first)", sig, sig),
            Self::EmptyCommand =>
                write!(f, "empty command (use 'trap \"\" SIGNAL' to clear)"),
        }
    }
}
```

## Testing Strategy

### Unit Tests

**Data structure tests**:
- TrapRegistry creation, insertion, removal
- SignalSpec parsing (valid/invalid inputs)
- Error message formatting

**Test data**:
```rust
#[test]
fn test_registry_operations() {
    let mut registry = TrapRegistry::new();

    // Register
    assert!(registry.register(Signal::SIGINT, "cleanup".to_string()).is_ok());

    // Duplicate error
    assert!(registry.register(Signal::SIGINT, "other".to_string()).is_err());

    // Get
    assert_eq!(registry.get(Signal::SIGINT), Some(&"cleanup".to_string()));

    // Clear
    registry.clear(Signal::SIGINT);
    assert_eq!(registry.get(Signal::SIGINT), None);
}
```

### Integration Tests

**End-to-end scenarios**:
- Register trap → send signal → verify handler executed
- List traps → verify output format
- Clear trap → send signal → verify default behavior

## Future Considerations

### Post-MVP Extensions

**Not in scope for 037**:
- Stacked handlers (multiple handlers per signal)
- DEBUG pseudo-signal (executed before each command)
- ERR pseudo-signal (executed on command error)
- RETURN pseudo-signal (executed on function return)

**Extensibility**:
- Current HashMap design allows easy addition of new pseudo-signals
- exit_handler pattern can be replicated for other pseudo-signals
- No breaking changes required for future extensions
