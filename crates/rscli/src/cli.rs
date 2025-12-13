//! CLI argument parsing for rscli

use clap::{Parser, Subcommand};

/// rscli - Rust Station development toolkit
#[derive(Parser, Debug)]
#[command(name = "rscli")]
#[command(version)]
#[command(about = "Rust Station development toolkit", long_about = None)]
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
