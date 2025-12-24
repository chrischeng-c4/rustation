//! MCP protocol types for rstn.
//!
//! Defines JSON-RPC types, tool schemas, and response formats
//! for MCP (Model Context Protocol) communication.

use serde::{Deserialize, Serialize};

// ============================================================================
// JSON-RPC 2.0 Types
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
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<serde_json::Value>, code: i32, message: &str) -> Self {
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
    pub fn text(message: &str) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: message.to_string(),
            }],
            is_error: Some(false),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: message.to_string(),
            }],
            is_error: Some(true),
        }
    }

    pub fn error_with_suggestion(message: &str, suggestion: &str) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: format!("{} | Suggestion: {}", message, suggestion),
            }],
            is_error: Some(true),
        }
    }
}

// ============================================================================
// MCP Server Configuration
// ============================================================================

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
            port: 0,
            name: "rstn".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
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

/// Arguments for rstn_complete_task tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTaskArgs {
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_validation: Option<bool>,
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

// ============================================================================
// MCP State Types
// ============================================================================

/// MCP status types for rstn_report_status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpStatus {
    NeedsInput,
    Completed,
    Error,
}

impl std::fmt::Display for McpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpStatus::NeedsInput => write!(f, "needs_input"),
            McpStatus::Completed => write!(f, "completed"),
            McpStatus::Error => write!(f, "error"),
        }
    }
}

/// Spec artifact types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecArtifact {
    Spec,
    Plan,
    Tasks,
    Checklist,
    Analysis,
}

impl SpecArtifact {
    pub fn filename(&self) -> &'static str {
        match self {
            SpecArtifact::Spec => "spec.md",
            SpecArtifact::Plan => "plan.md",
            SpecArtifact::Tasks => "tasks.md",
            SpecArtifact::Checklist => "checklist.md",
            SpecArtifact::Analysis => "analysis.md",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "spec" => Some(SpecArtifact::Spec),
            "plan" => Some(SpecArtifact::Plan),
            "tasks" => Some(SpecArtifact::Tasks),
            "checklist" => Some(SpecArtifact::Checklist),
            "analysis" => Some(SpecArtifact::Analysis),
            _ => None,
        }
    }
}

// ============================================================================
// Tool Schemas
// ============================================================================

/// Get all available MCP tools
pub fn get_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "rstn_get_app_state".to_string(),
            description: Some("Get current application state as JSON".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "rstn_report_status".to_string(),
            description: Some(
                "Report current task status to rstn. For needs_input status, blocks until user provides input."
                    .to_string(),
            ),
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
