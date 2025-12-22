//! Cargo command wrapper

use crate::tui::claude_stream::ClaudeStreamMessage;
use crate::tui::event::Event;
use crate::{Result, RscliError};
use std::process::{Output, Stdio};
use std::sync::mpsc;
use tokio::process::Command;

/// System prompt for RSCLI MCP integration
///
/// This is appended via `--append-system-prompt` to instruct Claude
/// to use MCP tools to communicate with rstn.
const RSCLI_SYSTEM_PROMPT: &str = r#"
## RSCLI MCP Integration

Use these MCP tools to communicate status and task progress:

- **rstn_report_status**: Report task status changes
  - status: "needs_input" (with prompt), "completed", or "error" (with message)

- **rstn_complete_task**: Mark tasks complete
  - task_id: Task ID (e.g., "T001", "T002")

- **rstn_read_spec**: Read spec artifacts
  - artifact: "spec", "plan", "tasks", "checklist", or "analysis"

- **rstn_get_context**: Get current feature context

Use these tools instead of text-based status output.
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
                cmd.arg("-c")
                    .arg("echo 'Spec status: Use Claude Code /spec-status command'");
            } else if args.first().map(|s| s.as_str()) == Some("list") {
                cmd.arg("-c").arg("cat specs/features.json | head -50");
            } else {
                cmd.arg("-c").arg("echo 'Unknown spec command'");
            }
        }
        _ => {
            return Err(RscliError::CommandNotFound(format!(
                "Unknown command: {}",
                name
            )));
        }
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| RscliError::CommandNotFound(e.to_string()))?;

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

/// Permission mode for Claude CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionMode {
    /// Plan before executing (like TUI Shift+Tab)
    Plan,
    /// Execute without asking
    Auto,
    /// Ask before each tool (default)
    Ask,
}

impl PermissionMode {
    /// Convert to CLI argument string
    pub fn as_cli_arg(&self) -> &'static str {
        match self {
            PermissionMode::Plan => "plan",
            PermissionMode::Auto => "auto",
            PermissionMode::Ask => "ask",
        }
    }
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
    /// Custom system prompt file path (for spec-kit prompts)
    pub system_prompt_file: Option<std::path::PathBuf>,
    /// Additional directories for Claude to access (for image pastes via --add-dir)
    pub add_dirs: Vec<std::path::PathBuf>,
    /// Permission mode (plan/auto/ask)
    pub permission_mode: Option<PermissionMode>,
    /// Additional files for context (multi-file context feature)
    pub context_files: Vec<std::path::PathBuf>,
}

/// Result from a Claude streaming command
#[derive(Debug, Clone, Default)]
pub struct ClaudeResult {
    /// Session ID for resuming conversation
    pub session_id: Option<String>,
    /// Whether the command exited successfully
    pub success: bool,
    /// Accumulated text content from assistant messages
    pub content: String,
    /// Captured stderr output (for debugging failures)
    pub stderr: String,
    /// Process exit code (None if process didn't exit normally)
    pub exit_code: Option<i32>,
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
/// Build JSONL message with context files for multi-file context feature
async fn build_jsonl_with_context(prompt: &str, context_files: &[std::path::PathBuf]) -> Result<String> {
    use serde_json::json;

    // Build content blocks array
    let mut content_blocks = vec![json!({
        "type": "text",
        "text": prompt
    })];

    // Add context files
    for path in context_files {
        // Read file content
        let file_content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to read context file {:?}: {}", path, e)))?;

        // Add file header and content
        let header = format!("=== {} ===\n\n", path.display());
        content_blocks.push(json!({
            "type": "text",
            "text": format!("{}{}", header, file_content)
        }));
    }

    // Build JSONL message
    let message = json!({
        "role": "user",
        "content": content_blocks
    });

    // Convert to JSONL (single line with newline)
    let jsonl = format!("{}\n", serde_json::to_string(&message)
        .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to serialize JSONL: {}", e)))?);

    Ok(jsonl)
}

pub async fn run_claude_command_streaming(
    command: &str,
    options: &ClaudeCliOptions,
    sender: Option<mpsc::Sender<Event>>,
) -> Result<ClaudeResult> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    // Find claude binary using unified discovery
    let claude_path = crate::claude_discovery::ClaudeDiscovery::find_claude()
        .await
        .map_err(|e| RscliError::CommandNotFound(format!("claude: {}", e)))?;

    let mut cmd = Command::new(&claude_path);

    // Add options
    if let Some(max) = options.max_turns {
        cmd.arg("--max-turns").arg(max.to_string());
    }
    // Add allowed tools for autonomous operation (instead of skip_permissions)
    if !options.allowed_tools.is_empty() {
        let tools_str = options.allowed_tools.join(",");
        cmd.arg("--allowedTools").arg(&tools_str);
        tracing::debug!("Added --allowedTools: {}", tools_str);
    }

    // Legacy: skip_permissions still works as fallback
    if options.skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
        tracing::warn!("Using skip_permissions (consider --allowedTools instead)");
    }

