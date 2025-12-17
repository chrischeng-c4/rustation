//! MCP Server implementation for rstn
//!
//! Provides an embedded MCP (Model Context Protocol) server that enables
//! Claude Code to communicate with rstn via structured tool calls instead
//! of fragile text parsing.
//!
//! Architecture:
//! - Uses HTTP/SSE transport since TUI owns stdio
//! - Runs on localhost with configurable port (default: 19560)
//! - Sends events to TUI via mpsc channel
//! - Supports tool registration for status reporting, spec reading, etc.

use crate::tui::event::Event;
use anyhow::{Context, Result};
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::server::HttpMcpServer;
use prism_mcp_rs::transport::http::HttpServerTransport;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{error, info, warn};

/// Default port for MCP server
pub const DEFAULT_MCP_PORT: u16 = 19560;

/// MCP server configuration
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Server name for MCP protocol
    pub name: String,
    /// Server version for MCP protocol
    pub version: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_MCP_PORT,
            name: "rstn".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Shared state accessible by tool handlers
#[derive(Debug, Default)]
pub struct McpState {
    /// Current feature number (e.g., "060")
    pub feature_number: Option<String>,
    /// Current feature name (e.g., "mcp-server-infrastructure")
    pub feature_name: Option<String>,
    /// Current branch name
    pub branch: Option<String>,
    /// Current SDD phase
    pub phase: Option<String>,
    /// Spec directory path
    pub spec_dir: Option<String>,
}

/// Result from a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    /// Whether the tool call succeeded
    pub success: bool,
    /// Response message or data
    pub message: String,
    /// Optional structured data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Arguments for rstn_report_status tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStatusArgs {
    /// Status type: "needs_input" | "completed" | "error"
    pub status: String,
    /// Prompt text for needs_input status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Error message for error status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Arguments for rstn_read_spec tool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used by MCP handlers
pub struct ReadSpecArgs {
    /// Artifact to read: "spec" | "plan" | "tasks" | "checklist" | "analysis"
    pub artifact: String,
}

/// Feature context response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used by MCP handlers
pub struct FeatureContext {
    pub feature_number: Option<String>,
    pub feature_name: Option<String>,
    pub branch: Option<String>,
    pub phase: Option<String>,
    pub spec_dir: Option<String>,
}

/// JSON schema for rstn_report_status tool
fn status_tool_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "status": {
                "type": "string",
                "enum": ["needs_input", "completed", "error"],
                "description": "Current task status"
            },
            "prompt": {
                "type": "string",
                "description": "Prompt to show user (for needs_input)"
            },
            "message": {
                "type": "string",
                "description": "Error message (for error status)"
            }
        },
        "required": ["status"]
    })
}

/// JSON schema for rstn_read_spec tool
fn read_spec_tool_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "artifact": {
                "type": "string",
                "enum": ["spec", "plan", "tasks", "checklist", "analysis"],
                "description": "Which artifact to read"
            }
        },
        "required": ["artifact"]
    })
}

/// JSON schema for rstn_get_context tool
fn get_context_tool_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    })
}

/// Handle for controlling the MCP server
pub struct McpServerHandle {
    /// Shutdown signal sender
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Server port
    port: u16,
    /// Server state
    state: Arc<Mutex<McpState>>,
}

impl McpServerHandle {
    /// Get the port the server is running on
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the server URL
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Update the server state with current feature info
    pub async fn update_state(
        &self,
        feature_number: Option<String>,
        feature_name: Option<String>,
        branch: Option<String>,
        phase: Option<String>,
        spec_dir: Option<String>,
    ) {
        let mut state = self.state.lock().await;
        state.feature_number = feature_number;
        state.feature_name = feature_name;
        state.branch = branch;
        state.phase = phase;
        state.spec_dir = spec_dir;
    }

    /// Shutdown the server
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            info!("MCP server shutdown signal sent");
        }
    }
}

/// Handler for rstn_report_status tool
struct ReportStatusHandler {
    event_tx: mpsc::Sender<Event>,
}

#[async_trait::async_trait]
impl prism_mcp_rs::prelude::ToolHandler for ReportStatusHandler {
    async fn call(
        &self,
        arguments: std::collections::HashMap<String, serde_json::Value>,
    ) -> prism_mcp_rs::prelude::McpResult<prism_mcp_rs::prelude::ToolResult> {
        use prism_mcp_rs::prelude::*;

        // Extract status (required)
        let status = arguments
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::validation("Missing 'status' field"))?
            .to_string();

