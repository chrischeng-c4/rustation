//! MCP HTTP Server implementation.
//!
//! Provides an embedded MCP (Model Context Protocol) HTTP server
//! that enables Claude Code to communicate with rstn via structured
//! tool calls.

use crate::mcp::types::{get_tools, JsonRpcRequest, JsonRpcResponse, McpServerConfig, ToolResult};
use crate::state::AppState;
use anyhow::{Context, Result};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

// ============================================================================
// Axum Application State
// ============================================================================

/// Shared application state for Axum handlers
#[derive(Clone)]
struct McpAppState {
    app_state: Arc<RwLock<AppState>>,
    server_name: String,
    server_version: String,
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// MCP endpoint - handles all JSON-RPC requests
async fn mcp_handler(
    State(state): State<McpAppState>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    info!("MCP request: method={}", request.method);

    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&state, &request).await,
        "tools/list" => handle_tools_list(&request).await,
        "tools/call" => handle_tools_call(&state, &request).await,
        _ => JsonRpcResponse::error(
            request.id.clone(),
            -32601,
            &format!("Method not found: {}", request.method),
        ),
    };

    Json(response)
}

/// Handle initialize method
async fn handle_initialize(state: &McpAppState, request: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": state.server_name,
                "version": state.server_version
            }
        }),
    )
}

/// Handle tools/list method
async fn handle_tools_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "tools": get_tools()
        }),
    )
}

/// Handle tools/call method
async fn handle_tools_call(state: &McpAppState, request: &JsonRpcRequest) -> JsonRpcResponse {
    let params = &request.params;

    let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments: HashMap<String, serde_json::Value> = params
        .get("arguments")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    info!("Tool call: {}", tool_name);

    let result = match tool_name {
        "rstn_get_app_state" => handle_get_app_state(state).await,
        "rstn_report_status" => handle_report_status(state, arguments).await,
        "rstn_read_spec" => handle_read_spec(state, arguments).await,
        "rstn_get_context" => handle_get_context(state).await,
        "rstn_complete_task" => handle_complete_task(state, arguments).await,
        _ => ToolResult::error(&format!("Unknown tool: {}", tool_name)),
    };

    JsonRpcResponse::success(request.id.clone(), serde_json::to_value(result).unwrap())
}

// ============================================================================
// Tool Handlers
// ============================================================================

/// Handle rstn_get_app_state tool
async fn handle_get_app_state(state: &McpAppState) -> ToolResult {
    let app_state = state.app_state.read().await;
    match serde_json::to_string_pretty(&*app_state) {
        Ok(json) => ToolResult::text(&json),
        Err(e) => ToolResult::error(&format!("Failed to serialize state: {}", e)),
    }
}

/// Handle rstn_report_status tool
async fn handle_report_status(
    state: &McpAppState,
    arguments: HashMap<String, serde_json::Value>,
) -> ToolResult {
    let status = match arguments.get("status").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return ToolResult::error("Missing 'status' field"),
    };

    if !["needs_input", "completed", "error"].contains(&status.as_str()) {
        return ToolResult::error("status must be 'needs_input', 'completed', or 'error'");
    }

    let prompt = arguments
        .get("prompt")
        .and_then(|v| v.as_str())
        .map(String::from);
    let message = arguments
        .get("message")
        .and_then(|v| v.as_str())
        .map(String::from);

    info!("rstn_report_status: status={}", status);

    // Update app state with MCP status
    {
        let mut app_state = state.app_state.write().await;
        app_state.mcp_status = Some(crate::state::McpStatusInfo {
            status: status.clone(),
            prompt,
            message,
        });
    }

    // TODO: For needs_input, implement blocking behavior with oneshot channel
    // For now, just report success
    ToolResult::text(&format!("Status '{}' reported successfully", status))
}

/// Handle rstn_read_spec tool
async fn handle_read_spec(
    state: &McpAppState,
    arguments: HashMap<String, serde_json::Value>,
) -> ToolResult {
    use crate::mcp::types::SpecArtifact;

    let artifact_str = match arguments.get("artifact").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return ToolResult::error("Missing 'artifact' field"),
    };

    let artifact = match SpecArtifact::from_str(artifact_str) {
        Some(a) => a,
        None => {
            return ToolResult::error_with_suggestion(
                &format!("Invalid artifact: {}", artifact_str),
                "Valid artifacts: spec, plan, tasks, checklist, analysis",
            )
        }
    };

    let spec_dir = {
        let app_state = state.app_state.read().await;
        app_state.feature_context.spec_dir.clone()
    };

    let spec_dir = match spec_dir {
        Some(d) => d,
        None => {
            return ToolResult::error_with_suggestion(
                "No active feature",
                "Select a feature from the worktree list first",
            )
        }
    };

    let file_path = std::path::PathBuf::from(&spec_dir).join(artifact.filename());

    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            info!(
                "rstn_read_spec: artifact={}, path={}",
                artifact_str,
                file_path.display()
            );
            ToolResult::text(&content)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let suggestion = match artifact {
                SpecArtifact::Spec => "Run /speckit.specify to generate spec first",
                SpecArtifact::Plan => "Run /speckit.plan to generate plan first",
                SpecArtifact::Tasks => "Run /speckit.tasks to generate tasks first",
                SpecArtifact::Checklist => "Run /speckit.specify to create a feature with checklist",
                SpecArtifact::Analysis => "Run /speckit.clarify to generate analysis first",
            };

            ToolResult::error_with_suggestion(&format!("{} not found", artifact.filename()), suggestion)
        }
        Err(e) => ToolResult::error(&format!("Could not read {}: {}", artifact_str, e)),
    }
}

