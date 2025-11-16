//! rush - A modern, fast, fish-like shell written in Rust
//!
//! rush is designed to provide a delightful shell experience with:
//! - Real-time syntax highlighting
//! - Autosuggestions from history
//! - Smart tab completions
//! - Persistent command history
//! - Job control
//! - Script execution
//!
//! All with zero configuration required.

use clap::Parser;
use rush::{cli::Cli, Config, Repl};
use std::fs;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize logging based on CLI flags
    initialize_logging(&cli);

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "rush shell starting");

    // Handle utility commands that don't start the REPL
    if cli.dump_config {
        dump_configuration(&cli);
        return;
    }

    if cli.doctor {
        run_health_check(&cli);
        return;
    }

    // Load or create configuration
    let config = load_configuration(&cli);

    // Handle single command execution mode
    if let Some(ref cmd) = cli.command {
        execute_command_and_exit(cmd, &config);
        return;
    }

    // Create and run REPL
    let mut repl = match Repl::with_config(config) {
        Ok(repl) => repl,
        Err(err) => {
            eprintln!("rush: failed to initialize: {}", err);
            tracing::error!(error = %err, "REPL initialization failed");
            std::process::exit(1);
        }
    };

    // Run REPL loop
    let exit_code = match repl.run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("rush: error: {}", err);
            tracing::error!(error = %err, "REPL error");
            1
        }
    };

    tracing::info!(exit_code, "rush shell exiting");
    std::process::exit(exit_code);
}

/// Initialize logging based on CLI arguments
fn initialize_logging(cli: &Cli) {
    if cli.quiet {
        // Quiet mode: no logging
        return;
    }

    if cli.is_verbose() {
        // Verbose mode: log to file (and optionally console)
        let log_file_path = cli.get_log_file_path();

        // Create log directory if it doesn't exist
        if let Some(parent) = log_file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Create file appender
        let file_appender = tracing_appender::rolling::never(
            log_file_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new(".")),
            log_file_path.file_name().unwrap_or_default(),
        );

        // Build log level filter
        let log_level = cli.log_level();
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(format!("rush={},info", log_level)));

        // Configure layers based on verbosity level
        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_target(true)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true);

        if cli.verbose >= 2 {
            // -vv: Log to both file AND console
            let console_layer = fmt::layer()
                .with_target(true)
                .with_file(true)
                .with_line_number(true);

            tracing_subscriber::registry()
                .with(filter)
                .with(file_layer)
                .with(console_layer)
                .init();

            println!("rush: logging to {} and console", log_file_path.display());
        } else {
            // -v: Log to file only
            tracing_subscriber::registry()
                .with(filter)
                .with(file_layer)
                .init();

            println!("rush: logging to {}", log_file_path.display());
        }
    }
    // else: Normal mode (no -v) - no logging initialization (silent)
}

/// Load configuration based on CLI arguments
fn load_configuration(cli: &Cli) -> Config {
    if cli.no_config {
        tracing::info!("Using default configuration (--no-config)");
        return Config::default();
    }

    let mut config = if let Some(ref config_path) = cli.config {
        tracing::info!(path = %config_path.display(), "Loading custom config file");
        // TODO: Implement Config::load_from_file()
        // For now, just use defaults
        tracing::warn!("Custom config loading not yet implemented, using defaults");
        Config::default()
    } else {
        Config::load()
    };

    // Apply CLI overrides
    if let Some(history_size) = cli.history_size {
        tracing::info!(history_size, "Overriding history size from CLI");
        config.history_size = history_size;
    }

    config
}

/// Print resolved configuration and exit
fn dump_configuration(cli: &Cli) {
    let config = load_configuration(cli);

    println!("Rush Shell Configuration");
    println!("========================");
    println!();
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Configuration:");
    println!("  history_size: {}", config.history_size);
    println!("  prompt: {:?}", config.prompt);
    println!("  completion_timeout_ms: {}", config.completion_timeout_ms);
    println!("  suggestion_delay_ms: {}", config.suggestion_delay_ms);
    println!();
    println!("CLI Flags:");
    println!("  verbose: {}", cli.verbose);
    println!("  quiet: {}", cli.quiet);
    println!("  no_config: {}", cli.no_config);
    println!("  no_history: {}", cli.no_history);
    println!("  no_highlight: {}", cli.no_highlight);
    println!("  no_suggestions: {}", cli.no_suggestions);
    println!("  no_completion: {}", cli.no_completion);
    println!();
    println!("Paths:");
    println!("  config_file: {}", Config::config_path().display());
    if cli.is_verbose() {
        println!("  log_file: {}", cli.get_log_file_path().display());
    }

    std::process::exit(0);
}

/// Run health check and print diagnostics
fn run_health_check(_cli: &Cli) {
    println!("Rush Shell Health Check");
    println!("=======================");
    println!();

    let mut all_ok = true;

    // Check version
    println!("✓ Version: {}", env!("CARGO_PKG_VERSION"));

    // Check config file
    let config_path = Config::config_path();
    if config_path.exists() {
        println!("✓ Config file exists: {}", config_path.display());
        let config = Config::load();
        println!("  └─ Loaded successfully");
        println!("     history_size: {}", config.history_size);
    } else {
        println!("ℹ Config file not found (using defaults): {}", config_path.display());
    }

    // Check history directory
    let history_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("rush");

    if history_path.exists() {
        println!("✓ History directory exists: {}", history_path.display());
    } else {
        println!("ℹ History directory will be created: {}", history_path.display());
    }

    // Check write permissions
    match fs::create_dir_all(&history_path) {
        Ok(_) => {
            println!("✓ History directory is writable");
        }
        Err(e) => {
            println!("✗ Cannot write to history directory: {}", e);
            all_ok = false;
        }
    }

    println!();
    if all_ok {
        println!("✓ All checks passed!");
        std::process::exit(0);
    } else {
        println!("✗ Some checks failed");
        std::process::exit(1);
    }
}

/// Execute a single command and exit
fn execute_command_and_exit(cmd: &str, _config: &Config) {
    tracing::info!(cmd = %cmd, "Executing single command");

    // Execute command directly without REPL
    use rush::executor::execute::CommandExecutor;
    let executor = CommandExecutor::new();

    match executor.execute(cmd) {
        Ok(exit_code) => {
            tracing::info!(exit_code, "Single command completed");
            std::process::exit(exit_code);
        }
        Err(err) => {
            eprintln!("rush: error: {}", err);
            tracing::error!(error = %err, "Single command execution failed");
            std::process::exit(1);
        }
    }
}