        // Validate status enum
        if !["needs_input", "completed", "error"].contains(&status.as_str()) {
            return Err(McpError::validation(
                "status must be 'needs_input', 'completed', or 'error'",
            ));
        }

        // Extract optional fields
        let prompt = arguments
            .get("prompt")
            .and_then(|v| v.as_str())
            .map(String::from);

        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Send event to TUI main loop
        self.event_tx
            .send(Event::McpStatus {
                status: status.clone(),
                prompt: prompt.clone(),
                message: message.clone(),
            })
            .await
            .map_err(|e| McpError::internal(format!("Failed to send event: {}", e)))?;

        info!("MCP tool rstn_report_status called: status={}", status);

        // Return success
        Ok(ToolResult {
            content: vec![ContentBlock::text(format!(
                "Status '{}' reported successfully",
                status
            ))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

/// Handler for rstn_read_spec tool
struct ReadSpecHandler {
    state: Arc<Mutex<McpState>>,
}

impl ReadSpecHandler {
    /// Map artifact name to filename
    fn artifact_to_filename(artifact: &str) -> Option<&'static str> {
        match artifact {
            "spec" => Some("spec.md"),
            "plan" => Some("plan.md"),
            "tasks" => Some("tasks.md"),
            "checklist" => Some("checklist.md"),
            "analysis" => Some("analysis.md"),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl prism_mcp_rs::prelude::ToolHandler for ReadSpecHandler {
    async fn call(
        &self,
        arguments: std::collections::HashMap<String, serde_json::Value>,
    ) -> prism_mcp_rs::prelude::McpResult<prism_mcp_rs::prelude::ToolResult> {
        use prism_mcp_rs::prelude::*;

        // Extract artifact (required)
        let artifact = arguments
            .get("artifact")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::validation("Missing 'artifact' field"))?;

        // Get filename
        let filename = Self::artifact_to_filename(artifact)
            .ok_or_else(|| McpError::validation(format!("Invalid artifact: {}", artifact)))?;

        // Get spec directory from state
        let state = self.state.lock().await;
        let spec_dir = state.spec_dir.clone();
        drop(state);

        let spec_dir = spec_dir.ok_or_else(|| {
            McpError::validation("No active feature. Run a spec phase first (e.g., specify, plan)")
        })?;

        // Build file path
        let file_path = std::path::PathBuf::from(&spec_dir).join(filename);

        // Read file
        let content = std::fs::read_to_string(&file_path).map_err(|e| {
            McpError::validation(format!(
                "Could not read {}: {}. File may not exist yet - try running the corresponding phase first.",
                artifact, e
            ))
        })?;

        info!(
            "MCP tool rstn_read_spec called: artifact={}, path={}",
            artifact,
            file_path.display()
        );

        // Return content
        Ok(ToolResult {
            content: vec![ContentBlock::text(content)],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

/// Handler for rstn_get_context tool
struct GetContextHandler {
    state: Arc<Mutex<McpState>>,
}

#[async_trait::async_trait]
impl prism_mcp_rs::prelude::ToolHandler for GetContextHandler {
    async fn call(
        &self,
        _arguments: std::collections::HashMap<String, serde_json::Value>,
    ) -> prism_mcp_rs::prelude::McpResult<prism_mcp_rs::prelude::ToolResult> {
        use prism_mcp_rs::prelude::*;

        // Get current context from state
        let state = self.state.lock().await;
        let context = FeatureContext {
            feature_number: state.feature_number.clone(),
            feature_name: state.feature_name.clone(),
            branch: state.branch.clone(),
            phase: state.phase.clone(),
            spec_dir: state.spec_dir.clone(),
        };
        drop(state);

        info!("MCP tool rstn_get_context called");

        // Serialize context to JSON string
        let context_json = serde_json::to_string_pretty(&context)
            .map_err(|e| McpError::internal(format!("Failed to serialize context: {}", e)))?;

        // Return as text content
        Ok(ToolResult {
            content: vec![ContentBlock::text(context_json)],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

/// Start the MCP server
///
/// # Arguments
/// * `config` - Server configuration
/// * `event_tx` - Channel to send events to the TUI
///
/// # Returns
/// A handle to control the server
pub async fn start_server(
    config: McpServerConfig,
    event_tx: mpsc::Sender<Event>,
) -> Result<McpServerHandle> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let state = Arc::new(Mutex::new(McpState::default()));
    let state_clone = state.clone();

    let port = config.port;
    let bind_addr = format!("127.0.0.1:{}", port);

    info!("Starting MCP server on {}", bind_addr);

    // Create HTTP MCP server
    let mut http_server = HttpMcpServer::new(config.name.clone(), config.version.clone());

    // Get the underlying McpServer to register tools
    let server_ref = http_server.server().await;
    {
        let server = server_ref.lock().await;

        // Initialize the server
        server.initialize().await.map_err(|e| anyhow::anyhow!("Failed to initialize MCP server: {}", e))?;

        // Register tools
        register_tools(&server, state_clone.clone(), event_tx.clone()).await?;
    }

    // Create HTTP transport
    let transport = HttpServerTransport::new(&bind_addr);

    // Spawn server task
    tokio::spawn(async move {
        // Start the HTTP server with transport
        if let Err(e) = http_server.start(transport).await {
            error!("MCP server failed to start: {}", e);
            return;
        }

        info!("MCP server started on {}", bind_addr);

        // Wait for shutdown signal
        let _ = shutdown_rx.await;

        // Stop the server
        if let Err(e) = http_server.stop().await {
            warn!("Error stopping MCP server: {}", e);
        }

        info!("MCP server stopped");
    });

    Ok(McpServerHandle {
        shutdown_tx: Some(shutdown_tx),
        port,
        state,
    })
}

/// Register all MCP tools
async fn register_tools(
    server: &McpServer,
    state: Arc<Mutex<McpState>>,
    event_tx: mpsc::Sender<Event>,
) -> Result<()> {
    // Register rstn_report_status tool (Feature 061)
    server
        .add_tool(
            "rstn_report_status",
            Some("Report current task status to rstn control plane"),
            status_tool_schema(),
            ReportStatusHandler { event_tx },
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register rstn_report_status: {}", e))?;

    info!("Registered MCP tool: rstn_report_status");

    // Register rstn_read_spec tool (Feature 062)
    server
        .add_tool(
            "rstn_read_spec",
            Some("Read a spec artifact for the current feature"),
            read_spec_tool_schema(),
            ReadSpecHandler {
                state: state.clone(),
            },
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register rstn_read_spec: {}", e))?;

    info!("Registered MCP tool: rstn_read_spec");

    // Register rstn_get_context tool (Feature 062)
    server
        .add_tool(
            "rstn_get_context",
            Some("Get current feature context and metadata"),
            get_context_tool_schema(),
            GetContextHandler { state },
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register rstn_get_context: {}", e))?;

    info!("Registered MCP tool: rstn_get_context");

    // Future tools (feature 063):
    // - rstn_complete_task (063): Mark tasks complete

    Ok(())
}

/// Write MCP configuration file for Claude Code to discover
pub fn write_mcp_config(port: u16) -> Result<std::path::PathBuf> {
    let config_dir = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".rstn");

    std::fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("mcp-session.json");
    let config = serde_json::json!({
        "mcpServers": {
            "rstn": {
                "transport": "sse",
                "url": format!("http://127.0.0.1:{}/sse", port)
            }
        }
    });

    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

    info!("Wrote MCP config to {:?}", config_path);
    Ok(config_path)
}

/// Remove MCP configuration file on shutdown
pub fn cleanup_mcp_config() -> Result<()> {
    let config_path = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".rstn")
        .join("mcp-session.json");

    if config_path.exists() {
        std::fs::remove_file(&config_path)?;
        info!("Removed MCP config from {:?}", config_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = McpServerConfig::default();
        assert_eq!(config.port, DEFAULT_MCP_PORT);
        assert_eq!(config.name, "rstn");
    }

    #[test]
    fn test_server_url() {
        let handle = McpServerHandle {
            shutdown_tx: None,
            port: 19560,
            state: Arc::new(Mutex::new(McpState::default())),
        };
        assert_eq!(handle.url(), "http://127.0.0.1:19560");
    }

    #[tokio::test]
    async fn test_state_update() {
        let handle = McpServerHandle {
            shutdown_tx: None,
            port: 19560,
            state: Arc::new(Mutex::new(McpState::default())),
        };

        handle
            .update_state(
                Some("060".to_string()),
                Some("mcp-server".to_string()),
                Some("060-mcp-server".to_string()),
                Some("implement".to_string()),
                Some("specs/060-mcp-server".to_string()),
            )
            .await;

        let state = handle.state.lock().await;
        assert_eq!(state.feature_number, Some("060".to_string()));
        assert_eq!(state.feature_name, Some("mcp-server".to_string()));
    }
}
