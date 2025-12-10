//! Implementation of the `trap` builtin command
//!
//! The `trap` builtin manages signal handlers for cleanup operations and interruption handling.
//!
//! # Usage
//!
//! - `trap 'command' SIGNAL [SIGNAL...]` - Register a handler for signals
//! - `trap` - List all active trap handlers
//! - `trap "" SIGNAL [SIGNAL...]` - Clear trap handlers for signals
//!
//! # Examples
//!
//! ```ignore
//! // Register cleanup handler for SIGINT (Ctrl+C)
//! trap 'rm /tmp/lockfile' INT
//!
//! // Register same handler for multiple signals
//! trap 'cleanup_function' INT TERM QUIT
//!
//! // Register EXIT pseudo-signal for cleanup on shell termination
//! trap 'echo Exiting' EXIT
//!
//! // List all active traps
//! trap
//!
//! // Clear a trap handler
//! trap "" INT
//! ```

use crate::error::{Result, RushError};
use crate::executor::execute::CommandExecutor;
use nix::sys::signal::Signal;
use std::collections::HashMap;

/// Signal specification from user input
#[derive(Debug, Clone, PartialEq)]
pub enum SignalSpec {
    /// Signal name (INT, SIGINT, etc.)
    Name(String),
    /// Signal number (2, 15, etc.)
    Number(i32),
    /// Pseudo-signal (EXIT)
    Pseudo(String),
}

impl SignalSpec {
    /// Parse signal specification from user input
    ///
    /// Accepts:
    /// - Names: INT, SIGINT, int, sigint (case-insensitive)
    /// - Numbers: 2, 15, 34 (POSIX signal numbers)
    /// - Pseudo: EXIT (case-insensitive)
    pub fn parse(input: &str) -> Result<Self> {
        // Check for EXIT pseudo-signal (case-insensitive)
        if input.eq_ignore_ascii_case("EXIT") {
            return Ok(SignalSpec::Pseudo("EXIT".to_string()));
        }

        // Try to parse as number
        if let Ok(num) = input.parse::<i32>() {
            return Ok(SignalSpec::Number(num));
        }

        // Otherwise it's a signal name
        Ok(SignalSpec::Name(input.to_string()))
    }

    /// Convert SignalSpec to nix::Signal
    ///
    /// Returns error for:
    /// - Invalid names (SIGFOO, XYZ)
    /// - Invalid numbers (negative, out of range)
    /// - Uncatchable signals (SIGKILL=9, SIGSTOP=19)
    pub fn to_signal(&self) -> Result<Signal> {
        match self {
            SignalSpec::Pseudo(_) => {
                // EXIT pseudo-signal doesn't convert to a real signal
                Err(RushError::InvalidSignal("EXIT is a pseudo-signal, not a real signal".to_string()))
            }
            SignalSpec::Number(num) => {
                // Try to convert number to signal
                let signal = Signal::try_from(*num)
                    .map_err(|_| RushError::InvalidSignal(num.to_string()))?;

                // Check for uncatchable signals
                if matches!(signal, Signal::SIGKILL | Signal::SIGSTOP) {
                    return Err(RushError::UncatchableSignal(format!("{:?}", signal)));
                }

                Ok(signal)
            }
            SignalSpec::Name(name) => {
                // Normalize signal name (add SIG prefix if missing, convert to uppercase)
                let normalized = if name.to_uppercase().starts_with("SIG") {
                    name.to_uppercase()
                } else {
                    format!("SIG{}", name.to_uppercase())
                };

                // Try to parse signal from string
                let signal = match normalized.as_str() {
                    "SIGHUP" => Signal::SIGHUP,
                    "SIGINT" => Signal::SIGINT,
                    "SIGQUIT" => Signal::SIGQUIT,
                    "SIGILL" => Signal::SIGILL,
                    "SIGTRAP" => Signal::SIGTRAP,
                    "SIGABRT" => Signal::SIGABRT,
                    "SIGBUS" => Signal::SIGBUS,
                    "SIGFPE" => Signal::SIGFPE,
                    "SIGKILL" => return Err(RushError::UncatchableSignal("SIGKILL".to_string())),
                    "SIGUSR1" => Signal::SIGUSR1,
                    "SIGSEGV" => Signal::SIGSEGV,
                    "SIGUSR2" => Signal::SIGUSR2,
                    "SIGPIPE" => Signal::SIGPIPE,
                    "SIGALRM" => Signal::SIGALRM,
                    "SIGTERM" => Signal::SIGTERM,
                    "SIGCHLD" => Signal::SIGCHLD,
                    "SIGCONT" => Signal::SIGCONT,
                    "SIGSTOP" => return Err(RushError::UncatchableSignal("SIGSTOP".to_string())),
                    "SIGTSTP" => Signal::SIGTSTP,
                    "SIGTTIN" => Signal::SIGTTIN,
                    "SIGTTOU" => Signal::SIGTTOU,
                    "SIGURG" => Signal::SIGURG,
                    "SIGXCPU" => Signal::SIGXCPU,
                    "SIGXFSZ" => Signal::SIGXFSZ,
                    "SIGVTALRM" => Signal::SIGVTALRM,
                    "SIGPROF" => Signal::SIGPROF,
                    "SIGWINCH" => Signal::SIGWINCH,
                    "SIGIO" => Signal::SIGIO,
                    "SIGSYS" => Signal::SIGSYS,
                    _ => return Err(RushError::InvalidSignal(name.clone())),
                };

                Ok(signal)
            }
        }
    }
}

