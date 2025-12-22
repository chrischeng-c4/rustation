//! CLI argument parsing for rstn

use clap::{Parser, Subcommand};

use crate::version;

/// rstn - Rustation development toolkit
#[derive(Parser, Debug)]
#[command(name = "rstn")]
#[command(version = version::FULL_VERSION)]
#[command(about = "Rustation development toolkit", long_about = None)]
#[command(author)]
pub struct Cli {
    /// Increase logging verbosity
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run tests with enhanced output and filtering
    Test {
        /// Test name filter pattern
        filter: Option<String>,

        /// Run only library tests
        #[arg(short, long)]
        lib: bool,

        /// Run only integration tests
        #[arg(short, long)]
        integration: bool,

        /// Run tests in watch mode (rerun on file changes)
        #[arg(short, long)]
        watch: bool,

        /// Show all test output (verbose mode)
        #[arg(short = 'v', long)]
        test_verbose: bool,
    },

    /// Build the project
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },

    /// Fast compilation check (no codegen)
    Check,

    /// Run clippy lints
    Lint {
        /// Auto-fix issues
        #[arg(long)]
        fix: bool,
    },

    /// Format code with rustfmt
    Fmt {
        /// Check formatting without making changes
        #[arg(long)]
        check: bool,
    },

    /// Run all CI checks (test + lint + fmt + build)
    Ci {
        /// Auto-fix lint and format issues
        #[arg(long)]
        fix: bool,
    },

    /// Spec-driven development workflow
    Spec {
        #[command(subcommand)]
        command: SpecCommands,
    },

    /// Run health check and print diagnostics
    Doctor,

    /// Git worktree management
    Worktree {
        #[command(subcommand)]
        command: WorktreeCommands,
    },

    /// MCP (Model Context Protocol) configuration
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },

    /// Development service management
    Service {
        #[command(subcommand)]
        command: ServiceCommands,
    },

    /// Session management (query and inspect rstn sessions)
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Send a prompt directly to Claude and stream the response
    Prompt {
        /// The prompt message to send to Claude
        message: String,

        /// Maximum number of agentic turns (default: 1)
        #[arg(long, default_value = "1")]
        max_turns: u32,

        /// Skip permission prompts for automation
        #[arg(long)]
        skip_permissions: bool,

        /// Continue from a previous session
        #[arg(long)]
        continue_session: bool,

        /// Specific session ID to resume (requires --continue-session)
        #[arg(long, requires = "continue_session")]
        session_id: Option<String>,

        /// Allowed tools (comma-separated, empty = all)
        #[arg(long, value_delimiter = ',')]
        allowed_tools: Vec<String>,

        /// Additional files for context (comma-separated paths)
        #[arg(long, value_delimiter = ',')]
        context: Vec<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorktreeCommands {
    /// List all worktrees
    List {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show current worktree status
    Status,

    /// Create new worktree for feature
    Create {
        /// Feature number or name (e.g., "042" or "042-worktree")
        feature: String,

        /// Base path for worktree (default: parent of current repo)
        #[arg(short, long)]
        path: Option<std::path::PathBuf>,
    },

    /// Remove a worktree
    Remove {
        /// Path to worktree to remove
        path: String,

        /// Force removal even with uncommitted changes
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// List available MCP servers
    List {
        /// Show verbose output with args
        #[arg(short, long)]
        verbose: bool,
    },

    /// Generate .mcp.json configuration
    Generate {
        /// Component name to filter servers (optional)
        component: Option<String>,

        /// Output path (default: .mcp.json)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Validate MCP configuration
    Validate,

    /// Show detailed info about an MCP server
    Info {
        /// Server name
        server: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ServiceCommands {
    /// List all running development services
    List,

    /// Show status of a specific service
    Status {
        /// Service name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// List recent sessions
    List {
        /// Maximum number of sessions to display
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Filter by command type (e.g., "prompt")
        #[arg(long)]
        filter_type: Option<String>,

        /// Filter by status (e.g., "completed", "failed")
        #[arg(long)]
        filter_status: Option<String>,
    },

    /// Show detailed session information
    Info {
        /// Session ID or prefix (e.g., "abc123")
        session_id: String,
    },

    /// Display session log file
    Logs {
        /// Session ID or prefix
        session_id: String,

        /// Show last N lines
        #[arg(long)]
        tail: Option<usize>,

        /// Show first N lines
        #[arg(long)]
        head: Option<usize>,

        /// Follow log file (tail -f mode)
        #[arg(short, long)]
        follow: bool,
    },

    /// Clean up old sessions
    Cleanup {
        /// Delete sessions older than N days
        #[arg(long, default_value = "30")]
        older_than_days: u32,

        /// Preview changes without deleting
        #[arg(long)]
        dry_run: bool,

        /// Delete even active sessions
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum SpecCommands {
    /// Show feature status
    Status {
        /// Feature number (auto-detects from branch if not specified)
        feature: Option<String>,
    },

    /// List all features
    List,

    /// Create new feature
    New {
        /// Feature description
        description: String,

        /// Short name for feature
        #[arg(long)]
        name: Option<String>,
    },

    /// Run workflow phase
    Run {
        /// Workflow phase (specify, clarify, plan, tasks, implement, etc.)
        phase: String,

        /// Feature number (auto-detects from branch if not specified)
        feature: Option<String>,
    },
}

impl Cli {
    /// Check if verbose logging is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Check if quiet mode is enabled
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test basic parsing works
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