    // Permission mode (plan/auto/ask)
    if let Some(mode) = options.permission_mode {
        cmd.arg("--permission-mode").arg(mode.as_cli_arg());
        tracing::debug!("Added --permission-mode: {}", mode.as_cli_arg());
    }

    if let Some(ref session) = options.session_id {
        cmd.arg("--resume").arg(session);
    } else if options.continue_session {
        cmd.arg("--continue");
    }

    // Core args: prompt with optional context files
    let use_stdin_input = !options.context_files.is_empty();

    if use_stdin_input {
        // Multi-file context mode: use stdin with JSONL format
        cmd.arg("--input-format").arg("stream-json");
    } else {
        // Simple prompt mode: use -p flag
        cmd.arg("-p").arg(command);
    }

    cmd.arg("--output-format").arg("stream-json");
    cmd.arg("--verbose"); // Required when using -p or input-format with stream-json
    cmd.arg("--include-partial-messages"); // Show incremental output as Claude types

    // Point Claude to rstn's MCP server config
    if let Some(mcp_config_path) = crate::domain::paths::mcp_config_path()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
    {
        if std::path::Path::new(&mcp_config_path).exists() {
            cmd.arg("--mcp-config").arg(&mcp_config_path);
        }
    }

    // Add directories for image access (for paste placeholders)
    for dir in &options.add_dirs {
        if dir.exists() {
            cmd.arg("--add-dir").arg(dir);
        } else {
            tracing::warn!("Skipping non-existent --add-dir path: {:?}", dir);
        }
    }

    // If a custom system prompt file is provided, use it
    // Otherwise just append the RSCLI protocol instructions
    if let Some(ref prompt_file) = options.system_prompt_file {
        cmd.arg("--system-prompt-file").arg(prompt_file);
        // Still append the RSCLI protocol on top of the custom prompt
        cmd.arg("--append-system-prompt").arg(RSCLI_SYSTEM_PROMPT);
    } else {
        cmd.arg("--append-system-prompt").arg(RSCLI_SYSTEM_PROMPT);
    }