/// Registry of active trap handlers
pub struct TrapRegistry {
    /// Maps signals to their handler commands
    handlers: HashMap<Signal, String>,
    /// Special EXIT pseudo-signal handler
    exit_handler: Option<String>,
}

impl TrapRegistry {
    /// Create a new empty trap registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            exit_handler: None,
        }
    }

    /// Register a trap handler for a signal
    ///
    /// Returns error if handler already exists for this signal (FR-006)
    pub fn register(&mut self, signal: Signal, command: String) -> Result<()> {
        if self.handlers.contains_key(&signal) {
            return Err(RushError::DuplicateTrap(format!("{:?}", signal)));
        }
        self.handlers.insert(signal, command);
        Ok(())
    }

    /// Register EXIT pseudo-signal handler
    pub fn register_exit(&mut self, command: String) -> Result<()> {
        if self.exit_handler.is_some() {
            return Err(RushError::DuplicateTrap("EXIT".to_string()));
        }
        self.exit_handler = Some(command);
        Ok(())
    }

    /// Clear trap handler for a signal
    pub fn clear(&mut self, signal: Signal) {
        self.handlers.remove(&signal);
    }

    /// Clear EXIT trap handler
    pub fn clear_exit(&mut self) {
        self.exit_handler = None;
    }

    /// Get handler command for a signal
    pub fn get(&self, signal: Signal) -> Option<&String> {
        self.handlers.get(&signal)
    }

    /// Get EXIT handler
    pub fn get_exit(&self) -> Option<&String> {
        self.exit_handler.as_ref()
    }

    /// List all registered handlers (sorted by signal number)
    pub fn list(&self) -> Vec<(Signal, &String)> {
        let mut handlers: Vec<(Signal, &String)> = self.handlers.iter()
            .map(|(sig, cmd)| (*sig, cmd))
            .collect();
        handlers.sort_by_key(|(sig, _)| *sig as i32);
        handlers
    }

    /// Check if signal has registered handler
    pub fn has_handler(&self, signal: Signal) -> bool {
        self.handlers.contains_key(&signal)
    }
}

/// Execute the `trap` builtin command
///
/// Handles signal trap registration, listing, and clearing.
///
/// # Arguments
///
/// * `executor` - Mutable reference to the command executor
/// * `args` - Command arguments (command string and signal specifications)
///
/// # Returns
///
/// * `Ok(0)` - Success
/// * `Ok(1)` - Error (invalid signal, duplicate trap, etc.)
pub fn execute(executor: &mut CommandExecutor, args: &[String]) -> Result<i32> {
    // T022: Handle "trap" with no args (list all traps)
    if args.is_empty() {
        return list_traps(executor);
    }

    // T021: Parse command-line arguments
    // Syntax: trap 'command' SIGNAL [SIGNAL...]
    // Or:     trap '' SIGNAL [SIGNAL...] (clear handler)

    if args.len() < 2 {
        eprintln!("trap: usage: trap 'command' SIGNAL [SIGNAL...]");
        return Ok(1);
    }

    let command = &args[0];
    let signal_specs = &args[1..];

    // T024: Handle "trap '' SIGNAL..." (clear handler)
    if command.is_empty() {
        return clear_traps(executor, signal_specs);
    }

    // T023: Handle "trap 'command' SIGNAL..." (register handler)
    register_traps(executor, command, signal_specs)
}

