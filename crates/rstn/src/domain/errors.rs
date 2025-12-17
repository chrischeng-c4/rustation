//! Error types for rstn-core
//!
//! Uses thiserror for structured, type-safe error handling.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    // Build and test errors
    #[error("Build failed: {0}")]
    BuildFailed(String),

    #[error("Test execution failed: {0}")]
    TestFailed(String),

    #[error("Cargo command failed: {0}")]
    CargoFailed(String),

    // Git errors
    #[error("Git operation failed: {0}")]
    Git(String),

    #[error("Repository root not found")]
    RepoNotFound,

    #[error("Could not detect feature from branch")]
    FeatureDetectionFailed,

    #[error("Worktree not found: {0}")]
    WorktreeNotFound(String),

    // Service errors
    #[error("Service '{0}' not found")]
    ServiceNotFound(String),

    #[error("Service '{0}' already running")]
    ServiceAlreadyRunning(String),

    #[error("Service '{0}' failed to start: {1}")]
    ServiceStartFailed(String, String),

    #[error("Service '{0}' is not running")]
    ServiceNotRunning(String),

    // Health check errors
    #[error("Health check failed for {0}: {1}")]
    HealthCheckFailed(String, String),

    #[error("Health check timeout for {0}")]
    HealthCheckTimeout(String),

    // Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid configuration: {0}")]
    Validation(String),

    // MCP errors
    #[error("MCP server '{0}' not found in registry")]
    McpServerNotFound(String),

    #[error("MCP configuration generation failed: {0}")]
    McpConfigFailed(String),

    #[error("MCP registry not found at {0}")]
    McpRegistryNotFound(String),

    // Command errors
    #[error("Command '{0}' not found")]
    CommandNotFound(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    // Port errors
    #[error("Port {0} is already in use")]
    PortInUse(u16),

    #[error("Could not find available port in range {0}-{1}")]
    NoAvailablePort(u16, u16),

    // Path errors
    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    // Standard error conversions
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

/// Result type alias for rstn-core operations
pub type Result<T> = std::result::Result<T, CoreError>;
