//! MCP Server implementation for rstn
//!
//! Provides an embedded MCP (Model Context Protocol) server that enables
//! Claude Code to communicate with rstn via structured tool calls instead
//! of fragile text parsing.
//!
//! Architecture:
//! - Uses HTTP transport (recommended by Claude Code, SSE is deprecated)
//! - Runs on localhost with dynamic port allocation (port 0)
//! - Sends events to TUI via shared McpState
//! - Supports tool registration for status reporting, spec reading, etc.

use crate::tui::event::Event;
use anyhow::{Context, Result};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

/// Default port for MCP server (0 = auto-assign)
pub const DEFAULT_MCP_PORT: u16 = 0;

/// MCP server configuration
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Port to listen on (0 for auto-assign)
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

/// Event recorded for MCP server activity tracking
#[derive(Debug, Clone)]
pub struct McpEvent {
    /// Timestamp when event occurred
    pub timestamp: std::time::Instant,
    /// Event type: "STATUS", "READ", "CONTEXT", "TASK"
    pub event_type: String,
    /// Event details/message
    pub details: String,
}

/// Shared state accessible by tool handlers
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
    /// Server start time for uptime calculation
    pub server_start_time: std::time::Instant,
    /// Count of calls per tool
    pub tool_call_counts: HashMap<String, usize>,
    /// Last tool call (tool name, timestamp)
    pub last_tool_call: Option<(String, std::time::Instant)>,
    /// Recent events (max 50)
    pub recent_events: std::collections::VecDeque<McpEvent>,
    /// Pending events for TUI to process (polled by main loop)
    pub pending_tui_events: std::collections::VecDeque<Event>,
    /// Sender for user input response (set when needs_input, cleared when response received)
    pub input_response_tx: Option<oneshot::Sender<String>>,
}

impl std::fmt::Debug for McpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpState")
            .field("feature_number", &self.feature_number)
            .field("feature_name", &self.feature_name)
            .field("branch", &self.branch)
            .field("phase", &self.phase)
            .field("spec_dir", &self.spec_dir)
            .field("server_start_time", &self.server_start_time)
            .field("tool_call_counts", &self.tool_call_counts)
            .field("last_tool_call", &self.last_tool_call)
            .field("recent_events", &self.recent_events)
            .field("pending_tui_events", &self.pending_tui_events.len())
            .field("input_response_tx", &self.input_response_tx.is_some())
            .finish()
    }
}

impl Default for McpState {
    fn default() -> Self {
        Self {
            feature_number: None,
            feature_name: None,
            branch: None,
            phase: None,
            spec_dir: None,
            server_start_time: std::time::Instant::now(),
            tool_call_counts: HashMap::new(),
            last_tool_call: None,
            recent_events: std::collections::VecDeque::new(),
            pending_tui_events: std::collections::VecDeque::new(),
            input_response_tx: None,
        }
    }
}

impl McpState {
    /// Send user input response (called by TUI when user submits)
    pub fn send_input_response(&mut self, response: String) -> bool {
        if let Some(tx) = self.input_response_tx.take() {
            tx.send(response).is_ok()
        } else {
            false
        }
    }

    /// Push a TUI event for polling by main loop
    pub fn push_tui_event(&mut self, event: Event) {
        self.pending_tui_events.push_back(event);
    }

    /// Drain all pending TUI events
    pub fn drain_tui_events(&mut self) -> Vec<Event> {
        self.pending_tui_events.drain(..).collect()
    }

    /// Record a tool call for metrics
    fn record_tool_call(&mut self, tool_name: &str, details: &str, event_type: &str) {
        *self
            .tool_call_counts
            .entry(tool_name.to_string())
            .or_insert(0) += 1;
        self.last_tool_call = Some((tool_name.to_string(), std::time::Instant::now()));
        self.recent_events.push_back(McpEvent {
            timestamp: std::time::Instant::now(),
            event_type: event_type.to_string(),
            details: details.to_string(),
        });
        if self.recent_events.len() > 50 {
            self.recent_events.pop_front();
        }
    }
}

