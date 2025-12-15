//! Git commit workflow with security and AI-powered messages

use super::security::{scan_all_changes, scan_staged_changes, SecurityScanResult, SecurityWarning, SensitiveFile};
use crate::errors::Result;
use tokio::process::Command;

/// A logical group of related file changes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommitGroup {
    pub files: Vec<String>,
    pub message: String,
    pub description: String,
    pub category: Option<String>,
}

/// Result of the interactive commit workflow
#[derive(Debug, Clone)]
pub enum CommitResult {
    /// Commit blocked due to critical security issues
    Blocked(SecurityScanResult),
    /// Ready to commit with generated message and warnings
    ReadyToCommit {
        message: String,
        warnings: Vec<SecurityWarning>,
        sensitive_files: Vec<SensitiveFile>,
    },
    /// Multiple commit groups ready for review
    GroupedCommits {
        groups: Vec<CommitGroup>,
        warnings: Vec<SecurityWarning>,
        sensitive_files: Vec<SensitiveFile>,
    },
}

/// Generate a conventional commit message using Claude
pub async fn generate_commit_message() -> Result<String> {
    // Get diff summary
    let diff_stat_output = Command::new("git")
        .args(&["diff", "--cached", "--stat"])
        .output()
        .await?;

    if !diff_stat_output.status.success() {
        return Err(crate::errors::CoreError::Git(
            "Failed to get diff stats".to_string(),
        ));
    }

    let diff_stat = String::from_utf8_lossy(&diff_stat_output.stdout);

    // Check if there are any changes
    if diff_stat.trim().is_empty() {
        return Err(crate::errors::CoreError::Git(
            "No staged changes to commit".to_string(),
        ));
    }

    // Get recent commit messages for style reference
    let log_output = Command::new("git")
        .args(&["log", "--oneline", "-5", "--format=%s"])
        .output()
        .await?;

    let recent_commits = if log_output.status.success() {
        String::from_utf8_lossy(&log_output.stdout).to_string()
    } else {
        // No git history yet (new repo)
        String::from("feat: initial commit")
    };

    // Build prompt for Claude
    let prompt = format!(
        r#"Generate a conventional commit message for these staged changes.

STAGED CHANGES:
{}

RECENT COMMIT STYLE:
{}

RULES:
- Return ONLY the commit message (no explanation or markdown)
- Use conventional commits format: <type>(<scope>): <subject>
- Types: feat, fix, docs, style, refactor, test, chore, perf, ci
- Keep subject under 72 characters
- Match the style of recent commits shown above
- Be specific about what changed

Example format:
feat(rscli): add git commit security scanning

Return only the commit message:"#,
        diff_stat, recent_commits
    );

    // Call Claude CLI directly for simple text generation
    call_claude_for_message(&prompt).await
}

/// Call Claude CLI for simple text generation
async fn call_claude_for_message(prompt: &str) -> Result<String> {
    // Use claude CLI in simple mode (not streaming)
    let output = Command::new("claude")
        .arg("-p")
        .arg(prompt)
        .arg("--output-format")
        .arg("text")
        .output()
        .await
        .map_err(|e| {
            crate::errors::CoreError::CommandFailed(format!("Failed to run Claude CLI: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::errors::CoreError::CommandFailed(format!(
            "Claude CLI failed: {}",
            stderr
        )));
    }

    let response = String::from_utf8_lossy(&output.stdout);

    // Clean up the response
    let message = response
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !line.starts_with("```")) // Remove markdown code blocks
        .filter(|line| !line.contains("commit message:")) // Remove prompt echoes
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n");

    // Take first line as the commit message (subject line)
    let first_line = message
        .lines()
        .next()
        .unwrap_or("chore: update files")
        .to_string();

    // Validate it's not too long
    if first_line.len() > 72 {
        // Truncate at 72 chars
        Ok(first_line.chars().take(69).collect::<String>() + "...")
    } else {
        Ok(first_line)
    }
}

/// Run the complete interactive commit workflow
///
/// This function:
/// 1. Scans staged changes for security issues
/// 2. Blocks if critical issues found
/// 3. Generates commit message using Claude
/// 4. Returns result for user review
pub async fn interactive_commit() -> Result<CommitResult> {
    // 1. Security scan
    let scan = scan_staged_changes().await?;

    if scan.blocked {
        return Ok(CommitResult::Blocked(scan));
    }

    // 2. Generate message
    let message = generate_commit_message().await?;

    // 3. Return for user review
    Ok(CommitResult::ReadyToCommit {
        message,
        warnings: scan.warnings,
        sensitive_files: scan.sensitive_files,
    })
}

