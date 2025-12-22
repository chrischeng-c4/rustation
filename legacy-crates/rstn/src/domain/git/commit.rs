//! Git commit workflow with security and AI-powered messages

use super::security::{
    scan_all_changes, scan_staged_changes, SecurityScanResult, SecurityWarning, SensitiveFile,
};
use crate::domain::errors::Result;
use tokio::process::Command;

/// A logical group of related file changes
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
        .args(["diff", "--cached", "--stat"])
        .output()
        .await?;

    if !diff_stat_output.status.success() {
        return Err(crate::domain::errors::CoreError::Git(
            "Failed to get diff stats".to_string(),
        ));
    }

    let diff_stat = String::from_utf8_lossy(&diff_stat_output.stdout);

    // Check if there are any changes
    if diff_stat.trim().is_empty() {
        return Err(crate::domain::errors::CoreError::Git(
            "No staged changes to commit".to_string(),
        ));
    }

    // Get recent commit messages for style reference
    let log_output = Command::new("git")
        .args(["log", "--oneline", "-5", "--format=%s"])
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
            crate::domain::errors::CoreError::CommandFailed(format!(
                "Failed to run Claude CLI: {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::domain::errors::CoreError::CommandFailed(format!(
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
    tracing::info!("Starting intelligent commit workflow");

    // 1. Security scan ALL changes (staged, unstaged, untracked)
    tracing::debug!("Running security scan on all changes");
    let scan = scan_all_changes().await?;
    tracing::debug!(
        "Security scan complete: {} warnings, {} sensitive files, blocked={}",
        scan.warnings.len(),
        scan.sensitive_files.len(),
        scan.blocked
    );

    if scan.blocked {
        tracing::warn!("Commit blocked by security scan");
        return Ok(CommitResult::Blocked(scan));
    }

    // 2. Get staged files and diff stats
    tracing::debug!("Executing: git diff --cached --name-status");
    let files_output = Command::new("git")
        .args(["diff", "--cached", "--name-status"])
        .output()
        .await?;

    if !files_output.status.success() {
        tracing::error!("git diff --cached --name-status failed");
        return Err(crate::domain::errors::CoreError::Git(
            "Failed to get staged files".to_string(),
        ));
    }

    tracing::debug!("Executing: git diff --cached --stat");
    let diff_stat_output = Command::new("git")
        .args(["diff", "--cached", "--stat"])
        .output()
        .await?;

    if !diff_stat_output.status.success() {
        tracing::error!("git diff --cached --stat failed");
        return Err(crate::domain::errors::CoreError::Git(
            "Failed to get diff stats".to_string(),
        ));
    }

    let mut files = String::from_utf8_lossy(&files_output.stdout).to_string();
    let mut diff_stat = String::from_utf8_lossy(&diff_stat_output.stdout).to_string();

    // Check if there are any staged changes - if not, auto-stage everything
    if files.trim().is_empty() {
        tracing::info!("No staged changes found, auto-staging all changes");

        // Try to stage all changes
        tracing::debug!("Executing: git add --all");
        let add_output = Command::new("git").args(["add", "--all"]).output().await?;

        if !add_output.status.success() {
            tracing::error!("git add --all failed");
            return Err(crate::domain::errors::CoreError::Git(
                "No changes to commit and failed to auto-stage".to_string(),
            ));
        }

        // Re-check staged files
        tracing::debug!("Re-checking staged files after auto-staging");
        let files_output = Command::new("git")
            .args(["diff", "--cached", "--name-status"])
            .output()
            .await?;

        let diff_stat_output = Command::new("git")
            .args(["diff", "--cached", "--stat"])
            .output()
            .await?;

        files = String::from_utf8_lossy(&files_output.stdout).to_string();
        diff_stat = String::from_utf8_lossy(&diff_stat_output.stdout).to_string();

        // If still empty, nothing to commit
        if files.trim().is_empty() {
            tracing::warn!("No changes to commit after auto-staging");
            return Err(crate::domain::errors::CoreError::Git(
                "No changes to commit".to_string(),
            ));
        }

        tracing::info!(
            "Auto-staging successful, found {} files",
            files.lines().count()
        );
    } else {
        tracing::debug!("Found {} staged files", files.lines().count());
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

CRITICAL: Respond with ONLY a valid JSON array. No explanations, no markdown formatting, no code fences.
Just the raw JSON array starting with [ and ending with ].

Expected format:
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
    tracing::info!("Executing Claude CLI to analyze changes");

    // Try to find Claude CLI executable using unified discovery
    let claude_path = crate::claude_discovery::ClaudeDiscovery::find_claude()
        .await
        .map_err(|e| {
            crate::domain::errors::CoreError::CommandFailed(format!(
                "Failed to find Claude CLI: {}",
                e
            ))
        })?;
    tracing::debug!("Found Claude CLI at: {}", claude_path.display());

    // Log the prompt being sent
    tracing::info!("=== CLAUDE PROMPT ===");
    tracing::info!("Prompt length: {} chars", prompt.len());
    tracing::info!("Prompt content:\n{}", prompt);
    tracing::info!("====================");

    // Build argument list for logging
    let args = vec!["-p", &prompt, "--output-format", "text", "--max-turns", "5"];

    // Log the complete command
    tracing::info!("=== CLAUDE CLI COMMAND ===");
    tracing::info!("Executable: {}", claude_path.display());
    tracing::info!(
        "Arguments: -p \"{}\" --output-format text --max-turns 5",
        if prompt.len() > 50 {
            format!("{}...", &prompt[..50])
        } else {
            prompt.clone()
        }
    );
    tracing::info!("==========================");

    // Also output to console for user visibility
    eprintln!(
        "$ claude -p \"<{} chars>\" --output-format text --max-turns 5",
        prompt.len()
    );

    let output = Command::new(&claude_path)
        .args(&args)
        .output()
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute Claude CLI at {}: {}", claude_path.display(), e);
            crate::domain::errors::CoreError::CommandFailed(
                format!("Failed to execute 'claude' command: {}\n\nPlease install Claude Code CLI: https://docs.claude.com/claude-code", e)
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!(
            "Claude command failed with exit code: {:?}",
            output.status.code()
        );
        tracing::error!("stderr: {}", stderr);
        tracing::debug!("stdout: {}", stdout);
        return Err(crate::domain::errors::CoreError::CommandFailed(format!(
            "Claude command failed to analyze changes:\n{}",
            stderr.trim()
        )));
    }

    tracing::debug!("Claude command completed successfully");

    // 5. Parse JSON response
    let response = String::from_utf8_lossy(&output.stdout);
    let stderr_output = String::from_utf8_lossy(&output.stderr);

    // Log complete response
    tracing::info!("=== CLAUDE RESPONSE ===");
    tracing::info!("Exit code: {:?}", output.status.code());
    tracing::info!("Stdout length: {} bytes", response.len());
    tracing::info!("Stderr length: {} bytes", stderr_output.len());

    if !stderr_output.is_empty() {
        tracing::info!("Stderr content:\n{}", stderr_output);
    }

    tracing::info!("Stdout content:\n{}", response);
    tracing::info!("======================");

    tracing::debug!("Extracting JSON from response");
    let json_str = extract_json_from_response(&response)?;
    tracing::trace!("Extracted JSON:\n{}", json_str);

    tracing::debug!("Parsing commit groups from JSON");
    let groups: Vec<CommitGroup> = serde_json::from_str(&json_str).map_err(|e| {
        tracing::error!("Failed to parse commit groups JSON: {}", e);
        tracing::debug!("Invalid JSON: {}", json_str);
        crate::domain::errors::CoreError::Git(format!("Failed to parse commit groups: {}", e))
    })?;

    if groups.is_empty() {
        tracing::warn!("Claude returned empty commit groups");
        return Err(crate::domain::errors::CoreError::Git(
            "No commit groups generated".to_string(),
        ));
    }

    tracing::info!("Successfully parsed {} commit groups", groups.len());
    for (i, group) in groups.iter().enumerate() {
        tracing::debug!(
            "Group {}: {} files - {}",
            i + 1,
            group.files.len(),
            group.message
        );
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

    // Try to extract JSON from markdown code fence
    if let Some(json_start) = trimmed.find("```json") {
        if let Some(fence_end) = trimmed[json_start..].find("```") {
            let json_content = &trimmed[json_start + 7..json_start + fence_end];
            let json_trimmed = json_content.trim();
            if json_trimmed.starts_with('[') && json_trimmed.ends_with(']') {
                return Ok(json_trimmed.to_string());
            }
        }
    }

    // Try to extract JSON from generic code fence
    if let Some(fence_start) = trimmed.find("```") {
        let after_fence = &trimmed[fence_start + 3..];
        // Skip language identifier if present (e.g., "```json\n")
        let content_start = after_fence.find('\n').map(|i| i + 1).unwrap_or(0);
        if let Some(fence_end) = after_fence[content_start..].find("```") {
            let json_content = &after_fence[content_start..content_start + fence_end];
            let json_trimmed = json_content.trim();
            if json_trimmed.starts_with('[') && json_trimmed.ends_with(']') {
                return Ok(json_trimmed.to_string());
            }
        }
    }

    // Try to find raw JSON array
    if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            return Ok(trimmed[start..=end].to_string());
        }
    }

    // Better error message with what was actually received
    let preview = if response.len() > 200 {
        format!(
            "{}... ({} more bytes)",
            &response[..200],
            response.len() - 200
        )
    } else {
        response.to_string()
    };

    tracing::error!(
        "No JSON array found in Claude response. Response preview: {}",
        preview
    );
    Err(crate::domain::errors::CoreError::Git(format!(
        "No JSON array in response. Claude returned: {}",
        preview
    )))
}

/// Commit a specific group of files with a given message (Feature 050)
///
/// This function stages the files in the group and creates a git commit
/// with the provided message. Used in the commit review workflow.
///
/// # Arguments
/// * `group` - The commit group containing files to commit
/// * `message` - The commit message (user-edited)
///
/// # Returns
/// * `Ok(())` - Commit succeeded
/// * `Err(CoreError)` - Commit failed
///
/// # Errors
/// * No files in group
/// * Git add failed
/// * Git commit failed
pub async fn commit_group(group: CommitGroup, message: String) -> Result<()> {
    // T037: Validate group has files
    if group.files.is_empty() {
        return Err(crate::domain::errors::CoreError::Git(
            "Commit group has no files".to_string(),
        ));
    }

    // Validate message is not empty
    if message.trim().is_empty() {
        return Err(crate::domain::errors::CoreError::Git(
            "Commit message cannot be empty".to_string(),
        ));
    }

    tracing::info!(
        "Committing group with {} files: {}",
        group.files.len(),
        message.lines().next().unwrap_or("")
    );

    // T037: Stage the specific files in this group
    // Use git add -- <files> to stage only these files
    let add_output = Command::new("git")
        .arg("add")
        .arg("--")
        .args(&group.files)
        .output()
        .await?;

    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        tracing::error!("Git add failed: {}", stderr);
        return Err(crate::domain::errors::CoreError::Git(format!(
            "Failed to stage files: {}",
            stderr
        )));
    }

    // T038: Create the commit with the message
    let commit_output = Command::new("git")
        .args(["commit", "-m", &message])
        .output()
        .await?;

    // T039: Handle errors
    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        tracing::error!("Git commit failed: {}", stderr);
        return Err(crate::domain::errors::CoreError::Git(format!(
            "Commit failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&commit_output.stdout);
    tracing::info!("Commit succeeded: {}", stdout.lines().next().unwrap_or(""));

    // T040: Return success
    Ok(())
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
        use crate::domain::git::security::{SecurityScanResult, SecurityWarning, Severity};

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
