//! Git worktree management

use crate::errors::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Information about a git worktree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub commit: String,
    pub is_bare: bool,
    pub is_detached: bool,
    pub locked: Option<String>,
}

/// Parsed feature information from branch name
#[derive(Debug, Clone)]
pub struct FeatureInfo {
    pub number: String,            // e.g., "042"
    pub name: String,              // e.g., "worktree-management"
    pub component: Option<String>, // e.g., "rscli"
}

/// List all worktrees
pub async fn list_worktrees() -> Result<Vec<WorktreeInfo>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CoreError::Git(format!("Failed to execute git worktree list: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CoreError::Git(format!(
            "git worktree list failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_worktree_list(&stdout)
}

/// Parse git worktree list --porcelain output
fn parse_worktree_list(output: &str) -> Result<Vec<WorktreeInfo>> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_commit = String::new();
    let mut current_branch: Option<String> = None;
    let mut is_bare = false;
    let mut is_detached = false;
    let mut locked: Option<String> = None;

    for line in output.lines() {
        if line.is_empty() {
            // End of current worktree entry
            if let Some(path) = current_path.take() {
                worktrees.push(WorktreeInfo {
                    path,
                    branch: current_branch.take(),
                    commit: current_commit.clone(),
                    is_bare,
                    is_detached,
                    locked: locked.take(),
                });
                current_commit.clear();
                is_bare = false;
                is_detached = false;
            }
            continue;
        }

        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(commit) = line.strip_prefix("HEAD ") {
            current_commit = commit.to_string();
        } else if let Some(branch) = line.strip_prefix("branch ") {
            // Branch format: "refs/heads/main" or "refs/heads/feature/042-worktree"
            if let Some(branch_name) = branch.strip_prefix("refs/heads/") {
                current_branch = Some(branch_name.to_string());
            }
        } else if line == "bare" {
            is_bare = true;
        } else if line == "detached" {
            is_detached = true;
        } else if let Some(reason) = line.strip_prefix("locked ") {
            locked = Some(reason.to_string());
        }
    }

    // Handle last entry if output doesn't end with blank line
    if let Some(path) = current_path {
        worktrees.push(WorktreeInfo {
            path,
            branch: current_branch,
            commit: current_commit,
            is_bare,
            is_detached,
            locked,
        });
    }

    Ok(worktrees)
}

/// Parse feature information from branch name
/// Supports formats:
/// - "042-worktree-management" → { number: "042", name: "worktree-management", component: None }
/// - "feature/042-worktree" → { number: "042", name: "worktree", component: None }
/// - "rscli/042-worktree" → { number: "042", name: "worktree", component: Some("rscli") }
pub fn parse_feature_branch(branch: &str) -> Option<FeatureInfo> {
    // Remove "feature/" prefix if present
    let branch = branch.strip_prefix("feature/").unwrap_or(branch);

    // Check for component/feature format
    if let Some(slash_pos) = branch.find('/') {
        let component = &branch[..slash_pos];
        let feature_part = &branch[slash_pos + 1..];

        // Parse feature number and name from the part after slash
        if let Some((number, name)) = parse_feature_number_and_name(feature_part) {
            return Some(FeatureInfo {
                number: number.to_string(),
                name: name.to_string(),
                component: Some(component.to_string()),
            });
        }
    }

    // Try direct parsing without component
    if let Some((number, name)) = parse_feature_number_and_name(branch) {
        return Some(FeatureInfo {
            number: number.to_string(),
            name: name.to_string(),
            component: None,
        });
    }

    None
}

fn parse_feature_number_and_name(s: &str) -> Option<(&str, &str)> {
    // Look for pattern: NNN-name or NNN_name
    if s.len() < 4 {
        return None;
    }

    // Check if starts with 3 digits
    if !s.chars().take(3).all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Check for separator
    if let Some(c) = s.chars().nth(3) {
        if c == '-' || c == '_' {
            let number = &s[..3];
            let name = &s[4..];
            if !name.is_empty() {
                return Some((number, name));
            }
        }
    }

    None
}

