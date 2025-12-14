//! Git commit security scanning
//!
//! Provides trustworthy, Rust-based security scanning for staged changes.
//! Detects secrets, sensitive files, and potential security issues before commit.

use regex::Regex;
use std::sync::OnceLock;
use tokio::process::Command;

use crate::errors::Result;

/// Security scan result for staged changes
#[derive(Debug, Clone)]
pub struct SecurityScanResult {
    pub blocked: bool,
    pub warnings: Vec<SecurityWarning>,
    pub sensitive_files: Vec<SensitiveFile>,
}

/// A security warning found during scanning
#[derive(Debug, Clone)]
pub struct SecurityWarning {
    pub file_path: String,
    pub line_number: usize,
    pub pattern_matched: String,
    pub severity: Severity,
    pub message: String,
}

/// Severity level of a security issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Definite secret - BLOCKS commit (private keys, etc.)
    Critical,
    /// Likely secret - WARNS but allows (API keys, passwords, etc.)
    High,
    /// Possible secret - INFO only (base64, long hex, etc.)
    Medium,
}

/// A sensitive file found in staged changes
#[derive(Debug, Clone)]
pub struct SensitiveFile {
    pub path: String,
    pub reason: String,
    pub suggest_gitignore: bool,
}

/// Secret detection patterns
/// Format: (regex_pattern, description, severity)
const SECRET_PATTERNS: &[(&str, &str, Severity)] = &[
    // Critical (block commit)
    (
        r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----",
        "Private key detected",
        Severity::Critical,
    ),
    (
        r"-----BEGIN CERTIFICATE-----",
        "Certificate detected (may contain private key)",
        Severity::Critical,
    ),
    // High (warn but allow)
    (
        r#"(?i)api[_-]?key["\']?\s*[:=]\s*["\'][^"\']{10,}["\']"#,
        "API key pattern",
        Severity::High,
    ),
    (
        r#"(?i)secret[_-]?key["\']?\s*[:=]\s*["\'][^"\']{10,}["\']"#,
        "Secret key pattern",
        Severity::High,
    ),
    (
        r#"(?i)password["\']?\s*[:=]\s*["\'][^"\']{8,}["\']"#,
        "Password pattern",
        Severity::High,
    ),
    (
        r#"(?i)token["\']?\s*[:=]\s*["\'][^"\']{20,}["\']"#,
        "Token pattern",
        Severity::High,
    ),
    (
        r#"(?i)auth[_-]?token["\']?\s*[:=]\s*["\'][^"\']{20,}["\']"#,
        "Auth token pattern",
        Severity::High,
    ),
    (
        r"gh[ps]_[a-zA-Z0-9]{36,}",
        "GitHub token",
        Severity::High,
    ),
    (
        r"sk-[a-zA-Z0-9]{20,}",
        "OpenAI/Anthropic API key",
        Severity::High,
    ),
    (
        r"AIza[0-9A-Za-z\\-_]{35}",
        "Google API key",
        Severity::High,
    ),
    (
        r#"(?i)aws[_-]?access[_-]?key[_-]?id["\']?\s*[:=]\s*["\'][^"\']{16,}["\']"#,
        "AWS Access Key",
        Severity::High,
    ),
    // Medium (info only)
    (
        r"[a-zA-Z0-9+/]{40,}={0,2}",
        "Base64 string (possible secret)",
        Severity::Medium,
    ),
    (
        r"[0-9a-fA-F]{64,}",
        "Long hex string (possible key)",
        Severity::Medium,
    ),
];

/// Sensitive filename patterns
const SENSITIVE_FILES: &[(&str, &str)] = &[
    (".env", "Environment file with secrets"),
    (".env.", "Environment file variant"),
    ("credentials.json", "Credentials file"),
    ("secrets.yaml", "Secrets configuration"),
    ("secrets.yml", "Secrets configuration"),
    ("*.pem", "Private key file"),
    ("*.key", "Private key file"),
    ("id_rsa", "SSH private key"),
    ("id_ed25519", "SSH private key"),
    ("id_ecdsa", "SSH private key"),
    ("*.pfx", "Certificate file"),
    ("*.p12", "Certificate file"),
    (".npmrc", "NPM credentials"),
    (".pypirc", "PyPI credentials"),
];

/// Scan staged changes for security issues
pub async fn scan_staged_changes() -> Result<SecurityScanResult> {
    // Get diff content
    let diff_output = Command::new("git")
        .args(&["diff", "--cached"])
        .output()
        .await?;

    if !diff_output.status.success() {
        return Err(crate::errors::CoreError::Git(
            "Failed to get staged diff".to_string(),
        ));
    }

    let diff = String::from_utf8_lossy(&diff_output.stdout);

    // Get staged filenames
    let files_output = Command::new("git")
        .args(&["diff", "--cached", "--name-only"])
        .output()
        .await?;

    if !files_output.status.success() {
        return Err(crate::errors::CoreError::Git(
            "Failed to get staged files".to_string(),
        ));
    }

    let files = String::from_utf8_lossy(&files_output.stdout);

    // Scan diff for secrets (only + lines)
    let warnings = scan_diff_for_secrets(&diff);

    // Check filenames
    let sensitive_files = check_sensitive_filenames(&files);

    // Block if Critical severity found
    let blocked = warnings
        .iter()
        .any(|w| matches!(w.severity, Severity::Critical));

    Ok(SecurityScanResult {
        blocked,
        warnings,
        sensitive_files,
    })
}

