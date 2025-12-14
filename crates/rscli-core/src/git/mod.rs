//! Git operations module

pub mod commit;
pub mod security;
pub mod worktree;

pub use commit::{generate_commit_message, interactive_commit, CommitResult};
pub use security::{scan_staged_changes, SecurityScanResult, SecurityWarning, SensitiveFile, Severity};
pub use worktree::{FeatureInfo, WorktreeInfo};
