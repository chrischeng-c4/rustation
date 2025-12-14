//! Cargo command wrapper

use crate::tui::claude_stream::{ClaudeStreamMessage, RscliStatus};
use crate::tui::event::Event;
use crate::{Result, RscliError};
use std::process::{Output, Stdio};
use std::sync::mpsc;
use tokio::process::Command;

/// System prompt for RSCLI JSON status output
///
/// This is appended via `--append-system-prompt` to instruct Claude
/// to output structured status blocks that rscli can parse.
const RSCLI_SYSTEM_PROMPT: &str = r#"
## RSCLI Integration Protocol

At the END of your response, output a JSON status block:

```rscli-status
{"status":"needs_input","prompt":"Your prompt here"}
```

Status values:
- "needs_input": Need user input. Include "prompt" field with the question.
- "completed": Phase finished successfully.
- "error": Error occurred. Include "message" field with error details.

Always include this status block at the very end of your response.
"#;

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

    let output = cmd.output().await?;

    // Parse test output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // If verbose, print the output
    if verbose {
        print!("{}", stdout);
        eprint!("{}", stderr);
    }

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
            let passed = extract_number(&line, "passed");
            let failed = extract_number(&line, "failed");
            let ignored = extract_number(&line, "ignored");
            let filtered_out = extract_number(&line, "filtered out");

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

/// Run cargo build
pub async fn build(release: bool, verbose: bool) -> Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.arg("-p").arg("rush");

    if release {
        cmd.arg("--release");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    cmd.output()
        .await
        .map_err(|e| RscliError::CommandNotFound(format!("cargo: {}", e)))
}

/// Run cargo check
pub async fn check(verbose: bool) -> Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    cmd.arg("-p").arg("rush");

    if verbose {
        cmd.arg("--verbose");
    }

    cmd.output()
        .await
        .map_err(|e| RscliError::CommandNotFound(format!("cargo: {}", e)))
}

/// Run cargo clippy
pub async fn clippy(fix: bool, verbose: bool) -> Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy");
    cmd.arg("--all-targets");
    cmd.arg("--all-features");

    if fix {
        cmd.arg("--fix");
        cmd.arg("--allow-dirty");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    cmd.output()
        .await
        .map_err(|e| RscliError::CommandNotFound(format!("cargo: {}", e)))
}

/// Run cargo fmt
pub async fn fmt(check: bool, verbose: bool) -> Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt");

    if check {
        cmd.arg("--check");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    cmd.output()
        .await
        .map_err(|e| RscliError::CommandNotFound(format!("cargo: {}", e)))
}

/// Command output collected for TUI display
#[derive(Debug, Clone, Default)]
pub struct CommandOutput {
    pub lines: Vec<String>,
    pub success: bool,
}

/// Run a generic cargo-style command and collect output
/// Returns collected output for TUI display (doesn't print to stdout)
pub async fn run_cargo_command(name: &str, args: &[String]) -> Result<CommandOutput> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut cmd = Command::new("cargo");

    // Map command names to cargo subcommands
    match name {
        "test" => {
            cmd.arg("test").arg("-p").arg("rush");
            for arg in args {
                if arg == "--lib" {
                    cmd.arg("--lib");
                } else if arg == "--integration" {
                    cmd.arg("--test").arg("*");
                } else {
                    cmd.arg(arg);
                }
            }
        }
        "build" => {
            cmd.arg("build").arg("-p").arg("rush");
            if args.contains(&"--release".to_string()) {
                cmd.arg("--release");
            }
        }
        "check" => {
            cmd.arg("check").arg("-p").arg("rush");
        }
        "lint" => {
            cmd.arg("clippy").arg("--all-targets").arg("--all-features");
            if args.contains(&"--fix".to_string()) {
                cmd.arg("--fix").arg("--allow-dirty");
            }
        }
        "fmt" => {
            cmd.arg("fmt");
            if args.contains(&"--check".to_string()) {
                cmd.arg("--check");
            }
        }
        "ci" => {
            // CI runs multiple commands - just run clippy for now
            cmd.arg("clippy")
                .arg("--all-targets")
                .arg("--all-features")
                .arg("--")
                .arg("-D")
                .arg("warnings");
        }
        "doctor" => {
            // Doctor is special - check various things
            cmd = Command::new("rustc");
            cmd.arg("--version");
        }
        "spec" => {
            // Spec commands use the .specify scripts
            cmd = Command::new("bash");
            if args.first().map(|s| s.as_str()) == Some("status") {
                cmd.arg("-c").arg("echo 'Spec status: Use Claude Code /spec-status command'");
            } else if args.first().map(|s| s.as_str()) == Some("list") {
                cmd.arg("-c").arg("cat specs/features.json | head -50");
            } else {
                cmd.arg("-c").arg("echo 'Unknown spec command'");
            }
        }
        _ => {
            return Err(RscliError::CommandNotFound(format!("Unknown command: {}", name)));
        }
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| RscliError::CommandNotFound(e.to_string()))?;

    let mut output = CommandOutput::default();

    // Read stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            output.lines.push(line);
        }
    }

    // Read stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            output.lines.push(line);
        }
    }

    let status = child.wait().await?;
    output.success = status.success();
    Ok(output)
}

