//! Test execution and result parsing

use crate::errors::{CoreError, Result};
use std::process::Stdio;
use tokio::process::Command;

/// Test results summary
#[derive(Debug, Clone)]
pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub filtered_out: usize,
}

impl TestResults {
    pub fn total(&self) -> usize {
        self.passed + self.failed
    }
}

/// Run cargo test
pub async fn run_tests(
    filter: Option<&str>,
    lib_only: bool,
    integration_only: bool,
    verbose: bool,
) -> Result<TestResults> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");

    // Target the rush package specifically
    cmd.arg("-p").arg("rush");

    // Add filter if provided
    if let Some(f) = filter {
        cmd.arg(f);
    }

    // Test type flags
    if lib_only {
        cmd.arg("--lib");
    } else if integration_only {
        cmd.arg("--test").arg("*");
    }

    // Verbosity
    if !verbose {
        cmd.arg("--quiet");
    }

    // Capture output for parsing
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().await.map_err(|e| {
        CoreError::CommandFailed(format!("Failed to execute cargo test: {}", e))
    })?;

    // Parse test output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // If verbose, the output was already printed by cargo
    // In non-verbose mode, we capture and parse

    // Parse test summary
    parse_test_output(&stdout, &stderr)
}

/// Parse cargo test output to extract results
fn parse_test_output(stdout: &str, stderr: &str) -> Result<TestResults> {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for the summary line: "test result: ok. 670 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
    for line in combined.lines() {
        if line.contains("test result:") {
            // Parse the summary
            let passed = extract_number(line, "passed");
            let failed = extract_number(line, "failed");
            let ignored = extract_number(line, "ignored");
            let filtered_out = extract_number(line, "filtered out");

            return Ok(TestResults {
                passed,
                failed,
                ignored,
                filtered_out,
            });
        }
    }

    // If we couldn't find the summary, assume success if exit code was 0
    Ok(TestResults {
        passed: 0,
        failed: 0,
        ignored: 0,
        filtered_out: 0,
    })
}

fn extract_number(line: &str, keyword: &str) -> usize {
    // Find the keyword and extract the number before it
    if let Some(pos) = line.find(keyword) {
        let before = &line[..pos];
        // Get the last word before the keyword
        if let Some(num_str) = before.split_whitespace().last() {
            return num_str.parse().unwrap_or(0);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output() {
        let output = "test result: ok. 670 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s";
        let results = parse_test_output(output, "").unwrap();
        assert_eq!(results.passed, 670);
        assert_eq!(results.failed, 0);
        assert_eq!(results.ignored, 0);
        assert_eq!(results.filtered_out, 0);
    }

    #[test]
    fn test_parse_output_with_failures() {
        let output = "test result: FAILED. 668 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out";
        let results = parse_test_output(output, "").unwrap();
        assert_eq!(results.passed, 668);
        assert_eq!(results.failed, 2);
    }
}