/// Handle rstn_get_context tool
async fn handle_get_context(state: &McpAppState) -> ToolResult {
    let context = {
        let app_state = state.app_state.read().await;
        app_state.feature_context.clone()
    };

    match serde_json::to_string_pretty(&context) {
        Ok(json) => ToolResult::text(&json),
        Err(e) => ToolResult::error(&format!("Failed to serialize context: {}", e)),
    }
}

/// Handle rstn_complete_task tool
async fn handle_complete_task(
    _state: &McpAppState,
    arguments: HashMap<String, serde_json::Value>,
) -> ToolResult {
    let task_id = match arguments.get("task_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => return ToolResult::error("Missing 'task_id' field"),
    };

    info!("rstn_complete_task: task_id={}", task_id);

    // TODO: Implement actual task completion logic
    // For now, just acknowledge
    ToolResult::text(&format!(
        "Task {} marked for completion. Processing...",
        task_id
    ))
}

// ============================================================================
// Server Handle
// ============================================================================

/// Handle for controlling the MCP server
pub struct McpServerHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    port: u16,
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

    /// Get the MCP endpoint URL
    pub fn mcp_url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    /// Shutdown the server
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            info!("MCP server shutdown signal sent");
        }
    }
}

// ============================================================================
// Server Startup
// ============================================================================

/// Start the MCP server
pub async fn start_server(
    config: McpServerConfig,
    app_state: Arc<RwLock<AppState>>,
) -> Result<McpServerHandle> {
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

    // Bind to port (0 = auto-assign)
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind MCP server")?;
    let actual_port = listener.local_addr()?.port();

    info!("MCP server binding to port {}", actual_port);

    // Create app state
    let mcp_state = McpAppState {
        app_state,
        server_name: config.name,
        server_version: config.version,
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/mcp", post(mcp_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(mcp_state);

    // Spawn server task
    tokio::spawn(async move {
        info!("MCP server started on http://127.0.0.1:{}", actual_port);

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = (&mut shutdown_rx).await;
                info!("MCP server shutting down");
            })
            .await
            .unwrap_or_else(|e| error!("MCP server error: {}", e));

        info!("MCP server stopped");
    });

    Ok(McpServerHandle {
        shutdown_tx: Some(shutdown_tx),
        port: actual_port,
    })
}

// ============================================================================
// Config File Management
// ============================================================================

/// Write MCP configuration file for Claude Code to discover
pub fn write_mcp_config(port: u16) -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let config_path = home.join(".rstn").join("mcp-session.json");

    let config_dir = config_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid config path"))?;

    std::fs::create_dir_all(config_dir)?;

    let config = serde_json::json!({
        "mcpServers": {
            "rstn": {
                "type": "http",
                "url": format!("http://127.0.0.1:{}/mcp", port)
            }
        }
    });

    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

    info!("Wrote MCP config to {:?}", config_path);
    Ok(config_path)
}

/// Remove MCP configuration file on shutdown
pub fn cleanup_mcp_config() -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let config_path = home.join(".rstn").join("mcp-session.json");

    if config_path.exists() {
        std::fs::remove_file(&config_path)?;
        info!("Removed MCP config from {:?}", config_path);
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = McpServerConfig::default();
        assert_eq!(config.port, 0);
        assert_eq!(config.name, "rstn");
    }

    #[test]
    fn test_server_url() {
        let handle = McpServerHandle {
            shutdown_tx: None,
            port: 19560,
        };
        assert_eq!(handle.url(), "http://127.0.0.1:19560");
        assert_eq!(handle.mcp_url(), "http://127.0.0.1:19560/mcp");
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(
            Some(serde_json::json!(1)),
            serde_json::json!({"result": "ok"}),
        );
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response =
            JsonRpcResponse::error(Some(serde_json::json!(1)), -32600, "Invalid Request");
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32600);
    }
}