// ============================================================================
// Global MCP State (shared between TUI and CLI modes)
// ============================================================================

use once_cell::sync::Lazy;

/// Global MCP state storage (shared between TUI and CLI modes)
///
/// This allows CLI mode to access the same MCP state that TUI mode uses,
/// enabling interactive prompts via Mini TUI mode.
///
/// Note: Wraps tokio::sync::Mutex in std::sync::Mutex for static storage
static GLOBAL_MCP_STATE_MUT: Lazy<std::sync::Mutex<Option<Arc<Mutex<McpState>>>>> =
    Lazy::new(|| std::sync::Mutex::new(None));

/// Get the global MCP state (for CLI mode integration)
///
/// Returns None if MCP server hasn't been initialized yet.
pub fn get_global_mcp_state() -> Option<Arc<Mutex<McpState>>> {
    let guard = GLOBAL_MCP_STATE_MUT.lock().ok()?;
    guard.as_ref().cloned()
}

/// Initialize the global MCP state (called from main.rs)
///
/// This should be called after creating the MCP state in main.rs,
/// so both TUI and CLI modes can access the same state.
pub fn init_global_mcp_state(state: Arc<Mutex<McpState>>) {
    if let Ok(mut guard) = GLOBAL_MCP_STATE_MUT.lock() {
        *guard = Some(state);
    }
}

// ============================================================================
// JSON-RPC Types for MCP Protocol
// ============================================================================

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Option<serde_json::Value>, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}

// ============================================================================
// MCP Protocol Types
// ============================================================================

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// MCP Content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

/// MCP Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ContentBlock>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolResult {
    fn text(message: &str) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: message.to_string(),
            }],
            is_error: Some(false),
        }
    }

    fn error(message: &str) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: message.to_string(),
            }],
            is_error: Some(true),
        }
    }

    /// Create an error result with a suggestion
    /// Format: "Error: {message} | Suggestion: {suggestion}"
    fn error_with_suggestion(message: &str, suggestion: &str) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: format!("{} | Suggestion: {}", message, suggestion),
            }],
            is_error: Some(true),
        }
    }
}

// ============================================================================
// Tool Argument Types
// ============================================================================

/// Arguments for rstn_report_status tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStatusArgs {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Arguments for rstn_read_spec tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadSpecArgs {
    pub artifact: String,
}

/// Feature context response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContext {
    pub feature_number: Option<String>,
    pub feature_name: Option<String>,
    pub branch: Option<String>,
    pub phase: Option<String>,
    pub spec_dir: Option<String>,
}

/// Arguments for rstn_complete_task tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTaskArgs {
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_validation: Option<bool>,
}

// ============================================================================
// Tool Schemas
// ============================================================================

fn get_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "rstn_report_status".to_string(),
            description: Some("Report current task status to rstn control plane. For needs_input status, this tool blocks until user provides input.".to_string()),
            input_schema: serde_json::json!({
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
            }),
        },
        McpTool {
            name: "rstn_read_spec".to_string(),
            description: Some("Read a spec artifact for the current feature".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "artifact": {
                        "type": "string",
                        "enum": ["spec", "plan", "tasks", "checklist", "analysis"],
                        "description": "Which artifact to read"
                    }
                },
                "required": ["artifact"]
            }),
        },
        McpTool {
            name: "rstn_get_context".to_string(),
            description: Some("Get current feature context and metadata".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "rstn_complete_task".to_string(),
            description: Some("Mark a task as complete with validation".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to mark complete (e.g., T001, T002)"
                    },
                    "skip_validation": {
                        "type": "boolean",
                        "description": "Skip validation checks (optional)"
                    }
                },
                "required": ["task_id"]
            }),
        },
    ]
}

// ============================================================================
// Axum Application State
// ============================================================================