/// T022: List all registered trap handlers
fn list_traps(executor: &CommandExecutor) -> Result<i32> {
    let registry = executor.trap_registry();

    // List regular signal handlers (sorted by signal number)
    for (signal, command) in registry.list() {
        println!("trap -- '{}' {:?}", command, signal);
    }

    // List EXIT handler if present
    if let Some(command) = registry.get_exit() {
        println!("trap -- '{}' EXIT", command);
    }

    Ok(0)
}

/// T023, T025-T033: Register trap handlers for multiple signals
fn register_traps(executor: &mut CommandExecutor, command: &str, signal_specs: &[String]) -> Result<i32> {
    // T025: Validate signal specifications first (fail fast)
    let mut parsed_signals = Vec::new();

    for spec_str in signal_specs {
        // Parse signal specification
        let spec = SignalSpec::parse(spec_str)?;

        // T032: Support for EXIT pseudo-signal
        match spec {
            SignalSpec::Pseudo(ref name) if name == "EXIT" => {
                // Handle EXIT separately
                parsed_signals.push((spec_str.clone(), None));
            }
            _ => {
                // T025, T028: Validate signal (rejects SIGKILL, SIGSTOP, invalid signals)
                let signal = spec.to_signal()?;
                parsed_signals.push((spec_str.clone(), Some(signal)));
            }
        }
    }

    // T029: Check for duplicate traps before registering any (FR-006)
    let registry = executor.trap_registry();
    for (spec_str, signal_opt) in &parsed_signals {
        if let Some(signal) = signal_opt {
            if registry.has_handler(*signal) {
                return Err(RushError::DuplicateTrap(spec_str.clone()));
            }
        } else {
            // EXIT pseudo-signal
            if registry.get_exit().is_some() {
                return Err(RushError::DuplicateTrap("EXIT".to_string()));
            }
        }
    }

    // T026: Register handlers in TrapRegistry (all validations passed)
    // T031: Support for multiple signals in one command
    let registry = executor.trap_registry_mut();
    for (_, signal_opt) in parsed_signals {
        if let Some(signal) = signal_opt {
            registry.register(signal, command.to_string())?;
        } else {
            // EXIT pseudo-signal
            registry.register_exit(command.to_string())?;
        }
    }

    // T033: Return success exit code
    Ok(0)
}

