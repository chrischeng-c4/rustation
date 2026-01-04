//! MCP Server implementation for rustation.
//!
//! Provides an embedded Model Context Protocol server that exposes
//! project-specific context to AI clients (Claude Desktop, Claude Code).
//!
//! Uses axum for HTTP with SSE transport, implementing MCP JSON-RPC protocol.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

// Note: McpState and McpStatus are defined in app_state.rs

// ============================================================================
// JSON-RPC Types (MCP Protocol)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// MCP Tool Definitions
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

fn get_available_tools() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            name: "read_file".to_string(),
            description: "Read the contents of a file within the worktree".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file (relative to worktree root)"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolInfo {
            name: "list_directory".to_string(),
            description: "List files and directories within the worktree".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory (relative to worktree root)"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolInfo {
            name: "get_project_context".to_string(),
            description: "Get high-level project context information".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolInfo {
            name: "run_just_task".to_string(),
            description: "Run a Just task and return the output".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_name": {
                        "type": "string",
                        "description": "Name of the Just task to run"
                    }
                },
                "required": ["task_name"]
            }),
        },
        // ====================================================================
        // ReviewGate Tools (CESDD ReviewGate Layer)
        // ====================================================================
        ToolInfo {
            name: "submit_for_review".to_string(),
            description: "Submit content for human review. Returns a session_id that can be used to check feedback and update content.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "workflow_node_id": {
                        "type": "string",
                        "description": "ID of the workflow node triggering this review"
                    },
                    "content_type": {
                        "type": "string",
                        "enum": ["Plan", "Proposal", "Code", "Artifact"],
                        "description": "Type of content being reviewed"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to be reviewed (markdown format)"
                    },
                    "file_changes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" },
                                "action": { "type": "string", "enum": ["create", "modify", "delete"] },
                                "summary": { "type": "string" }
                            },
                            "required": ["path", "action", "summary"]
                        },
                        "description": "List of file changes (optional)"
                    },
                    "policy": {
                        "type": "string",
                        "enum": ["AutoApprove", "AgentDecides", "AlwaysReview"],
                        "description": "Review policy (default: AlwaysReview)"
                    }
                },
                "required": ["workflow_node_id", "content_type", "content"]
            }),
        },
        ToolInfo {
            name: "get_review_feedback".to_string(),
            description: "Get user feedback (unresolved comments) for a review session. Call this after submitting for review to see if user has provided feedback.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "The review session ID returned by submit_for_review"
                    }
                },
                "required": ["session_id"]
            }),
        },
        ToolInfo {
            name: "update_review_content".to_string(),
            description: "Update the review content after addressing user feedback. This increments the iteration count and moves status back to 'reviewing'.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "The review session ID"
                    },
                    "content": {
                        "type": "string",
                        "description": "Updated content (markdown format)"
                    },
                    "file_changes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" },
                                "action": { "type": "string", "enum": ["create", "modify", "delete"] },
                                "summary": { "type": "string" }
                            },
                            "required": ["path", "action", "summary"]
                        },
                        "description": "Updated list of file changes (optional)"
                    }
                },
                "required": ["session_id", "content"]
            }),
        },
    ]
}

// ============================================================================
// MCP Server Context
// ============================================================================

/// Context for the MCP server instance
#[derive(Clone)]
pub struct McpServerContext {
    /// Root path of the worktree (for sandboxing)
    pub worktree_root: PathBuf,
    /// Worktree ID
    pub worktree_id: String,
    /// Project name
    pub project_name: String,
}

impl McpServerContext {
    /// Validate that a path is within the worktree root (security sandbox)
    fn validate_path(&self, relative_path: &str) -> Result<PathBuf, String> {
        let full_path = self.worktree_root.join(relative_path);

        // Canonicalize to resolve .. and symlinks
        let canonical = full_path
            .canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;

        // Ensure the path is within the worktree root
        let root_canonical = self
            .worktree_root
            .canonicalize()
            .map_err(|e| format!("Invalid worktree root: {}", e))?;

        if !canonical.starts_with(&root_canonical) {
            return Err("Access denied: path is outside worktree".to_string());
        }

        Ok(canonical)
    }