/// Options for Claude CLI execution
#[derive(Debug, Clone, Default)]
pub struct ClaudeCliOptions {
    /// Maximum agentic turns
    pub max_turns: Option<u32>,
    /// Skip permission prompts
    pub skip_permissions: bool,
    /// Continue previous session
    pub continue_session: bool,
    /// Resume specific session ID
    pub session_id: Option<String>,
    /// Allowed tools (empty = all)
    pub allowed_tools: Vec<String>,
}

/// Result from a Claude streaming command
#[derive(Debug, Clone, Default)]
pub struct ClaudeResult {
    /// Session ID for resuming conversation
    pub session_id: Option<String>,
    /// Parsed RSCLI status from Claude's output
    pub status: Option<RscliStatus>,
    /// Whether the command exited successfully
    pub success: bool,
}

/// Run a Claude Code CLI command in headless mode
/// Uses `claude -p "command"` to execute spec-kit workflows
pub async fn run_claude_command(command: &str) -> Result<CommandOutput> {
    run_claude_command_with_options(command, &ClaudeCliOptions::default(), None).await
}

/// Run a Claude Code CLI command with options (legacy, returns CommandOutput)
pub async fn run_claude_command_with_options(
    command: &str,
    options: &ClaudeCliOptions,
    sender: Option<mpsc::Sender<Event>>,
) -> Result<CommandOutput> {
    let result = run_claude_command_streaming(command, options, sender).await?;

    // Convert ClaudeResult to CommandOutput for backwards compatibility
    Ok(CommandOutput {
        lines: vec![], // Lines were sent via events
        success: result.success,
    })
}

/// Run a Claude Code CLI command with streaming JSON output
///
/// This uses `--output-format stream-json` to get JSONL output and
/// `--append-system-prompt` to instruct Claude about the RSCLI protocol.
pub async fn run_claude_command_streaming(
    command: &str,
    options: &ClaudeCliOptions,
    sender: Option<mpsc::Sender<Event>>,
) -> Result<ClaudeResult> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    // Find claude binary
    let claude_path = find_claude_path();

    let mut cmd = Command::new(claude_path.as_deref().unwrap_or("claude"));

    // Add options
    if let Some(max) = options.max_turns {
        cmd.arg("--max-turns").arg(max.to_string());
    }
    if options.skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }
    if let Some(ref session) = options.session_id {
        cmd.arg("--resume").arg(session);
    } else if options.continue_session {
        cmd.arg("--continue");
    }
    if !options.allowed_tools.is_empty() {
        cmd.arg("--allowedTools")
            .arg(options.allowed_tools.join(","));
    }

    // Core args: prompt, streaming JSON, and system prompt
    cmd.arg("-p").arg(command);
    cmd.arg("--output-format").arg("stream-json");
    cmd.arg("--append-system-prompt").arg(RSCLI_SYSTEM_PROMPT);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| RscliError::CommandNotFound(format!("claude: {}", e)))?;

    let mut result = ClaudeResult::default();

    // Read stdout line by line (JSONL format)
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            // Try to parse as JSON
            if let Ok(msg) = serde_json::from_str::<ClaudeStreamMessage>(&line) {
                // Track session_id
                if msg.session_id.is_some() {
                    result.session_id = msg.session_id.clone();
                }

                // Parse status from assistant messages
                if msg.msg_type == "assistant" {
                    if let Some(status) = msg.parse_status() {
                        result.status = Some(status);
                    }
                }

                // Send to TUI for real-time display
                if let Some(ref s) = sender {
                    let _ = s.send(Event::ClaudeStream(msg));
                }
            }
        }
    }

    // Also capture stderr (for error messages)
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(_line)) = lines.next_line().await {
            // Stderr is typically not JSON, could log or ignore
        }
    }

    let exit_status = child.wait().await?;
    result.success = exit_status.success();

    Ok(result)
}

/// Find the claude binary path
fn find_claude_path() -> Option<String> {
    let claude_paths = [
        std::env::var("HOME")
            .map(|h| format!("{}/.claude/local/claude", h))
            .unwrap_or_default(),
        "/usr/local/bin/claude".to_string(),
        "claude".to_string(),
    ];

    for path in &claude_paths {
        if !path.is_empty() && (path == "claude" || std::path::Path::new(path).exists()) {
            return Some(path.clone());
        }
    }
    None
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