/// T024, T027: Clear trap handlers for multiple signals
fn clear_traps(executor: &mut CommandExecutor, signal_specs: &[String]) -> Result<i32> {
    // Parse and validate all signals first
    let mut parsed_signals = Vec::new();

    for spec_str in signal_specs {
        let spec = SignalSpec::parse(spec_str)?;

        match spec {
            SignalSpec::Pseudo(ref name) if name == "EXIT" => {
                parsed_signals.push(None); // None = EXIT
            }
            _ => {
                // Validate signal (rejects SIGKILL, SIGSTOP, invalid signals)
                let signal = spec.to_signal()?;
                parsed_signals.push(Some(signal));
            }
        }
    }

    // T027: Clear handlers in TrapRegistry
    let registry = executor.trap_registry_mut();
    for signal_opt in parsed_signals {
        if let Some(signal) = signal_opt {
            registry.clear(signal);
        } else {
            // EXIT pseudo-signal
            registry.clear_exit();
        }
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::execute::CommandExecutor;

    // ===== T013: Unit test for signal parsing (valid names) =====
    #[test]
    fn test_signal_spec_parse_valid_names() {
        // Case-insensitive name variations
        assert_eq!(SignalSpec::parse("INT").unwrap(), SignalSpec::Name("INT".to_string()));
        assert_eq!(SignalSpec::parse("int").unwrap(), SignalSpec::Name("int".to_string()));
        assert_eq!(SignalSpec::parse("SIGINT").unwrap(), SignalSpec::Name("SIGINT".to_string()));
        assert_eq!(SignalSpec::parse("sigint").unwrap(), SignalSpec::Name("sigint".to_string()));
        assert_eq!(SignalSpec::parse("SiGiNt").unwrap(), SignalSpec::Name("SiGiNt".to_string()));

        // Numbers
        assert_eq!(SignalSpec::parse("2").unwrap(), SignalSpec::Number(2));
        assert_eq!(SignalSpec::parse("15").unwrap(), SignalSpec::Number(15));
        assert_eq!(SignalSpec::parse("9").unwrap(), SignalSpec::Number(9));
    }

    #[test]
    fn test_signal_spec_to_signal_valid_names() {
        // Names with SIG prefix
        assert_eq!(SignalSpec::Name("SIGINT".to_string()).to_signal().unwrap(), Signal::SIGINT);
        assert_eq!(SignalSpec::Name("SIGTERM".to_string()).to_signal().unwrap(), Signal::SIGTERM);
        assert_eq!(SignalSpec::Name("SIGHUP".to_string()).to_signal().unwrap(), Signal::SIGHUP);

        // Names without SIG prefix (case-insensitive)
        assert_eq!(SignalSpec::Name("INT".to_string()).to_signal().unwrap(), Signal::SIGINT);
        assert_eq!(SignalSpec::Name("int".to_string()).to_signal().unwrap(), Signal::SIGINT);
        assert_eq!(SignalSpec::Name("term".to_string()).to_signal().unwrap(), Signal::SIGTERM);

        // Numbers
        assert_eq!(SignalSpec::Number(2).to_signal().unwrap(), Signal::SIGINT);
        assert_eq!(SignalSpec::Number(15).to_signal().unwrap(), Signal::SIGTERM);
    }

    // ===== T014: Unit test for EXIT pseudo-signal =====
    #[test]
    fn test_signal_spec_parse_exit_pseudo_signal() {
        // Case-insensitive EXIT
        assert_eq!(SignalSpec::parse("EXIT").unwrap(), SignalSpec::Pseudo("EXIT".to_string()));
        assert_eq!(SignalSpec::parse("exit").unwrap(), SignalSpec::Pseudo("EXIT".to_string()));
        assert_eq!(SignalSpec::parse("Exit").unwrap(), SignalSpec::Pseudo("EXIT".to_string()));
        assert_eq!(SignalSpec::parse("ExIt").unwrap(), SignalSpec::Pseudo("EXIT".to_string()));
    }

    #[test]
    fn test_signal_spec_exit_does_not_convert_to_signal() {
        let spec = SignalSpec::Pseudo("EXIT".to_string());
        let result = spec.to_signal();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pseudo-signal"));
    }

    // ===== T015: Unit test for signal parsing error cases =====
    #[test]
    fn test_signal_spec_invalid_names() {
        // Invalid signal names should error when converting to Signal
        let invalid = SignalSpec::Name("SIGFOO".to_string());
        assert!(invalid.to_signal().is_err());

        let invalid2 = SignalSpec::Name("XYZ".to_string());
        assert!(invalid2.to_signal().is_err());

        let invalid3 = SignalSpec::Name("NOTASIGNAL".to_string());
        assert!(invalid3.to_signal().is_err());
    }

    #[test]
    fn test_signal_spec_uncatchable_signals() {
        // SIGKILL (9) cannot be caught
        let kill_name = SignalSpec::Name("SIGKILL".to_string());
        let result = kill_name.to_signal();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SIGKILL"));

        let kill_num = SignalSpec::Number(9);
        let result = kill_num.to_signal();
        assert!(result.is_err());

        // SIGSTOP (19 on macOS) cannot be caught
        let stop_name = SignalSpec::Name("SIGSTOP".to_string());
        let result = stop_name.to_signal();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SIGSTOP"));
    }

    #[test]
    fn test_signal_spec_invalid_numbers() {
        // Invalid signal numbers
        let invalid = SignalSpec::Number(999);
        assert!(invalid.to_signal().is_err());

        let negative = SignalSpec::Number(-1);
        assert!(negative.to_signal().is_err());
    }

    // ===== T016: Unit test for TrapRegistry::register (success path) =====
    #[test]
    fn test_trap_registry_register_success() {
        let mut registry = TrapRegistry::new();

        // Register first trap
        let result = registry.register(Signal::SIGINT, "echo Interrupted".to_string());
        assert!(result.is_ok());

        // Verify it was registered
        assert!(registry.has_handler(Signal::SIGINT));
        assert_eq!(registry.get(Signal::SIGINT).unwrap(), "echo Interrupted");

        // Register different signal
        let result = registry.register(Signal::SIGTERM, "cleanup".to_string());
        assert!(result.is_ok());
        assert_eq!(registry.get(Signal::SIGTERM).unwrap(), "cleanup");
    }

    // ===== T017: Unit test for TrapRegistry::register (duplicate error - FR-006) =====
    #[test]
    fn test_trap_registry_register_duplicate_error() {
        let mut registry = TrapRegistry::new();

        // Register first trap
        registry.register(Signal::SIGINT, "first handler".to_string()).unwrap();

        // Try to register duplicate - should error (FR-006)
        let result = registry.register(Signal::SIGINT, "second handler".to_string());
        assert!(result.is_err());

        // Verify error message mentions duplicate
        let err = result.unwrap_err();
        assert!(err.to_string().contains("already exists"));

        // Original handler should still be there
        assert_eq!(registry.get(Signal::SIGINT).unwrap(), "first handler");
    }

    // ===== T018: Unit test for TrapRegistry::register_exit (success + duplicate) =====
    #[test]
    fn test_trap_registry_register_exit_success() {
        let mut registry = TrapRegistry::new();

        // Register EXIT handler
        let result = registry.register_exit("echo Exiting".to_string());
        assert!(result.is_ok());

        // Verify it was registered
        assert_eq!(registry.get_exit().unwrap(), "echo Exiting");
    }

    #[test]
    fn test_trap_registry_register_exit_duplicate_error() {
        let mut registry = TrapRegistry::new();

        // Register first EXIT handler
        registry.register_exit("first exit".to_string()).unwrap();

        // Try to register duplicate - should error
        let result = registry.register_exit("second exit".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        // Original handler should still be there
        assert_eq!(registry.get_exit().unwrap(), "first exit");
    }

    #[test]
    fn test_trap_registry_clear() {
        let mut registry = TrapRegistry::new();

        // Register and clear normal signal
        registry.register(Signal::SIGINT, "handler".to_string()).unwrap();
        assert!(registry.has_handler(Signal::SIGINT));

        registry.clear(Signal::SIGINT);
        assert!(!registry.has_handler(Signal::SIGINT));
        assert!(registry.get(Signal::SIGINT).is_none());
    }

    #[test]
    fn test_trap_registry_clear_exit() {
        let mut registry = TrapRegistry::new();

        // Register and clear EXIT
        registry.register_exit("exit handler".to_string()).unwrap();
        assert!(registry.get_exit().is_some());

        registry.clear_exit();
        assert!(registry.get_exit().is_none());
    }

    #[test]
    fn test_trap_registry_list() {
        let mut registry = TrapRegistry::new();

        // Register multiple handlers
        registry.register(Signal::SIGTERM, "term handler".to_string()).unwrap();
        registry.register(Signal::SIGINT, "int handler".to_string()).unwrap();
        registry.register(Signal::SIGHUP, "hup handler".to_string()).unwrap();

        // List should return all handlers sorted by signal number
        let list = registry.list();
        assert_eq!(list.len(), 3);

        // Verify sorted order (SIGHUP=1, SIGINT=2, SIGTERM=15)
        assert_eq!(list[0].0, Signal::SIGHUP);
        assert_eq!(list[1].0, Signal::SIGINT);
        assert_eq!(list[2].0, Signal::SIGTERM);
    }

    // ===== T019: Integration test for trap command registration (single signal) =====
    #[test]
    fn test_trap_execute_register_single_signal() {
        let mut executor = CommandExecutor::new();

        // trap 'echo Interrupted' INT
        let result = execute(
            &mut executor,
            &["echo Interrupted".to_string(), "INT".to_string()],
        );

        // Should succeed
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify handler was registered
        let registry = executor.trap_registry();
        assert_eq!(registry.get(Signal::SIGINT).unwrap(), "echo Interrupted");
    }

    // ===== T020: Integration test for trap command registration (multiple signals) =====
    #[test]
    fn test_trap_execute_register_multiple_signals() {
        let mut executor = CommandExecutor::new();

        // trap 'cleanup' INT TERM QUIT
        let result = execute(
            &mut executor,
            &[
                "cleanup".to_string(),
                "INT".to_string(),
                "TERM".to_string(),
                "QUIT".to_string(),
            ],
        );

        // Should succeed
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify all handlers were registered
        let registry = executor.trap_registry();
        assert_eq!(registry.get(Signal::SIGINT).unwrap(), "cleanup");
        assert_eq!(registry.get(Signal::SIGTERM).unwrap(), "cleanup");
        assert_eq!(registry.get(Signal::SIGQUIT).unwrap(), "cleanup");
    }

    #[test]
    fn test_trap_execute_register_exit_signal() {
        let mut executor = CommandExecutor::new();

        // trap 'echo Exiting' EXIT
        let result = execute(
            &mut executor,
            &["echo Exiting".to_string(), "EXIT".to_string()],
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify EXIT handler was registered
        let registry = executor.trap_registry();
        assert_eq!(registry.get_exit().unwrap(), "echo Exiting");
    }

    #[test]
    fn test_trap_execute_clear_single_signal() {
        let mut executor = CommandExecutor::new();

        // First register a trap
        execute(&mut executor, &["handler".to_string(), "INT".to_string()]).unwrap();
        assert!(executor.trap_registry().has_handler(Signal::SIGINT));

        // Clear it with empty command
        let result = execute(&mut executor, &["".to_string(), "INT".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify it was cleared
        assert!(!executor.trap_registry().has_handler(Signal::SIGINT));
    }

    #[test]
    fn test_trap_execute_clear_multiple_signals() {
        let mut executor = CommandExecutor::new();

        // Register multiple traps
        execute(
            &mut executor,
            &[
                "handler".to_string(),
                "INT".to_string(),
                "TERM".to_string(),
            ],
        )
        .unwrap();

        // Clear them all
        let result = execute(
            &mut executor,
            &["".to_string(), "INT".to_string(), "TERM".to_string()],
        );
        assert!(result.is_ok());

        // Verify all cleared
        assert!(!executor.trap_registry().has_handler(Signal::SIGINT));
        assert!(!executor.trap_registry().has_handler(Signal::SIGTERM));
    }

    #[test]
    fn test_trap_execute_duplicate_error() {
        let mut executor = CommandExecutor::new();

        // Register first trap
        execute(&mut executor, &["first".to_string(), "INT".to_string()]).unwrap();

        // Try to register duplicate - should error
        let result = execute(&mut executor, &["second".to_string(), "INT".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        // Original should still be there
        assert_eq!(executor.trap_registry().get(Signal::SIGINT).unwrap(), "first");
    }

    #[test]
    fn test_trap_execute_invalid_signal() {
        let mut executor = CommandExecutor::new();

        // Try to trap SIGKILL - should error
        let result = execute(&mut executor, &["handler".to_string(), "KILL".to_string()]);
        assert!(result.is_err());

        // Try invalid signal name
        let result = execute(&mut executor, &["handler".to_string(), "INVALID".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_trap_execute_no_args_empty_list() {
        let mut executor = CommandExecutor::new();

        // List with no traps registered - should succeed
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_trap_execute_insufficient_args() {
        let mut executor = CommandExecutor::new();

        // Only one argument - should error
        let result = execute(&mut executor, &["handler".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Error exit code
    }

    #[test]
    fn test_trap_builtin_exists() {
        let mut executor = CommandExecutor::new();
        let result = execute(&mut executor, &[]);
        assert!(result.is_ok());
    }

    // ===== Phase 4: User Story 2 - Enhanced Listing Tests =====

    // T034: Unit test for empty trap listing
    #[test]
    fn test_trap_list_empty() {
        let executor = CommandExecutor::new();
        let registry = executor.trap_registry();

        // Empty registry should return empty list
        let list = registry.list();
        assert_eq!(list.len(), 0);

        // EXIT handler should also be None
        assert!(registry.get_exit().is_none());
    }

    // T035: Unit test for single trap listing format
    #[test]
    fn test_trap_list_single_format() {
        let mut executor = CommandExecutor::new();

        // Register single trap
        execute(&mut executor, &["echo test".to_string(), "INT".to_string()]).unwrap();

        let registry = executor.trap_registry();
        let list = registry.list();

        // Should have exactly one entry
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, Signal::SIGINT);
        assert_eq!(list[0].1, "echo test");
    }

    // T036: Unit test for multiple trap listing format
    #[test]
    fn test_trap_list_multiple_sorted() {
        let mut executor = CommandExecutor::new();

        // Register multiple traps in random order
        execute(&mut executor, &["handler_term".to_string(), "TERM".to_string()]).unwrap();
        execute(&mut executor, &["handler_hup".to_string(), "HUP".to_string()]).unwrap();
        execute(&mut executor, &["handler_int".to_string(), "INT".to_string()]).unwrap();

        let registry = executor.trap_registry();
        let list = registry.list();

        // Should have 3 entries, sorted by signal number
        assert_eq!(list.len(), 3);

        // SIGHUP=1, SIGINT=2, SIGTERM=15 (sorted order)
        assert_eq!(list[0].0, Signal::SIGHUP);
        assert_eq!(list[0].1, "handler_hup");

        assert_eq!(list[1].0, Signal::SIGINT);
        assert_eq!(list[1].1, "handler_int");

        assert_eq!(list[2].0, Signal::SIGTERM);
        assert_eq!(list[2].1, "handler_term");
    }

    // T037: Unit test for EXIT trap inclusion in listing
    #[test]
    fn test_trap_list_includes_exit() {
        let mut executor = CommandExecutor::new();

        // Register both regular signals and EXIT
        execute(&mut executor, &["int_handler".to_string(), "INT".to_string()]).unwrap();
        execute(&mut executor, &["exit_handler".to_string(), "EXIT".to_string()]).unwrap();
        execute(&mut executor, &["term_handler".to_string(), "TERM".to_string()]).unwrap();

        let registry = executor.trap_registry();

        // Regular signals should be in sorted list
        let list = registry.list();
        assert_eq!(list.len(), 2); // INT and TERM

        // EXIT should be separate
        assert_eq!(registry.get_exit().unwrap(), "exit_handler");
    }

    #[test]
    fn test_trap_list_exit_only() {
        let mut executor = CommandExecutor::new();

        // Register only EXIT trap
        execute(&mut executor, &["cleanup".to_string(), "EXIT".to_string()]).unwrap();

        let registry = executor.trap_registry();

        // No regular signals
        assert_eq!(registry.list().len(), 0);

        // But EXIT is present
        assert_eq!(registry.get_exit().unwrap(), "cleanup");
    }

    // ===== Phase 5: User Story 3 - Clear Trap Tests =====

    // T045: Unit test for clearing non-existent trap (idempotent)
    #[test]
    fn test_trap_clear_nonexistent_idempotent() {
        let mut executor = CommandExecutor::new();

        // Clear a trap that doesn't exist - should succeed silently
        let result = execute(&mut executor, &["".to_string(), "INT".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Still no trap registered
        assert!(!executor.trap_registry().has_handler(Signal::SIGINT));

        // Clearing again should also succeed (idempotent)
        let result2 = execute(&mut executor, &["".to_string(), "INT".to_string()]);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 0);
    }

    #[test]
    fn test_trap_clear_exit_nonexistent() {
        let mut executor = CommandExecutor::new();

        // Clear EXIT when it doesn't exist - should succeed silently
        let result = execute(&mut executor, &["".to_string(), "EXIT".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Still no EXIT handler
        assert!(executor.trap_registry().get_exit().is_none());
    }

    #[test]
    fn test_trap_reregister_after_clear() {
        let mut executor = CommandExecutor::new();

        // Register, clear, then re-register same signal
        execute(&mut executor, &["first".to_string(), "INT".to_string()]).unwrap();
        execute(&mut executor, &["".to_string(), "INT".to_string()]).unwrap();

        // Re-registration should succeed (no longer duplicate)
        let result = execute(&mut executor, &["second".to_string(), "INT".to_string()]);
        assert!(result.is_ok());
        assert_eq!(executor.trap_registry().get(Signal::SIGINT).unwrap(), "second");
    }
}