    /// Execute a tool and return the result
    async fn execute_tool(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match tool_name {
            "read_file" => {
                let path = params
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'path' parameter")?;

                let full_path = self.validate_path(path)?;
                let content = tokio::fs::read_to_string(&full_path)
                    .await
                    .map_err(|e| format!("Failed to read file: {}", e))?;

                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": content
                    }]
                }))
            }

            "list_directory" => {
                let path = params
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");

                let full_path = self.validate_path(path)?;
                let mut entries = Vec::new();

                let mut read_dir = tokio::fs::read_dir(&full_path)
                    .await
                    .map_err(|e| format!("Failed to read directory: {}", e))?;

                while let Some(entry) = read_dir
                    .next_entry()
                    .await
                    .map_err(|e| format!("Failed to read entry: {}", e))?
                {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Skip hidden files and common ignore patterns
                    if name.starts_with('.') || name == "node_modules" || name == "target" {
                        continue;
                    }

                    let file_type = entry
                        .file_type()
                        .await
                        .map_err(|e| format!("Failed to get file type: {}", e))?;

                    let entry_type = if file_type.is_dir() { "directory" } else { "file" };
                    entries.push(serde_json::json!({
                        "name": name,
                        "type": entry_type
                    }));
                }

                entries.sort_by(|a, b| {
                    a.get("name")
                        .and_then(|v| v.as_str())
                        .cmp(&b.get("name").and_then(|v| v.as_str()))
                });

                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&entries).unwrap()
                    }]
                }))
            }

            "get_project_context" => {
                let context = serde_json::json!({
                    "project_name": self.project_name,
                    "worktree_id": self.worktree_id,
                    "worktree_root": self.worktree_root.display().to_string(),
                });

                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&context).unwrap()
                    }]
                }))
            }

            "run_just_task" => {
                let task_name = params
                    .get("task_name")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'task_name' parameter")?;

                let output = tokio::process::Command::new("just")
                    .arg(task_name)
                    .current_dir(&self.worktree_root)
                    .output()
                    .await
                    .map_err(|e| format!("Failed to run just task: {}", e))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    Ok(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": stdout.to_string()
                        }]
                    }))
                } else {
                    Err(format!("Task failed:\nstdout: {}\nstderr: {}", stdout, stderr))
                }
            }

            // ================================================================
            // ReviewGate Tools
            // ================================================================
            "submit_for_review" => {
                let workflow_node_id = params
                    .get("workflow_node_id")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'workflow_node_id' parameter")?
                    .to_string();

                let content_type_str = params
                    .get("content_type")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'content_type' parameter")?;

                let content = params
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'content' parameter")?
                    .to_string();

                let policy_str = params
                    .get("policy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("AlwaysReview");

                // Parse file_changes if provided
                let file_changes: Vec<crate::actions::ReviewFileChangeData> = params
                    .get("file_changes")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|fc| {
                                let path = fc.get("path")?.as_str()?.to_string();
                                let action_str = fc.get("action")?.as_str()?;
                                let summary = fc.get("summary")?.as_str()?.to_string();
                                let action = match action_str {
                                    "create" => crate::actions::ReviewFileActionData::Create,
                                    "modify" => crate::actions::ReviewFileActionData::Modify,
                                    "delete" => crate::actions::ReviewFileActionData::Delete,
                                    _ => return None,
                                };
                                Some(crate::actions::ReviewFileChangeData {
                                    path,
                                    action,
                                    summary,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Parse content_type
                let content_type = match content_type_str {
                    "Plan" => crate::actions::ReviewContentTypeData::Plan,
                    "Proposal" => crate::actions::ReviewContentTypeData::Proposal,
                    "Code" => crate::actions::ReviewContentTypeData::Code,
                    "Artifact" => crate::actions::ReviewContentTypeData::Artifact,
                    _ => return Err(format!("Invalid content_type: {}", content_type_str)),
                };

                // Parse policy
                let policy = match policy_str {
                    "AutoApprove" => crate::actions::ReviewPolicyData::AutoApprove,
                    "AgentDecides" => crate::actions::ReviewPolicyData::AgentDecides,
                    "AlwaysReview" => crate::actions::ReviewPolicyData::AlwaysReview,
                    _ => crate::actions::ReviewPolicyData::AlwaysReview,
                };

                // Create review content
                let review_content = crate::actions::ReviewContentData {
                    content_type,
                    content,
                    file_changes,
                };

                // Dispatch StartReview action
                let action = crate::actions::Action::StartReview {
                    workflow_node_id,
                    content: review_content,
                    policy,
                };

                {
                    let mut state = crate::get_app_state().write().await;
                    crate::reducer::reduce(&mut state, action);
                }

                // Get the session_id that was just created
                let session_id = {
                    let state = crate::get_app_state().read().await;
                    state
                        .active_project()
                        .and_then(|p| p.active_worktree())
                        .and_then(|w| w.tasks.review_gate.active_session_id.clone())
                        .ok_or("Failed to create review session")?
                };

                // Notify state update
                crate::notify_state_update().await;

                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Review session created. Session ID: {}\n\nThe content has been submitted for human review. Call get_review_feedback with this session_id to check for user feedback.", session_id)
                    }],
                    "session_id": session_id
                }))
            }

            "get_review_feedback" => {
                let session_id = params
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'session_id' parameter")?;

                let state = crate::get_app_state().read().await;
                let session = state
                    .active_project()
                    .and_then(|p| p.active_worktree())
                    .and_then(|w| w.tasks.review_gate.sessions.get(session_id))
                    .ok_or("Review session not found")?;

                // Get session status
                let status = match session.status {
                    crate::app_state::ReviewStatus::Pending => "pending",
                    crate::app_state::ReviewStatus::Reviewing => "reviewing",
                    crate::app_state::ReviewStatus::Iterating => "iterating",
                    crate::app_state::ReviewStatus::Approved => "approved",
                    crate::app_state::ReviewStatus::Rejected => "rejected",
                };

                // Get unresolved comments
                let unresolved_comments: Vec<_> = session
                    .comments
                    .iter()
                    .filter(|c| !c.resolved)
                    .map(|c| {
                        let target = match &c.target {
                            crate::app_state::CommentTarget::Document => "document".to_string(),
                            crate::app_state::CommentTarget::Section { id } => {
                                format!("section:{}", id)
                            }
                            crate::app_state::CommentTarget::File { path } => {
                                format!("file:{}", path)
                            }
                        };
                        serde_json::json!({
                            "id": c.id,
                            "target": target,
                            "content": c.content,
                            "author": match c.author {
                                crate::app_state::CommentAuthor::User => "user",
                                crate::app_state::CommentAuthor::System => "system",
                            }
                        })
                    })
                    .collect();

                let response = serde_json::json!({
                    "session_id": session_id,
                    "status": status,
                    "iteration": session.iteration,
                    "unresolved_comments": unresolved_comments,
                    "total_comments": session.comments.len(),
                });

                let message = if status == "approved" {
                    "Review has been APPROVED. You may proceed with implementing the plan.".to_string()
                } else if status == "rejected" {
                    "Review has been REJECTED. Check the comments for the rejection reason.".to_string()
                } else if unresolved_comments.is_empty() {
                    format!("No feedback yet. Status: {}. The user is still reviewing.", status)
                } else {
                    format!(
                        "Found {} unresolved comment(s). Address the feedback and call update_review_content.",
                        unresolved_comments.len()
                    )
                };

                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("{}\n\n{}", message, serde_json::to_string_pretty(&response).unwrap())
                    }],
                    "feedback": response
                }))
            }

            "update_review_content" => {
                let session_id = params
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'session_id' parameter")?
                    .to_string();

                let content = params
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'content' parameter")?
                    .to_string();

                // Parse file_changes if provided
                let file_changes: Vec<crate::actions::ReviewFileChangeData> = params
                    .get("file_changes")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|fc| {
                                let path = fc.get("path")?.as_str()?.to_string();
                                let action_str = fc.get("action")?.as_str()?;
                                let summary = fc.get("summary")?.as_str()?.to_string();
                                let action = match action_str {
                                    "create" => crate::actions::ReviewFileActionData::Create,
                                    "modify" => crate::actions::ReviewFileActionData::Modify,
                                    "delete" => crate::actions::ReviewFileActionData::Delete,
                                    _ => return None,
                                };
                                Some(crate::actions::ReviewFileChangeData {
                                    path,
                                    action,
                                    summary,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Get the current content_type from the session
                let content_type = {
                    let state = crate::get_app_state().read().await;
                    let session = state
                        .active_project()
                        .and_then(|p| p.active_worktree())
                        .and_then(|w| w.tasks.review_gate.sessions.get(&session_id))
                        .ok_or("Review session not found")?;

                    match session.content.content_type {
                        crate::app_state::ReviewContentType::Plan => {
                            crate::actions::ReviewContentTypeData::Plan
                        }
                        crate::app_state::ReviewContentType::Proposal => {
                            crate::actions::ReviewContentTypeData::Proposal
                        }
                        crate::app_state::ReviewContentType::Code => {
                            crate::actions::ReviewContentTypeData::Code
                        }
                        crate::app_state::ReviewContentType::Artifact => {
                            crate::actions::ReviewContentTypeData::Artifact
                        }
                    }
                };

                // Dispatch UpdateReviewContent action
                let action = crate::actions::Action::UpdateReviewContent {
                    session_id: session_id.clone(),
                    content: crate::actions::ReviewContentData {
                        content_type,
                        content,
                        file_changes,
                    },
                };

                {
                    let mut state = crate::get_app_state().write().await;
                    crate::reducer::reduce(&mut state, action);
                }

                // Get the new iteration count
                let iteration = {
                    let state = crate::get_app_state().read().await;
                    state
                        .active_project()
                        .and_then(|p| p.active_worktree())
                        .and_then(|w| w.tasks.review_gate.sessions.get(&session_id))
                        .map(|s| s.iteration)
                        .unwrap_or(0)
                };

                // Notify state update
                crate::notify_state_update().await;

                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Review content updated. Now on iteration {}. The user will review the updated content.", iteration)
                    }],
                    "session_id": session_id,
                    "iteration": iteration
                }))
            }

            _ => Err(format!("Unknown tool: {}", tool_name)),
        }
    }
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Handle MCP JSON-RPC requests
async fn handle_mcp_request(
    State(context): State<Arc<McpServerContext>>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let result = match request.method.as_str() {
        "initialize" => {
            Ok(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": format!("rustation-{}", context.project_name),
                    "version": env!("CARGO_PKG_VERSION")
                }
            }))
        }

        "tools/list" => {
            let tools = get_available_tools();
            Ok(serde_json::json!({
                "tools": tools
            }))
        }

        "tools/call" => {
            let tool_name = request
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let arguments = request
                .params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            context.execute_tool(tool_name, &arguments).await
        }

        "notifications/initialized" => {
            // Client is ready, just acknowledge
            Ok(serde_json::json!({}))
        }

        _ => Err(format!("Unknown method: {}", request.method)),
    };

    let response = match result {
        Ok(result) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(result),
            error: None,
        },
        Err(message) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message,
                data: None,
            }),
        },
    };

    Json(response)
}