/// Shared application state for Axum handlers
#[derive(Clone)]
struct AppState {
    mcp_state: Arc<Mutex<McpState>>,
    event_tx: mpsc::Sender<Event>,
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
    State(state): State<AppState>,
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
async fn handle_initialize(state: &AppState, request: &JsonRpcRequest) -> JsonRpcResponse {
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
async fn handle_tools_call(state: &AppState, request: &JsonRpcRequest) -> JsonRpcResponse {
    let params = &request.params;

    let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments: HashMap<String, serde_json::Value> = params
        .get("arguments")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    info!("Tool call: {}", tool_name);

    let result = match tool_name {
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

/// Handle rstn_report_status tool
async fn handle_report_status(
    state: &AppState,
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

    match status.as_str() {
        "needs_input" => {
            // Create oneshot channel for response
            let (tx, rx) = oneshot::channel();

            // Store sender and push event
            {
                let mut mcp_state = state.mcp_state.lock().await;
                mcp_state.record_tool_call("rstn_report_status", "needs_input", "STATUS");
                mcp_state.input_response_tx = Some(tx);
                mcp_state.push_tui_event(Event::McpStatus {
                    status: status.clone(),
                    prompt: prompt.clone(),
                    message: message.clone(),
                });
            }

            info!("Waiting for user input response...");

            // Block until user responds
            match rx.await {
                Ok(response) => {
                    info!("Got user input: {}", response);
                    ToolResult::text(&format!("User response: {}", response))
                }
                Err(_) => {
                    warn!("Input request cancelled");
                    ToolResult::error("Input request was cancelled")
                }
            }
        }
        _ => {
            // For completed/error, push event and return immediately
            {
                let mut mcp_state = state.mcp_state.lock().await;
                mcp_state.record_tool_call("rstn_report_status", &status, "STATUS");
                mcp_state.push_tui_event(Event::McpStatus {
                    status: status.clone(),
                    prompt,
                    message,
                });
            }

            ToolResult::text(&format!("Status '{}' reported successfully", status))
        }
    }
}

/// Handle rstn_read_spec tool
async fn handle_read_spec(
    state: &AppState,
    arguments: HashMap<String, serde_json::Value>,
) -> ToolResult {
    let artifact = match arguments.get("artifact").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return ToolResult::error("Missing 'artifact' field"),
    };

    let filename = match artifact {
        "spec" => "spec.md",
        "plan" => "plan.md",
        "tasks" => "tasks.md",
        "checklist" => "checklist.md",
        "analysis" => "analysis.md",
        _ => {
            return ToolResult::error_with_suggestion(
                &format!("Invalid artifact: {}", artifact),
                "Valid artifacts: spec, plan, tasks, checklist, analysis",
            )
        }
    };

    let spec_dir = {
        let mut mcp_state = state.mcp_state.lock().await;
        mcp_state.record_tool_call("rstn_read_spec", &format!("{}.md", artifact), "READ");
        mcp_state.spec_dir.clone()
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

    let file_path = std::path::PathBuf::from(&spec_dir).join(filename);

    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            info!(
                "rstn_read_spec: artifact={}, path={}",
                artifact,
                file_path.display()
            );
            ToolResult::text(&content)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Provide specific suggestion based on artifact type
            let suggestion = match artifact {
                "spec" => "Run /speckit.specify to generate spec first",
                "plan" => "Run /speckit.plan to generate plan first",
                "tasks" => "Run /speckit.tasks to generate tasks first",
                "checklist" => "Run /speckit.specify to create a feature with checklist",
                "analysis" => "Run /speckit.clarify to generate analysis first",
                _ => "Check if the file exists in the spec directory",
            };

            ToolResult::error_with_suggestion(
                &format!("{} not found", filename),
                suggestion,
            )
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            ToolResult::error_with_suggestion(
                &format!("Permission denied reading {}", filename),
                "Check file permissions or run with appropriate access rights",
            )
        }
        Err(e) => {
            ToolResult::error(&format!("Could not read {}: {}", artifact, e))
        }
    }
}

/// Handle rstn_get_context tool
async fn handle_get_context(state: &AppState) -> ToolResult {
    let context = {
        let mut mcp_state = state.mcp_state.lock().await;
        let feature_desc = mcp_state
            .feature_number
            .as_ref()
            .map(|n| format!("Feature {}", n))
            .unwrap_or_else(|| "No feature".to_string());
        mcp_state.record_tool_call("rstn_get_context", &feature_desc, "CONTEXT");

        FeatureContext {
            feature_number: mcp_state.feature_number.clone(),
            feature_name: mcp_state.feature_name.clone(),
            branch: mcp_state.branch.clone(),
            phase: mcp_state.phase.clone(),
            spec_dir: mcp_state.spec_dir.clone(),
        }
    };

    info!("rstn_get_context called");

    match serde_json::to_string_pretty(&context) {
        Ok(json) => ToolResult::text(&json),
        Err(e) => ToolResult::error(&format!("Failed to serialize context: {}", e)),
    }
}

/// Handle rstn_complete_task tool
async fn handle_complete_task(
    state: &AppState,
    arguments: HashMap<String, serde_json::Value>,
) -> ToolResult {
    let task_id = match arguments.get("task_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => return ToolResult::error("Missing 'task_id' field"),
    };

    // Record metrics
    {
        let mut mcp_state = state.mcp_state.lock().await;
        mcp_state.record_tool_call(
            "rstn_complete_task",
            &format!("{} completed", task_id),
            "TASK",
        );
    }

    // Send event to TUI
    if let Err(e) = state
        .event_tx
        .send(Event::McpTaskCompleted {
            task_id: task_id.clone(),
            success: true,
            message: format!("Task {} completion requested", task_id),
        })
        .await
    {
        error!("Failed to send task completion event: {}", e);
        return ToolResult::error(&format!("Failed to send event: {}", e));
    }

    info!("rstn_complete_task: task_id={}", task_id);

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

// ============================================================================
// Server Startup
// ============================================================================

/// Start the MCP server
pub async fn start_server(
    config: McpServerConfig,
    event_tx: mpsc::Sender<Event>,
    state: Arc<Mutex<McpState>>,
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
    let app_state = AppState {
        mcp_state: state.clone(),
        event_tx,
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
        .with_state(app_state);

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
        state,
    })
}

// ============================================================================
// Config File Management
// ============================================================================

/// Write MCP configuration file for Claude Code to discover
pub fn write_mcp_config(port: u16) -> Result<std::path::PathBuf> {
    let config_path =
        crate::domain::paths::mcp_config_path().context("Could not determine MCP config path")?;

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
    let config_path =
        crate::domain::paths::mcp_config_path().context("Could not determine MCP config path")?;

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

    #[test]
    fn test_tool_result_text() {
        let result = ToolResult::text("Hello");
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Oops");
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn test_tool_result_error_with_suggestion() {
        let result = ToolResult::error_with_suggestion(
            "spec.md not found",
            "Run /speckit.specify to generate spec first",
        );
        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.content.len(), 1);

        if let ContentBlock::Text { text } = &result.content[0] {
            assert!(
                text.contains(" | Suggestion: "),
                "Should contain suggestion separator"
            );
            assert!(
                text.contains("spec.md not found"),
                "Should contain error message"
            );
            assert!(
                text.contains("Run /speckit.specify"),
                "Should contain suggestion"
            );
        } else {
            panic!("Expected text content block");
        }
    }

    #[test]
    fn test_get_tools() {
        let tools = get_tools();
        assert_eq!(tools.len(), 4);
        assert_eq!(tools[0].name, "rstn_report_status");
        assert_eq!(tools[1].name, "rstn_read_spec");
        assert_eq!(tools[2].name, "rstn_get_context");
        assert_eq!(tools[3].name, "rstn_complete_task");
    }
}
