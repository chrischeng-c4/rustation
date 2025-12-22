//! Library root for rstn
//!
//! This crate contains both the CLI/TUI application code and the core domain logic.

// CLI/TUI modules
pub mod claude_discovery;
pub mod cli;
pub mod commands;
pub mod logging;
pub mod runners;
pub mod session;
pub mod session_manager;
pub mod settings;
pub mod tui;
pub mod ui;
pub mod version;

// Domain logic modules (merged from rstn-core)
pub mod domain {
    pub mod build;
    pub mod clarify;
    pub mod errors;
    pub mod git;
    pub mod mcp;
    pub mod paths;
    pub mod plan;
    pub mod prompts;
    pub mod service;
    pub mod specify;
    pub mod test;
}

use thiserror::Error;

/// Error types for rstn CLI/TUI
#[derive(Error, Debug)]
pub enum RscliError {
    #[error("Cargo command failed: {0}")]
    CargoFailed(String),

    #[error("Test execution failed: {0}")]
    TestFailed(String),

    #[error("Build failed: {0}")]
    BuildFailed(String),

    #[error("Repository root not found. Are you inside the rustation project?")]
    RepoNotFound,

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type alias for rstn CLI/TUI
pub type Result<T> = std::result::Result<T, RscliError>;

// Conversion from CoreError to RscliError
impl From<CoreError> for RscliError {
    fn from(err: CoreError) -> Self {
        RscliError::Other(anyhow::anyhow!("{}", err))
    }
}

// Re-export commonly used domain types
pub use domain::build::CommandOutput;
pub use domain::errors::{CoreError, Result as CoreResult};
pub use domain::git::{
    CommitGroup, CommitResult, FeatureInfo, SecurityScanResult, SecurityWarning, SensitiveFile,
    Severity, WorktreeInfo,
};
pub use domain::mcp::{McpConfig, McpRegistry, McpServer};
pub use domain::prompts::{PromptManager, PromptSource, SpecPhase};
pub use domain::service::{ServiceInfo, ServiceState};
pub use domain::test::TestResults;

// Specify module re-exports
pub use domain::specify::{
    FeatureEntry, FeaturesCatalog, NewFeature, SpecResult, SpecifyConfig, SpecifyError,
};

// Clarify module re-exports
pub use domain::clarify::{
    AnalysisResult, Answer, Category, ClarifyConfig, ClarifyError, ClarifyReport, CoverageMap,
    CoverageStatus, Question, QuestionFormat, QuestionOption, RecommendedAnswer,
};

// Plan module re-exports
pub use domain::plan::{
    ArtifactKind, ArtifactWriter, PlanArtifact, PlanConfig, PlanContext, PlanError, PlanResult,
};

// Session manager re-exports (dual-layer session management)
pub use session_manager::{
    ClaudeSession, ClaudeSessionStatus, RstnSession, SessionManager, SessionRecord, WorkflowType,
};
