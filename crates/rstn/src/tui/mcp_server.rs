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
    _event_tx: mpsc::Sender<Event>,
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

        // Register tools (empty for now - will be added in feature 061-063)
        // This is where rstn_report_status, rstn_read_spec, etc. will be registered
        register_tools(&server, state_clone.clone()).await?;
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
    _server: &McpServer,
    _state: Arc<Mutex<McpState>>,
) -> Result<()> {
    // Tools will be registered in feature 061-063:
    // - rstn_report_status (061): Report status changes
    // - rstn_read_spec (062): Read spec artifacts
    // - rstn_get_context (062): Get feature context
    // - rstn_complete_task (063): Mark tasks complete

    info!("MCP tools registered (none yet - pending features 061-063)");
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
