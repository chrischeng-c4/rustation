//! Application state management for rstn.
//!
//! The AppState is the single source of truth for the application.
//! All state is JSON serializable for debugging and persistence.

use serde::{Deserialize, Serialize};

/// Main application state.
///
/// This is the single source of truth for the entire application.
/// The Frontend (React) receives updates via Tauri events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// Application version
    pub version: String,

    /// Currently active tab
    pub active_tab: Tab,

    /// Feature context (for MCP tools)
    pub feature_context: FeatureContext,

    /// MCP status (set by rstn_report_status tool)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_status: Option<McpStatusInfo>,

    /// Workflows tab state
    pub workflows: WorkflowsState,

    /// Dockers tab state
    pub dockers: DockersState,

    /// Settings state
    pub settings: SettingsState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            active_tab: Tab::Workflows,
            feature_context: FeatureContext::default(),
            mcp_status: None,
            workflows: WorkflowsState::default(),
            dockers: DockersState::default(),
            settings: SettingsState::default(),
        }
    }
}

/// Active tab in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tab {
    Workflows,
    Dockers,
    Settings,
}

/// Feature context for MCP tools
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureContext {
    pub feature_number: Option<String>,
    pub feature_name: Option<String>,
    pub branch: Option<String>,
    pub phase: Option<String>,
    pub spec_dir: Option<String>,
}

/// MCP status info (from rstn_report_status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStatusInfo {
    pub status: String,
    pub prompt: Option<String>,
    pub message: Option<String>,
}

// ============================================================================
// Tab-specific State
// ============================================================================

/// Workflows tab state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowsState {
    /// Available workflows
    pub workflows: Vec<WorkflowInfo>,

    /// Currently active workflow ID
    pub active_workflow: Option<String>,

    /// Chat messages for current session
    pub messages: Vec<ChatMessage>,

    /// Whether currently streaming a response
    pub is_streaming: bool,

    /// Current streaming text buffer
    pub streaming_buffer: String,
}

/// Workflow info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: String,
}

/// Message role
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Dockers tab state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DockersState {
    /// Docker services
    pub services: Vec<DockerService>,

    /// Selected service ID
    pub selected_service: Option<String>,

    /// Whether log panel is open
    pub log_panel_open: bool,
}

/// Docker service info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerService {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: ServiceStatus,
    pub port: Option<u16>,
}

/// Service status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Running,
    Stopped,
    Starting,
    Error,
}

/// Settings state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsState {
    /// Theme preference
    pub theme: Theme,

    /// Claude API key (masked in JSON)
    #[serde(skip_serializing)]
    pub api_key: Option<String>,

    /// Default project path
    pub default_project_path: Option<String>,
}

/// Theme preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_serialization_roundtrip() {
        let state = AppState::default();
        let json = serde_json::to_string(&state).unwrap();
        let loaded: AppState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.version, loaded.version);
        assert_eq!(state.active_tab, loaded.active_tab);
    }

    #[test]
    fn test_feature_context_serialization() {
        let context = FeatureContext {
            feature_number: Some("001".to_string()),
            feature_name: Some("test-feature".to_string()),
            branch: Some("001-test-feature".to_string()),
            phase: Some("implement".to_string()),
            spec_dir: Some("specs/001-test-feature".to_string()),
        };

        let json = serde_json::to_string_pretty(&context).unwrap();
        let loaded: FeatureContext = serde_json::from_str(&json).unwrap();

        assert_eq!(context.feature_number, loaded.feature_number);
        assert_eq!(context.spec_dir, loaded.spec_dir);
    }

    #[test]
    fn test_mcp_status_info() {
        let mut state = AppState::default();
        assert!(state.mcp_status.is_none());

        state.mcp_status = Some(McpStatusInfo {
            status: "needs_input".to_string(),
            prompt: Some("What is the feature name?".to_string()),
            message: None,
        });

        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("needs_input"));
    }
}