/// Get current worktree path
pub async fn get_current_worktree() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CoreError::Git(format!("Failed to get current worktree: {}", e)))?;

    if !output.status.success() {
        return Err(CoreError::RepoNotFound);
    }

    let path = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(path.trim()))
}

/// Get current branch name
pub async fn get_current_branch() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CoreError::Git(format!("Failed to get current branch: {}", e)))?;

    if !output.status.success() {
        return Ok(None);
    }

    let branch = String::from_utf8_lossy(&output.stdout);
    let branch = branch.trim();

    if branch == "HEAD" {
        // Detached HEAD state
        Ok(None)
    } else {
        Ok(Some(branch.to_string()))
    }
}

/// Create a new worktree
pub async fn create_worktree(feature: &str, base_path: Option<PathBuf>) -> Result<PathBuf> {
    let branch_name = format!("feature/{}", feature);

    // Determine worktree path
    let worktree_path = if let Some(base) = base_path {
        base.join(feature)
    } else {
        // Use parent of current repo + feature name
        let current = get_current_worktree().await?;
        current
            .parent()
            .ok_or_else(|| CoreError::Git("Cannot determine parent directory".into()))?
            .join(feature)
    };

    // Create worktree
    let output = Command::new("git")
        .args(["worktree", "add", "-b", &branch_name])
        .arg(&worktree_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CoreError::Git(format!("Failed to create worktree: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CoreError::Git(format!(
            "Failed to create worktree: {}",
            stderr
        )));
    }

    Ok(worktree_path)
}

/// Remove a worktree
pub async fn remove_worktree(path: &str, force: bool) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(["worktree", "remove"]);

    if force {
        cmd.arg("--force");
    }

    cmd.arg(path);

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CoreError::Git(format!("Failed to remove worktree: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CoreError::Git(format!(
            "Failed to remove worktree: {}",
            stderr
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_feature_branch() {
        // Test direct format
        let info = parse_feature_branch("042-worktree-management").unwrap();
        assert_eq!(info.number, "042");
        assert_eq!(info.name, "worktree-management");
        assert_eq!(info.component, None);

        // Test with feature/ prefix
        let info = parse_feature_branch("feature/042-worktree").unwrap();
        assert_eq!(info.number, "042");
        assert_eq!(info.name, "worktree");

        // Test with component
        let info = parse_feature_branch("rscli/042-worktree").unwrap();
        assert_eq!(info.number, "042");
        assert_eq!(info.name, "worktree");
        assert_eq!(info.component, Some("rscli".to_string()));

        // Test with component and feature/ prefix
        let info = parse_feature_branch("feature/rscli/042-test").unwrap();
        assert_eq!(info.number, "042");
        assert_eq!(info.name, "test");
        assert_eq!(info.component, Some("rscli".to_string()));

        // Test invalid formats
        assert!(parse_feature_branch("main").is_none());
        assert!(parse_feature_branch("feature-branch").is_none());
        assert!(parse_feature_branch("42-short").is_none()); // Need 3 digits
    }

    #[test]
    fn test_parse_worktree_list() {
        let output = r#"worktree /path/to/main
HEAD 1234567890abcdef
branch refs/heads/main

worktree /path/to/feature-042
HEAD abcdefg1234567
branch refs/heads/feature/042-worktree-management

"#;

        let worktrees = parse_worktree_list(output).unwrap();
        assert_eq!(worktrees.len(), 2);

        assert_eq!(worktrees[0].path, PathBuf::from("/path/to/main"));
        assert_eq!(worktrees[0].branch, Some("main".to_string()));
        assert_eq!(worktrees[0].commit, "1234567890abcdef");
        assert!(!worktrees[0].is_bare);

        assert_eq!(worktrees[1].path, PathBuf::from("/path/to/feature-042"));
        assert_eq!(
            worktrees[1].branch,
            Some("feature/042-worktree-management".to_string())
        );
        assert_eq!(worktrees[1].commit, "abcdefg1234567");
    }
}
