//! Main entry point for rscli binary
//!
//! Launches TUI mode by default. Use --cli flag for classic CLI mode.

use clap::Parser;
use rscli::cli::{Commands, McpCommands, ServiceCommands, SpecCommands, WorktreeCommands};
use rscli::tui::App;
use rscli::version;
use rscli::{commands, Result};

macro_rules! debug {
    ($($arg:tt)*) => {
        if std::env::var("RSCLI_DEBUG").is_ok() {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

macro_rules! log_to_file {
    ($($arg:tt)*) => {{
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/rscli.log")
        {
            let _ = writeln!(file, "{}", format!($($arg)*));
            let _ = file.flush();
        }
    }};
}

#[derive(Parser, Debug)]
#[command(name = "rscli")]
#[command(version = version::FULL_VERSION)]
#[command(about = "Rust Station development toolkit - TUI mode by default")]
struct Args {
    /// Run in classic CLI mode instead of TUI
    #[arg(long)]
    cli: bool,

    /// The CLI command to run (only used with --cli)
    #[command(subcommand)]
    command: Option<Commands>,

    /// Increase logging verbosity
    #[arg(short, long)]
    verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    debug!("rscli starting");
    let args = Args::parse();
    debug!("Args: cli={}, command={:?}", args.cli, args.command.is_some());

    // If --cli flag is provided OR a command is specified, run in CLI mode
    if args.cli || args.command.is_some() {
        debug!("Running in CLI mode");
        run_cli_mode(args).await
    } else {
        // Default: run TUI mode
        debug!("Running in TUI mode");
        run_tui_mode()
    }
}

fn run_tui_mode() -> Result<()> {
    log_to_file!("=== rscli starting TUI mode ===");
    debug!("Starting TUI mode");

    // Check if we have a TTY
    log_to_file!("TTY check...");
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        log_to_file!("TTY check FAILED");
        eprintln!("ERROR: TUI mode requires a terminal. Stdout is not a TTY.");
        eprintln!("Use --cli flag for non-interactive mode: rscli --cli <command>");
        debug!("TTY check failed: stdout is not a terminal");
        return Err(rscli::RscliError::Other(anyhow::anyhow!("No TTY available")));
    }
    log_to_file!("TTY check passed");

    log_to_file!("Creating App instance...");
    debug!("Creating App instance");
    let mut app = App::new();
    log_to_file!("App created");

    log_to_file!("Calling app.run()...");
    debug!("Running app main loop");
    let result = app.run();
    log_to_file!("app.run() returned: {:?}", result.as_ref().map(|_| "Ok"));

    debug!("App finished with result: {:?}", result.as_ref().map(|_| "Ok").map_err(|e| e.to_string()));
    result.map_err(|e| rscli::RscliError::Other(anyhow::anyhow!("{}", e)))
}

async fn run_cli_mode(args: Args) -> Result<()> {
    let Some(command) = args.command else {
        // No command provided with --cli, show help
        eprintln!("No command provided. Use --help for usage.");
        return Ok(());
    };

    match command {
        Commands::Test {
            filter,
            lib,
            integration,
            watch,
            test_verbose,
        } => {
            commands::test::run(
                filter.as_deref(),
                lib,
                integration,
                watch,
                test_verbose,
                args.verbose,
            )
            .await?;
        }

        Commands::Build { release } => {
            commands::build::run(release, args.verbose).await?;
        }

        Commands::Check => {
            commands::build::check(args.verbose).await?;
        }

        Commands::Lint { fix } => {
            commands::build::lint(fix, args.verbose).await?;
        }

        Commands::Fmt { check } => {
            commands::build::fmt(check, args.verbose).await?;
        }

        Commands::Ci { fix } => {
            commands::build::ci(fix, args.verbose).await?;
        }

        Commands::Spec { command } => match command {
            SpecCommands::Status { feature } => {
                commands::spec::status(feature, args.verbose).await?;
            }
            SpecCommands::List => {
                commands::spec::list(args.verbose).await?;
            }
            SpecCommands::New { description, name } => {
                commands::spec::create(&description, name.as_deref(), args.verbose).await?;
            }
            SpecCommands::Run { phase, feature } => {
                commands::spec::run(&phase, feature, args.verbose).await?;
            }
        },

        Commands::Doctor => {
            commands::doctor::run(args.verbose).await?;
        }

        Commands::Worktree { command } => match command {
            WorktreeCommands::List { verbose } => {
                commands::worktree::list(verbose).await?;
            }
            WorktreeCommands::Status => {
                commands::worktree::status().await?;
            }
            WorktreeCommands::Create { feature, path } => {
                commands::worktree::create(feature, path).await?;
            }
            WorktreeCommands::Remove { path, force } => {
                commands::worktree::remove(path, force).await?;
            }
        },

        Commands::Mcp { command } => match command {
            McpCommands::List { verbose } => {
                commands::mcp::list(verbose).await?;
            }
            McpCommands::Generate { component, output } => {
                commands::mcp::generate(component, output).await?;
            }
            McpCommands::Validate => {
                commands::mcp::validate().await?;
            }
            McpCommands::Info { server } => {
                commands::mcp::info(server).await?;
            }
        },

        Commands::Service { command } => match command {
            ServiceCommands::List => {
                commands::service::list().await?;
            }
            ServiceCommands::Status { name } => {
                commands::service::status(name).await?;
            }
        },
    }

    Ok(())
}
