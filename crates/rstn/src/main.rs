//! Main entry point for rstn binary
//!
//! Launches TUI mode by default. Use --cli flag for classic CLI mode.

use clap::Parser;
use rstn::cli::{Commands, McpCommands, ServiceCommands, SpecCommands, WorktreeCommands};
use rstn::settings::Settings;
use rstn::tui::App;
use rstn::version;
use rstn::{commands, logging, Result};
use tracing::{debug, info};

#[derive(Parser, Debug)]
#[command(name = "rstn")]
#[command(version = version::FULL_VERSION)]
#[command(about = "Rustation development toolkit - TUI mode by default")]
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
    use std::time::Instant;

    // Track session start time for duration calculation
    let session_start = Instant::now();

    // Load settings and initialize logging
    let settings = Settings::load();
    let session_id = logging::init(&settings);

    info!(
        session_id = %session_id,
        version = version::FULL_VERSION,
        "📍 Session started"
    );

    let args = Args::parse();
    debug!(cli = args.cli, command = ?args.command, "parsed arguments");

    // If --cli flag is provided OR a command is specified, run in CLI mode
    let result = if args.cli || args.command.is_some() {
        debug!("running in CLI mode");
        run_cli_mode(args).await
    } else {
        // Default: run TUI mode
        debug!("running in TUI mode");
        run_tui_mode(session_id).await
    };

    // Log session end with duration
    let duration = session_start.elapsed();
    match &result {
        Ok(_) => info!(
            duration_secs = duration.as_secs_f64(),
            "✅ Session ended successfully ({}s)",
            duration.as_secs()
        ),
        Err(e) => tracing::error!(
            error = %e,
            duration_secs = duration.as_secs_f64(),
            "❌ Session ended with error ({}s): {}",
            duration.as_secs(),
            e
        ),
    }

    result
}

async fn run_tui_mode(session_id: String) -> Result<()> {
    use rstn::tui::mcp_server::{self, McpServerConfig, McpState};
    use std::sync::Arc;
    use tokio::sync::{mpsc, Mutex};

    info!("starting TUI mode");

    // Check if we have a TTY
    debug!("checking TTY availability");
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        tracing::error!("TTY check failed: stdout is not a terminal");
        eprintln!("ERROR: TUI mode requires a terminal. Stdout is not a TTY.");
        eprintln!("Use --cli flag for non-interactive mode: rstn --cli <command>");
        return Err(rstn::RscliError::Other(anyhow::anyhow!("No TTY available")));
    }
    debug!("TTY check passed");

    // Create shared MCP state for metrics tracking
    let mcp_state = Arc::new(Mutex::new(McpState::default()));

    // Start MCP server for Claude Code communication
    debug!("starting MCP server");
    let (mcp_event_tx, _mcp_event_rx) = mpsc::channel(100);
    let mcp_config = McpServerConfig::default();
    let mcp_handle = match mcp_server::start_server(mcp_config, mcp_event_tx, mcp_state.clone()).await {
        Ok(handle) => {
            info!("MCP server started on {}", handle.url());
            // Write MCP config for Claude Code discovery
            if let Err(e) = mcp_server::write_mcp_config(handle.port()) {
                tracing::warn!("Failed to write MCP config: {}", e);
            }
            Some(handle)
        }
        Err(e) => {
            tracing::warn!("Failed to start MCP server (continuing without it): {}", e);
            None
        }
    };

    debug!("creating App instance with session_id: {}", session_id);
    let mut app = App::new_with_session(mcp_state.clone(), Some(session_id));
    debug!("App created successfully");

    debug!("running app main loop");
    let result = app.run();

    // Cleanup MCP server
    if let Some(handle) = mcp_handle {
        debug!("shutting down MCP server");
        handle.shutdown().await;
        if let Err(e) = mcp_server::cleanup_mcp_config() {
            tracing::warn!("Failed to cleanup MCP config: {}", e);
        }
        debug!("MCP server shutdown complete");
    }

    result.map_err(|e| rstn::RscliError::Other(anyhow::anyhow!("{}", e)))
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