    // Log the CLI command being executed
    if let Some(ref s) = sender {
        let args: Vec<String> = cmd
            .as_std()
            .get_args()
            .map(|a| {
                let s = a.to_string_lossy();
                // Quote args containing spaces or special chars
                if s.contains(' ') || s.contains('"') || s.len() > 100 {
                    // Truncate very long args (like system prompts)
                    let truncated = if s.len() > 100 {
                        format!("{}...", &s[..100])
                    } else {
                        s.to_string()
                    };
                    format!("\"{}\"", truncated.replace('"', "\\\""))
                } else {
                    s.to_string()
                }
            })
            .collect();
        let cmd_string = format!("$ claude {}", args.join(" "));
        let _ = s.send(Event::CommandOutput(cmd_string));
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // If using stdin input, set up stdin pipe
    if use_stdin_input {
        cmd.stdin(Stdio::piped());
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| RscliError::CommandNotFound(format!("claude: {}", e)))?;

    // If using stdin input, write JSONL message with context files
    if use_stdin_input {
        use tokio::io::AsyncWriteExt;

        if let Some(mut stdin) = child.stdin.take() {
            // Build JSONL message with context files
            let jsonl_message = build_jsonl_with_context(command, &options.context_files).await?;

            // Write to stdin
            stdin
                .write_all(jsonl_message.as_bytes())
                .await
                .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to write to stdin: {}", e)))?;

            // Flush and close stdin
            stdin
                .flush()
                .await
                .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to flush stdin: {}", e)))?;

            drop(stdin); // Explicitly close stdin
            tracing::debug!("JSONL message written to Claude stdin ({} bytes)", jsonl_message.len());
        }
    }

    let mut result = ClaudeResult::default();
    let mut stderr_buffer = String::new(); // Accumulate stderr for error reporting
    let start_time = std::time::Instant::now(); // Track command duration

    // Read stdout line by line (JSONL format)
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut line_count = 0;

        while let Ok(Some(line)) = lines.next_line().await {
            line_count += 1;
            tracing::debug!(target: "claude_stream", "Read line {}: {} chars", line_count, line.len());

            // Try to parse as JSON
            match serde_json::from_str::<ClaudeStreamMessage>(&line) {
                Ok(msg) => {
                    tracing::debug!(target: "claude_stream", "Parsed message type: {}", msg.msg_type);

                    // Track session_id
                    if msg.session_id.is_some() {
                        result.session_id = msg.session_id.clone();
                    }

                    // Accumulate assistant text content for return value
                    if msg.msg_type == "assistant" {
                        if let Some(text) = msg.get_text() {
                            tracing::debug!(target: "claude_stream", "Assistant text: {} chars", text.len());
                            if !result.content.is_empty() {
                                result.content.push('\n');
                            }
                            result.content.push_str(&text);
                        }
                    }

                    // Send to TUI for real-time display (status comes via MCP tools)
                    if let Some(ref s) = sender {
                        match s.send(Event::ClaudeStream(msg)) {
                            Ok(_) => {
                                tracing::debug!(target: "claude_stream", "Sent ClaudeStream event to TUI")
                            }
                            Err(e) => {
                                tracing::error!(target: "claude_stream", "Failed to send ClaudeStream event: {}", e)
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(target: "claude_stream", "Failed to parse JSONL line {}: {}", line_count, e);
                }
            }
        }

        tracing::info!(target: "claude_stream", "Finished reading stdout: {} lines total", line_count);
    }

    // Capture and log stderr (for error messages)
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            // Accumulate for error reporting
            if !stderr_buffer.is_empty() {
                stderr_buffer.push('\n');
            }
            stderr_buffer.push_str(&line);

            // Log each line for real-time debugging
            tracing::debug!(target: "claude_cli", "stderr: {}", line);

            // Send to UI for display
            if let Some(ref s) = sender {
                let _ = s.send(Event::CommandOutput(format!("[stderr] {}", line)));
            }
        }
    }

    let exit_status = child.wait().await?;
    let exit_code = exit_status.code();
    let duration = start_time.elapsed();

    result.success = exit_status.success();
    result.stderr = stderr_buffer.clone();
    result.exit_code = exit_code;

    // Log completion summary
    if result.success {
        tracing::info!(
            exit_code = exit_code.unwrap_or(-1),
            duration_ms = duration.as_millis(),
            stdout_chars = result.content.len(),
            "Claude CLI completed successfully"
        );
    } else {
        tracing::error!(
            exit_code = exit_code.unwrap_or(-1),
            duration_ms = duration.as_millis(),
            stderr_lines = stderr_buffer.lines().count(),
            "Claude CLI failed"
        );

        // Log stderr at ERROR level (first 1000 chars for log visibility)
        if !stderr_buffer.is_empty() {
            let stderr_preview = if stderr_buffer.len() > 1000 {
                format!("{}... (truncated)", &stderr_buffer[..1000])
            } else {
                stderr_buffer.clone()
            };
            tracing::error!(stderr = %stderr_preview, "Claude CLI error output");
        }

        // Return Err with detailed error message
        let error_msg = build_claude_error_message(exit_code, &stderr_buffer, &result.content);
        return Err(RscliError::CommandFailed(error_msg));
    }

    Ok(result)
}

/// Build a detailed error message for Claude CLI failures
fn build_claude_error_message(
    exit_code: Option<i32>,
    stderr: &str,
    partial_content: &str,
) -> String {
    let mut msg = String::new();

    // Header with exit code
    msg.push_str(&format!(
        "Claude CLI command failed (exit code: {})\n\n",
        exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    ));

    // Error output
    if !stderr.is_empty() {
        msg.push_str("Error output:\n");
        msg.push_str(stderr);
        msg.push('\n');
    } else {
        msg.push_str("No error output captured.\n");
    }

    // Partial content if any
    if !partial_content.is_empty() {
        msg.push_str(&format!(
            "\nPartial output before failure ({} chars):\n{}\n",
            partial_content.len(),
            if partial_content.len() > 200 {
                format!("{}...", &partial_content[..200])
            } else {
                partial_content.to_string()
            }
        ));
    }

    // Pattern detection for common errors
    if let Some(hint) = detect_error_pattern(stderr) {
        msg.push_str("\nPossible cause:\n");
        msg.push_str(hint);
        msg.push('\n');
    }

    // Log file reference
    if let Ok(log_file) = crate::domain::paths::rstn_log_file() {
        msg.push_str(&format!("\nSee {} for full details.", log_file.display()));
    }

    msg
}

/// Detect common error patterns and provide hints
fn detect_error_pattern(stderr: &str) -> Option<&'static str> {
    let lower = stderr.to_lowercase();