/// Scan diff content for secret patterns
fn scan_diff_for_secrets(diff: &str) -> Vec<SecurityWarning> {
    let mut warnings = Vec::new();
    let mut current_file = String::new();
    let mut line_number = 0;

    for line in diff.lines() {
        // Track current file from diff headers (+++  b/path/to/file)
        if let Some(file_path) = line.strip_prefix("+++ b/") {
            current_file = file_path.to_string();
            line_number = 0;
            continue;
        }

        // Track line numbers from diff hunks (@@ -1,2 +3,4 @@)
        if line.starts_with("@@") {
            line_number = parse_line_number_from_hunk(line);
            continue;
        }

        // Only scan added lines (starting with +)
        if !line.starts_with('+') {
            continue;
        }

        line_number += 1;

        // Apply each regex pattern
        for (pattern, message, severity) in SECRET_PATTERNS {
            if let Ok(re) = get_regex(pattern) {
                if re.is_match(line) {
                    // Skip if it's clearly a comment about the pattern itself
                    if line.contains("example") || line.contains("TODO") || line.contains("FIXME") {
                        continue;
                    }

                    warnings.push(SecurityWarning {
                        file_path: current_file.clone(),
                        line_number,
                        pattern_matched: pattern.to_string(),
                        severity: *severity,
                        message: message.to_string(),
                    });
                }
            }
        }
    }

    warnings
}

/// Parse line number from diff hunk header
fn parse_line_number_from_hunk(hunk: &str) -> usize {
    // Format: @@ -1,2 +3,4 @@
    // We want the line number after the +, which is 3 in this example
    if let Some(plus_part) = hunk.split('+').nth(1) {
        if let Some(num_str) = plus_part.split(',').next() {
            return num_str.trim().parse().unwrap_or(0);
        }
    }
    0
}

/// Check staged filenames for sensitive patterns
fn check_sensitive_filenames(files: &str) -> Vec<SensitiveFile> {
    let mut sensitive = Vec::new();

    for file in files.lines() {
        for (pattern, reason) in SENSITIVE_FILES {
            if file_matches_pattern(file, pattern) {
                sensitive.push(SensitiveFile {
                    path: file.to_string(),
                    reason: reason.to_string(),
                    suggest_gitignore: true,
                });
                break; // Only add each file once
            }
        }
    }

    sensitive
}

/// Check if filename matches pattern (supports wildcards)
fn file_matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern.starts_with('*') {
        // Wildcard at start (*.ext)
        let ext = &pattern[1..];
        filename.ends_with(ext)
    } else if pattern.ends_with('*') {
        // Wildcard at end (prefix*)
        let prefix = &pattern[..pattern.len() - 1];
        filename.starts_with(prefix)
    } else if pattern.contains('*') {
        // Wildcard in middle
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            filename.starts_with(parts[0]) && filename.ends_with(parts[1])
        } else {
            filename.contains(pattern.trim_matches('*'))
        }
    } else {
        // Exact match or contains
        filename == pattern || filename.contains(pattern)
    }
}

/// Lazy-compiled regex cache
fn get_regex(pattern: &str) -> std::result::Result<&'static Regex, regex::Error> {
    static CACHE: OnceLock<Vec<(String, Regex)>> = OnceLock::new();

    let cache = CACHE.get_or_init(|| {
        SECRET_PATTERNS
            .iter()
            .filter_map(|(pattern, _, _)| {
                Regex::new(pattern)
                    .ok()
                    .map(|re| (pattern.to_string(), re))
            })
            .collect()
    });

    cache
        .iter()
        .find(|(p, _)| p == pattern)
        .map(|(_, re)| re)
        .ok_or_else(|| regex::Error::Syntax("Pattern not found in cache".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_matches_pattern() {
        assert!(file_matches_pattern(".env", ".env"));
        assert!(file_matches_pattern(".env.local", ".env."));
        assert!(file_matches_pattern("key.pem", "*.pem"));
        assert!(file_matches_pattern("id_rsa", "id_rsa"));
        assert!(!file_matches_pattern("safe.txt", ".env"));
    }

    #[test]
    fn test_parse_line_number() {
        assert_eq!(parse_line_number_from_hunk("@@ -1,2 +3,4 @@"), 3);
        assert_eq!(parse_line_number_from_hunk("@@ -10,5 +20,8 @@"), 20);
    }

    #[test]
    fn test_secret_pattern_private_key() {
        let diff = "+-----BEGIN RSA PRIVATE KEY-----\n";
        let warnings = scan_diff_for_secrets(diff);
        assert!(!warnings.is_empty());
        assert!(matches!(warnings[0].severity, Severity::Critical));
    }

    #[test]
    fn test_secret_pattern_api_key() {
        let diff = "+api_key = \"sk-1234567890abcdef\"\n";
        let warnings = scan_diff_for_secrets(diff);
        assert!(!warnings.is_empty());
        assert!(matches!(warnings[0].severity, Severity::High));
    }

    #[test]
    fn test_ignore_unchanged_lines() {
        let diff = " api_key = \"sk-1234567890abcdef\"\n"; // No +
        let warnings = scan_diff_for_secrets(diff);
        assert!(warnings.is_empty());
    }
}
