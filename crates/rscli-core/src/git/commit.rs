//! Git commit workflow with security and AI-powered messages

use super::security::{scan_staged_changes, SecurityScanResult, SecurityWarning, SensitiveFile};
use crate::errors::Result;
use tokio::process::Command;

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
