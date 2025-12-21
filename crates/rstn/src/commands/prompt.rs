//! CLI command for direct Claude prompting

use crate::runners::cargo::ClaudeCliOptions;
use crate::tui::claude_stream::ClaudeStreamMessage;
use crate::tui::event::Event;
use crate::tui::mcp_server::McpState;
use crate::{Result, RscliError};
use colored::Colorize;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

/// CLI execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum _InputMode {
    Normal,
    Editing,
}

/// System prompt for RSCLI MCP integration
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

/// Result from a Claude streaming command
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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

/// Run prompt command with streaming output to stdout
pub async fn run(
    message: &str,
    max_turns: u32,
    skip_permissions: bool,
    continue_session: bool,
    session_id: Option<String>,
    allowed_tools: Vec<String>,
    context: Vec<std::path::PathBuf>,
    verbose: bool,
) -> Result<ClaudeResult> {
    // Print initial status
    if verbose {
        eprintln!("{}", "🤖 Sending prompt to Claude...".bright_blue());
        if !context.is_empty() {
            eprintln!(
                "   {} context file(s): {}",
                "📎".bright_yellow(),
                context.len()
            );
        }
        eprintln!();
    }

    // Build Claude CLI options
    let options = ClaudeCliOptions {
        max_turns: Some(max_turns),
        skip_permissions,
        continue_session,
        session_id,
        allowed_tools,
        system_prompt_file: None,
        add_dirs: vec![],
        permission_mode: None,
        context_files: context,
    };

    // Get MCP state if available (from global accessor)
    let mcp_state = crate::tui::mcp_server::get_global_mcp_state();

    // Run Claude command with custom streaming handler
    let result = run_claude_with_cli_streaming(message, &options, mcp_state).await?;

    // Print completion message
    eprintln!();
    if result.success {
        eprintln!(
            "{}",
            format!(
                "✓ Response complete (session: {})",
                result.session_id.as_deref().unwrap_or("unknown")
            )
            .green()
        );
    } else {
        eprintln!("{}", "✗ Command failed".red());
        if !result.stderr.is_empty() {
            eprintln!("{}", result.stderr.bright_red());
        }
        return Err(RscliError::CommandFailed(
            "Claude CLI execution failed".to_string(),
        ));
    }

    Ok(result)
}

/// CLI-specific streaming handler (prints directly to stdout)
///
/// Optionally accepts MCP state for interactive prompts via Mini TUI mode.
async fn run_claude_with_cli_streaming(
    message: &str,
    options: &ClaudeCliOptions,
    mcp_state: Option<Arc<Mutex<McpState>>>,
) -> Result<ClaudeResult> {
    // Find claude binary
    let claude_path = crate::claude_discovery::ClaudeDiscovery::find_claude()
        .await
        .map_err(|e| RscliError::CommandNotFound(format!("claude: {}", e)))?;

    let mut cmd = Command::new(&claude_path);

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

    // Core args: prompt, streaming JSON with partial messages
    cmd.arg("-p").arg(message);
    cmd.arg("--output-format").arg("stream-json");
    cmd.arg("--verbose"); // Required when using -p with stream-json
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

    // Append the RSCLI protocol instructions
    cmd.arg("--append-system-prompt").arg(RSCLI_SYSTEM_PROMPT);

    // Spawn process
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to spawn Claude: {}", e)))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RscliError::Other(anyhow::anyhow!("Failed to capture stdout")))?;

    // Stream JSONL output
    let mut result = ClaudeResult {
        session_id: None,
        success: false,
        content: String::new(),
        stderr: String::new(),
        exit_code: None,
    };

    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        // Poll for MCP events if MCP server is active
        if let Some(ref state) = mcp_state {
            if let Ok(mut state_guard) = state.try_lock() {
                // Drain TUI events (MCP sends events here)
                let events: Vec<Event> = state_guard.drain_tui_events();
                for event in events {
                    if let Event::McpStatus { status, prompt, .. } = event {
                        if status == "needs_input" {
                            // Show mini dialog to collect user input
                            let prompt_text = prompt.unwrap_or_else(|| "Enter input:".to_string());

                            eprintln!(); // Add newline before dialog
                            eprintln!("{}", "📥 MCP Input Request".bright_blue());

                            let dialog = crate::tui::mini_dialog::MiniTUIDialog::new(prompt_text);
                            match dialog.run() {
                                Ok(Some(input)) => {
                                    // User provided input - send to MCP
                                    state_guard.send_input_response(input.clone());
                                    eprintln!("{}", format!("✓ Response sent: {}", input).green());
                                }
                                Ok(None) => {
                                    // User cancelled - send empty response
                                    state_guard.send_input_response(String::new());
                                    eprintln!("{}", "⚠ Cancelled - empty response sent".yellow());
                                }
                                Err(e) => {
                                    eprintln!("{}", format!("✗ Dialog error: {}", e).red());
                                    state_guard.send_input_response(String::new());
                                }
                            }

                            eprintln!(); // Add newline after dialog
                            eprintln!("{}", "Resuming Claude output...".bright_blue());
                        }
                    }
                }
            }
        }

        // Parse JSONL message
        if let Ok(msg) = serde_json::from_str::<ClaudeStreamMessage>(&line) {
            // Track session ID
            if msg.session_id.is_some() {
                result.session_id = msg.session_id.clone();
            }

            // Print assistant text to stdout (real-time streaming)
            if msg.msg_type == "assistant" {
                if let Some(text) = msg.get_text() {
                    print!("{}", text);
                    std::io::Write::flush(&mut std::io::stdout())?;
                    result.content.push_str(&text);
                }
            }
        }
    }

    // Wait for process completion
    let status = child
        .wait()
        .await
        .map_err(|e| RscliError::Other(anyhow::anyhow!("Wait failed: {}", e)))?;

    result.exit_code = status.code();
    result.success = status.success();

    // Capture stderr if failed
    if !result.success {
        if let Some(mut stderr) = child.stderr.take() {
            let mut stderr_content = String::new();
            stderr
                .read_to_string(&mut stderr_content)
                .await
                .map_err(|e| RscliError::Other(anyhow::anyhow!("Failed to read stderr: {}", e)))?;
            result.stderr = stderr_content;
        }
    }

    Ok(result)
}