    if lower.contains("mcp server") || lower.contains("mcp config") {
        Some("MCP server configuration issue. Check that rstn's MCP server is running and the config at ~/.rstn/mcp-session.json is valid.")
    } else if lower.contains("connection refused") {
        Some("Connection refused. The MCP server may not be running or the port may be blocked.")
    } else if lower.contains("api key") || lower.contains("authentication") {
        Some("API authentication issue. Ensure ANTHROPIC_API_KEY is set correctly.")
    } else if lower.contains("rate limit") {
        Some("API rate limit exceeded. Wait a few moments and try again.")
    } else if lower.contains("permission denied") {
        Some("File permission error. Check that rstn has write access to the specs/ directory.")
    } else if lower.contains("timeout") {
        Some("Operation timed out. This may indicate network issues or a very large request.")
    } else if lower.contains("command not found") || lower.contains("no such file") {
        Some("Claude CLI executable not found. Ensure 'claude' is installed and in PATH.")
    } else {
        None
    }
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
        let output =
            "test result: FAILED. 668 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out";
        let results = parse_test_output(output, "").unwrap();
        assert_eq!(results.passed, 668);
        assert_eq!(results.failed, 2);
    }

    #[test]
    fn test_detect_mcp_error() {
        let stderr = "Error: MCP server not found in config\nFailed to connect";
        let hint = detect_error_pattern(stderr);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("MCP server"));
    }

    #[test]
    fn test_detect_connection_refused_error() {
        let stderr = "Connection refused on port 12345";
        let hint = detect_error_pattern(stderr);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("Connection refused"));
    }

    #[test]
    fn test_detect_api_key_error() {
        let stderr = "Error: Invalid API key or authentication failed";
        let hint = detect_error_pattern(stderr);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("API authentication"));
    }

    #[test]
    fn test_detect_rate_limit_error() {
        let stderr = "API rate limit exceeded, please wait";
        let hint = detect_error_pattern(stderr);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("rate limit"));
    }

    #[test]
    fn test_detect_permission_error() {
        let stderr = "permission denied: cannot write to file";
        let hint = detect_error_pattern(stderr);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("permission"));
    }

    #[test]
    fn test_detect_no_pattern() {
        let stderr = "Some random error message";
        let hint = detect_error_pattern(stderr);
        assert!(hint.is_none());
    }

    #[test]
    fn test_build_error_message_format() {
        let msg = build_claude_error_message(
            Some(1),
            "Connection refused on port 12345",
            "Partial output...",
        );

        assert!(msg.contains("exit code: 1"));
        assert!(msg.contains("Connection refused"));
        assert!(msg.contains("Partial output"));
    }

    #[test]
    fn test_build_error_message_with_hint() {
        let msg = build_claude_error_message(Some(1), "MCP server configuration is invalid", "");

        assert!(msg.contains("MCP server"));
        assert!(msg.contains("Possible cause:"));
    }

    #[test]
    fn test_build_error_message_no_stderr() {
        let msg = build_claude_error_message(Some(1), "", "");

        assert!(msg.contains("exit code: 1"));
        assert!(msg.contains("No error output captured"));
    }

    #[test]
    fn test_build_error_message_truncates_long_content() {
        let long_content = "a".repeat(300);
        let msg = build_claude_error_message(Some(1), "Error", &long_content);

        assert!(msg.contains("Partial output"));
        assert!(msg.contains("...")); // Should be truncated
    }
}