/// SSE endpoint for MCP streaming
async fn handle_sse(
    State(_context): State<Arc<McpServerContext>>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = async_stream::stream! {
        // Send initial connection event
        yield Ok(Event::default().data("connected"));

        // Keep connection alive with periodic pings
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            yield Ok(Event::default().event("ping").data(""));
        }
    };

    Sse::new(stream)
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "rustation-mcp"
    }))
}

// ============================================================================
// Server Management
// ============================================================================

/// Running server instance
pub struct RunningServer {
    /// Cancellation token to stop the server
    pub cancel_token: CancellationToken,
    /// Port the server is listening on
    pub port: u16,
    /// Handle to the server task
    pub handle: tokio::task::JoinHandle<()>,
}

/// Manager for MCP server instances (one per worktree)
pub struct McpServerManager {
    /// Map of worktree_id -> running server
    servers: RwLock<HashMap<String, RunningServer>>,
}

impl Default for McpServerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServerManager {
    pub fn new() -> Self {
        Self {
            servers: RwLock::new(HashMap::new()),
        }
    }

    /// Start an MCP server for a worktree
    pub async fn start_server(
        &self,
        worktree_id: String,
        worktree_root: PathBuf,
        project_name: String,
        preferred_port: Option<u16>,
    ) -> Result<u16, String> {
        // Check if server is already running
        {
            let servers = self.servers.read().await;
            if servers.contains_key(&worktree_id) {
                return Err("Server already running for this worktree".to_string());
            }
        }

        // Create the MCP server context
        let context = Arc::new(McpServerContext {
            worktree_root,
            worktree_id: worktree_id.clone(),
            project_name,
        });

        // Find an available port
        let port = preferred_port.unwrap_or(3000);
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        // Try to bind to the port (or find next available)
        let listener = Self::try_bind_port(addr, port).await?;
        let actual_port = listener.local_addr().unwrap().port();

        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();

        // Build the router
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/sse", get(handle_sse))
            .route("/mcp", post(handle_mcp_request))
            .with_state(context)
            .layer(
                tower_http::cors::CorsLayer::new()
                    .allow_origin(tower_http::cors::Any)
                    .allow_methods(tower_http::cors::Any)
                    .allow_headers(tower_http::cors::Any),
            );

        // Spawn the server task
        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    cancel_clone.cancelled().await;
                })
                .await
                .ok();
        });

        // Store the running server
        {
            let mut servers = self.servers.write().await;
            servers.insert(
                worktree_id,
                RunningServer {
                    cancel_token,
                    port: actual_port,
                    handle,
                },
            );
        }

        Ok(actual_port)
    }

    async fn try_bind_port(addr: SocketAddr, preferred_port: u16) -> Result<TcpListener, String> {
        if let Ok(listener) = TcpListener::bind(addr).await {
            return Ok(listener);
        }

        // Try ports preferred_port+1 to preferred_port+10
        for p in (preferred_port + 1)..=(preferred_port + 10) {
            let addr = SocketAddr::from(([127, 0, 0, 1], p));
            if let Ok(listener) = TcpListener::bind(addr).await {
                return Ok(listener);
            }
        }

        Err("No available ports".to_string())
    }

    /// Stop an MCP server for a worktree
    pub async fn stop_server(&self, worktree_id: &str) -> Result<(), String> {
        let server = {
            let mut servers = self.servers.write().await;
            servers.remove(worktree_id)
        };

        if let Some(server) = server {
            server.cancel_token.cancel();
            // Wait for the server to shut down gracefully
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), server.handle).await;
            Ok(())
        } else {
            Err("No server running for this worktree".to_string())
        }
    }

    /// Get the port of a running MCP server (None if not running)
    pub async fn get_port(&self, worktree_id: &str) -> Option<u16> {
        let servers = self.servers.read().await;
        servers.get(worktree_id).map(|s| s.port)
    }

    /// Check if a server is running for a worktree
    pub async fn is_running(&self, worktree_id: &str) -> bool {
        let servers = self.servers.read().await;
        servers.contains_key(worktree_id)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_jsonrpc_request_parsing() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        }"#;

        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "tools/list");
        assert_eq!(request.jsonrpc, "2.0");
    }

    #[test]
    fn test_available_tools() {
        let tools = get_available_tools();
        assert_eq!(tools.len(), 7); // 4 base tools + 3 ReviewGate tools

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        // Base tools
        assert!(tool_names.contains(&"read_file"));
        assert!(tool_names.contains(&"list_directory"));
        assert!(tool_names.contains(&"get_project_context"));
        assert!(tool_names.contains(&"run_just_task"));
        // ReviewGate tools
        assert!(tool_names.contains(&"submit_for_review"));
        assert!(tool_names.contains(&"get_review_feedback"));
        assert!(tool_names.contains(&"update_review_content"));
    }

    #[tokio::test]
    async fn test_path_validation_valid() {
        let dir = tempdir().unwrap();
        let context = McpServerContext {
            worktree_root: dir.path().to_path_buf(),
            worktree_id: "test-worktree".to_string(),
            project_name: "test-project".to_string(),
        };

        // Create a test file
        let test_file = dir.path().join("test.txt");
        std::fs::write(&test_file, "hello").unwrap();

        // Valid path should work
        let result = context.validate_path("test.txt");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_path_validation_escape_attempt() {
        let dir = tempdir().unwrap();
        let context = McpServerContext {
            worktree_root: dir.path().to_path_buf(),
            worktree_id: "test-worktree".to_string(),
            project_name: "test-project".to_string(),
        };

        // Attempt to escape worktree should fail
        let result = context.validate_path("../../../etc/passwd");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_get_project_context() {
        let dir = tempdir().unwrap();
        let context = McpServerContext {
            worktree_root: dir.path().to_path_buf(),
            worktree_id: "test-worktree".to_string(),
            project_name: "test-project".to_string(),
        };

        let result = context
            .execute_tool("get_project_context", &serde_json::json!({}))
            .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        let content = result.get("content").unwrap().as_array().unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_mcp_server_manager_start_stop() {
        match TcpListener::bind("127.0.0.1:0").await {
            Ok(listener) => drop(listener),
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("Skipping: sandbox disallows binding to localhost");
                return;
            }
            Err(err) => panic!("Failed to probe TCP bind availability: {}", err),
        }

        let manager = McpServerManager::new();
        let dir = tempdir().unwrap();

        // Start server
        let result = manager
            .start_server(
                "test-worktree".to_string(),
                dir.path().to_path_buf(),
                "test-project".to_string(),
                Some(0),
            )
            .await;

        assert!(result.is_ok());
        let port = result.unwrap();
        assert!(port > 0);

        // Check it's running
        assert!(manager.is_running("test-worktree").await);

        // Stop server
        let stop_result = manager.stop_server("test-worktree").await;
        assert!(stop_result.is_ok());

        // Check it's stopped
        assert!(!manager.is_running("test-worktree").await);
    }

    // ========================================================================
    // ReviewGate Tool Tests
    // ========================================================================

    #[test]
    fn test_review_gate_tool_schemas() {
        let tools = get_available_tools();

        // Find submit_for_review tool
        let submit_tool = tools.iter().find(|t| t.name == "submit_for_review").unwrap();
        let schema = &submit_tool.input_schema;

        // Check required fields
        let required = schema.get("required").unwrap().as_array().unwrap();
        assert!(required.contains(&serde_json::json!("workflow_node_id")));
        assert!(required.contains(&serde_json::json!("content_type")));
        assert!(required.contains(&serde_json::json!("content")));

        // Find get_review_feedback tool
        let feedback_tool = tools
            .iter()
            .find(|t| t.name == "get_review_feedback")
            .unwrap();
        let schema = &feedback_tool.input_schema;
        let required = schema.get("required").unwrap().as_array().unwrap();
        assert!(required.contains(&serde_json::json!("session_id")));

        // Find update_review_content tool
        let update_tool = tools
            .iter()
            .find(|t| t.name == "update_review_content")
            .unwrap();
        let schema = &update_tool.input_schema;
        let required = schema.get("required").unwrap().as_array().unwrap();
        assert!(required.contains(&serde_json::json!("session_id")));
        assert!(required.contains(&serde_json::json!("content")));
    }
}
