//! Git operations module

pub mod commit;
pub mod security;
pub mod worktree;

pub use commit::{
    commit_group, generate_commit_message, intelligent_commit, interactive_commit, CommitGroup,
    CommitResult,
};
pub use security::{
    scan_all_changes, scan_staged_changes, SecurityScanResult, SecurityWarning, SensitiveFile,
    Severity,
};
pub use worktree::{FeatureInfo, WorktreeInfo};