/// Run intelligent commit workflow with AI-powered grouping
///
/// This function:
/// 1. Scans ALL changes (staged, unstaged, untracked) for security issues
/// 2. Blocks if critical issues found
/// 3. Calls Claude Code to analyze and group staged changes
/// 4. Returns all groups for user review
pub async fn intelligent_commit() -> Result<CommitResult> {
    // 1. Security scan ALL changes (staged, unstaged, untracked)
    let scan = scan_all_changes().await?;
    if scan.blocked {
        return Ok(CommitResult::Blocked(scan));
    }

    // 2. Get staged files and diff stats
    let files_output = Command::new("git")
        .args(&["diff", "--cached", "--name-status"])
        .output()
        .await?;

    if !files_output.status.success() {
        return Err(crate::errors::CoreError::Git(
            "Failed to get staged files".to_string(),
        ));
    }

    let diff_stat_output = Command::new("git")
        .args(&["diff", "--cached", "--stat"])
        .output()
        .await?;

    if !diff_stat_output.status.success() {
        return Err(crate::errors::CoreError::Git(
            "Failed to get diff stats".to_string(),
        ));
    }

    let mut files = String::from_utf8_lossy(&files_output.stdout).to_string();
    let mut diff_stat = String::from_utf8_lossy(&diff_stat_output.stdout).to_string();

    // Check if there are any staged changes - if not, auto-stage everything
    if files.trim().is_empty() {
        // Try to stage all changes
        let add_output = Command::new("git")
            .args(&["add", "--all"])
            .output()
            .await?;

        if !add_output.status.success() {
            return Err(crate::errors::CoreError::Git(
                "No changes to commit and failed to auto-stage".to_string(),
            ));
        }

        // Re-check staged files
        let files_output = Command::new("git")
            .args(&["diff", "--cached", "--name-status"])
            .output()
            .await?;

        let diff_stat_output = Command::new("git")
            .args(&["diff", "--cached", "--stat"])
            .output()
            .await?;

        files = String::from_utf8_lossy(&files_output.stdout).to_string();
        diff_stat = String::from_utf8_lossy(&diff_stat_output.stdout).to_string();

        // If still empty, nothing to commit
        if files.trim().is_empty() {
            return Err(crate::errors::CoreError::Git(
                "No changes to commit".to_string(),
            ));
        }
    }

    // 3. Build prompt for Claude Code
    let prompt = format!(
        r#"Analyze staged changes and group them into logical commits.

STAGED FILES:
{}

DIFF SUMMARY:
{}

INSTRUCTIONS:
1. Group related changes (separate refactoring from features)
2. Generate conventional commit message for each group
3. Format: <type>(<scope>): <subject>
4. Types: feat, fix, docs, style, refactor, test, chore
5. Keep subjects under 72 characters

Return ONLY a JSON array:
[
  {{
    "files": ["path/to/file.rs"],
    "message": "feat(core): add intelligent commit",
    "description": "New feature: multi-group commits",
    "category": "feat"
  }}
]"#,
        files, diff_stat
    );

    // 4. Call Claude Code CLI
    let output = Command::new("claude")
        .arg("-p")
        .arg(prompt)
        .arg("--output-format")
        .arg("text")
        .arg("--max-turns")
        .arg("1")
        .output()
        .await
        .map_err(|e| crate::errors::CoreError::CommandFailed(
            format!("Claude CLI not available: {}", e)
        ))?;

    if !output.status.success() {
        return Err(crate::errors::CoreError::CommandFailed(
            "Claude failed to analyze changes".to_string()
        ));
    }

    // 5. Parse JSON response
    let response = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json_from_response(&response)?;
    let groups: Vec<CommitGroup> = serde_json::from_str(&json_str)
        .map_err(|e| crate::errors::CoreError::Git(
            format!("Failed to parse commit groups: {}", e)
        ))?;

    if groups.is_empty() {
        return Err(crate::errors::CoreError::Git("No commit groups generated".to_string()));
    }

    Ok(CommitResult::GroupedCommits {
        groups,
        warnings: scan.warnings,
        sensitive_files: scan.sensitive_files,
    })
}

/// Extract JSON array from Claude's response
fn extract_json_from_response(response: &str) -> Result<String> {
    let trimmed = response.trim();
    if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            return Ok(trimmed[start..=end].to_string());
        }
    }
    Err(crate::errors::CoreError::Git("No JSON array in response".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_truncation() {
        let long_msg = "a".repeat(80);
        let truncated = if long_msg.len() > 72 {
            long_msg.chars().take(69).collect::<String>() + "..."
        } else {
            long_msg
        };
        assert_eq!(truncated.len(), 72);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_commit_result_variants() {
        use crate::git::security::{SecurityScanResult, SecurityWarning, Severity};

        let blocked = CommitResult::Blocked(SecurityScanResult {
            blocked: true,
            warnings: vec![SecurityWarning {
                file_path: "test.rs".to_string(),
                line_number: 10,
                pattern_matched: "private key".to_string(),
                severity: Severity::Critical,
                message: "Private key detected".to_string(),
            }],
            sensitive_files: vec![],
        });

        match blocked {
            CommitResult::Blocked(_) => {} // Expected
            _ => panic!("Expected Blocked variant"),
        }
    }
}
